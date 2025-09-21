use crate::agent::{Task, Priority, TaskType};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Test setup utilities
pub struct TestSetup;

impl TestSetup {
    /// Create a test task
    pub fn create_test_task(id: &str, description: &str, priority: Priority) -> Task {
        Task::new(id.to_string(), description.to_string(), priority, TaskType::Development)
    }

    /// Create a test workspace
    pub async fn create_test_workspace(name: &str) -> String {
        format!("/tmp/test_workspace_{}", name)
    }

    /// Clean up test resources
    pub async fn cleanup_test_workspace(path: &str) {
        if std::path::Path::new(path).exists() {
            let _ = tokio::fs::remove_dir_all(path).await;
        }
    }
}

/// Test context for managing test state
pub struct TestContext {
    pub tasks: Arc<Mutex<Vec<Task>>>,
    pub workspace: String,
}

impl TestContext {
    pub fn new(workspace: String) -> Self {
        Self {
            tasks: Arc::new(Mutex::new(Vec::new())),
            workspace,
        }
    }

    pub async fn add_task(&self, task: Task) {
        let mut tasks = self.tasks.lock().await;
        tasks.push(task);
    }

    pub async fn get_task_count(&self) -> usize {
        let tasks = self.tasks.lock().await;
        tasks.len()
    }
}
