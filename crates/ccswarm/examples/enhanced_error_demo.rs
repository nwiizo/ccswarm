//! Demonstration of enhanced error visualization

use ccswarm::utils::{show_diagram, CommonErrors, ErrorDiagrams};
use colored::Colorize;

#[tokio::main]
async fn main() {
    println!(
        "\n{}",
        "🎨 Enhanced Error Visualization Demo".bright_cyan().bold()
    );
    println!("{}", "====================================".bright_cyan());
    println!();

    // Demo 1: API Key Error with Diagram
    println!("1️⃣  {}", "API Key Missing Error:".bright_yellow());
    println!("{}", "─".repeat(50).dimmed());
    CommonErrors::api_key_missing("Anthropic").display();

    println!("\nPress Enter to continue...");
    let _ = std::io::stdin().read_line(&mut String::new());

    // Demo 2: Session Error with Lifecycle Diagram
    println!("\n2️⃣  {}", "Session Not Found Error:".bright_yellow());
    println!("{}", "─".repeat(50).dimmed());
    CommonErrors::session_not_found("agent-123").display();

    println!("\nPress Enter to continue...");
    let _ = std::io::stdin().read_line(&mut String::new());

    // Demo 3: Git Worktree Error
    println!("\n3️⃣  {}", "Git Worktree Conflict:".bright_yellow());
    println!("{}", "─".repeat(50).dimmed());
    CommonErrors::worktree_conflict("feature/auth").display();

    println!("\nPress Enter to continue...");
    let _ = std::io::stdin().read_line(&mut String::new());

    // Demo 4: Network Error
    println!("\n4️⃣  {}", "Network Connection Error:".bright_yellow());
    println!("{}", "─".repeat(50).dimmed());
    CommonErrors::network_error("https://api.anthropic.com").display();

    println!("\nPress Enter to continue...");
    let _ = std::io::stdin().read_line(&mut String::new());

    // Demo 5: Permission Error
    println!("\n5️⃣  {}", "Permission Denied Error:".bright_yellow());
    println!("{}", "─".repeat(50).dimmed());
    CommonErrors::permission_denied("/etc/sensitive-file").display();

    println!("\nPress Enter to continue...");
    let _ = std::io::stdin().read_line(&mut String::new());

    // Demo 6: Task Format Error
    println!("\n6️⃣  {}", "Invalid Task Format:".bright_yellow());
    println!("{}", "─".repeat(50).dimmed());
    CommonErrors::invalid_task_format().display();

    println!("\nPress Enter to continue...");
    let _ = std::io::stdin().read_line(&mut String::new());

    // Demo 7: Agent Communication Error
    println!("\n7️⃣  {}", "Agent Busy Error:".bright_yellow());
    println!("{}", "─".repeat(50).dimmed());
    CommonErrors::agent_busy("Frontend").display();

    println!("\nPress Enter to continue...");
    let _ = std::io::stdin().read_line(&mut String::new());

    // Demo 8: Configuration Error
    println!("\n8️⃣  {}", "Configuration Not Found:".bright_yellow());
    println!("{}", "─".repeat(50).dimmed());
    CommonErrors::config_not_found().display();

    println!();
    println!("{}", "✨ Demo Complete!".bright_green().bold());
    println!();
    println!("{}:", "Interactive Features".bright_white());
    println!("  • Visual diagrams explain error context");
    println!("  • Step-by-step recovery instructions");
    println!("  • Auto-fix available for common errors");
    println!("  • Error codes for quick diagnosis");
    println!();
    println!("{}:", "Try these commands".bright_white());
    println!(
        "  {} - Diagnose specific error",
        "ccswarm doctor --error ENV001".bright_cyan()
    );
    println!(
        "  {} - Auto-fix errors",
        "ccswarm doctor --fix".bright_cyan()
    );
    println!(
        "  {} - Check API connectivity",
        "ccswarm doctor --check-api".bright_cyan()
    );
    println!(
        "  {} - Fix errors globally",
        "ccswarm <command> --fix".bright_cyan()
    );
}
