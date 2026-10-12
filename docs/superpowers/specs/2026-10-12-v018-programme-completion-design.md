# v0.18.0 programme completion (PRs 4–8) — design

## Purpose

Finish the security programme defined in `2026-10-09-ratatui-0.30-upgrade-design.md` (§ Programme). PRs 1–3 have merged:
- #521 lockfile refresh;
- #522 MSRV 1.88;
- #524 ratatui 0.30.2.

Two supporting fixes also merged: #520 (flaky tests) and #523 (Coverage). This document designs the remaining five items. Each ships as its own branch and PR.

The goal is unchanged from the maintainer: **the library should be as free of security issues as possible.** v0.18.0 is released once PRs 4–7 have merged (PR 8 is upstream and does not block). The full pre-release audit follows, including the `-Zminimal-versions` audit step.

### Maintainer decisions (2026-10-12)

| Decision | Choice |
|---|---|
| PR 4 | Bump `compact_str` to 0.9; keep it in the public API. |
| PR 5 | envision always enables ratatui's `scrolling-regions` feature and always implements the scroll methods. |
| Merges | Squash-merge PRs 4–7 after an independent review and all-green CI. Public actions outside envision (upstream issues) are confirmed first. |

### Order and dependencies

| PR | Depends on | Why |
|---|---|---|
| 7 dev-deps | — | Removes the dev-only `itertools` 0.10 duplicate needed for PR 6. |
| 4 `compact_str` | — | Removes the `compact_str` 0.8/0.9 duplicate needed for PR 6. |
| 5 `DualBackend` / scroll regions | — | Independent. Must update the CHANGELOG/MIGRATION "`scrolling-regions` is not yet supported" text from #524. |
| 6 `cargo deny` gate | 4, 7 | Its duplicate policy is designed against the post-4/7 tree. |
| 8 upstream `lru` floor | — | Draft only; posting needs maintainer approval. |

Execution order: 7 → 4 → 5 → 6, with PR 8's draft presented alongside.

---

## PR 7 — dev-dependency upgrades

**Changes** (`Cargo.toml` `[dev-dependencies]`):
- `criterion` `0.5` → `0.8`. Needs Rust 1.86, which is within MSRV 1.88. Drops `rayon`/`crossbeam-*` from the bench graph as far as 0.8 allows; the plan verifies with `cargo tree`.
- `proptest` `~1.8` → `1.11`. The `~1.8` pin existed only for MSRV 1.81, so it is stale.
- `rand` `0.9` → `0.10`.

**Code fallout**, fixed at its source, never silenced:
- `criterion::black_box` → `std::hint::black_box`. It is used in `benches/component_events.rs` and possibly elsewhere; the plan enumerates with grep.
- Any criterion 0.8 API changes.
- `rand` 0.10 API changes in `benches/sort_bench.rs` and any other user.
- proptest 1.11 strategy changes, if any.

**Acceptance:**
- `cargo bench --all-features --no-run` builds every bench.
- `cargo tree -d --all-features` no longer shows `itertools` 0.10.
- Clippy with `-D warnings` is clean on stable and 1.88, including `--all-targets`.
- The full suite is green. The four CI bench jobs still run; their saved baselines become stale and are re-established by the first run on `main` after merge, which is noted in the PR.
- `cargo audit --deny warnings` exits 0.

**Not in scope:** the pre-existing `benches/capture_backend.rs` failure without the `serialization` feature (Known Deferred Findings). It is a separate fix, because it is about feature-gating a bench, not dependency versions.

---

## PR 4 — `compact_str` 0.8 → 0.9

**Current public exposure:**
- `component::cell::SortKey::String(CompactString)` (`src/component/cell.rs:27`);
- `impl Into<CompactString>` parameters on `Cell::new`, `with_text`, `success`, `warning`, `error`, `muted` and `severity` (`src/component/cell.rs:193-351`);
- `EnhancedCell::with_symbol` and `set_symbol` (`src/backend/cell/mod.rs:126,159`).

ratatui-core 0.1.2 already depends on `compact_str` 0.9. After #524 the tree carries both 0.8 (envision) and 0.9 (ratatui-core).

**Change:**
- `compact_str = "0.9"`.
- The `serialization` feature keeps `compact_str/serde`.
- Fix any 0.9 API fallout.
- **Re-export** `envision::compact_str`, a `pub use compact_str;` at the crate root, for the same reason as `pub use ratatui;`. `CompactString` appears in a public enum variant, so consumers who match on `SortKey::String` must name the exact version envision uses. The rustdoc states the policy: a `compact_str` major bump is a breaking envision release.

**Breaking-change documentation:**
- CHANGELOG `[Unreleased]` and MIGRATION v0.17.x → v0.18.0 cover the `CompactString` type identity change. Code that only passes `&str`/`String` into the `impl Into<…>` parameters is unaffected.
- The "commit-or-drop" Known Deferred Finding is closed with this decision: commit, because the no-allocation storage benefits capture of ~2k cells per frame and adds no crates.

**Acceptance:**
- `cargo tree -d -e normal` shows no `compact_str` duplicate.
- The serde round-trip of `SortKey` and `EnhancedCell` JSON is unchanged. An existing JSON test covers this; the plan confirms it, and adds one if missing.
- The full suite is green, clippy is clean, and `cargo audit --deny warnings` exits 0.

---

## PR 5 — `DualBackend` forwarding and scrolling regions

**Problems on `main`:**
1. `DualBackend` does not forward `append_lines`. The trait default, a no-op, runs, so inline-viewport output never reaches the real terminal.
2. `scroll_region_up` / `scroll_region_down` are required `Backend` methods under ratatui's `scrolling-regions` feature (`ratatui-core-0.1.2/src/backend.rs:362-367`). envision implements neither. Any crate in a consumer's graph enabling `ratatui/scrolling-regions` makes envision fail to compile (E0046). This is documented as a known limitation in #524.
3. `CaptureBackend` does not implement `append_lines`. The trait default is a no-op, so captures of inline viewports are wrong.

**Design:**
- **Always enable the feature:** `Cargo.toml` ratatui features gain `"scrolling-regions"`. The feature only toggles cfg flags in ratatui-core and ratatui-crossterm; no crates are added. The plan verifies with `cargo tree` before and after.
- **Always implement** `scroll_region_up`, `scroll_region_down` and `append_lines` for `CaptureBackend` and `DualBackend`, with no `#[cfg]`. They compile because the feature is always on, so users can never hit E0046.
- **`CaptureBackend` semantics mirror ratatui's `TestBackend`** (`ratatui-core-0.1.2/src/backend/test.rs:326-448`), minus scrollback, which `CaptureBackend` does not model; rows scrolled off the top are discarded:
  - **`scroll_region_up(region, n)`:** within the rows `region` (clamped to the height), shift rows up by `n` and fill the vacated bottom rows with reset blanks (`EnhancedCell::new()`). If `n` ≥ region height, clear the whole region.
  - **`scroll_region_down(region, n)`:** the mirror image; vacated top rows become reset blanks.
  - **`append_lines(n)`:** as `TestBackend`. Move the cursor down `n` lines, scrolling the whole buffer up by the overflow and filling with reset blanks. The cursor x advances as in `TestBackend` (`new_cursor_x = (x+1).min(width-1)`).
  - **Frame stamps:** every cell whose content or position changed is stamped with the current frame, i.e. every cell in the affected rows. A moved cell counts as written, matching what a terminal redraw shows.
  - **Per-row wide-owner flags** (`row_may_hold_owner`, #524 ruling R11) move with their rows (the same rotate as the cells). Vacated rows get `false`. The flag invariant stays conservative.
- **`DualBackend`:** forwards all three to the primary with `?`, then to the capture half with `let Ok(()) = …`, matching the existing forwarding pattern.
- **Behaviour change, documented:** with the feature on, ratatui's `Terminal::insert_before` in inline viewports uses scrolling regions, which means less flicker. CHANGELOG `[Unreleased]` notes this. #524's "`scrolling-regions` is not yet supported" text in the CHANGELOG (Supported ratatui features + Known Deferred Findings) and MIGRATION § 7 is replaced with "supported and always enabled".

**Tests (TDD):**
- **Scroll-up within an interior region:** rows shift, the bottom is blank, rows outside the region are untouched, and frames are stamped.
- **Region starting at row 0:** top rows are discarded.
- **`n` ≥ height:** the region is cleared.
- **Scroll-down:** the mirror cases of the above.
- **`append_lines`:** with and without overflow; the cursor ends where `TestBackend` puts it.
- **Owner flags:** move with rows. A wide glyph scrolled into another row still gets rule-3 blanking. Add to the `assert_flags_conservative` invariant tests.
- **`DualBackend`:** each of the three reaches both halves. A `FailingBackend`-style primary error propagates and the capture half is not called after the primary fails, matching existing forwarding.
- **Equivalence oracle:** for random small grids and region/`n` combinations (proptest), `CaptureBackend`'s visible text after `scroll_region_*` equals `TestBackend`'s buffer after the same operation, ignoring scrollback.
- **Integration:** a `Terminal::with_options(..Viewport::Inline(..))` + `insert_before` scenario on `DualBackend<CaptureBackend>` captures the inserted lines.

**Acceptance:**
- Full suite green, clippy clean, files under 1000 lines (`capture/mod.rs` is ~735 lines; new code goes in a `capture/scroll.rs` module).
- `cargo tree` unchanged apart from features.

---

## PR 6 — `cargo deny` gate

**`deny.toml`** (new, repo root):
- **`[advisories]`:** deny vulnerabilities, unmaintained (all scopes), unsound and yanked crates. **Empty ignore list.**
- **`[sources]`:** crates.io only; `unknown-registry` and `unknown-git` are `deny`.
- **`[bans]`:**
  - `multiple-versions = "deny"` and `wildcards = "deny"`;
  - a short `skip` list. Each entry is an upstream-caused duplicate envision cannot fix, carrying in a comment **a link to an upstream issue or PR and the upstream version that removes it**.

  Expected entries after PRs 4 and 7, to be re-derived from `cargo deny check bans`:
  - `hashbrown` 0.16 vs 0.17: ratatui-core via `kasuari` vs `lru`;
  - `miniz_oxide` 0.8 vs 0.9: `arboard` → `image` → `png` vs `flate2`;
  - `syn` 2 vs 3: proc-macro build dependencies.

  For each, the implementer searches the upstream tracker for an existing issue. **Where none exists, a draft issue is prepared and filed only after maintainer approval** (a public action). Until it is filed, the PR cannot merge, because an entry without a link is not allowed.
- **`[graph]`:** all features, all targets, so optional backends are evaluated as compiled. This is the feature-aware complement to `cargo audit`'s lockfile view.
- **`[licenses]`:** out of scope for this PR. The CI invocation is `cargo deny check advisories bans sources`. A licence policy is a separate decision for the maintainer.

**CI** (`.github/workflows/ci.yml`):
- **`cargo-deny` job:** runs `cargo deny check advisories bans sources`. It uses `EmbarkStudios/cargo-deny-action` **pinned to a full commit SHA**, a supply-chain measure, with a comment naming the release tag. Alternatively it uses `cargo install cargo-deny --locked --version <x>`, matching #523's tarpaulin pinning; the plan picks one and records why.
- **`minimal-versions` job:** `cargo +nightly-YYYY-MM-DD update -Zdirect-minimal-versions`, then `cargo build --all-features --all-targets`, `cargo nextest run --all-features --profile ci` and `cargo audit --deny warnings`. The nightly is pinned to a date and bumped deliberately (CI/QA review of #518).
- **Required checks:** both jobs are proposed as required. Changing branch protection is a repository-settings action, confirmed with the maintainer at merge time, as for #522.
- **Floors the job uncovers:** if it reveals a declared floor in envision's `Cargo.toml` that is too low (fails to build, or audits dirty), raise that floor in this PR. That is the job doing its purpose.

**Acceptance:**
- `cargo deny check advisories bans sources` passes locally with an empty advisory ignore list.
- Both jobs pass in CI on the PR.
- Every `skip` entry has its upstream link and removal version.

---

## PR 8 — upstream `lru` floor (draft only)

ratatui-core 0.1.2 declares `lru = "0.18"`, which admits 0.18.0 and 0.18.1. Both are affected by RUSTSEC-2026-0253, fixed in 0.18.2. Through ratatui the issue is unreachable (spec of #518 § 8), but consumers' lockfiles can still resolve a flagged version, and their audits will report it.

**Draft issue for `ratatui/ratatui`:**
- **Title:** "ratatui-core: require `lru >= 0.18.2` (RUSTSEC-2026-0253)".
- **Body:**
  - the advisory and the affected range;
  - the fact that ratatui-core only calls `get_or_insert`/`put`/`resize`, so it is not exploitable through ratatui;
  - the request: raise the floor to `0.18.2` so downstream `cargo audit` and minimal-versions builds are clean;
  - an offer to send the one-line PR.

**Not posted** without explicit maintainer approval. Once upstream ships a fixed release, envision raises its ratatui floor to it in a follow-up.

---

## Release

After PRs 4–7 have merged:
1. Run the pre-release audit, including the full `-Zminimal-versions` + `cargo audit` step recorded in the audit report.
2. Bump the version to 0.18.0, including the README's `0.17` install snippet.
3. Run the `/release` skill.

Releasing is a public action, confirmed with the maintainer.

## Out of scope

- #525: export escape sanitising, to be designed with #125.
- #125: hyperlinks.
- The deferred findings listed in CHANGELOG `[Unreleased]`, apart from those closed here (the `compact_str` decision; `scrolling-regions` support).
- The licence policy for `cargo deny`.
