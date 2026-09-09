# Astra Capabilities In The Sangha Design

Status: researched design; no new runtime integration is implemented here.
Sources and local adapter reviewed: 2026-09-09.

Scope assumption: Astra means OpenAI's **GPT-6 Astra**, consistent with the
Codex investigation and the locally available model catalog. Other products
named Astra are outside this document.

## Product Decision

Use Astra's capabilities to make the
[Sangha process](SANGHA_PRODUCT_CORE.md) responsive during long work: accept
new guidance, continue independent work while results are pending, apply more
reasoning where a decision is difficult, and preserve useful context.
Acceptance still depends on checks, reports, and resolved objections.

The model is an execution option. Its provider's product claims do not
establish that a ccswarm run is correct, cheaper, isolated, or recoverable.
Keep Claude and Codex usable under the same workflow decision policy.

## Verified Capabilities And Their Use

The middle column summarizes official documentation. The final column is a
ccswarm design recommendation, not an existing integration.

| Capability | Officially documented behavior | What to incorporate into ccswarm |
| --- | --- | --- |
| Sustained work and focused clarification | Astra handles multistep work and can incorporate guidance while retaining the broader task. [Model guidance](https://developers.openai.com/api/docs/guides/latest-model) | Preserve the user's objective and settled constraints across revisions; ask only for input that changes the result |
| Async tool calling | Application-run function/custom tools can return later using their original call IDs; the application still owns execution. [Async tools](https://developers.openai.com/api/docs/guides/async-tool-calling) | Track pending evidence checks, continue independent work, and wait only where a decision needs the missing result |
| Mid-turn steering | Astra supports steering over Responses WebSockets; queued input is distinct from applied input and does not undo actions or cancel started tools. [Steering](https://developers.openai.com/api/docs/guides/steering) | Accept a correction during a run, show when it takes effect, and invalidate affected review evidence |
| Adjustable reasoning | Codex offers model/effort selection; Astra's API also supports effort updates between responses in standard single-agent mode. [Codex models](https://learn.chatgpt.com/docs/models), [Reasoning updates](https://developers.openai.com/api/docs/guides/reasoning#change-reasoning-mid-conversation) | Use appropriate effort for each stage; increase it for unresolved tradeoffs without raising every worker's effort |
| Native delegation | Codex supports subagents on explicit request or applicable instructions; Ultra uses subagents for divisible complex work. [Subagents](https://learn.chatgpt.com/docs/agent-configuration/subagents#availability), [Models](https://learn.chatgpt.com/docs/models) | Keep optional worker delegation inside a stage, with assigned scope, a shared run budget, and individually attributable results |
| Context continuity | Supported Codex clients have an opt-in Astra context-management experiment with task notes and search over earlier task context; availability depends on sign-in and plan. [Models](https://learn.chatgpt.com/docs/models#experimental-context-management) | Preserve a compact task brief and evidence references in ccswarm checkpoints; provider notes supplement that record |

## Behavior To Build

### Continue Useful Work While Waiting

Track each pending job with its original call identity, decision revision,
dependencies, execution state, and result location. Deliver completion through
events rather than repeatedly asking a model whether a tool has finished.
Report a missing result as pending or failed, never as a guessed check pass.

For example, a long test run can overlap with independent documentation
inspection against the same frozen revision. Final acceptance waits for the
test result. Editing files under that test changes its subject and requires
fresh affected verification.

This applies the async pattern to Sangha; it does not require adopting an API
transport before the basic decision process works. With a future Responses
adapter, match each tool result to its original `call_id`. Async tools are not
hosted background jobs, and the documented API excludes programmatic-tool
execution and the combination of async tools with parallel tool calls in
multi-agent mode. [Async tool compatibility](https://developers.openai.com/api/docs/guides/async-tool-calling#compatibility).

### Accept Corrections Without Losing The Task

Track incoming guidance as received, pending application, applied, or failed.
Show the affected stage and preserve the original objective unless the user
changes it. A scope or acceptance-criteria change creates a new decision
revision; old votes cannot approve the revised request.

For example, “keep the public API compatible” during implementation should
update the task constraints, trigger assessment of affected work, and retain
unaffected evidence. Already executed actions remain visible. Cancellation
and cleanup are separate operations.

On disconnect, reconcile a steering request against recorded events before
resending it. The API's pending steering is connection-local; receiving its
acknowledgment is not proof that the update was applied. A subprocess backend
without live steering can apply recorded guidance at the next safe stage
boundary, with that limitation visible to the user.
[Steering lifecycle](https://developers.openai.com/api/docs/guides/steering).

### Spend Reasoning And Delegation On The Hard Part

Start with the configured default effort. Increase it for ambiguous plans,
conflicting evidence, or difficult defect analysis. Use a bounded, cheaper
execution option for well-specified work only when it meets the task's needs.
Record the effective choice and compare accepted outcomes, latency, and usage.
Do not hardcode an Astra coordinator and a fixed fleet of workers for every run.

A future native-delegation adapter must respect explicit user or applicable
workflow instructions, cap total descendants, and keep worker ownership and
results visible. Internal helpers do not become additional Sangha voters.
One member with several helpers still contributes one member report. Native
delegation is an execution aid, not proof of independent review.

The API's `configuration_update` changes effort only in standard single-agent
mode. It is incompatible with automatic compaction/truncation and is not a
generic setting for nested multi-agent runs. Keep the effective setting in
ccswarm's records rather than assuming response-level metadata reflects an
in-conversation update.
[Reasoning compatibility](https://developers.openai.com/api/docs/guides/reasoning#change-reasoning-mid-conversation).

### Keep Prompts And Verification Proportionate

Specify the task, constraints, evidence, expected report, and stopping
conditions. Carry forward settled user choices. Make instruction precedence
explicit and explain which instruction causes a necessary pause. Complete
authorized preparatory work before requesting a decision that depends on it.

Avoid mandatory reviewer fleets and repeated broad test runs for minor
changes. Run the required checks and relevant behavioral verification; extend
testing when a change or unresolved concern justifies it. These choices adapt
OpenAI's guidance about Astra's sensitivity to instructions, clarification,
delegation, and thorough testing to this repository.
[Astra prompting guidance](https://developers.openai.com/api/docs/guides/latest-model#prompting-best-practices).

## What The Current CLI Can Do

The existing Codex adapter already forwards a stage model as `codex exec
--model ...`. The pipeline command accepts `--model-override`. A preview needs no
new provider implementation:

```bash
ccswarm pipeline --provider codex --model-override gpt-6-astra \
  --task "Review the proposed change" --dry-run
```

Removing `--dry-run` starts the selected flow and may edit the workspace;
the wording of the task does not make the default flow read-only. Actual model
access depends on the user's Codex account and client.

The local review found Codex CLI `0.153.4` and `gpt-6-astra` in its model
catalog. Catalog presence and command validation are not a live inference
test. No paid model execution is needed for this design review.

The [current adapter](../crates/ccswarm/src/providers/codex.rs) starts a
subprocess and optionally resumes a provider thread. It does not expose
Responses async tools, WebSocket steering, in-conversation reasoning updates,
or ccswarm-controlled native subagents. Selecting Astra alone does not add
those integration features. Its current writable review sandbox is also a
separate prerequisite to fix.

## Delivery Priorities

| Order | Deliverable | Acceptance evidence |
| --- | --- | --- |
| First | Valid Sangha decisions, enforceable review permissions, bounded checks and reports | The core failure cases in the [roadmap](ROADMAP.md) cannot produce acceptance |
| Next | Durable guidance, pending-result identities, context references, and effective execution settings | Delayed or duplicated results and changed requirements cannot authorize the wrong revision |
| Then | One measured experiment with live steering, async tools, or native delegation through a supported backend | Less waiting or repeated work without lost guidance, weaker permissions, or inflated votes |

Do not make a new Responses adapter, a native-delegation integration, or
experimental provider context management a v0.10.0 release prerequisite.
Computer/browser execution remains a later option for tasks with observable
UI acceptance criteria; it is not required for every coding review.

Measure the same coding tasks with and without each adopted capability.
Include corrections during execution, delayed checks, provider disconnects,
and conflicting reviewer evidence. Prefer a capability when it improves a
verified outcome or reduces intervention and repeated work within the user's
budget, not merely because the model supports it.
