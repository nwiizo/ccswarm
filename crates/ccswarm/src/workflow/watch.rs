//! Watch mode for Piece/Movement workflows.
//!
//! Monitors file system changes and automatically triggers workflow execution.
//! Inspired by takt's watch mode feature.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Configuration for watch mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchConfig {
    /// Directories to watch
    pub watch_paths: Vec<PathBuf>,
    /// File patterns to include (glob patterns)
    #[serde(default)]
    pub include_patterns: Vec<String>,
    /// File patterns to exclude (glob patterns)
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
    /// Debounce duration in milliseconds
    #[serde(default = "default_debounce")]
    pub debounce_ms: u64,
    /// Piece to execute when changes are detected
    pub piece_name: String,
    /// Whether to run the full piece or just validate
    #[serde(default = "default_true")]
    pub full_execution: bool,
    /// Whether to clear the terminal before each run
    #[serde(default)]
    pub clear_screen: bool,
    /// Maximum consecutive runs without user intervention
    #[serde(default = "default_max_consecutive")]
    pub max_consecutive_runs: u32,
}

fn default_debounce() -> u64 {
    500
}

fn default_true() -> bool {
    true
}

fn default_max_consecutive() -> u32 {
    10
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            watch_paths: vec![PathBuf::from(".")],
            include_patterns: vec!["**/*.rs".to_string(), "**/*.ts".to_string()],
            exclude_patterns: vec![
                "**/target/**".to_string(),
                "**/node_modules/**".to_string(),
                "**/.git/**".to_string(),
            ],
            debounce_ms: default_debounce(),
            piece_name: "default".to_string(),
            full_execution: true,
            clear_screen: false,
            max_consecutive_runs: default_max_consecutive(),
        }
    }
}

/// A detected file change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    /// Path of the changed file
    pub path: PathBuf,
    /// Type of change
    pub change_type: ChangeType,
    /// Timestamp of detection
    pub detected_at: std::time::SystemTime,
}

/// Type of file change
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    /// File was created
    Created,
    /// File was modified
    Modified,
    /// File was deleted
    Deleted,
}

/// State of the watch mode
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WatchState {
    /// Watching for changes
    Idle,
    /// Debouncing (waiting for changes to settle)
    Debouncing,
    /// Executing the workflow
    Executing,
    /// Paused (max runs reached or user paused)
    Paused,
    /// Stopped
    Stopped,
}

/// Watch mode controller (non-blocking, poll-based)
pub struct WatchController {
    config: WatchConfig,
    state: WatchState,
    /// Accumulated changes during debounce period
    pending_changes: Vec<FileChange>,
    /// Last execution time
    last_execution: Option<Instant>,
    /// Consecutive run count
    consecutive_runs: u32,
    /// Debounce start time
    debounce_start: Option<Instant>,
    /// Known file modification times for change detection
    known_files: std::collections::HashMap<PathBuf, std::time::SystemTime>,
}

impl WatchController {
    pub fn new(config: WatchConfig) -> Self {
        Self {
            config,
            state: WatchState::Idle,
            pending_changes: Vec::new(),
            last_execution: None,
            consecutive_runs: 0,
            debounce_start: None,
            known_files: std::collections::HashMap::new(),
        }
    }

    /// Get the current watch state
    pub fn state(&self) -> &WatchState {
        &self.state
    }

    /// Record a file change event
    pub fn record_change(&mut self, change: FileChange) {
        if self.state == WatchState::Stopped {
            return;
        }

        // Filter by include/exclude patterns
        let path_str = change.path.to_string_lossy().to_string();

        if !self.config.exclude_patterns.is_empty()
            && self
                .config
                .exclude_patterns
                .iter()
                .any(|p| simple_glob_match(p, &path_str))
        {
            debug!("Excluded change: {}", path_str);
            return;
        }

        if !self.config.include_patterns.is_empty()
            && !self
                .config
                .include_patterns
                .iter()
                .any(|p| simple_glob_match(p, &path_str))
        {
            debug!("Not included: {}", path_str);
            return;
        }

        info!(
            "Change detected: {:?} {:?}",
            change.change_type, change.path
        );
        self.pending_changes.push(change);

        // Start or reset debounce timer
        self.debounce_start = Some(Instant::now());
        self.state = WatchState::Debouncing;
    }

    /// Poll the controller to check if a workflow should be triggered.
    /// Returns Some(changes) if debounce period has elapsed and execution should start.
    pub fn poll(&mut self) -> Option<Vec<FileChange>> {
        if self.state != WatchState::Debouncing {
            return None;
        }

        if let Some(start) = self.debounce_start {
            let debounce = Duration::from_millis(self.config.debounce_ms);
            if start.elapsed() >= debounce {
                // Check max consecutive runs
                if self.consecutive_runs >= self.config.max_consecutive_runs {
                    warn!(
                        "Max consecutive runs ({}) reached, pausing watch",
                        self.config.max_consecutive_runs
                    );
                    self.state = WatchState::Paused;
                    return None;
                }

                // Drain pending changes and trigger
                let changes: Vec<FileChange> = self.pending_changes.drain(..).collect();
                self.state = WatchState::Executing;
                self.last_execution = Some(Instant::now());
                self.consecutive_runs += 1;
                self.debounce_start = None;

                return Some(changes);
            }
        }

        None
    }

    /// Mark execution as complete, return to idle
    pub fn execution_complete(&mut self) {
        self.state = WatchState::Idle;
    }

    /// Resume from paused state, reset consecutive counter
    pub fn resume(&mut self) {
        self.consecutive_runs = 0;
        self.state = WatchState::Idle;
        info!("Watch mode resumed");
    }

    /// Pause watching
    pub fn pause(&mut self) {
        self.state = WatchState::Paused;
    }

    /// Stop watching
    pub fn stop(&mut self) {
        self.state = WatchState::Stopped;
        self.pending_changes.clear();
    }

    /// Get summary of pending changes
    pub fn pending_summary(&self) -> WatchSummary {
        let unique_files: HashSet<&PathBuf> =
            self.pending_changes.iter().map(|c| &c.path).collect();

        WatchSummary {
            pending_count: self.pending_changes.len(),
            unique_files: unique_files.len(),
            consecutive_runs: self.consecutive_runs,
            state: self.state.clone(),
        }
    }

    /// Scan directories for changes (compare with known state)
    pub fn scan_for_changes(&mut self) -> Result<Vec<FileChange>> {
        let mut changes = Vec::new();

        let watch_paths = self.config.watch_paths.clone();
        for watch_path in &watch_paths {
            if !watch_path.exists() {
                continue;
            }
            self.scan_directory(watch_path, &mut changes)?;
        }

        // Record all detected changes
        for change in &changes {
            self.record_change(change.clone());
        }

        Ok(changes)
    }

    fn scan_directory(&mut self, dir: &Path, changes: &mut Vec<FileChange>) -> Result<()> {
        let entries = match std::fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(e) => {
                warn!("Cannot read directory {}: {}", dir.display(), e);
                return Ok(());
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                // Skip excluded directories
                let path_str = path.to_string_lossy().to_string();
                if self
                    .config
                    .exclude_patterns
                    .iter()
                    .any(|p| simple_glob_match(p, &path_str))
                {
                    continue;
                }
                self.scan_directory(&path, changes)?;
            } else if path.is_file()
                && let Ok(metadata) = std::fs::metadata(&path)
                && let Ok(modified) = metadata.modified()
            {
                match self.known_files.get(&path) {
                    Some(known_time) if *known_time != modified => {
                        changes.push(FileChange {
                            path: path.clone(),
                            change_type: ChangeType::Modified,
                            detected_at: std::time::SystemTime::now(),
                        });
                    }
                    None => {
                        changes.push(FileChange {
                            path: path.clone(),
                            change_type: ChangeType::Created,
                            detected_at: std::time::SystemTime::now(),
                        });
                    }
                    _ => {} // No change
                }
                self.known_files.insert(path, modified);
            }
        }

        Ok(())
    }
}

/// Summary of watch state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchSummary {
    /// Number of pending changes
    pub pending_count: usize,
    /// Number of unique files with changes
    pub unique_files: usize,
    /// Consecutive runs since last pause
    pub consecutive_runs: u32,
    /// Current state
    pub state: WatchState,
}

/// Simple glob-style match (supports ** and *)
fn simple_glob_match(pattern: &str, path: &str) -> bool {
    if pattern.contains("**") {
        // Split by ** and check that all literal segments appear in order
        let segments: Vec<&str> = pattern.split("**").collect();

        // Trim path separators from each segment
        let segments: Vec<&str> = segments.iter().map(|s| s.trim_matches('/')).collect();

        // For patterns like **/foo/** or **/*.rs
        // Check each non-empty segment appears in the path
        let mut remaining = path;
        for seg in &segments {
            if seg.is_empty() {
                continue;
            }
            // If segment contains *, match it as a simple glob against the filename
            if seg.contains('*') {
                let filename = path.rsplit('/').next().unwrap_or(path);
                if !simple_star_match(seg, filename) {
                    return false;
                }
            } else if let Some(pos) = remaining.find(seg) {
                remaining = &remaining[pos + seg.len()..];
            } else {
                return false;
            }
        }
        return true;
    }

    if pattern.contains('*') {
        return simple_star_match(pattern, path);
    }

    path == pattern || path.ends_with(pattern)
}

/// Match a pattern with single `*` wildcards (no `**`)
fn simple_star_match(pattern: &str, text: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 2 {
        return text.starts_with(parts[0]) && text.ends_with(parts[1]);
    }
    // Multiple single stars: check segments appear in order
    let mut remaining = text;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if i == 0 {
            if !remaining.starts_with(part) {
                return false;
            }
            remaining = &remaining[part.len()..];
        } else if i == parts.len() - 1 {
            if !remaining.ends_with(part) {
                return false;
            }
        } else if let Some(pos) = remaining.find(part) {
            remaining = &remaining[pos + part.len()..];
        } else {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watch_config_default() {
        let config = WatchConfig::default();
        assert_eq!(config.debounce_ms, 500);
        assert!(config.full_execution);
        assert_eq!(config.max_consecutive_runs, 10);
    }

    #[test]
    fn test_record_and_poll() {
        let config = WatchConfig {
            debounce_ms: 0, // Immediate
            ..WatchConfig::default()
        };
        let mut controller = WatchController::new(config);

        controller.record_change(FileChange {
            path: PathBuf::from("src/main.rs"),
            change_type: ChangeType::Modified,
            detected_at: std::time::SystemTime::now(),
        });

        assert_eq!(*controller.state(), WatchState::Debouncing);

        // Poll should return changes since debounce is 0
        std::thread::sleep(Duration::from_millis(1));
        let changes = controller.poll();
        assert!(changes.is_some());
        assert_eq!(changes.unwrap().len(), 1);
        assert_eq!(*controller.state(), WatchState::Executing);
    }

    #[test]
    fn test_max_consecutive_runs() {
        let config = WatchConfig {
            debounce_ms: 0,
            max_consecutive_runs: 2,
            ..WatchConfig::default()
        };
        let mut controller = WatchController::new(config);

        // Run 1
        controller.record_change(FileChange {
            path: PathBuf::from("a.rs"),
            change_type: ChangeType::Modified,
            detected_at: std::time::SystemTime::now(),
        });
        std::thread::sleep(Duration::from_millis(1));
        assert!(controller.poll().is_some());
        controller.execution_complete();

        // Run 2
        controller.record_change(FileChange {
            path: PathBuf::from("b.rs"),
            change_type: ChangeType::Modified,
            detected_at: std::time::SystemTime::now(),
        });
        std::thread::sleep(Duration::from_millis(1));
        assert!(controller.poll().is_some());
        controller.execution_complete();

        // Run 3 should be paused
        controller.record_change(FileChange {
            path: PathBuf::from("c.rs"),
            change_type: ChangeType::Modified,
            detected_at: std::time::SystemTime::now(),
        });
        std::thread::sleep(Duration::from_millis(1));
        assert!(controller.poll().is_none());
        assert_eq!(*controller.state(), WatchState::Paused);
    }

    #[test]
    fn test_resume_after_pause() {
        let config = WatchConfig {
            debounce_ms: 0,
            max_consecutive_runs: 1,
            ..WatchConfig::default()
        };
        let mut controller = WatchController::new(config);

        controller.record_change(FileChange {
            path: PathBuf::from("a.rs"),
            change_type: ChangeType::Modified,
            detected_at: std::time::SystemTime::now(),
        });
        std::thread::sleep(Duration::from_millis(1));
        controller.poll();
        controller.execution_complete();

        controller.record_change(FileChange {
            path: PathBuf::from("b.rs"),
            change_type: ChangeType::Modified,
            detected_at: std::time::SystemTime::now(),
        });
        std::thread::sleep(Duration::from_millis(1));
        controller.poll(); // Paused

        controller.resume();
        assert_eq!(*controller.state(), WatchState::Idle);
        assert_eq!(controller.consecutive_runs, 0);
    }

    #[test]
    fn test_stop() {
        let mut controller = WatchController::new(WatchConfig::default());
        controller.stop();
        assert_eq!(*controller.state(), WatchState::Stopped);

        // Changes after stop should be ignored
        controller.record_change(FileChange {
            path: PathBuf::from("a.rs"),
            change_type: ChangeType::Modified,
            detected_at: std::time::SystemTime::now(),
        });
        assert_eq!(*controller.state(), WatchState::Stopped);
    }

    #[test]
    fn test_exclude_patterns() {
        let config = WatchConfig {
            debounce_ms: 0,
            exclude_patterns: vec!["**/target/**".to_string()],
            include_patterns: vec![], // Allow all
            ..WatchConfig::default()
        };
        let mut controller = WatchController::new(config);

        controller.record_change(FileChange {
            path: PathBuf::from("target/debug/main"),
            change_type: ChangeType::Modified,
            detected_at: std::time::SystemTime::now(),
        });

        // Should not be recorded (excluded)
        assert_eq!(controller.pending_changes.len(), 0);
    }

    #[test]
    fn test_glob_match() {
        assert!(simple_glob_match("**/*.rs", "src/main.rs"));
        assert!(simple_glob_match("**/*.rs", "deep/nested/file.rs"));
        assert!(!simple_glob_match("**/*.rs", "src/main.ts"));
        assert!(simple_glob_match("**/target/**", "target/debug/build"));
        assert!(simple_glob_match("*.txt", "readme.txt"));
    }

    #[test]
    fn test_pending_summary() {
        let config = WatchConfig {
            debounce_ms: 10000, // Long debounce so changes stay pending
            include_patterns: vec![],
            exclude_patterns: vec![],
            ..WatchConfig::default()
        };
        let mut controller = WatchController::new(config);

        controller.record_change(FileChange {
            path: PathBuf::from("a.rs"),
            change_type: ChangeType::Modified,
            detected_at: std::time::SystemTime::now(),
        });
        controller.record_change(FileChange {
            path: PathBuf::from("a.rs"),
            change_type: ChangeType::Modified,
            detected_at: std::time::SystemTime::now(),
        });
        controller.record_change(FileChange {
            path: PathBuf::from("b.rs"),
            change_type: ChangeType::Created,
            detected_at: std::time::SystemTime::now(),
        });

        let summary = controller.pending_summary();
        assert_eq!(summary.pending_count, 3);
        assert_eq!(summary.unique_files, 2);
    }
}
