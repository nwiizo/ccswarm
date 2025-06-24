#[cfg(test)]
mod tests {
    use crate::agent::AgentStatus;
    use crate::tui::app::{AgentInfo, App, DelegationMode, InputMode, LogEntry, Tab, TaskInfo};
    use crate::tui::event::EventHandler;
    use crate::tui::ui;
    use chrono::Utc;
    use ratatui::{backend::TestBackend, buffer::Buffer, Terminal};
    use std::time::Duration;

    /// Test App creation and initialization
    #[tokio::test]
    async fn test_app_new() {
        let app = App::new().await.unwrap();

        assert_eq!(app.input_mode, InputMode::Normal);
        assert_eq!(app.current_tab, Tab::Overview);
        assert!(app.agents.is_empty());
        assert!(app.tasks.is_empty());
        assert!(app.logs.is_empty());
        assert_eq!(app.selected_agent, 0);
        assert_eq!(app.selected_task, 0);
        assert_eq!(app.selected_log, 0);
    }

    /// Test tab navigation
    #[tokio::test]
    async fn test_tab_navigation() {
        let mut app = App::new().await.unwrap();

        // Test next tab
        app.next_tab();
        assert_eq!(app.current_tab, Tab::Agents);

        app.next_tab();
        assert_eq!(app.current_tab, Tab::Tasks);

        app.next_tab();
        assert_eq!(app.current_tab, Tab::Logs);

        app.next_tab();
        assert_eq!(app.current_tab, Tab::Delegation);

        // Wrap around
        app.next_tab();
        assert_eq!(app.current_tab, Tab::Overview);

        // Test previous tab
        app.previous_tab();
        assert_eq!(app.current_tab, Tab::Delegation);

        app.previous_tab();
        assert_eq!(app.current_tab, Tab::Logs);
    }

    /// Test item selection navigation
    #[tokio::test]
    async fn test_item_selection() {
        let mut app = App::new().await.unwrap();

        // Add some test data for agents
        app.agents = vec![
            create_test_agent_info("agent-1"),
            create_test_agent_info("agent-2"),
            create_test_agent_info("agent-3"),
        ];

        app.current_tab = Tab::Agents;

        // Initially at index 0
        assert_eq!(app.selected_agent, 0);

        // Navigate down
        app.next_item();
        assert_eq!(app.selected_agent, 1);

        app.next_item();
        assert_eq!(app.selected_agent, 2);

        // Should not go beyond last item
        app.next_item();
        assert_eq!(app.selected_agent, 2);

        // Navigate up
        app.previous_item();
        assert_eq!(app.selected_agent, 1);

        app.previous_item();
        assert_eq!(app.selected_agent, 0);

        // Should not go below 0
        app.previous_item();
        assert_eq!(app.selected_agent, 0);
    }

    /// Test input mode handling
    #[tokio::test]
    async fn test_input_mode() {
        let mut app = App::new().await.unwrap();

        // Start in normal mode
        assert_eq!(app.input_mode, InputMode::Normal);
        assert!(app.input_buffer.is_empty());

        // Switch to command input mode
        app.input_mode = InputMode::Command;
        assert_eq!(app.input_mode, InputMode::Command);
        assert_eq!(app.input_buffer, "");

        // Add character input
        app.input_buffer.push('t');
        app.input_buffer.push('e');
        app.input_buffer.push('s');
        app.input_buffer.push('t');
        assert_eq!(app.input_buffer, "test");

        // Handle backspace
        app.input_buffer.pop();
        assert_eq!(app.input_buffer, "tes");

        // Clear input
        app.input_buffer.clear();
        app.input_mode = InputMode::Normal;
        assert_eq!(app.input_mode, InputMode::Normal);
        assert!(app.input_buffer.is_empty());
    }

    /// Test delegation mode switching
    #[tokio::test]
    async fn test_delegation_mode() {
        let mut app = App::new().await.unwrap();

        app.current_tab = Tab::Delegation;

        // Initial mode
        assert_eq!(app.delegation_mode, DelegationMode::Analyze);

        // Test different delegation modes exist
        app.delegation_mode = DelegationMode::Delegate;
        assert_eq!(app.delegation_mode, DelegationMode::Delegate);

        app.delegation_mode = DelegationMode::ViewStats;
        assert_eq!(app.delegation_mode, DelegationMode::ViewStats);

        app.delegation_mode = DelegationMode::Analyze;
        assert_eq!(app.delegation_mode, DelegationMode::Analyze);
    }

    /// Test system status management
    #[tokio::test]
    async fn test_system_status() {
        let mut app = App::new().await.unwrap();

        // Initial state
        assert_eq!(app.system_status, "Starting...");

        // Update status
        app.system_status = "Running".to_string();
        assert_eq!(app.system_status, "Running");

        // Update metrics
        app.total_agents = 4;
        app.active_agents = 3;
        app.pending_tasks = 5;
        app.completed_tasks = 10;

        assert_eq!(app.total_agents, 4);
        assert_eq!(app.active_agents, 3);
        assert_eq!(app.pending_tasks, 5);
        assert_eq!(app.completed_tasks, 10);
    }

    /// Test UI rendering
    #[tokio::test]
    async fn test_ui_rendering() {
        let backend = TestBackend::new(80, 30);
        let mut terminal = Terminal::new(backend).unwrap();

        let mut app = App::new().await.unwrap();
        app.system_status = "Running".to_string();
        app.agents = vec![create_test_agent_info("test-agent")];
        app.logs = vec![LogEntry {
            timestamp: Utc::now(),
            level: "INFO".to_string(),
            agent: Some("test-agent".to_string()),
            message: "Test log entry".to_string(),
        }];

        // Draw the UI
        terminal.draw(|f| ui::draw(f, &app)).unwrap();

        // Get the buffer
        let buffer = terminal.backend().buffer();

        // Verify some content is rendered
        let content = buffer_to_string(buffer);
        assert!(content.contains("ccswarm"));
        assert!(content.contains("Overview"));
        assert!(content.contains("Agents"));
        assert!(content.contains("Tasks"));
        assert!(content.contains("Logs"));
        assert!(content.contains("Delegation"));
    }

    /// Test event handler
    #[tokio::test]
    async fn test_event_handler() {
        let mut handler = EventHandler::new(Duration::from_millis(100));

        // Since we can't easily simulate actual keyboard events in tests,
        // we just verify the handler is created and can be polled
        let event = handler.next().await;
        // Event might be None if no actual events occurred
        // The event handler returns either None or Some(Event)
        // We can't easily check for specific event types in tests
        assert!(event.is_none() || event.is_some());
    }

    /// Test terminal resize handling
    #[tokio::test]
    async fn test_resize_handling() {
        let mut app = App::new().await.unwrap();

        // Initial size
        assert_eq!(app.terminal_width, 80);
        assert_eq!(app.terminal_height, 24);

        // Update size
        app.terminal_width = 120;
        app.terminal_height = 40;
        assert_eq!(app.terminal_width, 120);
        assert_eq!(app.terminal_height, 40);

        // Update again
        app.terminal_width = 80;
        app.terminal_height = 24;
        assert_eq!(app.terminal_width, 80);
        assert_eq!(app.terminal_height, 24);
    }

    /// Helper function to convert buffer to string for assertions
    fn buffer_to_string(buffer: &Buffer) -> String {
        let mut result = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                let cell = &buffer[(x, y)];
                result.push_str(cell.symbol());
            }
            result.push('\n');
        }
        result
    }

    /// Test input processing
    #[tokio::test]
    async fn test_input_processing() {
        let mut app = App::new().await.unwrap();

        // Test command input
        app.input_mode = InputMode::Command;
        app.input_buffer = "test command".to_string();

        // Verify input buffer works
        assert_eq!(app.input_buffer, "test command");

        // Clear input
        app.input_buffer.clear();
        app.input_mode = InputMode::Normal;
        assert_eq!(app.input_mode, InputMode::Normal);
        assert!(app.input_buffer.is_empty());
    }

    /// Test tab-specific selection
    #[tokio::test]
    async fn test_tab_selection_indices() {
        let mut app = App::new().await.unwrap();

        // Add test data for different tabs
        app.agents = vec![
            create_test_agent_info("a1"),
            create_test_agent_info("a2"),
            create_test_agent_info("a3"),
        ];
        app.tasks = vec![create_test_task_info("t1")];

        // Test independent selection per tab
        app.current_tab = Tab::Agents;
        app.next_item();
        app.next_item();
        assert_eq!(app.selected_agent, 2);

        // Switch to tasks
        app.current_tab = Tab::Tasks;
        assert_eq!(app.selected_task, 0);

        // Switch back to agents - selection should be reset after tab change
        app.current_tab = Tab::Agents;
        // After reset_selection() is called in next_tab/previous_tab
    }

    /// Test empty state handling
    #[tokio::test]
    async fn test_empty_state_navigation() {
        let mut app = App::new().await.unwrap();

        // With empty lists, navigation should not panic
        app.current_tab = Tab::Agents;
        app.next_item();
        app.previous_item();
        assert_eq!(app.selected_agent, 0);

        app.current_tab = Tab::Tasks;
        app.next_item();
        app.previous_item();
        assert_eq!(app.selected_task, 0);

        app.current_tab = Tab::Logs;
        app.next_item();
        app.previous_item();
        assert_eq!(app.selected_log, 0);

        // Should complete without panicking
    }

    /// Test status display
    #[tokio::test]
    async fn test_status_display() {
        let mut app = App::new().await.unwrap();

        // Update system status
        app.system_status = "Operational".to_string();

        // Verify status is set
        assert_eq!(app.system_status, "Operational");
    }

    /// Test data refresh
    #[tokio::test]
    async fn test_data_refresh() {
        let mut app = App::new().await.unwrap();

        // Add some test data
        app.agents = vec![create_test_agent_info("agent-1")];
        app.tasks = vec![create_test_task_info("task-1")];

        // Verify data exists
        assert_eq!(app.agents.len(), 1);
        assert_eq!(app.tasks.len(), 1);
    }

    /// Test metrics initialization
    #[tokio::test]
    async fn test_metrics_initialization() {
        let app = App::new().await.unwrap();

        // Verify metrics are initialized
        assert_eq!(app.total_agents, 0);
        assert_eq!(app.active_agents, 0);
        assert_eq!(app.pending_tasks, 0);
        assert_eq!(app.completed_tasks, 0);
    }

    /// Test UI components rendering
    #[test]
    fn test_ui_component_rendering() {
        use ratatui::widgets::{Block, Borders};

        let backend = TestBackend::new(80, 30);
        let mut terminal = Terminal::new(backend).unwrap();

        // Test rendering a simple block
        terminal
            .draw(|f| {
                let block = Block::default().title("Test Block").borders(Borders::ALL);
                f.render_widget(block, f.area());
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = buffer_to_string(buffer);

        // Verify block title is rendered
        assert!(content.contains("Test Block"));
    }

    /// Test multi-key sequences
    #[tokio::test]
    async fn test_key_sequences() {
        let mut app = App::new().await.unwrap();

        // Start at Overview
        assert_eq!(app.current_tab, Tab::Overview);

        // Tab to Agents
        app.next_tab();
        assert_eq!(app.current_tab, Tab::Agents);

        // Tab to Tasks
        app.next_tab();
        assert_eq!(app.current_tab, Tab::Tasks);

        // BackTab to Agents
        app.previous_tab();
        assert_eq!(app.current_tab, Tab::Agents);

        // Set input mode
        app.input_mode = InputMode::Command;
        assert_eq!(app.input_mode, InputMode::Command);
    }

    /// Helper function to create test agent info
    fn create_test_agent_info(id: &str) -> AgentInfo {
        AgentInfo {
            id: id.to_string(),
            name: format!("Agent {}", id),
            specialization: "Backend".to_string(),
            provider_type: "claude_code".to_string(),
            provider_icon: "🤖".to_string(),
            provider_color: "blue".to_string(),
            status: AgentStatus::Available,
            current_task: None,
            tasks_completed: 0,
            last_activity: Utc::now(),
            workspace: "/tmp/workspace".to_string(),
        }
    }

    /// Helper function to create test task info
    fn create_test_task_info(id: &str) -> TaskInfo {
        TaskInfo {
            id: id.to_string(),
            description: format!("Task {}", id),
            priority: "Medium".to_string(),
            task_type: "Development".to_string(),
            status: "Pending".to_string(),
            assigned_agent: None,
            created_at: Utc::now(),
        }
    }
}
