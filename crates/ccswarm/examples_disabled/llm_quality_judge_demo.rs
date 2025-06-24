use anyhow::Result;
use ccswarm::agent::{Priority, Task, TaskResult, TaskType};
use ccswarm::identity::AgentRole;
use ccswarm::orchestrator::llm_quality_judge::{LLMQualityJudge, QualityRubric};
use std::time::Duration;
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🔍 LLM Quality Judge Demo\n");

    // Create a quality judge with heuristics (no Claude required for demo)
    let mut judge = LLMQualityJudge::new();
    judge.use_claude = false; // Use heuristics for demo

    // Demo 1: High-quality code evaluation
    println!("📝 Example 1: High-Quality Frontend Code");
    let frontend_task = Task::new(
        "ui-001".to_string(),
        "Create user profile component".to_string(),
        Priority::High,
        TaskType::Development,
    );

    let good_result = TaskResult {
        success: true,
        output: serde_json::json!({
            "response": r#"
import React from 'react';
import { render, screen } from '@testing-library/react';

// Component implementation
export const UserProfile = ({ user }) => {
  try {
    if (!user) {
      return <div>No user data available</div>;
    }
    
    return (
      <div className="user-profile">
        <h2>{user.name}</h2>
        <p>{user.email}</p>
        <img src={user.avatar} alt={user.name} />
      </div>
    );
  } catch (error) {
    console.error('Error rendering user profile:', error);
    return <div>Error loading profile</div>;
  }
};

// Unit tests
describe('UserProfile', () => {
  test('renders user information', () => {
    const user = { name: 'John', email: 'john@example.com' };
    render(<UserProfile user={user} />);
    expect(screen.getByText('John')).toBeInTheDocument();
  });
});
"#
        }),
        error: None,
        duration: Duration::from_secs(5),
    };

    let frontend_role = AgentRole::Frontend {
        technologies: vec!["React".to_string()],
        responsibilities: vec![],
        boundaries: vec![],
    };

    let evaluation1 = judge
        .evaluate_task(&frontend_task, &good_result, &frontend_role, "/tmp")
        .await?;

    println!("✅ Evaluation Results:");
    println!("   Overall Score: {:.2}", evaluation1.overall_score);
    println!("   Passes Standards: {}", evaluation1.passes_standards);
    println!("   Issues Found: {}", evaluation1.issues.len());
    println!("   Confidence: {:.2}", evaluation1.confidence);
    println!("   Feedback: {}", evaluation1.feedback);

    if !evaluation1.issues.is_empty() {
        println!("\n   📋 Issues:");
        for issue in &evaluation1.issues {
            println!("      - [{:?}] {}", issue.severity, issue.description);
        }
    }

    // Demo 2: Code with quality issues
    println!("\n\n📝 Example 2: Backend Code with Issues");
    let backend_task = Task::new(
        "api-002".to_string(),
        "Create user authentication endpoint".to_string(),
        Priority::Critical,
        TaskType::Development,
    );

    let problematic_result = TaskResult {
        success: true,
        output: serde_json::json!({
            "response": r#"
const express = require('express');
const router = express.Router();

// No error handling, no tests, security issues
router.post('/login', (req, res) => {
  const { username, password } = req.body;
  
  // Hardcoded credentials - security issue!
  if (username === 'admin' && password === 'password123') {
    res.json({ token: 'secret-token' });
  } else {
    res.status(401).send('Invalid credentials');
  }
});

module.exports = router;
"#
        }),
        error: None,
        duration: Duration::from_secs(3),
    };

    let backend_role = AgentRole::Backend {
        technologies: vec!["Node.js".to_string()],
        responsibilities: vec![],
        boundaries: vec![],
    };

    let evaluation2 = judge
        .evaluate_task(&backend_task, &problematic_result, &backend_role, "/tmp")
        .await?;

    println!("❌ Evaluation Results:");
    println!("   Overall Score: {:.2}", evaluation2.overall_score);
    println!("   Passes Standards: {}", evaluation2.passes_standards);
    println!("   Issues Found: {}", evaluation2.issues.len());
    println!("   Confidence: {:.2}", evaluation2.confidence);
    println!("   Feedback: {}", evaluation2.feedback);

    if !evaluation2.issues.is_empty() {
        println!("\n   📋 Issues:");
        for issue in &evaluation2.issues {
            println!(
                "      - [{:?}] {}: {}",
                issue.severity,
                format!("{:?}", issue.category),
                issue.description
            );
        }

        // Generate fix instructions
        let fix_instructions = judge.generate_fix_instructions(&evaluation2.issues, "Backend");
        println!("\n   🔧 Fix Instructions:");
        println!("{}", fix_instructions);
    }

    // Demo 3: DevOps configuration with security focus
    println!("\n\n📝 Example 3: DevOps Configuration");
    let devops_task = Task::new(
        "infra-003".to_string(),
        "Create Docker deployment configuration".to_string(),
        Priority::High,
        TaskType::Infrastructure,
    );

    let devops_result = TaskResult {
        success: true,
        output: serde_json::json!({
            "response": r#"
FROM node:14

# Security issue: running as root
WORKDIR /app

COPY package*.json ./
RUN npm install

COPY . .

# Security issue: exposing unnecessary information
ENV NODE_ENV=production
ENV API_KEY=sk-1234567890

EXPOSE 3000

# No health check defined
CMD ["node", "server.js"]
"#
        }),
        error: None,
        duration: Duration::from_secs(2),
    };

    let devops_role = AgentRole::DevOps {
        technologies: vec!["Docker".to_string()],
        responsibilities: vec![],
        boundaries: vec![],
    };

    let evaluation3 = judge
        .evaluate_task(&devops_task, &devops_result, &devops_role, "/tmp")
        .await?;

    println!("⚠️  Evaluation Results:");
    println!("   Overall Score: {:.2}", evaluation3.overall_score);
    println!("   Passes Standards: {}", evaluation3.passes_standards);
    println!("   Issues Found: {}", evaluation3.issues.len());

    // Demo rubric customization
    println!("\n\n🎯 Custom Rubric Example");
    let mut custom_rubric = QualityRubric::default();

    // Adjust weights for a security-focused evaluation
    custom_rubric.dimensions.insert("security".to_string(), 0.4);
    custom_rubric
        .dimensions
        .insert("test_quality".to_string(), 0.1);

    let _security_judge = LLMQualityJudge::with_rubric(custom_rubric);
    println!("   Created security-focused judge with custom weights");

    println!("\n✨ Demo completed successfully!");

    Ok(())
}
