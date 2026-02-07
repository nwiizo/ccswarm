use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::agent::{Priority, Task, TaskResult};
use crate::utils::common::{errors, time};

/// Common trait for updating task status with timestamp
trait TaskStatusUpdate {
    fn update_status_with_timestamp(&mut self, status: TaskStatus);
}

impl TaskStatusUpdate for QueuedTask {
    fn update_status_with_timestamp(&mut self, status: TaskStatus) {
        self.status = status;
        self.updated_at = time::now();
    }
}

/// Status of a task in the queue
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    /// Task is waiting to be assigned
    Pending,
    /// Task has been assigned to an agent
    Assigned { agent_id: String },
    /// Task is currently being executed
    InProgress {
        agent_id: String,
        started_at: DateTime<Utc>,
    },
    /// Task completed successfully
    Completed {
        agent_id: String,
        completed_at: DateTime<Utc>,
        result: TaskResult,
    },
    /// Task failed with error
    Failed {
        agent_id: String,
        failed_at: DateTime<Utc>,
        error: String,
    },
    /// Task was cancelled
    Cancelled {
        cancelled_at: DateTime<Utc>,
        reason: Option<String>,
    },
}

/// Extended task information for queue management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedTask {
    pub task: Task,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub assigned_agent: Option<String>,
    pub execution_history: Vec<TaskExecutionAttempt>,
    pub metadata: HashMap<String, String>,
}

/// Record of task execution attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecutionAttempt {
    pub attempt_id: String,
    pub agent_id: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: Option<TaskResult>,
    pub error: Option<String>,
}

/// Task queue manager with priority handling
pub struct TaskQueue {
    /// Tasks organized by priority
    pending_tasks: Arc<RwLock<HashMap<Priority, VecDeque<QueuedTask>>>>,
    /// All tasks by ID for quick lookup
    tasks_by_id: Arc<RwLock<HashMap<String, QueuedTask>>>,
    /// Currently executing tasks
    active_tasks: Arc<RwLock<HashMap<String, QueuedTask>>>,
    /// Task execution history
    completed_tasks: Arc<RwLock<VecDeque<QueuedTask>>>,
    /// Maximum history size
    max_history: usize,
}

impl Default for TaskQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskQueue {
    pub fn new() -> Self {
        let mut pending_tasks = HashMap::new();
        pending_tasks.insert(Priority::Critical, VecDeque::new());
        pending_tasks.insert(Priority::High, VecDeque::new());
        pending_tasks.insert(Priority::Medium, VecDeque::new());
        pending_tasks.insert(Priority::Low, VecDeque::new());

        Self {
            pending_tasks: Arc::new(RwLock::new(pending_tasks)),
            tasks_by_id: Arc::new(RwLock::new(HashMap::new())),
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
            completed_tasks: Arc::new(RwLock::new(VecDeque::new())),
            max_history: 1000,
        }
    }

    /// Common helper for completing or failing tasks
    async fn finish_task(
        &self,
        task_id: &str,
        status: TaskStatus,
        update_attempt: impl FnOnce(&mut TaskExecutionAttempt),
    ) -> anyhow::Result<()> {
        let mut active = self.active_tasks.write().await;
        let mut completed = self.completed_tasks.write().await;

        if let Some(mut task) = active.remove(task_id) {
            // Validate task is in progress
            if !matches!(task.status, TaskStatus::InProgress { .. }) {
                return Err(errors::invalid_state_error(
                    "in_progress",
                    &format!("{:?}", task.status),
                ));
            }

            task.update_status_with_timestamp(status);

            // Update last execution attempt
            if let Some(attempt) = task.execution_history.last_mut() {
                attempt.completed_at = Some(time::now());
                update_attempt(attempt);
            }

            // Add to completed queue (maintain size limit)
            completed.push_back(task);
            if completed.len() > self.max_history {
                completed.pop_front();
            }

            Ok(())
        } else {
            Err(errors::not_found_error("Active task", task_id))
        }
    }

    /// Add a new task to the queue
    pub async fn add_task(&self, task: Task) -> String {
        let task_id = task.id.clone();
        let now = time::now();

        let queued_task = QueuedTask {
            task: task.clone(),
            status: TaskStatus::Pending,
            created_at: now,
            updated_at: now,
            assigned_agent: None,
            execution_history: Vec::new(),
            metadata: HashMap::new(),
        };

        // Add to priority queue
        {
            let mut pending = self.pending_tasks.write().await;
            if let Some(queue) = pending.get_mut(&task.priority) {
                queue.push_back(queued_task.clone());
            }
        }

        // Add to lookup table
        {
            let mut tasks = self.tasks_by_id.write().await;
            tasks.insert(task_id.clone(), queued_task);
        }

        task_id
    }

    /// Get next task for execution (highest priority first)
    pub async fn get_next_task(&self) -> Option<QueuedTask> {
        let mut pending = self.pending_tasks.write().await;

        // Check priorities in order: Critical, High, Medium, Low
        for priority in [
            Priority::Critical,
            Priority::High,
            Priority::Medium,
            Priority::Low,
        ] {
            if let Some(queue) = pending.get_mut(&priority)
                && let Some(task) = queue.pop_front()
            {
                return Some(task);
            }
        }
        None
    }

    /// Assign task to an agent
    pub async fn assign_task(&self, task_id: &str, agent_id: &str) -> anyhow::Result<()> {
        let mut tasks = self.tasks_by_id.write().await;

        if let Some(task) = tasks.get_mut(task_id) {
            task.status = TaskStatus::Assigned {
                agent_id: agent_id.to_string(),
            };
            task.assigned_agent = Some(agent_id.to_string());
            task.updated_at = time::now();
            Ok(())
        } else {
            Err(errors::not_found_error("Task", task_id))
        }
    }

    /// Mark task as in progress
    pub async fn start_task_execution(&self, task_id: &str, agent_id: &str) -> anyhow::Result<()> {
        let now = time::now();

        // Move from tasks_by_id to active_tasks
        let mut tasks = self.tasks_by_id.write().await;
        let mut active = self.active_tasks.write().await;

        if let Some(mut task) = tasks.remove(task_id) {
            task.status = TaskStatus::InProgress {
                agent_id: agent_id.to_string(),
                started_at: now,
            };
            task.updated_at = now;

            // Add execution attempt
            let attempt = TaskExecutionAttempt {
                attempt_id: Uuid::new_v4().to_string(),
                agent_id: agent_id.to_string(),
                started_at: now,
                completed_at: None,
                result: None,
                error: None,
            };
            task.execution_history.push(attempt);

            active.insert(task_id.to_string(), task);
            Ok(())
        } else {
            Err(errors::not_found_error("Task", task_id))
        }
    }

    /// Extract agent_id from active task (DRY helper)
    async fn extract_agent_id(&self, task_id: &str) -> anyhow::Result<String> {
        let active = self.active_tasks.read().await;
        match active.get(task_id) {
            Some(task) => match &task.status {
                TaskStatus::InProgress { agent_id, .. } => Ok(agent_id.clone()),
                _ => Err(errors::invalid_state_error(
                    "in_progress",
                    &format!("{:?}", task.status),
                )),
            },
            None => Err(errors::not_found_error("Active task", task_id)),
        }
    }

    /// Complete task execution
    pub async fn complete_task(&self, task_id: &str, result: TaskResult) -> anyhow::Result<()> {
        let now = time::now();
        let agent_id = self.extract_agent_id(task_id).await?;

        let status = TaskStatus::Completed {
            agent_id,
            completed_at: now,
            result: result.clone(),
        };

        self.finish_task(task_id, status, |attempt| {
            attempt.result = Some(result);
        })
        .await
    }

    /// Mark task as failed
    pub async fn fail_task(&self, task_id: &str, error: String) -> anyhow::Result<()> {
        let now = time::now();
        let agent_id = self.extract_agent_id(task_id).await?;

        let status = TaskStatus::Failed {
            agent_id,
            failed_at: now,
            error: error.clone(),
        };

        self.finish_task(task_id, status, |attempt| {
            attempt.error = Some(error);
        })
        .await
    }

    /// Cancel a task
    pub async fn cancel_task(&self, task_id: &str, reason: Option<String>) -> anyhow::Result<()> {
        let now = time::now();

        // Try to find task in pending, active, or lookup table
        let mut pending = self.pending_tasks.write().await;
        let mut active = self.active_tasks.write().await;
        let mut tasks = self.tasks_by_id.write().await;
        let mut completed = self.completed_tasks.write().await;

        // Check active tasks first
        if let Some(mut task) = active.remove(task_id) {
            task.status = TaskStatus::Cancelled {
                cancelled_at: now,
                reason,
            };
            task.updated_at = now;
            completed.push_back(task);
            return Ok(());
        }

        // Check pending tasks
        for (_, queue) in pending.iter_mut() {
            if let Some(pos) = queue.iter().position(|t| t.task.id == task_id) {
                // Safe to remove since we just found the position
                if let Some(mut task) = queue.remove(pos) {
                    task.status = TaskStatus::Cancelled {
                        cancelled_at: now,
                        reason,
                    };
                    task.updated_at = now;
                    completed.push_back(task);
                    tasks.remove(task_id);
                    return Ok(());
                }
            }
        }

        Err(anyhow::anyhow!("Task not found: {}", task_id))
    }

    /// Get task by ID
    pub async fn get_task(&self, task_id: &str) -> Option<QueuedTask> {
        // Check active tasks first
        {
            let active = self.active_tasks.read().await;
            if let Some(task) = active.get(task_id) {
                return Some(task.clone());
            }
        }

        // Check pending tasks
        {
            let tasks = self.tasks_by_id.read().await;
            if let Some(task) = tasks.get(task_id) {
                return Some(task.clone());
            }
        }

        // Check completed tasks
        {
            let completed = self.completed_tasks.read().await;
            for task in completed.iter() {
                if task.task.id == task_id {
                    return Some(task.clone());
                }
            }
        }

        None
    }

    /// List tasks with filters
    pub async fn list_tasks(
        &self,
        status_filter: Option<&str>,
        agent_filter: Option<&str>,
    ) -> Vec<QueuedTask> {
        let mut result = Vec::new();

        // Collect from all sources
        {
            let active = self.active_tasks.read().await;
            result.extend(active.values().cloned());
        }

        {
            let tasks = self.tasks_by_id.read().await;
            result.extend(tasks.values().cloned());
        }

        {
            let completed = self.completed_tasks.read().await;
            result.extend(completed.iter().cloned());
        }

        // Apply filters
        if let Some(status) = status_filter {
            result.retain(|task| {
                matches!(
                    (&task.status, status),
                    (TaskStatus::Pending, "pending")
                        | (TaskStatus::Assigned { .. }, "assigned")
                        | (TaskStatus::InProgress { .. }, "in_progress")
                        | (TaskStatus::Completed { .. }, "completed")
                        | (TaskStatus::Failed { .. }, "failed")
                        | (TaskStatus::Cancelled { .. }, "cancelled")
                )
            });
        }

        if let Some(agent) = agent_filter {
            result.retain(|task| task.assigned_agent.as_deref() == Some(agent));
        }

        // Sort by priority and creation time
        result.sort_by(|a, b| {
            a.task
                .priority
                .cmp(&b.task.priority)
                .then(a.created_at.cmp(&b.created_at))
        });

        result
    }

    /// Get queue statistics
    pub async fn get_stats(&self) -> TaskQueueStats {
        let pending_count = {
            let pending = self.pending_tasks.read().await;
            pending.values().map(|q| q.len()).sum()
        };

        let active_count = {
            let active = self.active_tasks.read().await;
            active.len()
        };

        let completed_count = {
            let completed = self.completed_tasks.read().await;
            completed.len()
        };

        let failed_count = {
            let completed = self.completed_tasks.read().await;
            completed
                .iter()
                .filter(|t| matches!(t.status, TaskStatus::Failed { .. }))
                .count()
        };

        TaskQueueStats {
            pending_count,
            active_count,
            completed_count,
            failed_count,
            total_count: pending_count + active_count + completed_count,
        }
    }
}

/// Task queue statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskQueueStats {
    pub pending_count: usize,
    pub active_count: usize,
    pub completed_count: usize,
    pub failed_count: usize,
    pub total_count: usize,
}
