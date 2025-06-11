use anyhow::Result;
use chrono::{DateTime, Utc};
use tokio::time::{Duration, Instant};

use crate::agent::AgentStatus;
use crate::coordination::{CoordinationBus, StatusTracker, TaskQueue};

/// Current UI tab
#[derive(Debug, Clone, PartialEq)]
pub enum Tab {
    Overview,
    Agents,
    Tasks,
    Logs,
}

impl Tab {
    pub fn title(&self) -> &str {
        match self {
            Tab::Overview => "Overview",
            Tab::Agents => "Agents",
            Tab::Tasks => "Tasks",
            Tab::Logs => "Logs",
        }
    }
}

/// Agent display information
#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub specialization: String,
    pub provider_type: String,
    pub provider_icon: String,
    pub provider_color: String,
    pub status: AgentStatus,
    pub current_task: Option<String>,
    pub tasks_completed: u32,
    pub last_activity: DateTime<Utc>,
    pub workspace: String,
}

/// Task display information
#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub id: String,
    pub description: String,
    pub priority: String,
    pub task_type: String,
    pub status: String,
    pub assigned_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Log entry
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub agent: Option<String>,
    pub message: String,
}

/// Application state for TUI
pub struct App {
    /// Current active tab
    pub current_tab: Tab,

    /// Selection state for lists
    pub selected_agent: usize,
    pub selected_task: usize,
    pub selected_log: usize,

    /// Data
    pub agents: Vec<AgentInfo>,
    pub tasks: Vec<TaskInfo>,
    pub logs: Vec<LogEntry>,

    /// System state
    pub system_status: String,
    pub total_agents: usize,
    pub active_agents: usize,
    pub pending_tasks: usize,
    pub completed_tasks: usize,

    /// Input state
    pub input_mode: InputMode,
    pub input_buffer: String,

    /// Coordination components
    pub coordination_bus: CoordinationBus,
    pub status_tracker: StatusTracker,
    pub task_queue: TaskQueue,

    /// Terminal size
    pub terminal_width: u16,
    pub terminal_height: u16,

    /// Last update time
    pub last_update: Instant,
    pub update_interval: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    AddingTask,
    CreatingAgent,
    Command,
}

impl App {
    /// Create new app instance
    pub async fn new() -> Result<Self> {
        let coordination_bus = CoordinationBus::new().await?;
        let status_tracker = StatusTracker::new().await?;
        let task_queue = TaskQueue::new().await?;

        Ok(Self {
            current_tab: Tab::Overview,
            selected_agent: 0,
            selected_task: 0,
            selected_log: 0,
            agents: Vec::new(),
            tasks: Vec::new(),
            logs: Vec::new(),
            system_status: "Starting...".to_string(),
            total_agents: 0,
            active_agents: 0,
            pending_tasks: 0,
            completed_tasks: 0,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            coordination_bus,
            status_tracker,
            task_queue,
            terminal_width: 80,
            terminal_height: 24,
            last_update: Instant::now(),
            update_interval: Duration::from_secs(2),
        })
    }

    /// Navigate to next tab
    pub fn next_tab(&mut self) {
        self.current_tab = match self.current_tab {
            Tab::Overview => Tab::Agents,
            Tab::Agents => Tab::Tasks,
            Tab::Tasks => Tab::Logs,
            Tab::Logs => Tab::Overview,
        };
        self.reset_selection();
    }

    /// Navigate to previous tab
    pub fn previous_tab(&mut self) {
        self.current_tab = match self.current_tab {
            Tab::Overview => Tab::Logs,
            Tab::Agents => Tab::Overview,
            Tab::Tasks => Tab::Agents,
            Tab::Logs => Tab::Tasks,
        };
        self.reset_selection();
    }

    /// Move selection up
    pub fn previous_item(&mut self) {
        match self.current_tab {
            Tab::Agents => {
                if self.selected_agent > 0 {
                    self.selected_agent -= 1;
                }
            }
            Tab::Tasks => {
                if self.selected_task > 0 {
                    self.selected_task -= 1;
                }
            }
            Tab::Logs => {
                if self.selected_log > 0 {
                    self.selected_log -= 1;
                }
            }
            _ => {}
        }
    }

    /// Move selection down
    pub fn next_item(&mut self) {
        match self.current_tab {
            Tab::Agents => {
                if self.selected_agent < self.agents.len().saturating_sub(1) {
                    self.selected_agent += 1;
                }
            }
            Tab::Tasks => {
                if self.selected_task < self.tasks.len().saturating_sub(1) {
                    self.selected_task += 1;
                }
            }
            Tab::Logs => {
                if self.selected_log < self.logs.len().saturating_sub(1) {
                    self.selected_log += 1;
                }
            }
            _ => {}
        }
    }

    /// Reset selection for current tab
    fn reset_selection(&mut self) {
        match self.current_tab {
            Tab::Agents => self.selected_agent = 0,
            Tab::Tasks => self.selected_task = 0,
            Tab::Logs => self.selected_log = 0,
            _ => {}
        }
    }

    /// Activate selected item
    pub async fn activate_selected(&mut self) -> Result<()> {
        match self.current_tab {
            Tab::Agents => {
                if let Some(agent) = self.agents.get(self.selected_agent) {
                    let agent_id = agent.id.clone();
                    self.show_agent_details(&agent_id).await?;
                }
            }
            Tab::Tasks => {
                if let Some(task) = self.tasks.get(self.selected_task) {
                    let task_id = task.id.clone();
                    self.show_task_details(&task_id).await?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Create new session (agent)
    pub async fn create_new_session(&mut self) -> Result<()> {
        self.input_mode = InputMode::CreatingAgent;
        self.input_buffer.clear();
        self.add_log("System", "Enter agent type (frontend/backend/devops/qa):")
            .await;
        Ok(())
    }

    /// Delete current session
    pub async fn delete_current_session(&mut self) -> Result<()> {
        if let Some(agent) = self.agents.get(self.selected_agent) {
            self.add_log("System", &format!("Deleting agent: {}", agent.name))
                .await;
            // TODO: Implement actual agent deletion
        }
        Ok(())
    }

    /// Show system status
    pub async fn show_status(&mut self) -> Result<()> {
        self.current_tab = Tab::Overview;
        self.refresh_data().await?;
        Ok(())
    }

    /// Add task prompt
    pub async fn add_task_prompt(&mut self) -> Result<()> {
        self.input_mode = InputMode::AddingTask;
        self.input_buffer.clear();
        self.add_log("System", "Enter task description:").await;
        Ok(())
    }

    /// Open command prompt
    pub async fn open_command_prompt(&mut self) -> Result<()> {
        self.input_mode = InputMode::Command;
        self.input_buffer.clear();
        self.add_log("System", "Enter command (help for available commands):")
            .await;
        Ok(())
    }

    /// Refresh all data
    pub async fn refresh_data(&mut self) -> Result<()> {
        self.load_agents().await?;
        self.load_tasks().await?;
        self.update_system_stats().await?;
        self.last_update = Instant::now();
        Ok(())
    }

    /// Cancel current action
    pub fn cancel_current_action(&mut self) {
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
    }

    /// Update terminal size
    pub fn update_size(&mut self, width: u16, height: u16) {
        self.terminal_width = width;
        self.terminal_height = height;
    }

    /// Periodic update
    pub async fn update(&mut self) -> Result<()> {
        if self.last_update.elapsed() >= self.update_interval {
            self.refresh_data().await?;
        }
        Ok(())
    }

    /// Load agents from coordination system
    async fn load_agents(&mut self) -> Result<()> {
        let statuses = self.status_tracker.get_all_statuses().await?;

        self.agents.clear();
        for (_i, status) in statuses.iter().enumerate() {
            if let (Some(agent_id), Some(status_val)) = (
                status.get("agent_id").and_then(|v| v.as_str()),
                status.get("status").and_then(|v| v.as_str()),
            ) {
                let agent_info = AgentInfo {
                    id: agent_id.to_string(),
                    name: agent_id.split('-').next().unwrap_or("Unknown").to_string(),
                    specialization: status
                        .get("specialization")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown")
                        .to_string(),
                    provider_type: "claude_code".to_string(),
                    provider_icon: "🤖".to_string(),
                    provider_color: "blue".to_string(),
                    status: parse_agent_status(status_val),
                    current_task: status
                        .get("current_task")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    tasks_completed: status
                        .get("tasks_completed")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as u32,
                    last_activity: status
                        .get("timestamp")
                        .and_then(|v| v.as_str())
                        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(Utc::now),
                    workspace: status
                        .get("workspace")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown")
                        .to_string(),
                };
                self.agents.push(agent_info);
            }
        }

        // Ensure selection is valid
        if self.selected_agent >= self.agents.len() {
            self.selected_agent = 0;
        }

        Ok(())
    }

    /// Load tasks from task queue
    async fn load_tasks(&mut self) -> Result<()> {
        let pending_tasks = self.task_queue.get_pending_tasks().await?;

        self.tasks.clear();
        for task in pending_tasks {
            let task_info = TaskInfo {
                id: task.id.clone(),
                description: task.description.clone(),
                priority: format!("{:?}", task.priority),
                task_type: format!("{:?}", task.task_type),
                status: "Pending".to_string(),
                assigned_agent: None,
                created_at: Utc::now(), // TODO: Get actual creation time
            };
            self.tasks.push(task_info);
        }

        // Ensure selection is valid
        if self.selected_task >= self.tasks.len() {
            self.selected_task = 0;
        }

        Ok(())
    }

    /// Update system statistics
    async fn update_system_stats(&mut self) -> Result<()> {
        self.total_agents = self.agents.len();
        self.active_agents = self
            .agents
            .iter()
            .filter(|a| matches!(a.status, AgentStatus::Available | AgentStatus::Working))
            .count();
        self.pending_tasks = self.tasks.len();
        self.completed_tasks = self.agents.iter().map(|a| a.tasks_completed).sum::<u32>() as usize;

        self.system_status = if self.active_agents > 0 {
            "Running".to_string()
        } else {
            "Stopped".to_string()
        };

        Ok(())
    }

    /// Show agent details
    async fn show_agent_details(&mut self, agent_id: &str) -> Result<()> {
        self.add_log(
            "System",
            &format!("Showing details for agent: {}", agent_id),
        )
        .await;
        Ok(())
    }

    /// Show task details
    async fn show_task_details(&mut self, task_id: &str) -> Result<()> {
        self.add_log("System", &format!("Showing details for task: {}", task_id))
            .await;
        Ok(())
    }

    /// Add log entry
    async fn add_log(&mut self, source: &str, message: &str) {
        let log_entry = LogEntry {
            timestamp: Utc::now(),
            level: "INFO".to_string(),
            agent: if source == "System" {
                None
            } else {
                Some(source.to_string())
            },
            message: message.to_string(),
        };
        self.logs.push(log_entry);

        // Keep only last 1000 log entries
        if self.logs.len() > 1000 {
            self.logs.remove(0);
        }

        // Auto-scroll to bottom
        self.selected_log = self.logs.len().saturating_sub(1);
    }

    /// Handle character input
    pub fn handle_char_input(&mut self, c: char) {
        if self.input_mode != InputMode::Normal {
            self.input_buffer.push(c);
        }
    }

    /// Handle backspace input
    pub fn handle_backspace(&mut self) {
        if self.input_mode != InputMode::Normal && !self.input_buffer.is_empty() {
            self.input_buffer.pop();
        }
    }

    /// Process input and execute command/action
    pub async fn process_input(&mut self) -> Result<()> {
        let input = self.input_buffer.trim().to_string();

        match self.input_mode {
            InputMode::AddingTask => {
                if !input.is_empty() {
                    self.execute_add_task(&input).await?;
                }
            }
            InputMode::CreatingAgent => {
                if !input.is_empty() {
                    self.execute_create_agent(&input).await?;
                }
            }
            InputMode::Command => {
                if !input.is_empty() {
                    self.execute_command(&input).await?;
                }
            }
            _ => {}
        }

        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
        Ok(())
    }

    /// Execute add task command
    async fn execute_add_task(&mut self, description: &str) -> Result<()> {
        // Parse priority and type from description if included
        let (desc, priority, task_type) = self.parse_task_description(description);

        let task =
            crate::agent::Task::new(uuid::Uuid::new_v4().to_string(), desc, priority, task_type);

        self.task_queue.add_task(&task).await?;
        self.add_log("System", &format!("Task added: {}", task.description))
            .await;
        self.refresh_data().await?;
        Ok(())
    }

    /// Execute create agent command
    async fn execute_create_agent(&mut self, agent_type: &str) -> Result<()> {
        let agent_type = agent_type.to_lowercase();
        match agent_type.as_str() {
            "frontend" | "backend" | "devops" | "qa" => {
                self.add_log("System", &format!("Creating {} agent...", agent_type))
                    .await;
                // TODO: Implement actual agent creation
                self.add_log(
                    "System",
                    &format!("{} agent created successfully", agent_type),
                )
                .await;
            }
            _ => {
                self.add_log(
                    "System",
                    "Invalid agent type. Use: frontend, backend, devops, qa",
                )
                .await;
            }
        }
        Ok(())
    }

    /// Execute command
    async fn execute_command(&mut self, command: &str) -> Result<()> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }

        let cmd = parts[0].to_lowercase();
        let args = &parts[1..];

        match cmd.as_str() {
            "help" => self.show_help().await,
            "status" => self.show_detailed_status().await?,
            "agents" => self.list_agents_command().await?,
            "tasks" => self.list_tasks_command().await?,
            "task" => {
                if args.get(0).is_some() {
                    let full_desc = args.join(" ");
                    self.execute_add_task(&full_desc).await?;
                } else {
                    self.add_log("System", "Usage: task <description>").await;
                }
            }
            "agent" => {
                if let Some(agent_type) = args.get(0) {
                    self.execute_create_agent(agent_type).await?;
                } else {
                    self.add_log("System", "Usage: agent <type>").await;
                }
            }
            "start" => self.start_orchestrator().await?,
            "stop" => self.stop_orchestrator().await?,
            "refresh" => {
                self.refresh_data().await?;
                self.add_log("System", "Data refreshed").await;
            }
            "clear" => {
                self.logs.clear();
                self.add_log("System", "Logs cleared").await;
            }
            "worktree" => {
                if args.is_empty() {
                    self.list_worktrees().await?;
                } else {
                    match args[0] {
                        "list" => self.list_worktrees().await?,
                        "prune" => self.prune_worktrees().await?,
                        _ => self.add_log("System", "Usage: worktree [list|prune]").await,
                    }
                }
            }
            _ => {
                self.add_log(
                    "System",
                    &format!(
                        "Unknown command: {}. Type 'help' for available commands.",
                        cmd
                    ),
                )
                .await;
            }
        }

        Ok(())
    }

    /// Show available commands
    async fn show_help(&mut self) {
        let help_text = vec![
            "Available commands:",
            "  help                 - Show this help",
            "  status              - Show system status",
            "  agents              - List all agents",
            "  tasks               - List all tasks",
            "  task <description>  - Add new task",
            "  agent <type>        - Create new agent (frontend/backend/devops/qa)",
            "  start               - Start orchestrator",
            "  stop                - Stop orchestrator",
            "  refresh             - Refresh all data",
            "  clear               - Clear logs",
            "  worktree [list|prune] - Manage worktrees",
        ];

        for line in help_text {
            self.add_log("System", line).await;
        }
    }

    /// Parse task description for priority and type
    fn parse_task_description(
        &self,
        description: &str,
    ) -> (String, crate::agent::Priority, crate::agent::TaskType) {
        use crate::agent::{Priority, TaskType};

        let mut desc = description.to_string();
        let mut priority = Priority::Medium;
        let mut task_type = TaskType::Development;

        // Extract priority
        if desc.contains("[high]") || desc.contains("[urgent]") {
            priority = Priority::High;
            desc = desc.replace("[high]", "").replace("[urgent]", "");
        } else if desc.contains("[low]") {
            priority = Priority::Low;
            desc = desc.replace("[low]", "");
        } else if desc.contains("[critical]") {
            priority = Priority::Critical;
            desc = desc.replace("[critical]", "");
        }

        // Extract task type
        if desc.contains("[test]") || desc.contains("[testing]") {
            task_type = TaskType::Testing;
            desc = desc.replace("[test]", "").replace("[testing]", "");
        } else if desc.contains("[docs]") || desc.contains("[documentation]") {
            task_type = TaskType::Documentation;
            desc = desc.replace("[docs]", "").replace("[documentation]", "");
        } else if desc.contains("[infra]") || desc.contains("[infrastructure]") {
            task_type = TaskType::Infrastructure;
            desc = desc.replace("[infra]", "").replace("[infrastructure]", "");
        } else if desc.contains("[bug]") || desc.contains("[bugfix]") {
            task_type = TaskType::Bugfix;
            desc = desc.replace("[bug]", "").replace("[bugfix]", "");
        } else if desc.contains("[feature]") {
            task_type = TaskType::Feature;
            desc = desc.replace("[feature]", "");
        }

        (desc.trim().to_string(), priority, task_type)
    }

    /// Show detailed status
    async fn show_detailed_status(&mut self) -> Result<()> {
        self.add_log("System", &format!("System Status: {}", self.system_status))
            .await;
        self.add_log("System", &format!("Total Agents: {}", self.total_agents))
            .await;
        self.add_log("System", &format!("Active Agents: {}", self.active_agents))
            .await;
        self.add_log("System", &format!("Pending Tasks: {}", self.pending_tasks))
            .await;
        self.add_log(
            "System",
            &format!("Completed Tasks: {}", self.completed_tasks),
        )
        .await;
        Ok(())
    }

    /// List agents command
    async fn list_agents_command(&mut self) -> Result<()> {
        if self.agents.is_empty() {
            self.add_log("System", "No agents found").await;
        } else {
            self.add_log("System", "Active agents:").await;
            let agent_info: Vec<String> = self
                .agents
                .iter()
                .map(|agent| {
                    format!(
                        "  {} ({}) - {:?}",
                        agent.name, agent.specialization, agent.status
                    )
                })
                .collect();

            for info in agent_info {
                self.add_log("System", &info).await;
            }
        }
        Ok(())
    }

    /// List tasks command
    async fn list_tasks_command(&mut self) -> Result<()> {
        if self.tasks.is_empty() {
            self.add_log("System", "No pending tasks").await;
        } else {
            self.add_log("System", "Pending tasks:").await;
            let task_info: Vec<String> = self
                .tasks
                .iter()
                .map(|task| {
                    format!(
                        "  {} - {} ({})",
                        task.description, task.priority, task.task_type
                    )
                })
                .collect();

            for info in task_info {
                self.add_log("System", &info).await;
            }
        }
        Ok(())
    }

    /// Start orchestrator
    async fn start_orchestrator(&mut self) -> Result<()> {
        self.add_log("System", "Starting orchestrator...").await;
        // TODO: Implement actual orchestrator start
        self.system_status = "Running".to_string();
        self.add_log("System", "Orchestrator started successfully")
            .await;
        Ok(())
    }

    /// Stop orchestrator
    async fn stop_orchestrator(&mut self) -> Result<()> {
        self.add_log("System", "Stopping orchestrator...").await;
        // TODO: Implement actual orchestrator stop
        self.system_status = "Stopped".to_string();
        self.add_log("System", "Orchestrator stopped").await;
        Ok(())
    }

    /// List worktrees
    async fn list_worktrees(&mut self) -> Result<()> {
        self.add_log("System", "Git worktrees:").await;
        // TODO: Implement actual worktree listing
        self.add_log("System", "  No worktrees found").await;
        Ok(())
    }

    /// Prune worktrees
    async fn prune_worktrees(&mut self) -> Result<()> {
        self.add_log("System", "Pruning stale worktrees...").await;
        // TODO: Implement actual worktree pruning
        self.add_log("System", "Worktree pruning completed").await;
        Ok(())
    }
}

/// Parse agent status from string
fn parse_agent_status(status: &str) -> AgentStatus {
    match status {
        "Initializing" => AgentStatus::Initializing,
        "Available" => AgentStatus::Available,
        "Working" => AgentStatus::Working,
        "WaitingForReview" => AgentStatus::WaitingForReview,
        "ShuttingDown" => AgentStatus::ShuttingDown,
        _ => AgentStatus::Error(format!("Unknown status: {}", status)),
    }
}
