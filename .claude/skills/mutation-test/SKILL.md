---
name: mutation-test
description: Run mutation tests with cargo-mutants to verify test quality. Reports caught, missed, and timed-out mutants.
user-invocable: true
argument-hint: "[crate-name]"
---

Run mutation tests on the specified crate (default: ccswarm):

```bash
# Install if needed
cargo mutants --version || cargo install cargo-mutants

# List mutants
cargo mutants --list -p $ARGUMENTS | head -50

# Run mutation tests
cargo mutants -p ${1:-ccswarm} --timeout 120 -j 4

# Report results
echo "=== Caught ===" && wc -l mutants.out/caught.txt
echo "=== Missed ===" && cat mutants.out/missed.txt
echo "=== Timeout ===" && wc -l mutants.out/timeout.txt
```

Focus on `missed` mutants in business logic - these indicate insufficient test coverage.
