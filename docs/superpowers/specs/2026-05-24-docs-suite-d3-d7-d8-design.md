# Documentation Suite (D3 + D7 + D8) Design

**Status:** Draft
**Date:** 2026-05-24
**Cadence:** 10 of 10 (FINAL) in the May 2026 leadline gap suite
**Source brief:** `~/workspace/ryanoneill/rust-ai-explorations/notes/envision_documentation_suite.md` (commit 999391b)
**Tracking:** `docs/customer-feedback/2026-05-01-leadline-gaps.md` (D3, D7, D8 rows)

## Goal

Close the three documentation-suite gaps remaining from the May 2026 leadline brief:

- **D3** — `Column::new` canonical Length+Min docstring + render-time clipping warning
- **D7** — TestHarness/AppHarness/`Runtime::virtual_terminal` compare-and-contrast docs + canonical snapshot-diff recipe
- **D8** — Master+detail drill-down example using post-cadence envision surface + Router-vs-in-state-enum guidance

After this cadence merges, the original May 2026 brief queue is closed.

## Why one combined cadence

leadline sent these as a single brief. The three items share a single review pass and a single closing tracking-doc update. Splitting into three cadences would multiply review work without materially reducing risk — D3/D7/D8 touch independent files (table render, harness module docs, examples) with no cross-coupling. One impl PR with three logically-separated commits keeps the diff reviewable while honoring the brief framing.

## Architecture overview

| Item | Files touched | Doc surface | Code surface |
|---|---|---|---|
| D3 | `src/component/table/types.rs`, `src/component/table/render.rs`, `src/component/table/mod.rs` | `Column::new` docstring expansion | `detect_clipped_columns()` helper + render-path emission |
| D7 | `src/harness/mod.rs`, `src/harness/snapshot/mod.rs` | Decision table + canonical snapshot recipe doc-test | None (pure docs) |
| D8 | `examples/router.rs`, `examples/drilldown.rs` (new), `src/component/router/mod.rs` | Router-vs-state-enum guidance | New ~120-line example + router.rs envision-surface refresh |

All three items leave existing public API surface unchanged. D3 adds one `pub(crate)` helper for testability. No new dependencies.

---

## D3 — Column docstring + render-time clipping warning

### Docstring expansion

**Location:** `src/component/table/types.rs:60-95` (the `Column::new` doc block).

**Current state:** Single-column doc-test using `Constraint::Length(20)`. No guidance on multi-column composition or on what happens when columns can't fit.

**Replacement:** Multi-column canonical doc-test demonstrating the Length+Min idiom (fixed columns plus one flexible column):

```rust
/// Creates a column with a fully-specified width constraint.
///
/// # Width semantics
///
/// `Column::new` takes any `ratatui::layout::Constraint`. The most common
/// patterns:
///
/// - [`Constraint::Length(n)`] — a hard request for exactly `n` cells.
/// - [`Constraint::Min(n)`] — a minimum of `n` cells, growing to fill
///   available space. Typical choice for one "flexible" column.
/// - [`Constraint::Percentage(n)`] — `n%` of the resolved area, partitioned
///   left-to-right with the other constraints.
///
/// `Length` and `Min` both declare an absolute floor. When the resolved
/// area is narrower than the declared floor — a `Length(n)` column that
/// got `<n` cells, or a `Min(n)` column that got `<n` cells — the column
/// emits a warning (see "Clipping diagnostics"). `Percentage` is a share
/// of the resolved area and has no absolute floor, so it is never
/// flagged.
///
/// For these three idioms the shorthand constructors [`Column::fixed`],
/// [`Column::min`], and [`Column::percent`] read more directly than
/// `Column::new`.
///
/// # Example
///
/// Three-column layout: fixed ID, fixed price, flexible description.
///
/// ```rust
/// use envision::component::Column;
/// use envision::prelude::Constraint;
///
/// let cols = vec![
///     Column::new("ID",          Constraint::Length(8)),
///     Column::new("Price",       Constraint::Length(10)),
///     Column::new("Description", Constraint::Min(20)),
/// ];
/// assert_eq!(cols.len(), 3);
/// assert_eq!(cols[0].width(), Constraint::Length(8));
/// assert_eq!(cols[2].width(), Constraint::Min(20));
/// ```
///
/// # Clipping diagnostics
///
/// When a `Length(n)` or `Min(n)` column resolves to fewer than `n`
/// cells, the column emits a `tracing::warn!` once. Dedup is per
/// `(column index, area width)`: the warning re-arms when the terminal
/// is resized or the column set is replaced via `set_columns`. This is
/// best-effort observability — the table still renders. Enable with
/// the `tracing` feature.
```

The existing `Column::fixed`, `Column::min`, `Column::percent` doc-tests stay as-is. The cross-references in the new docstring give readers a single canonical entry point.

### Render-time clipping warning

**Goal:** When the resolved column width is less than the declared `Length(n)`, surface the mismatch as a tracing warning so consumers don't silently produce truncated tables.

**Detection logic** — extract to a `pub(crate)` helper in `src/component/table/render.rs`:

```rust
/// Identifies columns whose declared lower-bound width constraint
/// (`Length(n)` or `Min(n)`) was violated by the resolved layout.
///
/// `Length` declares an exact width that doubles as both upper and
/// lower bound; `Min` declares an explicit lower bound. Both are
/// floors the consumer wrote into their column declaration, and both
/// produce the same actionable signal when the resolved width is
/// smaller. `Percentage` is a share of the resolved area with no
/// absolute floor, so it is never flagged.
pub(crate) fn detect_clipped_columns(
    columns: &[Column],
    resolved_widths: &[u16],
) -> Vec<ClippedColumn> {
    use ratatui::layout::Constraint;

    columns
        .iter()
        .zip(resolved_widths.iter())
        .enumerate()
        .filter_map(|(idx, (col, &resolved))| {
            let (declared, kind) = match col.width() {
                Constraint::Length(n) if resolved < n => (n, ClipKind::Length),
                Constraint::Min(n) if resolved < n => (n, ClipKind::Min),
                _ => return None,
            };
            Some(ClippedColumn { idx, declared, resolved, kind })
        })
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ClippedColumn {
    pub idx: usize,
    pub declared: u16,
    pub resolved: u16,
    pub kind: ClipKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ClipKind {
    Length,
    Min,
}

impl ClipKind {
    /// Renders as `"Length"` or `"Min"` for warning text.
    pub(crate) fn label(self) -> &'static str {
        match self {
            ClipKind::Length => "Length",
            ClipKind::Min => "Min",
        }
    }
}
```

Returning a struct (instead of a bare `Vec<usize>`) keeps the warning emission site free of constraint re-lookup and makes the diagnostic content (`declared`, `resolved`, `kind`) reviewable independently of `Column` internals.

**Render-path integration** — at `src/component/table/render.rs` (where `widths: Vec<Constraint>` is built at `render.rs:130-136`), perform a one-shot layout split using `ratatui::layout::Layout::horizontal(widths.clone()).split(inner_area)` to compute resolved widths. The split MUST mirror what ratatui actually feeds the Table widget so the detected resolved widths match the real render:

- The split's input vec is the full `widths` (including the leading `Constraint::Length(2)` status reservation pushed when `has_status` is true at `render.rs:131-133`) — not just the user-column slice. Splitting only the user portion over the full area produces resolved widths that are too generous by ~2 cells when `has_status` is set, masking real clipping.
- The split's input area is the inner area ratatui hands to the Table. Start from `area.inner(Margin{ horizontal: 1, vertical: 1 })` when `chrome_owned == false` (the Block::default().borders(Borders::ALL) wrap at `render.rs:155-163` shrinks by one cell on each side); otherwise `area` is already inner. Then subtract the highlight-symbol reservation: ratatui's default `HighlightSpacing::WhenSelected` reserves the highlight symbol's display width (2 cells for `"> "` set at `render.rs:153`) from the column-distribution area whenever `state.selected.is_some()`. When a row is selected, subtract 2 from the width of that area BEFORE calling `Layout::horizontal::split`. The Table widget is constructed with no explicit `highlight_spacing` call, so the default applies. (`state.selected` is the field at `state.rs:486`-vintage `TableState`.)

Map the resolved widths back to user columns by skipping the status reservation: pass `&resolved_widths[has_status as usize..]` alongside `&state.columns` to `detect_clipped_columns`. The returned `ClippedColumn.idx` values then correspond to user-column indices in `state.columns` (matching the indices the consumer sees), and `resolved` reflects what the renderer actually applied. For each `ClippedColumn` returned, check the lifetime-scoped dedup set on `TableState` and emit `tracing::warn!` on first detection (or on re-detection after a terminal resize — see below).

**Dedup state** — `Table::view` takes `state: &Self::State` (immutable, at `src/component/table/mod.rs:492`), so the dedup field uses interior mutability:

```rust
/// Transient observability state for clip-warning dedup. Tracks the
/// last area width at which clipping was evaluated and the set of
/// column indices already warned about. Reset when the area width
/// changes (terminal resize re-arms detection).
///
/// Runtime-only state; not part of logical equality and not serialized.
#[cfg_attr(feature = "serialization", serde(skip))]
clip_warn_state: RefCell<ClipWarnState>,
```

```rust
#[derive(Default, Debug, Clone)]
pub(super) struct ClipWarnState {
    last_area_width: Option<u16>,
    warned_cols: HashSet<usize>,
}
```

**Dedup semantics:**

1. On render, compare `area.width` against `last_area_width`. If different (or `None`), clear `warned_cols` and set `last_area_width = Some(area.width)`.
2. Run `detect_clipped_columns`. For each clipped index, if `warned_cols.insert(idx)` returns `true`, emit the warn.

This gives "one warn per (column index, area width) combination over the TableState's lifetime" — terminal resize re-arms; constructing a new TableState resets via `Default`; otherwise quiet steady-state.

**Column-replacement note** — TableState's column set is established at construction (`TableState::new(rows, columns)` at `src/component/table/state.rs:41`, `TableState::with_selected` at `state.rs:80`) and not mutated post-construction. There is no `set_columns()` method to hook. Replacing columns means constructing a new TableState, which resets the ClipWarnState to `Default`.

**PartialEq exclusion** — the existing custom `PartialEq for TableState` at `src/component/table/mod.rs:100-113` already excludes `cross_variant_warned_cols`. Add `clip_warn_state` to the exclusion list (same justification: transient observability).

**Warning text:**

```text
table column 'Description' clipped: declared Length(20), resolved 12 (table area 30)
table column 'Description' clipped: declared Min(20), resolved 12 (table area 30)
```

The constraint label (`Length` or `Min`) is supplied by `ClipKind::label()`. Includes column header, declared width, resolved width, table area width — actionable.

**Feature gating** — `#[cfg(feature = "tracing")]` around the emission block, matching `state.rs:686`. The detection helper itself is unconditional (pure function, trivially testable).

### Scope guards

- Only declared lower-bound constraints trigger a warning: `Length(n)` (exact, doubles as floor) and `Min(n)` (explicit floor). `Percentage(n)` is a share of the resolved area with no absolute floor and is never flagged.
- Best-effort observability — never panics, never changes render output.
- Dedup is per `(column index, area width)` over the TableState's lifetime — quiet in steady-state, re-arms on resize. Constructing a new TableState resets via `Default`.

### Tests for D3

1. `detect_clipped_columns_returns_empty_when_widths_fit` — Length(8)+Length(10) in 30 width → empty.
2. `detect_clipped_columns_identifies_truncated_length_columns` — Length(20) resolved to 10 → returns one `ClippedColumn` with `kind: Length, declared: 20, resolved: 10`.
3. `detect_clipped_columns_identifies_violated_min_constraints` — Min(20) resolved to 10 → returns one `ClippedColumn` with `kind: Min, declared: 20, resolved: 10`.
4. `detect_clipped_columns_ignores_min_when_resolved_meets_floor` — Min(10) resolved to 20 → empty (above floor is grow-friendly).
5. `detect_clipped_columns_ignores_percentage_constraints` — Percentage(50) resolved to 5 → empty (no declared floor).
6. `detect_clipped_columns_multiple_violations_mixed_kinds` — Length(20) + Min(20) + Percentage(50) all under-resolved → returns two `ClippedColumn` entries with the correct `kind` per entry; Percentage is excluded.
7. `clip_warn_state_dedupes_within_same_area_width` — render twice at the same area width with clipping; assert `warned_cols` matches the first pass's set on the second pass (no second insert).
8. `clip_warn_state_re_arms_on_area_width_change` — render at width 30 (clipped), then at width 50 (not clipped), then at width 30 again (clipped); assert the third render's `warned_cols` was reset by the intermediate width change.
9. `clip_warn_state_defaults_empty_on_new_table_state` — construct a fresh `TableState`; assert `clip_warn_state` is `Default` (no last_area_width, no warned_cols). Replacing a TableState gives a fresh dedup naturally.
10. Snapshot test `table_renders_when_columns_clipped` — render a clipped table, confirm output isn't corrupted and the table still functions.

(Tracing emission itself isn't tested — matches existing precedent at `state.rs:686` where `cross_variant_warned_cols` field is internal-only and the warn is observability, not behavior. Tests #7–#9 assert the dedup STATE directly via a `pub(super) fn clip_warn_state_for_test(&self) -> &RefCell<ClipWarnState>` accessor gated behind `#[cfg(test)]`.)

---

## D7 — Harness compare-and-contrast + snapshot diff recipe

### Compare-and-contrast docs

**Location:** `src/harness/mod.rs` (currently 38 lines, single TestHarness example only).

**Replacement:** Keep the existing TestHarness intro example. Add a "Choosing a Harness" section above it containing the decision table and per-harness blurbs.

```rust
//! Test harness for headless TUI testing.
//!
//! Envision provides three testing entry points. Pick the one that matches
//! the scope of what you're testing:
//!
//! | Harness | Use when… | Closure or App? | Time control |
//! |---|---|---|---|
//! | [`TestHarness`] | Testing a render closure or a widget in isolation | Closure | None (synchronous) |
//! | [`AppHarness`] | Testing a full `App` with async commands/subscriptions | App | Via `tokio::test(start_paused)` + `advance_time()` |
//! | [`Runtime::virtual_terminal`][vt] | Programmatic control (agents, scripted demos, integration tests) | App | None |
//!
//! [vt]: crate::app::Runtime
//!
//! ## `TestHarness` — widget-level testing
//!
//! Wraps a `CaptureBackend` with input simulation and assertion helpers.
//! Renders closures, not full `App` implementations. Synchronous — no
//! async runtime.
//!
//! ## `AppHarness` — App-level async testing
//!
//! Wraps a `Runtime<A, CaptureBackend>` and exposes time-control primitives
//! that pair with `#[tokio::test(start_paused = true)]`. Use when your `App`
//! has subscriptions, commands, or any time-dependent logic.
//!
//! ## `Runtime::virtual_terminal` — programmatic App control
//!
//! Constructed via `Runtime::<A, _>::virtual_builder(w, h).build()`. Returns
//! a `Runtime<A, CaptureBackend>` with `send()`, `dispatch()`, `tick()`, and
//! `display()` methods. Useful for AI agents, scripted demos, and
//! integration tests that need full App semantics without the time-control
//! ceremony of `AppHarness`.
//!
//! See `examples/test_harness.rs` for runnable examples of all three.
//!
//! # Example: TestHarness
//!
//! [existing example preserved]
```

**Header note in `examples/test_harness.rs`** — single comment block pointing back to the decision table:

```rust
//! See `envision::harness` module docs for the "Choosing a Harness"
//! decision table that contextualizes the three approaches demonstrated
//! here.
```

### Canonical snapshot-diff recipe

**Location:** `src/harness/snapshot/mod.rs` (currently houses `Snapshot`/`SnapshotFormat`).

**Addition:** A module-level "Golden-file pattern" section with a runnable doc-test that exercises the recipe end-to-end. No new public API.

**Recipe shape** (rendered into module docs as a doc-test):

The recipe splits creation from comparison into two functions. This matches the standard golden-file UX (create explicitly with `UPDATE_GOLDEN=1`, then compare on subsequent runs) AND keeps the doc-test fully runnable end-to-end without panic plumbing.

```rust
//! # Golden-file snapshot pattern
//!
//! Envision keeps snapshot testing dependency-light: a render produces a
//! string, and you compare it against a fixture on disk. The recipe below
//! provides two functions — `update_golden` writes the fixture
//! unconditionally, `assert_matches_golden` reads-and-compares and panics
//! on mismatch with a unified diff.
//!
//! Typical workflow: invoke `update_golden` once (often gated by an
//! `UPDATE_GOLDEN` env check at the call site) to capture the expected
//! output, then call `assert_matches_golden` from the test body to
//! verify on subsequent runs.
//!
//! ```rust
//! use std::fs;
//! use std::path::Path;
//!
//! fn update_golden(path: &Path, content: &str) {
//!     if let Some(parent) = path.parent() {
//!         fs::create_dir_all(parent).unwrap();
//!     }
//!     fs::write(path, content).unwrap();
//! }
//!
//! fn assert_matches_golden(path: &Path, actual: &str) {
//!     let expected = fs::read_to_string(path)
//!         .unwrap_or_else(|e| panic!(
//!             "golden fixture missing at {}: {} \
//!              (run with UPDATE_GOLDEN=1 to create)",
//!             path.display(), e,
//!         ));
//!     if expected != actual {
//!         panic!(
//!             "snapshot mismatch at {}:\n{}",
//!             path.display(),
//!             unified_diff(&expected, actual),
//!         );
//!     }
//! }
//!
//! fn unified_diff(expected: &str, actual: &str) -> String {
//!     let mut out = String::new();
//!     let e: Vec<&str> = expected.lines().collect();
//!     let a: Vec<&str> = actual.lines().collect();
//!     for i in 0..e.len().max(a.len()) {
//!         match (e.get(i), a.get(i)) {
//!             (Some(l), Some(r)) if l == r => out.push_str(&format!("  {l}\n")),
//!             (Some(l), Some(r)) => {
//!                 out.push_str(&format!("- {l}\n"));
//!                 out.push_str(&format!("+ {r}\n"));
//!             }
//!             (Some(l), None) => out.push_str(&format!("- {l}\n")),
//!             (None, Some(r)) => out.push_str(&format!("+ {r}\n")),
//!             (None, None) => {}
//!         }
//!     }
//!     out
//! }
//!
//! // End-to-end demo using a tempdir so the doc-test is self-contained.
//! let tmp = tempfile::tempdir().unwrap();
//! let path = tmp.path().join("golden.txt");
//! let rendered = "row 1\nrow 2\nrow 3\n";
//!
//! // First run: capture the fixture.
//! update_golden(&path, rendered);
//!
//! // Subsequent runs: assert match.
//! assert_matches_golden(&path, rendered);
//! ```
//!
//! ## Real-world call-site sketch
//!
//! ```ignore
//! let path = Path::new("tests/golden/dashboard.txt");
//! let actual = harness.snapshot_plain();
//!
//! if std::env::var("UPDATE_GOLDEN").is_ok() {
//!     update_golden(path, &actual);
//! } else {
//!     assert_matches_golden(path, &actual);
//! }
//! ```
//!
//! ## When to upgrade
//!
//! For richer diffs, review tooling (`cargo insta review`), and parallel
//! test isolation, switch to the [`insta`](https://docs.rs/insta) crate.
//! envision's own snapshot tests use `insta` internally; the pattern above
//! is offered as a starting point for downstream consumers who want
//! zero new dependencies.
```

The doc-test uses `tempfile` (already in dev-deps at `Cargo.toml:70`) so it's self-contained — every `cargo test --doc` run exercises the recipe end-to-end against an isolated tempdir.

### Tests for D7

- D7 compare-and-contrast docs: validated by `cargo doc --no-deps --all-features` (no broken intra-doc links) plus reviewability. No new unit tests — pure prose.
- D7 snapshot recipe: validated by `cargo test --all-features --doc` execution of the doc-test (recipe runs against a tempdir on every test run).

---

## D8 — Drill-down example + Router docs + router.rs refresh

### New `examples/drilldown.rs`

**Shape:** Roster (master) → Enter → PerOp (detail) → Esc → Roster (with selection preserved).

**State:**

```rust
#[derive(Clone, Debug)]
struct Operation {
    id: String,
    duration_ms: f64,
    status: String,
}

impl TableRow for Operation {
    // Two-column projection: id and duration.
    // (Full impl spelled out in the example.)
}

#[derive(Clone)]
enum Screen {
    Roster,
    PerOp { selected: usize },
}

struct State {
    screen: Screen,
    roster: TableState<Operation>,
    operations: Vec<Operation>,
}
```

Selection lives in `TableState::selected()` while on the Roster; pushed into `Screen::PerOp { selected }` on Enter; restored to TableState via `state.roster.set_selected(Some(selected))` on Esc. The design point: drill-down doesn't need `RouterState` because there's no history stack — only "this screen" and "that screen with this index."

**Envision surface used:**

- `TableState` + `Column::fixed` / `Column::min` for the Roster table
- `PaneLayout::view_with` for the PerOp detail screen (header pane + body pane)
- `styled_line` + `InlineStyle` for highlighting key metrics
- `with_title_style` + `with_color` for the PerOp pane title
- `StatusBar::with_right_separator(" ")` (post-D12) for the operation context line

**Rendering:** Via `Runtime::<DrillApp, _>::virtual_builder(80, 24).build()`. The `main()` walks through Roster → drill-in → drill-out, printing `vt.display()` at each step so the example doubles as a visual smoke test.

**Length target:** ~120 lines. Stays focused on the pattern; doesn't try to demonstrate every envision component.

### Router docs update

**Location:** `src/component/router/mod.rs` module-level docs.

**Addition:** A "When to use Router vs. an in-state enum" section:

```rust
//! # Choosing Router vs. an in-state enum
//!
//! `Router` provides a history stack with back navigation, breadcrumbs,
//! and deep-link semantics. Reach for it when:
//!
//! - Users need a "Back" button or breadcrumb trail
//! - Screens can be revisited in different orders
//! - You want to model "where did the user come from?"
//!
//! For simpler navigation — where screens are mutually exclusive and
//! "back" just means "return to the prior selection" — an in-state enum
//! (e.g., `enum Screen { Roster, PerOp { selected: usize } }`) is lighter
//! and clearer. See `examples/drilldown.rs` for the in-state-enum pattern
//! and `examples/router.rs` for the Router pattern.
```

### `examples/router.rs` refresh

**Problem:** Current screen-render bodies at lines 113-122 use `ratatui::widgets::Paragraph::new(body).block(Block::default().borders(...).title(title))`. The example demonstrates Router state but doesn't showcase envision's component layer.

**Fix:** Replace the Paragraph+Block render with `PaneLayout::view_with` + `styled_line` while preserving identical screen content. Functionally identical render, but the example earns its keep as an envision Router demo. Approximate diff: ~30 lines inside the existing `view()` fn.

The history-stack walk in `main()` stays unchanged — that's the demonstrated Router feature.

### Tests for D8

- `cargo build --examples --all-features` must compile both `drilldown.rs` and refreshed `router.rs`.
- Both examples must produce non-empty `vt.display()` output for their `main()` walks (verified by running the example binaries in CI's existing example-build step).
- Router module-docs additions validated by `cargo doc --no-deps --all-features` (no broken intra-doc links).

---

## Verification gauntlet (each impl commit, full suite at PR)

- `cargo nextest run --all-features` — all tests pass
- `cargo test --all-features --doc` — all doc-tests pass (D3 docstring, D7 snapshot recipe, D8 Router docs cross-link compile)
- `cargo clippy --all-features -- -D warnings` — no clippy warnings
- `cargo fmt --check` — clean
- `cargo build --no-default-features` — must pass (D5+D14 lesson: tracing-feature-gated code must compile out cleanly)
- `cargo build --examples --all-features` — drilldown.rs + refreshed router.rs both compile
- `./tools/audit/target/release/envision-audit all` — 8/9 baseline preserved (resource_gauge::set_values gap is pre-existing)

## File-size cap watch

- `src/component/table/types.rs` — currently ~600 lines (well under 1000); docstring expansion adds ~30 lines. No concern.
- `src/component/table/render.rs` — currently 189 lines; helper + integration add ~30 lines. Plenty of headroom.
- `src/component/table/mod.rs` — currently houses TableState; adding one HashSet field. Need to verify post-edit it's still under 1000.
- `src/harness/mod.rs` — currently 38 lines; expanding to ~100. Comfortable.
- `src/harness/snapshot/mod.rs` — currently <100 lines (header inspected); doc-test addition pushes it to ~180. Comfortable.
- `examples/drilldown.rs` — new file, ~120 lines.
- `examples/router.rs` — currently 174 lines; refresh is approximately net-zero LOC. No concern.

## CHANGELOG entry (under `[Unreleased]`)

```markdown
### Added
- `Column::new` now documents the canonical Length+Min multi-column idiom and emits a `tracing::warn!` (feature-gated) when a `Constraint::Length(n)` or `Constraint::Min(n)` column resolves to fewer cells than declared. Best-effort observability; no behavior change for consumers without `tracing` enabled. `Percentage` constraints are never flagged (no declared floor).
- `src/harness` module docs now include a "Choosing a Harness" decision table comparing `TestHarness`, `AppHarness`, and `Runtime::virtual_terminal`.
- `src/harness/snapshot` module docs now include a runnable canonical golden-file snapshot-diff recipe (dependency-free, `std::fs` + manual diff; `insta` linked as the upgrade path).
- `examples/drilldown.rs` — master+detail drill-down pattern using `TableState`, `PaneLayout::view_with`, `styled_line`, and `StatusBar::with_right_separator`.
- `Router` module docs now include guidance on choosing between `Router` (history stack) and an in-state enum (mutual-exclusion screens with restored selection).

### Changed
- `examples/router.rs` screen-render bodies now use `PaneLayout::view_with` + `styled_line` (was: raw `ratatui::widgets::Paragraph`). No behavior change; better showcase of envision surface.
```

## Cadence breakdown

1. **Spec PR** — this document, branch `docs-suite-d3-d7-d8-spec`.
2. **Plan PR** — implementation plan to `docs/superpowers/plans/2026-05-24-docs-suite-d3-d7-d8.md`, branch `docs-suite-d3-d7-d8-plan`.
3. **Implementation PR** — three signed commits (D3, D7, D8) + CHANGELOG entry + verification gauntlet. Branch `docs-suite-d3-d7-d8-impl`.
4. **Tracking-doc PR** — close out D3+D7+D8 rows in `docs/customer-feedback/2026-05-01-leadline-gaps.md` AND mark the May 2026 brief queue ✅ complete (this is the queue closure milestone). Branch `docs-suite-d3-d7-d8-tracking`.

## Lessons baked in from prior cadences

- **Doc-test coverage on every new `pub fn`** (G4+G5 PR #487 + G6 PR #491 regression lesson) — D3 helper is `pub(crate)`, so no doc-test required, but the multi-column `Column::new` example IS a new doc-test and counts toward the 100% baseline.
- **`cargo build --no-default-features` verification** (D5+D14 lesson) — the tracing-gated emission must compile cleanly when `tracing` is disabled.
- **fmt drift watch** (D6+D9, D5+D14, G4+G5, G6, D10+D12+D13 precedents) — fmt-check is in the gauntlet; one follow-up commit if drift surfaces.
- **File-size cap watch** (G4 `title_style.rs` sibling-file pattern, D12 `per_side_separators.rs`) — explicit line-count checks listed above; mitigation path is helper-into-sibling-file if anything breaches.

## Out of scope

- New harness public API. The decision-table docs and snapshot recipe are pure documentation. If consumers want a built-in `assert_matches_golden` helper, that's a separate future cadence.
- New `Constraint` types or column resize semantics. D3 documents and observes existing behavior; it doesn't change layout.
- Replacing `insta` in envision's own tests. The snapshot recipe is offered as a starting point for downstream consumers; envision internals continue to use `insta`.
- Tracing-output capture in tests. Existing `cross_variant_warned_cols` precedent doesn't test the warn itself, so D3 doesn't either — the pure `detect_clipped_columns` helper is the testable surface.
