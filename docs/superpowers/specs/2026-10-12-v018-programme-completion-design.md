# v0.18.0 programme completion — design

## Purpose

Finish the security programme from `2026-10-09-ratatui-0.30-upgrade-design.md` (§ Programme). Already merged:
- #521 lockfile refresh;
- #522 MSRV 1.88;
- #524 ratatui 0.30.2;
- the supporting fixes #520 (flaky tests) and #523 (Coverage).

This document designs everything left before v0.18.0 ships. Each item is its own branch and PR.

The maintainer's goal is unchanged: **the library should be as free of security issues as possible.**

**Review history.**
- Round 1: five personas (security, PR 5 implementer, CI/QA + dependencies, downstream API/semver, adversarial) plus the peer session "claudio". All six blocked, on fixable items.
- This revision folds in their findings and the maintainer's re-decisions. Reviewer attributions appear where a finding changed the design.

### Maintainer decisions

| Decision | Choice |
|---|---|
| `compact_str` | Bump to 0.9 (ratatui-core's major). Keep it public, re-exported. |
| `scrolling-regions` | **Opt-in envision feature** forwarding to ratatui's; scroll impls behind it. (Revised 2026-10-12: the earlier "always on" choice would have moved the E0046 onto every downstream `Backend` and made `insert_before` error on legacy Windows consoles.) |
| Advisory gate | **Required check plus a daily scheduled run.** Respond by fixing forward first. Only when no fix exists, add an ignore entry with a linked tracking issue and a re-check date, reviewed like code. |
| Merges | Squash-merge PRs after an independent review and all-green CI. Anything public outside envision is confirmed first. |

### Items and order

| # | PR | Depends on |
|---|---|---|
| 7 | Dev-dependency upgrades (+ bench baseline cache fix) | — |
| 4 | `compact_str` 0.9 | — |
| 5 | Inline-viewport correctness + opt-in `scrolling-regions` | — |
| 6a | Dependency floors (from a minimal-versions run) | — |
| 6 | `cargo deny` gate + minimal-versions job + scheduled advisory run | 4, 6a |
| 8 | Track upstream ratatui#2825 (`lru >= 0.18.2`) | — (no envision code until upstream releases) |
| R | Release preparation (docs.rs metadata, package contents, CHANGELOG/version) | 4, 5, 6, 6a, 7 |

**Execution order:** 7 → 4 → 5 → 6a → 6 → R. Then the pre-release audit, then the release, which is a public action confirmed with the maintainer.

**Dependency note.** PR 6 does **not** depend on PR 7. cargo-deny's `multiple-versions-include-dev` defaults to `false`, so dev-only duplicates never reach the gate. This corrects the round-1 rationale (security, adversarial and CI/QA reviewers; claudio).

---

## PR 7 — dev-dependency upgrades

**`[dev-dependencies]`:**
- **`criterion = { version = "0.8", features = ["html_reports"] }`**, with default features kept.
  - Rationale: dropping `rayon` needs `default-features = false` plus an explicit feature list (`plotters`, `cargo_bench_support`, `html_reports`). The benches gain nothing from that, and it is a dev-only tree.
  - Recorded decision: keep defaults, so `rayon`/`crossbeam-*` stay in the bench graph. This corrects round 1's "drops rayon" claim (CI/QA, adversarial).
  - criterion 0.8 needs Rust 1.86, which is within MSRV 1.88.
- **`proptest = "1.12"`.** 1.11 still pulls `rand` 0.9; 1.12's `rust-version` is 1.88, which equals envision's MSRV (CI/QA).
- **`rand = "0.10.1"`.** 0.10.0 falls under RUSTSEC-2026-0097 (unsound) (CI/QA).

**Code fallout**, fixed at the source and never silenced:
- **`criterion::black_box` is deprecated: 87 sites in 5 benches** (`component_view.rs`, `memory.rs`, `runtime.rs`, `component_events.rs`, `capture_backend.rs`) (claudio, CI/QA). Replace them with `std::hint::black_box`.
- **rand 0.10:** `random`/`random_range` moved to `rand::RngExt`. `benches/sort_bench.rs` imports `use rand::{RngExt, SeedableRng}`. No other file uses rand.

**Bench baselines.** Today the restore steps use `actions/cache@v4` with primary key `criterion-baseline-<bench>-${{ github.base_ref || 'main' }}` (`ci.yml:311` and siblings). The save on `main` uses the same fixed key (`:320-323`). Actions caches are immutable, so baselines are frozen at the first-ever save. An exact primary-key match also beats `restore-keys`, so adding a prefix alone would change nothing (CI/QA, round 2).

Fix, per bench job:
- **Restore:** `actions/cache/restore@v4` with `key: criterion-baseline-<bench>-main-${{ github.run_id }}` and `restore-keys: criterion-baseline-<bench>-main-`. The trailing dash means the old unsuffixed key never matches.
- **Save** (pushes to `main` only): `actions/cache/save@v4` with `key: criterion-baseline-<bench>-main-${{ github.run_id }}`.

This is in scope here because PR 7 is what invalidates the old baselines.

**Acceptance:**
- `cargo bench --all-features --no-run` builds every bench, with zero warnings.
- Clippy `-D warnings` is clean on stable and on 1.88 (`--all-targets --all-features`).
- The full suite is green.
- `cargo audit --deny warnings` exits 0.
- On 1.88, the msrv feature-set `--tests` build resolves and builds; verified in round 1.
- The CI bench jobs pass. **After merge, the second `main` run's restore log names a `-<run_id>` key**, proving new baselines are actually used. This is checked post-merge and recorded in the PR.

**Not in scope:** the dev-only `itertools` 0.13/0.14 duplicate (criterion vs ratatui), which the gate ignores. Also out: the pre-existing `capture_backend` bench failing without `serialization` (Known Deferred Findings).

---

## PR 4 — `compact_str` 0.8 → 0.9

**Public exposure** (API reviewer):
- `SortKey::String(CompactString)` (`src/component/cell.rs:27`);
- **`impl From<CompactString> for Cell`** (`cell.rs:368`);
- `impl Into<CompactString>` parameters on `Cell::new`, `with_text`, `success`, `warning`, `error`, `muted` and `severity` (`cell.rs:193-351`);
- `EnhancedCell::with_symbol` and `set_symbol` (`src/backend/cell/mod.rs:126,159`).

**Change:**
- Set `compact_str = "0.9"`. The serde support (`features/serde.rs`) is identical between 0.8.2 and 0.9.1 apart from formatting (CI/QA). The code builds unchanged (adversarial).
- **`pub use compact_str;`** at the crate root. Its rustdoc states the policy: *envision tracks the `compact_str` major used by ratatui-core; a change of that major is a breaking envision release.*
  - compact_str 0.10 already exists, but envision follows ratatui-core, not the latest release (API reviewer).
  - If a ratatui-core patch release moves to 0.10, the duplicate gate goes red until envision follows in its next breaking release. In that case a reviewed skip entry (per the gate policy) bridges the gap.

**Who breaks.** Code that only passes `&str`/`String` into the `impl Into<…>` parameters is unaffected. That covers every local consumer, including leadline (API reviewer).
- Code that **names** `CompactString` breaks: type annotations, constructing a `SortKey::String`, or calling `Cell::from(my_compact_string)`.
- A plain `match` on `SortKey::String(s)` still compiles.

**Docs:**
- MIGRATION § 1: a table row for `compact_str` 0.8 → 0.9.
- A short subsection on type identity and `envision::compact_str`, covering the `From` case.
- § 7 lists `envision::compact_str` among the re-exports.
- A CHANGELOG `[Unreleased]` entry. The "commit-or-drop" Known Deferred Finding is closed as **commit**.

**Acceptance:**
- **Before the bump:** add a golden-string `EnhancedCell` JSON test (exact serialized string), because round-trip tests cannot see a format change. `SortKey` has no serde (CI/QA, API reviewer). The test passes before and after.
- `cargo tree -d -e normal` shows no `compact_str` duplicate.
- The full suite is green, clippy is clean, and audit is clean.

---

## PR 5 — inline-viewport correctness and opt-in `scrolling-regions`

### Problems on `main`

1. **`DualBackend` doesn't forward `append_lines`.** The trait default, a no-op, runs, so inline-viewport growth never reaches the real terminal.
2. **`CaptureBackend` doesn't implement `append_lines`** (the default no-op), so captures of inline viewports are wrong.
3. **E0046 under ratatui's `scrolling-regions`.** `scroll_region_up`/`down` are required `Backend` methods under that feature (`ratatui-core-0.1.2/src/backend.rs:362-367`), and envision implements neither. If ratatui's `scrolling-regions` is enabled anywhere in the graph, envision fails with E0046. #524 documents this as a known limitation.
4. **Cursor desynchronisation.** `DualBackend::get_cursor_position` returns only the primary's cursor (`src/adapter/dual/mod.rs:178`). ratatui's `compute_inline_size` reads it and then calls `append_lines` (`ratatui-core-0.1.2/src/terminal/inline.rs:396-405`), so once forwarding exists, the capture half would scroll from its own stale cursor (scroll5, adversarial).

### Design

**Always (no feature).** `append_lines` is not feature-gated in ratatui.
- **`CaptureBackend::append_lines(n)`** mirrors `TestBackend` (`ratatui-core-0.1.2/src/backend/test.rs:326-375`), minus scrollback, which `CaptureBackend` does not model; rows scrolled off the top are discarded. The cursor ends at `((x+1).min(w-1), (y+n).min(h-1))`.
  - This is TestBackend's convention, not crossterm's. Crossterm prints `\n` and keeps x in raw mode. It is mirrored deliberately so captures match ratatui's reference backend, and the rustdoc says so (scroll5).
- **`DualBackend::append_lines(n)`:**
  1. read the primary's cursor;
  2. set the capture half's cursor to it;
  3. forward to the primary with `?`;
  4. forward to the capture half with `let Ok(()) = …`.

  The primary is the ground truth for the cursor. Reading and syncing happen only inside `append_lines`, so `get_cursor_position` keeps its current meaning (scroll5, adversarial).

**Behind envision's new `scrolling-regions` feature.** `Cargo.toml` gains `[features] scrolling-regions = ["ratatui/scrolling-regions"]`. It is not in `default` or `full`.
- **`CaptureBackend::scroll_region_up(region, n)` / `scroll_region_down(region, n)`** have `TestBackend`'s visible semantics (`test.rs:384-448`), minus scrollback:
  - **`scroll_region_up`:** within rows `region` (each bound clamped to the height), shift rows up by `n` and fill the vacated bottom rows with reset blanks (`EnhancedCell::new()`). If `n` ≥ the region height, the region becomes reset blanks. `scroll_region_down` is the mirror image.
  - **Degenerate input is a no-op that touches nothing:** an inverted region (`start > end`, where TestBackend underflows and panics), an empty region, or `n == 0` (scroll5, adversarial).
  - **Cursor:** unchanged. The trait leaves it undefined, and TestBackend leaves it unchanged (`backend.rs:337`) (scroll5).
  - **Frame stamps:** every cell in rows whose content or position changed is stamped with the current frame. This intentionally differs from `clear()`/`clear_region`, whose `reset()` does not stamp. The difference is documented: a moved cell's old stamp is meaningless, and `diff_from` will report every shifted cell as changed (scroll5, adversarial).
  - **Per-row wide-owner flags** (`row_may_hold_owner`) move with their rows, and vacated rows get `false`. Rows move whole, so owners and their continuation cells stay together and rule 3 keeps working (adversarial). The conservative invariant holds.
- **`DualBackend`** forwards both scroll methods to the primary with `?`, then to the capture half with `let Ok(()) = …`.
- **Code layout:** new code goes in `src/backend/capture/scroll.rs` (`capture/mod.rs` is 735 lines).

**Remaining hazard (documented).** A consumer who enables `ratatui/scrolling-regions` directly, instead of envision's feature, still gets E0046 in envision. envision cannot observe ratatui-core's cfg. The documented fix is one line: enable `envision/scrolling-regions`.

**Behaviour note, documented with the feature.** With the feature on, ratatui's inline `insert_before` uses scroll regions, which means less flicker. On legacy non-VT Windows consoles, crossterm's `ScrollUpInRegion` returns `Unsupported` (`ratatui-crossterm-0.1.2/src/lib.rs:737`), so `insert_before` errors there. The feature is opt-in for this reason (adversarial).

**In-repo backends.** Under the feature, these also need the two methods:
- `tests/backend_errors.rs:30`;
- `tests/backend_error_types.rs:30`;
- `tests/trybuild_backend/foreign_error.rs:26`;
- the doctests at `src/error.rs:109` and `src/error.rs:147`.

Each gains them behind `#[cfg(feature = "scrolling-regions")]`, so they compile with and without the feature. The `compile_fail,E0277` doctest must still fail only for E0277; check on nightly (scroll5, adversarial).

**Docs:**
- **CHANGELOG `[Unreleased]`:** replace #524's "`scrolling-regions` is not yet supported" text with "supported via envision's opt-in `scrolling-regions` feature", in two places:
  - the "Supported ratatui features" paragraph under `#### ratatui re-exports` (`CHANGELOG.md:77`);
  - Known Deferred Findings (`:115`).
- **MIGRATION:**
  - § 7 (`MIGRATION.md:239`): describe the feature and the direct-enable hazard with its fix. Remove "as was already the case in v0.17".
  - § 3 (Custom backends): add a recipe for implementing `scroll_region_up`/`down` behind a forwarded feature, for backend authors who want to support it. Also note that enabling `envision/scrolling-regions` alongside a **direct** `ratatui-termion`/`-termwiz`/`-termina` dependency requires that crate's own `scrolling-regions` feature as well; otherwise that crate fails with E0046 (API reviewer).
  - § 6: note that inline-viewport captures now change with `append_lines`.
- The `insert_before` behaviour and legacy-Windows note.

### Implementation notes (PR 5 specialist, round 2)

- **Extra cursor round-trip.** `DualBackend::append_lines` adds a `get_cursor_position` call on the primary, which is a real terminal round-trip with crossterm. The rustdoc says so. A failed read propagates **before** either half is changed.
- **Clippy with the feature off.** CI's clippy job runs only `--all-features`, so locally also run `cargo clippy --all-targets --features <the default set> -- -D warnings`, so the feature-off cfg arms get clippy coverage.
- **trybuild.** trybuild passes the crate's enabled features to the fixture project (trybuild 1.0.119 `run.rs:131-165`; adversarial). Still, run `tests/trybuild_pass.rs` with and without the feature.

### Tests (TDD)

- **`append_lines`:**
  - with and without overflow;
  - cursor placement exactly as `TestBackend`;
  - near `u16::MAX`, as `TestBackend`'s own test does at `test.rs:931` (security).
- **`DualBackend::append_lines`:** the primary's cursor is on the last row and the capture's is at the origin. After the call, both halves scrolled identically and both cursors agree. Primary error propagation is unchanged.
- **Feature-gated scroll tests**, compiled only with the feature:
  - interior region;
  - region starting at row 0;
  - `n` ≥ height;
  - scroll-down mirror cases;
  - inverted, empty and `n == 0` regions are no-ops (no panic, nothing stamped);
  - frame stamps on affected rows only;
  - owner flags move with rows, and rule 3 still blanks a scrolled wide glyph;
  - additions to `assert_flags_conservative`.
- **Oracle** (proptest, under the feature):
  - The generator draws **width-1 symbols only**. Wide glyphs are excluded because `CaptureBackend` deliberately applies wide-glyph rules 1–4 that `TestBackend` does not.
  - Regions are constrained to `start ≤ end`.
  - Assertion: after the same `scroll_region_*`, each cell's **symbol and style** equals `TestBackend`'s, ignoring scrollback. Text-only comparison would miss vacated rows that weren't reset (scroll5, adversarial).
  - A **self-relative oracle** also covers wide glyphs: post-state row `r` equals pre-state row `σ(r)`, or is a reset blank.
- **Integration:** an inline viewport (`Viewport::Inline(h)` with `h` < the terminal height, so inserted lines are not pushed straight to scrollback; scroll5) plus `insert_before` on `DualBackend<CaptureBackend>`. Run once without and once with the feature; the capture shows the inserted lines.
- **CI coverage:** no new job is needed. The stable legs run `--all-features`, which enables `scrolling-regions`, so the feature-gated tests run there. The msrv leg (component features) and the No Default Features job build and test with the feature off, so both configurations are covered on every PR.

**Acceptance:**
- The full suite is green with and without the feature.
- `cargo tree` is unchanged apart from features. Verified: the feature adds no crates, and `Cargo.lock` locks 0 new packages (security, claudio).
- No file exceeds 1000 lines.

---

## PR 6a — dependency floors

A `-Zdirect-minimal-versions` run (CI/QA, security) shows that envision's declared floors admit versions that **fail to build, fail to resolve, or carry advisories**:
- `tokio = "1"` → 1.0.1: RUSTSEC-2021-0124, RUSTSEC-2021-0072, and unsound RUSTSEC-2023-0005 and RUSTSEC-2025-0023.
- `regex = "1"` → admits RUSTSEC-2022-0013, and conflicts with criterion.

Consumers of the published crate resolve these floors on their own lockfiles, so raising them is a direct security improvement.

**Method:**
1. On a pinned nightly, run `cargo update -Zdirect-minimal-versions` in a scratch worktree.
2. Raise each floor in `Cargo.toml` (normal **and** dev dependencies) to the lowest version that builds, passes tests, and audits clean.
3. Repeat until `cargo build --all-features --all-targets`, `cargo nextest run --all-features --profile ci` and `cargo audit --deny warnings` all pass on the minimised lockfile.

Expected raises, to be confirmed by the run:
- `regex` ≥ 1.5.5
- `serde` ≥ 1.0.228
- `serde_json` ≥ 1.0.100
- `tokio` ≥ 1.44.2
- `async-stream` ≥ 0.3.6
- `tracing` ≥ 0.1.41
- `trybuild` ≈ 1.0.86
- `arboard`: to be checked (it pulls unmaintained `adler`)
- `tokio-stream` / `tokio-util` / `futures-util`: to be checked

Each raise is recorded with its reason: build failure, advisory ID, or resolution conflict.

**Committed lockfile:** unchanged, apart from what the raised floors force.

**Consumer-visible:** a CHANGELOG `[Unreleased]` entry lists the raised floors (API reviewer).

**Acceptance:** the minimal-versions procedure above passes, and the normal suite and audit are green.

---

## PR 6 — `cargo deny` gate

**`deny.toml`** (repo root). The key names are validated against cargo-deny 0.20.2: the security reviewer's scratch config passes today. cargo-deny 0.20 removed the `vulnerability`/`notice`/`severity-threshold` keys, and `unsound`/`unmaintained` take a scope, not a lint level.

```toml
[graph]
all-features = true          # optional backends evaluated as compiled; no `targets` → all platforms

[advisories]
version = 2
unmaintained = "all"         # explicit (already the default)
unsound = "all"              # default is direct-deps-only scope; "all" covers transitive (e.g. lru)
yanked = "deny"
unused-ignored-advisory = "deny"
ignore = []                  # exceptions only per the advisory policy below

[bans]
multiple-versions = "deny"
wildcards = "deny"
# multiple-versions-include-dev defaults to false: dev-only duplicates are out of scope
skip = [ … ]                 # see skip policy

[sources]
unknown-registry = "deny"
unknown-git = "deny"
```

**Skip policy.** Skip **only the older version**, as `name@version`, which is a caret requirement. Each entry's `reason` field contains:
- **a tracking link** (an upstream issue/PR, or a tracker search showing the split is ecosystem-wide);
- **a re-check trigger**: a specific upstream release, or "re-check at each envision release".

A removal *version* is not required, because ecosystem-wide splits (`syn` 2/3, `windows-sys`) have none (adversarial). Stale or unneeded skips are surfaced by running with `-D warnings` (below) (security).

Expected entries on the post-4 tree, to be re-derived from `cargo deny check bans`:
- `hashbrown` 0.16 vs 0.17: ratatui-core via `kasuari` vs `lru`.
- `miniz_oxide` 0.8 vs 0.9: `arboard` → `image` → `png` vs `flate2`.
- `syn` 2 vs 3: via proc-macros. Build dependencies **are** in the gate's scope (only dev dependencies are excluded), so this needs a skip (claudio).
- `windows-sys` 0.60 vs 0.61: `arboard` 3.6.1 vs `rustix`/`errno` (security, adversarial, CI/QA).

Where an upstream issue is the right tracking link and none exists, a draft is prepared and **filed only with maintainer approval**. Until then, the ecosystem-search link with an "at each release" trigger satisfies the policy, so the gate does not block on public actions.

**Advisory response policy** (maintainer decision). When the required job turns red on a new advisory:
1. Fix forward: update the dependency or raise its floor (as in PR 6a).
2. Only if no fixed version exists, add an `ignore` entry with the advisory ID, a linked tracking issue, a reason, and a re-check date. Review it like code.

`unused-ignored-advisory = "deny"` removes entries once they are no longer needed.

**CI** (`.github/workflows/ci.yml`):
- **`cargo-deny` job:** `cargo install cargo-deny --locked --version 0.20.2`, then `cargo deny --locked check -D warnings advisories bans sources`.
  - It does not use `EmbarkStudios/cargo-deny-action`. That action's Dockerfile fetches the release tarball with no checksum, runs an unpinned `apk add`, and uses a Rust 1.85 image (security).
  - The version is pinned and bumped by hand, as for tarpaulin in #523.
  - It runs on PRs and on pushes to `main` (all three checks).
  - A **daily `schedule:`** run checks `advisories` only, so a red scheduled run always means "new advisory". New transitive duplicates surface on the next PR (security, CI/QA).
- **`minimal-versions` job:**
  1. `cargo +nightly-YYYY-MM-DD update -Zdirect-minimal-versions`;
  2. `cargo build --all-features --all-targets`;
  3. `cargo nextest run --all-features --profile ci`;
  4. `cargo audit --deny warnings`, with cargo-audit installed `--locked` at a pinned version.

  Details:
  - The nightly date is pinned and bumped deliberately. A trybuild `.stderr` drift on a nightly bump is handled in that bump's PR (CI/QA).
  - **Cache key:** `Cargo.toml` hash plus the nightly date, not `Cargo.lock`, because the job regenerates the lockfile (CI/QA).
- **Change detection:** neither job is gated by the `changes` (Detect Changes) filter. A skipped job satisfies a required check, so they must always run (CI/QA).
- **Required checks:** both jobs are proposed as required. Changing branch protection is confirmed with the maintainer at merge time, as for #522.

**Scope note.** `-Zdirect-minimal-versions` checks only envision's own floors. Transitive floors such as ratatui-core's `lru` are covered by the full `-Zminimal-versions` audit in the pre-release audit (security, claudio).

**Acceptance:**
- `cargo deny --locked check -D warnings advisories bans sources` passes locally.
- Both jobs pass in CI on the PR.
- Every skip has a tracking link and a re-check trigger.
- `[licenses]` is out of scope; a licence policy is a separate maintainer decision.

---

## PR 8 — upstream `lru` floor: track, don't file

ratatui-core 0.1.2 declares `lru = "0.18"`, which admits 0.18.0 and 0.18.1. Both are affected by RUSTSEC-2026-0253, which is unreachable through ratatui (#518 § 8).

**Upstream PR ratatui/ratatui#2825** ("build(deps): require patched lru 0.18.2", opened 2026-10-10) already does this (security). **No new issue is filed.**

When a ratatui release containing it ships, envision raises its `ratatui` floor to that version in a small follow-up PR. Until then, the CHANGELOG's existing `cargo update -p lru` note stands.

---

## R — release preparation (one PR)

The adversarial reviewer found that publishing today would ship a degraded package.
- **docs.rs:** add `[package.metadata.docs.rs] all-features = true`, so `tracing`, `test-utils` (AppHarness) and `scrolling-regions` APIs appear on docs.rs.
  - envision uses no `doc_cfg`/`docsrs` today, and this PR does not introduce it.
  - Instead, the rustdoc of every feature-gated item states its feature in prose, e.g. "Requires the `scrolling-regions` feature." That applies to `CaptureBackend`'s and `DualBackend`'s type docs for the scroll methods (adversarial, API reviewer).
- **Package contents:** add an `exclude` (or `include`) list so internal material is not published: `docs/superpowers/`, `docs/audits/`, `fastbreak.toml`, `tarpaulin.toml`, `.claude/`, `.superpowers/`. Verify with `cargo package --list`. Excluded paths must not break `include_str!` doctests (README stays included).
- **Version and docs:**
  - `version = "0.18.0"`;
  - README install snippet `0.17` → `0.18`;
  - CHANGELOG `[Unreleased]` → `[0.18.0] - <date>`, keeping a fresh empty `[Unreleased]`.
- **Pre-release audit:** a full audit run that includes `-Zminimal-versions` + `cargo audit`, with findings recorded in `docs/audits/`, then `/release`. Both the audit and the release are confirmed with the maintainer.

## Out of scope

- #525: export escape sanitising, to be designed with #125.
- #125: hyperlinks.
- A `cargo deny` licence policy.
- The remaining CHANGELOG Known Deferred Findings.
- Workflow-level hardening noted by the security reviewer (tag-pinned actions, no top-level `permissions:`, no Dependabot) — recorded here for a follow-up security-hardening PR.
