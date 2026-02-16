//! AI Judge and tag-based condition evaluation for Piece/Movement workflows.
//!
//! Implements takt-style three-phase evaluation:
//! 1. **Tag-based routing**: `[STEP:N]` tags injected into prompts, detected in output
//! 2. **AI judge**: `ai("condition text")` for LLM-powered condition evaluation
//! 3. **Aggregate conditions**: `all("X")` / `any("X")` for parallel movement results

use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

use super::piece::{MovementRule, RuleCondition};

/// Tag format used for step routing: `[STEP:N]`
const STEP_TAG_PATTERN: &str = r"\[STEP:(\d+)\]";

/// Result of a judge evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeResult {
    /// Index of the matched rule (None = no match)
    pub matched_rule_index: Option<usize>,
    /// The method used to determine the match
    pub match_method: MatchMethod,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// Explanation of the judgment
    pub explanation: String,
}

/// How the match was determined
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MatchMethod {
    /// Matched via [STEP:N] tag in output
    StepTag,
    /// Matched via simple string condition
    SimpleCondition,
    /// Matched via AI judge evaluation
    AiJudge,
    /// Matched via aggregate condition (all/any)
    Aggregate,
    /// No match found
    NoMatch,
}

/// Parsed condition from rule condition strings
#[derive(Debug, Clone)]
pub enum ParsedCondition {
    /// Simple string match
    Simple(String),
    /// AI judge: ai("condition text")
    Ai(String),
    /// All sub-conditions must match: all("tag")
    All(String),
    /// Any sub-condition must match: any("tag")
    Any(String),
}

/// Configuration for the AI judge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeConfig {
    /// Whether to enable AI-based condition evaluation
    pub enable_ai_judge: bool,
    /// Minimum confidence threshold for AI judge
    pub ai_confidence_threshold: f64,
    /// Maximum retries for AI evaluation
    pub max_ai_retries: u32,
}

impl Default for JudgeConfig {
    fn default() -> Self {
        Self {
            enable_ai_judge: true,
            ai_confidence_threshold: 0.7,
            max_ai_retries: 2,
        }
    }
}

/// The movement judge evaluates output and determines routing.
///
/// Evaluation priority (from takt):
/// 1. Aggregate conditions: `all("X")` / `any("X")`
/// 2. Phase 3 tags: `[STEP:N]`
/// 3. Simple string conditions
/// 4. AI judgment via `ai(...)` expressions
/// 5. Default fallback (first "success" rule or None)
pub struct MovementJudge {
    config: JudgeConfig,
}

impl MovementJudge {
    pub fn new(config: JudgeConfig) -> Self {
        Self { config }
    }

    /// Generate tag injection instructions for an agent prompt.
    ///
    /// Tells the agent to output `[STEP:N]` tags based on the rules.
    pub fn generate_tag_instructions(rules: &[MovementRule]) -> String {
        if rules.is_empty() {
            return String::new();
        }

        let mut instructions = String::from(
            "\n\n--- Status Output Instructions ---\n\
             After completing your task, output ONE of the following status tags \
             on its own line to indicate the result:\n\n",
        );

        for (i, rule) in rules.iter().enumerate() {
            let condition_desc = match &rule.condition {
                RuleCondition::Simple(s) => s.clone(),
                RuleCondition::AiCondition { ai } => ai.clone(),
                RuleCondition::Compound(_) => format!("compound condition {}", i),
            };
            instructions.push_str(&format!("[STEP:{}] - {}\n", i, condition_desc));
        }

        instructions.push_str("\nOutput exactly one tag that best describes your result.\n");
        instructions
    }

    /// Evaluate movement output against rules and determine the next movement.
    ///
    /// Returns the ID of the next movement, or None if no rule matched.
    pub fn evaluate(
        &self,
        output: &str,
        rules: &[MovementRule],
        parallel_outputs: Option<&HashMap<String, String>>,
    ) -> Result<JudgeResult> {
        if rules.is_empty() {
            return Ok(JudgeResult {
                matched_rule_index: None,
                match_method: MatchMethod::NoMatch,
                confidence: 1.0,
                explanation: "No rules defined (terminal movement)".to_string(),
            });
        }

        // 1. Check aggregate conditions first (for parallel movements)
        if let Some(parallel_out) = parallel_outputs
            && let Some(result) = self.evaluate_aggregate(rules, parallel_out)?
        {
            return Ok(result);
        }

        // 2. Check for [STEP:N] tags in output
        if let Some(result) = self.evaluate_step_tags(output, rules)? {
            return Ok(result);
        }

        // 3. Check simple string conditions
        if let Some(result) = self.evaluate_simple_conditions(output, rules)? {
            return Ok(result);
        }

        // 4. AI judge evaluation (if enabled)
        if self.config.enable_ai_judge
            && let Some(result) = self.evaluate_ai_conditions(output, rules)?
        {
            return Ok(result);
        }

        // 5. Fallback: look for default "success" rule
        if let Some(result) = self.evaluate_fallback(rules)? {
            return Ok(result);
        }

        Ok(JudgeResult {
            matched_rule_index: None,
            match_method: MatchMethod::NoMatch,
            confidence: 0.0,
            explanation: "No rule matched output".to_string(),
        })
    }

    /// Check for [STEP:N] tags in output
    fn evaluate_step_tags(
        &self,
        output: &str,
        rules: &[MovementRule],
    ) -> Result<Option<JudgeResult>> {
        let re = Regex::new(STEP_TAG_PATTERN).context("Invalid step tag regex")?;

        if let Some(captures) = re.captures(output)
            && let Some(index_str) = captures.get(1)
            && let Ok(index) = index_str.as_str().parse::<usize>()
        {
            if index < rules.len() {
                info!("Step tag [STEP:{}] matched rule {}", index, index);
                return Ok(Some(JudgeResult {
                    matched_rule_index: Some(index),
                    match_method: MatchMethod::StepTag,
                    confidence: 1.0,
                    explanation: format!("Matched [STEP:{}] tag in output", index),
                }));
            }
            warn!(
                "Step tag [STEP:{}] out of range (only {} rules)",
                index,
                rules.len()
            );
        }

        Ok(None)
    }

    /// Evaluate simple string conditions against output
    fn evaluate_simple_conditions(
        &self,
        output: &str,
        rules: &[MovementRule],
    ) -> Result<Option<JudgeResult>> {
        let output_lower = output.to_lowercase();

        // Sort by priority (higher first)
        let mut indexed_rules: Vec<(usize, &MovementRule)> = rules.iter().enumerate().collect();
        indexed_rules.sort_by(|a, b| b.1.priority.cmp(&a.1.priority));

        for (index, rule) in indexed_rules {
            if let RuleCondition::Simple(condition) = &rule.condition {
                let matched = match condition.as_str() {
                    "success" | "complete" | "done" => {
                        output_lower.contains("success")
                            || output_lower.contains("completed")
                            || output_lower.contains("done")
                            || (!output_lower.contains("error")
                                && !output_lower.contains("failed")
                                && !output_lower.contains("failure"))
                    }
                    "failure" | "error" | "fail" => {
                        output_lower.contains("error")
                            || output_lower.contains("failed")
                            || output_lower.contains("failure")
                    }
                    "needs_fix" | "fixes_needed" => {
                        output_lower.contains("fix")
                            || output_lower.contains("issue")
                            || output_lower.contains("problem")
                    }
                    "needs_clarification" | "unclear" => {
                        output_lower.contains("clarif")
                            || output_lower.contains("unclear")
                            || output_lower.contains("ambiguous")
                    }
                    "test_failure" | "tests_failed" => {
                        output_lower.contains("test failed")
                            || output_lower.contains("tests failed")
                            || output_lower.contains("test failure")
                    }
                    other => output_lower.contains(&other.to_lowercase()),
                };

                if matched {
                    debug!(
                        "Simple condition '{}' matched at rule index {}",
                        condition, index
                    );
                    return Ok(Some(JudgeResult {
                        matched_rule_index: Some(index),
                        match_method: MatchMethod::SimpleCondition,
                        confidence: 0.8,
                        explanation: format!("Simple condition '{}' matched in output", condition),
                    }));
                }
            }
        }

        Ok(None)
    }

    /// Evaluate AI conditions (ai("...") syntax)
    fn evaluate_ai_conditions(
        &self,
        output: &str,
        rules: &[MovementRule],
    ) -> Result<Option<JudgeResult>> {
        for (index, rule) in rules.iter().enumerate() {
            if let RuleCondition::AiCondition { ai: condition } = &rule.condition {
                debug!("Evaluating AI condition: '{}' against output", condition);

                // Build a judgment prompt
                let judgment = self.ai_judge_evaluate(condition, output)?;

                if judgment.matched {
                    info!(
                        "AI judge matched condition '{}' with confidence {:.2}",
                        condition, judgment.confidence
                    );
                    if judgment.confidence >= self.config.ai_confidence_threshold {
                        return Ok(Some(JudgeResult {
                            matched_rule_index: Some(index),
                            match_method: MatchMethod::AiJudge,
                            confidence: judgment.confidence,
                            explanation: judgment.explanation,
                        }));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Evaluate aggregate conditions (all/any) for parallel movement outputs
    fn evaluate_aggregate(
        &self,
        rules: &[MovementRule],
        parallel_outputs: &HashMap<String, String>,
    ) -> Result<Option<JudgeResult>> {
        for (index, rule) in rules.iter().enumerate() {
            if let RuleCondition::Compound(compound) = &rule.condition {
                let matched = match compound {
                    super::piece::CompoundCondition::All(conditions) => {
                        conditions.iter().all(|cond| {
                            parallel_outputs
                                .values()
                                .all(|out| out.to_lowercase().contains(&cond.to_lowercase()))
                        })
                    }
                    super::piece::CompoundCondition::Any(conditions) => {
                        conditions.iter().any(|cond| {
                            parallel_outputs
                                .values()
                                .any(|out| out.to_lowercase().contains(&cond.to_lowercase()))
                        })
                    }
                };

                if matched {
                    info!("Aggregate condition matched at rule index {}", index);
                    return Ok(Some(JudgeResult {
                        matched_rule_index: Some(index),
                        match_method: MatchMethod::Aggregate,
                        confidence: 1.0,
                        explanation: format!(
                            "Aggregate condition matched across {} parallel outputs",
                            parallel_outputs.len()
                        ),
                    }));
                }
            }
        }

        Ok(None)
    }

    /// Fallback: match the first "success"-like rule
    fn evaluate_fallback(&self, rules: &[MovementRule]) -> Result<Option<JudgeResult>> {
        for (index, rule) in rules.iter().enumerate() {
            if let RuleCondition::Simple(cond) = &rule.condition
                && matches!(cond.as_str(), "success" | "complete" | "done" | "default")
            {
                return Ok(Some(JudgeResult {
                    matched_rule_index: Some(index),
                    match_method: MatchMethod::SimpleCondition,
                    confidence: 0.5,
                    explanation: format!("Fallback to '{}' rule", cond),
                }));
            }
        }
        Ok(None)
    }

    /// Perform AI-based judgment of a condition against output.
    ///
    /// In production, this calls the LLM. For now, uses heuristic evaluation.
    fn ai_judge_evaluate(&self, condition: &str, output: &str) -> Result<AiJudgment> {
        // Heuristic AI judge: analyze semantic overlap between condition and output
        let condition_words: Vec<&str> = condition
            .split_whitespace()
            .filter(|w| w.len() > 3)
            .collect();

        let output_lower = output.to_lowercase();
        let matched_words = condition_words
            .iter()
            .filter(|w| output_lower.contains(&w.to_lowercase()))
            .count();

        let confidence = if condition_words.is_empty() {
            0.0
        } else {
            matched_words as f64 / condition_words.len() as f64
        };

        let matched = confidence >= 0.5;

        Ok(AiJudgment {
            matched,
            confidence,
            explanation: format!(
                "Heuristic AI judge: {}/{} condition words found in output (threshold: 0.5)",
                matched_words,
                condition_words.len()
            ),
        })
    }

    /// Parse a condition string to determine its type
    pub fn parse_condition(condition_str: &str) -> ParsedCondition {
        let trimmed = condition_str.trim();

        // Check for ai("...") syntax
        if let Some(inner) = extract_function_arg(trimmed, "ai") {
            return ParsedCondition::Ai(inner);
        }

        // Check for all("...") syntax
        if let Some(inner) = extract_function_arg(trimmed, "all") {
            return ParsedCondition::All(inner);
        }

        // Check for any("...") syntax
        if let Some(inner) = extract_function_arg(trimmed, "any") {
            return ParsedCondition::Any(inner);
        }

        ParsedCondition::Simple(trimmed.to_string())
    }
}

impl Default for MovementJudge {
    fn default() -> Self {
        Self::new(JudgeConfig::default())
    }
}

/// Internal AI judgment result
struct AiJudgment {
    matched: bool,
    confidence: f64,
    explanation: String,
}

/// Extract the argument from a function-style syntax: `fn_name("arg")`
fn extract_function_arg(input: &str, fn_name: &str) -> Option<String> {
    let prefix = format!("{}(", fn_name);
    if input.starts_with(&prefix) && input.ends_with(')') {
        let inner = &input[prefix.len()..input.len() - 1];
        // Strip surrounding quotes if present
        let stripped = inner
            .trim()
            .strip_prefix('"')
            .and_then(|s| s.strip_suffix('"'))
            .unwrap_or(inner.trim());
        Some(stripped.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::piece::CompoundCondition;

    fn make_simple_rule(condition: &str, next: &str) -> MovementRule {
        MovementRule {
            condition: RuleCondition::Simple(condition.to_string()),
            next: next.to_string(),
            priority: 0,
        }
    }

    fn make_ai_rule(condition: &str, next: &str) -> MovementRule {
        MovementRule {
            condition: RuleCondition::AiCondition {
                ai: condition.to_string(),
            },
            next: next.to_string(),
            priority: 0,
        }
    }

    #[test]
    fn test_step_tag_detection() {
        let judge = MovementJudge::default();
        let rules = vec![
            make_simple_rule("approved", "deploy"),
            make_simple_rule("needs_fix", "fix"),
        ];

        let output = "Task completed successfully.\n[STEP:0]\nAll good.";
        let result = judge.evaluate(output, &rules, None).unwrap();
        assert_eq!(result.matched_rule_index, Some(0));
        assert_eq!(result.match_method, MatchMethod::StepTag);

        let output2 = "Found issues.\n[STEP:1]\nNeeds fixing.";
        let result2 = judge.evaluate(output2, &rules, None).unwrap();
        assert_eq!(result2.matched_rule_index, Some(1));
        assert_eq!(result2.match_method, MatchMethod::StepTag);
    }

    #[test]
    fn test_simple_condition_success() {
        let judge = MovementJudge::default();
        let rules = vec![
            make_simple_rule("success", "next-step"),
            make_simple_rule("failure", "error-handler"),
        ];

        // Output with no error keywords -> matches "success"
        let output = "All tasks completed without issues.";
        let result = judge.evaluate(output, &rules, None).unwrap();
        assert_eq!(result.matched_rule_index, Some(0));
        assert_eq!(result.match_method, MatchMethod::SimpleCondition);
    }

    #[test]
    fn test_simple_condition_failure() {
        let judge = MovementJudge::default();
        let rules = vec![
            make_simple_rule("failure", "error-handler"),
            make_simple_rule("success", "next-step"),
        ];

        let output = "Build failed with 3 errors.";
        let result = judge.evaluate(output, &rules, None).unwrap();
        assert_eq!(result.matched_rule_index, Some(0));
        assert_eq!(result.match_method, MatchMethod::SimpleCondition);
    }

    #[test]
    fn test_ai_condition_evaluation() {
        let judge = MovementJudge::default();
        let rules = vec![make_ai_rule("code quality meets standards", "deploy")];

        // Output contains words from the condition
        let output = "The code quality is excellent and meets all standards.";
        let result = judge.evaluate(output, &rules, None).unwrap();
        assert_eq!(result.matched_rule_index, Some(0));
        assert_eq!(result.match_method, MatchMethod::AiJudge);
    }

    #[test]
    fn test_aggregate_all_condition() {
        let judge = MovementJudge::default();
        let rules = vec![MovementRule {
            condition: RuleCondition::Compound(CompoundCondition::All(vec![
                "approved".to_string(),
            ])),
            next: "deploy".to_string(),
            priority: 0,
        }];

        let mut parallel_outputs = HashMap::new();
        parallel_outputs.insert("reviewer-1".to_string(), "Code approved, LGTM".to_string());
        parallel_outputs.insert(
            "reviewer-2".to_string(),
            "Approved with minor nits".to_string(),
        );

        let result = judge.evaluate("", &rules, Some(&parallel_outputs)).unwrap();
        assert_eq!(result.matched_rule_index, Some(0));
        assert_eq!(result.match_method, MatchMethod::Aggregate);
    }

    #[test]
    fn test_aggregate_any_condition() {
        let judge = MovementJudge::default();
        let rules = vec![MovementRule {
            condition: RuleCondition::Compound(CompoundCondition::Any(vec![
                "rejected".to_string(),
            ])),
            next: "fix".to_string(),
            priority: 0,
        }];

        let mut parallel_outputs = HashMap::new();
        parallel_outputs.insert("reviewer-1".to_string(), "Code approved, LGTM".to_string());
        parallel_outputs.insert(
            "reviewer-2".to_string(),
            "Code rejected - security issue".to_string(),
        );

        let result = judge.evaluate("", &rules, Some(&parallel_outputs)).unwrap();
        assert_eq!(result.matched_rule_index, Some(0));
        assert_eq!(result.match_method, MatchMethod::Aggregate);
    }

    #[test]
    fn test_tag_instruction_generation() {
        let rules = vec![
            make_simple_rule("approved", "deploy"),
            make_simple_rule("needs_fix", "fix"),
            make_simple_rule("blocked", "abort"),
        ];

        let instructions = MovementJudge::generate_tag_instructions(&rules);
        assert!(instructions.contains("[STEP:0]"));
        assert!(instructions.contains("[STEP:1]"));
        assert!(instructions.contains("[STEP:2]"));
        assert!(instructions.contains("approved"));
        assert!(instructions.contains("needs_fix"));
        assert!(instructions.contains("blocked"));
    }

    #[test]
    fn test_parse_condition() {
        match MovementJudge::parse_condition("success") {
            ParsedCondition::Simple(s) => assert_eq!(s, "success"),
            _ => panic!("Expected Simple"),
        }

        match MovementJudge::parse_condition("ai(\"code quality is good\")") {
            ParsedCondition::Ai(s) => assert_eq!(s, "code quality is good"),
            _ => panic!("Expected Ai"),
        }

        match MovementJudge::parse_condition("all(\"approved\")") {
            ParsedCondition::All(s) => assert_eq!(s, "approved"),
            _ => panic!("Expected All"),
        }

        match MovementJudge::parse_condition("any(\"rejected\")") {
            ParsedCondition::Any(s) => assert_eq!(s, "rejected"),
            _ => panic!("Expected Any"),
        }
    }

    #[test]
    fn test_empty_rules() {
        let judge = MovementJudge::default();
        let result = judge.evaluate("some output", &[], None).unwrap();
        assert!(result.matched_rule_index.is_none());
        assert_eq!(result.match_method, MatchMethod::NoMatch);
    }

    #[test]
    fn test_fallback_to_success() {
        let judge = MovementJudge::default();
        let rules = vec![
            make_ai_rule("very specific condition nobody matches", "specific"),
            make_simple_rule("success", "default-next"),
        ];

        // Output that doesn't match the AI condition
        let output = "x";
        let result = judge.evaluate(output, &rules, None).unwrap();
        // Should fallback to "success" rule
        assert_eq!(result.matched_rule_index, Some(1));
    }
}
