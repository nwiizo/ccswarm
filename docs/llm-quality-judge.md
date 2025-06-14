# LLM Quality Judge System

## Overview

The LLM Quality Judge is an advanced code quality evaluation system inspired by Anthropic's multi-agent research system. It provides intelligent, context-aware quality assessments of agent-generated code using both LLM-based evaluation (Claude) and heuristic analysis.

## Key Features

- **Multi-dimensional Evaluation**: Assesses code across 8 quality dimensions
- **Role-specific Scoring**: Adjusts evaluation criteria based on agent specialization
- **Intelligent Issue Detection**: Categorizes issues by severity and type
- **Actionable Remediation**: Generates specific fix instructions with effort estimates
- **Performance Optimization**: Caches evaluations to reduce API calls
- **Flexible Rubrics**: Customizable quality standards for different contexts

## Architecture

### Quality Dimensions

1. **Correctness** (30% weight) - Does the code solve the intended problem?
2. **Maintainability** (20% weight) - Is the code readable and well-structured?
3. **Test Quality** (20% weight) - Are there adequate tests with good coverage?
4. **Security** (15% weight) - Are security best practices followed?
5. **Performance** (10% weight) - Is the code optimized for efficiency?
6. **Documentation** (5% weight) - Is the code well-documented?
7. **Architecture** - Does the code follow good architectural patterns?
8. **Error Handling** - Are errors properly handled and recovered?

### Role-Specific Adjustments

#### Frontend Agents
- Higher weight on accessibility (10%)
- Performance focus on client-side optimization
- UI/UX considerations in evaluation

#### Backend Agents
- Security gets highest priority (20%)
- Test quality emphasized (25%)
- Performance focused on scalability

#### DevOps Agents
- Security paramount (30%)
- Infrastructure best practices
- Configuration management focus

## Usage

### Basic Evaluation

```rust
use ccswarm::orchestrator::llm_quality_judge::LLMQualityJudge;

// Create a quality judge
let mut judge = LLMQualityJudge::new();

// Evaluate task output
let evaluation = judge.evaluate_task(
    &task,
    &result,
    &agent_role,
    &workspace_path
).await?;

// Check results
if evaluation.passes_standards {
    println!("Quality standards met!");
} else {
    // Generate fix instructions
    let instructions = judge.generate_fix_instructions(
        &evaluation.issues,
        agent_role.name()
    );
}
```

### Custom Rubrics

```rust
use ccswarm::orchestrator::llm_quality_judge::QualityRubric;

// Create custom rubric for security-critical applications
let mut rubric = QualityRubric::default();
rubric.dimensions.insert("security".to_string(), 0.4); // 40% weight
rubric.thresholds.insert("security".to_string(), 0.95); // 95% minimum

let security_judge = LLMQualityJudge::with_rubric(rubric);
```

## Integration with ccswarm

The LLM Quality Judge is integrated into the orchestrator's quality review cycle:

1. **Task Completion** - Agent completes a task
2. **Quality Evaluation** - Judge evaluates the output
3. **Issue Detection** - Problems are categorized and prioritized
4. **Remediation Creation** - Fix tasks are automatically generated
5. **Agent Assignment** - Original agent receives remediation task
6. **Re-evaluation** - Fixed code is re-evaluated

## Issue Categories

- **Security** - Vulnerabilities, hardcoded secrets, unsafe practices
- **Performance** - Inefficient algorithms, resource waste
- **TestCoverage** - Missing or inadequate tests
- **CodeComplexity** - High cyclomatic complexity, poor structure
- **Documentation** - Missing comments, unclear code
- **ErrorHandling** - Unhandled exceptions, poor recovery
- **Architecture** - Design pattern violations, poor modularity
- **BestPractices** - Convention violations, anti-patterns
- **Accessibility** - Missing ARIA labels, keyboard navigation
- **TypeSafety** - Type errors, unsafe casts

## Severity Levels

1. **Critical** - Must fix immediately (security vulnerabilities)
2. **High** - Should fix before deployment (missing tests)
3. **Medium** - Should fix soon (complexity issues)
4. **Low** - Nice to fix (documentation gaps)

## Configuration

### Environment Variables

- `CCSWARM_USE_CLAUDE_JUDGE` - Enable Claude-based evaluation (default: true)
- `CCSWARM_QUALITY_THRESHOLD` - Overall score required to pass (default: 0.85)
- `CCSWARM_JUDGE_CACHE_SIZE` - Number of evaluations to cache (default: 100)

### Quality Standards

Configure in `ccswarm.json`:

```json
{
  "quality_standards": {
    "min_test_coverage": 0.85,
    "max_complexity": 10,
    "security_scan_required": true,
    "performance_threshold": "100ms"
  }
}
```

## Performance Considerations

- **Caching**: Evaluations are cached by task ID and output hash
- **Batch Processing**: Multiple evaluations can be batched
- **Fallback**: Heuristic evaluation when Claude is unavailable
- **Async**: All evaluations are non-blocking

## Best Practices

1. **Regular Reviews**: Run quality reviews every 30 seconds
2. **Early Detection**: Catch issues before they compound
3. **Iterative Improvement**: Track quality trends over time
4. **Custom Rubrics**: Adjust weights for your domain
5. **Fix Prioritization**: Address critical issues first

## Troubleshooting

### Common Issues

**Low Confidence Scores**
- Check if Claude API is accessible
- Verify workspace paths are correct
- Ensure code samples are complete

**False Positives**
- Adjust rubric thresholds
- Add context to task descriptions
- Use role-specific evaluations

**Performance Impact**
- Increase cache size
- Use heuristics for non-critical code
- Batch related evaluations

## Future Enhancements

- **Learning System**: Improve evaluations based on feedback
- **Cross-Agent Analysis**: Detect architectural inconsistencies
- **Performance Profiling**: Actual runtime analysis
- **Security Scanning**: Integration with security tools
- **Custom Plugins**: Extensible evaluation system