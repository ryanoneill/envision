# PR 6a — Dependency floors Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Raise every declared dependency floor in `Cargo.toml`, normal and dev, to the lowest version that builds, passes the suite, and audits clean under `-Zdirect-minimal-versions`.

**Architecture:** An iterative, evidence-driven procedure on a pinned nightly. Each raise is recorded with its reason: a build failure, a resolution conflict, or an advisory ID. The committed `Cargo.lock` changes only as far as the raised floors force.

**Tech Stack:** a pinned nightly (`-Zdirect-minimal-versions`), cargo-audit (`--locked`, pinned), cargo-nextest.

**Spec:** `docs/superpowers/specs/2026-10-12-v018-programme-completion-design.md` § PR 6a.

## Global Constraints

- **Toolchain:** pin one nightly by date, e.g. `nightly-2026-10-01`; use whatever recent dated nightly is installed or installable. The same date is reused by PR 6's CI job; record it in the PR body.
- **Floor raises:** each one is the **lowest** version satisfying all three acceptance commands, not "latest".
- **Advisory floors:** when a floor is set to clear an advisory, verify that version against the advisory's patched range with `cargo audit` on the minimised lockfile.
- **The final committed `Cargo.lock`** is produced by a normal `cargo update -p <raised crates>`, or is left unchanged if the raises are already satisfied. It is never the minimised lockfile.
- **Commits** are signed, with the `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` trailer. Branch `v018-dependency-floors`.

## Review Focus

1. **No floor is raised above the minimum**, which would gratuitously force consumers to update. Each raise's justification line names the failing version and the passing version.
2. **Optional dependencies** (`serde`, `serde_json`, `tracing`, `arboard`, `pulldown-cmark`, `regex`) are minimised and tested with `--all-features`, so their floors are exercised.
3. **Transitive floors** (for example `lru` via ratatui-core) are **not** in scope. They are covered by the release-time full `-Zminimal-versions` audit, so don't add direct dependencies to force them.

---

### Task 1: Minimise, iterate, record

- [ ] **Step 1: Scratch worktree.** `git worktree add <scratchpad>/minv origin/main`, where the scratchpad is `/tmp/claude-1000/-home-ryano-workspace-ryanoneill-envision/8fad2841-9a35-46ff-96fb-e7c00d6c3523/scratchpad`. Work there; the branch's real changes are only `Cargo.toml`, `Cargo.lock` and docs.

- [ ] **Step 2: Minimise.**

```bash
cd <scratchpad>/minv
rustup toolchain install nightly-YYYY-MM-DD --profile minimal
cargo +nightly-YYYY-MM-DD update -Zdirect-minimal-versions 2>&1 | tail -20
```

If resolution fails (round 1 saw `regex 1.0.0` conflict with criterion's `^1.5.1`), the failing requirement names the floor to raise. Raise it in `Cargo.toml` and repeat.

- [ ] **Step 3: Build, test and audit loop.** After each raise, run:

```bash
cargo +nightly-YYYY-MM-DD update -Zdirect-minimal-versions
cargo build --all-features --all-targets
cargo nextest run --all-features --profile ci
cargo audit --deny warnings
```

Build and test use the stable toolchain on the minimised lock. For each failure, identify the declared dependency responsible (`cargo tree -i <crate> -e normal,dev,build`), raise its floor to the lowest fixing version, record a line `crate: old → new — reason`, and repeat until all three pass.

Round-1 reviewers expect at least these:
- `regex` ≥ 1.5.5 (RUSTSEC-2022-0013, and the criterion conflict)
- `serde` ≥ 1.0.228
- `serde_json` ≥ 1.0.100
- `tokio` ≥ 1.44.2 (RUSTSEC-2021-0124, -2021-0072, -2023-0005, -2025-0023)
- `async-stream` ≥ 0.3.6
- `tracing` ≥ 0.1.41
- `trybuild` ≈ 1.0.86
- `arboard`: check it (the unmaintained `adler` comes in via minimal 3.0.0)
- `tokio-stream` / `tokio-util` / `futures-util`: check them

Confirm each one by the loop; don't copy these blindly.

- [ ] **Step 4: Apply on the branch.** In the main worktree, run `git checkout -b v018-dependency-floors origin/main`. Apply exactly the recorded `Cargo.toml` floor changes, then `cargo update -p <each raised crate>` only if the current lock no longer satisfies a floor (`cargo metadata --locked` errors if so). Then:

```bash
cargo nextest run --all-features --profile ci
cargo nextest run --no-default-features --profile ci
cargo +1.88 nextest run --no-default-features --features serialization,input-components,data-components,display-components,navigation-components,overlay-components,compound-components --profile ci
cargo clippy --all-targets --all-features -- -D warnings
cargo audit --deny warnings
cargo fmt --all -- --check
```

- [ ] **Step 5: Commit** with signature: "Raise dependency floors to minimal versions that build, test and audit clean". The message body holds the full `crate: old → new — reason` list.

- [ ] **Step 6: CHANGELOG.** Under `[Unreleased]` → `### Changed`, add a "Minimum dependency versions raised" bullet listing each raise with its reason (advisory IDs where applicable). Note it is consumer-visible and needs a `cargo update` if a lockfile pins below the new floors. Make a signed commit.

- [ ] **Step 7:** Remove the scratch worktree. Push, open a PR (body: the raise table, the nightly date, and the final loop output; `🤖 Generated with [Claude Code](https://claude.com/claude-code)`). Get an independent review, then squash-merge once green.
