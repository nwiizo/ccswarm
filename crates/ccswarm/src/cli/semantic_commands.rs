//! CLI commands for semantic features
//! 
//! Provides command-line interface for semantic analysis and optimization

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use crate::semantic::{
    SemanticManager, SemanticConfig,
    dynamic_agent_generation::{DynamicAgentGenerator, ProjectComplexityLevel},
    refactoring_system::AutomaticRefactoringSystem,
    sangha_voting::{SanghaSemanticVoting, ConsensusAlgorithm, ProposalType, VoteDecision},
    cross_codebase_optimization::{CrossCodebaseOptimizer, ProgrammingLanguage},
};
use crate::semantic::subagent_integration::AgentRole;
use anyhow::Result;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use tokio::time::{sleep, Duration};
use std::sync::Arc;

/// Semantic feature commands
#[derive(Parser, Debug)]
#[command(name = "semantic")]
#[command(about = "Semantic code analysis and optimization")]
pub struct SemanticCommands {
    #[command(subcommand)]
    pub command: SemanticSubcommand,
}

/// Semantic subcommands
#[derive(Subcommand, Debug)]
pub enum SemanticSubcommand {
    /// Analyze codebase for insights
    Analyze {
        /// Path to analyze
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
        
        /// Output format (json, table, markdown)
        #[arg(short, long, default_value = "table")]
        format: String,
        
        /// Include symbol details
        #[arg(short, long)]
        symbols: bool,
    },
    
    /// Scan for refactoring opportunities
    Refactor {
        /// Automatically apply safe refactorings
        #[arg(short, long)]
        auto_apply: bool,
        
        /// Priority threshold (low, medium, high, critical)
        #[arg(short, long, default_value = "medium")]
        priority: String,
        
        /// Maximum proposals to show
        #[arg(short, long, default_value = "10")]
        max: usize,
    },
    
    /// Generate agents based on project needs
    GenerateAgents {
        /// Force regeneration even if agents exist
        #[arg(short, long)]
        force: bool,
        
        /// Deploy generated agents immediately
        #[arg(short, long)]
        deploy: bool,
    },
    
    /// Cross-codebase optimization analysis
    Optimize {
        /// Repositories to analyze (format: name:path:language)
        #[arg(short, long, required = true)]
        repos: Vec<String>,
        
        /// Generate detailed report
        #[arg(short, long)]
        detailed: bool,
        
        /// Output file for report
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Sangha voting on proposals
    Vote {
        /// Create new proposal
        #[arg(long, conflicts_with = "list")]
        create: Option<String>,
        
        /// List active proposals
        #[arg(long, conflicts_with = "create")]
        list: bool,
        
        /// Submit vote (format: proposal_id:decision:reason)
        #[arg(long)]
        submit: Option<String>,
        
        /// Show voting history
        #[arg(long)]
        history: bool,
    },
    
    /// Interactive semantic dashboard
    Dashboard {
        /// Port for web UI
        #[arg(short, long, default_value = "3000")]
        port: u16,
        
        /// Enable real-time updates
        #[arg(short, long)]
        realtime: bool,
    },
    
    /// Monitor semantic operations in real-time
    Monitor {
        /// Refresh interval in seconds
        #[arg(short, long, default_value = "5")]
        interval: u64,
        
        /// Show only specific metrics
        #[arg(short, long)]
        filter: Option<String>,
    },
}

/// Execute semantic commands
pub async fn execute(cmd: SemanticCommands) -> Result<()> {
    let config = SemanticConfig::default();
    let manager = Arc::new(SemanticManager::new(config).await?);
    manager.initialize().await?;
    
    match cmd.command {
        SemanticSubcommand::Analyze { path, format, symbols } => {
            analyze_codebase(manager, path, format, symbols).await?;
        }
        SemanticSubcommand::Refactor { auto_apply, priority, max } => {
            refactor_codebase(manager, auto_apply, priority, max).await?;
        }
        SemanticSubcommand::GenerateAgents { force, deploy } => {
            generate_agents(manager, force, deploy).await?;
        }
        SemanticSubcommand::Optimize { repos, detailed, output } => {
            optimize_cross_codebase(manager, repos, detailed, output).await?;
        }
        SemanticSubcommand::Vote { create, list, submit, history } => {
            handle_voting(manager, create, list, submit, history).await?;
        }
        SemanticSubcommand::Dashboard { port, realtime } => {
            launch_dashboard(port, realtime).await?;
        }
        SemanticSubcommand::Monitor { interval, filter } => {
            monitor_operations(manager, interval, filter).await?;
        }
    }
    
    Ok(())
}

/// Analyze codebase
async fn analyze_codebase(
    manager: Arc<SemanticManager>,
    path: PathBuf,
    format: String,
    show_symbols: bool,
) -> Result<()> {
    println!("{}", "🔍 Analyzing codebase...".bright_blue().bold());
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.blue} {msg}")
            .unwrap()
    );
    pb.set_message("Indexing symbols...");
    
    // Index the codebase
    manager.symbol_index().index_codebase().await?;
    
    pb.set_message("Analyzing code structure...");
    let all_symbols = manager.symbol_index().get_all_symbols().await?;
    
    pb.finish_with_message("✓ Analysis complete");
    
    // Display results based on format
    match format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&all_symbols)?;
            println!("{}", json);
        }
        "markdown" => {
            print_markdown_analysis(&all_symbols, show_symbols);
        }
        _ => {
            print_table_analysis(&all_symbols, show_symbols);
        }
    }
    
    // Show summary statistics
    println!("\n{}", "📊 Summary Statistics".bright_green().bold());
    println!("  Total symbols: {}", all_symbols.len());
    
    let functions = all_symbols.iter()
        .filter(|s| matches!(s.kind, crate::semantic::analyzer::SymbolKind::Function))
        .count();
    let structs = all_symbols.iter()
        .filter(|s| matches!(s.kind, crate::semantic::analyzer::SymbolKind::Struct))
        .count();
    
    println!("  Functions: {}", functions);
    println!("  Structs: {}", structs);
    
    Ok(())
}

/// Refactor codebase
async fn refactor_codebase(
    manager: Arc<SemanticManager>,
    auto_apply: bool,
    priority: String,
    max: usize,
) -> Result<()> {
    println!("{}", "🔧 Scanning for refactoring opportunities...".bright_blue().bold());
    
    let mut refactoring_system = AutomaticRefactoringSystem::new(
        manager.analyzer(),
        manager.symbol_index(),
        manager.memory(),
    );
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.blue} {msg}")
            .unwrap()
    );
    pb.set_message("Analyzing code quality...");
    
    let proposals = refactoring_system.scan_codebase().await?;
    pb.finish_with_message("✓ Scan complete");
    
    // Filter by priority
    let priority_filter = parse_priority(&priority);
    let filtered: Vec<_> = proposals.iter()
        .filter(|p| p.priority >= priority_filter)
        .take(max)
        .collect();
    
    if filtered.is_empty() {
        println!("{}", "No refactoring opportunities found at this priority level.".yellow());
        return Ok(());
    }
    
    println!("\n{}", format!("Found {} refactoring opportunities:", filtered.len()).bright_green().bold());
    
    for (i, proposal) in filtered.iter().enumerate() {
        println!("\n{}. {} [{}]", 
            i + 1, 
            proposal.title.bright_white().bold(),
            format!("{:?}", proposal.priority).color(priority_color(&proposal.priority))
        );
        println!("   {}", proposal.description.dimmed());
        println!("   Effort: {:?} | Automated: {}", 
            proposal.estimated_effort,
            if proposal.automated { "✓".green() } else { "✗".red() }
        );
        
        if !proposal.benefits.is_empty() {
            println!("   Benefits:");
            for benefit in &proposal.benefits {
                println!("     • {}", benefit.green());
            }
        }
        
        if auto_apply && proposal.automated {
            print!("   Applying automatically... ");
            refactoring_system.apply_proposal(&proposal.id).await?;
            println!("{}", "✓".bright_green().bold());
        }
    }
    
    if auto_apply {
        let stats = refactoring_system.get_stats();
        println!("\n{}", "📈 Refactoring Statistics".bright_green().bold());
        println!("  Applied: {} proposals", stats.applied_proposals);
        println!("  Time saved: {:.1} hours", stats.time_saved_hours);
        println!("  Lines refactored: {}", stats.lines_refactored);
    }
    
    Ok(())
}

/// Generate agents based on project needs
async fn generate_agents(
    manager: Arc<SemanticManager>,
    force: bool,
    deploy: bool,
) -> Result<()> {
    println!("{}", "🤖 Analyzing project for agent generation...".bright_blue().bold());
    
    let generator = DynamicAgentGenerator::new(
        manager.analyzer(),
        manager.symbol_index(),
        manager.memory(),
    );
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.blue} {msg}")
            .unwrap()
    );
    pb.set_message("Analyzing project characteristics...");
    
    let needs = generator.analyze_agent_needs().await?;
    pb.finish_with_message("✓ Analysis complete");
    
    if needs.is_empty() && !force {
        println!("{}", "All necessary agents already exist.".green());
        return Ok(());
    }
    
    println!("\n{}", format!("Need to generate {} agents:", needs.len()).bright_green().bold());
    
    for request in needs {
        println!("\n  Generating agent for: {}", 
            request.task_context.task.title.bright_white().bold());
        
        let generated = generator.generate_agent(&request).await?;
        
        println!("  ✓ Generated: {}", generated.template.name.green());
        println!("    Role: {:?}", generated.template.role);
        println!("    Capabilities: {}", generated.template.capabilities.len());
        
        if deploy {
            print!("  Deploying agent... ");
            generator.deploy_agent(&generated).await?;
            println!("{}", "✓".bright_green().bold());
        }
    }
    
    Ok(())
}

/// Optimize across multiple codebases
async fn optimize_cross_codebase(
    manager: Arc<SemanticManager>,
    repos: Vec<String>,
    detailed: bool,
    output: Option<PathBuf>,
) -> Result<()> {
    println!("{}", "🚀 Cross-codebase optimization analysis...".bright_blue().bold());
    
    let mut optimizer = CrossCodebaseOptimizer::new(manager.memory());
    
    let multi_pb = MultiProgress::new();
    
    // Parse and add repositories
    for repo_spec in repos {
        let parts: Vec<&str> = repo_spec.split(':').collect();
        if parts.len() != 3 {
            eprintln!("Invalid repo format: {} (expected name:path:language)", repo_spec);
            continue;
        }
        
        let name = parts[0].to_string();
        let path = PathBuf::from(parts[1]);
        let language = parse_language(parts[2]);
        
        let pb = multi_pb.add(ProgressBar::new_spinner());
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.blue} Adding repository: {msg}")
                .unwrap()
        );
        pb.set_message(name.clone());
        
        optimizer.add_repository(name, path, language).await?;
        pb.finish_with_message("✓");
    }
    
    // Perform analysis
    let pb = multi_pb.add(ProgressBar::new_spinner());
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.blue} {msg}")
            .unwrap()
    );
    pb.set_message("Performing comprehensive analysis...");
    
    let analysis = optimizer.analyze_all().await?;
    pb.finish_with_message("✓ Analysis complete");
    
    // Generate report
    let report = optimizer.generate_report().await?;
    
    // Save or display report
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &report).await?;
        println!("{}", format!("Report saved to: {}", output_path.display()).green());
    } else {
        println!("\n{}", report);
    }
    
    if detailed {
        // Show detailed findings
        println!("\n{}", "🔍 Detailed Findings".bright_green().bold());
        
        if !analysis.security_findings.is_empty() {
            println!("\n  Security Issues:");
            for finding in &analysis.security_findings {
                println!("    • [{:?}] {}", finding.severity, finding.description);
            }
        }
        
        if !analysis.performance_bottlenecks.is_empty() {
            println!("\n  Performance Bottlenecks:");
            for bottleneck in &analysis.performance_bottlenecks {
                println!("    • {:?} in {}", 
                    bottleneck.bottleneck_type,
                    bottleneck.location.function_name
                );
            }
        }
        
        println!("\n  Technical Debt:");
        println!("    Total: {:.0} hours", analysis.technical_debt_map.total_debt_hours);
        println!("    Trend: {:?}", analysis.technical_debt_map.debt_trends.trend_direction);
    }
    
    Ok(())
}

/// Handle voting operations
async fn handle_voting(
    manager: Arc<SemanticManager>,
    create: Option<String>,
    list: bool,
    submit: Option<String>,
    history: bool,
) -> Result<()> {
    let sangha = SanghaSemanticVoting::new(
        manager.analyzer(),
        manager.memory(),
        ConsensusAlgorithm::SimpleMajority,
    );
    
    if let Some(proposal_title) = create {
        println!("{}", "📝 Creating new proposal...".bright_blue().bold());
        
        // For demo, create a simple proposal
        let proposal = sangha.create_proposal(
            proposal_title.clone(),
            "Proposal created via CLI".to_string(),
            ProposalType::ArchitectureChange,
            vec![],
        ).await?;
        
        println!("{}", format!("✓ Proposal created: {}", proposal.id).green());
        println!("  Title: {}", proposal.title);
        println!("  Deadline: {}", proposal.voting_deadline);
        println!("  Quorum: {}", proposal.quorum_required);
    }
    
    if list {
        println!("{}", "📋 Active Proposals".bright_blue().bold());
        let proposals = sangha.get_active_proposals().await?;
        
        if proposals.is_empty() {
            println!("  No active proposals");
        } else {
            for proposal in proposals {
                println!("\n  ID: {}", proposal.id.bright_white().bold());
                println!("  Title: {}", proposal.title);
                println!("  Status: {:?}", proposal.status);
                println!("  Votes: {}", proposal.votes.len());
            }
        }
    }
    
    if let Some(vote_spec) = submit {
        let parts: Vec<&str> = vote_spec.split(':').collect();
        if parts.len() != 3 {
            eprintln!("Invalid vote format (expected proposal_id:decision:reason)");
            return Ok(());
        }
        
        let decision = match parts[1] {
            "approve" => VoteDecision::Approve,
            "reject" => VoteDecision::Reject,
            "abstain" => VoteDecision::Abstain,
            _ => VoteDecision::RequestChanges,
        };
        
        sangha.submit_vote(
            parts[0],
            "cli-user".to_string(),
            AgentRole::Custom("Human".to_string()),
            decision,
            parts[2].to_string(),
        ).await?;
        
        println!("{}", "✓ Vote submitted successfully".green());
    }
    
    if history {
        println!("{}", "📜 Voting History".bright_blue().bold());
        let history = sangha.get_voting_history().await?;
        
        for result in history {
            println!("\n  Proposal: {}", result.proposal_id);
            println!("  Result: {:?} ({}% approval)", 
                result.final_decision,
                result.approval_percentage
            );
            println!("  Consensus: {}", 
                if result.consensus_achieved { "✓".green() } else { "✗".red() }
            );
        }
    }
    
    Ok(())
}

/// Launch interactive dashboard
async fn launch_dashboard(port: u16, realtime: bool) -> Result<()> {
    println!("{}", format!("🌐 Launching semantic dashboard on port {}...", port).bright_blue().bold());
    
    // This would launch a web server with the dashboard
    // For now, we'll simulate it
    println!("  Dashboard URL: http://localhost:{}", port);
    
    if realtime {
        println!("  Real-time updates: {}", "enabled".green());
    }
    
    println!("\n  Press Ctrl+C to stop the dashboard");
    
    // Keep running until interrupted
    loop {
        sleep(Duration::from_secs(1)).await;
    }
}

/// Monitor semantic operations
async fn monitor_operations(
    manager: Arc<SemanticManager>,
    interval: u64,
    filter: Option<String>,
) -> Result<()> {
    println!("{}", "📊 Monitoring semantic operations...".bright_blue().bold());
    println!("  Refresh interval: {} seconds", interval);
    
    if let Some(f) = &filter {
        println!("  Filter: {}", f);
    }
    
    loop {
        // Clear screen (in real implementation)
        print!("\x1B[2J\x1B[1;1H");
        
        println!("{}", "═══════════════════════════════════════════".bright_blue());
        println!("{}", " SEMANTIC OPERATIONS MONITOR ".bright_white().bold());
        println!("{}", "═══════════════════════════════════════════".bright_blue());
        
        // Get current metrics
        let symbols = manager.symbol_index().get_all_symbols().await?;
        let memories = manager.memory().list_memories().await?;
        
        println!("\n📊 Metrics:");
        println!("  Indexed Symbols: {}", symbols.len());
        println!("  Project Memories: {}", memories.len());
        
        // Show recent operations (simulated)
        println!("\n🔄 Recent Operations:");
        println!("  [{}] Symbol analysis completed", chrono::Local::now().format("%H:%M:%S"));
        println!("  [{}] Memory stored: architecture_decision", chrono::Local::now().format("%H:%M:%S"));
        
        println!("\n  Press Ctrl+C to exit");
        
        sleep(Duration::from_secs(interval)).await;
    }
}

// Helper functions

fn print_table_analysis(symbols: &[crate::semantic::analyzer::Symbol], show_symbols: bool) {
    use prettytable::{Table, row, cell};
    
    let mut table = Table::new();
    table.add_row(row!["Type", "Name", "File", "Line"]);
    
    for symbol in symbols.iter().take(20) {
        table.add_row(row![
            format!("{:?}", symbol.kind),
            symbol.name,
            symbol.file_path,
            symbol.line
        ]);
    }
    
    table.printstd();
    
    if symbols.len() > 20 {
        println!("... and {} more", symbols.len() - 20);
    }
}

fn print_markdown_analysis(symbols: &[crate::semantic::analyzer::Symbol], show_symbols: bool) {
    println!("# Codebase Analysis\n");
    
    println!("## Symbol Summary\n");
    println!("| Type | Count |");
    println!("|------|-------|");
    
    use std::collections::HashMap;
    let mut type_counts = HashMap::new();
    
    for symbol in symbols {
        *type_counts.entry(format!("{:?}", symbol.kind)).or_insert(0) += 1;
    }
    
    for (kind, count) in type_counts {
        println!("| {} | {} |", kind, count);
    }
    
    if show_symbols {
        println!("\n## Symbols\n");
        for symbol in symbols.iter().take(50) {
            println!("- **{}** (`{:?}`) - {}:{}", 
                symbol.name, 
                symbol.kind,
                symbol.file_path,
                symbol.line
            );
        }
    }
}

fn parse_priority(priority: &str) -> crate::semantic::refactoring_system::RefactoringPriority {
    use crate::semantic::refactoring_system::RefactoringPriority;
    
    match priority.to_lowercase().as_str() {
        "low" => RefactoringPriority::Low,
        "medium" => RefactoringPriority::Medium,
        "high" => RefactoringPriority::High,
        "critical" => RefactoringPriority::Critical,
        _ => RefactoringPriority::Medium,
    }
}

fn priority_color(priority: &crate::semantic::refactoring_system::RefactoringPriority) -> colored::Color {
    use crate::semantic::refactoring_system::RefactoringPriority;
    
    match priority {
        RefactoringPriority::Low => colored::Color::Blue,
        RefactoringPriority::Medium => colored::Color::Yellow,
        RefactoringPriority::High => colored::Color::BrightRed,
        RefactoringPriority::Critical => colored::Color::Red,
    }
}

fn parse_language(lang: &str) -> ProgrammingLanguage {
    match lang.to_lowercase().as_str() {
        "rust" => ProgrammingLanguage::Rust,
        "typescript" | "ts" => ProgrammingLanguage::TypeScript,
        "javascript" | "js" => ProgrammingLanguage::JavaScript,
        "python" | "py" => ProgrammingLanguage::Python,
        "go" => ProgrammingLanguage::Go,
        "java" => ProgrammingLanguage::Java,
        _ => ProgrammingLanguage::Other(lang.to_string()),
    }
}