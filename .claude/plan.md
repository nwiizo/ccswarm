# Carlini Patterns Integration Plan

Carlini氏の「Building a C compiler with a team of parallel Claudes」から、ccswarmに取り込む6つの設計パターンを実装する。

## Pattern Mapping (記事 → ccswarm)

| # | Carlini Pattern | ccswarm Module | Status |
|---|----------------|----------------|--------|
| 1 | Infinite Loop Harness | `session/restart_policy.rs` | **New** |
| 2 | File-based Task Claiming | `coordination/task_claim.rs` | **New** |
| 3 | Context Pollution Prevention | `streaming/context_guard.rs` | **New** |
| 4 | Fast Sampling Mode | `execution/sampling.rs` | **New** |
| 5 | Progress File Auto-Update | `coordination/progress_tracker.rs` | **New** |
| 6 | CI Verification Hook | `hooks/builtin.rs` (extend) | **Extend** |

---

## Phase 1: Session Auto-Restart (Infinite Loop Harness)

**What**: Agent session crashes or exits → automatically restart and resume work.
Carlini: `while true` loop in Docker, session ends → immediate restart.

**File**: `crates/ccswarm/src/session/restart_policy.rs`

```rust
pub struct RestartPolicy {
    pub mode: RestartMode,           // Always, OnFailure, Never
    pub max_restarts: u32,           // Default: 10
    pub backoff: BackoffStrategy,    // Fixed(1s), Exponential(1s, 30s)
    pub cooldown_secs: u64,          // Minimum time between restarts
    pub preserve_context: bool,      // Re-inject last task context on restart
}

pub struct SessionRestarter {
    policy: RestartPolicy,
    restart_count: AtomicU32,
    last_restart: RwLock<Option<Instant>>,
}
```

- `session/mod.rs` に `RestartPolicy` を追加、`AgentSession` に `restarter` フィールド
- `SessionManager::monitor_and_restart()` async method追加

---

## Phase 2: File-based Task Claiming (Deduplication)

**What**: Distributed agent間のタスク重複防止。Text fileベースのロック機構。
Carlini: `current_tasks/` directory + git sync for claim deduplication.

**File**: `crates/ccswarm/src/coordination/task_claim.rs`

```rust
pub struct TaskClaimManager {
    claims_dir: PathBuf,             // coordination/current_tasks/
}

pub struct TaskClaim {
    pub task_id: String,
    pub agent_id: String,
    pub claimed_at: DateTime<Utc>,
    pub description: String,
}
```

- `claim_task()` → ファイル作成（atomic write）、Git commitでsync
- `release_task()` → ファイル削除
- `list_claimed_tasks()` → directory scan
- `is_task_claimed()` → ファイル存在チェック
- Git push conflict = 別agentが先に claim → 自動的に次のタスクへ

---

## Phase 3: Context Pollution Prevention (Output Guard)

**What**: 大量出力によるLLM文脈汚染を防ぐ。結果はN行サマリー、詳細はログへ。
Carlini: "thousands of useless bytes degrade reasoning", grep-friendly ERROR format.

**File**: `crates/ccswarm/src/streaming/context_guard.rs`

```rust
pub struct ContextGuard {
    pub config: ContextGuardConfig,
}

pub struct ContextGuardConfig {
    pub max_context_lines: usize,    // Default: 20
    pub log_full_output: bool,       // Default: true
    pub log_dir: PathBuf,            // logs/agent-output/
    pub summary_format: SummaryFormat, // Compact, Detailed
    pub error_prefix: String,        // "ERROR:" (grep-friendly)
}

pub struct GuardedOutput {
    pub summary: String,             // Truncated for agent context
    pub stats: OutputStats,          // Pre-computed: passed/failed/errors
    pub log_path: PathBuf,           // Full output location
    pub has_errors: bool,
}
```

- Test output: `23 passed, 2 failed. Details: logs/test-run-001.log`
- Build output: `Build FAILED. 3 errors. See logs/build-001.log`
- `MonitoringSystem` / `StreamingManager` に `ContextGuard` を統合

---

## Phase 4: Fast Sampling Mode

**What**: テストの1-10%をランダム実行して高速フィードバック。
Carlini: `--fast` option runs random 1-10% sample, deterministic per-agent.

**File**: `crates/ccswarm/src/execution/sampling.rs`

```rust
pub struct SamplingConfig {
    pub enabled: bool,
    pub sample_ratio: f64,           // 0.01-0.10 (1%-10%)
    pub seed_strategy: SeedStrategy, // PerAgent, Random, Fixed(u64)
    pub always_include: Vec<String>, // Critical tests always run
}

pub struct TestSampler {
    config: SamplingConfig,
}
```

- `TestSampler::sample()` → テストリストからランダム抽出
- Agent IDベースのseedで、各agentが異なるサブセットを実行
- `ParallelConfig` に `sampling: Option<SamplingConfig>` 追加
- CLI: `ccswarm task execute --fast` flag

---

## Phase 5: Progress File Auto-Update

**What**: 新しいcontextに投入されるagentのself-orientation支援。
Carlini: Extensive README + progress files, frequently updated.

**File**: `crates/ccswarm/src/coordination/progress_tracker.rs`

```rust
pub struct ProgressTracker {
    pub progress_file: PathBuf,      // PROGRESS.md
    pub update_interval_secs: u64,   // Default: 60
}

pub struct ProjectProgress {
    pub overall_status: String,
    pub completed_tasks: Vec<TaskSummary>,
    pub active_tasks: Vec<TaskSummary>,
    pub pending_tasks: Vec<TaskSummary>,
    pub known_issues: Vec<String>,
    pub last_updated: DateTime<Utc>,
}
```

- `ProgressTracker::update()` → PROGRESS.md を自動更新
- Task完了/失敗時にhookで呼び出し
- Agent起動時にPROGRESS.mdを読ませてcontext注入
- `CoordinationBus` のメッセージから状態を集約

---

## Phase 6: CI Verification Hook

**What**: 新commitが既存テストを壊さないことを自動検証。
Carlini: "Give Claude a way to verify its own work."

**File**: `crates/ccswarm/src/hooks/builtin.rs` (extend existing)

```rust
pub struct CIVerificationHook {
    pub test_command: String,        // Default: "cargo test"
    pub run_before_commit: bool,     // Default: true
    pub fail_on_regression: bool,    // Default: true
    pub sampling: Option<SamplingConfig>, // Fast mode integration
}
```

- `ExecutionHooks::post_execution()` で実装
- Task完了後にテスト実行、regression検出で自動reject
- Phase 4の`SamplingConfig`と連携（`--fast`モード対応）
- 既存の`SecurityHook`/`LoggingHook`と並列動作

---

## Implementation Order

```
Phase 1 (restart_policy.rs)     ← 他に依存なし、単独実装可
Phase 2 (task_claim.rs)         ← 他に依存なし、単独実装可
Phase 3 (context_guard.rs)      ← streaming/monitoring統合必要
Phase 4 (sampling.rs)           ← execution統合必要
Phase 5 (progress_tracker.rs)   ← coordination統合必要
Phase 6 (CI hook拡張)           ← Phase 4 のSamplingConfig使用
```

Phase 1-2 は並列実装可能。Phase 3-5 も並列可能。Phase 6 は Phase 4 完了後。

## File Changes Summary

| Action | File |
|--------|------|
| **New** | `crates/ccswarm/src/session/restart_policy.rs` |
| **New** | `crates/ccswarm/src/coordination/task_claim.rs` |
| **New** | `crates/ccswarm/src/streaming/context_guard.rs` |
| **New** | `crates/ccswarm/src/execution/sampling.rs` |
| **New** | `crates/ccswarm/src/coordination/progress_tracker.rs` |
| Edit | `crates/ccswarm/src/session/mod.rs` - RestartPolicy統合 |
| Edit | `crates/ccswarm/src/coordination/mod.rs` - re-exports追加 |
| Edit | `crates/ccswarm/src/streaming/mod.rs` - ContextGuard統合 |
| Edit | `crates/ccswarm/src/execution/task_queue.rs` - Sampling統合 |
| Edit | `crates/ccswarm/src/hooks/builtin.rs` - CIVerificationHook追加 |
| Edit | `crates/ccswarm/src/subagent/parallel_executor.rs` - Sampling/ContextGuard統合 |
| Edit | `crates/ccswarm/src/lib.rs` - module宣言追加 |
| Edit | `crates/ccswarm/src/cli/mod.rs` - `--fast` flag追加 |
| Edit | `crates/ccswarm/src/config/mod.rs` - 新Config構造体追加 |
