# CLAUDE.md

## Project Overview

ccswarm v0.6.0 — Multi-Agent Orchestration Pipeline.

複数のClaude Codeエージェントをパイプラインで協調させるワークフローエンジン。
OK/NG駆動設計: ユーザーは `y` か `n` しか押さない。

## Usage

```bash
ccswarm                            # 対話モード: 何を作るか聞いてくれる
ccswarm "勤怠管理アプリを作って"      # 即実行: pipeline → test → commit → PR
ccswarm runs                       # 過去のrun一覧
ccswarm pieces                     # 利用可能なワークフロー
ccswarm doctor                     # 環境チェック
```

## Post-Pipeline Flow (自動)

```
Pipeline完了 → テスト自動実行 → 失敗なら自動修復(最大3回)
→ "Commit? [Y/n]" → "Create PR? [Y/n]"
```

## Quick Commands (development)

```bash
cargo fmt && cargo clippy -- -D warnings && cargo test  # Before commit
cargo run -p ccswarm -- --help                          # Full CLI
```

## Workspace Architecture

```
ccswarm (workflow engine) ──depends on──> ai-session (session management)
                                              ──depends on──> portable-pty, tokio, zstd
```

### ccswarm crate

| Module | Purpose |
|--------|---------|
| `cli/` | 3 entry modes (interactive / direct task / subcommands) |
| `workflow/` | PieceEngine, Pipeline, Faceted prompting, movement reports |
| `session/` | AISessionBridge (Claude CLI execution, --dangerously-skip-permissions, --allowed-tools, --system-prompt, retry) |
| `events/` | NDJSON EventRecorder, duration tracking, run summaries |
| `agent/` | AgentRole, Type-State TaskBuilder |
| `identity/` | AgentIdentity, role boundaries |
| `hooks/` | HookRegistry |
| `coordination/` | AgentMailbox, conversion layer |

### ai-session crate

| Module | Purpose |
|--------|---------|
| `core/` | AISession, SessionManager, PTY/headless |
| `context/` | TokenEfficientHistory (zstd compression) |
| `output/` | OutputParser (cargo/Playwright/npm/Jest patterns) |
| `persistence/` | Session snapshots |

## Key Design Decisions

- **OK/NG駆動**: ユーザーの操作を y/n に削減。テスト自動実行、自動修復ループ。
- **Claude Code CLI フラグ自動マッピング**: Movement YAML → --allowed-tools, --model, --system-prompt
- **--dangerously-skip-permissions**: 全movement共通（non-interactiveパイプライン実行のため）
- **ローカルcomplete**: 終端movementはClaude呼び出し不要（instant）
- **部分成功**: タイムアウトでもmovements完了分は成功扱い

## Builtin Pieces

| Piece | Description | Agents |
|-------|-------------|--------|
| `default` | plan → implement → review → complete | planner, coder, reviewer |
| `team` | plan → parallel(frontend + backend) → review → complete | planner, frontend-specialist, backend-specialist, supervisor |
| `quick` | single-shot execution (1 step) | coder |
| `review-fix` | review → fix loop | reviewer, coder |
| `research` | investigate → report | researcher |

Custom: `.ccswarm/pieces/*.yaml`

### Multi-Agent Pipeline Example

```yaml
# team piece: plan → parallel agents → supervisor review
movements:
  - id: plan
    persona: planner
  - id: parallel-implement
    parallel: true
    sub_movements: [frontend-impl, backend-impl]
  - id: frontend-impl
    agent: frontend-specialist    # Routes to .claude/agents/frontend-specialist.md
  - id: backend-impl
    agent: backend-specialist     # Routes to .claude/agents/backend-specialist.md
  - id: review
    persona: supervisor           # Final validation
```

## Personas (builtin)

planner, coder, reviewer, researcher, supervisor, ai-antipattern-reviewer

## Pipeline Learnings

- タスク記述は簡潔に（500語以下）。長いとimplementがタイムアウト。
- `{task}`, `{plan_output}` テンプレート変数で movement 間コンテキスト受け渡し。
- `pass_previous_response: false` で fix movement のコンテキストをリセット。

## Rules

- [development-standards](.claude/rules/development-standards.md)
- [architecture-patterns](.claude/rules/architecture-patterns.md)
- [security-guidelines](.claude/rules/security-guidelines.md)
- [performance](.claude/rules/performance.md)

## Documentation

@docs/ARCHITECTURE.md
@docs/APPLICATION_SPEC.md
