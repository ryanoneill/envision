# Envision pre-release-hygiene audit — 2026-07-04

**Auditor:** Fable (via `superpowers:audit` skill)
**Target commit:** `5560125` (main HEAD prior to the release-readiness cadence, after May 2026 leadline queue closure)
**Envision version at time of audit:** v0.16.0 tag published 2026-04-20; ~11 weeks of unreleased API-quality work on main
**Session context:** Audit output preserved in the release-readiness cadence conversation transcript. This file is the index/summary; the full 25-category evaluation is captured in the accompanying post-cadence audit at [`2026-07-05-post-release-hygiene.md`](2026-07-05-post-release-hygiene.md), which was run against the post-cadence tree at commit `21594c6` after the release-readiness impl PR #504 merged.

## Verdict

- **Overall grade:** `A-` (weighted GPA 3.62)
- **Prior baseline (v0.15.1, 2026-04-18):** `A` (4.02)
- **Delta:** −0.40 GPA — regression driven entirely by release-hygiene surface (docs, MIGRATION.md, CHANGELOG structure), not engineering substance.

## Executive summary (verbatim from audit)

> Envision remains a demanding-standard TUI framework — 74 components, 100% doc-test coverage (1776/1776), zero unsafe, zero clippy suppressions, 648 insta snapshots, 50K-item stress tests, and a compile-time-enforced App::init args design are the hallmarks of a serious library. Since v0.15.1 (A, 4.02), the picture has regressed slightly: three separate `[Unreleased]` sections have piled up in the CHANGELOG above a shipped 0.16.0, README code examples no longer compile against the current `App` trait, MIGRATION.md stops at v0.14→v0.15 (missing 0.15→0.16 and the two pending breaking changes), and the G1/G3/G7 "sort/cell unification" left `file_browser` with its own separate `FileSortField`/`FileSortDirection` enums. The library is release-ready in engineering substance; the release-hygiene meta layer is not, and a demanding consumer will notice the README rot in their first five minutes.

## Group-level GPAs

| Group | Weight | GPA |
|---|---|---|
| 1. First Impressions | 15% | 3.13 |
| 2. API Design | 25% | 3.46 |
| 3. Engineering Quality | 20% | 3.67 |
| 4. Testing | 20% | **4.00** |
| 5. Architecture | 10% | 3.80 |
| 6. Missing Pieces | 10% | 3.68 |

## 5 blocking findings (drove the regression)

Each finding is closed by the release-readiness cadence — see accompanying [post-cadence audit](2026-07-05-post-release-hygiene.md) for the verification pass.

1. **README code examples do not compile against the current API.** TEA snippet at `README.md:71-91` omits `type Args`; testing snippet at `README.md:103-109` calls `Runtime::<MyApp>::virtual_terminal(80, 24)`, `runtime.dispatch`, `runtime.render`, `runtime.contains_text` — none of which exist on the real `Runtime` (those are `AppHarness` methods, and construction is `virtual_builder(w, h).build()`). Nothing catches this because README is not `include_str!`'d as a doctest. First 5 minutes fail.

2. **CHANGELOG has THREE stacked `## [Unreleased]` sections above the released `0.16.0`** (lines 8, 362, 389). Two are marked breaking. A demanding consumer reads this as "the maintainer is behind on releases; what will I be adopting?"

3. **MIGRATION.md stops at `v0.14.x to v0.15.0`.** No `v0.15→v0.16` entry; no entries for the two pending breaking changes (App::init args, Table sort/cell redesign) despite CHANGELOG marking them Breaking. On upgrade, users have to reverse-engineer from CHANGELOG prose.

4. **Two divergent sort systems post-"unification".** G1/G3/G7 unified Table around `SortKey` / `SortDirection` / `InitialSort`, but `file_browser` still defines its own `FileSortField` and `FileSortDirection` enums with a `with_sort_field` / `with_sort_direction` builder pair. Two `*SortDirection` enums for the same concept.

5. **`ResourceGauge::set_values(actual, request, limit)`** at `src/component/resource_gauge/mod.rs:503` has no matching `values()` getter — the only unpaired mutator in the library (scorecard: 8/9, was 9/9). The same component's `ResourceGauge::new(actual, request, limit)` is three unlabeled positional `f64`s with no builder alternative; transposing `request`/`limit` is silent. And `ResourceGauge` is the only new component with zero examples.

## Additional trust-eroding findings (deferred to future cadences)

Per the spec's Known Deferred Findings block in CHANGELOG under `[Unreleased]`, two audit findings intentionally deferred beyond v0.17.0:

- **Finding #6:** `selected_value` / `selected_item` / `active_tab` accessor shape divergence across `dropdown`, `select`, `heatmap`, `tab_bar`, and `data_grid`. Requires a dedicated consistency-sweep cadence.
- **Finding #8:** Dependency leakage in 8 public signatures (`ratatui::layout::Position`, `ratatui::buffer::Cell`, `ratatui::style::Color/Style`, `ratatui::widgets::Widget`, `tokio::sync::mpsc::Sender` at `harness/app_harness/mod.rs:264`, plus 3 others). Architectural discussion; not release-blocking.

## Top 5 improvements (highest ROI, all addressed by the release-readiness cadence)

1. Fix the README code examples to match current API and wire them into the crate via `#[doc = include_str!("../README.md")]` + `cargo test --doc` so they can't rot silently again. Add a Feature Flags section.
2. Collapse the three stacked `[Unreleased]` CHANGELOG sections into a single coherent entry, and cut the 0.17.0 release.
3. Add MIGRATION.md entries for v0.15→v0.16 and for the pending breakings (App::init args, Table sort/cell unification).
4. Unify `file_browser`'s sort with the new `table::SortDirection` / `SortKey`-shaped primitives.
5. Close the `resource_gauge` gaps: add `values()`, ship an example, and consider replacing the three-`f64` `new()` with a builder chain.

## Cadence that closed these findings

- **Spec:** [`docs/superpowers/specs/2026-07-04-release-readiness-cadence-design.md`](../superpowers/specs/2026-07-04-release-readiness-cadence-design.md) (PR #502, commit `bb0e8fe`)
- **Plan:** [`docs/superpowers/plans/2026-07-04-release-readiness-cadence.md`](../superpowers/plans/2026-07-04-release-readiness-cadence.md) (PR #503, commit `36c36cc`)
- **Impl:** PR #504, merge commit `21594c6` — three signed commits (release-hygiene / file-browser-sort / resource-gauge-closure) + MIGRATION.md fill-in + 2 small fix commits (grep-gate reword, `ReadmeDoctests` feature gate)
- **Post-cadence audit:** [`2026-07-05-post-release-hygiene.md`](2026-07-05-post-release-hygiene.md) — verifies the 5 findings closed and grade recovered to `A` or above
