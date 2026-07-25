# Envision Library Audit — Post-Cleanup (2026-07-05)

```
==========================================================================
ENVISION LIBRARY AUDIT REPORT
Date: 2026-07-05
Version: 0.16.0 (v0.17.0 pre-release, main)
Commit: 4f3bd06
Test Results: 7306 unit / 2591 doc / 7466 integration (17,363 total, 0 failed, 10 ignored)
==========================================================================

OVERALL GRADE: A (3.90 weighted GPA)

EXECUTIVE SUMMARY: The consistency-cleanup cadence delivered: selection
accessors are canonicalized on selected_item(), the tokio Sender leak on
AppHarness is closed behind a first-party MessageSender<M> newtype, and the
scorecard is 9/9 with zero clippy suppressions across 195K lines of source.
The grade holds at A rather than climbing because two hygiene items remain
open on main (CHANGELOG/MIGRATION over the 1000-line cap, redundant ratatui
imports in examples — both queued in PRs #510/#511) and this pass surfaced
new cosmetic README defects (duplicate component tables, a version snippet
that will not resolve until v0.17.0 ships).

GRADE BREAKDOWN:
--------------------------------------------------------------------------
Group 1: First Impressions (15%)                          GPA 3.80
  1.  Getting Started .............. A-  Excellent quick start, doc-tested README; marred by duplicate tables + "0.17" version snippet
  2.  Examples ..................... A-  91 examples, 100% component coverage, all compile; redundant ratatui imports pending PR #511
  3.  Documentation ................ A   100% doc-test coverage on 1777 pub fns, missing_docs enforced, 232 # Examples sections

Group 2: API Design (25%)                                 GPA 3.88
  4.  Consistency & Symmetry ....... A-  selected_item() canonical, 0 accessor gaps, 73/73 derives; 17 undeprecated selected() aliases remain
  5.  Modularity & Composability ... A-  14 feature flags, minimal leakage (6 sites, mostly deliberate); tokio is mandatory
  6.  Usability & Ergonomics ....... A   Complete prelude, instance methods, typestate builder turns missing args into compile errors
  7.  Complexity Hiding ............ A   MessageSender<M> newtype verified; prelude ships first-party layout/style types; no cfg(test) gating
  8.  Type Safety & Errors ......... A   Structured EnvisionError, first-party TrySendError/MessageSendError, trybuild compile-fail tests

Group 3: Engineering Quality (20%)                        GPA 4.00
  9.  Algorithms & Data Structures . A-  compact_str consistent, alloc-tracking benches; no hot-path red flags found
  10. Smart Library Usage .......... A+  0 unsafe, 0 clippy suppressions in 195K lines, every heavy dep optional and gated
  11. Performance & Benchmarking ... A   6 criterion suites incl. 1000-item view + alloc benches, run in CI with reports

Group 4: Testing (20%)                                    GPA 4.00
  12. Unit Testing ................. A-  4316 component tests; per-component counts uneven (box_plot 112 vs conversation_view 14)
  13. Integration & E2E Testing .... A   11 suites: proptest x2, stress, async, serialization, trybuild
  14. Doc Test Coverage ............ A+  2591 doc tests, 100.0% of public fns — best in class

Group 5: Architecture (10%)                               GPA 3.77
  15. Solving Pain Points .......... A   Headless testing is first-class; 6 built-in themes, custom themes via public Theme
  16. Code Organization ............ B+  CHANGELOG (1469) and MIGRATION (1265) violate the project's own 1000-line cap; PR #510 pending
  17. Extensibility ................ A   Custom components/themes/subscriptions first-class; disciplined CHANGELOG + semver

Group 6: Missing Pieces (10%)                             GPA 3.85
  18. Feature Flags ................ A   Component-group opt-outs, README table documents every flag with defaults
  19. Error Infrastructure ......... A-  Structured hierarchy; only 33 # Errors sections and no error-handling guide
  20. Logging & Debugging .......... A-  tracing feature, 30 call sites in runtime/command/table; not full event-flow tracing
  21. Guides & Migration ........... A   MIGRATION covers v0.5 through v0.17 with before/after tables; all 6 v0.17 breaks present
  22. Advanced Features ............ A   Clipboard, undo/redo, search/filter, lifecycle hooks all present
  23. Serialization ................ A   serialization feature + load_state + dedicated test suite
  24. Security ..................... A-  SECURITY.md (102 lines), ANSI parser isolated; no escape-sequence fuzzing
  25. Ecosystem Integration ........ A-  Idiomatic ratatui, DualBackend; hard-wired to tokio, no alternative async runtime
--------------------------------------------------------------------------

Weighted GPA: 3.80(.15) + 3.88(.25) + 4.00(.20) + 4.00(.20) + 3.77(.10) + 3.85(.10) = 3.90 -> A
```

## Changes vs 2026-07-05 post-release-hygiene baseline (A, 3.91)

| Category | Baseline | Now | Why |
|---|---|---|---|
| 4. Consistency & Symmetry | B+ | A- | Consistency-cleanup canonicalized `selected_item()` across 6 components (finding #6 closed) |
| 7. Complexity Hiding | A- | A | `MessageSender<M>` newtype removed `tokio::sync::mpsc::Sender` from `AppHarness::message_sender()` (finding #8 closed) |
| 1. Getting Started | A | A- | New: duplicate "Display Components" tables + unresolvable `version = "0.17"` snippet in README |
| 16. Code Organization | B+ | B+ | Held — CHANGELOG/MIGRATION still over 1000 lines on main; PR #510 pending |
| 2. Examples | A- | A- | Held — redundant `use ratatui::` imports still on main; PR #511 pending |

Net: 3.91 → 3.90. The two closed findings were offset by newly surfaced README defects and the two still-open hygiene PRs.

## Trust-Eroding Findings (ranked by severity)

1. **[HELD — pending PR #510] CHANGELOG.md (1469 lines) and MIGRATION.md (1265 lines) exceed the project's own 1000-line cap.** The scorecard passes only because it counts `.rs` files. A project that enforces a rule on itself and visibly violates it in its two most-read documents erodes trust. Affects category 16. *Resolution queued: PR #510 splits to 356/224 lines with `-legacy.md` archives.*
2. **[NEW — cosmetic/moderate] README has two `### Display Components` tables** (README.md:241 and README.md:279) with ~10 overlapping rows (Spinner, StatusBar, StatusLog, StyledText, TitleCard, Toast, ScrollableText, ProgressBar, MultiProgress, KeyHints each listed twice, with *different descriptions*, e.g. "Visual progress indicator with percentage" vs "Progress display with ETA and rate"). This is the front page; a prospective user notices in the first 5 minutes. Affects categories 1, 3.
3. **[NEW — cosmetic, self-healing at release] README feature-flags snippet pins `version = "0.17"`** (README.md:47) while the latest published crate is 0.16.0. Anyone copying the snippet today gets a resolution failure (`^0.17` matches nothing). Self-heals when v0.17.0 ships, but should be verified as part of the release checklist. Affects category 1.
4. **[HELD — pending PR #511] 46 `use ratatui::` lines across 23 of 91 examples**, roughly half redundant with the prelude. Teaches users a dependency-awareness they don't need. Affects categories 2, 7. *Resolution queued: PR #511 drops 26 redundant lines from 8 files; remainder are legitimately-not-in-prelude concrete widgets.*
5. **[DEFERRED — Cadence D backlog] 17 components still carry an undeprecated `selected()` alias for `selected_index()`** alongside the canonical `selected_item()`. Not a regression, but two ways to ask the same question, with no `#[deprecated]` marker steering users to the canonical one. Affects category 4.
6. **[NEW — cosmetic] Naming outliers:** `tab_bar` uses `set_selected_index()` where every sibling uses `set_selected()`; `checkbox` exposes both `checked()` and `is_checked()`; `gauge` uses `label_text()` where siblings use `label()`. Affects category 4. Roll into Cadence D.
7. **[NEW — cosmetic] `restore_terminal` is exported at crate root under a different name** (`pub use crate::app::restore_terminal as restore;`, lib.rs) — the audit tool flags it as a re-export gap. One concept, two names depending on import path. Affects category 4.
8. **[NEW — cosmetic] README "Utility" table lists `AppShell`** but AppShell lives in `src/layout/` and is not one of the 74 components counted two paragraphs earlier. Affects categories 1, 3.

## Top 5 Improvements (highest ROI)

1. **Merge PR #510** (doc split) — category 16 from B+ to A-. Already queued; check `gh pr checks 510` and land it.
2. **Fix the duplicate Display Components tables in README.md** — category 1 from A- back to A. Merge the two tables into one (or split display vs. feedback components with distinct, non-overlapping membership) and reconcile the conflicting per-component descriptions. Small, high-visibility, should ride with the doc-hygiene cadence.
3. **Merge PR #511** (example imports) — category 2 from A- to A. Already queued.
4. **Add `#[deprecated(note = "use selected_item()/selected_index()")]` to the 17 remaining `selected()` aliases** — category 4 from A- toward A. Mechanical, and turns the Cadence D backlog into a compiler-guided migration for consumers.
5. **Add a release-checklist assertion that README version snippets match the about-to-publish version** — protects category 1 permanently. The `/release` skill should verify `grep 'version = "' README.md` against `Cargo.toml` post-bump.

## Detailed Findings

### Group 1: First Impressions

**1. Getting Started — A-**
- README (409 lines) states purpose in the first sentence, badges for CI/coverage/docs/crates/license.
- README is doc-tested via `#[doc = include_str!("../README.md")]` (src/lib.rs:431) — the 2026-07-04 finding #1 (compile-broken README) remains closed, structurally.
- Feature Flags section (README.md:23-48) survived the release-readiness cadence intact: full 14-flag table with defaults and an opt-out example. This is better feature-flag documentation than most A-list crates ship.
- Quick start: `cargo add envision`, then four progressive snippets (CaptureBackend → TEA → AppHarness → TestHarness). MSRV 1.85 declared (README.md:405) and tested in CI.
- Defects: duplicate Display Components tables (finding 2 above); `version = "0.17"` unresolvable snippet (finding 3); `AppShell` misplaced in the Utility table (finding 8).

**2. Examples — A-**
- 91 example files, 17,852 lines, progressive complexity from `counter_app` to `component_showcase` (891 lines) and `beautiful_dashboard` (625). All compile (`cargo build --examples --all-features` PASS).
- Component coverage: 73/73 (100%) per the audit tool.
- Pattern coverage: focus management (`focus_manager`, `component_showcase`), testing (`test_harness`), async (`async_counter`), composition (`file_manager`, `log_explorer`, `chat_client`).
- Held back by the 46 `use ratatui::` lines across 23 files pending PR #511.

**3. Documentation — A**
- `#![warn(missing_docs)]` enforced; type-level docs 492/492 (100%); module docs 115/124 (93% — the 9 missing are all internal `tests/mod.rs` files, invisible to consumers).
- Doc-test coverage 1777/1777 public fns (100%). 232 `# Examples`, 33 `# Errors`, 15 `# Panics` sections.
- 28 `no_run`/`ignore` doc fences; sampled ones are legitimate (terminal-mode runtimes that cannot run headless: src/lib.rs:14, src/app/runtime/terminal.rs:39; `MessageSender` async examples: src/harness/message_sender.rs:35). One `rust,ignore` in src/component/chart/grid/mod.rs:10 without an inline explanation — minor.
- docs/CHOOSING.md provides a component-selection guide; architecture diagram in README.

### Group 2: API Design

**4. Consistency & Symmetry — A-**
- Consistency-cleanup verified: `selected_item()` is now present and canonical across the list-like components (dropdown, menu, radio_group, searchable_list, select, selectable_list, tabs, tree, loading_list, accordion, command_palette, file_browser, tab_bar, multi_progress).
- Accessor symmetry: **0 gaps** (every `set_X` has a matching `X()`).
- Standard derives: Debug/Clone/Default/PartialEq on 73/73 State types.
- Builders: 71/74 components have `with_*` builders; data_grid, form, histogram have none — a mild asymmetry, though all three construct through other idioms.
- Remaining divergences: 17 undeprecated `selected()` aliases (Cadence D); `tab_bar::set_selected_index()` vs universal `set_selected()`; `checkbox::checked()` + `is_checked()`; `gauge::label_text()` vs `label()`; `restore_terminal`→`restore` rename at crate root (the tool's single REEXPORT_GAP).

**5. Modularity & Composability — A-**
- 14 feature flags with component-group granularity; `default-features = false` plus group selection works (dedicated "No Default Features" CI job proves it).
- Dependency leakage: 6 public signatures reference `ratatui::` types — `from_ratatui_cell` (src/backend/cell/mod.rs:69, deliberate interop), `render_widget<W: ratatui::widgets::Widget>` (src/component/context.rs:167, deliberate escape hatch), and four `status_bar` item methods taking `ratatui::style::{Color, Style}` (src/component/status_bar/item.rs:420-490) — these should take the crate-re-exported `envision::style` paths for cosmetic consistency, though the types are identical.
- tokio, tokio-stream, tokio-util, async-stream, futures-util are unconditional — you cannot opt out of the async runtime. This is an architectural stance (the Runtime is async-first), but it is the one real "pay for what you don't use" residue.

**6. Usability & Ergonomics — A**
- Prelude covers App/Command/Runtime, all components (`pub use crate::component::*`), input types, theme, layout/style — a typical app needs `use envision::prelude::*` plus concrete ratatui widgets only.
- The v0.17 `RuntimeBuilder::with_args` typestate (sealed `OptionalArgs`) converts a former runtime panic into a compile error — exactly the right direction.
- Edge cases covered by proptest suites (tests/property.rs, tests/property_extended.rs) and stress tests.

**7. Complexity Hiding — A**
- **Verified:** `AppHarness::message_sender()` returns `crate::harness::MessageSender<A::Message>` (src/harness/app_harness/mod.rs:272); `MessageSender<M>` (src/harness/message_sender.rs:42) wraps `tokio::sync::mpsc::Sender<M>` with passthrough methods plus `into_inner()` for the minority who need tokio semantics. First-party `MessageSendError<T>` (line 132) and `TrySendError<T>` (line 147). Finding #8 closed.
- Prelude re-exports first-party `layout::{Frame, Rect, Layout, ...}` and `style::{Color, Style, ...}` rather than `ratatui::prelude::*`; the remaining `ratatui::text::{Line, Span, Text}` and `{StatefulWidget, Widget}` re-exports are deliberate, labeled interop.
- Test gating: no non-module `#[cfg(test)]` items in src/harness/; `test-utils` feature exposes AppHarness async utilities to downstream crates.

**8. Type Safety & Errors — A**
- `EnvisionError` (src/error.rs:46) with structured `Io`, `Render{component, detail}`, `Config{field, reason}`, `Subscription{subscription_type, detail}`, `Other(BoxedError)` variants — matchable failure modes.
- trybuild compile-fail test (tests/trybuild_app_args/missing_with_args.rs) locks in the typestate guarantee.
- 0 unsafe blocks. `SortKey` enum replaces stringly-typed comparators in the v0.17 Table redesign.

### Group 3: Engineering Quality

**9. Algorithms & Data Structures — A-**
- compact_str is a first-class dependency with serde integration wired through the `serialization` feature.
- memory bench suite (benches/memory.rs) tracks allocations for 1000-item list/table/tree views and 10K state creation — allocation regressions are observable.
- No O(n)-in-render red flags surfaced; not exhaustively re-verified this pass.

**10. Smart Library Usage — A+**
- **0 unsafe blocks, 0 clippy suppressions** across 440 source files / 195,191 lines, with clippy clean under `--all-features`. This is genuinely best-in-class discipline.
- Every heavy dependency (serde, serde_json, tracing, arboard, pulldown-cmark, regex) is optional and feature-gated. 3-platform CI (ubuntu/macos/windows) × stable + 1.85.

**11. Performance & Benchmarking — A**
- 6 criterion suites (1,754 lines): backend, runtime, component view (100/1000 items × 2 terminal sizes), component events, memory allocs, 10K-row sort.
- Benchmarks run in CI with criterion report jobs (Bench Backend/Runtime/Component View/Component Events + report publishing).

### Group 4: Testing

**12. Unit Testing — A-**
- 7,306 lib tests; 4,316 component tests (4,113 unit + 203 insta snapshots).
- Distribution is uneven: box_plot 112, histogram 103, timeline 102 at the top; conversation_view, log_correlation, log_viewer, multi_progress at 14 each and tree/loading_list at 16 (the tool may undercount tests in `tests/` subdirectories — these components have dedicated tests/mod.rs dirs — but the asymmetry vs API surface warrants a look before v0.18).
- Snapshot coverage is concentrated: ~20 of 74 components have any snapshot tests.

**13. Integration & E2E — A**
- 11 integration files, 5,933 lines, 7,466 tests: multi-component (integration_new_components*.rs), async (integration_async.rs), stress (integration_stress.rs, 10K items), property-based (proptest, 2 files), serialization round-trips, trybuild compile-fail, `with_args` runtime wiring (integration_with_args.rs).
- AppHarness used in integration tests and demonstrated in examples/test_harness.rs.

**14. Doc Test Coverage — A+**
- 2,591 doc tests, 100.0% of 1,777 public fns across all 74 components. As both documentation and regression suite, this is the crate's standout asset.

### Group 5: Architecture

**15. Solving Pain Points — A**
- Headless testing (CaptureBackend, AppHarness, DualBackend) is the crate's founding thesis and it delivers: full input→update→render cycles without a TTY.
- 6 built-in themes; `Theme`/`Palette`/`NamedColor` public for custom themes. Widget annotations (`Annotate`, `AnnotationRegistry`) address accessibility/testability metadata.

**16. Code Organization — B+**
- Source discipline is excellent: 0 of 548 `.rs` files over 1000 lines (largest: src/component/text_area/tests.rs at 997 — five files sit in the 980-997 band and will need splitting soon).
- **CHANGELOG.md at 1,469 and MIGRATION.md at 1,265 lines violate the project's stated 1000-line cap.** PR #510 (open) splits them to 356/224 with `-legacy.md` archives. Graded against current main; merging #510 lifts this to A-.
- lib.rs re-exports 522 items into a flat namespace — large, but organized in labeled blocks, and module paths remain available.

**17. Extensibility — A**
- Custom components implement `Component` and get FocusManager/harness/annotation support for free; `Subscription` trait + combinators (Debounce/Filter/Throttle/Take/Mapped) are user-implementable; overlays generic via `Overlay` trait.
- CHANGELOG is disciplined with a structured `[Unreleased]` section; breaking changes tracked with spec cross-references.

### Group 6: Missing Pieces

**18. Feature Flags — A** — Component-group opt-outs, all heavy deps gated, documented in README with defaults; dedicated no-default-features CI job. Only gap: no tokio opt-out (see cat 5/25).

**19. Error Infrastructure — A-** — Structured `EnvisionError` + first-party harness errors. Only 33 `# Errors` sections against a large API (most of which is infallible, mitigating); no error-handling guide document.

**20. Logging & Debugging — A-** — `tracing` feature gates 30 call sites across app/command, app/runtime (incl. terminal), and component/table. Covers runtime dispatch and table diagnostics, but not a systematic event→message→update trace across all components. Debug derived on 73/73 states.

**21. Guides & Migration — A** — MIGRATION.md is complete: v0.16→v0.17 covers all six breaking changes (App::init args + builder split, Table sort/cell redesign, FileSortDirection removal, ResourceGauge builder, selected_item() unification, MessageSender), each with before/after tables, and history runs back through v0.14→v0.15→v0.16 to v0.5. CONTRIBUTING.md (152 lines), SECURITY.md (102), docs/CHOOSING.md all present. Missing: a performance-tuning guide. File length handled under cat 16, not double-counted here.

**22. Advanced Features — A** — Clipboard (arboard feature on TextArea), undo/redo + history (LineInput), search/filter (SearchableList, LogViewer, EventStream regex), animation (Spinner styles, Toast auto-dismiss), lifecycle hooks (since v0.6).

**23. Serialization — A** — `serialization` default feature: serde on component state, `load_state` at crate root, dedicated tests/serialization.rs (320 lines) with round-trip coverage.

**24. Security — A-** — SECURITY.md with policy; ANSI escape parsing isolated in src/component/terminal_output/ansi.rs (981 lines) with tests. No fuzz target for escape-sequence input specifically; proptest partially compensates.

**25. Ecosystem Integration — A-** — Idiomatic ratatui usage; DualBackend enables simultaneous real+capture rendering; works as a layer over the ratatui ecosystem (concrete widgets renderable via `RenderContext::render_widget`). Hard-wired to tokio — no async-std/smol path — and to the pinned ratatui major.

## Scorecard (automated)

```
Files over 1000 lines (.rs)                       0  PASS
Accessor symmetry gaps                            0  PASS
Doc test coverage                 100.0% (1777/1777) PASS
Debug/Clone/Default/PartialEq on State        73/73  PASS (x4)
Unsafe blocks                                     0  PASS
Clippy suppressions                               0  PASS
Result: 9/9 — cargo test/clippy/doc/examples/doc-tests all PASS
```

## Disposition of prior findings

| Finding (2026-07-04 report) | Status |
|---|---|
| #1 README compile-broken | CLOSED (verified: doc-tested via include_str, all doc tests pass) |
| #2 Stacked `[Unreleased]` | CLOSED (single structured Unreleased section) |
| #3 MIGRATION.md behind | CLOSED (v0.17 section complete, all 6 breaks documented) |
| #4 Dual sort systems | CLOSED (SortKey/Cell unification, ResourceTable removed) |
| #5 resource_gauge asymmetries | CLOSED (ResourceValues + values() getter) |
| #6 selected accessor divergence | CLOSED (canonical selected_item(); alias deprecation deferred to Cadence D) |
| #8 tokio Sender leakage | CLOSED (MessageSender<M> newtype, verified this audit) |
| CHANGELOG/MIGRATION > 1000 lines | OPEN — pending PR #510 |
| Redundant example imports | OPEN — pending PR #511 |

## New findings this audit (follow-up material)

| # | Finding | Severity | Suggested cadence |
|---|---|---|---|
| N1 | Duplicate `### Display Components` tables in README with conflicting descriptions | Moderate (front-page doc defect) | v0.17.0-blocker (trivial fix, high visibility) |
| N2 | README `version = "0.17"` snippet unresolvable until release | Cosmetic (self-heals at release; add release-checklist guard) | /release checklist |
| N3 | `tab_bar::set_selected_index` / `checkbox::is_checked` / `gauge::label_text` naming outliers | Cosmetic | v0.18+ (Cadence D) |
| N4 | `restore_terminal` renamed to `restore` at crate root | Cosmetic | v0.18+ |
| N5 | `AppShell` listed in README Utility component table but lives in `layout`, outside the 74-component count | Cosmetic | Ride with N1 fix |
| N6 | 5 source files in the 980-997 line band (text_area/tests.rs 997, loading_list/mod.rs 993, ...) — headroom nearly exhausted | Cosmetic (watch item) | v0.18+ |
| N7 | Snapshot-test coverage concentrated in ~20/74 components; 4 compound components report 14 tests each | Architectural (verify tool undercount vs real gap) | v0.18+ |
