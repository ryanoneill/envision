# Table sort & cell unification — design spec

**Date:** 2026-05-02
**Author:** envision Claude (with detailed review from leadline Claude)
**Status:** Approved design; implementation pending
**Scope:** Single coherent breaking-change pass on `Table` and the sort family. Merges `ResourceTable` into `Table`, replaces the string-comparator sort path with a typed-key path, rewrites the sort message vocabulary, adds declarative initial-sort and per-column default-sort builders, unifies `Cell` across all table-like components.

## Context

Three independent customer-feedback gaps surfaced building the leadline TUI converged onto the same surface — the `Table` component's row trait, sort path, and cell representation:

- **G1** Typed `TableRow::sort_key`. Today `TableRow::cells() -> Vec<String>` means `Table` sees only the rendered string for each cell. Sort comparators receive `(&str, &str)` and parse on every comparison. Numeric columns sort lexicographically by default ("11.48x" < "4.12x"), and the `Column::with_comparator(numeric_comparator())` API exists but is undiscoverable.
- **G3 / D4** Per-cell styling. `TableRow::cells() -> Vec<String>` precludes per-cell color. Consumers force severity/status coloring into a sibling pane for the focused row, losing at-a-glance comparison across all rows.
- **G7** `TableMessage::SortBy(usize)` overloads three different intents into one message: first press sets Asc, second flips to Desc, third clears. Every consumer that wants a 2-cycle (Asc ↔ Desc) toggle writes the same ~30-line wrapper. Pinned by the leadline test `apply_table_msg_skips_cleared_state_on_third_press`.

In parallel, `ResourceTable` (a separate component shipped as v0.16's flagship) was originally justified by "richer cells (per-cell style), age formatting helpers, status-dot indicator." After this spec lands, only the status-dot is genuinely different from `Table` — the cells, sort, and age-formatting concerns collapse into shared infrastructure. Carrying two near-identical components forward violates the "one consistent way to do it across components" principle.

This spec is the result of a brainstorm conducted on 2026-05-01/02 with leadline Claude as reviewer. Five rounds of section-by-section review converged the design.

## Decisions summary

| Decision | Choice | Rationale |
|---|---|---|
| Back-compat for `with_comparator` | **A** — Replace, no deprecation | Pre-1.0; deprecation cycles explicitly not done on this library |
| Where the sort key lives | **Y** — On the cell | Display text and sort key live next to each other in the impl, can't drift |
| `SortKey` shape | **β** — Rich enum, no `Custom` | Eight variants cover every K8s/observability datatype; `Custom` is a footgun (`Arc<dyn Ord>` can't sort across heterogeneous types) |
| Cell type | **Q** — One shared `Cell` | One mental model across all tabular components; `Table` can grow per-cell styling without forking a new component |
| Component count | **1** — Merge `ResourceTable` into `Table` | After cell+sort unification, only the status-dot is a real architectural difference; one component handles both use cases |
| Sort message vocabulary | **Replace** — explicit primitives + 2-cycle toggle | `SortBy`'s "infer intent from prior state" overload is exactly where the bug lives |
| `Column::default_sort` | **Per-column** | Latency / regression / error-count columns naturally start Desc; consumer encodes that on the column rather than in init dispatch |
| Initial sort declaration | **`TableState::with_initial_sort`** | Initial sort is a table-level concept; per-column form invites precedence ambiguity |

## Architecture

**One component for tabular data.** `src/component/resource_table/` is deleted in its entirety (mod.rs, render.rs, state.rs, tests.rs, snapshots/). Everything `ResourceTable` did becomes part of `Table` or part of a shared types module.

**Shared cell type.** A new `Cell` struct lives at `src/component/cell.rs` carrying display text, optional `CellStyle`, and optional `SortKey`. Both styling (formerly RT-only) and sort key (new) flow through this single type.

**`TableRow` gains one method, loses one shape.** `TableRow::cells()` returns `Vec<Cell>` instead of `Vec<String>`. `TableRow::status()` is added with a default impl returning `RowStatus::None`. The status column appears in render only if at least one row returns non-None status. `Column::with_comparator` and `SortComparator` and the `numeric_comparator()` / `date_comparator()` helpers are deleted.

**Sort drives off the cell.** Sort comparator becomes: pull `cells()[col].sort_key`, fall back to `SortKey::String(text)` if `None`, compare per `SortKey` rules. No more string parsing per compare.

**Sort vocabulary redesigned.** `SortBy` / `AddSort` / `ClearSort` are deleted. Replaced by an explicit primitive family (`SortAsc`, `SortDesc`, `SortToggle`, `SortClear`, `RemoveSort`) and a parallel tiebreaker family (`AddSortAsc`, `AddSortDesc`, `AddSortToggle`). `SortToggle` is a 2-cycle that never clears.

**Per-column natural direction and declarative initial sort.** `Column::with_default_sort(SortDirection)` declares the column's natural direction; `SortToggle` uses it on first activation. `TableState::with_initial_sort(col, dir)` and `with_initial_sorts(Vec<InitialSort>)` bootstrap the table into a sorted state on frame 1 — no more "dispatch SortBy twice in init()".

### Module layout

```
src/component/
    cell.rs              (NEW — Cell, CellStyle, SortKey, RowStatus, ~300 lines)
    table/
        mod.rs           (MODIFIED — gains status() on TableRow, switches to Cell, new sort variant arms)
        types.rs         (MODIFIED — Column loses comparator field, gains default_sort; TableMessage rewritten)
        state.rs         (MODIFIED — sort comparator switches to SortKey; gains with_initial_sort builders)
        render.rs        (MODIFIED — gains per-cell styles + optional status column)
        ...
    resource_table/      (DELETED — entire directory)
    data_grid/           (MODIFIED — TableRow impls return Vec<Cell>; editable gating unchanged)
```

## Types

### `Cell` (new, in `src/component/cell.rs`)

```rust
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Cell {
    text: CompactString,
    style: CellStyle,
    sort_key: Option<SortKey>,
}

impl Cell {
    /// Plain cell — default style, no typed sort key (will fall back to lexicographic on cell text).
    pub fn new(text: impl Into<CompactString>) -> Self;

    // Builder methods (chainable, return Self)
    pub fn with_style(self, style: CellStyle) -> Self;
    pub fn with_sort_key(self, key: SortKey) -> Self;
    pub fn with_text(self, text: impl Into<CompactString>) -> Self;

    // Convenience constructors that bind text + sort key in one call.
    // text is built via Display::fmt of value. For mixed-precision columns,
    // chain `.with_text(format!(...))` to override the display string.
    pub fn number(value: f64) -> Self;     // text = format!("{}", value), key = SortKey::F64(value)
    pub fn int(value: i64) -> Self;        // text = format!("{}", value), key = SortKey::I64(value)
    pub fn uint(value: u64) -> Self;
    pub fn bool(value: bool) -> Self;
    pub fn duration(d: Duration) -> Self;  // text = format_duration(d) ("3m12s"), key = SortKey::Duration(d)
    pub fn datetime(t: SystemTime) -> Self;

    // Style-flavored constructors (parallel to former ResourceCell::*)
    pub fn success(text: impl Into<CompactString>) -> Self;     // CellStyle::Success
    pub fn warning(text: impl Into<CompactString>) -> Self;
    pub fn error(text: impl Into<CompactString>) -> Self;
    pub fn muted(text: impl Into<CompactString>) -> Self;

    // Accessors
    pub fn text(&self) -> &str;
    pub fn style(&self) -> &CellStyle;
    pub fn sort_key(&self) -> Option<&SortKey>;
}

impl From<&str> for Cell { /* ... */ }
impl From<String> for Cell { /* ... */ }
impl From<CompactString> for Cell { /* ... */ }
```

`Cell` implements `From<&str>`, `From<String>`, `From<CompactString>` so common `cells()` impls don't need explicit `Cell::new`:

```rust
fn cells(&self) -> Vec<Cell> {
    vec![
        (&self.name).into(),                  // plain text
        Cell::number(self.cpu),               // typed numeric, Display formatting
        Cell::number(self.delta_ms)
            .with_text(format!("+{:.2} ms", self.delta_ms)),  // mixed precision
    ]
}
```

#### `Cell::number` mixed-precision caveat

`Cell::number(v)` defaults the display string to `format!("{}", v)`. For fixed-precision columns (`{:.2}`, custom unit suffixes, etc.) chain `.with_text(format!(...))` — the value will appear twice at the call site. This is a Rust constraint (compile-time format strings), not an envision design choice. A small per-column helper in your `cells()` impl is the conventional answer:

```rust
fn ms_cell(v: f64) -> Cell { Cell::number(v).with_text(format!("{:.2} ms", v)) }
```

A runtime format-string variant (`Cell::number(v).with_format("{:.2}")`) was considered and rejected: implementing it requires a custom format-spec mini-parser (error-prone, edge cases on fill/width/alignment/precision/sign), pulling in `strfmt`/`dyn-fmt` (extra dep + runtime-parsing perf), or a closed enum of preset patterns (won't cover the long tail). None justifies the cost.

### `SortKey` (new)

```rust
// DO NOT REORDER VARIANTS — discriminant order is part of the cross-variant
// fallback contract. See `cell::SortKey::compare` for the exact rules.
#[derive(Clone, Debug, PartialEq)]
pub enum SortKey {
    String(CompactString),
    I64(i64),
    U64(u64),
    /// Uses `f64::total_cmp`; NaN sorts after `+∞` (and `SortKey::None` sorts
    /// after NaN). Real values first, then NaN, then absent.
    F64(f64),
    Bool(bool),
    Duration(std::time::Duration),
    DateTime(std::time::SystemTime),
    /// Sorts last in ascending, first in descending — i.e., always at the
    /// bottom of the visible list ("nulls last").
    None,
}

impl SortKey {
    pub fn compare(a: &Self, b: &Self) -> std::cmp::Ordering;
}
```

### `CellStyle` (moved from `resource_table`, unchanged)

```rust
#[derive(Clone, Debug, Default, PartialEq)]
pub enum CellStyle {
    #[default]
    Default,
    Success,
    Warning,
    Error,
    Muted,
    Custom(Style),
}
```

### `RowStatus` (moved from `resource_table`, unchanged)

```rust
#[derive(Clone, Debug, Default, PartialEq)]
pub enum RowStatus {
    #[default]
    None,           // no dot rendered
    Healthy,        // ● green
    Warning,        // ▲ yellow
    Error,          // ✖ red
    Unknown,        // ? gray
    Custom { symbol: &'static str, color: Color },
}

impl RowStatus {
    pub fn indicator(&self) -> Option<(&'static str, Color)>;
}
```

### `TableRow` (modified)

```rust
pub trait TableRow: Clone {
    fn cells(&self) -> Vec<Cell>;

    /// Optional row-level status indicator. Default: `None` — no status
    /// column is rendered. If any row in the table returns non-None, the
    /// status column is rendered for all rows.
    fn status(&self) -> RowStatus { RowStatus::None }
}
```

### `Column` (modified)

```rust
pub struct Column {
    header: String,
    width: Constraint,
    sortable: bool,
    visible: bool,
    editable: bool,                  // unchanged — DataGrid reads this
    default_sort: SortDirection,     // NEW
    // comparator: Option<SortComparator>,  // DELETED
}

impl Column {
    /// Declares this column's natural sort direction. `SortToggle` and
    /// `AddSortToggle` use this when activating the column for the first
    /// time. Default: `Ascending`.
    ///
    /// Use `Descending` for columns where bigger-is-worse (latency,
    /// regression delta, error count) — the user's first instinct on those
    /// is almost always "show me the worst first."
    pub fn with_default_sort(mut self, dir: SortDirection) -> Self;
    pub fn default_sort(&self) -> SortDirection;

    // ... existing builders (sortable, visible, editable, etc.) unchanged
}
```

`SortComparator`, `numeric_comparator()`, `date_comparator()`, and `Column::with_comparator` / `Column::comparator` are deleted.

### `TableMessage` (sort family rewritten)

```rust
pub enum TableMessage {
    // Navigation, filter, resize variants unchanged

    // ===== Primary sort family =====
    /// Set the primary sort to this column, ascending. Replaces the entire
    /// sort stack with just this entry.
    SortAsc(usize),

    /// Set the primary sort to this column, descending. Replaces the entire
    /// sort stack with just this entry.
    SortDesc(usize),

    /// 2-cycle toggle. Never clears.
    /// - If this column is already the primary sort: flip Asc ↔ Desc.
    /// - If this column is not currently in the sort stack: activate it
    ///   using `Column::default_sort()`. Default fallback: `Ascending`.
    /// This is the variant most consumers want for header clicks and
    /// keyboard shortcuts. Pressing it N times keeps the column sorted —
    /// the indicator never disappears.
    SortToggle(usize),

    /// Drop the primary sort and any tiebreakers. Returns to load order.
    SortClear,

    /// Drop just one column from the multi-sort stack (primary or
    /// tiebreaker). The remaining columns keep their relative order
    /// and directions. If the dropped column was primary, the next
    /// tiebreaker is promoted.
    RemoveSort(usize),

    // ===== Multi-column tiebreaker family =====
    /// Add this column to the sort stack as a lowest-priority Asc
    /// tiebreaker. If the column is already in the stack, replace its
    /// direction in place — do not reorder.
    AddSortAsc(usize),

    /// Add this column to the sort stack as a lowest-priority Desc
    /// tiebreaker. If already in the stack, replace direction in place.
    AddSortDesc(usize),

    /// Toggle this column's tiebreaker direction. If not in the stack,
    /// add it using `Column::default_sort()`.
    AddSortToggle(usize),
}
```

### `TableOutput` (unchanged + new `InitialSort`)

```rust
pub enum TableOutput<T: Clone> {
    Selected(T),
    SelectionChanged(usize),
    Sorted { column: usize, direction: SortDirection },
    SortCleared,
    FilterChanged(String),
    // ... existing variants unchanged
}

pub struct InitialSort {
    pub column: usize,
    pub direction: SortDirection,
}
```

### `TableState` builders (additions)

```rust
impl<T: TableRow> TableState<T> {
    /// Set the initial primary sort declaratively. Rebuilds `display_order`
    /// immediately so the first frame renders sorted (no "dispatch SortBy
    /// twice in init()" bootstrap dance).
    ///
    /// If `col` is non-sortable, the sort is set anyway (this is a
    /// declarative bootstrap and the consumer is asserting intent).
    /// Downstream `SortToggle(col)` etc. on a non-sortable column remain
    /// silent no-ops.
    pub fn with_initial_sort(self, col: usize, dir: SortDirection) -> Self;

    /// Multi-column variant: primary plus tiebreakers, in priority order.
    pub fn with_initial_sorts(self, sorts: Vec<InitialSort>) -> Self;
}
```

## Sort dispatch and semantics

### Comparator path

In `state.rs`'s `rebuild_display_order`:

```rust
// For each (col, direction) in self.sort_columns, in priority order:
let key_a = self.rows[a].cells().get(col)
    .and_then(|c| c.sort_key().cloned())
    .unwrap_or_else(|| {
        SortKey::String(self.rows[a].cells().get(col)
            .map(|c| c.text().into())
            .unwrap_or_default())
    });
let key_b = /* analogous for row b */;
let cmp = SortKey::compare(&key_a, &key_b);
match direction { Ascending => cmp, Descending => cmp.reverse() }
```

(Implementation will memoize `cells()` per row per sort to avoid re-calling — implementation detail, not spec-level.)

### Same-variant comparison rules

| Variant | Rule |
|---|---|
| `String(a, b)` | `a.cmp(b)` (lexicographic) |
| `I64(a, b)` | `a.cmp(b)` |
| `U64(a, b)` | `a.cmp(b)` |
| `F64(a, b)` | `f64::total_cmp(&a, &b)` — total order, NaN sorts after `+∞` |
| `Bool(a, b)` | `false < true` |
| `Duration(a, b)` | `a.cmp(b)` |
| `DateTime(a, b)` | `a.cmp(b)` (older < newer) |
| `None` vs `None` | `Equal` |

### `SortKey::None` policy

When paired with any non-`None` value, the non-`None` always wins. `None` sorts to the bottom in ascending and to the top in descending — standard SQL "nulls last in ascending order." Combined with the F64 rule above, the full ascending order is **real values < NaN < `None`** ("real → NaN → absent").

### Cross-variant comparison

Cross-variant compares (e.g., `I64(7)` vs `F64(3.5)` in the same column) shouldn't happen in well-formed code — a column should produce one variant — but sort must define behavior for invalid inputs.

**Fall back to discriminant order** (the order variants appear in the enum definition: `String < I64 < U64 < F64 < Bool < Duration < DateTime < None`) and emit exactly **one `tracing::warn!` per `(render_pass, column_index)`** with the column index. Rationale: keeps sort total and panic-free; the warn surfaces the bug for the consumer to fix without crashing their app.

The "always equal" alternative was considered and rejected: equal pairs cause flicker on every sort rebuild, which is worse UX than a well-defined-but-arbitrary order.

The `// DO NOT REORDER VARIANTS` doc comment on the `SortKey` enum locks this contract: a routine "alphabetize the variants" cleanup would silently change behavior for any consumer relying on the warn-and-still-sort path.

### Stable sort

The implementation uses `slice::sort_by`, never `sort_unstable_by`. Required for `SortKey::None`-vs-`SortKey::None` and any same-key pair to preserve insertion order — otherwise consecutive equal-key rows shuffle on every rebuild.

### Output behavior table

| Message | State change | Output |
|---|---|---|
| `SortAsc(col)` when not currently `(col, Asc)` | new primary `(col, Asc)` | `Sorted { column: col, direction: Asc }` |
| `SortAsc(col)` when already `(col, Asc)` | unchanged | `None` (idempotent) |
| `SortDesc(col)` when not currently `(col, Desc)` | new primary `(col, Desc)` | `Sorted { column: col, direction: Desc }` |
| `SortDesc(col)` when already `(col, Desc)` | unchanged | `None` (idempotent) |
| `SortToggle(col)`, col not in stack | activate using `column.default_sort()` | `Sorted { column, direction }` |
| `SortToggle(col)`, col is primary | flip direction | `Sorted { column, direction }` |
| `SortToggle(col)`, col is non-sortable | unchanged | `None` (silent no-op) |
| `SortClear` when stack non-empty | emptied | `SortCleared` |
| `SortClear` when stack already empty | unchanged | `None` |
| `RemoveSort(col)` removes primary, stack non-empty | next tiebreaker promoted | `Sorted { column: new_primary, direction }` |
| `RemoveSort(col)` removes tiebreaker, primary unchanged | stack mutated | `None` (no observable primary change) |
| `RemoveSort(col)` removes the only sort | stack now empty | `SortCleared` |
| `RemoveSort(col)` where col not in stack | unchanged | `None` |
| `AddSortAsc(col)` / `AddSortDesc(col)` when col not in stack | appended | `Sorted { column, direction }` (primary unchanged) |
| `AddSortAsc(col)` when col already in stack with Asc | unchanged | `None` (idempotent) |
| `AddSortAsc(col)` when col already in stack with Desc | direction replaced in place, position unchanged | `Sorted { column, direction: Asc }` |
| `AddSortToggle(col)` not in stack | append using `column.default_sort()` | `Sorted { column, direction }` |
| `AddSortToggle(col)` in stack | flip direction in place | `Sorted { column, new_direction }` |

The implementer-trap row is the one most likely to be missed: **`RemoveSort(col)` removing the primary while leaving tiebreakers behind emits `Sorted { new_primary }`, not `SortCleared`.** Tested explicitly.

The principle: outputs reflect actual state changes. No-ops emit `None`. Consumers that want to react to "user pressed Sort regardless of state change" listen on the input message they dispatched, not on `TableOutput`.

A `TableOutput::SortStackChanged { stack: Vec<(usize, SortDirection)> }` variant for tiebreaker-only changes was considered and deferred — YAGNI until someone asks.

## Migration

### File-level changes

| Action | Path | Notes |
|---|---|---|
| DELETE | `src/component/resource_table/` (entire dir) | mod.rs, state.rs, render.rs, tests.rs, snapshots — gone |
| NEW | `src/component/cell.rs` (~300 lines) | `Cell`, `SortKey`, `CellStyle`, `RowStatus` + tests |
| MODIFY | `src/component/table/types.rs` | Drop `comparator` field + `with_comparator`/`comparator()`/`SortComparator`/`numeric_comparator`/`date_comparator`. Add `default_sort` field + `with_default_sort`/`default_sort()`. Replace sort message variants. Update `TableRow::cells() -> Vec<Cell>` and add `status()` default impl. |
| MODIFY | `src/component/table/state.rs` | Sort comparator switches to `SortKey`. Add `with_initial_sort`/`with_initial_sorts`/`InitialSort`. |
| MODIFY | `src/component/table/mod.rs` | New variant arms in `update()`; delete `SortBy`/`AddSort`/`ClearSort` arms. |
| MODIFY | `src/component/table/render.rs` | Apply per-cell styles. Render optional status column when any row has non-None status. |
| MODIFY | `src/component/data_grid/{mod,state}.rs` | Re-implement test `TableRow` impls returning `Vec<Cell>`. `editable` gating unchanged. |
| MODIFY | `src/component/mod.rs` | Remove `resource_table` re-export; add `cell` module. |
| MODIFY | `src/lib.rs` | Add top-level re-exports: `Cell`, `SortKey`, `CellStyle`, `RowStatus`, `InitialSort`. Remove `ResourceTable*` re-exports. |
| MODIFY | `examples/{table,data_grid,component_showcase}.rs` | Update `TableRow` impls; switch `SortBy` → `SortToggle`; remove `ResourceTable` demo. |
| MODIFY | `tests/{integration,serialization,integration_stress,property_extended}.rs` | Migrate row impls + sort dispatches. |
| MODIFY | `CHANGELOG.md` | Breaking-change section + migration table. |

### Migration sequence

Single PR, single (or small number of logically-grouped) commit(s), squashed at merge per project rule. Trying to keep intermediate commits compiling while changing a trait signature implemented across 26 files isn't worth the bookkeeping; the bisect surface is the same either way and the diff is cleaner as one coherent change.

### Consumer migration table

| Old | New |
|---|---|
| `TableMessage::SortBy(col)` for header-click intent | `TableMessage::SortToggle(col)` |
| `TableMessage::SortBy(col)` for "always Asc" (e.g. menu item) | `TableMessage::SortAsc(col)` |
| `TableMessage::SortBy(col)` for "always Desc" | `TableMessage::SortDesc(col)` |
| `SortBy(col); SortBy(col)` (init-time bootstrap to Desc) | `TableState::with_initial_sort(col, Descending)` |
| `TableMessage::AddSort(col)` for tiebreaker click | `TableMessage::AddSortToggle(col)` |
| `TableMessage::AddSort(col)` for "always Asc tiebreaker" | `TableMessage::AddSortAsc(col)` |
| `TableMessage::ClearSort` | `TableMessage::SortClear` |
| `Column::with_comparator(numeric_comparator())` | `Cell::number(value)` per cell (sort key inferred). Mixed-precision: `Cell::number(value).with_text(format!("{:.2}", value))` |
| `Column::with_comparator(date_comparator())` | `Cell::datetime(value)` per cell |
| `Column::with_comparator(custom_fn)` | `Cell::new(text).with_sort_key(SortKey::...)` per cell |
| `TableRow::cells() -> Vec<String>` | `TableRow::cells() -> Vec<Cell>` (use `Cell::new(s)` or `s.into()`) |
| `ResourceTable*` (any item) | `Table` with optional `TableRow::status()` for the status dot |
| `ResourceCell::*` constructors | `Cell::*` (constructors map 1:1) |
| `RowStatus` (formerly in `resource_table`) | `RowStatus` (in `envision::cell`, re-exported at crate root) |

### CHANGELOG sketch

```markdown
## [Unreleased] — Breaking: Table sort & cell API redesign

### Removed
- `TableMessage::{SortBy, AddSort, ClearSort}` — replaced by explicit primitives
- `Column::with_comparator` / `Column::comparator` / `numeric_comparator` /
  `date_comparator` / `SortComparator`
- `ResourceTable`, `ResourceRow`, `ResourceCell`, `ResourceColumn`,
  `ResourceTableState`, `ResourceTableMessage`, `ResourceTableOutput`

### Added
- `Cell { text, style, sort_key }` — unified cell type for all tabular components
- `SortKey` enum (`String`, `I64`, `U64`, `F64`, `Bool`, `Duration`, `DateTime`, `None`)
- `TableMessage::{SortAsc, SortDesc, SortToggle, SortClear, RemoveSort,
  AddSortAsc, AddSortDesc, AddSortToggle}`
- `Column::with_default_sort(SortDirection)` — declare per-column natural direction
- `TableState::with_initial_sort(col, dir)` and `with_initial_sorts(Vec<InitialSort>)`
- `TableRow::status()` (default `RowStatus::None`) — optional row-status dot column

### Migration
[migration table above, verbatim]

See `docs/customer-feedback/2026-05-01-leadline-gaps.md` for the trail of
consumer-side workarounds this redesign retires.
See `docs/superpowers/specs/2026-05-02-table-sort-cell-unification-design.md`
for the full design.
```

### Implementation PR description requirements

- Lists every deleted public item (so the breaking change is unmissable in review)
- Lists every new public item
- Reproduces the migration table inline (CHANGELOG link doesn't suffice on first review)
- Notes the `Cell::number` mixed-precision caveat
- References this spec doc and the customer-feedback tracking doc
- References leadline's source notes (`envision_gaps.md`, `envision_table_sort_api_redesign.md`) so reviewers see the consumer-side justification

## Testing & verification

### Unit tests (17 categories)

Grouped by module.

**`src/component/cell.rs`:**

1. `SortKey` per-variant comparison: `String`, `I64`, `U64`, `F64` (incl. `total_cmp` for NaN), `Bool`, `Duration`, `DateTime`, `None`-vs-`None` — all eight return correct `Ordering` for documented inputs.
3. `SortKey::F64`: `+∞ < NaN`, `NaN < SortKey::None` in ascending (real → NaN → absent).
14. `SortKey::None` stable: `[Some(1), None_a, None_b, None_c, Some(2)]` sorts to `[Some(1), Some(2), None_a, None_b, None_c]` ascending — the three Nones in original order. Implementation must use `slice::sort_by`, never `sort_unstable_by`.
- `Cell::number(v).with_text(format!(...))` doc test demonstrating the mixed-precision pattern.
- `Cell` constructors: `success`/`warning`/`error`/`muted` set the right `CellStyle`; `int`/`uint`/`bool`/`duration`/`datetime` set the right `SortKey`.
- `Cell: From<&str>`, `From<String>`, `From<CompactString>` round-trip with default style and no sort key.

**`src/component/table/types.rs`:**

12. `Column::default_sort()` returns `Ascending` when `with_default_sort` was never called.
- `Column::with_default_sort(Descending)` round-trips through getter.
- New `TableMessage` variants are `Clone + Debug + PartialEq`.

**`src/component/table/state.rs` (sort behavior):**

2. Cross-variant fallback: column with `I64(7)` and `F64(3.5)` rows sorts deterministically by discriminant order; emits exactly **one** `tracing::warn!` per `(render_pass, column_index)` regardless of row count (test with 100-row mixed-variant column → 1 warn).
4. `SortToggle` 2-cycle never clears: dispatch 10× → state always `Some((col, _))`, direction strictly alternates.
5. `SortToggle` honors `with_default_sort(Descending)` on first activation: column with no current sort, dispatch `SortToggle(col)` → direction is `Descending`.
6. `with_initial_sort(col, Descending)` produces sorted rows on frame 1 (no `update()` calls). Same for `with_initial_sorts(vec![...])`.
7. `RemoveSort` and `SortClear` output behavior — all six rows verified as separate tests:
   - removes primary, stack non-empty → `Sorted { column: new_primary, direction }`
   - removes tiebreaker, primary unchanged → `None`
   - removes only sort → `SortCleared`
   - col not in stack → `None`
   - `SortClear` when stack non-empty → `SortCleared`
   - `SortClear` when stack already empty → `None`
9. Idempotent dispatches: `SortAsc(col)` when `(col, Asc)` already primary → `None`. Same for `SortDesc`, `AddSortAsc`, `AddSortDesc`.
13. `SortToggle` / `SortAsc` / `SortDesc` / `AddSort*` on a non-sortable column → state unchanged, output `None`. Pinned per the forward-compat hedge so a future "be strict on bad input" cleanup must consciously change the test.
15. **`sort_toggle_arrow_persists_on_repeated_press`** — the originating bug. Dispatch `SortToggle(col)` 10×; query the rendered column header for the indicator character on every iteration; assert it's always present. Pinned by name so future cleanup can't silently regress.
16. `SortToggle` column-switch honors the new column's `default_sort`: stack already has `(col_A, Asc)`; dispatch `SortToggle(col_B)` where `col_B` has `with_default_sort(Descending)` → result is `(col_B, Descending)`. Catches "first activation" being mistakenly read as "stack empty" rather than "this column is new to the stack."
17. `AddSort*` position-preservation on existing entries: stack `[(0, Asc), (1, Desc), (2, Asc)]`; dispatch `AddSortAsc(1)` → result `[(0, Asc), (1, Asc), (2, Asc)]` — col 1 stays at position 1, doesn't move to the end. Same shape for `AddSortToggle`.

**`src/component/table/render.rs` (snapshot tests via `insta`):**

10. Status column: not rendered when all rows return `RowStatus::None`; rendered when at least one row returns non-None; correct symbol+color per variant.
11. Per-cell `CellStyle`: snapshot per variant (`Default`, `Success`, `Warning`, `Error`, `Muted`, `Custom(Style)`); one snapshot showing mixed styles in one row.
8. Sort indicator visibility: indicator disappears on `SortClear` output; reappears on `SortAsc`/`SortDesc`/`SortToggle`.

### Property-based tests (proptest)

In `src/component/cell.rs` and `src/component/table/sort_proptests.rs`:

- **`SortKey` ordering is total**: any two `SortKey` values `a, b` produce `Ordering` satisfying trichotomy (`a < b`, `a == b`, or `a > b`); `compare(a, b) == compare(b, a).reverse()`; transitivity holds. Generators cover all eight variants.
- **Sort is stable across permutations**: input vector with duplicate sort keys, shuffle N times, sort by same column → relative order of equal-key elements preserved across all N runs.
- **Multi-column sort respects priority**: stack `[(col_0, dir_0), (col_1, dir_1)]` produces `display_order` where rows with the same `col_0` value are sorted by `col_1` direction.

### Doc tests

- Every new public item gets a doc test with realistic usage (audit-scorecard requirement: 100% public-method doc-test coverage on `cell.rs`).
- Migration patterns shown in module-level docs:
  - `Vec<String>` → `Vec<Cell>` migration
  - `with_comparator(numeric_comparator())` → `Cell::number(v)` migration
  - `SortBy(col)` → `SortToggle(col)` / `SortAsc(col)` / `SortDesc(col)` (intent-driven)
- `Cell::number(v).with_text(format!(...))` doc test pinned per the mixed-precision caveat.

### Migration-impact tests

- All 26 existing files that touch `TableRow` / `ResourceRow` rewritten and re-verified.
- `data_grid` still gates editing on `Column::is_editable()` (regression test for the editable interaction with `Cell` migration).
- `examples/{table,data_grid,component_showcase}.rs` build clean and exercise representative code paths.
- Integration tests in `tests/{integration,serialization,integration_stress,property_extended}.rs` migrated and passing.
- `tests/serialization.rs`: serialized form of `Cell` round-trips through serde when `serialization` feature is on; `SortKey` doesn't break round-trip on `f64::NaN`.

### Bench gate

Add a sort-performance benchmark to `benches/component_view.rs` (or `component_events.rs`):

- Measures: 10k rows × 10 columns, primary sort by a numeric column. Records baseline before merge; CI bench gate fails on >10% regression.
- Rationale: the new `Cell` carries text + style + sort_key bytes per cell vs the old `String`. Sort allocates `Vec<Cell>` per row via `cells()`. Sets a perf floor before the redesign ships so post-merge regressions surface visibly.

### Audit-scorecard targets

- File-size: `cell.rs` ~300 lines (under 1000-line ceiling).
- Doc-test coverage: 100% on `cell.rs`; no regression elsewhere.
- Accessor symmetry: every `with_*` builder has matching getter; every `set_*` mirrors a field.
- Standard derives: `Cell`, `SortKey`, `CellStyle`, `RowStatus` all derive `Clone + Debug + PartialEq` (and `Default` where the default has clear semantics — `Cell::default()` = empty, `CellStyle::default()` = `Default`, `RowStatus::default()` = `None`).
- Public-item count: net should be lower (deleted RT exports + `numeric_comparator`/`date_comparator`/`SortComparator` outweigh added Cell/SortKey/`InitialSort`/new TableMessage variants).
- Target: **9/9 scorecard before merge**.

### Pre-merge verification checklist

```
cargo nextest run --all-features                       # all unit + integration tests
cargo test --doc --all-features                        # all doc tests
cargo clippy --all-features -- -D warnings             # zero warnings
cargo fmt --check                                      # formatting clean
cargo build --examples --all-features                  # examples compile
./tools/audit/target/release/envision-audit scorecard  # 9/9
```

### CI gates

Per project rule, all 16 checks green before merge:

- Test on (ubuntu / macos / windows) × (stable / 1.85) — six jobs
- Clippy, Format, Documentation, No-Default-Features, Coverage, Detect Changes
- Bench Component View, Bench Component Events (no regressions ≥10%)

## Cross-reference handshake with leadline

Once this spec doc lands and the implementation PR opens (with PR number `#N`), leadline updates `notes/envision_gaps.md` G1 / G3 / G7:

- Status field gains: "tracked upstream → `envision/docs/superpowers/specs/2026-05-02-table-sort-cell-unification-design.md`"
- Removal trigger gains: "removed once envision PR #N lands"

Trail closes on both sides.

## References

- **leadline source notes** (consumer-side, in the `rust-ai-explorations` repo):
  - `notes/envision_gaps.md` — G1 (typed sort_key), G3 (per-cell styling), G7 (sort message redesign)
  - `notes/envision_table_sort_api_redesign.md` — detailed brief for G7 that this spec absorbs
- **envision-side tracking**: `docs/customer-feedback/2026-05-01-leadline-gaps.md`
- **Brainstorm path** (resolved decisions, in chronological order):
  - A — replace `with_comparator`, no deprecation
  - Y — sort key on the cell
  - β — rich `SortKey` enum, no `Custom`
  - Q — one shared `Cell` type
  - 1 — merge `ResourceTable` into `Table`
  - Folded in: G7 sort vocabulary redesign
