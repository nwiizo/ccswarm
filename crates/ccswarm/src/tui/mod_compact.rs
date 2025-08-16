/// Compact TUI module - Unified implementation
use crate::utils::generic_handler::{EventBus, StateManager};
use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyCode};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
    Frame, Terminal,
};
use std::sync::Arc;

/// Unified TUI application state
#[derive(Default, Clone)]
pub struct AppState {
    pub selected_tab: usize,
    pub selected_item: usize,
    pub items: Vec<String>,
    pub messages: Vec<String>,
    pub is_running: bool,
}

/// Generic TUI component trait
pub trait Component {
    fn render(&self, f: &mut Frame, area: Rect, state: &AppState);
    fn handle_input(&mut self, key: KeyCode, state: &mut AppState) -> Result<()>;
}

/// Main TUI application using generic state management
pub struct TuiApp {
    state: StateManager<AppState>,
    events: Arc<EventBus<TuiEvent>>,
    components: Vec<Box<dyn Component + Send + Sync>>,
}

#[derive(Debug, Clone)]
pub enum TuiEvent {
    TabChanged(usize),
    ItemSelected(usize),
    MessageAdded(String),
    Quit,
}

impl Default for TuiApp {
    fn default() -> Self {
        Self::new()
    }
}

impl TuiApp {
    pub fn new() -> Self {
        let mut app = Self {
            state: StateManager::new(),
            events: Arc::new(EventBus::new()),
            components: Vec::new(),
        };

        // Initialize with default components
        app.components.push(Box::new(TabsComponent::new()));
        app.components.push(Box::new(ListComponent::new()));
        app.components.push(Box::new(LogComponent::new()));

        app
    }

    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        self.state
            .update(|s| {
                s.is_running = true;
                Ok(())
            })
            .await?;

        while self.state.read(|s| Ok(s.is_running)).await? {
            self.draw(terminal).await?;
            self.handle_events().await?;
        }

        Ok(())
    }

    async fn draw<B: Backend>(&self, terminal: &mut Terminal<B>) -> Result<()> {
        let state = self.state.read(|s| Ok(s.clone())).await?;

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(5),
                ])
                .split(f.area());

            // Render components in their areas
            if let Some(tabs) = self.components.first() {
                tabs.render(f, chunks[0], &state);
            }
            if let Some(list) = self.components.get(1) {
                list.render(f, chunks[1], &state);
            }
            if let Some(log) = self.components.get(2) {
                log.render(f, chunks[2], &state);
            }
        })?;

        Ok(())
    }

    async fn handle_events(&mut self) -> Result<()> {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let CrosstermEvent::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        self.state
                            .update(|s| {
                                s.is_running = false;
                                Ok(())
                            })
                            .await?;
                        self.events.publish(TuiEvent::Quit).await;
                    }
                    KeyCode::Tab => {
                        self.state
                            .update(|s| {
                                s.selected_tab = (s.selected_tab + 1) % 4;
                                Ok(())
                            })
                            .await?;
                    }
                    KeyCode::Up => {
                        self.state
                            .update(|s| {
                                if s.selected_item > 0 {
                                    s.selected_item -= 1;
                                }
                                Ok(())
                            })
                            .await?;
                    }
                    KeyCode::Down => {
                        self.state
                            .update(|s| {
                                if s.selected_item < s.items.len().saturating_sub(1) {
                                    s.selected_item += 1;
                                }
                                Ok(())
                            })
                            .await?;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    pub async fn add_message(&self, message: String) {
        let _ = self
            .state
            .update(|s| {
                s.messages.push(message.clone());
                if s.messages.len() > 100 {
                    s.messages.remove(0);
                }
                Ok(())
            })
            .await;

        self.events.publish(TuiEvent::MessageAdded(message)).await;
    }
}

/// Generic tabs component
struct TabsComponent {
    _titles: Vec<&'static str>,
}

impl TabsComponent {
    fn new() -> Self {
        Self {
            _titles: vec!["Overview", "Agents", "Tasks", "Sessions"],
        }
    }
}

impl Component for TabsComponent {
    fn render(&self, f: &mut Frame, area: Rect, state: &AppState) {
        let titles: Vec<Line> = self._titles.iter().map(|t| Line::from(*t)).collect();

        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title("Navigation"))
            .select(state.selected_tab)
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_widget(tabs, area);
    }

    fn handle_input(&mut self, _key: KeyCode, _state: &mut AppState) -> Result<()> {
        Ok(())
    }
}

/// Generic list component
struct ListComponent;

impl ListComponent {
    fn new() -> Self {
        Self
    }
}

impl Component for ListComponent {
    fn render(&self, f: &mut Frame, area: Rect, state: &AppState) {
        let items: Vec<ListItem> = state
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let style = if i == state.selected_item {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(Line::from(item.as_str())).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Items"))
            .style(Style::default().fg(Color::White));

        f.render_widget(list, area);
    }

    fn handle_input(&mut self, _key: KeyCode, _state: &mut AppState) -> Result<()> {
        Ok(())
    }
}

/// Generic log component
struct LogComponent;

impl LogComponent {
    fn new() -> Self {
        Self
    }
}

impl Component for LogComponent {
    fn render(&self, f: &mut Frame, area: Rect, state: &AppState) {
        let messages: Vec<Line> = state
            .messages
            .iter()
            .rev()
            .take(area.height as usize - 2)
            .map(|msg| Line::from(msg.as_str()))
            .collect();

        let log = Paragraph::new(messages)
            .block(Block::default().borders(Borders::ALL).title("Logs"))
            .style(Style::default().fg(Color::Gray));

        f.render_widget(log, area);
    }

    fn handle_input(&mut self, _key: KeyCode, _state: &mut AppState) -> Result<()> {
        Ok(())
    }
}

/// Simplified dashboard using generic components
pub struct Dashboard {
    app: TuiApp,
}

impl Default for Dashboard {
    fn default() -> Self {
        Self::new()
    }
}

impl Dashboard {
    pub fn new() -> Self {
        Self { app: TuiApp::new() }
    }

    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        // Initialize with sample data
        self.app
            .state
            .update(|s| {
                s.items = vec![
                    "Agent 1: Frontend Specialist".to_string(),
                    "Agent 2: Backend Specialist".to_string(),
                    "Agent 3: DevOps Engineer".to_string(),
                    "Agent 4: QA Tester".to_string(),
                ];
                s.messages = vec![
                    "System initialized".to_string(),
                    "Agents loaded".to_string(),
                    "Ready for tasks".to_string(),
                ];
                Ok(())
            })
            .await?;

        self.app.run(terminal).await
    }
}
