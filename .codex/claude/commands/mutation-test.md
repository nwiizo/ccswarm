# /mutation-test - Mutation Test Execution

Uses cargo-mutants to run mutation tests and verify test quality.

## Execution Content

1. Check if cargo-mutants is installed
2. Run mutation tests on specified crate
3. Analyze results and report undetected mutants (survivors)

## Commands

```bash
# Installation check
cargo mutants --version || cargo install cargo-mutants

# List mutants (pre-execution check)
cargo mutants --list -p <crate-name>

# Run mutation tests
cargo mutants -p <crate-name> --timeout 120 -j 4

# Check results
cat mutants.out/caught.txt    # Detected mutants
cat mutants.out/missed.txt    # Undetected mutants (needs improvement)
cat mutants.out/timeout.txt   # Timed out mutants
```

## Result Interpretation

| Result | Meaning | Action |
|--------|---------|--------|
| caught | Tests detected the mutant | Good, no action needed |
| missed | Tests failed to detect the mutant | Tests need to be added |
| timeout | Test execution timed out | Consider adjusting timeout value |
| unviable | Mutant cannot be built | No action needed |

## Areas to Focus On

- Functions with many `missed` mutants have insufficient test coverage
- Prioritize addressing `missed` in business logic (usecase/)
- `missed` in error handling (error/) is also important

## Example Output

```
317 mutants tested
- 280 caught (88%)
- 20 missed (6%)
- 10 timeout (3%)
- 7 unviable (2%)
```

## Reference

- [cargo-mutants documentation](https://mutants.rs/)
- Mutation testing is a technique for measuring test quality
- "Killing a mutant" = Tests can detect code changes

## Execution Steps

Run mutation tests on the ccswarm crate:

```bash
# 1. Installation check
cargo mutants --version || cargo install cargo-mutants

# 2. Check mutant list (optional)
cargo mutants --list -p ccswarm | head -50

# 3. Run mutation tests (parallel 4, timeout 120 seconds)
cargo mutants -p ccswarm --timeout 120 -j 4

# 4. Display results summary
echo "=== Caught ===" && wc -l mutants.out/caught.txt
echo "=== Missed ===" && cat mutants.out/missed.txt
echo "=== Timeout ===" && wc -l mutants.out/timeout.txt
```
