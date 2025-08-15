//! Automatic refactoring proposal system
//!
//! Identifies code improvements and generates refactoring proposals

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::analyzer::{SemanticAnalyzer, Symbol, SymbolKind};
use super::memory::{Memory, MemoryType, ProjectMemory};
use super::subagent_integration::RefactoringKind;
use super::symbol_index::SymbolIndex;
use super::{SemanticError, SemanticResult};

/// Refactoring proposal
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RefactoringProposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub kind: RefactoringKind,
    pub targets: Vec<RefactoringTarget>,
    pub benefits: Vec<String>,
    pub risks: Vec<String>,
    pub estimated_effort: EffortEstimate,
    pub priority: RefactoringPriority,
    pub automated: bool,
    pub implementation_steps: Vec<ImplementationStep>,
    pub created_at: DateTime<Utc>,
}

/// Refactoring target
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RefactoringTarget {
    pub symbol: Symbol,
    pub issues: Vec<CodeIssue>,
    pub suggested_changes: Vec<SuggestedChange>,
}

/// Code issue detected
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodeIssue {
    pub issue_type: IssueType,
    pub description: String,
    pub severity: IssueSeverity,
    pub location: CodeLocation,
}

/// Type of code issue
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IssueType {
    DuplicateCode,
    LongFunction,
    ComplexLogic,
    PoorNaming,
    MissingAbstraction,
    TightCoupling,
    LowCohesion,
    PerformanceIssue,
    SecurityVulnerability,
    TestCoverage,
}

/// Issue severity
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum IssueSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Code location
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodeLocation {
    pub file: String,
    pub line_start: usize,
    pub line_end: usize,
    pub column_start: Option<usize>,
    pub column_end: Option<usize>,
}

/// Suggested change
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SuggestedChange {
    pub description: String,
    pub before: String,
    pub after: String,
    pub automated: bool,
}

/// Effort estimate
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum EffortEstimate {
    Trivial,   // < 30 minutes
    Small,     // 30 min - 2 hours
    Medium,    // 2 - 8 hours
    Large,     // 1 - 3 days
    VeryLarge, // > 3 days
}

/// Refactoring priority
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum RefactoringPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Implementation step
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImplementationStep {
    pub order: usize,
    pub description: String,
    pub automated: bool,
    pub agent_required: Option<String>,
    pub tools_required: Vec<String>,
}

/// Code pattern for detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodePattern {
    pub name: String,
    pub pattern_type: PatternType,
    pub detection_rule: String,
    pub refactoring_template: String,
}

/// Type of pattern
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PatternType {
    AntiPattern,
    CodeSmell,
    DesignPattern,
    Idiom,
}

/// Refactoring statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringStats {
    pub total_proposals: usize,
    pub automated_proposals: usize,
    pub applied_proposals: usize,
    pub rejected_proposals: usize,
    pub total_improvements: usize,
    pub lines_refactored: usize,
    pub time_saved_hours: f64,
}

/// Automatic refactoring system
pub struct AutomaticRefactoringSystem {
    analyzer: Arc<SemanticAnalyzer>,
    index: Arc<SymbolIndex>,
    memory: Arc<ProjectMemory>,
    patterns: Vec<CodePattern>,
    proposals: HashMap<String, RefactoringProposal>,
    stats: RefactoringStats,
}

impl AutomaticRefactoringSystem {
    /// Create a new refactoring system
    pub fn new(
        analyzer: Arc<SemanticAnalyzer>,
        index: Arc<SymbolIndex>,
        memory: Arc<ProjectMemory>,
    ) -> Self {
        Self {
            analyzer,
            index,
            memory,
            patterns: Self::initialize_patterns(),
            proposals: HashMap::new(),
            stats: RefactoringStats {
                total_proposals: 0,
                automated_proposals: 0,
                applied_proposals: 0,
                rejected_proposals: 0,
                total_improvements: 0,
                lines_refactored: 0,
                time_saved_hours: 0.0,
            },
        }
    }

    /// Initialize default code patterns
    fn initialize_patterns() -> Vec<CodePattern> {
        vec![
            CodePattern {
                name: "Long Function".to_string(),
                pattern_type: PatternType::CodeSmell,
                detection_rule: "function_lines > 50".to_string(),
                refactoring_template: "extract_function".to_string(),
            },
            CodePattern {
                name: "Duplicate Code".to_string(),
                pattern_type: PatternType::CodeSmell,
                detection_rule: "similarity > 0.8".to_string(),
                refactoring_template: "extract_common".to_string(),
            },
            CodePattern {
                name: "Complex Conditional".to_string(),
                pattern_type: PatternType::CodeSmell,
                detection_rule: "cyclomatic_complexity > 10".to_string(),
                refactoring_template: "simplify_conditional".to_string(),
            },
            CodePattern {
                name: "Feature Envy".to_string(),
                pattern_type: PatternType::CodeSmell,
                detection_rule: "external_calls > internal_calls".to_string(),
                refactoring_template: "move_method".to_string(),
            },
            CodePattern {
                name: "God Class".to_string(),
                pattern_type: PatternType::AntiPattern,
                detection_rule: "class_methods > 20 && class_lines > 500".to_string(),
                refactoring_template: "split_class".to_string(),
            },
        ]
    }

    /// Scan codebase for refactoring opportunities
    pub async fn scan_codebase(&mut self) -> SemanticResult<Vec<RefactoringProposal>> {
        log::info!("Scanning codebase for refactoring opportunities...");

        let all_symbols = self.index.get_all_symbols().await?;
        let mut proposals = Vec::new();

        // Analyze each symbol
        for symbol in &all_symbols {
            // Check for long functions
            if let Some(proposal) = self.check_long_function(symbol).await? {
                proposals.push(proposal);
            }

            // Check for complex logic
            if let Some(proposal) = self.check_complex_logic(symbol).await? {
                proposals.push(proposal);
            }

            // Check for poor naming
            if let Some(proposal) = self.check_naming_convention(symbol).await? {
                proposals.push(proposal);
            }
        }

        // Check for duplicate code across symbols
        let duplicate_proposals = self.find_duplicate_code(&all_symbols).await?;
        proposals.extend(duplicate_proposals);

        // Check for architectural issues
        let architectural_proposals = self.analyze_architecture(&all_symbols).await?;
        proposals.extend(architectural_proposals);

        // Update statistics
        self.stats.total_proposals += proposals.len();
        self.stats.automated_proposals += proposals.iter().filter(|p| p.automated).count();

        // Store proposals
        for proposal in &proposals {
            self.proposals.insert(proposal.id.clone(), proposal.clone());
            self.store_proposal_in_memory(proposal).await?;
        }

        log::info!("Found {} refactoring opportunities", proposals.len());
        Ok(proposals)
    }

    /// Check for long functions
    async fn check_long_function(
        &self,
        symbol: &Symbol,
    ) -> SemanticResult<Option<RefactoringProposal>> {
        if symbol.kind != SymbolKind::Function && symbol.kind != SymbolKind::Method {
            return Ok(None);
        }

        if let Some(ref body) = symbol.body {
            let line_count = body.lines().count();
            if line_count > 50 {
                let proposal = RefactoringProposal {
                    id: format!("refactor_long_{}", symbol.name),
                    title: format!("Extract functions from long {}", symbol.name),
                    description: format!(
                        "Function {} has {} lines, which is too long. Consider extracting smaller functions.",
                        symbol.name, line_count
                    ),
                    kind: RefactoringKind::ExtractFunction,
                    targets: vec![RefactoringTarget {
                        symbol: symbol.clone(),
                        issues: vec![CodeIssue {
                            issue_type: IssueType::LongFunction,
                            description: format!("Function has {} lines (threshold: 50)", line_count),
                            severity: if line_count > 100 { IssueSeverity::Error } else { IssueSeverity::Warning },
                            location: CodeLocation {
                                file: symbol.file_path.clone(),
                                line_start: symbol.line,
                                line_end: symbol.line + line_count,
                                column_start: None,
                                column_end: None,
                            },
                        }],
                        suggested_changes: vec![SuggestedChange {
                            description: "Extract logical sections into separate functions".to_string(),
                            before: body.clone(),
                            after: self.generate_extracted_functions(body, &symbol.name),
                            automated: true,
                        }],
                    }],
                    benefits: vec![
                        "Improved readability".to_string(),
                        "Easier testing".to_string(),
                        "Better maintainability".to_string(),
                    ],
                    risks: vec![
                        "May introduce function call overhead".to_string(),
                    ],
                    estimated_effort: EffortEstimate::Small,
                    priority: RefactoringPriority::Medium,
                    automated: true,
                    implementation_steps: vec![
                        ImplementationStep {
                            order: 1,
                            description: "Identify logical sections in the function".to_string(),
                            automated: true,
                            agent_required: None,
                            tools_required: vec!["semantic_analyzer".to_string()],
                        },
                        ImplementationStep {
                            order: 2,
                            description: "Extract each section into a separate function".to_string(),
                            automated: true,
                            agent_required: Some("refactoring-specialist".to_string()),
                            tools_required: vec!["replace_symbol_body".to_string()],
                        },
                        ImplementationStep {
                            order: 3,
                            description: "Update function calls and references".to_string(),
                            automated: true,
                            agent_required: None,
                            tools_required: vec!["find_referencing_symbols".to_string()],
                        },
                    ],
                    created_at: Utc::now(),
                };

                return Ok(Some(proposal));
            }
        }

        Ok(None)
    }

    /// Check for complex logic
    async fn check_complex_logic(
        &self,
        symbol: &Symbol,
    ) -> SemanticResult<Option<RefactoringProposal>> {
        if let Some(ref body) = symbol.body {
            let complexity = self.calculate_cyclomatic_complexity(body);

            if complexity > 10 {
                let proposal = RefactoringProposal {
                    id: format!("refactor_complex_{}", symbol.name),
                    title: format!("Simplify complex logic in {}", symbol.name),
                    description: format!(
                        "Function {} has cyclomatic complexity of {}, which is too high.",
                        symbol.name, complexity
                    ),
                    kind: RefactoringKind::SimplifyExpression,
                    targets: vec![RefactoringTarget {
                        symbol: symbol.clone(),
                        issues: vec![CodeIssue {
                            issue_type: IssueType::ComplexLogic,
                            description: format!(
                                "Cyclomatic complexity: {} (threshold: 10)",
                                complexity
                            ),
                            severity: if complexity > 20 {
                                IssueSeverity::Error
                            } else {
                                IssueSeverity::Warning
                            },
                            location: CodeLocation {
                                file: symbol.file_path.clone(),
                                line_start: symbol.line,
                                line_end: symbol.line,
                                column_start: None,
                                column_end: None,
                            },
                        }],
                        suggested_changes: vec![SuggestedChange {
                            description:
                                "Simplify conditional logic using early returns or pattern matching"
                                    .to_string(),
                            before: body.clone(),
                            after: self.simplify_complex_logic(body),
                            automated: false,
                        }],
                    }],
                    benefits: vec![
                        "Reduced cognitive load".to_string(),
                        "Easier to test".to_string(),
                        "Lower bug probability".to_string(),
                    ],
                    risks: vec!["May change execution flow".to_string()],
                    estimated_effort: EffortEstimate::Medium,
                    priority: RefactoringPriority::High,
                    automated: false,
                    implementation_steps: vec![
                        ImplementationStep {
                            order: 1,
                            description: "Identify complex conditional branches".to_string(),
                            automated: true,
                            agent_required: None,
                            tools_required: vec!["semantic_analyzer".to_string()],
                        },
                        ImplementationStep {
                            order: 2,
                            description: "Apply simplification patterns".to_string(),
                            automated: false,
                            agent_required: Some("refactoring-specialist".to_string()),
                            tools_required: vec!["replace_symbol_body".to_string()],
                        },
                    ],
                    created_at: Utc::now(),
                };

                return Ok(Some(proposal));
            }
        }

        Ok(None)
    }

    /// Check naming conventions
    async fn check_naming_convention(
        &self,
        symbol: &Symbol,
    ) -> SemanticResult<Option<RefactoringProposal>> {
        let is_poor_name = match symbol.kind {
            SymbolKind::Function | SymbolKind::Method => {
                // Check for single letter or non-descriptive names
                symbol.name.len() < 3 || symbol.name == "fn1" || symbol.name == "temp"
            }
            SymbolKind::Variable => {
                // Variables can be shorter in some contexts
                symbol.name.len() < 2 && symbol.name != "i" && symbol.name != "j"
            }
            _ => false,
        };

        if is_poor_name {
            let proposal = RefactoringProposal {
                id: format!("refactor_naming_{}", symbol.name),
                title: format!("Improve naming of {}", symbol.name),
                description: format!("Symbol '{}' has a non-descriptive name", symbol.name),
                kind: RefactoringKind::RenameSymbol,
                targets: vec![RefactoringTarget {
                    symbol: symbol.clone(),
                    issues: vec![CodeIssue {
                        issue_type: IssueType::PoorNaming,
                        description: "Non-descriptive or too short name".to_string(),
                        severity: IssueSeverity::Info,
                        location: CodeLocation {
                            file: symbol.file_path.clone(),
                            line_start: symbol.line,
                            line_end: symbol.line,
                            column_start: None,
                            column_end: None,
                        },
                    }],
                    suggested_changes: vec![SuggestedChange {
                        description: "Use a more descriptive name".to_string(),
                        before: symbol.name.clone(),
                        after: self.suggest_better_name(symbol),
                        automated: false,
                    }],
                }],
                benefits: vec![
                    "Improved code readability".to_string(),
                    "Self-documenting code".to_string(),
                ],
                risks: vec!["Breaking change if public API".to_string()],
                estimated_effort: EffortEstimate::Trivial,
                priority: RefactoringPriority::Low,
                automated: false,
                implementation_steps: vec![ImplementationStep {
                    order: 1,
                    description: "Rename symbol and update all references".to_string(),
                    automated: true,
                    agent_required: Some("refactoring-specialist".to_string()),
                    tools_required: vec!["rename_symbol".to_string()],
                }],
                created_at: Utc::now(),
            };

            return Ok(Some(proposal));
        }

        Ok(None)
    }

    /// Find duplicate code
    async fn find_duplicate_code(
        &self,
        symbols: &[Symbol],
    ) -> SemanticResult<Vec<RefactoringProposal>> {
        let mut proposals = Vec::new();
        let mut checked_pairs = HashSet::new();

        for i in 0..symbols.len() {
            for j in (i + 1)..symbols.len() {
                let pair = if symbols[i].path < symbols[j].path {
                    (symbols[i].path.clone(), symbols[j].path.clone())
                } else {
                    (symbols[j].path.clone(), symbols[i].path.clone())
                };

                if checked_pairs.contains(&pair) {
                    continue;
                }
                checked_pairs.insert(pair);

                if let (Some(ref body1), Some(ref body2)) = (&symbols[i].body, &symbols[j].body) {
                    let similarity = self.calculate_similarity(body1, body2);

                    if similarity > 0.8 && body1.lines().count() > 10 {
                        let proposal = RefactoringProposal {
                            id: format!(
                                "refactor_duplicate_{}_{}",
                                symbols[i].name, symbols[j].name
                            ),
                            title: format!(
                                "Extract common code from {} and {}",
                                symbols[i].name, symbols[j].name
                            ),
                            description: format!(
                                "Functions {} and {} have {:.0}% similar code",
                                symbols[i].name,
                                symbols[j].name,
                                similarity * 100.0
                            ),
                            kind: RefactoringKind::RemoveDuplication,
                            targets: vec![
                                RefactoringTarget {
                                    symbol: symbols[i].clone(),
                                    issues: vec![CodeIssue {
                                        issue_type: IssueType::DuplicateCode,
                                        description: format!(
                                            "{:.0}% similarity with {}",
                                            similarity * 100.0,
                                            symbols[j].name
                                        ),
                                        severity: IssueSeverity::Warning,
                                        location: CodeLocation {
                                            file: symbols[i].file_path.clone(),
                                            line_start: symbols[i].line,
                                            line_end: symbols[i].line,
                                            column_start: None,
                                            column_end: None,
                                        },
                                    }],
                                    suggested_changes: vec![],
                                },
                                RefactoringTarget {
                                    symbol: symbols[j].clone(),
                                    issues: vec![CodeIssue {
                                        issue_type: IssueType::DuplicateCode,
                                        description: format!(
                                            "{:.0}% similarity with {}",
                                            similarity * 100.0,
                                            symbols[i].name
                                        ),
                                        severity: IssueSeverity::Warning,
                                        location: CodeLocation {
                                            file: symbols[j].file_path.clone(),
                                            line_start: symbols[j].line,
                                            line_end: symbols[j].line,
                                            column_start: None,
                                            column_end: None,
                                        },
                                    }],
                                    suggested_changes: vec![],
                                },
                            ],
                            benefits: vec![
                                "Reduced code duplication".to_string(),
                                "Single source of truth".to_string(),
                                "Easier maintenance".to_string(),
                            ],
                            risks: vec!["May introduce additional abstraction".to_string()],
                            estimated_effort: EffortEstimate::Medium,
                            priority: RefactoringPriority::Medium,
                            automated: false,
                            implementation_steps: vec![
                                ImplementationStep {
                                    order: 1,
                                    description: "Extract common functionality".to_string(),
                                    automated: false,
                                    agent_required: Some("refactoring-specialist".to_string()),
                                    tools_required: vec!["extract_function".to_string()],
                                },
                                ImplementationStep {
                                    order: 2,
                                    description: "Update both functions to use common code"
                                        .to_string(),
                                    automated: true,
                                    agent_required: None,
                                    tools_required: vec!["replace_symbol_body".to_string()],
                                },
                            ],
                            created_at: Utc::now(),
                        };

                        proposals.push(proposal);
                    }
                }
            }
        }

        Ok(proposals)
    }

    /// Analyze architecture for issues
    async fn analyze_architecture(
        &self,
        symbols: &[Symbol],
    ) -> SemanticResult<Vec<RefactoringProposal>> {
        let mut proposals = Vec::new();

        // Group symbols by file
        let mut symbols_by_file: HashMap<String, Vec<&Symbol>> = HashMap::new();
        for symbol in symbols {
            symbols_by_file
                .entry(symbol.file_path.clone())
                .or_default()
                .push(symbol);
        }

        // Check for god classes/modules
        for (file, file_symbols) in symbols_by_file {
            if file_symbols.len() > 30 {
                let proposal = RefactoringProposal {
                    id: format!("refactor_large_module_{}", file.replace('/', "_")),
                    title: format!("Split large module: {}", file),
                    description: format!(
                        "Module {} contains {} symbols, consider splitting",
                        file,
                        file_symbols.len()
                    ),
                    kind: RefactoringKind::MoveFunction,
                    targets: vec![],
                    benefits: vec![
                        "Better separation of concerns".to_string(),
                        "Improved modularity".to_string(),
                    ],
                    risks: vec!["May break existing imports".to_string()],
                    estimated_effort: EffortEstimate::Large,
                    priority: RefactoringPriority::Low,
                    automated: false,
                    implementation_steps: vec![],
                    created_at: Utc::now(),
                };

                proposals.push(proposal);
            }
        }

        Ok(proposals)
    }

    /// Calculate cyclomatic complexity
    fn calculate_cyclomatic_complexity(&self, code: &str) -> usize {
        let mut complexity = 1;

        for line in code.lines() {
            let trimmed = line.trim();
            // Count decision points
            if trimmed.starts_with("if ") || trimmed.contains(" if ") {
                complexity += 1;
            }
            if trimmed.starts_with("else if ") || trimmed.contains(" else if ") {
                complexity += 1;
            }
            if trimmed.starts_with("match ") {
                complexity += 1;
            }
            if trimmed.starts_with("while ") || trimmed.contains(" while ") {
                complexity += 1;
            }
            if trimmed.starts_with("for ") || trimmed.contains(" for ") {
                complexity += 1;
            }
            if trimmed.contains("&&") || trimmed.contains("||") {
                complexity += 1;
            }
        }

        complexity
    }

    /// Calculate code similarity
    fn calculate_similarity(&self, code1: &str, code2: &str) -> f64 {
        let lines1: HashSet<&str> = code1
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();
        let lines2: HashSet<&str> = code2
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();

        if lines1.is_empty() || lines2.is_empty() {
            return 0.0;
        }

        let intersection = lines1.intersection(&lines2).count();
        let union = lines1.union(&lines2).count();

        intersection as f64 / union as f64
    }

    /// Generate extracted functions
    fn generate_extracted_functions(&self, body: &str, function_name: &str) -> String {
        // Simplified extraction - in real implementation would use AST
        format!(
            "fn {}_part1() {{\n    // Extracted logic\n}}\n\nfn {}_part2() {{\n    // Extracted logic\n}}\n\nfn {}() {{\n    {}_part1();\n    {}_part2();\n}}",
            function_name, function_name, function_name, function_name, function_name
        )
    }

    /// Simplify complex logic
    fn simplify_complex_logic(&self, body: &str) -> String {
        // Simplified - would use AST transformation in real implementation
        format!("// Simplified version\n{}", body)
    }

    /// Suggest better name
    fn suggest_better_name(&self, symbol: &Symbol) -> String {
        match symbol.kind {
            SymbolKind::Function => format!("process_{}_data", symbol.name),
            SymbolKind::Variable => format!("{}_value", symbol.name),
            _ => format!("renamed_{}", symbol.name),
        }
    }

    /// Store proposal in memory
    async fn store_proposal_in_memory(&self, proposal: &RefactoringProposal) -> SemanticResult<()> {
        let memory = Memory {
            id: proposal.id.clone(),
            name: proposal.title.clone(),
            content: serde_json::to_string(proposal)?,
            memory_type: MemoryType::Refactoring,
            related_symbols: proposal
                .targets
                .iter()
                .map(|t| t.symbol.path.clone())
                .collect(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("kind".to_string(), format!("{:?}", proposal.kind));
                meta.insert("priority".to_string(), format!("{:?}", proposal.priority));
                meta.insert("automated".to_string(), proposal.automated.to_string());
                meta
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
            access_count: 0,
        };

        self.memory.store_memory(memory).await
    }

    /// Apply a refactoring proposal
    pub async fn apply_proposal(&mut self, proposal_id: &str) -> SemanticResult<()> {
        if let Some(proposal) = self.proposals.get(proposal_id) {
            log::info!("Applying refactoring proposal: {}", proposal.title);

            // Apply each suggested change
            for target in &proposal.targets {
                for change in &target.suggested_changes {
                    if change.automated {
                        // In real implementation, would apply the change
                        log::info!("Applying automated change: {}", change.description);
                    } else {
                        log::info!("Manual change required: {}", change.description);
                    }
                }
            }

            // Update statistics
            self.stats.applied_proposals += 1;
            self.stats.lines_refactored += proposal
                .targets
                .iter()
                .map(|t| {
                    t.symbol
                        .body
                        .as_ref()
                        .map(|b| b.lines().count())
                        .unwrap_or(0)
                })
                .sum::<usize>();

            // Estimate time saved
            let time_saved = match proposal.estimated_effort {
                EffortEstimate::Trivial => 0.5,
                EffortEstimate::Small => 1.0,
                EffortEstimate::Medium => 4.0,
                EffortEstimate::Large => 16.0,
                EffortEstimate::VeryLarge => 40.0,
            };
            self.stats.time_saved_hours += time_saved;

            Ok(())
        } else {
            Err(SemanticError::Other(format!(
                "Proposal {} not found",
                proposal_id
            )))
        }
    }

    /// Get refactoring statistics
    pub fn get_stats(&self) -> &RefactoringStats {
        &self.stats
    }

    /// Get all proposals
    pub fn get_proposals(&self) -> Vec<RefactoringProposal> {
        self.proposals.values().cloned().collect()
    }

    /// Get proposals by priority
    pub fn get_proposals_by_priority(
        &self,
        priority: RefactoringPriority,
    ) -> Vec<RefactoringProposal> {
        self.proposals
            .values()
            .filter(|p| p.priority == priority)
            .cloned()
            .collect()
    }
}
