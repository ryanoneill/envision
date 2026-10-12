# R — v0.18.0 release preparation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the crate publishable as v0.18.0:
- docs.rs shows every feature's API;
- the package excludes internal material;
- version, README and CHANGELOG are finalised.

**Architecture:** `Cargo.toml` metadata and `exclude`, prose feature notes in rustdoc, a version bump, and the CHANGELOG section rename. Then the pre-release audit and `/release`, both confirmed with the maintainer.

**Tech Stack:** cargo package metadata, docs.rs.

**Spec:** `docs/superpowers/specs/2026-10-12-v018-programme-completion-design.md` § R. Depends on PRs 4, 5, 6, 6a and 7 having merged.

## Global Constraints

- `[package.metadata.docs.rs] all-features = true`. No `doc_cfg`/`docsrs` cfg is introduced.
- `exclude` extends today's `[".claude/", "CLAUDE.md", "tools/"]` with `docs/superpowers/`, `docs/audits/`, `fastbreak.toml`, `tarpaulin.toml`, `.superpowers/`. The README (used by an `include_str!` doctest) stays included.
- `version = "0.18.0"`. The README install snippet goes `0.17` → `0.18`. CHANGELOG `[Unreleased]` becomes `[0.18.0] - <date>`, with a fresh empty `[Unreleased]` above it.
- Commits are signed, with the trailer. Branch `v018-release-prep`.

## Review Focus

1. **`cargo package` succeeds with the exclusions:** no `include_str!` or `#[doc = include_str!(...)]` path is excluded. Task 1 Step 3.
2. **Every feature-gated public item states its feature in prose** (`scrolling-regions`, `test-utils`, `tracing`, `clipboard`, `markdown`, the component groups), because docs.rs will show them all. Task 1 Step 2 greps `#[cfg(feature` on `pub` items and checks each has a prose note.

---

### Task 1: Package metadata, docs, version

- [ ] **Step 1: `Cargo.toml`.**

```toml
exclude = [".claude/", "CLAUDE.md", "tools/", "docs/superpowers/", "docs/audits/", "fastbreak.toml", "tarpaulin.toml", ".superpowers/"]

[package.metadata.docs.rs]
all-features = true
```

Set `version = "0.18.0"`.

- [ ] **Step 2: Prose feature notes.** List feature-gated public items with `grep -rn -B2 "^\s*pub " src | grep -B2 "cfg(feature"`, and with module-level `#[cfg(feature = …)] pub mod`. For each item or module whose rustdoc lacks a "Requires the `<feature>` feature." line, add one. Modules may carry the note once at module level.
- [ ] **Step 3: Verify the package.**

```bash
cargo package --list --allow-dirty | grep -E "^docs/superpowers|^docs/audits|tarpaulin|fastbreak|\.superpowers"   # Expected: none
cargo package --allow-dirty 2>&1 | tail -3      # Expected: success (verifies build from the packaged tarball)
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
cargo test --doc --all-features
```

- [ ] **Step 4: README and CHANGELOG.** Change the README install snippet `envision = "0.17"` → `"0.18"`, and any other `0.17` install references (`grep -n '0\.17' README.md`). In the CHANGELOG, rename `## [Unreleased]` to `## [0.18.0] - YYYY-MM-DD` and insert a fresh `## [Unreleased]` above it. Add the compare link if the file uses link references.
- [ ] **Step 5: Commit** (signed): "Release prep: docs.rs metadata, package exclude, v0.18.0".

### Task 2: PR, audit, release (maintainer-confirmed)

- [ ] **Step 1:** Merge `origin/main`, push, and open a PR. Get an independent review, then squash-merge once green.
- [ ] **Step 2: Pre-release audit.** Ask the maintainer before running the audit skill (`/audit`). It must include the full `-Zminimal-versions` + `cargo audit` step: run `cargo +nightly-<date> update -Zminimal-versions` in a scratch worktree, then `cargo audit`, and record the findings with upstream causes in `docs/audits/<date>-pre-v0.18.0-release.md`. Compare against the transitive-floor exposure documented in #524 (`lru` 0.18.0/0.18.1 window, or none if ratatui#2825 has shipped).
- [ ] **Step 3: Release.** Run `/release`, which publishes to crates.io and creates the GitHub release. It goes ahead **only on the maintainer's explicit go-ahead**.
