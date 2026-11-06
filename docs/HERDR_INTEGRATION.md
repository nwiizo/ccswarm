# Herdr Integration

Status: v0.11.0 specification. Checked against Herdr 0.9.5 (socket protocol
22), Claude Code and Codex CLIs installed on 2026-11-06, and
[takt](https://github.com/nrslib/takt)'s `claude-terminal` provider.

## Product Decision

ccswarm runs local provider CLIs inside [Herdr](https://herdr.dev), the
persistent terminal workspace for coding agents. Every stage that ccswarm
executes on this machine is visible as a Herdr tab, reports its lifecycle to
Herdr, and can be inspected or interrupted from the same workspace the user
already works in. Herdr provides the terminal surface; ccswarm still owns
flows, Sangha decisions, events, and acceptance.

Local provider execution requires Herdr. Outside a Herdr-managed pane,
ccswarm stops before starting a provider and explains how to continue. Remote
A2A execution (`CCSWARM_A2A_ENDPOINT`) does not start a local provider and
does not require Herdr. Commands that do not execute providers, such as
`flow check`, `flow render`, `run list`, and `--help`, also work without it.

## Requirements

| Requirement | Detail |
| --- | --- |
| Herdr | 0.9.5 or later, running ccswarm from one of its panes (`HERDR_ENV=1`, `HERDR_WORKSPACE_ID`) |
| Herdr CLI | `HERDR_BIN_PATH` when Herdr provides it, otherwise `herdr` on `PATH` |
| Platform | Linux or macOS, matching ccswarm's release targets |
| Live Claude stages | Claude Code with `--session-id` and `--append-system-prompt-file` |

`ccswarm doctor` reports the Herdr CLI, the running server, whether the
current process is inside Herdr, and Herdr's Claude/Codex integration state.

## Stage Execution

Each provider call opens one background tab in the caller's Herdr workspace
(`tab create --no-focus --cwd <stage dir> --label "ccswarm <stage> (<provider>)"`).
The user's pane layout and focus do not change, and parallel stages, Sangha
members, and team-leader workers each get their own tab. ccswarm closes the
tab when the call finishes, fails, times out, or is cancelled; closing the tab
stops a provider that is still running.

Flow YAML selects how the provider runs in that tab:

```yaml
stages:
  - id: review
    provider: claude
    execution: live      # headless (default) | live
```

`execution` is inherited by Sangha members and team-leader workers.

### Headless (default)

ccswarm runs the same non-interactive command as before (`claude -p ...` or
`codex exec ...`). The tab shows provider stdout while it runs.

- ccswarm writes a POSIX script into a private (`0700`) temporary directory
  and submits only `/bin/sh '<script>'` to the tab's shell. The pane shell is
  the user's interactive shell (for example Fish), which can expand
  abbreviations and reject POSIX syntax; an absolute `/bin/sh` path and a
  single quoted argument avoid both. A leading space keeps the line out of
  shell history where the shell supports that convention.
- The script records stdout, stderr, and the exit code in that directory.
  The exit-code file is renamed into place only after stdout is flushed, so
  output, Claude stream-json and Codex `--json` telemetry, and exit handling
  are identical to v0.10.
- The provider runs with the pane's environment plus the command's explicit
  environment changes. ccswarm does not copy its own environment into the
  script, so credentials are not written to disk.
- A provider that has not started within 30 seconds, or a tab that disappears
  while the provider runs, fails the call.

Herdr's own detector does not classify `claude -p` as working (observed:
`unknown`, then `idle`, then no agent). ccswarm therefore reports the stage
itself with `pane report-agent --source ccswarm --agent <name> --state working`,
then `idle` when the provider exits, and releases the pane before closing the
tab. `pane report-metadata` adds the stage and provider as `$stage` and
`$provider` sidebar tokens. Herdr waits, notifications, and rollups see
ccswarm stages as ordinary agents.

### Live (Claude only)

A live stage runs an interactive Claude Code session that the user can watch,
answer, or steer. This follows takt's `claude-terminal` design: the terminal
is the surface, and the session transcript is the result.

1. Start the agent with `herdr agent start <name> --kind claude --pane <tab pane> -- <args>`.
   Herdr returns only after Claude is ready for input, replacing takt's screen
   polling. Arguments are `--session-id <uuid>`,
   `--dangerously-skip-permissions`, and the stage's model, tools, agent, and
   worktree. The persona prompt is passed with `--append-system-prompt-file`,
   because Herdr types the start command into the shell, where macOS limits
   one line of input.
2. Submit the composed stage prompt with
   `herdr agent prompt --wait --until working --until blocked`. Right after
   startup, Claude Code can keep the pasted prompt in its input box but drop
   the Enter (observed with Claude Code 2.1.319). When Herdr reports a stall
   or timeout, ccswarm presses Enter once, never pasting the prompt again,
   and fails with a pointer to the tab if Claude still does not start.
3. Read `<claude config>/projects/*/<uuid>.jsonl` (`CLAUDE_CONFIG_DIR` or
   `~/.claude`). The call completes when the transcript has a
   `system`/`turn_duration` entry after the latest user message and Herdr
   reports the agent `idle` or `done`.
4. The stage output is the assistant text after the latest user message, so a
   correction typed by the user during the stage is part of the recorded
   turn. Token counts are the per-message `usage` totals (input including cache
   reads/writes, and output), deduplicated by message ID.
5. When Herdr reports `blocked`, ccswarm shows a Herdr notification and keeps
   waiting for the user. It never answers a dialog itself.

Interactive Claude Code asks security questions that `claude -p` does not:
folder trust the first time it opens a project (each Git repository is a
separate project), and approval of `CLAUDE.md` imports outside the working
directory. Herdr reports these as `agent_not_ready` during startup. ccswarm
notifies the user, waits in the stage tab until Claude is ready, and fails
the stage with an explicit message if the user declines and Claude exits.
The stage timeout still bounds the wait.

The tab is closed after the call; `claude --resume <uuid>` reopens the
conversation. Live stages run a single turn: they reject same-thread
multi-turn continuation, and provider fallbacks (rate limits, promotion to
Codex) run headless. Claude does not report cost in the transcript, so
`total_cost_usd` is absent for live stages.

Codex live execution is not supported. In the 0.9.5 probe Herdr's Codex
integration did not report the provider-assigned thread ID, so parallel Codex
sessions cannot be matched to their rollout files reliably.

## Notifications

ccswarm calls `herdr notification show` (best effort) when a run finishes or
fails, when `auto --require-approval` waits for `ccswarm approve`, and when a
live stage is blocked. Delivery follows the user's `ui.toast.delivery`
setting, which is `off` by default; ccswarm does not change Herdr settings.
With delivery `off`, Herdr 0.9.5 answers `shown: true` but draws nothing in
the TUI (observed with an attached client).

## Herdr Plugin

`herdr-plugin/` contains a Herdr plugin (`id = "ccswarm"`):

```sh
herdr plugin install nwiizo/ccswarm/herdr-plugin --ref v0.11.0
# or, from a checkout
herdr plugin link /path/to/ccswarm/herdr-plugin
```

| Action | Opens |
| --- | --- |
| `run-board` | Split pane following the current run (`ccswarm tail`) |
| `runs` | Popup with recent runs (`ccswarm run list`) |
| `doctor` | Popup with `ccswarm doctor` |

Actions call `ccswarm herdr action <id>`, which reads
`HERDR_PLUGIN_CONTEXT_JSON` and opens the plugin pane for the focused pane's
directory (`focused_pane_cwd`, then `workspace_cwd`). Plugin pane commands
start in the plugin directory, so the repository is passed as
`CCSWARM_HERDR_REPO` and the pane command changes into it. Herdr 0.9.5 opens
popups over the active pane and rejects a target for them; split panes need
`--target-pane`, which it rejects together with `--workspace`. Herdr closes a
plugin pane when its command exits, so every view waits for Enter. One popup
can be open at a time.

The plugin needs ccswarm 0.11.0 or later as `ccswarm` on the Herdr server's
`PATH`; an older binary has no `herdr` subcommand. Herdr plugins cannot add keybindings
themselves; bind actions in the user's Herdr `config.toml`:

```toml
[[keys.command]]
key = "prefix+r"
type = "plugin_action"
command = "ccswarm.run-board"
description = "ccswarm run board"
```

## Not Included

- Herdr-managed worktrees (`herdr worktree create`). ccswarm keeps its
  existing worktree handling; one workspace per worktree is a later option.
- Socket-level event subscriptions. ccswarm uses the CLI wrappers, which the
  Herdr documentation recommends for portable tools.
- Remote Herdr machines (`--machine`). Live and headless stages run against
  the session that hosts the calling pane.
- Codex live stages (see above).

## Verification

- Unit tests cover script quoting, environment handling, Herdr JSON parsing,
  agent-name validation, transcript completion and usage parsing, and flow
  validation.
- An integration test runs the headless path end to end against a fake
  `herdr` executable and asserts output, stderr, exit code, state reports, and
  tab cleanup.
- Live checks use an isolated Herdr session (`herdr --session <name>`), then
  stop and delete that session.

Checked on 2026-11-06 with Herdr 0.9.5 in an isolated session:

| Scenario | Observed |
| --- | --- |
| Outside Herdr | The first provider call fails before retrying, with the Herdr guidance message |
| Headless `quick` flow | Stage tab opened with `$stage`/`$provider` tokens and `working` state, the file was written in the requested repository, the tab closed, and the run exited 0 |
| Relative `--repo .` | Herdr receives an absolute tab directory; a relative one resolved against the Herdr server's directory before this fix |
| Live review, startup trust declined | The stage waited while `blocked`, then failed with the declined-prompt message and closed its tab |
| Live review, startup prompt answered | The prompt was submitted after the answer; the transcript produced the reply ending in `VERDICT: APPROVE` and real token counts, and the tab closed |
| Plugin link | Herdr accepted the manifest with three actions and three panes |
| `run-board` action | Opened a split pane for the focused pane's repository and showed the latest run's events until Enter |
| `runs` / `doctor` actions | Opened popups; the same entrypoints in a split showed the run list and a doctor report that detected the Herdr pane |

With a Herdr TUI client attached (run inside a pane of a second isolated
session and read through that pane):

| Scenario | Observed |
| --- | --- |
| Headless stage | Tab bar showed `ccswarm coder (claude)`, the space showed activity, and the agents list showed `ccswarm-coder` until the tab closed |
| Live stage | The stage tab showed Claude Code with the composed prompt; three consecutive runs completed without input after the submission fix |
| Plugin `runs` / `run-board` | The popup rendered the run list and closed on Enter; the run board opened as a split beside the working pane |

The first attached live run left the prompt unsent in Claude's input box,
which led to the submission rule in step 2.

The plugin check initially exposed both pane-targeting rules above, and a
run board that closed immediately after a finished run; all three were fixed
before the passing runs. The plugin was unlinked afterwards, restoring
Herdr's plugin registry byte for byte.
