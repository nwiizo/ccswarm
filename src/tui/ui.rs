use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, Gauge, List, ListItem, ListState, Paragraph, Row,
        Table, Tabs, Wrap,
    },
    Frame,
};

use super::app::{App, DelegationMode, InputMode, Tab};

/// Draw the main UI
pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header/Tabs
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Footer/Status
        ])
        .split(f.size());

    // Draw header with tabs
    draw_tabs(f, chunks[0], app);

    // Draw main content based on current tab
    match app.current_tab {
        Tab::Overview => draw_overview(f, chunks[1], app),
        Tab::Agents => draw_agents(f, chunks[1], app),
        Tab::Tasks => draw_tasks(f, chunks[1], app),
        Tab::Logs => draw_logs(f, chunks[1], app),
        Tab::Delegation => draw_delegation(f, chunks[1], app),
    }

    // Draw footer
    draw_footer(f, chunks[2], app);

    // Draw input popup if needed
    if app.input_mode != InputMode::Normal {
        draw_input_popup(f, app);
    }
}

/// Draw tabs header
fn draw_tabs(f: &mut Frame, area: Rect, app: &App) {
    let tab_titles = vec![
        Tab::Overview.title(),
        Tab::Agents.title(),
        Tab::Tasks.title(),
        Tab::Logs.title(),
        Tab::Delegation.title(),
    ];

    let current_tab_index = match app.current_tab {
        Tab::Overview => 0,
        Tab::Agents => 1,
        Tab::Tasks => 2,
        Tab::Logs => 3,
        Tab::Delegation => 4,
    };

    let tabs = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" ccswarm TUI ")
                .title_alignment(Alignment::Center),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .select(current_tab_index);

    f.render_widget(tabs, area);
}

/// Draw overview tab
fn draw_overview(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(9),  // System stats with better spacing
            Constraint::Length(10), // Provider stats with more detail
            Constraint::Min(8),     // Agent summary with minimum height
        ])
        .split(area);

    // System stats with enhanced design
    draw_enhanced_system_stats(f, chunks[0], app);

    // Provider statistics with better layout
    draw_enhanced_provider_stats(f, chunks[1], app);

    // Agent summary with improved formatting
    draw_enhanced_agent_summary(f, chunks[2], app);
}

/// Draw enhanced system statistics with better visual design
fn draw_enhanced_system_stats(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    // Enhanced total agents with progress bar
    let agent_percentage = (app.active_agents * 100 / app.total_agents.max(1)) as u16;
    let total_agents = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" 🤖 Active Agents ")
                .title_alignment(Alignment::Center),
        )
        .gauge_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .percent(agent_percentage)
        .label(format!(
            "{}/{} ({}%)",
            app.active_agents, app.total_agents, agent_percentage
        ));
    f.render_widget(total_agents, chunks[0]);

    // Enhanced pending tasks with icon
    let pending_text = format!("📋 {}\nqueued", app.pending_tasks);
    let pending_tasks = Paragraph::new(pending_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Pending Tasks ")
                .title_alignment(Alignment::Center),
        )
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(pending_tasks, chunks[1]);

    // Enhanced completed tasks with icon
    let completed_text = format!("✅ {}\ncompleted", app.completed_tasks);
    let completed_tasks = Paragraph::new(completed_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Completed Tasks ")
                .title_alignment(Alignment::Center),
        )
        .style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(completed_tasks, chunks[2]);

    // Enhanced system status with icon and color
    let (status_icon, status_color) = match app.system_status.as_str() {
        "Running" => ("🟢", Color::Green),
        "Stopped" => ("🔴", Color::Red),
        _ => ("🟡", Color::Yellow),
    };
    let status_text = format!(
        "{} {}\nsystem",
        status_icon,
        app.system_status.to_lowercase()
    );
    let system_status = Paragraph::new(status_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" System Status ")
                .title_alignment(Alignment::Center),
        )
        .style(
            Style::default()
                .fg(status_color)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(system_status, chunks[3]);
}

/// Draw enhanced provider statistics with better layout
fn draw_enhanced_provider_stats(f: &mut Frame, area: Rect, app: &App) {
    // Count providers
    let mut provider_counts = std::collections::HashMap::new();
    for agent in &app.agents {
        *provider_counts
            .entry(agent.provider_type.clone())
            .or_insert(0) += 1;
    }

    // Create provider stats table
    let header = Row::new(vec![
        Cell::from("Provider").style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Count").style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Percentage").style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Status").style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    let rows: Vec<Row> = provider_counts
        .iter()
        .map(|(provider, count)| {
            let (icon, color) = match provider.as_str() {
                "Claude Code" => ("🤖", Color::Blue),
                "claude_code" => ("🤖", Color::Blue),
                "Aider" => ("🔧", Color::Green),
                "OpenAI Codex" => ("🧠", Color::Magenta),
                "Custom" => ("⚙️", Color::Gray),
                _ => ("❓", Color::White),
            };

            let percentage = (*count as f32 / app.agents.len().max(1) as f32) * 100.0;
            let status = if *count > 0 {
                "🟢 Active"
            } else {
                "⚪ Inactive"
            };

            Row::new(vec![
                Cell::from(format!("{} {}", icon, provider))
                    .style(Style::default().fg(color).add_modifier(Modifier::BOLD)),
                Cell::from(format!("{}", count)).style(Style::default().fg(Color::Yellow)),
                Cell::from(format!("{:.1}%", percentage)).style(Style::default().fg(Color::Cyan)),
                Cell::from(status).style(Style::default().fg(if *count > 0 {
                    Color::Green
                } else {
                    Color::Gray
                })),
            ])
        })
        .collect();

    let provider_table = Table::new(
        rows,
        [
            Constraint::Length(20), // Provider
            Constraint::Length(8),  // Count
            Constraint::Length(12), // Percentage
            Constraint::Length(12), // Status
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" 🔌 Provider Distribution ")
            .title_alignment(Alignment::Center),
    )
    .highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );

    f.render_widget(provider_table, area);
}

/// Draw enhanced agent summary with better visual hierarchy
fn draw_enhanced_agent_summary(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    // Enhanced agents table
    let header = Row::new(vec![
        Cell::from("Agent").style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Type").style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Status").style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Tasks").style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    let rows: Vec<Row> = app
        .agents
        .iter()
        .take(6)
        .map(|agent| {
            let (status_color, status_text, status_icon) = match agent.status {
                crate::agent::AgentStatus::Available => (Color::Green, "Available", "✅"),
                crate::agent::AgentStatus::Working => (Color::Cyan, "Working", "⚙️"),
                crate::agent::AgentStatus::Error(_) => (Color::Red, "Error", "❌"),
                crate::agent::AgentStatus::Initializing => (Color::Yellow, "Init", "🔄"),
                crate::agent::AgentStatus::WaitingForReview => (Color::Magenta, "Review", "⏳"),
                crate::agent::AgentStatus::ShuttingDown => (Color::DarkGray, "Shutdown", "⏹️"),
            };

            let is_master = agent.specialization.contains("Master");
            let (name_icon, name_color) = if is_master {
                ("👑", Color::Yellow)
            } else {
                match agent.specialization.as_str() {
                    s if s.contains("Frontend") => ("🎨", Color::Cyan),
                    s if s.contains("Backend") => ("⚙️", Color::Green),
                    s if s.contains("DevOps") => ("🔧", Color::Blue),
                    s if s.contains("QA") => ("🧪", Color::Magenta),
                    _ => ("🤖", Color::White),
                }
            };

            Row::new(vec![
                Cell::from(format!("{} {}", name_icon, agent.name)).style(
                    Style::default().fg(name_color).add_modifier(if is_master {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
                ),
                Cell::from(agent.specialization.clone()).style(Style::default().fg(Color::White)),
                Cell::from(format!("{} {}", status_icon, status_text))
                    .style(Style::default().fg(status_color)),
                Cell::from(format!("{}", agent.tasks_completed))
                    .style(Style::default().fg(Color::Yellow)),
            ])
        })
        .collect();

    let agents_table = Table::new(
        rows,
        [
            Constraint::Length(15), // Agent
            Constraint::Length(12), // Type
            Constraint::Length(12), // Status
            Constraint::Length(6),  // Tasks
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" 👥 Agents Overview ")
            .title_alignment(Alignment::Center),
    )
    .highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );

    f.render_widget(agents_table, chunks[0]);

    // System activity summary
    let activity_text = format!(
        "🔄 System Activity\n\n\
         ⏱️  Uptime: Active\n\
         📊 Total Agents: {}\n\
         🎯 Active: {}\n\
         📋 Tasks Queued: {}\n\
         ✅ Completed: {}\n\n\
         📈 Efficiency: {:.1}%",
        app.total_agents,
        app.active_agents,
        app.pending_tasks,
        app.completed_tasks,
        if app.total_agents > 0 {
            (app.active_agents as f32 / app.total_agents as f32) * 100.0
        } else {
            0.0
        }
    );

    let activity_paragraph = Paragraph::new(activity_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" 📊 System Metrics ")
                .title_alignment(Alignment::Center),
        )
        .style(Style::default().fg(Color::Cyan))
        .wrap(Wrap { trim: true });

    f.render_widget(activity_paragraph, chunks[1]);
}

/// Draw agents tab
fn draw_agents(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(area);

    // Create table rows for better formatting
    let header = Row::new(vec![
        Cell::from("#").style(Style::default().fg(Color::DarkGray)),
        Cell::from("Name").style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Provider").style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Type").style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Status").style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    let rows: Vec<Row> = app
        .agents
        .iter()
        .enumerate()
        .map(|(i, agent)| {
            let (status_color, status_text, status_icon) = match agent.status {
                crate::agent::AgentStatus::Available => (Color::Green, "Available", "✅"),
                crate::agent::AgentStatus::Working => (Color::Cyan, "Working", "⚙️"),
                crate::agent::AgentStatus::Error(_) => (Color::Red, "Error", "❌"),
                crate::agent::AgentStatus::Initializing => (Color::Yellow, "Initializing", "🔄"),
                crate::agent::AgentStatus::WaitingForReview => (Color::Magenta, "Waiting", "⏳"),
                crate::agent::AgentStatus::ShuttingDown => (Color::DarkGray, "Shutting Down", "⏹️"),
            };

            // Special styling for Master Claude Code
            let is_master = agent.specialization.contains("Master");
            let (name_color, name_icon, type_color) = if is_master {
                (Color::Yellow, "👑", Color::Yellow)
            } else {
                let type_icon = match agent.specialization.as_str() {
                    s if s.contains("Frontend") => "🎨",
                    s if s.contains("Backend") => "⚙️",
                    s if s.contains("DevOps") => "🔧",
                    s if s.contains("QA") => "🧪",
                    s if s.contains("Test") => "🔬",
                    s if s.contains("Error") => "🚨",
                    _ => "🤖",
                };
                (Color::Cyan, type_icon, Color::White)
            };

            let provider_display = if is_master {
                format!("{} claude_code", "👑")
            } else {
                format!("{} claude_code", "🤖")
            };

            Row::new(vec![
                Cell::from(format!("{}", i + 1)).style(Style::default().fg(Color::DarkGray)),
                Cell::from(format!("{} {}", name_icon, agent.name)).style(
                    Style::default().fg(name_color).add_modifier(if is_master {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
                ),
                Cell::from(provider_display).style(Style::default().fg(if is_master {
                    Color::Yellow
                } else {
                    Color::Blue
                })),
                Cell::from(agent.specialization.clone()).style(Style::default().fg(type_color)),
                Cell::from(format!("{} {}", status_icon, status_text)).style(
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ])
        })
        .collect();

    let agents_table = Table::new(
        rows,
        [
            Constraint::Length(3),  // #
            Constraint::Length(18), // Name
            Constraint::Length(16), // Provider
            Constraint::Length(20), // Type
            Constraint::Length(15), // Status
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" 🤖 Agent Management Dashboard ")
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(Color::White)),
    )
    .highlight_style(
        Style::default()
            .bg(Color::Blue)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("► ");

    // Create table state
    let mut table_state = ratatui::widgets::TableState::default();
    table_state.select(Some(app.selected_agent));

    f.render_stateful_widget(agents_table, chunks[0], &mut table_state);

    // Agent details
    if let Some(agent) = app.agents.get(app.selected_agent) {
        let details = vec![
            format!("ID: {}", agent.id),
            format!("Name: {}", agent.name),
            format!("Type: {}", agent.specialization),
            format!("Provider: {} {}", agent.provider_icon, agent.provider_type),
            format!("Status: {:?}", agent.status),
            format!("Tasks Completed: {}", agent.tasks_completed),
            format!("Last Activity: {}", agent.last_activity.format("%H:%M:%S")),
            format!("Workspace: {}", agent.workspace),
            format!(
                "Current Task: {}",
                agent.current_task.as_deref().unwrap_or("None")
            ),
        ];

        let details_text = details.join("\n");
        let details_paragraph = Paragraph::new(details_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Agent Details "),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(details_paragraph, chunks[1]);
    }
}

/// Draw tasks tab
fn draw_tasks(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);

    // Task list
    let header = Row::new(vec!["#", "Description", "Priority", "Type", "Status"])
        .style(Style::default().fg(Color::Yellow))
        .height(1)
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .tasks
        .iter()
        .enumerate()
        .map(|(i, task)| {
            let priority_color = match task.priority.as_str() {
                "High" => Color::Red,
                "Medium" => Color::Yellow,
                "Low" => Color::Green,
                _ => Color::White,
            };

            Row::new(vec![
                Cell::from(format!("{}", i + 1)),
                Cell::from(task.description.clone()),
                Cell::from(Span::styled(
                    task.priority.clone(),
                    Style::default().fg(priority_color),
                )),
                Cell::from(task.task_type.clone()),
                Cell::from(task.status.clone()),
            ])
        })
        .collect();

    let selected_style = Style::default()
        .bg(Color::Yellow)
        .fg(Color::Black)
        .add_modifier(Modifier::BOLD);

    let table = Table::new(
        rows,
        &[
            Constraint::Length(3),
            Constraint::Min(30),
            Constraint::Length(10),
            Constraint::Length(15),
            Constraint::Length(10),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Tasks (t to add new task) "),
    )
    .highlight_style(selected_style);

    let mut table_state = ratatui::widgets::TableState::default();
    table_state.select(Some(app.selected_task));

    f.render_stateful_widget(table, chunks[0], &mut table_state);

    // Task details
    if let Some(task) = app.tasks.get(app.selected_task) {
        let details = vec![
            format!("ID: {}", task.id),
            format!("Description: {}", task.description),
            format!("Priority: {}", task.priority),
            format!("Type: {}", task.task_type),
            format!("Status: {}", task.status),
            format!("Created: {}", task.created_at.format("%Y-%m-%d %H:%M:%S")),
            format!(
                "Assigned Agent: {}",
                task.assigned_agent.as_deref().unwrap_or("None")
            ),
        ];

        let details_text = details.join("\n");
        let details_paragraph = Paragraph::new(details_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Task Details "),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(details_paragraph, chunks[1]);
    }
}

/// Draw enhanced logs tab with better color coding and layout
fn draw_logs(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    // Log level summary bar
    let log_counts = app
        .logs
        .iter()
        .fold(std::collections::HashMap::new(), |mut acc, log| {
            *acc.entry(log.level.as_str()).or_insert(0) += 1;
            acc
        });

    let summary_text = format!(
        "📊 Logs: 🔴 {} errors | 🟡 {} warnings | 🟢 {} info | 🔵 {} debug | Total: {}",
        log_counts.get("ERROR").unwrap_or(&0),
        log_counts.get("WARN").unwrap_or(&0),
        log_counts.get("INFO").unwrap_or(&0),
        log_counts.get("DEBUG").unwrap_or(&0),
        app.logs.len()
    );

    let summary_paragraph = Paragraph::new(summary_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" 📋 Log Summary ")
                .title_alignment(Alignment::Center),
        )
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center);

    f.render_widget(summary_paragraph, chunks[0]);

    // Enhanced logs with better visual hierarchy
    let items: Vec<ListItem> = app
        .logs
        .iter()
        .rev() // Show newest logs first
        .take(100) // Limit to prevent performance issues
        .map(|log| {
            let (level_color, level_icon) = match log.level.as_str() {
                "ERROR" => (Color::Red, "🔴"),
                "WARN" => (Color::Yellow, "🟡"),
                "INFO" => (Color::Green, "🟢"),
                "DEBUG" => (Color::Cyan, "🔵"),
                _ => (Color::White, "⚪"),
            };

            let agent_name = log.agent.as_deref().unwrap_or("System");
            let (agent_icon, agent_color) = if agent_name == "System" {
                ("🖥️", Color::Gray)
            } else if agent_name.contains("master") || agent_name.contains("Master") {
                ("👑", Color::Yellow)
            } else {
                ("🤖", Color::Cyan)
            };

            let content = vec![Line::from(vec![
                Span::styled(
                    format!("{} ", log.timestamp.format("%H:%M:%S")),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(format!("{} ", level_icon), Style::default().fg(level_color)),
                Span::styled(
                    format!("{:<5} ", log.level),
                    Style::default()
                        .fg(level_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{} {:<10} ", agent_icon, agent_name),
                    Style::default().fg(agent_color),
                ),
                Span::styled(log.message.clone(), Style::default().fg(Color::White)),
            ])];
            ListItem::new(content)
        })
        .collect();

    let mut logs_state = ListState::default();
    logs_state.select(Some(app.selected_log));

    let logs_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" 📜 Live Logs (r: refresh, ↑↓: navigate) ")
                .title_alignment(Alignment::Center),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(logs_list, chunks[1], &mut logs_state);
}

/// Draw delegation tab
fn draw_delegation(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Mode selector
            Constraint::Min(10),   // Main content
            Constraint::Length(8), // Instructions/Help
        ])
        .split(area);

    // Mode selector
    draw_delegation_mode_selector(f, chunks[0], app);

    // Main content based on delegation mode
    match app.delegation_mode {
        DelegationMode::Analyze => draw_delegation_analysis(f, chunks[1], app),
        DelegationMode::Delegate => draw_delegation_interface(f, chunks[1], app),
        DelegationMode::ViewStats => draw_delegation_stats(f, chunks[1], app),
    }

    // Instructions
    draw_delegation_instructions(f, chunks[2], app);
}

/// Draw delegation mode selector
fn draw_delegation_mode_selector(f: &mut Frame, area: Rect, app: &App) {
    let mode_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(area);

    let modes = [
        ("🔍 Analyze", DelegationMode::Analyze, Color::Blue),
        ("🎯 Delegate", DelegationMode::Delegate, Color::Green),
        ("📊 Stats", DelegationMode::ViewStats, Color::Magenta),
    ];

    for (i, (title, mode, color)) in modes.iter().enumerate() {
        let is_selected = *mode == app.delegation_mode;

        let block = if is_selected {
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::default().fg(*color).add_modifier(Modifier::BOLD))
                .title(*title)
                .title_alignment(Alignment::Center)
                .style(Style::default().bg(Color::DarkGray))
        } else {
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Gray))
                .title(*title)
                .title_alignment(Alignment::Center)
        };

        let description = match mode {
            DelegationMode::Analyze => "Analyze tasks and\nget agent recommendations",
            DelegationMode::Delegate => "Manually delegate\ntasks to specific agents",
            DelegationMode::ViewStats => "View delegation\nstatistics and history",
        };

        let paragraph = Paragraph::new(description)
            .block(block)
            .style(Style::default().fg(if is_selected {
                Color::White
            } else {
                Color::Gray
            }))
            .alignment(Alignment::Center);

        f.render_widget(paragraph, mode_chunks[i]);
    }
}

/// Draw delegation analysis interface
fn draw_delegation_analysis(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Recent analysis results
    let analysis_items: Vec<ListItem> = app
        .delegation_decisions
        .iter()
        .rev()
        .take(10)
        .map(|decision| {
            let agent_icon = match decision.recommended_agent.as_str() {
                "Frontend" => "🎨",
                "Backend" => "⚙️",
                "DevOps" => "🔧",
                "QA" => "🧪",
                _ => "🤖",
            };

            let confidence_color = if decision.confidence >= 0.8 {
                Color::Green
            } else if decision.confidence >= 0.6 {
                Color::Yellow
            } else {
                Color::Red
            };

            let content = vec![Line::from(vec![
                Span::styled(format!("{} ", agent_icon), Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{:<8} ", decision.recommended_agent),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("({:.0}%) ", decision.confidence * 100.0),
                    Style::default().fg(confidence_color),
                ),
                Span::styled(
                    decision.task_description.clone(),
                    Style::default().fg(Color::Gray),
                ),
            ])];
            ListItem::new(content)
        })
        .collect();

    let mut analysis_state = ListState::default();
    analysis_state.select(Some(app.selected_delegation));

    let analysis_list = List::new(analysis_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" 🔍 Analysis Results ")
                .title_alignment(Alignment::Center),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ");

    f.render_stateful_widget(analysis_list, chunks[0], &mut analysis_state);

    // Analysis details
    if let Some(decision) = app.delegation_decisions.get(app.selected_delegation) {
        let details = format!(
            "📋 Task Analysis Details\n\n\
             Task: {}\n\n\
             🎯 Recommended Agent: {}\n\
             📊 Confidence: {:.1}%\n\
             🧠 Reasoning: {}\n\
             📅 Analyzed: {}\n\n\
             💡 Press Enter to delegate this task",
            decision.task_description,
            decision.recommended_agent,
            decision.confidence * 100.0,
            decision.reasoning,
            decision.created_at.format("%H:%M:%S")
        );

        let details_paragraph = Paragraph::new(details)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" 📋 Analysis Details ")
                    .title_alignment(Alignment::Center),
            )
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true });

        f.render_widget(details_paragraph, chunks[1]);
    } else {
        let empty_text = "🔍 Task Analysis\n\n\
                         No analysis results yet.\n\n\
                         Press Enter to analyze a new task\n\
                         and get Master's recommendation\n\
                         for optimal agent assignment.";

        let empty_paragraph = Paragraph::new(empty_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" 📋 Ready for Analysis ")
                    .title_alignment(Alignment::Center),
            )
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(empty_paragraph, chunks[1]);
    }
}

/// Draw delegation interface
fn draw_delegation_interface(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // Available agents
    let agent_items: Vec<ListItem> = vec![
        ListItem::new(Line::from(vec![
            Span::styled("🎨 ", Style::default().fg(Color::Cyan)),
            Span::styled(
                "Frontend",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" - UI/UX Development", Style::default().fg(Color::Gray)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("⚙️ ", Style::default().fg(Color::Green)),
            Span::styled(
                "Backend",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" - API/Database Work", Style::default().fg(Color::Gray)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("🔧 ", Style::default().fg(Color::Blue)),
            Span::styled(
                "DevOps",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" - Infrastructure/Deploy", Style::default().fg(Color::Gray)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("🧪 ", Style::default().fg(Color::Magenta)),
            Span::styled(
                "QA",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" - Testing/Quality", Style::default().fg(Color::Gray)),
        ])),
    ];

    let agents_list = List::new(agent_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" 🤖 Available Agents ")
                .title_alignment(Alignment::Center),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(agents_list, chunks[0]);

    // Delegation form and recent delegations
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(0)])
        .split(chunks[1]);

    // Delegation form
    let form_text = "🎯 Manual Task Delegation\n\n\
                     Format: <agent> <task_description>\n\n\
                     Examples:\n\
                     • frontend Create user dashboard\n\
                     • backend Add payment processing\n\
                     • qa Write end-to-end tests\n\
                     • devops Setup CI/CD pipeline";

    let form_paragraph = Paragraph::new(form_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" 📝 Delegation Form ")
                .title_alignment(Alignment::Center),
        )
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true });

    f.render_widget(form_paragraph, right_chunks[0]);

    // Recent delegations
    let delegation_items: Vec<ListItem> = app
        .delegation_decisions
        .iter()
        .rev()
        .take(6)
        .filter(|d| d.reasoning.contains("Manual delegation"))
        .map(|decision| {
            let agent_icon = match decision.recommended_agent.as_str() {
                "frontend" | "Frontend" => "🎨",
                "backend" | "Backend" => "⚙️",
                "devops" | "DevOps" => "🔧",
                "qa" | "QA" => "🧪",
                _ => "🤖",
            };

            let content = vec![Line::from(vec![
                Span::styled(format!("{} ", agent_icon), Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{:<8} ", decision.recommended_agent),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{} ", decision.created_at.format("%H:%M")),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    decision.task_description.clone(),
                    Style::default().fg(Color::White),
                ),
            ])];
            ListItem::new(content)
        })
        .collect();

    let delegations_list = List::new(delegation_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" 📋 Recent Delegations ")
            .title_alignment(Alignment::Center),
    );

    f.render_widget(delegations_list, right_chunks[1]);
}

/// Draw delegation statistics
fn draw_delegation_stats(f: &mut Frame, area: Rect, app: &App) {
    if app.delegation_decisions.is_empty() {
        let empty_text = "📊 Delegation Statistics\n\n\
                         No delegation data available yet.\n\n\
                         Start analyzing or delegating tasks\n\
                         to see statistics here.";

        let empty_paragraph = Paragraph::new(empty_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" 📊 Statistics ")
                    .title_alignment(Alignment::Center),
            )
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(empty_paragraph, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Agent distribution chart
    let mut agent_counts = std::collections::HashMap::new();
    let mut total_confidence = 0.0;

    for decision in &app.delegation_decisions {
        *agent_counts
            .entry(decision.recommended_agent.clone())
            .or_insert(0) += 1;
        total_confidence += decision.confidence;
    }

    let total = app.delegation_decisions.len();
    let avg_confidence = total_confidence / total as f64;

    // Create agent distribution table
    let header = Row::new(vec![
        Cell::from("Agent").style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Count").style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Percentage").style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    let rows: Vec<Row> = agent_counts
        .iter()
        .map(|(agent, count)| {
            let percentage = (*count as f64 / total as f64) * 100.0;
            let agent_icon = match agent.as_str() {
                "Frontend" | "frontend" => "🎨",
                "Backend" | "backend" => "⚙️",
                "DevOps" | "devops" => "🔧",
                "QA" | "qa" => "🧪",
                _ => "🤖",
            };

            Row::new(vec![
                Cell::from(format!("{} {}", agent_icon, agent)).style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from(format!("{}", count)).style(Style::default().fg(Color::Yellow)),
                Cell::from(format!("{:.1}%", percentage)).style(Style::default().fg(Color::Green)),
            ])
        })
        .collect();

    let distribution_table = Table::new(
        rows,
        [
            Constraint::Length(15), // Agent
            Constraint::Length(8),  // Count
            Constraint::Length(12), // Percentage
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" 📊 Agent Distribution ")
            .title_alignment(Alignment::Center),
    );

    f.render_widget(distribution_table, chunks[0]);

    // Statistics summary
    let stats_text = format!(
        "📈 Delegation Summary\n\n\
         Total Delegations: {}\n\
         Average Confidence: {:.1}%\n\n\
         📊 Distribution:\n\
         {}\n\n\
         🎯 Most Delegated: {}\n\
         📅 Latest: {}",
        total,
        avg_confidence * 100.0,
        agent_counts
            .iter()
            .map(|(agent, count)| format!("• {}: {}", agent, count))
            .collect::<Vec<_>>()
            .join("\n         "),
        agent_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(agent, _)| agent.as_str())
            .unwrap_or("None"),
        app.delegation_decisions
            .last()
            .map(|d| d.created_at.format("%H:%M:%S").to_string())
            .unwrap_or("None".to_string())
    );

    let stats_paragraph = Paragraph::new(stats_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" 📋 Statistics Summary ")
                .title_alignment(Alignment::Center),
        )
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true });

    f.render_widget(stats_paragraph, chunks[1]);
}

/// Draw delegation instructions
fn draw_delegation_instructions(f: &mut Frame, area: Rect, app: &App) {
    let instructions = match app.delegation_mode {
        DelegationMode::Analyze => {
            "🔍 Analysis Mode Instructions:\n\
             • Press Enter to analyze a new task description\n\
             • Master will recommend the optimal agent based on task content\n\
             • Use ↑/↓ to navigate through analysis results\n\
             • View detailed reasoning and confidence scores"
        }
        DelegationMode::Delegate => {
            "🎯 Delegation Mode Instructions:\n\
             • Press Enter to manually delegate a task to a specific agent\n\
             • Format: <agent_type> <task_description>\n\
             • Valid agents: frontend, backend, devops, qa\n\
             • Tasks are automatically added to the queue"
        }
        DelegationMode::ViewStats => {
            "📊 Statistics Mode Instructions:\n\
             • View delegation patterns and agent utilization\n\
             • Analyze confidence scores and distribution\n\
             • Track delegation history and trends\n\
             • Use Space to switch between modes"
        }
    };

    let instructions_paragraph = Paragraph::new(instructions)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" ℹ️  Instructions ")
                .title_alignment(Alignment::Center),
        )
        .style(Style::default().fg(Color::Cyan))
        .wrap(Wrap { trim: true });

    f.render_widget(instructions_paragraph, area);
}

/// Draw enhanced footer with better key binding display
fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
    let key_bindings = match app.current_tab {
        Tab::Overview => vec![
            ("Tab/Shift+Tab", "Switch tabs", Color::Cyan),
            ("c", "Command prompt", Color::Yellow),
            ("r", "Refresh data", Color::Green),
            ("q", "Quit", Color::Red),
        ],
        Tab::Agents => vec![
            ("Tab", "Switch tabs", Color::Cyan),
            ("↑/↓ j/k", "Navigate", Color::White),
            ("Enter", "Start/Details", Color::Green),
            ("n", "New Agent", Color::Blue),
            ("d", "Delete", Color::Red),
            ("c", "Command", Color::Yellow),
        ],
        Tab::Tasks => vec![
            ("Tab", "Switch tabs", Color::Cyan),
            ("↑/↓ j/k", "Navigate", Color::White),
            ("t", "Add Task", Color::Green),
            ("c", "Command", Color::Yellow),
            ("r", "Refresh", Color::Blue),
        ],
        Tab::Logs => vec![
            ("Tab", "Switch tabs", Color::Cyan),
            ("↑/↓ j/k", "Navigate", Color::White),
            ("r", "Refresh", Color::Green),
            ("c", "Command", Color::Yellow),
        ],
        Tab::Delegation => vec![
            ("Tab", "Switch tabs", Color::Cyan),
            ("Space", "Switch mode", Color::Blue),
            ("Enter", "Analyze/Delegate", Color::Green),
            ("↑/↓ j/k", "Navigate", Color::White),
            ("c", "Command", Color::Yellow),
        ],
    };

    let help_text: Vec<Line> = vec![Line::from(
        key_bindings
            .iter()
            .flat_map(|(key, desc, color)| {
                vec![
                    Span::styled(
                        format!(" {} ", key),
                        Style::default()
                            .fg(*color)
                            .bg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format!(" {} ", desc), Style::default().fg(Color::Gray)),
                    Span::raw(" │ "),
                ]
                .into_iter()
            })
            .collect::<Vec<_>>(),
    )];

    let footer = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" ⌨️  Keyboard Shortcuts ")
                .title_alignment(Alignment::Center),
        )
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    f.render_widget(footer, area);
}

/// Draw enhanced input popup with better styling
fn draw_input_popup(f: &mut Frame, app: &App) {
    let area = centered_rect(70, 25, f.size());

    let (title, _title_icon, border_color) = match app.input_mode {
        InputMode::AddingTask => (" 📋 Add New Task ", "📋", Color::Green),
        InputMode::CreatingAgent => (" 🤖 Create New Agent ", "🤖", Color::Blue),
        InputMode::Command => (" 💻 Command Prompt ", "💻", Color::Yellow),
        InputMode::DelegationInput => (" 🎯 Master Delegation ", "🎯", Color::Magenta),
        _ => (" ✏️  Input ", "✏️", Color::White),
    };

    let prompt = match app.input_mode {
        InputMode::AddingTask => {
            "Enter task description:\n\n\
             💡 Tips:\n\
             • Use [high]/[medium]/[low] for priority\n\
             • Use [test]/[docs]/[bug]/[feature] for type\n\
             • Example: \"Fix login bug [high] [bug]\""
        }
        InputMode::CreatingAgent => {
            "Select agent specialization:\n\n\
             🎨 frontend  - UI/UX development\n\
             ⚙️  backend   - API/Database work\n\
             🔧 devops    - Infrastructure/Deploy\n\
             🧪 qa        - Testing/Quality assurance"
        }
        InputMode::Command => {
            "Enter command:\n\n\
             💡 Quick commands:\n\
             • help - Show all commands\n\
             • start_agent master - Start master agent\n\
             • task <description> - Add new task\n\
             • status - Show system status"
        }
        InputMode::DelegationInput => match app.delegation_mode {
            DelegationMode::Analyze => {
                "🔍 Analyze Task for Delegation:\n\n\
                     Enter task description to get Master's recommendation\n\
                     💡 Examples:\n\
                     • \"Create login form with validation\"\n\
                     • \"Fix API endpoint error handling\"\n\
                     • \"Write unit tests for user service\""
            }
            DelegationMode::Delegate => {
                "🎯 Delegate Task to Agent:\n\n\
                     Format: <agent_type> <task_description>\n\
                     💡 Examples:\n\
                     • \"frontend Create responsive navigation\"\n\
                     • \"backend Add user authentication API\"\n\
                     • \"qa Write integration tests\""
            }
            _ => "Enter delegation input:",
        },
        _ => "Enter your input:",
    };

    // Create a shadow effect
    let shadow_area = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width,
        height: area.height,
    };
    f.render_widget(
        Block::default().style(Style::default().bg(Color::Black)),
        shadow_area,
    );

    f.render_widget(Clear, area); // Clear background

    let input_block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        );

    let input_area = input_block.inner(area);
    f.render_widget(input_block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // Prompt area
            Constraint::Length(3), // Input area
            Constraint::Length(2), // Help area
        ])
        .split(input_area);

    // Prompt text
    let prompt_paragraph = Paragraph::new(prompt)
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true });
    f.render_widget(prompt_paragraph, chunks[0]);

    // Input box with cursor
    let input_text = format!("{}_", app.input_buffer);
    let input_paragraph = Paragraph::new(input_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Input ")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(input_paragraph, chunks[1]);

    // Help text
    let help_text = vec![Line::from(vec![
        Span::styled(
            " Enter ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Confirm ", Style::default().fg(Color::Gray)),
        Span::raw("  "),
        Span::styled(
            " Esc ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Cancel ", Style::default().fg(Color::Gray)),
    ])];
    let help_paragraph = Paragraph::new(help_text).alignment(Alignment::Center);
    f.render_widget(help_paragraph, chunks[2]);
}

/// Calculate centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
