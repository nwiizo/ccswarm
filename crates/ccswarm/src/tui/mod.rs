pub mod app;
pub mod event;
pub mod ui;

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::io;
use tokio::time::Duration;

use crate::execution::ExecutionEngine;
use app::App;
use event::EventHandler;

#[cfg(test)]
mod tests;

/// Main TUI entry point
pub async fn run_tui() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new().await?;
    let mut event_handler = EventHandler::new(Duration::from_millis(100));

    // Main event loop
    let result = run_app(&mut terminal, &mut app, &mut event_handler).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

/// Main TUI entry point with execution engine
pub async fn run_tui_with_engine(execution_engine: ExecutionEngine) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state with execution engine
    let mut app = App::new().await?;
    app.set_execution_engine(execution_engine);
    let mut event_handler = EventHandler::new(Duration::from_millis(100));

    // Main event loop
    let result = run_app(&mut terminal, &mut app, &mut event_handler).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    event_handler: &mut EventHandler,
) -> Result<()> {
    loop {
        // Draw UI
        terminal.draw(|f| ui::draw(f, app))?;

        // Handle events
        if let Some(event) = event_handler.next().await {
            match event {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if app.input_mode == app::InputMode::Normal {
                        match key.code {
                            KeyCode::Char('q') => break,
                            KeyCode::Char('n') => app.create_new_session().await?,
                            KeyCode::Char('d') => app.delete_current_session().await?,
                            KeyCode::Char('s') => app.show_status().await?,
                            KeyCode::Char('t') => app.add_task_prompt().await?,
                            KeyCode::Char('c') => app.open_command_prompt().await?,
                            KeyCode::Char('r') => app.refresh_data().await?,
                            KeyCode::Up | KeyCode::Char('k') => app.previous_item(),
                            KeyCode::Down | KeyCode::Char('j') => app.next_item(),
                            KeyCode::Enter => {
                                if app.current_tab == app::Tab::Delegation {
                                    app.handle_delegation_enter().await?;
                                } else {
                                    app.activate_selected().await?;
                                }
                            }
                            KeyCode::Char(' ') => {
                                if app.current_tab == app::Tab::Delegation {
                                    app.switch_delegation_mode();
                                }
                            }
                            KeyCode::Tab => app.next_tab(),
                            KeyCode::BackTab => app.previous_tab(),
                            KeyCode::Esc => app.cancel_current_action(),
                            _ => {}
                        }
                    } else {
                        // Handle input mode
                        match key.code {
                            KeyCode::Enter => app.process_input().await?,
                            KeyCode::Esc => app.cancel_current_action(),
                            KeyCode::Backspace => app.handle_backspace(),
                            KeyCode::Char(c) => app.handle_char_input(c),
                            _ => {}
                        }
                    }
                }
                Event::Resize(width, height) => {
                    app.update_size(width, height);
                }
                _ => {}
            }
        }

        // Periodic updates
        app.update().await?;
    }

    Ok(())
}
