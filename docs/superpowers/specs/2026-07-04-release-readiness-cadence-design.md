# Release-readiness cadence (v0.17.0 pre-release) — design

## Purpose

Close all 5 blocking findings from Fable's 2026-07-04 audit (`A-`, 3.62 GPA — regressed from v0.15.1's `A`, 4.02 GPA) as one coordinated cadence so v0.17.0 ships with the API-quality work of the 10 leadline cadences AND with restored release hygiene. The audit findings are the "what to build"; this design captures the "how" for each.

- **Audit report:** stored in this session; five findings summarized in Scope below.
- **Prior audit baseline:** v0.15.1 scored `A` (9/9 scorecard).
- **Current state:** ~11 weeks of unreleased work sits between the `v0.16.0` tag and current `main`; the CHANGELOG carries three stacked `[Unreleased]` sections; MIGRATION.md hasn't been updated since v0.14→v0.15.
- **Terminal state:** Fable re-audit confirms grade ≥ `A` and scorecard 9/9, then `/release minor` cuts v0.17.0.

## Scope

Five findings from the Fable audit, bundled into three code-shaped units:

| # | Finding | Unit |
|---|---|---|
| 1 | README code examples don't compile against current App API | Release hygiene |
| 2 | Three stacked `## [Unreleased]` sections in CHANGELOG.md | Release hygiene |
| 3 | MIGRATION.md 1 released version + 2 pending breakings behind | Release hygiene |
| 4 | Dual sort systems: `file_browser::FileSortDirection` duplicates `table::SortDirection` | file_browser sort |
| 5 | `resource_gauge` asymmetries: `set_values` has no getter (scorecard 8/9); `new(f64, f64, f64)` unlabeled positional; no example | resource_gauge closure |

Out of scope:
- Fable audit finding #6 (`selected_value` incoherence across dropdown/select/heatmap/tab_bar/data_grid) — real issue, needs its own consistency-sweep cadence.
- Fable audit finding #8 (8 dep-leakage sites in public signatures) — architectural discussion; not release-blocking.
- v0.17.0 feature work (K8s-driven components from the roadmap) — separate cadence after this one.

## Cadence structure

Same 4-PR pattern as the 10 leadline cadences, plus a re-audit gate before release:

1. Spec PR (this document)
2. Plan PR (task decomposition for the three code units)
3. Impl PR — three signed commits, one per unit:
   - `release-hygiene`: README + CHANGELOG + MIGRATION.md + `src/lib.rs` `include_str!` wire
   - `file-browser-sort`: `FileSortDirection` deleted, `table::SortDirection` re-used
   - `resource-gauge-closure`: `ResourceGaugeState::new` replaced by builder + `values()` getter + new example
   - Plus a CHANGELOG entry commit consolidating the three-unit release-notes into the existing `[Unreleased]` block.
4. Tracking-doc PR: mark findings resolved; record Fable audit round.
5. Fable re-audit: verify grade ≥ `A` (≥ 3.85 GPA), scorecard 9/9. Fix any regression before release.
6. `/release minor` → v0.17.0 to crates.io + GitHub Release.

## Unit 1 — Release hygiene

Files touched: `README.md`, `CHANGELOG.md`, `MIGRATION.md`, `src/lib.rs`.

### 1a. README examples fixed

Current broken snippets (README.md:71-91, README.md:103-109):

- The TEA counter example omits `type Args` on the `App` impl and uses `fn init()` without an args parameter. Current `App` trait at `src/app/model/mod.rs:191` requires `type Args` explicitly (no default per stable-Rust constraint) and `fn init(args: Self::Args)`.
- The "Testing with Runtime" snippet calls `Runtime::<MyApp>::virtual_terminal(80, 24)`, `runtime.dispatch(Msg::Increment)`, `runtime.render().unwrap()`, `runtime.contains_text("Count: 2")`. None of these exist on `Runtime` — the correct API is `Runtime::<MyApp, _>::virtual_builder(80, 24).build()` for the low-level path, or `AppHarness::virtual_builder(80, 24).build()` + `.dispatch(...)` + `.assert_contains(...)` for the docs-recommended path.

### Fix

- **TEA example gains** `type Args = ();` on the `impl App for MyApp` block; `fn init()` becomes `fn init(_args: ())`.
- **Testing example replaced** with the `AppHarness` idiom (matches the D7 "Choosing a Harness" decision table — `AppHarness` is the closure/App tester most consumers reach for). Example imports `envision::harness::AppHarness` and uses the trio: `AppHarness::virtual_builder(w, h).build()` + `.dispatch(msg)` + `.assert_contains("...")`.
- **The specific method surface** used in the snippet must match `src/harness/app_harness/mod.rs` and must compile end-to-end (verified via the `include_str!` doctest wiring below).

### 1b. README `include_str!` doctest wiring

Add to `src/lib.rs` immediately above `pub mod prelude`:

```rust
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
pub struct ReadmeDoctests;
```

- The `#[cfg(doctest)]` gate keeps the struct out of both normal builds and the public API surface. Rustdoc still runs the code blocks in the README as doctests.
- The README's ` ```rust` blocks pick up hidden-setup lines (` # use envision::prelude::*;` etc.) so each block is a compilable unit. Any block that should not be run as a test uses ` ```rust,no_run` or ` ```rust,ignore` with a one-line reason comment.
- Verification: `cargo test --doc --all-features` picks up the README's code blocks; any breakage fails CI.

### 1c. README Feature Flags section

Add a new section between "Installation" and "Quick Start" — matches Fable audit finding #7 ("Feature flags entirely undocumented in README despite the whole component library being feature-gated").

Content sourced from `Cargo.toml:15-47`:

| Flag | On by default | What it turns on |
|---|---|---|
| `full` | yes | All component groups (`input-components`, `data-components`, `display-components`, `navigation-components`, `overlay-components`, `compound-components`) |
| `input-components` | yes (via `full`) | Interactive input widgets |
| `data-components` | yes (via `full`) | Data-display widgets (Table, DataGrid, Tree, ResourceGauge, …) |
| `display-components` | yes (via `full`) | Text and chart widgets |
| `navigation-components` | yes (via `full`) | PaneLayout, Router, TabBar, KeyHints |
| `overlay-components` | yes (via `full`) | Overlay stack primitives |
| `compound-components` | yes (via `full`) | Higher-level compositions (Diagram, LogViewer, ConversationView) |
| `serialization` | yes | `serde::Serialize`/`Deserialize` on component state |
| `tracing` | no | Emits `tracing::warn!` diagnostics (see D3 clip warning) |
| `clipboard` | no | `arboard`-backed clipboard on `TextArea` |
| `markdown` | no | Markdown rendering in `StyledText` |
| `regex` | no | Regex search in `EventStream`, `LogViewer` |
| `test-utils` | no | `AppHarness` + `TestHarness` + `CaptureBackend` at crate boundary for downstream tests |

Followed by a minimal opt-out example:

```toml
envision = { version = "0.17", default-features = false, features = ["data-components", "display-components"] }
```

### 1d. Component-count fix

Three occurrences in README.md say `73 components` (lines 13, 157, 323). Current count per `docs/CHOOSING.md:3` is 74. Fix all three to 74. Add an inline `<!-- component-count: keep in sync with docs/CHOOSING.md -->` HTML comment above each occurrence so future drift is catchable via grep.

### 1e. CHANGELOG consolidation

Current state: three `## [Unreleased]` headers at lines 8, 362, 389 of CHANGELOG.md, each carrying a topic-scoped set of entries.

- Merge into a single `## [Unreleased]` block at the top of the file.
- Organize under Keep-a-Changelog convention subsections: `### Breaking Changes`, `### Added`, `### Changed`, `### Fixed`. All existing content preserved verbatim; only the outer headers restructured.
- The `[Unreleased]` label is intentional — `/release minor` will rename it to `[0.17.0]` + date at release time.

### 1f. MIGRATION.md backfill

Two new sections added at the top of the file:

- **`## v0.15.x to v0.16.0`** — covers the single breaking change v0.16 shipped (`DependencyGraph` removed; replaced by `Diagram`). Content lifted from CHANGELOG.md's existing `## [0.16.0]` entry. All the Diagram-only surface (`GraphNode`, `GraphEdge`, `GraphOrientation`, `NodeStatus`, `DependencyGraph*` types) is called out with a "before → after" table.
- **`## v0.16.x to v0.17.0`** — comprehensive migration for everything in the consolidated `[Unreleased]` block. Covers, at minimum:
  - `App::init` args (D1) — table already exists in CHANGELOG, lift verbatim.
  - Table sort/cell redesign (G1/G3/G7) — `TableMessage::SortBy/AddSort/ClearSort` removed; `Column::with_comparator`/`comparator`/`SortComparator` removed; `TableRow::cells(&self)` → `TableRow::cells(&self) -> Vec<Cell>` with `Cell{ text, style, sort_key }`; `TableState::sort_by(...)` new API. Migration table lifted from the CHANGELOG entry.
  - `FileSortDirection` → `SortDirection` (this cadence, Unit 2).
  - `ResourceGaugeState::new` positional → builder (this cadence, Unit 3).
  - Any other breaking-labeled entries the CHANGELOG surface names.

## Unit 2 — file_browser sort unification

### Design

- **Delete `pub enum FileSortDirection { Ascending, Descending }`** at `src/component/file_browser/types.rs:314-323`.
- **Re-use `crate::component::table::SortDirection`** across every `FileSortDirection` reference — same 2-variant Ascending/Descending shape, same `toggle()` method, same `PartialEq/Eq` derives.
- `FileSortField` (Name/Size/Modified/Extension) stays as-is — legitimately file-specific.

### Migration surface (24 references across 6 files)

- `src/component/file_browser/types.rs` — enum definition (delete) + import path in `FileBrowserOutput::SortChanged(FileSortField, FileSortDirection)` at line 60 → `SortChanged(FileSortField, SortDirection)`.
- `src/component/file_browser/mod.rs` — state field at line 75, initializer at line 96, `with_sort_direction` at line 269, `sort_direction()` getter at line 533, sort comparator branches at lines 646-647, toggle at lines 927-928. Docstring examples at lines 262-267 and 527-530 also reference `FileSortDirection`.
- `src/component/mod.rs:379` — module re-export line.
- `src/component/file_browser/tests.rs` — 8 references (lines 72, 95-96, 196, 530, 543, plus `test_with_sort_direction` at line 93 and `test_toggle_sort_direction` at line 536).
- `src/component/file_browser/helper_tests.rs:99` — string assertion; leave as-is (checks the debug-format string still contains `"sort_direction"`, which it will).

### Import path decision

Since `table::SortDirection` lives under `envision::component::table`, either:

- (A) fully-qualified use at every `file_browser` reference: `use crate::component::table::SortDirection;`
- (B) local re-export in `file_browser/mod.rs`: `pub use crate::component::table::SortDirection;` — then `envision::component::file_browser::SortDirection` still resolves.

**Chosen: (B).** Preserves the ergonomic name at the file_browser boundary so consumers migrating from `FileSortDirection` can search-replace `FileSortDirection` → `SortDirection` without changing their import paths. `envision::component::file_browser::SortDirection` and `envision::component::table::SortDirection` both resolve to the same type.

### Breaking-change surface for MIGRATION.md

```markdown
### Sort direction unified

`file_browser::FileSortDirection` has been removed. `file_browser` now
re-uses `table::SortDirection` (identical 2-variant Ascending/Descending
shape, same `toggle()` method).

| Old | New |
|---|---|
| `use envision::component::file_browser::FileSortDirection;` | `use envision::component::file_browser::SortDirection;` (or `table::SortDirection` — same type) |
| `FileSortDirection::Ascending` | `SortDirection::Ascending` |
| `FileSortDirection::Descending` | `SortDirection::Descending` |
| `FileBrowserOutput::SortChanged(field, FileSortDirection::Ascending)` | `FileBrowserOutput::SortChanged(field, SortDirection::Ascending)` |
```

## Unit 3 — resource_gauge closure

### Design

**Delete** `pub fn new(actual: f64, request: f64, limit: f64) -> Self` at `src/component/resource_gauge/mod.rs:139`.

**`Default` impl already exists** at `src/component/resource_gauge/mod.rs:110-124` (actual=0.0, request=0.0, limit=0.0, label=None, units=None, title=None, show_legend=true, orientation=default, disabled=false). No change needed to `Default`.

**Introduce** four new items:

```rust
impl ResourceGaugeState {
    /// Sets the actual (in-use) value.
    ///
    /// # Example
    /// ```rust
    /// use envision::component::ResourceGaugeState;
    ///
    /// let state = ResourceGaugeState::default().with_actual(250.0);
    /// assert_eq!(state.actual(), 250.0);
    /// ```
    pub fn with_actual(mut self, actual: f64) -> Self {
        self.actual = actual;
        self
    }

    /// Sets the requested value (e.g., K8s pod resource request).
    ///
    /// # Example
    /// ```rust
    /// use envision::component::ResourceGaugeState;
    ///
    /// let state = ResourceGaugeState::default().with_request(500.0);
    /// assert_eq!(state.request(), 500.0);
    /// ```
    pub fn with_request(mut self, request: f64) -> Self {
        self.request = request;
        self
    }

    /// Sets the hard limit (e.g., K8s pod resource limit).
    ///
    /// # Example
    /// ```rust
    /// use envision::component::ResourceGaugeState;
    ///
    /// let state = ResourceGaugeState::default().with_limit(1000.0);
    /// assert_eq!(state.limit(), 1000.0);
    /// ```
    pub fn with_limit(mut self, limit: f64) -> Self {
        self.limit = limit;
        self
    }

    /// Returns all three values as `(actual, request, limit)`.
    ///
    /// Complements [`set_values`](Self::set_values); closes the audit
    /// scorecard accessor-symmetry gap from v0.15.1 → v0.16.0.
    ///
    /// # Example
    /// ```rust
    /// use envision::component::ResourceGaugeState;
    ///
    /// let state = ResourceGaugeState::default()
    ///     .with_actual(250.0)
    ///     .with_request(500.0)
    ///     .with_limit(1000.0);
    /// assert_eq!(state.values(), (250.0, 500.0, 1000.0));
    /// ```
    pub fn values(&self) -> (f64, f64, f64) {
        (self.actual, self.request, self.limit)
    }
}
```

### Migration surface (76 references across the resource_gauge module + tests)

- Every `ResourceGaugeState::new(a, r, l)` site becomes `ResourceGaugeState::default().with_actual(a).with_request(r).with_limit(l)` (or the equivalent per test's intent).
- `set_values(actual, request, limit)` at `mod.rs:503` stays as-is — it's a legitimate multi-field mutator, not a constructor. `values()` is its matching getter.
- All 76 call sites get mechanically migrated; no semantic change.

### New example: `examples/resource_gauge.rs`

~150 lines demonstrating the component in a realistic K8s pod-quota scenario:

- A `PodQuota` struct with CPU (millicores) and memory (MB) fields.
- A small table of 3-5 pods showing each's `ResourceGaugeState` for CPU and memory usage.
- Uses `ResourceGaugeState::default().with_actual(x).with_request(y).with_limit(z)` for construction — demonstrates the new builder idiom.
- Selection interaction: arrow keys move between pods; PgUp/PgDn cycles selected pod's usage to demonstrate the visual bands (Good/Mild/Bad/Critical via `theme.severity_color` — closes the D6/D9 loop).
- Wired into `Cargo.toml` `[[example]]` with `required-features = ["data-components"]` (following the D8 lesson — Cargo.toml gating is required or `cargo test --no-default-features` breaks).
- Runs via `cargo run --example resource_gauge --features data-components`.
- Verified via `cargo build --example resource_gauge --all-features`.

### Breaking-change surface for MIGRATION.md

```markdown
### `ResourceGaugeState::new` replaced by builder

`ResourceGaugeState::new(actual, request, limit)` took three unlabeled
positional `f64` arguments; transposing `request` and `limit` was silent.
Replaced by a builder chain that names each field.

| Old | New |
|---|---|
| `ResourceGaugeState::new(250.0, 500.0, 1000.0)` | `ResourceGaugeState::default().with_actual(250.0).with_request(500.0).with_limit(1000.0)` |
| `state.set_values(a, r, l); state.actual(); state.request(); state.limit()` | Same, plus new `state.values() -> (f64, f64, f64)` |
```

## Testing strategy

Every existing test continues to pass; three new categories of coverage added:

- **README doctest coverage** — the `include_str!` wiring turns the README's ` ```rust` blocks into `cargo test --doc` targets. All must compile and pass.
- **New builder doc-tests** — each of `with_actual`, `with_request`, `with_limit`, `values()` gets a `# Example` doc-test (audit-coverage discipline from G4+G5).
- **New example** — `cargo build --example resource_gauge --all-features` in the verification gauntlet.

Full gauntlet unchanged from prior cadences:

- `cargo fmt --check`
- `cargo clippy --all-features -- -D warnings`
- `cargo nextest run --all-features`
- `cargo test --all-features --doc` (now includes README blocks)
- `cargo build --no-default-features`
- `cargo test --no-default-features --no-run` (D8 lesson — catches example-gating drift)
- `cargo build --examples --all-features`
- `cargo doc --no-deps --all-features` (zero warnings — the D3 intra-doc-link discipline)
- `./tools/audit/target/release/envision-audit all` — **9/9 scorecard**, resource_gauge accessor-symmetry gap closed.

## Success criteria

1. All 5 Fable audit findings closed with visible evidence:
   - README examples compile via `cargo test --doc`.
   - CHANGELOG has one `[Unreleased]` block.
   - MIGRATION.md has `v0.15→v0.16` and `v0.16→v0.17` sections with tables.
   - `FileSortDirection` produces `unresolved import` error (proof of deletion).
   - `ResourceGaugeState::new` produces `no function or associated item named 'new'` error (proof of deletion); `values()` accessor exists.
2. Fable re-audit returns grade ≥ `A` (≥ 3.85 GPA, ideally recovering the v0.15.1 baseline of 4.02) and scorecard 9/9.
3. v0.17.0 published to crates.io + tagged + GitHub Release created with release notes lifted from the finalized `[0.17.0]` CHANGELOG section.

## Risk register

- **README block that doesn't compile in isolation.** Some blocks need `# use ...` hidden setup or `# fn main() { }` wrapping. Verified during Unit 1 implementation, not deferred to CI.
- **`ResourceGaugeState::new` migration missing a call site.** 76 references — mechanical but volume raises drift risk. Verified via `grep -r 'ResourceGaugeState::new' src/ examples/ tests/` returning zero hits post-migration.
- **`FileSortDirection` re-export shape.** The `pub use crate::component::table::SortDirection;` at the `file_browser` boundary means the same type appears under two paths (`envision::component::file_browser::SortDirection` and `envision::component::table::SortDirection`). Documented in the migration table; not a new problem (same pattern as other cross-module re-exports).
- **Fable re-audit doesn't return `A`.** If GPA lands `A-` again, examine the delta: if findings closed but new ones surfaced, address in a follow-up before release. If the same finding is still flagged, treat as spec-implementation gap and dispatch a fix subagent.
- **Release-hygiene cadence itself introducing a breaking change we didn't inventory.** The `include_str!` wiring is not breaking; the CHANGELOG restructure is not breaking; the MIGRATION.md backfill is not breaking. Only Units 2 and 3 add breaking changes, and both are explicitly documented in MIGRATION.md.

## Open questions

None. The three design decisions Fable's audit surfaced (file_browser unify vs alias vs split; resource_gauge dual-entry vs breaking replace; CHANGELOG consolidation target) have all been resolved to their recommended options during brainstorming.

## Reference

- Fable audit report (2026-07-04): `A-`, 3.62 GPA. Full report in prior session transcript; key findings summarized in Scope above.
- Prior audit baseline (v0.15.1, 2026-04-18): `A`, 4.02 GPA, 9/9 scorecard.
- Established cadence pattern: brainstorm → spec PR → plan PR → impl PR → tracking-doc PR (10 prior cadences during May 2026 leadline queue).
- Verification gauntlet lessons carried from D3+D7+D8:
  - `cargo test --no-default-features --no-run` (not just `cargo build --no-default-features`) to catch example-gating drift.
  - Feature-gate any impl block whose methods are only called under a feature (`impl ClipKind` lesson).
  - Doctest coverage on every new `pub fn` (G4+G5 + G6 audit-coverage regressions).
