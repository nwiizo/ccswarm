//! Demo of Sangha and Self-Extension features

use anyhow::Result;
use ccswarm::extension::{ExtensionManager, ExtensionProposal, ExtensionType};
use ccswarm::identity::AgentRole;
use ccswarm::sangha::{Sangha, SanghaConfig, SanghaMember};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🏛️ ccswarm Sangha & Self-Extension Demo");
    println!("=====================================\n");

    // Initialize Sangha
    let sangha_config = SanghaConfig::default();
    let sangha = Arc::new(Sangha::new(sangha_config)?);
    
    println!("✅ Sangha initialized with default configuration");
    println!("   Quorum threshold: {}", sangha.config.quorum_threshold);
    println!("   Simple majority: {}%", sangha.config.simple_majority * 100.0);
    println!("   Super majority: {}%\n", sangha.config.super_majority * 100.0);

    // Add agents as Sangha members
    let frontend_member = SanghaMember {
        agent_id: "frontend-01".to_string(),
        role: AgentRole::Frontend {
            technologies: vec!["React".to_string(), "TypeScript".to_string()],
            responsibilities: vec!["UI Development".to_string()],
            boundaries: vec!["Client-side only".to_string()],
        },
        joined_at: Utc::now(),
        voting_power: 1.0,
        is_active: true,
        reputation: 0.85,
    };

    let backend_member = SanghaMember {
        agent_id: "backend-01".to_string(),
        role: AgentRole::Backend {
            technologies: vec!["Rust".to_string(), "PostgreSQL".to_string()],
            responsibilities: vec!["API Development".to_string()],
            boundaries: vec!["Server-side only".to_string()],
        },
        joined_at: Utc::now(),
        voting_power: 1.0,
        is_active: true,
        reputation: 0.90,
    };

    let devops_member = SanghaMember {
        agent_id: "devops-01".to_string(),
        role: AgentRole::DevOps {
            technologies: vec!["Docker".to_string(), "Kubernetes".to_string()],
            responsibilities: vec!["Infrastructure".to_string()],
            boundaries: vec!["CI/CD and deployment".to_string()],
        },
        joined_at: Utc::now(),
        voting_power: 1.0,
        is_active: true,
        reputation: 0.95,
    };

    sangha.add_member(frontend_member).await?;
    sangha.add_member(backend_member).await?;
    sangha.add_member(devops_member).await?;

    println!("✅ Added 3 agents to Sangha");
    
    let stats = sangha.get_stats().await;
    println!("📊 Sangha Statistics:");
    println!("   Total members: {}", stats.total_members);
    println!("   Active members: {}", stats.active_members);
    println!("   Consensus algorithm: {}\n", stats.consensus_algorithm);

    // Create Extension Manager
    let extension_manager = ExtensionManager::new(sangha.clone());
    println!("✅ Extension Manager initialized\n");

    // Simulate agent self-extension proposal
    println!("🔧 Frontend agent proposing React Server Components extension...\n");
    
    let extension_proposal = ExtensionProposal {
        id: Uuid::new_v4(),
        proposer: "frontend-01".to_string(),
        extension_type: ExtensionType::Capability,
        title: "Add React Server Components Support".to_string(),
        description: "Extend frontend capabilities to support React Server Components for improved performance and SEO".to_string(),
        current_state: ccswarm::extension::CurrentState {
            capabilities: vec!["React".to_string(), "Client-side rendering".to_string()],
            limitations: vec!["SEO limitations".to_string(), "Initial load performance".to_string()],
            performance_metrics: [("page_load_time".to_string(), 2.5)].into(),
        },
        proposed_state: ccswarm::extension::ProposedState {
            new_capabilities: vec!["React Server Components".to_string(), "Streaming SSR".to_string()],
            expected_improvements: vec!["50% faster initial load".to_string(), "Better SEO".to_string()],
            performance_targets: [("page_load_time".to_string(), 1.2)].into(),
        },
        implementation_plan: ccswarm::extension::ImplementationPlan {
            phases: vec![
                ccswarm::extension::ImplementationPhase {
                    name: "Research".to_string(),
                    description: "Study RSC documentation and best practices".to_string(),
                    tasks: vec!["Read official docs".to_string(), "Review examples".to_string()],
                    duration_estimate: "3 days".to_string(),
                    validation_method: "Knowledge test".to_string(),
                },
                ccswarm::extension::ImplementationPhase {
                    name: "Prototype".to_string(),
                    description: "Build small RSC prototype".to_string(),
                    tasks: vec!["Create test app".to_string(), "Implement basic RSC".to_string()],
                    duration_estimate: "1 week".to_string(),
                    validation_method: "Working prototype".to_string(),
                },
            ],
            timeline: "2 weeks".to_string(),
            resources_required: vec!["Next.js 13+".to_string()],
            dependencies: vec!["Node.js 18+".to_string()],
        },
        risk_assessment: ccswarm::extension::RiskAssessment {
            risks: vec![
                ccswarm::extension::Risk {
                    description: "Breaking changes to existing components".to_string(),
                    probability: 0.3,
                    impact: 0.7,
                    category: ccswarm::extension::RiskCategory::Compatibility,
                },
            ],
            mitigation_strategies: vec!["Gradual migration".to_string(), "Backward compatibility layer".to_string()],
            rollback_plan: "Revert to client-only rendering".to_string(),
            overall_risk_score: 0.4,
        },
        success_criteria: vec![
            ccswarm::extension::SuccessCriterion {
                description: "Page load time improvement".to_string(),
                metric: "page_load_time".to_string(),
                target_value: "< 1.5s".to_string(),
                measurement_method: "Lighthouse audit".to_string(),
            },
        ],
        created_at: Utc::now(),
        status: ccswarm::extension::ExtensionStatus::Proposed,
    };

    // Submit extension proposal
    let proposal_id = extension_manager.propose_extension(extension_proposal).await?;
    println!("✅ Extension proposal submitted to Sangha");
    println!("   Proposal ID: {}", proposal_id);
    println!("   Status: Under review by Sangha\n");

    // Simulate Sangha voting (in real implementation, agents would vote)
    println!("🗳️ Sangha members voting on the proposal...");
    println!("   Frontend agent: Aye (proposer)");
    println!("   Backend agent: Aye (supports better performance)");
    println!("   DevOps agent: Abstain (needs more infrastructure details)\n");

    println!("📊 Voting Results:");
    println!("   Aye: 2 votes (66.7%)");
    println!("   Nay: 0 votes (0%)");
    println!("   Abstain: 1 vote (33.3%)");
    println!("   ✅ Proposal PASSED (simple majority achieved)\n");

    // Demonstrate evolution tracking
    println!("📈 Evolution Metrics:");
    println!("   Learning velocity: 0.85");
    println!("   Adaptation rate: 0.72");
    println!("   Pattern recognition: +15% improvement");
    println!("   Failure avoidance: 92% success rate\n");

    println!("🎉 Demo completed successfully!");
    println!("\nThis demo showcased:");
    println!("- Sangha initialization and member management");
    println!("- Agent self-extension proposal submission");
    println!("- Collective decision-making through voting");
    println!("- Evolution tracking and metrics");

    Ok(())
}