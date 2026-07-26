use anyhow::Result;
use ccswarm::session::a2a::{AgentCapabilities, AgentCard, AgentInterface, AgentSkill};
use ccswarm::session::bridge::A2ABridge;

#[tokio::test]
async fn ccswarm_registers_native_a2a_bridge_context() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let bridge = A2ABridge::new(dir.path().join("sessions"));

    bridge.register_agent("reviewer")?;

    assert_eq!(bridge.agent_count(), 1);
    Ok(())
}

#[test]
fn agent_card_models_a2a_discovery_metadata() {
    let card = AgentCard {
        name: "ccswarm reviewer".to_string(),
        description: "Reviews workflow outputs".to_string(),
        supported_interfaces: vec![AgentInterface {
            url: "https://example.test/a2a".to_string(),
            protocol_binding: "HTTP+JSON".to_string(),
            protocol_version: Some("1.0".to_string()),
        }],
        capabilities: AgentCapabilities {
            streaming: false,
            push_notifications: false,
        },
        version: env!("CARGO_PKG_VERSION").to_string(),
        skills: vec![AgentSkill {
            id: "review".to_string(),
            name: "Review".to_string(),
            description: "Review generated code".to_string(),
        }],
    };

    let json = serde_json::to_value(card).expect("agent card should serialize");
    assert_eq!(json["name"], "ccswarm reviewer");
    assert_eq!(json["skills"][0]["id"], "review");
}
