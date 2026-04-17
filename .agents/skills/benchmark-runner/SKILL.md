---
name: benchmark-runner
description: Run performance benchmarks with criterion. Compare against baselines and profile hotspots.
user-invocable: true
argument-hint: "[bench-name]"
---

Run performance benchmarks on ccswarm.

## Run Benchmarks

```bash
cargo build --release --workspace

# All benchmarks
cargo bench --workspace

# Specific benchmark
cargo bench -- "$ARGUMENTS"

# Save baseline for comparison
cargo bench -- --save-baseline before
# ... make changes ...
cargo bench -- --baseline before
```

## Profile

```bash
cargo flamegraph --bench ${1:-orchestrator_bench}
```

## Key Metrics

| Area | Target |
|------|--------|
| Task delegation | < 1ms |
| Full workflow | < 100ms |
| Memory per agent | < 10MB |
| Variance | < 20% |

Results in `target/criterion/report/index.html`.
