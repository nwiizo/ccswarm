---
name: deploy-workflow
description: Release deployment process for ccswarm. Version update, quality gates, build, tag, publish.
user-invocable: true
argument-hint: "[version]"
---

Deploy a new ccswarm release using `.github/workflows/release.yml`.
The workspace has one published crate, `crates/ccswarm`. Follow the scope in
`docs/ROADMAP.md` and the version-specific notes in `docs/releases/`.

## Pre-Deployment

```bash
# Update crates/ccswarm/Cargo.toml and its Cargo.lock package version.
# Update CHANGELOG.md and docs/releases/v<VERSION>.md.

# Quality gates
cargo fmt --all
cargo clippy --workspace --locked -- -D warnings
cargo test --workspace --locked
cargo audit
cargo doc --no-deps --workspace --locked
cargo build --release --workspace --locked
cargo publish -p ccswarm --dry-run --locked
actionlint .github/workflows/release.yml
```

## Build & Release

When release publication is authorized, verify the branch, remote, diff, and
unused tag. Commit and push the reviewed release preparation. Run the Release
workflow manually on that exact commit and verify all four native archive
builds, smoke tests, and quality checks. Manual runs do not publish.

Then create and push the annotated version tag on the verified commit. The
tag workflow rebuilds and validates, creates a complete draft, verifies its
downloaded assets, attests the archives, publishes the stable crate, and
publishes the GitHub release. Prerelease tags do not publish a crate.

Do not run a second manual publish command alongside the workflow. Verify the
completed workflow, all four archives and checksums, provenance, registry
version, and an installed binary before reporting success.

## Failed Publication And Recovery

Inspect the failed job and current GitHub/crates.io state before retrying.
The two services cannot publish atomically. A workflow retry verifies an
existing crate's checksum against the candidate before skipping its upload.
Investigate a mismatch rather than bypassing that check. Do not replace an already published
tag or delete a published release as routine recovery. Use a follow-up version
for code fixes; yanking a broken crate requires explicit authorization.
