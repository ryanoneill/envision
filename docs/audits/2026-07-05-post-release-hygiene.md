# Envision post-release-hygiene audit — 2026-07-05

Bounded-scope re-audit of main at commit `21594c6` (release-readiness cadence, PR #504, merged) targeting the v0.17.0 release; verifies the 5 blocking findings from the [2026-07-04 pre-release audit](2026-07-04-pre-release-hygiene.md) are closed and that the grade recovers to A per the [release-readiness cadence spec](../superpowers/specs/2026-07-04-release-readiness-cadence-design.md).

```
==========================================================================
ENVISION LIBRARY AUDIT REPORT
Date: 2026-07-05
Version: 0.16.0 (Cargo.toml; v0.17.0 release target)
Commit: 21594c6
Test Results: 7299 unit / 2596 doc / 7459 integration (17,354 passed, 0 failed, 10 ignored)
==========================================================================

OVERALL GRADE: A (weighted GPA 3.91)

EXECUTIVE SUMMARY: The release-readiness cadence did exactly what it
promised: all 5 blocking findings from 2026-07-04 are verifiably closed,
the scorecard is back to 9/9, and the first-five-minutes surface (README
doctests, single-block CHANGELOG, complete MIGRATION.md) is now the
strongest it has ever been — the README is compile-tested via
`ReadmeDoctests`, so it structurally cannot rot again. The grade recovers
from A- (3.62) to A (3.91); the remaining distance to the 4.02 v0.15.1
baseline is entirely the two honestly-documented deferred findings
(selected-accessor shape divergence, dependency leakage in 8 signatures)
plus the long-standing hard tokio coupling — none of which is
release-blocking for v0.17.0.

GRADE BREAKDOWN:
--------------------------------------------------------------------------
Group 1: First Impressions (15%) — GPA 4.00
  1.  Getting Started .............. A   README compiles as doctests; Feature Flags table; visible imports
  2.  Examples ..................... A   91 examples, 100% component coverage, resource_gauge gap closed
  3.  Documentation ................ A   1780/1780 doc-tested pub fns; single [Unreleased]; honest deferrals

Group 2: API Design (25%) — GPA 3.76
  4.  Consistency & Symmetry ....... A-  0 accessor gaps, one SortDirection; deferred #6 divergence remains
  5.  Modularity & Composability ... A-  14 feature flags, per-group opt-out; tokio mandatory, 8 leaked sigs
  6.  Usability & Ergonomics ....... A-  Prelude + builders + instance methods; occasional turbofish
  7.  Complexity Hiding ............ A-  Harness feature-gated for downstream; ratatui visible at view() edge
  8.  Type Safety & Errors ......... A   Sealed OptionalArgs compile-time guard; named ResourceValues fields

Group 3: Engineering Quality (20%) — GPA 3.90
  9.  Algorithms & Data Structures . A-  Alloc-counting benches, layout caching; compact_str sporadic (3 files)
  10. Smart Library Usage .......... A   0 unsafe, 0 clippy suppressions, optional deps properly gated
  11. Performance & Benchmarking ... A   6 bench suites in CI with criterion reports, 1000-item params

Group 4: Testing (20%) — GPA 4.10
  12. Unit Testing ................. A   7,299 unit tests; 4,320 component tests incl. 203 snapshots
  13. Integration & E2E Testing .... A   AppHarness-driven, proptest x2, stress, async, trybuild compile-fail
  14. Doc Test Coverage ............ A+  100.0% (1780/1780) enforced by tooling; README wired into --doc

Group 5: Architecture (10%) — GPA 3.90
  15. Solving Pain Points .......... A   Headless CaptureBackend + AppHarness; 6 themes; annotations
  16. Code Organization ............ A-  0 files >1000 lines; 518 flat lib.rs re-exports
  17. Extensibility ................ A   Component/Subscription/Palette all user-implementable; MIGRATION.md

Group 6: Missing Pieces (10%) — GPA 3.78
  18. Feature Flags ................ A-  Granular component groups; no tokio/async opt-out
  19. Error Infrastructure ......... A-  EnvisionError{Io,Render,Config,Subscription,Other}; matchable
  20. Logging & Debugging .......... A-  tracing feature (31 sites); Command inspection; Debug everywhere
  21. Guides & Migration ........... A   MIGRATION.md complete through v0.17.0 with old->new tables
  22. Advanced Features ............ A-  Clipboard, text_area undo/redo, search/filter; no lifecycle hooks
  23. Serialization ................ A   serialization feature on state; load_state; dedicated test file
  24. Security ..................... A-  SECURITY.md (102 lines); ANSI parser handles escape sequences
  25. Ecosystem Integration ........ A-  Idiomatic ratatui, raw-widget escape hatch; tokio-only async
--------------------------------------------------------------------------

WEIGHTED GPA: 4.00(.15) + 3.76(.25) + 3.90(.20) + 4.10(.20) + 3.90(.10) + 3.78(.10) = 3.91 -> A

TRUST-ERODING FINDINGS (ranked by severity):
1. [DEFERRED #6, pre-existing] selected-accessor shape divergence
   (dropdown::selected_value() -> &str vs heatmap::selected_value() -> f64;
   tab_bar::active_tab(); data_grid's four selection accessors) — affects
   categories 4, 6. Documented in CHANGELOG "Known Deferred Findings".
2. [DEFERRED #8, pre-existing] Dependency leakage in 8 public signatures
   (ratatui Position/Cell/Color/Style/Widget, tokio mpsc::Sender) —
   affects categories 5, 7. Documented in CHANGELOG.
3. [NEW, follow-up] tooling flags `restore_terminal` as a lib.rs re-export
   gap, but it IS re-exported as `restore` (src/lib.rs: `pub use
   crate::app::restore_terminal as restore;`) — alias defeats the check.
   Affects category 4 (tooling fidelity), cosmetic.
4. [NEW, follow-up] compact_str used in only 3 src files — sporadic rather
   than systematic small-string strategy. Affects category 9.
5. [NEW, follow-up] 9 internal tests/mod.rs files missing module-level
   `//!` docs; ~10 `ignore` doc blocks on private chart/harness helpers
   without per-block justification. Affects category 3, cosmetic.

TOP 5 IMPROVEMENTS (highest ROI):
1. Run the deferred consistency-sweep cadence for finding #6 (selected
   accessor shapes) — category 4 from A- to A/A+.
2. Decide the dependency-leakage policy for finding #8 (newtype vs
   documented re-export) — categories 5, 7 from A- toward A.
3. Teach tools/audit re-export-gap detection about `as` aliases — restores
   trust in the scorecard's re-export line.
4. Either adopt compact_str systematically in string-heavy state types or
   remove it in favor of String — category 9 from A- to A.
5. Evaluate a `tokio`-optional core (sync-only runtime behind a feature)
   for v0.18+ — categories 5, 18, 25 all gain headroom.
```

## Prior-finding verification (bounded-scope gate)

All 5 blocking findings from the 2026-07-04 audit are **CLOSED**, with evidence:

### Finding 1 — README code examples don't compile against current App API: CLOSED

- `src/lib.rs:427-430` — `#[cfg(all(doctest, feature = "full"))] #[doc = include_str!("../README.md")] pub struct ReadmeDoctests;` with a doc comment explaining the `full`-feature gate. README code blocks now compile under the default `cargo test --doc` invocation; `cargo test --doc --all-features` passes (2,596 tests, 0 failed).
- `README.md:103-105` — TEA snippet declares `type Args = ();` and `fn init(_args: ()) -> (State, Command<Msg>)`, matching the current `App` trait.
- `README.md:146-173` — testing snippet uses the correct idiom: `AppHarness::<MyApp>::new(80, 24).unwrap()`, `harness.dispatch(...)`, `harness.render().unwrap()`, `harness.assert_contains("Count: 2")`. The phantom `Runtime::virtual_terminal / runtime.contains_text` API from the old README is gone.
- No hidden `# use` lines in README code blocks (imports are visible); no ` ```ignore `/` ```no_run ` blocks in README.
- `README.md:23-51` — new Feature Flags section with a 13-row flag table and a `default-features = false` opt-out example.

### Finding 2 — Three stacked `## [Unreleased]` sections in CHANGELOG.md: CLOSED

- `grep -c "^## \[Unreleased\]" CHANGELOG.md` → **1**. Single `[Unreleased]` block sits directly above `## [0.16.0] - 2026-04-20` (CHANGELOG.md:166).
- Topic-preserved `####` sub-sub-sections under Keep-a-Changelog `###` kind headers (e.g. `#### examples/router.rs refresh (D8)` at CHANGELOG.md:150).
- `CHANGELOG.md:154` — `### Known Deferred Findings` names audit findings #6 (`selected_value`/`selected_item`/`active_tab` divergence across dropdown/select/heatmap/tab_bar/data_grid) and #8 (dependency leakage in 8 public signatures) as explicit v0.18+ follow-up cadences. Deferrals are honest and specific.

### Finding 3 — MIGRATION.md behind by 1 released version + 2 pending breakings: CLOSED

- `MIGRATION.md:3` — `## v0.16.x to v0.17.0` covering: `App::init(args)` + `RuntimeBuilder::with_args` (old→new table incl. `AppHarness::with_args`), the Table sort/cell redesign (13-row mapping table from `SortBy`/`AddSort`/`ClearSort`/`with_comparator` to `SortToggle`/`Cell`/`SortKey`, plus `ResourceTable` → `Table` + `RowStatus`), `FileSortDirection` removal (7-row table incl. the getter shape change `fn sort_direction(&self) -> &FileSortDirection` → `-> SortDirection` by value, and the `.toggle()` bonus), and the `ResourceGaugeState::new` replacement (named-struct and fluent-builder forms, accessor symmetry note).
- `MIGRATION.md:86` — backfilled `## v0.15.x to v0.16.0` (DependencyGraph → Diagram).
- Chain is now continuous: v0.17 → v0.16 → v0.15 → v0.14 → … → v0.5.

### Finding 4 — Dual sort systems (`FileSortDirection` vs `table::SortDirection`): CLOSED

- `grep -rn FileSortDirection src/ examples/ tests/` → **zero matches**. The type is deleted.
- `src/component/file_browser/types.rs:3` — `use crate::component::table::SortDirection;` (canonical path, Option A: no local re-export); `types.rs:62` — `SortChanged(FileSortField, SortDirection)`.
- `src/component/file_browser/mod.rs:536` — `pub fn sort_direction(&self) -> SortDirection` (by value; `SortDirection: Copy`).
- `src/component/file_browser/mod.rs:926` — `state.sort_direction = state.sort_direction.toggle();` (hand-rolled flip collapsed to `.toggle()`).

### Finding 5 — `resource_gauge` asymmetries: CLOSED

- `grep -rn "ResourceGaugeState::new\b" src/ examples/` → **zero matches**. The positional 3×`f64` constructor is deleted.
- `src/component/resource_gauge/values.rs:31` — `pub struct ResourceValues { actual, request, limit }` in a sibling file, re-exported at `mod.rs:48` and at the crate root / prelude (confirmed in lib.rs re-export block).
- `src/component/resource_gauge/mod.rs:151` — `pub fn with_values(mut self, values: ResourceValues) -> Self`; fluent `with_actual`/`with_request`/`with_limit` builders confirmed by the audit tool's builder listing (10 `with_*` methods on resource_gauge).
- `src/component/resource_gauge/mod.rs:567` — `pub fn values(&self) -> ResourceValues` getter pairing the pre-existing `set_values` mutator; its doc test destructures the returned struct and asserts all three fields.
- Scorecard accessor-symmetry line: **0 gaps** ("All setters have matching getters") — 8/9 → 9/9 regression closed.
- `examples/resource_gauge.rs` — new interactive example with a realistic K8s pod-quota shape (CPU millicores + memory MB per pod vs request/limit), exercising `with_values(ResourceValues { .. })` and `values()`.

## Scorecard

`./tools/audit/target/release/envision-audit all` at `21594c6`:

| Check | Result | Target | Status |
|---|---|---|---|
| Files over 1000 lines | 0 | 0 | PASS |
| Accessor symmetry gaps | 0 | 0 | PASS |
| Doc test coverage | 100.0% (1780/1780) | 100% | PASS |
| Debug on State types | 73/73 | 73/73 | PASS |
| Clone on State types | 73/73 | 73/73 | PASS |
| Default on State types | 73/73 | 73/73 | PASS |
| PartialEq on State types | 73/73 | 73/73 | PASS |
| Unsafe blocks | 0 | 0 | PASS |
| Clippy suppressions | 0 | 0 | PASS |

**Result: 9/9 checks passing.** Cargo checks: `test --all-features` PASS (10,055), `clippy --all-features` PASS (0 warnings), `doc --no-deps` PASS, `build --examples` PASS, `test --doc` PASS (2,596).

## Detailed findings

### Group 1: First Impressions (15%) — GPA 4.00

#### 1. Getting Started — A

The library's purpose is clear in the first sentence of README.md ("A ratatui framework for collaborative TUI development with headless testing support") followed by an 8-bullet feature list. `cargo add envision` is the documented install path (README.md:56). The TEA quick-start (README.md:~95-122) and AppHarness testing snippet (README.md:146-173) are both **compiled as doctests** via `ReadmeDoctests` (src/lib.rs:427-430) — the strongest possible guarantee against the rot that caused last audit's failure. Imports in the snippets are visible (no hidden `# use`). MSRV 1.85 is declared and tested in CI (stable + 1.85 across ubuntu/macos/windows). The prelude covers App/Runtime/Command/components/layout/style in one import. The new Feature Flags section (README.md:23-51) tells a minimal-footprint consumer exactly how to opt down before they ask. Withheld A+: the version in the opt-out example says `"0.17"` while Cargo.toml still says 0.16.0 pre-release — correct after the tag lands, momentarily forward-dated.

#### 2. Examples — A

91 examples, 17,852 lines, **100% component coverage (73/73)** per the audit tool. Progressive complexity holds: single-component demos (`button.rs`, `gauge.rs`) → mid-size compositions (`focus_manager.rs`, `drilldown.rs`) → real applications (`file_manager.rs` 533 lines, `log_explorer.rs` 515, `chat_client.rs` 486, `production_app.rs` 370, `component_showcase.rs` 891). `cargo build --examples --all-features` passes. The one gap from the prior audit — ResourceGauge with zero examples — is closed by `examples/resource_gauge.rs` (K8s pod-quota shape with navigation). Testing patterns are demonstrated (`test_harness.rs`, `capture_backend.rs`); focus, routing, and multi-pane composition all covered.

#### 3. Documentation Quality — A

`#![warn(missing_docs)]` enforced; type-level docs 489/489 (100%); module docs 115/124 (93% — the 9 missing are all internal `tests/mod.rs` files, not public surface). Doc test coverage is 100.0% of public component fns (1780/1780), tool-enforced. `# Panics` (15) and `# Errors` (33) sections present where relevant. CHANGELOG is now a single coherent `[Unreleased]` block over shipped 0.16.0, with the `### Known Deferred Findings` block (CHANGELOG.md:154) naming exactly what is deferred and why — a demanding consumer reads that as maturity, not weakness. Conceptual docs beyond API reference: docs/CHOOSING.md (component selection), diagram performance guide in module docs, README architecture section. Withheld A+: ~10 ` ```ignore ` blocks on private chart/harness helpers (e.g. src/component/chart/scale.rs:112, src/harness/snapshot/mod.rs:78) lack a one-line "why ignored" note, and the 9 test-module `//!` gaps remain.

### Group 2: API Design (25%) — GPA 3.76

#### 4. Consistency & Symmetry — A-

The category that blocked the prior release is materially recovered: **zero accessor-symmetry gaps** (scorecard), standard derives at 73/73 for all four traits, one canonical `SortDirection` (file_browser's duplicate deleted, `.toggle()` reused at mod.rs:926), and `ResourceValues` closing the last positional-constructor trap. Naming-pattern catalogs from the tool show strong regularity on `title`/`placeholder`/`label` triplets (`with_X`/`X()`/`set_X()` nearly everywhere). Held at A- (not A) by the honestly-deferred finding #6: `dropdown::selected_value() -> &str` vs `heatmap::selected_value() -> f64` is a genuine type incoherence for the same method name; `tab_bar` uses `active_tab()` where siblings use `selected_item()`; `data_grid` carries four selection accessors. Deliberate, documented, and scheduled — but still present in the surface a consumer sees today. Minor: the audit tool flags `restore_terminal` as a re-export gap, but src/lib.rs re-exports it as `restore` — a tooling alias-blindness issue, not an API gap (new follow-up).

#### 5. Modularity & Composability — A-

14 feature flags with genuine per-group opt-out (`input-components`, `data-components`, `display-components`, `navigation-components`, `overlay-components`, `compound-components`, plus `serialization`/`tracing`/`clipboard`/`markdown`/`regex`/`test-utils`); the No Default Features CI job proves the minimal build stays green. Custom components are first-class (`Component` trait + FocusManager integration); parent-child messaging is type-safe via typed `Output` enums per component; the overlay system is generic (`Overlay`/`OverlayAction`/`OverlayStack`). Held at A-: dependency leakage in 8 public signatures (deferred #8 — e.g. `tokio::sync::mpsc::Sender` at src/harness/app_harness/mod.rs:264, `ratatui::style::Color` at src/component/status_bar/item.rs:420), and tokio + 4 async companion crates are unconditional dependencies — a sync-only consumer cannot shed the async runtime.

#### 6. Usability & Ergonomics — A-

One `use envision::prelude::*` covers typical apps. Instance methods on state (`state.update(msg)`), fluent builders on 71/74 components, `Event::key/char/ctrl` test helpers, and convenience constructors (`Cell::number`, `Cell::datetime`) serve the 80% case. Edge cases are handled and property-tested (empty collections, boundary indices — tests/property.rs, property_extended.rs). The `ResourceValues` named-struct form eliminates the last silent-transposition footgun. Held at A-: `AppHarness::<MyApp>::new(80, 24)` needs a turbofish (shown in README, so at least discoverable), and the finding-#6 accessor divergence has ergonomic cost (users must re-learn selection accessors per component).

#### 7. Complexity Hiding — A-

Terminal setup/teardown is automatic (`TerminalRuntime`; `restore` re-export for panic recovery). ListState-style ratatui internals are managed inside component state. Testing requires near-zero boilerplate (3 lines to a rendered assertion, per README:167-173), and the harness is available to downstream crates via the `test-utils` feature — the tool confirms **no `#[cfg(test)]`-gated non-module items in src/harness/**. `Frame`/`Rect`/`Color`/`Style` are re-exported through `crate::layout`/`crate::style` so users import from envision paths. Held at A-: `view()` still takes ratatui's `Frame`/`Rect` shapes and the prelude re-exports `Line`/`Span`/`Text`/`Widget`/`StatefulWidget` — a pragmatic, documented leak rather than a sealed abstraction.

#### 8. Type Safety & Error Handling — A

The standout design win this cycle: forgetting `with_args` for a non-`()` `Args` is a **compile error** via the sealed `OptionalArgs` marker (verified by trybuild: tests/trybuild_app_args/missing_with_args.rs), replacing a runtime panic. Typed per-component `Message`/`Output` enums make cross-wiring a type error. `EnvisionError` gives matchable failure modes. Zero unsafe. Indices are clamped (property tests assert no panics on out-of-range operations). `VirtualRuntime<A>` aliases hide backend generics. `ResourceValues` makes the transposed-arguments state unrepresentable-by-accident. 10 ignored doc tests are the known internal-helper blocks, not API surface.

### Group 3: Engineering Quality (20%) — GPA 3.90

#### 9. Algorithms & Data Structures — A-

Evidence of real performance discipline: benches/memory.rs counts allocations for 1000-item `view()` calls on list/table/tree; Diagram caches layout and batch-writes edge buffers (CHANGELOG: 100 nodes ~250µs); sort_bench covers 10k-row table sorts. Scroll infrastructure is centralized in `ScrollState` (src/scroll/mod.rs) rather than reimplemented per component. Held at A-: `CompactString` appears in only 3 src files — as a strategy it is sporadic, which is the exact trust-eroder the rubric names; either commit or remove (new follow-up).

#### 10. Smart Library Usage — A

**0 unsafe blocks, 0 clippy suppressions** across 195K lines of src — rare at this scale. Optional deps genuinely optional (serde/serde_json/tracing/arboard/pulldown-cmark/regex all `dep:`-gated). tokio features scoped (`sync, rt, rt-multi-thread, macros, time, fs`) rather than `full` in the library (dev-deps use `full`, which is fine). Three platforms × two rustc versions in CI. `cargo clippy --all-features` PASS with 0 warnings.

#### 11. Performance & Benchmarking — A

Six benchmark suites (1,754 lines) covering the paths users care about: backend draw/diff/snapshot at 4 terminal sizes, runtime creation/dispatch/tick, component `view()` at 100/1000 items × 2 sizes (list/table/tree/diagram), `handle_event`/`dispatch_event` at 100/1000 items, allocation counting, and 10k-row sort. Benchmarks run in CI with criterion report artifacts (Bench Backend/Runtime/Component View/Component Events jobs). Stress tests separately push 10K+ items (tests/integration_stress.rs). Withheld A+: no automated regression *gating* (reports are produced, but a slowdown doesn't fail CI).

### Group 4: Testing (20%) — GPA 4.10

#### 12. Unit Test Coverage — A

7,299 lib tests; 4,320 per-component tests including 203 insta snapshots. Distribution is healthy — box_plot (112), histogram (103), timeline (102) at the top; even the thinnest (conversation_view, log_correlation, multi_progress at 14 each) pair unit + snapshot coverage. Focused/unfocused/disabled rendering covered via ViewContext-driven snapshot tests. Negative tests present (invalid indices, empty collections). The new work is tested: file_browser sort tests updated to canonical `SortDirection`, resource_gauge at 49 tests including `values()` round-trips.

#### 13. Integration & E2E Testing — A

11 integration files, 5,933 lines, 7,459 tests in the `--tests` phase. AppHarness drives real input→update→render cycles in tests/integration.rs, integration_async.rs, and integration_with_args.rs. Property-based testing in two files (proptest); stress testing (integration_stress.rs, 521 lines, 10K+ items); async subscription/command flows (integration_async.rs, 651 lines); compile-fail contract tests via trybuild (missing `with_args` = compile error). Serialization round-trips get a dedicated file. Withheld A+: no fuzz harness (cargo-fuzz) for the event/ANSI parsers — proptest partially covers this.

#### 14. Doc Test Coverage — A+

Best-in-class, cited as an example: **100.0% of public component fns (1780/1780) carry doc tests**, enforced as a failing scorecard check rather than aspiration. 2,596 doc tests pass. Sampled quality is real — `resource_gauge::values()`'s doc test (mod.rs:559-566) builds via three fluent builders, destructures the returned `ResourceValues`, and asserts all three fields; not `assert!(true)` filler. The README itself is now part of the doc-test corpus. The 10 ignored tests are internal-helper illustrations, 0.4% of the corpus.

### Group 5: Architecture (10%) — GPA 3.90

#### 15. Solving Customer Pain Points — A

Pain #1 (testing without a terminal): CaptureBackend + AppHarness + Snapshot/Assertion is the library's founding feature and it is deep (frame snapshots, text search, ANSI/JSON output). Pain #2 (state management): TEA with typed messages, subscriptions, and Command inspection. Pain #3 (component reuse): 74 components with uniform trait surface. Accessibility: widget annotation system (`Annotate`, `AnnotationRegistry`, `WidgetAnnotation`) for semantic metadata. Theming: 6 built-in themes + `Palette` for custom ones. Resize handled by the runtime. Customer-feedback loop is visibly institutionalized (docs/customer-feedback/, May 2026 leadline closure).

#### 16. Code Organization — A-

**Zero files over 1000 lines** in a 220K-line project — the 1000-line rule is enforced by the scorecard, with the largest file at 997. Module hierarchy is uniform (component/<name>/{mod,state,tests}). CI covers tests (3 OS × 2 rustc), no-default-features, clippy, fmt, coverage, docs, benches, and docs deploy. Held at A-: lib.rs re-exports 518 items into a flat root namespace — discoverable but noisy, and the rubric explicitly flags 100+ flat re-exports; several files sit within 3% of the 1000-line ceiling (text_area/tests.rs at 997) and will need proactive splits.

#### 17. Extensibility & Future-Proofing — A

Users can implement `Component` (first-class with FocusManager/ViewContext), custom `Subscription`s (the trait plus combinators like Debounce/Throttle/Take are public), and custom themes via `Palette`. Semver discipline is demonstrated, not claimed: every breaking change lands in CHANGELOG `### Breaking` *and* a MIGRATION.md table with old→new mappings. No sealed traits blocking extension (the one sealed marker, `OptionalArgs`, is sealed deliberately for compile-time safety and documented as such).

### Group 6: Missing Pieces (10%) — GPA 3.78

#### 18. Feature Flags — A-

Six component-group flags plus `serialization`, `tracing`, `clipboard`, `markdown`, `regex`, `test-utils`; README documents each with a defaults column; CI builds the no-default-features configuration. Serde is fully optional. Held at A-: async/tokio cannot be opted out — the one monolithic remnant in an otherwise granular crate.

#### 19. Error Handling Infrastructure — A-

`EnvisionError` with Io/Render/Config/Subscription/Other(BoxedError) variants — distinguishable, matchable failure modes with a crate-level `Result` alias. Held at A-: no dedicated error-handling guide showing recommended patterns (retry vs surface vs degrade) for runtime and subscription errors.

#### 20. Logging & Debugging — A-

`tracing` feature gates 31 diagnostic call sites (e.g. table column-clip warnings); `Debug` on 73/73 state types; `Command` exposes inspection methods so tests and debuggers can see what an update returned. Held at A-: no end-to-end event→message→update trace spans; a `tracing`-instrumented runtime dispatch would close the "why didn't my keypress do anything" debugging gap.

#### 21. Guides & Migration — A

Fully recovered from the prior audit's blocking gap. MIGRATION.md (1,225 lines) now runs unbroken from v0.4 to v0.17.0, and the new v0.16→v0.17 section is exemplary: old→new tables for every breaking change, including subtle shape changes (`sort_direction()` by-ref → by-value) and "bonus" idiom upgrades (`.toggle()`). CONTRIBUTING.md (152 lines) and docs/CHOOSING.md exist. Withheld A+: no standalone performance-tuning or error-handling guide (perf guidance lives only in Diagram module docs).

#### 22. Advanced Features — A-

Clipboard (`clipboard` feature, arboard-backed on TextArea); undo/redo in text_area (dedicated undo_tests.rs); search/filter across searchable_list, command_palette, log_viewer, event_stream (regex-capable), Diagram node search; animation via spinner styles, progress/multi-progress, toast timing. Held at A-: no component lifecycle hooks (mount/unmount) — overlay/visibility patterns substitute but aren't equivalent for resource setup/teardown.

#### 23. Serialization & Persistence — A

The `serialization` feature applies Serialize/Deserialize to component state (51 gated sites), `app::load_state` supports session restore, and tests/serialization.rs (320 lines) proves round-trips. The serde dependency demonstrably earns its place — the rubric's trust-eroder (serde present but unused for state) is the opposite of reality here.

#### 24. Security — A-

SECURITY.md (102 lines) with a disclosure policy; the terminal_output ANSI parser (src/component/terminal_output/ansi.rs, 981 lines) explicitly parses/normalizes escape sequences rather than passing raw bytes to the terminal; zero unsafe removes the memory-safety attack surface. Held at A-: no documented threat-model statement about rendering untrusted input (e.g. log_viewer fed hostile logs), which is exactly where TUI escape-injection bites.

#### 25. Ecosystem Integration — A-

Idiomatic ratatui: components render real widgets, and `Widget`/`StatefulWidget` + `Line`/`Span`/`Text` re-exports let users drop raw ratatui into an envision app (the escape hatch is deliberate and useful). CaptureBackend and DualBackend show the backend layer is abstracted, not hard-wired to crossterm alone. Held at A-: async is tokio-only (no async-std/smol path), and the ratatui version pin means major ratatui bumps are envision-release-gated — acceptable, but worth a documented policy.

## New findings this re-audit (follow-up cadence material — none release-blocking)

Per the bounded scope, these are logged for follow-up cadences and do not block v0.17.0:

| # | Finding | Severity | Categories |
|---|---|---|---|
| N1 | Audit tool's re-export-gap check is alias-blind: flags `restore_terminal` although lib.rs re-exports it as `restore` | Cosmetic (tooling) | 4 |
| N2 | `CompactString` used in only 3 src files — sporadic small-string strategy | Minor (perf hygiene) | 9 |
| N3 | 9 internal `tests/mod.rs` files missing `//!` module docs | Cosmetic | 3 |
| N4 | ~10 ` ```ignore ` doc blocks on private chart/harness helpers lack per-block justification | Cosmetic | 3, 8 |
| N5 | lib.rs flat re-export count now 518 and growing with each component | Minor (organization) | 16 |
| N6 | No CI benchmark regression *gating* (reports only) | Minor | 11 |
| N7 | Several files within 3% of the 1000-line ceiling (text_area/tests.rs 997, loading_list/mod.rs 993) | Cosmetic (preventive) | 16 |

Pre-existing deferred findings #6 (selected-accessor divergence) and #8 (dependency leakage, 8 signatures) remain open by design, documented in CHANGELOG's Known Deferred Findings, scheduled for v0.18+ cadences.

## Verdict

- **Overall grade: A (weighted GPA 3.91)** — recovers from A- (3.62) at `5560125`; meets the spec's ≥ 3.85 recovery threshold (stretch target 4.0 not reached solely due to the two documented deferrals and the tokio coupling).
- **All 5 blocking findings: CLOSED** (evidence above).
- **Scorecard: 9/9 passing**; all cargo checks green (17,354 tests, 0 failures, 0 warnings).
- **Release gate: PASS.** The release-readiness cadence may proceed to the v0.17.0 release.
