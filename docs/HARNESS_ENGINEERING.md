# Harness Engineering for ccswarm

Last updated: 2026-03-11

## 1. What is Harness Engineering?

Harness Engineering is the practice of designing and operating a deterministic evaluation system (a “harness”) that executes workflows under controlled conditions and validates outcomes against clear oracles. For ccswarm (multi‑agent orchestration), the harness ensures that flows/movements, providers, and guards collectively deliver correct, safe, and cost‑effective results.

Goals:
- Determinism and reproducibility: same inputs → same outcomes (within stochastic bounds)
- Observability and evidence: rich traces (NDJSON), summaries, artifacts
- Policy enforcement: quality gates, permissions, HITL checkpoints
- Fitness metrics: correctness, performance, cost, reliability, flakiness

Non‑Goals:
- Replace unit/integration tests — the harness complements them by validating system‑level behavior.

## 2. Design Principles
- Hermetic by default: isolate filesystem, network, and time where feasible; seed RNG.
- Idempotent scenarios: safe to re‑run; cleanup/teardown always succeeds.
- Small, composable steps: favor narrow movements with strong oracles.
- Evidence first: assertions must be grounded in observable outputs or events.
- Budget awareness: test within defined time/token/resource envelopes.

## 3. Architecture
- Scenario Loader: reads YAML scenarios; expands matrices/tags/filters.
- Fixture Manager: prepares inputs (repos, datasets), manages setup/teardown.
- Executor: runs `pipeline` (flow/movements) with timeouts and captures outputs.
- Oracles: evaluate assertions (text, files, tests, metrics, policies).
- Reporter: aggregates results, writes JSON/markdown, surfaces diffs vs baseline.
- Storage: organizes runs under `.ccswarm/runs/<run-id>/` with `events.ndjson` and summaries.

## 4. Scenario Schema v1 (YAML)

Place scenarios under `.ccswarm/harness/` or pass paths explicitly.

```yaml
name: "add-login-form"
tags: ["feature", "frontend"]

flow: "default"              # workflow flow
task: "Create login form"

timeout_secs: 600              # default 600
output_format: text            # text|json|markdown (default text)
seed: 42                       # optional random seed for providers/samplers

matrix:                        # optional Cartesian expansion
  provider: ["claude_code"]
  branch: ["main"]

setup:                         # optional shell or scripted setup
  - "git checkout ${branch}"
  - "cargo fetch"

assert:
  expect_success: true
  expect_text: ["Plan", "Review"]
  forbid_text: ["ABORT", "panic!"]
  files_exist: ["src/", "README.md"]            # paths must exist after run
  command_ok: ["cargo check", "cargo test -q"]   # exit code == 0
  metrics:
    max_duration_secs: 300
    max_tokens: 200000

teardown:                      # optional cleanup steps
  - "git reset --hard && git clean -fdx"
```

Notes:
- `matrix` expands into multiple scenario instances; `name` can be templated (future).
- `setup`/`teardown` are executed in repo context; prefer idempotent commands.
- `command_ok` is best‑effort unless sandbox constraints prohibit execution.

## 5. CLI

```bash
# Run all scenarios in .ccswarm/harness/
ccswarm harness run

# Run a specific scenario file and write a report
ccswarm harness run --scenario .ccswarm/harness/add-login.yaml \
  --report .ccswarm/runs/harness/report.json --format json

# Limit concurrency (defaults to #CPU cores)
ccswarm harness run --dir ./.ccswarm/harness --jobs 4

# List discovered scenarios
ccswarm harness list

# Create a sample scenario file
ccswarm harness init --output .ccswarm/harness/sample.yaml --name my-sample

# Show plan (matrix expansion) without executing
ccswarm harness plan

# Compare current results to a baseline (JSON or Markdown report)
ccswarm harness diff --baseline .ccswarm/harness/baseline.json --format json
ccswarm harness diff --baseline .ccswarm/harness/baseline.json --format markdown

# Approve current report as new baseline
ccswarm harness approve --report .ccswarm/runs/harness/report.json --baseline .ccswarm/harness/baseline.json --force
```

Planned additions:
- `harness plan` — expand matrix, print execution plan
- `harness diff` — compare current results to a saved baseline
- `harness approve` — update golden outputs after manual review

## 6. Oracles and Assertions
- Text oracles: contains/forbid substrings; regex matches (planned)
- File oracles: existence, size bounds, snapshot/golden diffs (planned)
- Command oracles: system commands must succeed (optional; env‑gated)
- Policy oracles: lint/test/security gates; permissions enforced per movement
- Metric oracles: duration, token/cost budget, failure/retry bounds

Failure guidance: include violating assertion names and observed evidence in the report.

## 7. Metrics & KPIs
- Correctness: pass rate, assertion coverage
- Performance: latency per movement, total duration, throughput for batch runs
- Cost: tokens used (if measurable), API calls, CPU/memory (sampling)
- Reliability: flakiness rate, retry counts, mean time to failure/success
- Determinism: variance across N repeated runs with fixed seed

## 8. Flakiness Mitigation
- Seeded operations; minimize nondeterminism (timestamps, randomized names)
- Retry with backoff for network‑bound steps (if allowed); mark non‑hermetic oracles
- Quarantine flaky scenarios; report separately with instability score

## 9. Environments & Matrices
- Dimensions: provider, model, OS, feature flags, permission modes
- Define skip/only rules: skip if not supported (e.g., `net` permission required)
- Cache warmups: prefetch dependencies during setup to reduce variance

## 10. Observability & Storage
- Events: `events.ndjson` with {ts, level, run_id, movement, task_id, message}
- Summaries: per‑scenario JSON with assertions and outcomes
- Attachments: stdout/stderr of commands, diffs, screenshots (future)

Harness report auto‑includes a best‑effort summary of the most recent `events.ndjson` under `.ccswarm/runs/` (path, total events, counts by level, first/last timestamp) for each executed scenario instance.

## 11. Security & Permissions in Harness
- Mirror movement permission modes: view|edit|exec|net
- Default to least privilege; deny network unless scenario sets `requires_net: true`
- Redact secrets from logs; mask tokens and credentials

## 12. CI Integration
- Exit codes: non‑zero if any scenario fails (gate)
- Artifacts: upload report JSON and NDJSON traces
- Baselines: compare against last successful run; flag regressions

## 13. Multi‑Agent Specifics
- Invariants: no deadlock/livelock; bounded queue growth; no orphan processes
- Fairness: scheduling does not starve a role under typical loads
- Safety: edits respect permission scopes; risky steps traverse HITL gates
- Coverage: exercise planner/executor/reviewer paths and fix loops

## 14. Current Implementation (v0)
- Minimal CLI: `harness run|list`
- Scenario fields: `flow`, `task`, `timeout_secs`, `output_format`, `assert` (expect_success/expect_text/forbid_text)
- Report: JSON or text/markdown; consolidated pass/fail with per‑scenario details

## 15. Roadmap
- v1: setup/teardown hooks, matrix expansion, file/command/metric oracles
- v2: baselines & diffs, golden snapshots, TUI integration, environment matrix
- v3: chaos/failure injection, time virtualization, provider cost accounting

## 16. References

- Stanford CRFM HELM — Framework for evaluation of foundation models
  - Source: docs/vendor_sources/helm/README.md:1
  - Notable: scenario/model separation, diverse metrics packages, pinned deps, CLI (`helm-run`, `helm-summarize`)
- EleutherAI LM Evaluation Harness — Standardized NLP eval harness
  - Source: docs/vendor_sources/lm-evaluation-harness/README.md:1
  - Notable: task plugins, deterministic eval loops, prompt/target adapters, leaderboard comparability
- SWE‑Bench — Software engineering benchmark + harness
  - Source: docs/vendor_sources/SWE-bench/CHANGELOG.md:42
  - Notable: containerized/sandboxed execution, harness API, environment provisioning, reproducibility bug reports
- General Test Harness Patterns
  - Concepts: hermeticism, fixtures, oracles, baselines/goldens, seeds, matrices
  - Mapping: reflected in §2 Design Principles, §3 Architecture, §4 Schema, §6 Oracles, §9 Matrices
