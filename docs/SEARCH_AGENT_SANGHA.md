# Search Agent Sangha Integration

## Overview

The Search Agent now has autonomous Sangha participation capabilities, enabling it to monitor proposals, conduct research, and cast informed votes based on evidence.

## Key Features

### 1. Autonomous Proposal Monitoring
- Continuously monitors active Sangha proposals
- Identifies proposals requiring research
- Prioritizes based on proposal type and urgency

### 2. Intelligent Research
- Extracts relevant search queries from proposals
- Conducts web searches using gemini CLI
- Analyzes search results for positive/negative indicators
- Calculates confidence scores based on evidence

### 3. Evidence-Based Voting
- Submits research findings as evidence
- Casts votes with confidence-weighted reasoning
- Supports emergency proposals with higher priority
- Abstains when insufficient evidence is available

### 4. Knowledge Gap Detection
- Identifies proposals lacking research
- Creates new proposals to address gaps
- Monitors research coverage across all proposals

## Implementation Details

### SanghaParticipant Trait
```rust
#[async_trait]
pub trait SanghaParticipant: Send + Sync {
    fn agent_id(&self) -> &str;
    fn role(&self) -> &AgentRole;
    async fn monitor_proposals(&mut self, sangha: Arc<Sangha>) -> Result<()>;
    async fn analyze_proposal(&mut self, proposal: &Proposal) -> Result<VotingDecision>;
    async fn submit_evidence(&mut self, proposal_id: Uuid, evidence: Evidence, sangha: Arc<Sangha>) -> Result<()>;
    async fn cast_informed_vote(&mut self, proposal: &Proposal, decision: VotingDecision, sangha: Arc<Sangha>) -> Result<()>;
    async fn propose_initiative(&mut self, need: IdentifiedNeed, sangha: Arc<Sangha>) -> Result<Uuid>;
}
```

### Search Agent Integration
```rust
// Enable Sangha participation
let mut search_agent = SearchAgent::new(agent_id, coordination_bus);
search_agent.enable_sangha_participation();

// Start monitoring in background
search_agent.start_sangha_monitoring(sangha).await?;
```

### Research Process
1. **Query Extraction**: Analyzes proposal title and description
2. **Search Execution**: Runs multiple searches with filters
3. **Result Analysis**: Counts positive/negative indicators
4. **Confidence Calculation**: Based on evidence quality and quantity
5. **Vote Decision**: Aye/Nay/Abstain based on confidence threshold

### Evidence Types
- **ResearchEvidence**: Web search findings
- **TechnicalEvidence**: Technical analysis and recommendations
- **HistoricalEvidence**: Past precedents and outcomes
- **ComparativeEvidence**: Alternative approaches

## Usage Example

```rust
// Create Sangha and Search Agent
let sangha = Arc::new(Sangha::new(SanghaConfig::default())?);
let mut search_agent = SearchAgent::new("search-1".to_string(), coordination_bus);

// Enable Sangha participation
search_agent.enable_sangha_participation();

// Add as member
let member = SanghaMember {
    agent_id: search_agent.agent_id.clone(),
    role: AgentRole::Search { /* ... */ },
    voting_power: 1.0,
    // ...
};
sangha.add_member(member).await?;

// Start autonomous monitoring
search_agent.start_sangha_monitoring(sangha).await?;
```

## Voting Logic

### Confidence Calculation
- Base confidence: 0.5 (neutral)
- +0.1 for each highly relevant result (>0.8 relevance)
- Final confidence capped at 0.9

### Vote Decision
- **Aye**: Evidence ratio > 0.7 or emergency proposal
- **Nay**: Evidence ratio < 0.3
- **Abstain**: Insufficient evidence or neutral ratio

### Weight Assignment
- Vote weight = confidence score
- Allows nuanced voting based on research quality

## Knowledge Gap Detection

The Search Agent monitors for:
- Proposals with < 3 evidence items
- Topics without recent research
- Technical areas needing investigation

When gaps are detected, it creates proposals with:
- **Type**: Based on gap type
- **Urgency**: Critical/High/Medium/Low
- **Supporting Data**: Current evidence count, affected proposals

## Future Enhancements

1. **Advanced NLP**: Better query extraction
2. **Source Ranking**: Prioritize authoritative sources
3. **Trend Analysis**: Track opinion changes over time
4. **Collaborative Research**: Coordinate with other agents
5. **Learning System**: Improve search strategies based on outcomes