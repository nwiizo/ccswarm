# ccswarm Release Scope And Roadmap

Scope updated: 2026-09-09.

## v0.10.1 Release Scope

v0.10.1 turns the browser example into a reproducible real-generation check.
`examples/e2e-playwright/run.sh --live` generates a new Order Desk app through
ccswarm and tests that output in Chromium. The default run tests the included
app and clearly identifies its provider-free preview. Run directories and
evidence are retained. Browser checks also gate CI and release publication.
Live generation also exposed and fixed false failure classification when
provider prose mentions error handling or storage failures. Explicit diagnostic
markers still identify error logs.
Standalone Node specs now use the built-in runner instead of Playwright;
unavailable loose-spec runners are reported as failed verification.

See the [example guide](../examples/e2e-playwright/README.md) and
[release notes](releases/v0.10.1.md). Existing Sangha and Astra limitations
remain; runtime changes cover response parsing and standalone test execution.

## v0.10.0 Release Scope

v0.10.0 delivers **Sangha vote validation and complete release distribution**.
The [release notes](releases/v0.10.0.md) describe the shipped behavior and
compatibility. Publication is tracked by the
[GitHub release](https://github.com/nwiizo/ccswarm/releases/tag/v0.10.0).

| Slice | Behavior |
| --- | --- |
| Valid approval votes | Only successfully completed Sangha members can approve; failed, partial, and unknown results abstain |
| Reachable quorum | Flow validation rejects a quorum above the effective member count, including default membership |
| Distinct member results | Generated member IDs cannot collide and overwrite another review |
| Complete distribution | Native Linux/macOS AMD64/ARM64 builds and archive smoke tests must pass before publication |
| Verified publication | Quality checks, package validation, checksums, downloaded draft assets, and provenance precede stable publication |
| Accurate documentation | Sangha and Astra design goals are separated from implemented behavior |

The default two-of-three quorum policy remains. This release does not claim
the full evidence-based Sangha protocol is implemented. A version increase
does not turn future milestones into release guarantees.

## Release Verification

The [Release workflow](../.github/workflows/release.yml) is the publication
path. A manual run validates the complete distribution without publishing;
a version tag runs the same checks and then publishes.

Required checks:

- Matching package, lockfile, tag, changelog, and release-note versions.
- Formatting, strict Clippy, workspace tests, dependency audit, and rustdoc.
- Package and publish dry-run validation.
- Browser verification of the included Order Desk app.
- Four native release builds with checksum verification and archive extraction.
- Packaged `--version`, `--help`, default-flow validation, and a Codex/Astra
  prompt preview on every target.
- Complete artifacts before draft creation; downloaded draft assets must match
  the verified builds before stable publication.

Semantic prereleases are marked as prereleases and do not publish a crate.
Publication failures leave the GitHub draft unpublished. A retry skips an
already published crate only when its registry checksum matches the candidate
package; mismatches stop publication. GitHub and crates.io do not provide an
atomic shared transaction.

## Known Runtime Limitations

- Enough valid approvals can still accept a decision despite a revision vote.
- Sangha dispatch bypasses ordinary command gates.
- The Codex adapter currently uses a writable workspace even when a review
  requests read-only permissions.
- Events and session persistence do not provide workflow checkpoint resume.
- `lab sangha` proposal storage is separate from workflow acceptance.
- Selecting Astra through Codex does not expose Responses steering, async
  tools, or ccswarm-controlled native delegation.

## Future Sangha Milestones

These are design goals, not v0.10.0 release requirements. The
[Sangha Product Core](SANGHA_PRODUCT_CORE.md) defines their user job: reach a
verified, reviewable change, understand concerns, and intervene where human
judgment is needed without losing valid accepted work.

### S1: Enforce Review And Evidence Boundaries

- Enforce read-only review or use an environment that protects its subject;
  reject unsupported required capabilities and prevent weaker fallback.
- Identify the reviewed task, criteria, content revision, policy, member
  reports, and machine-check results.
- Require applicable checks and successful designated reviews before acceptance.
- Add a final-change Sangha checkpoint after verification.

Acceptance: a reviewer cannot alter the subject; a failed or missing required
check cannot be overridden by unanimous approval; stale reports cannot approve
a changed revision.

### S2: Resolve Objections Within A Bounded Process

- Record blocking and advisory concerns with evidence and criterion references.
- Resolve blocking concerns through fixes or supported rebuttals and reassess.
- Separate accepted, needs-revision, and needs-input decisions from execution
  failure, timeout, or cancellation.
- Bound review/repair rounds, elapsed time, and measurable usage. Internal
  helpers do not create additional Sangha votes.

Acceptance: two approvals cannot dismiss a demonstrated blocking defect; the
report preserves its resolution; exhausted budgets stop with an explicit
outcome rather than lowering the decision threshold.

### S3: Supervise And Resume Decisions

- Record child state, terminal reasons, activity, pending work, and cleanup.
- Persist atomic checkpoints for reports, checks, decisions, and next actions.
- Validate repository, content, criteria, flow, and policy on resume; reuse
  only evidence whose inputs remain valid.
- Preserve user corrections and show when they apply. Backends without live
  steering apply guidance at a visible safe stage boundary.
- Present one decision report with remaining concerns, evidence, next action,
  recovery options, elapsed time, and available usage.

Acceptance: interruption does not leave ambiguous start-only records; resume
does not silently replay everything or reuse stale evidence; canceled local
children are reaped.

### S4: Evaluate Astra-Informed Execution

The [Astra plan](ASTRA_INTEGRATION.md) covers pending-result delivery, live
guidance, proportionate reasoning, optional bounded delegation, and context
references. Experiment with one capability at a time through a supported
backend after the decision and recovery boundaries are reliable.

Compare matched coding tasks against a single reviewer and simple quorum.
Measure human-confirmed defects, false blocking concerns, interventions,
latency, available usage, and repeated work after interruption. Agent count,
agreement, and model branding do not establish quality.

## Deferred Expansion

Full A2A lifecycle expansion, new providers, general task graphs, mailboxes,
Desk/UI surfaces, OTLP export, self-extension, evolution, and long-term memory
remain supporting options. They are not prerequisites for the Sangha core.
Keep orchestration in `workflow/`, execution and persistence primitives in
`session/`, and action authorization distinct from review acceptance.

The [Product Abstraction Plan](CCSWARM_PRODUCT_ABSTRACTION_PLAN.md) and
[Multi-Agent Redesign](MULTI_AGENT_REDESIGN.md) retain broader research. Their
earlier milestone lists do not add requirements to v0.10.0.
