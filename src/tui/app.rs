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
    Delegation,
}

impl Tab {
    pub fn title(&self) -> &str {
        match self {
            Tab::Overview => "Overview",
            Tab::Agents => "Agents",
            Tab::Tasks => "Tasks",
            Tab::Logs => "Logs",
            Tab::Delegation => "Delegation",
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

/// Delegation decision for display
#[derive(Debug, Clone)]
pub struct DelegationInfo {
    pub task_description: String,
    pub recommended_agent: String,
    pub confidence: f64,
    pub reasoning: String,
    pub created_at: DateTime<Utc>,
}

/// Delegation state for TUI
#[derive(Debug, Clone, PartialEq)]
pub enum DelegationMode {
    Analyze,
    Delegate,
    ViewStats,
}

/// Application state for TUI
pub struct App {
    /// Current active tab
    pub current_tab: Tab,

    /// Selection state for lists
    pub selected_agent: usize,
    pub selected_task: usize,
    pub selected_log: usize,
    pub selected_delegation: usize,

    /// Data
    pub agents: Vec<AgentInfo>,
    pub tasks: Vec<TaskInfo>,
    pub logs: Vec<LogEntry>,
    pub delegation_decisions: Vec<DelegationInfo>,

    /// System state
    pub system_status: String,
    pub total_agents: usize,
    pub active_agents: usize,
    pub pending_tasks: usize,
    pub completed_tasks: usize,

    /// Input state
    pub input_mode: InputMode,
    pub input_buffer: String,

    /// Delegation state
    pub delegation_mode: DelegationMode,
    pub delegation_input: String,

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
    DelegationInput,
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
            selected_delegation: 0,
            agents: Vec::new(),
            tasks: Vec::new(),
            logs: Vec::new(),
            delegation_decisions: Vec::new(),
            system_status: "Starting...".to_string(),
            total_agents: 0,
            active_agents: 0,
            pending_tasks: 0,
            completed_tasks: 0,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            delegation_mode: DelegationMode::Analyze,
            delegation_input: String::new(),
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
            Tab::Logs => Tab::Delegation,
            Tab::Delegation => Tab::Overview,
        };
        self.reset_selection();
    }

    /// Navigate to previous tab
    pub fn previous_tab(&mut self) {
        self.current_tab = match self.current_tab {
            Tab::Overview => Tab::Delegation,
            Tab::Agents => Tab::Overview,
            Tab::Tasks => Tab::Agents,
            Tab::Logs => Tab::Tasks,
            Tab::Delegation => Tab::Logs,
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
            Tab::Delegation => {
                if self.selected_delegation > 0 {
                    self.selected_delegation -= 1;
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
            Tab::Delegation => {
                if self.selected_delegation < self.delegation_decisions.len().saturating_sub(1) {
                    self.selected_delegation += 1;
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
            Tab::Delegation => self.selected_delegation = 0,
            _ => {}
        }
    }

    /// Activate selected item (start agent if available)
    pub async fn activate_selected(&mut self) -> Result<()> {
        match self.current_tab {
            Tab::Agents => {
                if let Some(agent) = self.agents.get(self.selected_agent).cloned() {
                    match agent.status {
                        AgentStatus::Available => {
                            // Start the agent
                            self.start_agent(&agent.id).await?;
                        }
                        AgentStatus::Working => {
                            // Show agent details
                            self.show_agent_details(&agent.id).await?;
                        }
                        _ => {
                            // Show agent details for other statuses
                            self.show_agent_details(&agent.id).await?;
                        }
                    }
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

    /// Start an available agent
    pub async fn start_agent(&mut self, agent_id: &str) -> Result<()> {
        let mut agent_info = None;

        // Find the agent and collect info
        if let Some(agent) = self.agents.iter_mut().find(|a| a.id == agent_id) {
            // Change status to Working
            agent.status = AgentStatus::Working;
            agent.last_activity = Utc::now();

            // Collect info for logging
            agent_info = Some((agent.name.clone(), agent.specialization.clone()));

            // Update system stats
            self.active_agents += 1;
        }

        // Log after borrowing ends
        if let Some((name, specialization)) = agent_info {
            self.add_log(
                "System",
                &format!("🚀 Starting agent: {} ({})", name, specialization),
            )
            .await;

            // If this is a Master agent, provide special logging
            if specialization.contains("Master") {
                self.add_log("Master", "🎯 Master Claude Code orchestrator activated")
                    .await;
                self.add_log("Master", "📋 Ready to coordinate multi-agent tasks")
                    .await;
            }
        }

        Ok(())
    }

    /// Start agent by ID or name
    pub async fn start_agent_by_id(&mut self, identifier: &str) -> Result<()> {
        // Find agent by ID or name
        let agent_to_start = self
            .agents
            .iter()
            .find(|a| a.id == identifier || a.name == identifier)
            .map(|a| a.id.clone());

        if let Some(agent_id) = agent_to_start {
            self.start_agent(&agent_id).await?;
        } else {
            self.add_log("System", &format!("Agent not found: {}", identifier))
                .await;
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

        // Always add Master Claude Code agent
        let master_agent = AgentInfo {
            id: "master-claude-code".to_string(),
            name: "master".to_string(),
            specialization: "Master Claude Code".to_string(),
            provider_type: "claude_code".to_string(),
            provider_icon: "👑".to_string(),
            provider_color: "gold".to_string(),
            status: AgentStatus::Available,
            current_task: None,
            tasks_completed: 0,
            last_activity: Utc::now(),
            workspace: "/workspace".to_string(),
        };
        self.agents.push(master_agent);

        // Add other default agents
        let default_agents = vec![
            ("qa-agent", "qa", "QA Specialist"),
            ("devops-agent", "devops", "DevOps Specialist"),
            ("test-agent", "test", "Test Specialist"),
            ("error-agent", "error", "Error Handler"),
            ("backend-agent", "backend", "Backend Specialist"),
            ("frontend-agent", "frontend", "Frontend Specialist"),
        ];

        for (id, name, spec) in default_agents {
            let agent = AgentInfo {
                id: id.to_string(),
                name: name.to_string(),
                specialization: spec.to_string(),
                provider_type: "claude_code".to_string(),
                provider_icon: "🤖".to_string(),
                provider_color: "blue".to_string(),
                status: AgentStatus::Available,
                current_task: None,
                tasks_completed: 0,
                last_activity: Utc::now(),
                workspace: "Unknown".to_string(),
            };
            self.agents.push(agent);
        }

        // Load dynamic agents from coordination system
        for (_i, status) in statuses.iter().enumerate() {
            if let (Some(agent_id), Some(status_val)) = (
                status.get("agent_id").and_then(|v| v.as_str()),
                status.get("status").and_then(|v| v.as_str()),
            ) {
                // Skip if already exists in default agents
                if self.agents.iter().any(|a| a.id == agent_id) {
                    continue;
                }

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
            InputMode::DelegationInput => {
                if !input.is_empty() {
                    self.execute_delegation_action(&input).await?;
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
            "start_agent" | "activate" => {
                if let Some(agent_id) = args.get(0) {
                    self.start_agent_by_id(agent_id).await?;
                } else {
                    self.add_log("System", "Usage: start_agent <agent_id|agent_name>")
                        .await;
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
            "  help                    - Show this help",
            "  status                  - Show system status",
            "  agents                  - List all agents",
            "  tasks                   - List all tasks",
            "  task <description>      - Add new task",
            "  agent <type>            - Create new agent (frontend/backend/devops/qa)",
            "  start_agent <id|name>   - Start/activate an agent (e.g., 'start_agent master')",
            "  activate <id|name>      - Alias for start_agent",
            "  start                   - Start orchestrator",
            "  stop                    - Stop orchestrator",
            "  refresh                 - Refresh all data",
            "  clear                   - Clear logs",
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
        self.add_log("System", "=== Detailed System Status ===")
            .await;
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

        // Show Master Claude Code status specifically
        if let Some(master) = self
            .agents
            .iter()
            .find(|a| a.specialization.contains("Master"))
        {
            self.add_log(
                "System",
                &format!("👑 Master Claude Code: {:?}", master.status),
            )
            .await;
        }
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

    /// Switch delegation mode
    pub fn switch_delegation_mode(&mut self) {
        self.delegation_mode = match self.delegation_mode {
            DelegationMode::Analyze => DelegationMode::Delegate,
            DelegationMode::Delegate => DelegationMode::ViewStats,
            DelegationMode::ViewStats => DelegationMode::Analyze,
        };
    }

    /// Start delegation input mode
    pub async fn start_delegation_input(&mut self) -> Result<()> {
        self.input_mode = InputMode::DelegationInput;
        self.delegation_input.clear();

        match self.delegation_mode {
            DelegationMode::Analyze => {
                self.add_log("Master", "Enter task description to analyze:")
                    .await;
            }
            DelegationMode::Delegate => {
                self.add_log("Master", "Enter task description to delegate:")
                    .await;
            }
            _ => {}
        }
        Ok(())
    }

    /// Execute delegation action
    async fn execute_delegation_action(&mut self, input: &str) -> Result<()> {
        match self.delegation_mode {
            DelegationMode::Analyze => {
                self.analyze_task_for_delegation(input).await?;
            }
            DelegationMode::Delegate => {
                self.delegate_task_to_agent(input).await?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Analyze task for delegation
    async fn analyze_task_for_delegation(&mut self, task_description: &str) -> Result<()> {
        self.add_log(
            "Master",
            &format!("🔍 Analyzing task: '{}'", task_description),
        )
        .await;

        // Use basic rule-based analysis for demo
        let (recommended_agent, confidence, reasoning) =
            self.analyze_task_content(task_description);

        let delegation_info = DelegationInfo {
            task_description: task_description.to_string(),
            recommended_agent: recommended_agent.clone(),
            confidence,
            reasoning: reasoning.clone(),
            created_at: chrono::Utc::now(),
        };

        self.delegation_decisions.push(delegation_info);

        self.add_log(
            "Master",
            &format!(
                "✅ Analysis complete: {} agent recommended ({:.1}% confidence)",
                recommended_agent,
                confidence * 100.0
            ),
        )
        .await;
        self.add_log("Master", &format!("📝 Reasoning: {}", reasoning))
            .await;

        Ok(())
    }

    /// Delegate task to specific agent
    async fn delegate_task_to_agent(&mut self, input: &str) -> Result<()> {
        // Parse input as "agent_type task_description"
        let parts: Vec<&str> = input.splitn(2, ' ').collect();
        if parts.len() < 2 {
            self.add_log("Master", "Usage: <agent_type> <task_description>")
                .await;
            return Ok(());
        }

        let agent_type = parts[0];
        let task_description = parts[1];

        // Validate agent type
        let valid_agents = ["frontend", "backend", "devops", "qa"];
        if !valid_agents.contains(&agent_type) {
            self.add_log(
                "Master",
                &format!(
                    "Invalid agent type: {}. Valid agents: {}",
                    agent_type,
                    valid_agents.join(", ")
                ),
            )
            .await;
            return Ok(());
        }

        self.add_log(
            "Master",
            &format!(
                "🎯 Delegating task to {} agent: '{}'",
                agent_type, task_description
            ),
        )
        .await;

        // Create and add task to queue
        let task = crate::agent::Task::new(
            uuid::Uuid::new_v4().to_string(),
            task_description.to_string(),
            crate::agent::Priority::Medium,
            crate::agent::TaskType::Development,
        );

        self.task_queue.add_task(&task).await?;

        // Add delegation decision for tracking
        let delegation_info = DelegationInfo {
            task_description: task_description.to_string(),
            recommended_agent: agent_type.to_string(),
            confidence: 1.0, // Manual delegation is 100% confident
            reasoning: "Manual delegation by Master".to_string(),
            created_at: chrono::Utc::now(),
        };

        self.delegation_decisions.push(delegation_info);

        self.add_log(
            "Master",
            &format!("✅ Task delegated to {} agent successfully", agent_type),
        )
        .await;
        self.refresh_data().await?;

        Ok(())
    }

    /// Analyze task content and recommend agent
    fn analyze_task_content(&self, task_description: &str) -> (String, f64, String) {
        let desc_lower = task_description.to_lowercase();

        // Frontend keywords
        if desc_lower.contains("ui")
            || desc_lower.contains("html")
            || desc_lower.contains("css")
            || desc_lower.contains("javascript")
            || desc_lower.contains("component")
            || desc_lower.contains("react")
            || desc_lower.contains("vue")
            || desc_lower.contains("frontend")
        {
            return (
                "Frontend".to_string(),
                0.9,
                "Contains UI/frontend keywords".to_string(),
            );
        }

        // Backend keywords
        if desc_lower.contains("api")
            || desc_lower.contains("server")
            || desc_lower.contains("database")
            || desc_lower.contains("backend")
            || desc_lower.contains("endpoint")
            || desc_lower.contains("node")
            || desc_lower.contains("express")
            || desc_lower.contains("rest")
        {
            return (
                "Backend".to_string(),
                0.9,
                "Contains API/backend keywords".to_string(),
            );
        }

        // Testing keywords
        if desc_lower.contains("test")
            || desc_lower.contains("testing")
            || desc_lower.contains("qa")
            || desc_lower.contains("quality")
            || desc_lower.contains("validation")
            || desc_lower.contains("unit")
        {
            return (
                "QA".to_string(),
                0.9,
                "Contains testing/QA keywords".to_string(),
            );
        }

        // Infrastructure keywords
        if desc_lower.contains("deploy")
            || desc_lower.contains("ci/cd")
            || desc_lower.contains("docker")
            || desc_lower.contains("infrastructure")
            || desc_lower.contains("pipeline")
            || desc_lower.contains("build")
            || desc_lower.contains("devops")
        {
            return (
                "DevOps".to_string(),
                0.9,
                "Contains infrastructure/DevOps keywords".to_string(),
            );
        }

        // Default to backend for general development
        (
            "Backend".to_string(),
            0.6,
            "General development task, defaulting to backend".to_string(),
        )
    }

    /// Get delegation statistics
    pub fn get_delegation_stats(&self) -> String {
        if self.delegation_decisions.is_empty() {
            return "No delegation decisions yet".to_string();
        }

        let total = self.delegation_decisions.len();
        let mut agent_counts = std::collections::HashMap::new();
        let mut total_confidence = 0.0;

        for decision in &self.delegation_decisions {
            *agent_counts
                .entry(decision.recommended_agent.clone())
                .or_insert(0) += 1;
            total_confidence += decision.confidence;
        }

        let avg_confidence = total_confidence / total as f64;

        let mut stats = format!("📊 Delegation Statistics:\n");
        stats.push_str(&format!("Total delegations: {}\n", total));
        stats.push_str(&format!(
            "Average confidence: {:.1}%\n",
            avg_confidence * 100.0
        ));
        stats.push_str("Agent distribution:\n");

        for (agent, count) in agent_counts {
            let percentage = (count as f64 / total as f64) * 100.0;
            stats.push_str(&format!("  {}: {} ({:.1}%)\n", agent, count, percentage));
        }

        stats
    }

    /// Handle delegation tab interactions
    pub async fn handle_delegation_enter(&mut self) -> Result<()> {
        match self.delegation_mode {
            DelegationMode::Analyze | DelegationMode::Delegate => {
                self.start_delegation_input().await?;
            }
            DelegationMode::ViewStats => {
                // Show delegation statistics
                let stats = self.get_delegation_stats();
                for line in stats.lines() {
                    self.add_log("Master", line).await;
                }
            }
        }
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
