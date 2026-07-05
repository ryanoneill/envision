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
4. Tracking-doc PR: mark findings resolved; record Fable audit round. Also lands the pre-cadence audit report at `docs/audits/2026-07-04-pre-release-hygiene.md` and the post-cadence audit report at `docs/audits/2026-07-XX-post-release-hygiene.md` so audit history is diffable from git rather than session-transcript-only.
5. Fable re-audit — **bounded scope, not open-ended:** verify the 5 named findings from the 2026-07-04 audit are closed AND grade recovers to ≥ `A` (≥ 3.85 GPA, target ≥ 4.0) AND scorecard is 9/9. If a NEW finding surfaces in the re-audit, it is logged as a follow-up cadence and does NOT block v0.17.0 unless its severity is release-blocker (data loss, panic on valid input, `-D warnings` regression, etc.). Prevents infinite-review loops.
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

**Chosen shape:** `#[cfg(doctest)]` gated struct — NOT a crate-root `#![doc = include_str!("../README.md")]`.

Rationale: the crate-root form would replace the current curated `//!` module docs at `src/lib.rs:1-141` (which teach the two-runtime-modes model and TEA architecture) with the README verbatim on docs.rs. That regresses the docs.rs landing page — a demanding consumer's second-five-minutes experience — for a compile-check win we can get via the gated form.

Add to `src/lib.rs` immediately above `pub mod prelude`:

```rust
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
pub struct ReadmeDoctests;
```

- The `#[cfg(doctest)]` gate keeps the struct out of both normal builds and the public API surface. Rustdoc runs the code blocks in the README as doctests.
- Verification: `cargo test --doc --all-features` picks up the README's code blocks; any breakage fails CI.

**Copy-paste fidelity — visible imports, not hidden setup.** README code blocks must be self-sufficient from the visible text alone. Real users copy the visible snippet, paste into a scratch `main.rs`, and expect it to compile. Hidden ` # use envision::prelude::*;` lines make the doctest pass but leave the visible snippet lying about what a real user sees.

- Imports (`use envision::prelude::*;`, `use envision::harness::AppHarness;`, etc.) appear in the visible block, not hidden.
- Only ` # fn main() { }` wrappers are legitimately noise and may be hidden.
- Any block that intentionally can't compile-and-run (e.g. requires a terminal) uses ` ```rust,no_run` with a `// no_run: reason` inline comment. `ignore` is banned unless justified inline.

Tradeoff: a few extra `use` lines per block (~2-4). Worth it — the whole point of the README is copy-paste onboarding.

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

Current state: three `## [Unreleased]` headers at lines 8, 362, 389 of CHANGELOG.md, each carrying a topic-scoped set of entries (`— Chrome ownership protocol (G2 + D2 + D11)`, `— Breaking: App::init takes args; RuntimeBuilder split`, `— Breaking: Table sort & cell API redesign`).

**Where the topic labels land:** the topic tags are load-bearing for scanning. Preserve them as `#### <topic>` sub-sub-sections under Keep-a-Changelog `### <kind>` subsections. Concretely:

**Before (three headers):**
```markdown
## [Unreleased] — Chrome ownership protocol (G2 + D2 + D11)
### Added
- `PaneLayout::view_with(...)` — closure-based renderer.
- `RenderContext::chrome_owned` flag.
### Changed
- `Table` skips its outer Block when embedded with `chrome_owned = true`.

## [Unreleased] — Breaking: `App::init` takes args; `RuntimeBuilder` split
### Breaking changes — `App::init` takes args
`App::init() -> (State, Command<Msg>)` is replaced with `App::init(args: Self::Args)`.
...

## [Unreleased] — Breaking: Table sort & cell API redesign
### Removed
- `TableMessage::SortBy`, `TableMessage::AddSort`, `TableMessage::ClearSort`.
...
```

**After (one header with topic-preserved sub-sub-sections):**
```markdown
## [Unreleased]

### Breaking Changes

#### `App::init` takes args; `RuntimeBuilder` split
`App::init() -> (State, Command<Msg>)` is replaced with `App::init(args: Self::Args)`.
...

#### Table sort & cell API redesign
- `TableMessage::SortBy`, `TableMessage::AddSort`, `TableMessage::ClearSort` removed.
...

#### `FileSortDirection` removed (this cadence, Unit 2)
See MIGRATION.md v0.16→v0.17.

#### `ResourceGaugeState::new` replaced by builder (this cadence, Unit 3)
See MIGRATION.md v0.16→v0.17.

### Added

#### Chrome ownership protocol (G2 + D2 + D11)
- `PaneLayout::view_with(...)` — closure-based renderer.
- `RenderContext::chrome_owned` flag.

#### D3 column clip warning
- `Column::new` canonical Length+Min docstring...

#### `resource_gauge` builder + values (this cadence, Unit 3)
- `ResourceGaugeState::default()` + `with_actual` + `with_request` + `with_limit` + `values() -> ResourceValues`.

### Changed

#### Chrome ownership protocol
- `Table` skips its outer Block when embedded with `chrome_owned = true`.
```

Topic labels survive as `####` under `###` Keep-a-Changelog kinds. Scannable, KaC-conforming, three-header regression eliminated.

**Additionally**, append a "Known incoherences deferred to future cadences" note to the `[Unreleased]` block:

```markdown
### Known Deferred Findings

The 2026-07-04 audit surfaced two API incoherences deliberately deferred beyond v0.17.0:

- `selected_value` / `selected_item` / `active_tab` accessor shape divergence across dropdown, select, heatmap, tab_bar, and data_grid. Requires a dedicated consistency-sweep cadence.
- Dependency leakage in 8 public signatures (`ratatui::layout::Position`, `ratatui::buffer::Cell`, `ratatui::style::Color/Style`, `ratatui::widgets::Widget`, `tokio::sync::mpsc::Sender`). Architectural discussion; not release-blocking.

Both are tracked as follow-up cadences and will be addressed in v0.18.0 or later.
```

Makes the deferral visible to consumers reading the release notes; prevents "why was v0.17.0 branded release-ready when these were still there" confusion in the next audit round.

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

- (A) fully-qualified use at every `file_browser` reference: `use crate::component::table::SortDirection;` — one canonical path.
- (B) local re-export in `file_browser/mod.rs`: `pub use crate::component::table::SortDirection;` — same type reachable under two paths.

**Chosen: (A).** The audit finding is *unification*. Option B perpetuates the "same type under two paths" pattern the finding calls out — a new consumer reading `use envision::component::file_browser::SortDirection;` in one module and `use envision::component::table::SortDirection;` in another has no compiler signal that they're the same type, just a cosmetic difference that reads like a bug. Option A costs the leadline migration one extra `sed` step (`file_browser::FileSortDirection` → `table::SortDirection`, not `file_browser::SortDirection`) but gives every future consumer a canonical import path. Migration note called out explicitly in MIGRATION.md.

### Derive-set delta (silent shape change if not handled)

`FileSortDirection` at `src/component/file_browser/types.rs:326-336` derives `Clone, Debug, PartialEq, Eq`.
`table::SortDirection` at `src/component/table/types.rs:434-445` derives `Clone, Copy, Debug, PartialEq, Eq, Default`.

The swap silently gains `Copy` and `Default`. Both are additive at the enum level, but they force downstream shape changes that MIGRATION.md must name explicitly, otherwise the migration produces idiomatic drift:

- **`sort_direction()` getter return type changes from `&SortDirection` to `SortDirection` (by value).**
  Current signature at `src/component/file_browser/mod.rs:533`: `pub fn sort_direction(&self) -> &FileSortDirection`. Returning a reference to a `Copy` type is unidiomatic — the compiler-preferred shape is by-value return. Change signature to `pub fn sort_direction(&self) -> SortDirection`. This is a **third** breaking change on the `file_browser` surface (in addition to the type swap and the constructor rename) and must appear in the MIGRATION.md table.
- **Toggle branches at `src/component/file_browser/mod.rs:927-928` collapse to `SortDirection::toggle()`.**
  Hand-rolled `match dir { Asc => Desc, Desc => Asc }` becomes `dir.toggle()`. The `toggle()` method already exists on `table::SortDirection`. Not a breaking change (internal); improves readability + removes redundant logic.
- **Field initializer at `src/component/file_browser/mod.rs:96` can drop to `Default::default()`.**
  Current: `sort_direction: FileSortDirection::Ascending`. If the surrounding `Default` impl uses field-level defaults, this line can drop entirely (Ascending is `SortDirection`'s `#[default]`). If the surrounding `Default` impl builds the struct field-by-field with explicit values (verify at impl time), leave it as `sort_direction: SortDirection::Ascending`. Judgment call at implementation.

### Breaking-change surface for MIGRATION.md

```markdown
### Sort direction unified

`file_browser::FileSortDirection` has been removed. `file_browser` now
uses `table::SortDirection` (identical 2-variant Ascending/Descending
shape, same `toggle()` method).

| Old | New |
|---|---|
| `use envision::component::file_browser::FileSortDirection;` | `use envision::component::table::SortDirection;` |
| `FileSortDirection::Ascending` | `SortDirection::Ascending` |
| `FileSortDirection::Descending` | `SortDirection::Descending` |
| `FileBrowserOutput::SortChanged(field, FileSortDirection::Ascending)` | `FileBrowserOutput::SortChanged(field, SortDirection::Ascending)` |
| `fn sort_direction(&self) -> &FileSortDirection` | `fn sort_direction(&self) -> SortDirection` (by value; `SortDirection: Copy`) |
| `let dir = *state.sort_direction();` | `let dir = state.sort_direction();` (no deref needed) |
| `match state.sort_direction() { FileSortDirection::Ascending => …, FileSortDirection::Descending => … }` | `match state.sort_direction() { SortDirection::Ascending => …, SortDirection::Descending => … }` |

Bonus: `SortDirection::toggle()` is available; use it to replace hand-rolled asc/desc flips.
```

## Unit 3 — resource_gauge closure

### Design

**Delete** `pub fn new(actual: f64, request: f64, limit: f64) -> Self` at `src/component/resource_gauge/mod.rs:139`.

**`Default` impl already exists** at `src/component/resource_gauge/mod.rs:110-124` (actual=0.0, request=0.0, limit=0.0, label=None, units=None, title=None, show_legend=true, orientation=default, disabled=false). No change needed to `Default`.

**Introduce a named-fields value type first, then the builder / setter / getter surface uses it:**

```rust
/// Named-fields carrier for the three resource-gauge values.
///
/// Named alternative to the previous positional `(f64, f64, f64)` triple —
/// eliminates the "transpose request/limit silently" hazard on both
/// construction and destructuring.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serialization", derive(serde::Serialize, serde::Deserialize))]
pub struct ResourceValues {
    /// Current in-use value.
    pub actual: f64,
    /// Requested value (e.g., K8s pod resource request).
    pub request: f64,
    /// Hard limit (e.g., K8s pod resource limit).
    pub limit: f64,
}
```

**Then five new items on `ResourceGaugeState`:**

```rust
impl ResourceGaugeState {
    /// Sets all three values from a named struct in a single call.
    ///
    /// The struct-literal form (`ResourceValues { actual, request, limit }`)
    /// names each field at construction, matching the intent of the removed
    /// positional `new(a, r, l)` without its transposition hazard.
    ///
    /// # Example
    /// ```rust
    /// use envision::component::{ResourceGaugeState, ResourceValues};
    ///
    /// let state = ResourceGaugeState::default().with_values(ResourceValues {
    ///     actual: 250.0,
    ///     request: 500.0,
    ///     limit: 1000.0,
    /// });
    /// assert_eq!(state.values().actual, 250.0);
    /// ```
    pub fn with_values(mut self, values: ResourceValues) -> Self {
        self.actual = values.actual;
        self.request = values.request;
        self.limit = values.limit;
        self
    }

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

    /// Returns all three values as a named struct.
    ///
    /// Complements [`set_values`](Self::set_values); closes the audit
    /// scorecard accessor-symmetry gap from v0.15.1 → v0.16.0.
    ///
    /// Returning `ResourceValues` (not a bare `(f64, f64, f64)` tuple)
    /// keeps destructuring safe — `let ResourceValues { actual, request, limit }`
    /// binds each field by name, so callers can't silently transpose
    /// `request` and `limit`.
    ///
    /// # Example
    /// ```rust
    /// use envision::component::{ResourceGaugeState, ResourceValues};
    ///
    /// let state = ResourceGaugeState::default()
    ///     .with_actual(250.0)
    ///     .with_request(500.0)
    ///     .with_limit(1000.0);
    /// let ResourceValues { actual, request, limit } = state.values();
    /// assert_eq!(actual, 250.0);
    /// assert_eq!(request, 500.0);
    /// assert_eq!(limit, 1000.0);
    /// ```
    pub fn values(&self) -> ResourceValues {
        ResourceValues {
            actual: self.actual,
            request: self.request,
            limit: self.limit,
        }
    }
}
```

**Construction ergonomics — two supported forms:**

- **Named-struct single call** (recommended when actual/request/limit are all known up front — test fixtures, K8s pod snapshots):
  ```rust
  ResourceGaugeState::default().with_values(ResourceValues { actual: 250.0, request: 500.0, limit: 1000.0 })
  ```
  Same shape as the removed `new(a, r, l)` but with named fields.
- **Fluent builder** (recommended when values are computed independently or conditionally):
  ```rust
  ResourceGaugeState::default().with_actual(a).with_request(r).with_limit(l)
  ```

Both are supported; consumers pick the form matching their construction pattern. Tests that migrate from `new(a, r, l)` can use whichever produces the tightest diff.

`values()` returning `ResourceValues` also unlocks safe re-use: `let vals = state.values(); other_state.with_values(vals)` copies a triple without positional-tuple risk.

### Migration surface (76 references across the resource_gauge module + tests)

- Every `ResourceGaugeState::new(a, r, l)` site migrates to either the named-struct form or the fluent-builder form — implementer picks per site to minimize diff noise (test fixtures generally cleaner with `with_values(ResourceValues { .. })`; sites building values from computed expressions cleaner with `with_actual().with_request().with_limit()`).
- `set_values(actual, request, limit)` at `mod.rs:503` stays as-is — legitimate multi-field mutator, not a constructor. `values()` (now returning `ResourceValues`) is its matching getter.
- All 76 call sites get mechanically migrated; no semantic change.
- `ResourceValues` is re-exported at the crate root alongside `ResourceGaugeState` (mirrors the `SortDirection`/`InitialSort`/etc. re-export pattern from `component::table`).

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
### `ResourceGaugeState::new` replaced by named-struct + builder

`ResourceGaugeState::new(actual, request, limit)` took three unlabeled
positional `f64` arguments; transposing `request` and `limit` was silent.
Replaced by two named forms — pick whichever fits your construction site:

**Named-struct single call** (recommended when all three values are known up front):

| Old | New |
|---|---|
| `ResourceGaugeState::new(250.0, 500.0, 1000.0)` | `ResourceGaugeState::default().with_values(ResourceValues { actual: 250.0, request: 500.0, limit: 1000.0 })` |

**Fluent builder** (recommended when values are computed independently):

| Old | New |
|---|---|
| `let a = compute_actual(); let r = ..; let l = ..; ResourceGaugeState::new(a, r, l)` | `ResourceGaugeState::default().with_actual(a).with_request(r).with_limit(l)` |

**Accessor symmetry** (this cadence also closes the `set_values`/no-getter gap):

| Old | New |
|---|---|
| `state.actual(); state.request(); state.limit()` (three separate calls) | Still supported. Plus new `state.values() -> ResourceValues` for all three at once. |
| `state.set_values(a, r, l)` (existing, unchanged) | Existing, unchanged. Getter counterpart is `state.values()`. |

**New type**: `envision::component::ResourceValues` (re-exported at the crate root).
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

- **README block that doesn't compile in isolation.** Per section 1b, imports (`use envision::prelude::*;` etc.) must appear in the visible block, not hidden. Only `# fn main() { }` wrappers may be hidden. Verified during Unit 1 implementation, not deferred to CI — every code block must produce a passing `cargo test --doc` invocation before the impl PR is opened.
- **`ResourceGaugeState::new` migration missing a call site.** 76 references — mechanical but volume raises drift risk. Verified via `grep -r 'ResourceGaugeState::new' src/ examples/ tests/` returning zero hits post-migration.
- **`sort_direction()` return-type break not immediately obvious.** Option A (chosen) forces every `use envision::component::file_browser::FileSortDirection` to migrate to `use envision::component::table::SortDirection`. The `sort_direction()` getter also changes shape (`&FileSortDirection` → `SortDirection` by value). Both are captured in the MIGRATION.md table, but a consumer skimming the CHANGELOG's Breaking header without reading the migration table could hit compile errors on the getter's `&`-deref. Mitigation: MIGRATION.md entry explicitly names both changes with a `let dir = *state.sort_direction()` → `let dir = state.sort_direction()` before/after row.
- **Fable re-audit doesn't return `A`.** If GPA lands `A-` again, examine the delta: if findings closed but new ones surfaced, address in a follow-up before release. If the same finding is still flagged, treat as spec-implementation gap and dispatch a fix subagent.
- **Release-hygiene cadence itself introducing a breaking change we didn't inventory.** The `include_str!` wiring is not breaking; the CHANGELOG restructure is not breaking; the MIGRATION.md backfill is not breaking. Only Units 2 and 3 add breaking changes, and both are explicitly documented in MIGRATION.md.

## Open questions

None. Design decisions resolved through two review rounds:

- **Brainstorm (2026-07-04):** file_browser unify vs alias vs document-split (chosen: unify); resource_gauge builder-only vs dual-entry (chosen: builder-only); CHANGELOG single `[Unreleased]` vs go-straight-to `[0.17.0]` (chosen: single `[Unreleased]` that `/release` renames).
- **Adversarial spec review (2026-07-04):** file_browser import path A vs B (upgraded to A — no re-export, canonical `table::SortDirection` at every use site); `values()` return type `(f64, f64, f64)` vs `ResourceValues` struct (upgraded to `ResourceValues` — same positional-f64 hazard as the constructor otherwise); derive-set delta on the `SortDirection` swap (Copy + Default added — `sort_direction()` return-type by-value, toggle branches collapse to `SortDirection::toggle()`, initializer cleanup at impl-time); README doctest wiring copy-paste fidelity (visible imports, no hidden `# use ...` lines); Fable re-audit scope bound (verify named findings closed; new findings logged but non-blocking unless severity-blocker); deferred findings visible in CHANGELOG.

## Reference

- **Fable audit report (2026-07-04):** `A-`, 3.62 GPA. Full report in prior session transcript; will be checked in at `docs/audits/2026-07-04-pre-release-hygiene.md` as part of the tracking-doc PR so audit history is diffable from git.
- **Adversarial spec review (2026-07-04):** three must-fix design bugs (tuple `values()` return, file_browser dual-path re-export, derive-set delta on SortDirection swap) and seven should-consider items folded into this spec via inline amendments. Original review preserved in session transcript.
- **Prior audit baseline (v0.15.1, 2026-04-18):** `A`, 4.02 GPA, 9/9 scorecard — the target this cadence recovers.
- **Established cadence pattern:** brainstorm → spec PR → plan PR → impl PR → tracking-doc PR (10 prior cadences during May 2026 leadline queue).
- **Verification gauntlet lessons carried from D3+D7+D8:**
  - `cargo test --no-default-features --no-run` (not just `cargo build --no-default-features`) to catch example-gating drift.
  - Feature-gate any impl block whose methods are only called under a feature (`impl ClipKind` lesson).
  - Doctest coverage on every new `pub fn` (G4+G5 + G6 audit-coverage regressions).
