use crossterm::event::{self, Event};
use std::time::Duration;
use tokio::time::{interval, Interval, MissedTickBehavior};

/// Event handler for TUI
pub struct EventHandler {
    tick_interval: Interval,
}

impl EventHandler {
    /// Create new event handler
    pub fn new(tick_rate: Duration) -> Self {
        let mut tick_interval = interval(tick_rate);
        tick_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        Self { tick_interval }
    }

    /// Get next event (either user input or tick)
    pub async fn next(&mut self) -> Option<Event> {
        tokio::select! {
            _ = self.tick_interval.tick() => {
                // Periodic tick for updates
                None
            }
            event = read_event() => {
                event.ok()
            }
        }
    }
}

/// Read crossterm event asynchronously
async fn read_event() -> anyhow::Result<Event> {
    loop {
        if event::poll(Duration::from_millis(0))? {
            return Ok(event::read()?);
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
