//! Example demonstrating the refactored error diagram template engine

use ccswarm::utils::error_diagrams::{show_diagram, ErrorDiagrams};

fn main() {
    println!("=== Error Diagram Template Engine Demo ===\n");

    // Network Error
    println!("1. Network Connection Error:");
    println!("   Shows connection flow from Computer → Network → API");
    show_diagram(ErrorDiagrams::network_error());

    // Session Lifecycle
    println!("\n2. Session Lifecycle:");
    println!("   Shows session states: Created → Active → Idle → Terminated");
    show_diagram(ErrorDiagrams::session_error());

    // Git Worktree
    println!("\n3. Git Worktree Structure:");
    println!("   Shows how agents work in isolated git worktrees");
    show_diagram(ErrorDiagrams::git_worktree_error());

    // Permissions
    println!("\n4. File Permissions:");
    println!("   Shows Unix file permission structure");
    show_diagram(ErrorDiagrams::permission_error());

    // Configuration
    println!("\n5. Configuration File:");
    println!("   Shows ccswarm.json structure");
    show_diagram(ErrorDiagrams::config_error());

    // Task Flow
    println!("\n6. Task Processing:");
    println!("   Shows task flow: Input → Parse → Assign → Execute");
    show_diagram(ErrorDiagrams::task_error());

    // API Key Setup
    println!("\n7. API Key Configuration:");
    println!("   Shows step-by-step API key setup");
    show_diagram(ErrorDiagrams::api_key_error());

    // Agent Communication
    println!("\n8. Agent Communication:");
    println!("   Shows Master Claude delegating to agents");
    show_diagram(ErrorDiagrams::agent_error());

    println!("\n=== Template Engine Benefits ===");
    println!("✓ Consistent box drawing characters across all diagrams");
    println!("✓ Reusable components (arrows, boxes, status symbols)");
    println!("✓ Centralized color management");
    println!("✓ Easy to add new diagram types");
    println!("✓ Maintainable and extensible");
}
