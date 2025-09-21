use colored::Colorize;
use std::io::{self, Write};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Progress indicator types
pub enum ProgressStyle {
    Spinner,
    Bar,
    Steps,
    Dots,
}

/// Real-time progress tracker with user-friendly output
pub struct ProgressTracker {
    message: String,
    style: ProgressStyle,
    start_time: Instant,
    current_step: usize,
    total_steps: Option<usize>,
    sub_messages: Vec<String>,
    is_running: Arc<Mutex<bool>>,
}

impl ProgressTracker {
    pub fn new(message: impl Into<String>, style: ProgressStyle) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            message: message.into(),
            style,
            start_time: Instant::now(),
            current_step: 0,
            total_steps: None,
            sub_messages: Vec::new(),
            is_running: Arc::new(Mutex::new(true)),
        }))
    }

    pub fn with_steps(message: impl Into<String>, total: usize) -> Arc<Mutex<Self>> {
        let tracker = Self {
            message: message.into(),
            style: ProgressStyle::Steps,
            start_time: Instant::now(),
            current_step: 0,
            total_steps: Some(total),
            sub_messages: Vec::new(),
            is_running: Arc::new(Mutex::new(true)),
        };
        Arc::new(Mutex::new(tracker))
    }

    pub async fn start(tracker: Arc<Mutex<Self>>) {
        let is_running = {
            let t = tracker.lock().await;
            Arc::clone(&t.is_running)
        };

        tokio::spawn(async move {
            let spinner_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            let mut frame_idx = 0;

            while *is_running.lock().await {
                let (message, style, elapsed, current, total, sub_msgs) = {
                    let t = tracker.lock().await;
                    (
                        t.message.clone(),
                        match t.style {
                            ProgressStyle::Spinner => 0,
                            ProgressStyle::Bar => 1,
                            ProgressStyle::Steps => 2,
                            ProgressStyle::Dots => 3,
                        },
                        t.start_time.elapsed(),
                        t.current_step,
                        t.total_steps,
                        t.sub_messages.clone(),
                    )
                };

                // Clear current line
                print!("\r\x1B[K");

                match style {
                    0 => {
                        // Spinner
                        print!(
                            "{} {} {}",
                            spinner_frames[frame_idx].bright_cyan(),
                            message.white(),
                            format!("({}s)", elapsed.as_secs()).dimmed()
                        );
                        frame_idx = (frame_idx + 1) % spinner_frames.len();
                    }
                    1 => {
                        // Progress bar
                        if let Some(total) = total {
                            let percent = (current as f32 / total as f32 * 100.0) as u32;
                            let filled = (percent as usize * 30) / 100;
                            let bar = "█".repeat(filled) + &"░".repeat(30 - filled);

                            print!(
                                "{} {} {}% {}",
                                message.white(),
                                bar.bright_green(),
                                percent,
                                format!("({}/{})", current, total).dimmed()
                            );
                        }
                    }
                    2 => {
                        // Steps
                        if let Some(total) = total {
                            print!(
                                "{} {} {} {}",
                                "📍".bright_yellow(),
                                format!("[{}/{}]", current, total).bright_cyan(),
                                message.white(),
                                format!("({}s)", elapsed.as_secs()).dimmed()
                            );
                        }
                    }
                    3 => {
                        // Dots
                        let dots = ".".repeat((elapsed.as_secs() % 4) as usize);
                        print!(
                            "{} {}{}",
                            message.white(),
                            dots.bright_cyan(),
                            &"   ".to_string()[dots.len()..]
                        );
                    }
                    _ => {
                        // Default to spinner style
                        print!(
                            "{} {} {}",
                            spinner_frames[frame_idx].bright_cyan(),
                            message.white(),
                            format!("({}s)", elapsed.as_secs()).dimmed()
                        );
                        frame_idx = (frame_idx + 1) % spinner_frames.len();
                    }
                }

                // Show latest sub-message if any
                if !sub_msgs.is_empty() {
                    if let Some(last_msg) = sub_msgs.last() {
                        print!(" - {}", last_msg.dimmed());
                    }
                }

                let _ = io::stdout().flush();
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });
    }

    pub async fn update(&mut self, step: usize) {
        self.current_step = step;
    }

    pub async fn add_message(&mut self, msg: impl Into<String>) {
        self.sub_messages.push(msg.into());
        if self.sub_messages.len() > 5 {
            self.sub_messages.remove(0);
        }
    }

    pub async fn complete(tracker: Arc<Mutex<Self>>, success: bool, final_message: Option<String>) {
        let (message, elapsed, _is_running) = {
            let t = tracker.lock().await;
            *t.is_running.lock().await = false;
            (
                t.message.clone(),
                t.start_time.elapsed(),
                Arc::clone(&t.is_running),
            )
        };

        // Wait a bit for the spinner to stop
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Clear line and show final status
        print!("\r\x1B[K");

        if success {
            println!(
                "{} {} {}",
                "✅".bright_green(),
                message.white(),
                format!("(completed in {:.1}s)", elapsed.as_secs_f32()).bright_green()
            );

            if let Some(msg) = final_message {
                println!("   {}", msg.bright_white());
            }
        } else {
            println!(
                "{} {} {}",
                "❌".bright_red(),
                message.white(),
                format!("(failed after {:.1}s)", elapsed.as_secs_f32()).red()
            );

            if let Some(msg) = final_message {
                println!("   {}", msg.bright_red());
            }
        }
    }
}

/// Multi-step process tracker
pub struct ProcessTracker {
    title: String,
    steps: Vec<ProcessStep>,
    current_step: usize,
    start_time: Instant,
}

pub struct ProcessStep {
    pub name: String,
    pub status: StepStatus,
    pub duration: Option<Duration>,
    pub message: Option<String>,
}

#[derive(Clone, PartialEq)]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

impl ProcessTracker {
    pub fn new(title: impl Into<String>, steps: Vec<String>) -> Self {
        let process_steps = steps
            .into_iter()
            .map(|name| ProcessStep {
                name,
                status: StepStatus::Pending,
                duration: None,
                message: None,
            })
            .collect();

        Self {
            title: title.into(),
            steps: process_steps,
            current_step: 0,
            start_time: Instant::now(),
        }
    }

    pub fn display(&self) {
        println!();
        println!("🚀 {}", self.title.bright_cyan().bold());
        println!();

        for step in self.steps.iter() {
            let icon = match step.status {
                StepStatus::Pending => "○".dimmed(),
                StepStatus::Running => "◉".bright_yellow(),
                StepStatus::Completed => "✓".bright_green(),
                StepStatus::Failed => "✗".bright_red(),
                StepStatus::Skipped => "⊘".dimmed(),
            };

            let name = match step.status {
                StepStatus::Running => step.name.bright_yellow(),
                StepStatus::Completed => step.name.bright_green(),
                StepStatus::Failed => step.name.bright_red(),
                _ => step.name.normal(),
            };

            print!("  {} {}", icon, name);

            if let Some(duration) = step.duration {
                print!(" {}", format!("({:.1}s)", duration.as_secs_f32()).dimmed());
            }

            if let Some(msg) = &step.message {
                print!(" - {}", msg.dimmed());
            }

            println!();
        }

        let elapsed = self.start_time.elapsed();
        println!();
        println!(
            "  {} Elapsed: {:.1}s",
            "⏱".bright_cyan(),
            elapsed.as_secs_f32()
        );
    }

    pub fn start_step(&mut self, index: usize) {
        if index < self.steps.len() {
            self.steps[index].status = StepStatus::Running;
            self.current_step = index;
            self.display();
        }
    }

    pub fn complete_step(&mut self, index: usize, success: bool, message: Option<String>) {
        if index < self.steps.len() {
            let step = &mut self.steps[index];
            step.status = if success {
                StepStatus::Completed
            } else {
                StepStatus::Failed
            };
            step.duration = Some(self.start_time.elapsed());
            step.message = message;
            self.display();
        }
    }

    pub fn skip_step(&mut self, index: usize, reason: String) {
        if index < self.steps.len() {
            let step = &mut self.steps[index];
            step.status = StepStatus::Skipped;
            step.message = Some(reason);
            self.display();
        }
    }
}

/// Simple inline status updates
pub struct StatusLine;

impl StatusLine {
    pub fn update(message: impl AsRef<str>) {
        print!("\r\x1B[K{}", message.as_ref());
        let _ = io::stdout().flush();
    }

    pub fn complete() {
        print!("\r\x1B[K");
        let _ = io::stdout().flush();
    }
}

/// Animated waiting indicator for unknown duration operations
#[allow(dead_code)]
pub async fn wait_with_message(message: impl Into<String>) -> tokio::task::JoinHandle<()> {
    let msg = message.into();
    tokio::spawn(async move {
        let frames = ["   ", ".  ", ".. ", "..."];
        let mut i = 0;

        loop {
            print!("\r{} {}", msg, frames[i].bright_cyan());
            let _ = io::stdout().flush();
            i = (i + 1) % frames.len();
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    })
}
