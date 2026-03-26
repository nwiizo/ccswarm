---
name: deploy-workflow
description: Release deployment process for ccswarm. Version update, quality gates, build, tag, publish.
user-invocable: true
argument-hint: "[version]"
---

Deploy a new ccswarm release.

## Pre-Deployment

```bash
# Update version in Cargo.toml files (root, crates/ccswarm, crates/ai-session)
# Update CHANGELOG.md

# Quality gates
cargo fmt --all && cargo clippy --workspace -- -D warnings && cargo test --workspace
cargo audit
cargo doc --no-deps --workspace
```

## Build & Release

```bash
cargo build --release --workspace

# Tag and push
git tag -a v$ARGUMENTS -m "Release v$ARGUMENTS"
git push origin v$ARGUMENTS

# GitHub release
gh release create v$ARGUMENTS --title "ccswarm v$ARGUMENTS" --notes-file CHANGELOG.md

# Crates.io (publish ai-session first, then ccswarm)
cargo publish -p ai-session
cargo publish -p ccswarm
```

## Rollback

```bash
git tag -d v$ARGUMENTS && git push origin :refs/tags/v$ARGUMENTS
gh release delete v$ARGUMENTS
cargo yank --version $ARGUMENTS ccswarm
```
