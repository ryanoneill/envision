# PR 6 — `cargo deny` gate Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `deny.toml` and two CI jobs:
- `cargo-deny` (advisories, bans and sources on PRs and pushes to `main`; advisories only on a daily schedule);
- `minimal-versions` (`-Zdirect-minimal-versions` build + test + audit on a pinned nightly).

**Architecture:** The policy lives in `deny.toml`, with exceptions documented inline. The tools are installed `--locked` at pinned versions, with no third-party action. Neither job is gated by `changes`.

**Tech Stack:** cargo-deny 0.20.2, cargo-audit (pinned), a pinned nightly, GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-10-12-v018-programme-completion-design.md` § PR 6. Depends on PR 4 and PR 6a having merged.

## Global Constraints

- **The `deny.toml` content is exactly the spec's block**, with the `unsound` comment on the `unsound` line:
  - `[graph] all-features = true`;
  - `[advisories]`: `version = 2`, `unmaintained = "all"`, `unsound = "all"`, `yanked = "deny"`, `unused-ignored-advisory = "deny"`, `ignore = []`;
  - `[bans]`: `multiple-versions = "deny"`, `wildcards = "deny"`, plus `skip`;
  - `[sources]`: `unknown-registry = "deny"`, `unknown-git = "deny"`.
- **Skip entries:** only the older version, written as `"name@version"` objects with a `reason` field. Each reason contains a **tracking link** and a **re-check trigger**.
- **CI invocations:**
  - PRs and pushes to `main`: `cargo deny --locked check -D warnings advisories bans sources`.
  - Schedule: `cargo deny --locked check -D warnings advisories`.
- **Installs:** `cargo install cargo-deny --locked --version 0.20.2`. cargo-audit is likewise installed `--locked --version <pinned>`.
- **Minimal-versions job:**
  - nightly pinned to the date used in PR 6a;
  - cache key = `hashFiles('Cargo.toml')` + the nightly date.
- **Neither job uses the `changes` filter.**
- **Commits** are signed, with the trailer. Branch `v018-cargo-deny`.

## Review Focus

1. **A stale skip entry fails the job.** `-D warnings` turns "unmatched skip" into an error. Task 1 Step 3 demonstrates it by temporarily adding a bogus skip, confirming failure, then removing it.
2. **The scheduled run cannot fail on duplicates.** Advisories only (Task 2).
3. **PR builds from forks** don't need secrets; neither job uses any.

---

### Task 1: `deny.toml`

**Files:** Create `deny.toml`.

- [ ] **Step 1: Write the config.**

```toml
# Supply-chain policy (spec 2026-10-12 § PR 6). Run:
#   cargo deny --locked check -D warnings advisories bans sources

[graph]
all-features = true          # optional backends evaluated as compiled; no `targets` → all platforms

[advisories]
version = 2
unmaintained = "all"         # explicit (already the default)
unsound = "all"              # default is direct-deps-only scope; "all" covers transitive (e.g. lru)
yanked = "deny"
unused-ignored-advisory = "deny"
# Exceptions only when no fixed version exists: advisory ID, linked tracking
# issue, reason and re-check date; reviewed like code (spec § PR 6 policy).
ignore = []

[bans]
multiple-versions = "deny"   # dev-only duplicates are out of scope (multiple-versions-include-dev = false)
wildcards = "deny"
skip = [
  # filled in Step 2
]

[sources]
unknown-registry = "deny"
unknown-git = "deny"
```

- [ ] **Step 2: Derive the skips.** Run `cargo deny --locked check -D warnings bans 2>&1 | grep -E "^error|duplicate"`. For each duplicate, run `cargo tree -i <crate>@<ver> -e normal,build --all-features` to find both chains. Then:
  1. Search the upstream tracker (`gh search issues --repo <owner>/<repo> "<crate> <ver>"`, or `gh api`) for an issue or PR tracking the old version.
  2. If one exists, link it, with the re-check trigger "the upstream release that merges <link>".
  3. If none exists **and** the split is ecosystem-wide (`syn` 2/3, `windows-sys`), link a tracker search URL or the ecosystem tracking issue, with the trigger "re-check at each envision release".
  4. If none exists for a crate-specific split, use a tracker-search link with the trigger "re-check at each envision release". **No upstream issues are filed or drafted.** This is the maintainer's decision (2026-10-12), and it satisfies the spec's skip policy.

Expected entries:
- `hashbrown@0.16`: via `kasuari` (ratatui-core) vs `lru`.
- `miniz_oxide@0.8`: `arboard` → `image` → `png` vs `flate2`.
- `syn@2`: proc-macros. Note that build dependencies **are** in scope.
- `windows-sys@0.60`: `arboard` 3.6.1 vs `rustix`/`errno`.

Write each entry as:

```toml
  { crate = "hashbrown@0.16.1", reason = "ratatui-core: kasuari pins 0.16, lru uses 0.17. Tracking: <link>. Re-check: <trigger>." },
```

Use the exact versions from the check output.

- [ ] **Step 3: Verify.** `cargo deny --locked check -D warnings advisories bans sources` must report `advisories ok, bans ok, sources ok`. Demonstrate Review Focus 1: add `{ crate = "bogus@0.0.1", reason = "test" }`, confirm the run fails, then remove it.

- [ ] **Step 4: Commit** with signature: "deny.toml: supply-chain policy (advisories, bans, sources)".

---

### Task 2: CI jobs

**Files:** Modify `.github/workflows/ci.yml`.

- [ ] **Step 1: Add the schedule trigger** to the workflow's `on:` block:

```yaml
  schedule:
    - cron: "17 6 * * *"   # daily advisories check (deny job only)
```

Then gate every **existing** job so the schedule doesn't run the full matrix: add `if: github.event_name != 'schedule'` to each existing job, combined with any existing `if:` using `&&`. Do this in this PR, with a comment.

- [ ] **Step 2: `cargo-deny` job:**

```yaml
  cargo-deny:
    name: Cargo Deny
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: stable
      - name: Install cargo-deny (pinned, --locked)
        run: cargo install cargo-deny --locked --version 0.20.2
      - name: Check advisories, bans and sources
        if: github.event_name != 'schedule'
        run: cargo deny --locked check -D warnings advisories bans sources
      - name: Check advisories (scheduled)
        if: github.event_name == 'schedule'
        run: cargo deny --locked check -D warnings advisories
```

- [ ] **Step 3: `minimal-versions` job:**

```yaml
  minimal-versions:
    name: Minimal Versions
    if: github.event_name != 'schedule'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust (stable + pinned nightly)
        uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: stable
      - run: rustup toolchain install nightly-YYYY-MM-DD --profile minimal
      - name: Cache cargo registry
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-minver-nightly-YYYY-MM-DD-${{ hashFiles('Cargo.toml') }}
      - name: Install cargo-nextest
        uses: taiki-e/install-action@v2
        with:
          tool: cargo-nextest
      - name: Install cargo-audit (pinned, --locked)
        run: cargo install cargo-audit --locked --version <pinned>
      - name: Minimise direct dependencies
        run: cargo +nightly-YYYY-MM-DD update -Zdirect-minimal-versions
      - run: cargo build --all-features --all-targets
      - run: cargo nextest run --all-features --profile ci
      - run: cargo audit --deny warnings
```

Use the nightly date from PR 6a, and pin cargo-audit to the version installed locally (`cargo audit --version`). Match the repo's existing nextest install step if it differs.

- [ ] **Step 4: Validate.** YAML parses, `actionlint` passes if available, and no new job references `needs: changes` or `steps.changes`.
- [ ] **Step 5: Commit** with signature: "CI: cargo-deny gate (daily advisories) and direct-minimal-versions job".

---

### Task 3: PR, branch protection, merge

- [ ] **Step 1:** Add CHANGELOG `[Unreleased]` → `### Changed` lines: the supply-chain policy, its CI gates, and the advisory response policy. Make a signed commit.
- [ ] **Step 2:** Merge `origin/main`, push, and open a PR. The body includes:
  - the skip table with links and triggers;
  - the stale-skip demonstration;
  - CI run links;
  - `🤖 Generated with [Claude Code](https://claude.com/claude-code)`.
- [ ] **Step 3:** Get an independent review, and wait until all checks are green, including both new jobs.
- [ ] **Step 4: Branch protection.** This is a settings change, so **confirm with the maintainer** before adding `Cargo Deny` and `Minimal Versions` to the required checks (app_id 15368). Then PATCH the checks and squash-merge.
