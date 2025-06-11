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

use super::app::{App, InputMode, Tab};

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
    ];

    let current_tab_index = match app.current_tab {
        Tab::Overview => 0,
        Tab::Agents => 1,
        Tab::Tasks => 2,
        Tab::Logs => 3,
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
            Constraint::Length(7), // System stats
            Constraint::Length(8), // Provider stats
            Constraint::Min(0),    // Agent summary
        ])
        .split(area);

    // System stats
    draw_system_stats(f, chunks[0], app);

    // Provider statistics
    draw_provider_stats(f, chunks[1], app);

    // Agent summary
    draw_agent_summary(f, chunks[2], app);
}

/// Draw system statistics
fn draw_system_stats(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    // Total agents
    let total_agents = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Total Agents "),
        )
        .gauge_style(Style::default().fg(Color::Blue))
        .percent((app.active_agents * 100 / app.total_agents.max(1)) as u16)
        .label(format!("{}/{}", app.active_agents, app.total_agents));
    f.render_widget(total_agents, chunks[0]);

    // Pending tasks
    let pending_tasks = Paragraph::new(format!("{}", app.pending_tasks))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Pending Tasks "),
        )
        .alignment(Alignment::Center);
    f.render_widget(pending_tasks, chunks[1]);

    // Completed tasks
    let completed_tasks = Paragraph::new(format!("{}", app.completed_tasks))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Completed Tasks "),
        )
        .alignment(Alignment::Center);
    f.render_widget(completed_tasks, chunks[2]);

    // System status
    let status_color = match app.system_status.as_str() {
        "Running" => Color::Green,
        "Stopped" => Color::Red,
        _ => Color::Yellow,
    };
    let system_status = Paragraph::new(app.system_status.clone())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" System Status "),
        )
        .style(Style::default().fg(status_color))
        .alignment(Alignment::Center);
    f.render_widget(system_status, chunks[3]);
}

/// Draw provider statistics
fn draw_provider_stats(f: &mut Frame, area: Rect, app: &App) {
    // Count providers
    let mut provider_counts = std::collections::HashMap::new();
    for agent in &app.agents {
        *provider_counts
            .entry(agent.provider_type.clone())
            .or_insert(0) += 1;
    }

    // Create provider stats items
    let items: Vec<ListItem> = provider_counts
        .iter()
        .map(|(provider, count)| {
            let (icon, color) = match provider.as_str() {
                "Claude Code" => ("🤖", Color::Blue),
                "Aider" => ("🔧", Color::Green),
                "OpenAI Codex" => ("🧠", Color::Magenta),
                "Custom" => ("⚙️", Color::Gray),
                _ => ("❓", Color::White),
            };

            let content = vec![Line::from(vec![
                Span::styled(
                    format!("{} {:<15}", icon, provider),
                    Style::default().fg(color),
                ),
                Span::styled(
                    format!("{} agent{}", count, if *count == 1 { "" } else { "s" }),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    format!(
                        " ({:.1}%)",
                        (*count as f32 / app.agents.len().max(1) as f32) * 100.0
                    ),
                    Style::default().fg(Color::Gray),
                ),
            ])];
            ListItem::new(content)
        })
        .collect();

    let provider_list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Provider Distribution "),
    );

    f.render_widget(provider_list, area);
}

/// Draw agent summary
fn draw_agent_summary(f: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .agents
        .iter()
        .map(|agent| {
            let status_color = match agent.status {
                crate::agent::AgentStatus::Available => Color::Green,
                crate::agent::AgentStatus::Working => Color::Yellow,
                crate::agent::AgentStatus::Error(_) => Color::Red,
                _ => Color::Gray,
            };

            let provider_color = match agent.provider_color.as_str() {
                "blue" => Color::Blue,
                "green" => Color::Green,
                "purple" => Color::Magenta,
                "gray" => Color::Gray,
                _ => Color::White,
            };

            let content = vec![Line::from(vec![
                Span::styled(
                    format!("{:<20}", agent.name),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    format!("{} {:<12}", agent.provider_icon, agent.provider_type),
                    Style::default().fg(provider_color),
                ),
                Span::styled(
                    format!("{:<15}", agent.specialization),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("{:<12}", format!("{:?}", agent.status)),
                    Style::default().fg(status_color),
                ),
                Span::styled(
                    format!("{}", agent.tasks_completed),
                    Style::default().fg(Color::Green),
                ),
            ])];
            ListItem::new(content)
        })
        .collect();

    let agents_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Agents Overview "),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(agents_list, area);
}

/// Draw agents tab
fn draw_agents(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);

    // Agent list
    let items: Vec<ListItem> = app
        .agents
        .iter()
        .enumerate()
        .map(|(i, agent)| {
            let status_color = match agent.status {
                crate::agent::AgentStatus::Available => Color::Green,
                crate::agent::AgentStatus::Working => Color::Yellow,
                crate::agent::AgentStatus::Error(_) => Color::Red,
                _ => Color::Gray,
            };

            let provider_color = match agent.provider_color.as_str() {
                "blue" => Color::Blue,
                "green" => Color::Green,
                "purple" => Color::Magenta,
                "gray" => Color::Gray,
                _ => Color::White,
            };

            let content = vec![Line::from(vec![
                Span::styled(format!("{:<3}", i + 1), Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{:<20}", agent.name),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    format!("{} {:<10}", agent.provider_icon, agent.provider_type),
                    Style::default().fg(provider_color),
                ),
                Span::styled(
                    format!("{:<12}", agent.specialization),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("{:<15}", format!("{:?}", agent.status)),
                    Style::default().fg(status_color),
                ),
            ])];
            ListItem::new(content)
        })
        .collect();

    let mut agents_state = ListState::default();
    agents_state.select(Some(app.selected_agent));

    let agents_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Agents: #  Name                Provider      Type         Status          (↑/↓ to navigate, Enter to view details) "),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(agents_list, chunks[0], &mut agents_state);

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

/// Draw logs tab
fn draw_logs(f: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .logs
        .iter()
        .map(|log| {
            let level_color = match log.level.as_str() {
                "ERROR" => Color::Red,
                "WARN" => Color::Yellow,
                "INFO" => Color::Green,
                "DEBUG" => Color::Cyan,
                _ => Color::White,
            };

            let content = vec![Line::from(vec![
                Span::styled(
                    format!("{} ", log.timestamp.format("%H:%M:%S")),
                    Style::default().fg(Color::Gray),
                ),
                Span::styled(
                    format!("{:<5} ", log.level),
                    Style::default().fg(level_color),
                ),
                Span::styled(
                    format!("{:<12} ", log.agent.as_deref().unwrap_or("System")),
                    Style::default().fg(Color::Cyan),
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
                .title(" Logs (r to refresh) "),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_stateful_widget(logs_list, area, &mut logs_state);
}

/// Draw footer with key bindings
fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
    let key_bindings = match app.current_tab {
        Tab::Overview => "Tab: Switch | c: Command | r: Refresh | q: Quit",
        Tab::Agents => "Tab: Switch | ↑/↓: Navigate | Enter: Details | n: New Agent | d: Delete | c: Command | q: Quit",
        Tab::Tasks => "Tab: Switch | ↑/↓: Navigate | t: Add Task | c: Command | q: Quit",
        Tab::Logs => "Tab: Switch | ↑/↓: Navigate | r: Refresh | c: Command | q: Quit",
    };

    let footer = Paragraph::new(key_bindings)
        .block(Block::default().borders(Borders::ALL).title(" Controls "))
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);

    f.render_widget(footer, area);
}

/// Draw input popup
fn draw_input_popup(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 20, f.size());

    let title = match app.input_mode {
        InputMode::AddingTask => " Add New Task ",
        InputMode::CreatingAgent => " Create New Agent ",
        InputMode::Command => " Command Prompt ",
        _ => " Input ",
    };

    let prompt = match app.input_mode {
        InputMode::AddingTask => {
            "Enter task description (use [high]/[low] for priority, [test]/[docs]/etc for type):"
        }
        InputMode::CreatingAgent => "Enter agent type (frontend/backend/devops/qa):",
        InputMode::Command => "Enter command (help for available commands):",
        _ => "Enter input:",
    };

    f.render_widget(Clear, area); // Clear background

    let input_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let input_area = input_block.inner(area);
    f.render_widget(input_block, area);

    let text = vec![
        Line::from(prompt),
        Line::from(""),
        Line::from(app.input_buffer.as_str()),
        Line::from(""),
        Line::from("Press Enter to confirm, Esc to cancel"),
    ];

    let input_paragraph = Paragraph::new(text).wrap(Wrap { trim: true });
    f.render_widget(input_paragraph, input_area);
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
