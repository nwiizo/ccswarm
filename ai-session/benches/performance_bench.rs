//! Performance benchmarks for ai-session crate
//!
//! These benchmarks measure the performance of critical operations.

use ai_session::*;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Benchmark session creation and destruction
fn bench_session_lifecycle(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("session_creation", |b| {
        b.iter(|| {
            rt.block_on(async {
                let manager = SessionManager::new();
                let session = manager.create_session().await.unwrap();
                black_box(session);
            });
        });
    });

    c.bench_function("session_with_ai_features", |b| {
        b.iter(|| {
            rt.block_on(async {
                let manager = SessionManager::new();
                let mut config = SessionConfig::default();
                config.enable_ai_features = true;
                config.context_config.max_tokens = 8192;

                let session = manager.create_session_with_config(config).await.unwrap();
                black_box(session);
            });
        });
    });
}

/// Benchmark context management operations
fn bench_context_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("context_message_addition", |b| {
        b.iter(|| {
            rt.block_on(async {
                let session_id = core::SessionId::new();
                let mut context = context::SessionContext::new(session_id);

                for i in 0..100 {
                    context
                        .add_message(context::Message {
                            role: context::MessageRole::User,
                            content: format!("Test message {}", i),
                            timestamp: chrono::Utc::now(),
                            token_count: 10,
                        })
                        .await
                        .unwrap();
                }

                black_box(context);
            });
        });
    });
}

/// Benchmark output parsing performance
fn bench_output_parsing(c: &mut Criterion) {
    use output::OutputParser;

    let parser = OutputParser::new();

    let test_outputs = vec![
        (
            "small",
            "ls -la\ntotal 42\ndrwxr-xr-x  5 user  staff  160 Dec  1 10:00 .",
        ),
        ("medium", &"test output line\n".repeat(100)),
        ("large", &"test output line\n".repeat(1000)),
    ];

    for (size, output) in test_outputs {
        c.bench_with_input(
            BenchmarkId::new("output_parsing", size),
            &output,
            |b, output| {
                b.iter(|| {
                    let parsed = parser.parse(black_box(output.as_bytes())).unwrap();
                    black_box(parsed);
                });
            },
        );
    }
}

/// Benchmark coordination message passing
fn bench_coordination(c: &mut Criterion) {
    use coordination::{AgentId, MessageBus};
    let rt = Runtime::new().unwrap();

    c.bench_function("message_bus_operations", |b| {
        b.iter(|| {
            rt.block_on(async {
                let bus = MessageBus::new();
                let agent_id = AgentId::new();

                // Register and unregister agent
                bus.register_agent(agent_id.clone()).unwrap();
                bus.unregister_agent(&agent_id).unwrap();

                black_box(bus);
            });
        });
    });
}

/// Benchmark persistence operations
fn bench_persistence(c: &mut Criterion) {
    use persistence::{PersistenceManager, SessionMetadata, SessionState};
    use tempfile::TempDir;

    let rt = Runtime::new().unwrap();

    c.bench_function("session_persistence", |b| {
        b.iter(|| {
            rt.block_on(async {
                let temp_dir = TempDir::new().unwrap();
                let manager = PersistenceManager::new(temp_dir.path().to_path_buf());

                let session_id = core::SessionId::new();
                let config = SessionConfig::default();
                let context = context::SessionContext::new(session_id.clone());

                let state = SessionState {
                    session_id: session_id.clone(),
                    config,
                    status: core::SessionStatus::Running,
                    context,
                    command_history: vec![],
                    metadata: SessionMetadata::default(),
                };

                // Save and load
                manager.save_session(&session_id, &state).await.unwrap();
                let loaded = manager.load_session(&session_id).await.unwrap();

                black_box(loaded);
            });
        });
    });
}

/// Benchmark security checks
fn bench_security(c: &mut Criterion) {
    use security::{FileAccessMode, SecurityManager};

    let rt = Runtime::new().unwrap();
    let security = SecurityManager::new();

    c.bench_function("file_access_check", |b| {
        b.iter(|| {
            rt.block_on(async {
                let result = security
                    .check_file_access(black_box("/tmp/test.txt"), FileAccessMode::Read)
                    .await
                    .unwrap();
                black_box(result);
            });
        });
    });
}

/// Benchmark multi-agent scenarios
fn bench_multi_agent_scalability(c: &mut Criterion) {
    use coordination::{AgentId, MultiAgentSession};

    let rt = Runtime::new().unwrap();

    for num_agents in [1, 5, 10, 20].iter() {
        c.bench_with_input(
            BenchmarkId::new("multi_agent_coordination", num_agents),
            num_agents,
            |b, &num_agents| {
                b.iter(|| {
                    rt.block_on(async {
                        let coordinator = Arc::new(MultiAgentSession::new());
                        let manager = SessionManager::new();

                        let mut agents = Vec::new();

                        // Create agents
                        for i in 0..num_agents {
                            let mut config = SessionConfig::default();
                            config.agent_role = Some(format!("agent-{}", i));

                            let session = manager.create_session_with_config(config).await.unwrap();
                            let agent_id = AgentId::new();

                            coordinator
                                .register_agent(agent_id.clone(), session)
                                .unwrap();
                            agents.push(agent_id);
                        }

                        // Cleanup
                        for agent_id in agents {
                            coordinator.unregister_agent(&agent_id).unwrap();
                        }

                        black_box(coordinator);
                    });
                });
            },
        );
    }
}

/// Benchmark token efficiency features
fn bench_token_efficiency(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("context_compression", |b| {
        b.iter(|| {
            rt.block_on(async {
                let session_id = core::SessionId::new();
                let mut context = context::SessionContext::new(session_id);

                // Add many messages to trigger compression
                for i in 0..1000 {
                    context.add_message(context::Message {
                        role: context::MessageRole::User,
                        content: format!("This is a longer test message {} that contains more content to test compression efficiency", i),
                        timestamp: chrono::Utc::now(),
                        token_count: 20,
                    }).await.unwrap();
                }

                // Force compression by getting messages within limit
                let messages = context.get_messages_within_limit(2048);
                black_box(messages);
            });
        });
    });
}

criterion_group!(
    benches,
    bench_session_lifecycle,
    bench_context_operations,
    bench_output_parsing,
    bench_coordination,
    bench_persistence,
    bench_security,
    bench_multi_agent_scalability,
    bench_token_efficiency
);

criterion_main!(benches);
