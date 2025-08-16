/// Common CLI handler pattern to eliminate duplication
use anyhow::Result;
use colored::*;
use std::future::Future;

/// Generic CLI command handler
pub struct CommandHandler;

impl CommandHandler {
    /// Execute a generic CLI command with consistent error handling and output
    pub async fn execute<F, Fut, T>(command_name: &str, operation: F) -> Result<()>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T>>,
        T: std::fmt::Display,
    {
        println!(
            "{}",
            format!("🚀 Executing {}...", command_name)
                .bright_blue()
                .bold()
        );

        match operation().await {
            Ok(result) => {
                println!(
                    "{}",
                    format!("✅ {} completed successfully", command_name).bright_green()
                );
                println!("{}", result);
                Ok(())
            }
            Err(e) => {
                eprintln!(
                    "{}",
                    format!("❌ {} failed: {}", command_name, e).bright_red()
                );
                Err(e)
            }
        }
    }

    /// Execute a batch operation with progress tracking
    pub async fn execute_batch<F, Fut, T>(
        command_name: &str,
        items: Vec<String>,
        operation: F,
    ) -> Result<Vec<T>>
    where
        F: Fn(String) -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        println!(
            "{}",
            format!("🔄 Processing {} items for {}", items.len(), command_name).bright_blue()
        );

        let mut results = Vec::new();
        let mut errors = 0;
        let total_items = items.len();

        for (idx, item) in items.into_iter().enumerate() {
            print!(
                "  [{}/{}] Processing {}... ",
                idx + 1,
                total_items + errors,
                item
            );

            match operation(item).await {
                Ok(result) => {
                    println!("{}", "✓".bright_green());
                    results.push(result);
                }
                Err(e) => {
                    println!("{} - {}", "✗".bright_red(), e);
                    errors += 1;
                }
            }
        }

        if errors > 0 {
            eprintln!(
                "{}",
                format!("⚠️  {} completed with {} errors", command_name, errors).yellow()
            );
        } else {
            println!(
                "{}",
                format!("✅ {} completed successfully", command_name).bright_green()
            );
        }

        Ok(results)
    }

    /// Execute a status display operation
    pub async fn show_status<F, Fut>(entity_name: &str, fetcher: F) -> Result<()>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Box<dyn StatusInfo>>>,
    {
        println!(
            "{}",
            format!("📊 {} Status", entity_name).bright_cyan().bold()
        );

        match fetcher().await {
            Ok(status) => {
                status.display();
                Ok(())
            }
            Err(e) => {
                eprintln!(
                    "{}",
                    format!("Failed to get {} status: {}", entity_name, e).red()
                );
                Err(e)
            }
        }
    }

    /// Execute a list operation
    pub async fn list_items<F, Fut, T>(
        entity_name: &str,
        filter: Option<String>,
        fetcher: F,
    ) -> Result<()>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Vec<T>>>,
        T: std::fmt::Display,
    {
        let title = if let Some(ref f) = filter {
            format!("📋 {} (filtered: {})", entity_name, f)
        } else {
            format!("📋 All {}", entity_name)
        };

        println!("{}", title.bright_cyan().bold());

        match fetcher().await {
            Ok(items) => {
                if items.is_empty() {
                    println!("  No {} found", entity_name.to_lowercase());
                } else {
                    for (idx, item) in items.iter().enumerate() {
                        println!("  {}. {}", idx + 1, item);
                    }
                    println!("\nTotal: {} {}", items.len(), entity_name.to_lowercase());
                }
                Ok(())
            }
            Err(e) => {
                eprintln!("{}", format!("Failed to list {}: {}", entity_name, e).red());
                Err(e)
            }
        }
    }
}

/// Status information trait
pub trait StatusInfo {
    fn display(&self);
}

/// Default status implementation
pub struct DefaultStatus {
    pub fields: Vec<(String, String)>,
}

impl StatusInfo for DefaultStatus {
    fn display(&self) {
        for (key, value) in &self.fields {
            println!("  {}: {}", key.bright_white(), value);
        }
    }
}

/// Builder for creating standardized CLI responses
pub struct ResponseBuilder {
    sections: Vec<(String, Vec<String>)>,
}

impl Default for ResponseBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ResponseBuilder {
    pub fn new() -> Self {
        Self {
            sections: Vec::new(),
        }
    }

    pub fn add_section(mut self, title: &str, items: Vec<String>) -> Self {
        self.sections.push((title.to_string(), items));
        self
    }

    pub fn build(self) -> String {
        let mut output = String::new();

        for (title, items) in self.sections {
            output.push_str(&format!("\n{}\n", title.bright_cyan().bold()));

            if items.is_empty() {
                output.push_str("  (none)\n");
            } else {
                for item in items {
                    output.push_str(&format!("  • {}\n", item));
                }
            }
        }

        output
    }
}
