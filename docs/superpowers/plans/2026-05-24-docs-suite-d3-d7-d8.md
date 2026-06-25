# D3 + D7 + D8 Documentation Suite Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship the three documentation-suite gaps remaining from the May 2026 leadline brief — D3 (Column docs + clip warning), D7 (harness compare-and-contrast + snapshot recipe), D8 (drilldown example + Router docs + router.rs refresh) — as one impl PR with three logically separated commits, closing the May 2026 brief queue.

**Architecture:** Three independent file-set touchpoints with no cross-coupling: D3 touches `src/component/table/` (types, render, mod, state, tests), D7 touches `src/harness/` and one example header, D8 touches `examples/` (drilldown, router) and `src/component/router/mod.rs`. Detection in D3 is a `pub(crate)` pure function (consuming the same widths vec the renderer resolves, then sliced past the status-column reservation by `has_status as usize`) plus a render-path emission site with interior-mutability dedup (`RefCell<ClipWarnState>`) — render path stays `&state`. D7 is pure documentation expansion plus one runnable doc-test. D8 swaps raw-ratatui rendering in `router.rs` for envision components and adds a new ~160-line `drilldown.rs` showing master+detail without history-stack, with per-view `KeyHints` and `App::handle_event_with_state` so screen-gated key bindings match the advertised affordances.

**Tech Stack:** Rust 2024 (MSRV 1.85) · ratatui 0.29 · tracing 0.1 (optional feature) · tempfile (dev-dep) · insta (dev-dep; linked as upgrade path) · cargo-nextest

**Spec:** `docs/superpowers/specs/2026-05-24-docs-suite-d3-d7-d8-design.md` (commits b4f9761 + d8564a9 + 9b27761; PR #497).

**Cadence position:** 10th of 10 in the May 2026 leadline gap suite. After the impl PR + tracking-doc PR merge, the May 2026 queue is closed.

---

## File touchpoints

### Modified

- `src/component/table/types.rs` — `Column::new` docstring expansion (Task 1, Step 1)
- `src/component/table/render.rs` — add `ClippedColumn`, `ClipKind`, `detect_clipped_columns`; wire emission in `render_table`; add detection unit tests in nested `tests` module (Task 1, Steps 2-3, 5-6, 9)
- `src/component/table/mod.rs` — add `clip_warn_state: RefCell<ClipWarnState>` field to `TableState`; update PartialEq impl and Default impl; declare `mod clip_warn` submodule (Task 1, Step 4)
- `src/component/table/state.rs` — add `pub(super) fn clip_warn_state_for_test()` accessor (Task 1, Step 7); add unit tests in nested `state_dedup_tests` module (Task 1, Step 8)
- `src/component/table/snapshot_tests.rs` — add `table_renders_when_columns_clipped` snapshot test (Task 1, Step 10)
- `src/harness/mod.rs` — expand module-level docs with "Choosing a Harness" decision table + per-harness blurbs (Task 2, Step 1)
- `src/harness/snapshot/mod.rs` — add "Golden-file snapshot pattern" section to module docs + runnable doc-test (Task 2, Step 3)
- `examples/test_harness.rs` — add header pointer to harness module docs (Task 2, Step 2)
- `examples/router.rs` — replace screen-render bodies (lines 113-122) with `PaneLayout::view_with` + `styled_line` (Task 3, Step 2)
- `src/component/router/mod.rs` — add "When to use Router vs. an in-state enum" module-doc section (Task 3, Step 3)
- `CHANGELOG.md` — add `## [Unreleased]` Added + Changed entries covering D3/D7/D8 (Task 4, Step 1)

### Created

- `src/component/table/clip_warn.rs` — new module housing `ClipWarnState` struct + `Default` impl (Task 1, Step 4). Keeps `mod.rs` from creeping toward the 1000-line cap and matches the G4 sibling-file pattern.
- `examples/drilldown.rs` — new ~160-line master+detail example (Task 3, Step 1)

---

## Pre-execution gotchas (read once before Task 1)

- **Signed commits required.** If `git commit -S` fails at any point, STOP and ask the user. Never bypass with `--no-gpg-sign`.
- **Files must stay under 1000 lines.** Current sizes: `src/component/table/mod.rs` 516 → ~570 (adding field + clip_warn submodule decl + PartialEq line). Comfortable. `src/component/table/render.rs` 189 → ~280 (helper + types + emission + tests). Comfortable. `src/harness/mod.rs` 38 → ~100. `src/harness/snapshot/mod.rs` 347 → ~430 (doc-test addition; module is well under cap).
- **No clippy warnings allowed.** Run `cargo clippy --all-features -- -D warnings` before each commit.
- **`cargo build --no-default-features` must pass.** The clip-warn emission block is `#[cfg(feature = "tracing")]`-gated; the `RefCell<ClipWarnState>` field is unconditional. Verify both default and no-default builds in Task 4.
- **cargo-nextest is the test runner.** Use `cargo nextest run --all-features` for unit/integration; `cargo test --all-features --doc` for doc-tests (nextest does not currently run doc-tests).
- **Existing tracing dedup precedent at `src/component/table/state.rs:686`:** `#[cfg(feature = "tracing")]` on the emission block, dedup field excluded from custom PartialEq via comment. Mirror this exactly.
- **`Table::view` takes `state: &Self::State`** (immutable, at `src/component/table/mod.rs:492`). Render-path mutation of dedup state requires `RefCell` interior mutability.
- **`Layout::horizontal` API:** `ratatui::layout::Layout::horizontal(constraints).split(area)` returns `Rc<[Rect]>` in ratatui 0.29; iterate with `.iter().map(|r| r.width)` for resolved widths.
- **`Column::width()` returns `Constraint` by value** (it's `Copy`-able since variants are u16 or similar).
- **`TableState::set_selected(Option<usize>)`** at `state.rs:486` is the selection setter; NOT `select(...)`. The drilldown example uses this.
- **`PaneLayout::view_with` call shape** mirrors `examples/pane_layout.rs:79-95`: needs a `RenderContext::new(frame, area, &theme).focused(true)` constructed by the caller.
- **`TableState::new(rows, columns)`** at `state.rs:41` is the construction site — columns are immutable post-construction. Replacing columns means constructing a new TableState. No `set_columns` method exists or should be added in this PR (per spec amendment 9b27761).
- **Audit baseline:** 8/9 must be preserved (`resource_gauge::set_values` gap is pre-existing). Run `./tools/audit/target/release/envision-audit all` and confirm the score hasn't regressed.
- **Every new `pub fn` needs `# Example` doc test.** The D3 detection helper is `pub(crate)` so this doesn't apply. The new `Column::new` docstring example IS net-new public doc-test coverage. No other new `pub fn` is introduced in this plan.

---

## Task 1: D3 — Column docs + clip warning

**Files:**
- Modify: `src/component/table/types.rs` (Column::new doc block at lines 84-99)
- Modify: `src/component/table/render.rs` (add types + helper + wire emission; add nested `tests` module)
- Modify: `src/component/table/mod.rs` (add field; update PartialEq + Default; declare submodule)
- Modify: `src/component/table/state.rs` (add `clip_warn_state_for_test` accessor; add nested test module)
- Create: `src/component/table/clip_warn.rs` (new module housing `ClipWarnState`)
- Modify: `src/component/table/snapshot_tests.rs` (add `table_renders_when_columns_clipped`)

### Step 1: Expand `Column::new` docstring

- [ ] Open `src/component/table/types.rs`. Locate the existing `Column::new` doc block (currently lines 78-95).

Replace the existing doc block (lines 78-95) with:

```rust
    /// Creates a new column with the given header and width.
    ///
    /// # Width semantics
    ///
    /// `Column::new` takes any `ratatui::layout::Constraint`. The most
    /// common patterns:
    ///
    /// - [`Constraint::Length(n)`] — a hard request for exactly `n` cells.
    /// - [`Constraint::Min(n)`] — a minimum of `n` cells, growing to fill
    ///   available space. Typical choice for one "flexible" column.
    /// - [`Constraint::Percentage(n)`] — `n%` of the resolved area,
    ///   partitioned left-to-right with the other constraints.
    ///
    /// `Length` and `Min` both declare an absolute floor. When the
    /// resolved area is narrower than the declared floor — a `Length(n)`
    /// column that got `<n` cells, or a `Min(n)` column that got `<n`
    /// cells — the column emits a warning (see "Clipping diagnostics").
    /// `Percentage` is a share of the resolved area and has no absolute
    /// floor, so it is never flagged.
    ///
    /// For these three idioms the shorthand constructors
    /// [`Column::fixed`], [`Column::min`], and [`Column::percent`] read
    /// more directly than `Column::new`.
    ///
    /// The column is not sortable by default.
    ///
    /// # Example
    ///
    /// Three-column layout: fixed ID, fixed price, flexible description.
    ///
    /// ```rust
    /// use envision::component::Column;
    /// use ratatui::layout::Constraint;
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
    /// `(column index, area width)`: the warning re-arms when the
    /// terminal is resized. This is best-effort observability — the
    /// table still renders. Enable with the `tracing` feature.
    pub fn new(header: impl Into<String>, width: Constraint) -> Self {
```

- [ ] Run `cargo test --all-features --doc -p envision -- column` (or `cargo test --all-features --doc` if scoping doesn't work) to confirm the new doc-test compiles and passes.

Expected: PASS for the `Column::new` doc-test.

### Step 2: Add `ClippedColumn` + `ClipKind` types in render.rs

- [ ] Open `src/component/table/render.rs`. After the `use` statements (currently around lines 5-9), before the `cell_style_to_ratatui` fn, add:

```rust
/// Identifies columns whose declared lower-bound width constraint was
/// violated by the resolved layout.
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

- [ ] Save the file.

### Step 3: Add `detect_clipped_columns` helper

- [ ] In the same file, immediately after the `ClipKind` impl block, add:

```rust
/// Returns the set of columns whose declared lower-bound width
/// (`Length(n)` or `Min(n)`) was violated by the resolved layout.
///
/// `Length` declares an exact width that doubles as both upper and lower
/// bound; `Min` declares an explicit lower bound. Both are floors the
/// consumer wrote into their column declaration. `Percentage` is a share
/// of the resolved area with no absolute floor, so it is never flagged.
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
            Some(ClippedColumn {
                idx,
                declared,
                resolved,
                kind,
            })
        })
        .collect()
}
```

- [ ] Save the file. Run `cargo check --all-features 2>&1 | tail -10`.

Expected: clean check (no errors, no warnings).

### Step 4: Create `clip_warn` submodule with `ClipWarnState`

- [ ] Create `src/component/table/clip_warn.rs`:

```rust
//! Transient observability state for clip-warning dedup.
//!
//! Kept in a sibling module to keep `mod.rs` from creeping toward the
//! 1000-line cap. Used exclusively from the render path; not exposed
//! beyond `pub(super)`.

use std::collections::HashSet;

#[derive(Default, Debug, Clone)]
pub(super) struct ClipWarnState {
    pub(super) last_area_width: Option<u16>,
    pub(super) warned_cols: HashSet<usize>,
}
```

- [ ] Open `src/component/table/mod.rs`. Locate the existing `mod` declarations (search for `^mod ` near the top of the file, typically just after `pub use` re-exports).

Find the existing module declarations and add `mod clip_warn;` alongside them. The exact location depends on existing structure; based on `grep -n "^mod " src/component/table/mod.rs` output, insert near the top with other private module declarations. The line `mod render;` is at line 57 — add immediately after:

```rust
mod render;
mod clip_warn;
```

- [ ] In the same `mod.rs`, after the existing `use std::collections::HashSet;` line near the top, add `use std::cell::RefCell;` and `use clip_warn::ClipWarnState;`.

- [ ] Locate the `TableState<T>` struct (currently at lines 80-100). After the existing `cross_variant_warned_cols: HashSet<usize>` field (line ~99), but before the closing brace, add the new field:

```rust
    /// Transient observability state for clip-warning dedup. Tracks the
    /// last area width at which clipping was evaluated and the set of
    /// column indices already warned about. Reset when the area width
    /// changes (terminal resize re-arms detection).
    ///
    /// Runtime-only state; not part of logical equality and not
    /// serialized.
    #[cfg_attr(feature = "serialization", serde(skip))]
    clip_warn_state: RefCell<ClipWarnState>,
```

- [ ] Locate the custom `PartialEq` impl at lines 100-113. The body currently lists six fields. Update the comment to mention both excluded fields, but leave the field comparisons untouched — `clip_warn_state` stays out (matches `cross_variant_warned_cols`):

```rust
impl<T: TableRow + PartialEq> PartialEq for TableState<T> {
    fn eq(&self, other: &Self) -> bool {
        // `cross_variant_warned_cols` and `clip_warn_state` are
        // intentionally excluded — both are transient diagnostics state
        // (sort-pass and render-pass respectively), not part of the
        // logical equality of the table.
        self.rows == other.rows
            && self.columns == other.columns
            && self.selected == other.selected
            && self.sort_columns == other.sort_columns
            && self.display_order == other.display_order
            && self.filter_text == other.filter_text
    }
}
```

- [ ] Locate the `Default` impl at lines 115-130. Add the new field to the `Self { ... }` body, immediately after `cross_variant_warned_cols: HashSet::new(),`:

```rust
impl<T: TableRow> Default for TableState<T> {
    fn default() -> Self {
        Self {
            rows: Vec::new(),
            columns: Vec::new(),
            selected: None,
            sort_columns: Vec::new(),
            display_order: Vec::new(),
            filter_text: String::new(),
            scroll: ScrollState::default(),
            cross_variant_warned_cols: HashSet::new(),
            clip_warn_state: RefCell::new(ClipWarnState::default()),
        }
    }
}
```

- [ ] Open `src/component/table/state.rs`. Locate every constructor that hand-builds a `TableState { ... }` literal (the `new` fn at line 41 and `with_selected` at line 80, both spotted earlier). Each one needs the new field:

For each constructor body, add — alongside `cross_variant_warned_cols: HashSet::new()`:

```rust
            clip_warn_state: std::cell::RefCell::new(crate::component::table::clip_warn::ClipWarnState::default()),
```

(Or import `RefCell` + `ClipWarnState` at the top of `state.rs` and use unqualified names. Either works; pick whichever produces fewer line additions.)

- [ ] Save all files. Run `cargo check --all-features 2>&1 | tail -10`.

Expected: clean check.

### Step 5: Wire clip-warn emission in `render_table`

- [ ] Open `src/component/table/render.rs`. Locate the `widths` construction in `render_table` (currently around lines 130-135):

```rust
    let mut widths: Vec<Constraint> = Vec::new();
    if has_status {
        widths.push(Constraint::Length(2));
    }
    for col in state.columns.iter() {
        widths.push(col.width());
    }
```

- [ ] Immediately after the `widths` vector is fully populated, add the resolve-and-detect-and-emit block. The split MUST mirror what ratatui actually feeds the Table widget so detection matches the real render — see spec's "Render-path integration" section. Concretely: feed the FULL `widths` vec (incl. the status reservation) to `Layout::horizontal`, split over the column-distribution area ratatui actually uses (subtract the border margin when `!chrome_owned`, AND subtract the highlight-symbol reservation when a row is selected), then slice the resolved rects by `has_status as usize` before mapping back to user columns.

```rust
    // Best-effort clip-warning diagnostic: compute resolved column
    // widths via a one-shot Layout split that mirrors what ratatui
    // feeds the Table widget — full `widths` vec (incl. status
    // reservation) over the column-distribution area — then slice
    // off the status reservation before mapping back to user
    // columns. Detects Length/Min declared-floor violations, dedups
    // per (column index, area width) across the TableState's
    // lifetime, and emits a tracing warning on first detection.
    //
    // Skipped entirely when the table has no user columns.
    if !state.columns.is_empty() {
        // Two adjustments to mirror what ratatui actually distributes
        // column widths over:
        //   1. Border margin: when !chrome_owned the table is wrapped in
        //      Block::default().borders(Borders::ALL) below, which
        //      shrinks the inner rect by 1 cell on each side. When
        //      chrome_owned, `area` is already inner.
        //   2. Highlight-symbol reservation: the Table is constructed
        //      with .highlight_symbol("> ") (display width 2) and no
        //      explicit .highlight_spacing(...) call, so ratatui's
        //      default HighlightSpacing::WhenSelected applies — when
        //      state.selected.is_some(), ratatui reserves 2 cells from
        //      the column-distribution area before laying out columns.
        const HIGHLIGHT_SYMBOL_WIDTH: u16 = 2; // matches "> " set at render.rs:153
        let mut col_dist_area = if chrome_owned {
            area
        } else {
            area.inner(ratatui::layout::Margin { horizontal: 1, vertical: 1 })
        };
        if state.selected.is_some() {
            col_dist_area.width =
                col_dist_area.width.saturating_sub(HIGHLIGHT_SYMBOL_WIDTH);
        }
        let resolved_rects =
            ratatui::layout::Layout::horizontal(widths.iter().copied())
                .split(col_dist_area);
        // Skip the status reservation when mapping back to user columns.
        // resolved_rects[has_status as usize..] aligns 1:1 with state.columns.
        let user_resolved: Vec<u16> = resolved_rects
            .iter()
            .skip(has_status as usize)
            .map(|r| r.width)
            .collect();
        let clipped = detect_clipped_columns(state.columns.as_slice(), &user_resolved);

        if !clipped.is_empty() {
            let mut dedup = state.clip_warn_state.borrow_mut();
            if dedup.last_area_width != Some(area.width) {
                dedup.warned_cols.clear();
                dedup.last_area_width = Some(area.width);
            }
            for clip in &clipped {
                if dedup.warned_cols.insert(clip.idx) {
                    #[cfg(feature = "tracing")]
                    tracing::warn!(
                        column_header = %state.columns[clip.idx].header(),
                        declared_kind = clip.kind.label(),
                        declared = clip.declared,
                        resolved = clip.resolved,
                        area_width = area.width,
                        "table column clipped: declared {}({}), resolved {} (table area {})",
                        clip.kind.label(),
                        clip.declared,
                        clip.resolved,
                        area.width,
                    );
                    // `clip` referenced inside cfg-gated block; suppress
                    // unused-binding lint when tracing is disabled.
                    #[cfg(not(feature = "tracing"))]
                    let _ = clip;
                }
            }
        }
    }
```

- [ ] Add a `use` for HashSet at the top of render.rs if not already present (the helper uses `Vec`, so HashSet is only needed by the wired-in block above — which doesn't directly use HashSet either; only through ClipWarnState). The block as written needs no new imports beyond what's already in scope through `use super::*`.

- [ ] Verify `state.clip_warn_state` is accessible from render.rs. Since render.rs is in `src/component/table/` and uses `use super::*;` (line 8 of render.rs), it has access to TableState's fields via the same module. The new `clip_warn_state` field is `pub(super) — wait, no. The struct uses default Rust visibility on fields, which is `pub(self)` (private to module). render.rs is INSIDE the table module, so it has access. Verify by running `cargo check --all-features`.

- [ ] Run `cargo check --all-features 2>&1 | tail -10`.

Expected: clean.

- [ ] Run `cargo check --no-default-features 2>&1 | tail -10`.

Expected: clean. The `#[cfg(not(feature = "tracing"))]` branch consumes the `clip` binding so no unused-variable warning fires.

### Step 6: Write unit tests for `detect_clipped_columns`

- [ ] At the bottom of `src/component/table/render.rs`, add (or extend if a `tests` module already exists):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Constraint;

    fn cols(widths: &[Constraint]) -> Vec<Column> {
        widths
            .iter()
            .enumerate()
            .map(|(i, &w)| Column::new(format!("col{i}"), w))
            .collect()
    }

    #[test]
    fn detect_clipped_columns_returns_empty_when_widths_fit() {
        let columns = cols(&[Constraint::Length(8), Constraint::Length(10)]);
        let resolved = vec![8, 10];
        assert!(detect_clipped_columns(&columns, &resolved).is_empty());
    }

    #[test]
    fn detect_clipped_columns_identifies_truncated_length_columns() {
        let columns = cols(&[Constraint::Length(20)]);
        let resolved = vec![10];
        let result = detect_clipped_columns(&columns, &resolved);
        assert_eq!(
            result,
            vec![ClippedColumn {
                idx: 0,
                declared: 20,
                resolved: 10,
                kind: ClipKind::Length,
            }]
        );
    }

    #[test]
    fn detect_clipped_columns_identifies_violated_min_constraints() {
        let columns = cols(&[Constraint::Min(20)]);
        let resolved = vec![10];
        let result = detect_clipped_columns(&columns, &resolved);
        assert_eq!(
            result,
            vec![ClippedColumn {
                idx: 0,
                declared: 20,
                resolved: 10,
                kind: ClipKind::Min,
            }]
        );
    }

    #[test]
    fn detect_clipped_columns_ignores_min_when_resolved_meets_floor() {
        let columns = cols(&[Constraint::Min(10)]);
        let resolved = vec![20];
        assert!(detect_clipped_columns(&columns, &resolved).is_empty());
    }

    #[test]
    fn detect_clipped_columns_ignores_percentage_constraints() {
        let columns = cols(&[Constraint::Percentage(50)]);
        let resolved = vec![5];
        assert!(detect_clipped_columns(&columns, &resolved).is_empty());
    }

    #[test]
    fn detect_clipped_columns_multiple_violations_mixed_kinds() {
        let columns = cols(&[
            Constraint::Length(20),
            Constraint::Min(20),
            Constraint::Percentage(50),
        ]);
        let resolved = vec![10, 10, 5];
        let result = detect_clipped_columns(&columns, &resolved);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].idx, 0);
        assert_eq!(result[0].kind, ClipKind::Length);
        assert_eq!(result[1].idx, 1);
        assert_eq!(result[1].kind, ClipKind::Min);
    }

    #[test]
    fn clip_kind_label_returns_constraint_name() {
        assert_eq!(ClipKind::Length.label(), "Length");
        assert_eq!(ClipKind::Min.label(), "Min");
    }
}
```

- [ ] Run `cargo nextest run --all-features -E 'test(detect_clipped_columns) or test(clip_kind_label)' 2>&1 | tail -15`.

Expected: 7 passed, 0 failed.

### Step 7: Add `clip_warn_state_for_test` accessor

- [ ] Open `src/component/table/state.rs`. At the very bottom (after the last `impl` block, before any existing `#[cfg(test)] mod`), add:

```rust
#[cfg(test)]
impl<T: TableRow> TableState<T> {
    pub(super) fn clip_warn_state_for_test(
        &self,
    ) -> &std::cell::RefCell<crate::component::table::clip_warn::ClipWarnState> {
        &self.clip_warn_state
    }
}
```

- [ ] Save. Run `cargo check --all-features --tests 2>&1 | tail -5`.

Expected: clean.

### Step 8: Write `clip_warn_state` dedup tests

- [ ] Identify the right test location. Run `ls src/component/table/` to confirm structure. Based on prior exploration, candidates include `state.rs` (with nested `#[cfg(test)] mod` if pattern allows) or `tests/state.rs` (separate file). Use a new nested module in `state.rs`:

At the bottom of `src/component/table/state.rs`, add (extending the `#[cfg(test)]` section from Step 7 with a separate test module — or add immediately after it):

```rust
#[cfg(test)]
mod state_dedup_tests {
    use super::*;
    use crate::component::table::clip_warn::ClipWarnState;
    use ratatui::layout::{Constraint, Rect};

    // A trivial TableRow impl for tests.
    #[derive(Clone, Debug, PartialEq)]
    struct R {
        v: String,
    }
    impl TableRow for R {
        fn cells(&self) -> Vec<crate::component::cell::Cell> {
            vec![crate::component::cell::Cell::from(self.v.clone())]
        }
        fn sort_key(&self, _column: usize) -> Option<crate::component::table::types::SortKey> {
            None
        }
    }

    fn render_with_state(state: &TableState<R>, width: u16) {
        use crate::component::table::render;
        use crate::theme::Theme;
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let backend = TestBackend::new(width, 5);
        let mut term = Terminal::new(backend).unwrap();
        let theme = Theme::default();
        term.draw(|frame| {
            render::render_table(
                state,
                frame,
                Rect::new(0, 0, width, 5),
                &theme,
                false,
                false,
                false,
            );
        })
        .unwrap();
    }

    #[test]
    fn clip_warn_state_dedupes_within_same_area_width() {
        let columns = vec![Column::new("X", Constraint::Length(20))];
        let state = TableState::new(vec![R { v: "a".into() }], columns);
        render_with_state(&state, 10);
        let after_first = state.clip_warn_state_for_test().borrow().warned_cols.clone();
        render_with_state(&state, 10);
        let after_second = state.clip_warn_state_for_test().borrow().warned_cols.clone();
        assert_eq!(after_first, after_second);
        assert!(after_first.contains(&0));
    }

    #[test]
    fn clip_warn_state_re_arms_on_area_width_change() {
        let columns = vec![Column::new("X", Constraint::Length(20))];
        let state = TableState::new(vec![R { v: "a".into() }], columns);
        render_with_state(&state, 10); // clipped
        assert!(state.clip_warn_state_for_test().borrow().warned_cols.contains(&0));
        render_with_state(&state, 50); // not clipped, dedup re-arms
        assert!(state.clip_warn_state_for_test().borrow().warned_cols.is_empty());
        render_with_state(&state, 10); // clipped again, fresh re-arm
        assert!(state.clip_warn_state_for_test().borrow().warned_cols.contains(&0));
    }

    #[test]
    fn clip_warn_state_defaults_empty_on_new_table_state() {
        let state: TableState<R> = TableState::default();
        let s = state.clip_warn_state_for_test().borrow();
        assert!(s.last_area_width.is_none());
        assert!(s.warned_cols.is_empty());
    }

    #[test]
    fn clip_warn_state_initialized_in_constructor() {
        let state = TableState::new(vec![R { v: "x".into() }], vec![]);
        let s = state.clip_warn_state_for_test().borrow();
        assert!(s.last_area_width.is_none());
        assert!(s.warned_cols.is_empty());
    }
}
```

- [ ] Run `cargo nextest run --all-features -E 'test(clip_warn_state)' 2>&1 | tail -15`.

Expected: 4 passed, 0 failed.

### Step 9: Verify `cross_variant_warned_cols` precedent comment is still accurate

- [ ] Open `src/component/table/mod.rs`. The docstring on `cross_variant_warned_cols` (around lines 90-98) says "Cleared at the start of each `rebuild_display_order` call so each pass emits at most one warning per affected column." That's accurate and untouched. Confirm no edits drifted into that comment block.

### Step 10: Add snapshot test for clipped-table rendering

- [ ] Open `src/component/table/snapshot_tests.rs`. At the end of the file (before any closing `}` of a module if applicable), add:

```rust
#[test]
fn table_renders_when_columns_clipped() {
    use crate::component::table::{Column, Table, TableState};
    use crate::test_utils::setup_render;
    use ratatui::layout::Constraint;

    #[derive(Clone, Debug, PartialEq)]
    struct R(&'static str, &'static str);
    impl TableRow for R {
        fn cells(&self) -> Vec<crate::component::cell::Cell> {
            vec![
                crate::component::cell::Cell::from(self.0.to_string()),
                crate::component::cell::Cell::from(self.1.to_string()),
            ]
        }
        fn sort_key(&self, _: usize) -> Option<crate::component::table::types::SortKey> {
            None
        }
    }

    let columns = vec![
        Column::new("Long", Constraint::Length(20)),
        Column::new("Also Long", Constraint::Length(20)),
    ];
    let state = TableState::new(
        vec![R("aaa", "bbb"), R("ccc", "ddd")],
        columns,
    );

    // 20-wide area can't honor 2x Length(20). Both columns clip.
    // Table must still render without panic and content must appear.
    let (mut term, theme) = setup_render(20, 6);
    term.draw(|frame| {
        let ctx = &mut crate::component::RenderContext::new(frame, frame.area(), &theme);
        <Table<R> as crate::component::Component>::view(&state, ctx);
    })
    .unwrap();

    insta::assert_snapshot!(term.backend());
}
```

(If `setup_render` returns a different tuple shape — say `(Terminal, Theme, ...)` — adjust the destructure. Verify by running `grep -n "pub fn setup_render" src/test_utils.rs` and matching the signature exactly.)

- [ ] Run `cargo nextest run --all-features -E 'test(table_renders_when_columns_clipped)' 2>&1 | tail -15`.

Expected: PASS (first run creates a `.snap.new` file under `src/component/table/snapshots/`).

- [ ] Inspect the new snapshot file. Run `cargo insta review` to accept it, OR manually rename `.snap.new` → `.snap` after confirming the rendered output looks reasonable (text appears, no panics, layout is clipped but not corrupted).

- [ ] Re-run the test. Expected: PASS (snapshot matches).

### Step 11: Per-commit verification gauntlet

- [ ] `cargo fmt --check`
- [ ] `cargo clippy --all-features -- -D warnings`
- [ ] `cargo nextest run --all-features` — all tests pass
- [ ] `cargo test --all-features --doc` — doc-tests pass (including the new Column::new example)
- [ ] `cargo build --no-default-features`

Expected: all clean.

### Step 12: Commit D3

- [ ] Stage:

```bash
git add src/component/table/types.rs src/component/table/render.rs \
        src/component/table/mod.rs src/component/table/state.rs \
        src/component/table/clip_warn.rs src/component/table/snapshot_tests.rs
git add src/component/table/snapshots/  # newly accepted snapshot
```

- [ ] Commit:

```bash
git commit -S -m "$(cat <<'EOF'
D3: Column docs canonical Length+Min + clip warning diagnostic

Column::new docstring now leads with a multi-column canonical example
showing the fixed+fixed+Min(flex) idiom and includes "Width semantics"
+ "Clipping diagnostics" sections that cross-link the shorthand
constructors.

Render-time clip detection introduced as pub(crate) helper
`detect_clipped_columns` returning `Vec<ClippedColumn { idx, declared,
resolved, kind: Length|Min }>`. Both Length(n) and Min(n) constraints
flag when resolved < declared (declared lower-bound violation);
Percentage is silent (no absolute floor).

Emission is feature-gated (`tracing` feature) and deduped via
`RefCell<ClipWarnState>` on TableState — keyed by (column index, area
width) over the TableState's lifetime, re-arming on terminal resize.
ClipWarnState lives in a sibling module (table/clip_warn.rs) to keep
mod.rs from creeping toward the file-size cap.

Tests:
- 7 detection helper tests (empty, Length clip, Min violation,
  Min-above-floor, Percentage, mixed kinds, label())
- 4 dedup-state tests (same width dedup, area-width re-arm, default
  state, constructor init)
- 1 snapshot test confirming clipped rendering doesn't corrupt output

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

- [ ] Verify signature: `git log -1 --show-signature 2>&1 | head -5`. Expected: "Good signature".

If signing fails, STOP and ask the user. Never bypass.

---

## Task 2: D7 — Harness compare-and-contrast + snapshot recipe

**Files:**
- Modify: `src/harness/mod.rs` (expand module-level docs)
- Modify: `src/harness/snapshot/mod.rs` (add golden-file pattern doc-test)
- Modify: `examples/test_harness.rs` (header pointer)

### Step 1: Expand `src/harness/mod.rs` with decision table

- [ ] Open `src/harness/mod.rs` (currently 38 lines).

- [ ] Replace the existing module-level docstring (lines 1-26) with:

```rust
//! Test harness for headless TUI testing.
//!
//! Envision provides three testing entry points. Pick the one that
//! matches the scope of what you're testing:
//!
//! | Harness | Use when… | Closure or App? | Time control |
//! |---|---|---|---|
//! | [`TestHarness`] | Testing a render closure or a widget in isolation | Closure | None (synchronous) |
//! | [`AppHarness`] | Testing a full `App` with async commands/subscriptions | App | Via `tokio::test(start_paused)` + `advance_time()` |
//! | [`Runtime::virtual_builder`][vb] | Programmatic control (agents, scripted demos, integration tests) | App | None |
//!
//! [vb]: crate::app::Runtime::virtual_builder
//!
//! ## `TestHarness` — widget-level testing
//!
//! Wraps a `CaptureBackend` with input simulation and assertion helpers.
//! Renders closures, not full `App` implementations. Synchronous — no
//! async runtime.
//!
//! ## `AppHarness` — App-level async testing
//!
//! Wraps a `Runtime<A, CaptureBackend>` and exposes time-control
//! primitives that pair with `#[tokio::test(start_paused = true)]`. Use
//! when your `App` has subscriptions, commands, or any time-dependent
//! logic.
//!
//! ## `Runtime::virtual_builder` — programmatic App control
//!
//! Constructed via `Runtime::<A, _>::virtual_builder(w, h).build()`.
//! Returns a `Runtime<A, CaptureBackend>` with `send()`, `dispatch()`,
//! `tick()`, and `display()` methods. Useful for AI agents, scripted
//! demos, and integration tests that need full App semantics without
//! the time-control ceremony of `AppHarness`.
//!
//! See `examples/test_harness.rs` for runnable examples of all three.
//!
//! # Example: TestHarness
//!
//! ```rust
//! use envision::harness::{TestHarness, Assertion};
//! use envision::annotation::Annotation;
//! use ratatui::widgets::Paragraph;
//!
//! let mut harness = TestHarness::new(80, 24);
//!
//! harness.render(|frame| {
//!     frame.render_widget(
//!         Paragraph::new("Hello, World!"),
//!         frame.area(),
//!     );
//! }).unwrap();
//!
//! harness.assert_contains("Hello, World!");
//! ```
```

The rest of the file (mod declarations and `pub use` re-exports, lines 28-38) is unchanged.

- [ ] Save. Run `cargo doc --no-deps --all-features 2>&1 | grep -iE "warning|error" | head -10`.

Expected: no warnings on intra-doc links. The `[`Runtime::virtual_builder`][vb]` style should resolve cleanly.

### Step 2: Header pointer in `examples/test_harness.rs`

- [ ] Open `examples/test_harness.rs`. Locate the existing module docstring (currently lines 1-8). Replace with:

```rust
//! Test Harness example demonstrating testing utilities.
//!
//! See `envision::harness` module docs for the "Choosing a Harness"
//! decision table that contextualizes the three approaches demonstrated
//! here.
//!
//! This example shows different ways to test TUI applications:
//! - TestHarness for basic render testing with closures
//! - Runtime::headless for testing App implementations
//! - AppHarness for App testing
//! - Assertions and content queries
//!
//! Run with: cargo run --example test_harness
```

- [ ] Save. Run `cargo build --example test_harness --all-features 2>&1 | tail -5`.

Expected: clean build.

### Step 3: Expand `src/harness/snapshot/mod.rs` with golden-file pattern

- [ ] Open `src/harness/snapshot/mod.rs`. Locate the existing module docstring (line 1):

```rust
//! Snapshot testing support.
```

- [ ] Replace with the expanded version:

```rust
//! Snapshot testing support.
//!
//! # Golden-file snapshot pattern
//!
//! Envision keeps snapshot testing dependency-light: a render produces
//! a string, and you compare it against a fixture on disk. The recipe
//! below provides two functions — `update_golden` writes the fixture
//! unconditionally, `assert_matches_golden` reads-and-compares and
//! panics on mismatch with a unified diff.
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
//!     let expected = fs::read_to_string(path).unwrap_or_else(|e| {
//!         panic!(
//!             "golden fixture missing at {}: {} (run with UPDATE_GOLDEN=1 to create)",
//!             path.display(),
//!             e,
//!         )
//!     });
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
//! // End-to-end demo using a tempdir so the doc-test is
//! // self-contained.
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
//! For richer diffs, review tooling (`cargo insta review`), and
//! parallel test isolation, switch to the [`insta`](https://docs.rs/insta)
//! crate. envision's own snapshot tests use `insta` internally; the
//! pattern above is offered as a starting point for downstream
//! consumers who want zero new dependencies.
```

The rest of `src/harness/snapshot/mod.rs` (the `SnapshotFormat` enum and `Snapshot` struct, currently from line 3 onward) is unchanged.

- [ ] Save. Run `cargo test --all-features --doc -- snapshot 2>&1 | tail -10`.

Expected: doc-test passes (the recipe runs end-to-end against a tempdir).

### Step 4: Verify D7 doesn't break anything

- [ ] `cargo fmt --check`
- [ ] `cargo clippy --all-features -- -D warnings`
- [ ] `cargo nextest run --all-features` — all tests still pass
- [ ] `cargo test --all-features --doc` — doc-tests pass (including new snapshot recipe)
- [ ] `cargo build --no-default-features` — clean (D7 is pure docs, no feature gating concerns)
- [ ] `cargo doc --no-deps --all-features` — no broken intra-doc links

### Step 5: Commit D7

- [ ] Stage:

```bash
git add src/harness/mod.rs src/harness/snapshot/mod.rs examples/test_harness.rs
```

- [ ] Commit:

```bash
git commit -S -m "$(cat <<'EOF'
D7: harness compare-and-contrast docs + golden-file snapshot recipe

src/harness module-level docs now lead with a "Choosing a Harness"
decision table comparing TestHarness, AppHarness, and
Runtime::virtual_builder, followed by per-harness blurbs explaining
when each fits. The existing TestHarness intro example is preserved
as the runnable Example: block.

src/harness/snapshot module docs now include a "Golden-file snapshot
pattern" section with a runnable doc-test demonstrating a
dependency-free recipe: update_golden + assert_matches_golden +
unified_diff. The doc-test exercises the recipe end-to-end against
a tempdir on every cargo test --doc run. A real-world call-site
sketch shows UPDATE_GOLDEN env-var gating, and a "When to upgrade"
section points to insta for richer diffs and review tooling.

examples/test_harness.rs gains a header pointer back to the decision
table.

No new public API. No new dependencies.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

- [ ] Verify signature.

---

## Task 3: D8 — Drilldown example + Router docs + router.rs refresh

**Files:**
- Create: `examples/drilldown.rs`
- Modify: `examples/router.rs` (lines 113-122 — screen render bodies)
- Modify: `src/component/router/mod.rs` (module-level docs)

### Step 1: Create `examples/drilldown.rs`

- [ ] Create the new file. It should be ~160 lines demonstrating the master+detail pattern, with per-view `KeyHints` and state-aware event handling so global keys (Up/Down/Enter/Esc) only fire on the screen where they make sense:

```rust
//! Drilldown example — master+detail pattern with selection preservation.
//!
//! Demonstrates the in-state-enum approach to navigation (Screen enum)
//! as the lightweight alternative to Router for cases where you don't
//! need a history stack. The user lands on the Roster screen (a Table
//! of operations), presses Enter on a selected row to drill into the
//! PerOp detail screen, and presses Esc to return to the Roster with
//! the original selection preserved.
//!
//! Per-view KeyHints + state-aware event handling: the bottom row of
//! each screen renders a `KeyHints` bar listing only the keys active
//! on that screen, and `handle_event_with_state` gates Up/Down to the
//! Roster (no roster-selection ticks while drilled in) and Esc to the
//! PerOp (no drill-out attempts from the Roster). `q` quits from any
//! screen.
//!
//! Surface exercised:
//! - TableState<Operation> for the Roster
//! - PaneLayout::view_with for the PerOp split (header + body)
//! - styled_line + InlineStyle for emphasized metrics
//! - PaneConfig::with_title_style for the PerOp header pane
//! - KeyHints for per-view affordance hints
//! - App::handle_event_with_state for screen-gated key bindings
//!
//! Compare with examples/router.rs (history-stack navigation via
//! RouterState). See src/component/router/mod.rs module docs for
//! "When to use Router vs an in-state enum".
//!
//! Run with: cargo run --example drilldown --all-features

use envision::component::cell::Cell;
use envision::component::table::TableRow;
use envision::prelude::*;

/// One operation row in the Roster.
#[derive(Clone, Debug, PartialEq)]
struct Operation {
    id: String,
    duration_ms: f64,
    status: String,
}

impl TableRow for Operation {
    fn cells(&self) -> Vec<Cell> {
        vec![
            Cell::from(self.id.clone()),
            Cell::from(format!("{:.1} ms", self.duration_ms)),
            Cell::from(self.status.clone()),
        ]
    }

    fn sort_key(&self, _column: usize) -> Option<envision::component::table::types::SortKey> {
        None
    }
}

/// Application screen state.
#[derive(Clone, Debug)]
enum Screen {
    Roster,
    PerOp { selected: usize },
}

struct DrillApp;

#[derive(Clone)]
struct State {
    screen: Screen,
    roster: TableState<Operation>,
    operations: Vec<Operation>,
    roster_hints: KeyHintsState,
    perop_hints: KeyHintsState,
}

#[derive(Clone, Debug)]
enum Msg {
    DrillIn,
    DrillOut,
    SelectNext,
    SelectPrev,
    Quit,
}

/// Carve out the bottom row for KeyHints; return (main_area, hints_area).
fn split_hints_row(area: Rect) -> (Rect, Rect) {
    let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);
    (chunks[0], chunks[1])
}

impl App for DrillApp {
    type State = State;
    type Message = Msg;
    type Args = ();

    fn init(_args: ()) -> (State, Command<Msg>) {
        let operations = vec![
            Operation { id: "op-001".into(), duration_ms: 12.4, status: "ok".into() },
            Operation { id: "op-002".into(), duration_ms: 837.2, status: "slow".into() },
            Operation { id: "op-003".into(), duration_ms: 4.1, status: "ok".into() },
        ];
        let columns = vec![
            Column::new("ID", Constraint::Length(12)),
            Column::new("Duration", Constraint::Length(14)),
            Column::new("Status", Constraint::Min(10)),
        ];
        let mut roster = TableState::new(operations.clone(), columns);
        roster.set_selected(Some(0));

        let roster_hints = KeyHintsState::new()
            .hint("↑/↓", "select")
            .hint("Enter", "open")
            .hint("q", "quit");
        let perop_hints = KeyHintsState::new()
            .hint("Esc", "back")
            .hint("q", "quit");

        (
            State {
                screen: Screen::Roster,
                roster,
                operations,
                roster_hints,
                perop_hints,
            },
            Command::none(),
        )
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::DrillIn => {
                if let Some(idx) = state.roster.selected() {
                    state.screen = Screen::PerOp { selected: idx };
                }
            }
            Msg::DrillOut => {
                if let Screen::PerOp { selected } = state.screen {
                    state.roster.set_selected(Some(selected));
                    state.screen = Screen::Roster;
                }
            }
            Msg::SelectNext => {
                let next = state.roster.selected().map(|i| i + 1).unwrap_or(0)
                    .min(state.operations.len().saturating_sub(1));
                state.roster.set_selected(Some(next));
            }
            Msg::SelectPrev => {
                let prev = state.roster.selected().unwrap_or(0).saturating_sub(1);
                state.roster.set_selected(Some(prev));
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        use envision::component::pane_layout::{PaneConfig, PaneDirection, PaneLayout, PaneLayoutState};
        let area = frame.area();
        let theme = Theme::default();
        let (main_area, hints_area) = split_hints_row(area);

        match &state.screen {
            Screen::Roster => {
                let mut ctx = RenderContext::new(frame, main_area, &theme).focused(true);
                <Table<Operation> as Component>::view(&state.roster, &mut ctx);
                let mut hints_ctx = RenderContext::new(frame, hints_area, &theme).focused(true);
                <KeyHints as Component>::view(&state.roster_hints, &mut hints_ctx);
            }
            Screen::PerOp { selected } => {
                use envision::component::styled_text::StyledInline;
                use envision::render::styled_line;

                let op = &state.operations[*selected];
                let layout = PaneLayoutState::new(
                    PaneDirection::Vertical,
                    vec![
                        PaneConfig::new("header")
                            .with_title(format!(" {} ", op.id))
                            .with_title_style(
                                Style::default()
                                    .add_modifier(Modifier::BOLD)
                                    .fg(Color::Cyan),
                            )
                            .with_proportion(0.25),
                        PaneConfig::new("body")
                            .with_title(" Details ")
                            .with_proportion(0.75),
                    ],
                )
                .unwrap();

                PaneLayout::view_with(
                    &layout,
                    &mut RenderContext::new(frame, main_area, &theme).focused(true),
                    |pane_id, child_ctx| match pane_id {
                        "header" => {
                            let inlines = vec![
                                StyledInline::Plain("duration: ".to_string()),
                                StyledInline::bold(format!("{:.1} ms", op.duration_ms)),
                            ];
                            styled_line(child_ctx.frame, child_ctx.area, &inlines, child_ctx.theme);
                        }
                        "body" => {
                            let inlines = vec![
                                StyledInline::Plain("status: ".to_string()),
                                StyledInline::colored(op.status.clone(), Color::Green),
                            ];
                            styled_line(child_ctx.frame, child_ctx.area, &inlines, child_ctx.theme);
                        }
                        _ => {}
                    },
                );

                let mut hints_ctx = RenderContext::new(frame, hints_area, &theme).focused(true);
                <KeyHints as Component>::view(&state.perop_hints, &mut hints_ctx);
            }
        }
    }

    /// State-aware key handling: each screen owns only the keys it
    /// advertises. Up/Down moves the Roster selection only when the
    /// Roster is the active screen; Esc returns to the Roster only
    /// from PerOp. `q` quits from any screen.
    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        let key = event.as_key()?;
        if matches!(key.code, Key::Char('q')) {
            return Some(Msg::Quit);
        }
        match state.screen {
            Screen::Roster => match key.code {
                Key::Down => Some(Msg::SelectNext),
                Key::Up => Some(Msg::SelectPrev),
                Key::Enter => Some(Msg::DrillIn),
                _ => None,
            },
            Screen::PerOp { .. } => match key.code {
                Key::Esc => Some(Msg::DrillOut),
                _ => None,
            },
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 16 rows: ~13 for the main content + 1 for hints + table chrome.
    let mut vt = Runtime::<DrillApp, _>::virtual_builder(60, 16).build()?;

    println!("=== Drilldown Example ===\n");

    vt.tick()?;
    println!("Roster (initial):");
    println!("{}\n", vt.display());

    vt.dispatch(Msg::SelectNext);
    vt.tick()?;
    println!("After selecting row 1:");
    println!("{}\n", vt.display());

    vt.dispatch(Msg::DrillIn);
    vt.tick()?;
    println!("After Enter (PerOp detail for op-002, PerOp hints active):");
    println!("{}\n", vt.display());

    vt.dispatch(Msg::DrillOut);
    vt.tick()?;
    println!("After Esc (back to Roster, selection preserved, Roster hints restored):");
    println!("{}\n", vt.display());

    Ok(())
}
```

(Surface notes pre-verified during plan-writing: `styled_line` lives at `envision::render::styled_line` with signature `(frame: &mut Frame, area: Rect, inlines: &[StyledInline], theme: &Theme)` — it renders directly, it does NOT construct a `Line`. `StyledInline` constructors: `::Plain(String)`, `::bold(String)`, `::italic(String)`, `::underlined(String)`, `::colored(String, Color)`. `PaneConfig::with_title_style` accepts `ratatui::style::Style`. `PaneLayoutState::new` returns `Result` so the example uses `.unwrap()` since the configuration is static. `RenderContext` exposes `.theme` to the child closure via `child_ctx.theme`. `KeyHintsState::new().hint(key, action)` is the chainable builder for inline hint construction; `<KeyHints as Component>::view(state, ctx)` is the render call. `App::handle_event_with_state(state, event)` at `src/app/model/mod.rs:252` is the state-aware variant; its default delegates to `handle_event`, so overriding it does not require also implementing `handle_event`.)

- [ ] Save. Run `cargo build --example drilldown --all-features 2>&1 | tail -10`.

If errors surface (most likely: import paths or `styled_line` signature mismatch), adjust the imports and the construction of styled lines to match the actual prelude. Don't proceed to the next step until the example compiles cleanly.

Expected: clean build.

- [ ] Run the example end-to-end:

```bash
cargo run --example drilldown --all-features 2>&1 | head -30
```

Expected: four sections of output (Roster initial / After SelectNext / After DrillIn / After DrillOut) each showing rendered TUI content. Selection should be preserved across the drill-in/drill-out cycle (row 1 highlighted both times).

### Step 2: Refresh `examples/router.rs` screen rendering

- [ ] Open `examples/router.rs`. Locate the screen-rendering block (lines 113-122):

```rust
        let widget = ratatui::widgets::Paragraph::new(body).block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title(title),
        );
        frame.render_widget(widget, chunks[0]);
```

- [ ] Replace with a `PaneLayout::view_with`-based render. Single pane (no split — the demo is one body screen per Router state):

```rust
        use envision::component::pane_layout::{
            PaneConfig, PaneDirection, PaneLayout, PaneLayoutState,
        };
        let theme = Theme::default();
        let pane_layout = PaneLayoutState::new(
            PaneDirection::Vertical,
            vec![PaneConfig::new("screen")
                .with_title(format!(" {} ", title))
                .with_proportion(1.0)],
        )
        .unwrap();
        PaneLayout::view_with(
            &pane_layout,
            &mut RenderContext::new(frame, chunks[0], &theme).focused(true),
            |_pane_id, child_ctx| {
                child_ctx
                    .frame
                    .render_widget(ratatui::widgets::Paragraph::new(body), child_ctx.area);
            },
        );
```

(If `PaneConfig::new` requires more args or the API has drifted, adjust accordingly — confirm against `src/component/pane_layout/mod.rs`. The intent is "use envision's PaneLayout chrome instead of raw ratatui Block::borders".)

- [ ] Save. Run:

```bash
cargo build --example router --all-features 2>&1 | tail -5
```

Expected: clean build.

- [ ] Run the example end-to-end:

```bash
cargo run --example router --all-features 2>&1 | head -30
```

Expected: same five-screen walk as before, but each screen rendered with envision's pane chrome (the title styling will be slightly different — that's the showcase).

### Step 3: Add Router-vs-state-enum guidance to Router module docs

- [ ] Open `src/component/router/mod.rs`. Locate the existing module-level docstring at the top of the file (search for the `//!` block).

- [ ] After the existing module docs but before the first `use` or `mod` declaration, add (or insert into the existing docstring if already substantial):

```rust
//!
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
//! "back" just means "return to the prior selection" — an in-state
//! enum (e.g., `enum Screen { Roster, PerOp { selected: usize } }`) is
//! lighter and clearer. See `examples/drilldown.rs` for the
//! in-state-enum pattern and `examples/router.rs` for the Router
//! pattern.
```

- [ ] Save. Run `cargo doc --no-deps --all-features 2>&1 | grep -iE "warning|error" | head -10`.

Expected: no warnings on the new module doc section.

### Step 4: Per-commit verification gauntlet

- [ ] `cargo fmt --check`
- [ ] `cargo clippy --all-features -- -D warnings`
- [ ] `cargo nextest run --all-features`
- [ ] `cargo test --all-features --doc`
- [ ] `cargo build --no-default-features`
- [ ] `cargo build --examples --all-features` — both `drilldown` and refreshed `router` compile

Expected: all clean.

### Step 5: Commit D8

- [ ] Stage:

```bash
git add examples/drilldown.rs examples/router.rs src/component/router/mod.rs
```

- [ ] Commit:

```bash
git commit -S -m "$(cat <<'EOF'
D8: drilldown example + Router vs in-state-enum docs + router.rs refresh

New examples/drilldown.rs demonstrates master+detail navigation
without a history stack: Roster (TableState<Operation>) → Enter →
PerOp (PaneLayout::view_with showing header + body panes with
styled_line + InlineStyle) → Esc → Roster (selection preserved via
set_selected). The pattern is the in-state-enum alternative to
Router for navigation that doesn't need back-stack semantics.

Each screen renders a per-view KeyHints bar listing only the keys
active on that screen, and the example uses handle_event_with_state
to gate Up/Down to the Roster (no roster-selection ticks while
drilled in) and Esc to the PerOp. Demonstrates the discipline of
matching advertised affordances to active bindings.

src/component/router module-level docs now include a "Choosing Router
vs an in-state enum" section explaining when each fits and
cross-linking both examples.

examples/router.rs screen-rendering refreshed: replaced raw
ratatui::widgets::Paragraph + Block::borders with PaneLayout::view_with
chrome. Functionally identical render; better showcase of envision
component surface in an envision example.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

- [ ] Verify signature.

---

## Task 4: CHANGELOG entry + verification gauntlet

**Files:**
- Modify: `CHANGELOG.md`

### Step 1: Add CHANGELOG entry under `[Unreleased]`

- [ ] Open `CHANGELOG.md`. Locate the `## [Unreleased]` section header (top of the file, after any preamble).

- [ ] If the section is empty (or has minimal content), insert the following under it. If it already has `### Added` / `### Changed` subsections, append to them rather than creating duplicates:

```markdown
### Added

- `Column::new` now documents the canonical Length+Min multi-column idiom and emits a `tracing::warn!` (feature-gated) when a `Constraint::Length(n)` or `Constraint::Min(n)` column resolves to fewer cells than declared. Best-effort observability; no behavior change for consumers without `tracing` enabled. `Percentage` constraints are never flagged (no declared floor).
- `src/harness` module docs now include a "Choosing a Harness" decision table comparing `TestHarness`, `AppHarness`, and `Runtime::virtual_builder`.
- `src/harness/snapshot` module docs now include a runnable canonical golden-file snapshot-diff recipe (dependency-free, `std::fs` + manual diff; `insta` linked as the upgrade path).
- `examples/drilldown.rs` — master+detail drill-down pattern using `TableState`, `PaneLayout::view_with`, `styled_line`, per-view `KeyHints`, and `App::handle_event_with_state` for screen-gated key bindings; selection preserved across drill-in/drill-out.
- `Router` module docs now include guidance on choosing between `Router` (history stack) and an in-state enum (mutual-exclusion screens with restored selection).

### Changed

- `examples/router.rs` screen-render bodies now use `PaneLayout::view_with` (was: raw `ratatui::widgets::Paragraph` + `Block::borders`). No behavior change; better showcase of envision surface.
```

- [ ] Save.

### Step 2: Final verification gauntlet (full sweep)

- [ ] `cargo fmt --check`
- [ ] `cargo clippy --all-features -- -D warnings`
- [ ] `cargo nextest run --all-features` — all tests pass
- [ ] `cargo test --all-features --doc` — all doc-tests pass (Column::new new example + snapshot recipe doc-test)
- [ ] `cargo build --no-default-features` — clean (verifies tracing-gated code compiles out)
- [ ] `cargo build --examples --all-features` — all examples compile
- [ ] `cargo doc --no-deps --all-features` — no intra-doc link warnings
- [ ] `./tools/audit/target/release/envision-audit all 2>&1 | grep -i "scorecard\|baseline" | head -5` — 8/9 preserved

Expected: every check clean.

### Step 3: Commit CHANGELOG

- [ ] Stage:

```bash
git add CHANGELOG.md
```

- [ ] Commit:

```bash
git commit -S -m "$(cat <<'EOF'
CHANGELOG: D3 + D7 + D8 documentation suite (May 2026 closure)

Added entries for the Column::new docstring + clip warning, harness
compare-and-contrast docs + snapshot recipe, drilldown example +
Router guidance. Changed entry for examples/router.rs refresh from
raw-ratatui to envision PaneLayout chrome.

Final cadence in the May 2026 leadline brief queue.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

- [ ] Verify signature.

---

## Task 5: Push impl branch + open impl PR

### Step 1: Confirm branch state

- [ ] Run `git log --oneline -6` and confirm the branch has (in order, most recent first):
  - CHANGELOG entry commit
  - D8 commit
  - D7 commit
  - D3 commit
  - The 3 spec commits from the chained spec branch

If any commit is missing or misordered, STOP and reconcile before pushing.

### Step 2: Merge latest main into the impl branch

- [ ] Run:

```bash
git fetch origin main
git merge origin/main --no-ff -S -m "Merge origin/main into docs-suite-d3-d7-d8-impl"
```

If merge conflicts, resolve them (most likely candidates: CHANGELOG.md, recent changes to table/render.rs or table/mod.rs from concurrent landings).

If signing the merge commit fails, STOP and ask the user.

### Step 3: Push impl branch

- [ ] Run:

```bash
git push -u origin docs-suite-d3-d7-d8-impl
```

### Step 4: Open impl PR

- [ ] Open the PR with `gh pr create`:

```bash
gh pr create --title "Impl: docs suite D3+D7+D8 (May 2026 closure)" --body "$(cat <<'EOF'
## Summary

Implementation of the 10th and FINAL cadence in the May 2026 leadline brief queue. Three documentation-suite items shipped as three logically-separated signed commits:

- **D3** — `Column::new` canonical Length+Min docstring + render-time clipping warning. New `pub(crate)` `detect_clipped_columns` helper returning `Vec<ClippedColumn { idx, declared, resolved, kind: Length|Min }>`. Emission feature-gated on `tracing`; dedup via `RefCell<ClipWarnState>` on TableState (interior mutability required because `Table::view` takes `&state`); keyed by `(column index, area width)` with terminal-resize re-arm.
- **D7** — `src/harness` module docs gained "Choosing a Harness" decision table + per-harness blurbs for TestHarness / AppHarness / Runtime::virtual_builder. `src/harness/snapshot` module docs gained runnable canonical golden-file snapshot-diff recipe (dependency-free, `std::fs` + manual diff; `insta` linked as upgrade path).
- **D8** — New `examples/drilldown.rs` showing master+detail pattern using `TableState`, `PaneLayout::view_with`, `styled_line`, per-view `KeyHints`, `App::handle_event_with_state` for screen-gated key bindings, and selection preservation. Router docs gained Router-vs-in-state-enum guidance. `examples/router.rs` refreshed to use `PaneLayout::view_with` chrome instead of raw `ratatui::widgets::Paragraph + Block`.

After this PR + the tracking-doc PR merge, the May 2026 brief queue is closed.

## Spec / plan

- Spec: `docs/superpowers/specs/2026-05-24-docs-suite-d3-d7-d8-design.md` (PR #497)
- Plan: `docs/superpowers/plans/2026-05-24-docs-suite-d3-d7-d8.md`

## Lessons baked in from prior cadences

- **Doc-test coverage** (G4+G5 PR #487, G6 PR #491 regressions): new `Column::new` doc-test added; D3 helper is `pub(crate)` so no doc-test needed.
- **`cargo build --no-default-features`** (D5+D14 lesson): tracing-gated emission block has `#[cfg(not(feature = "tracing"))] let _ = clip;` to suppress unused-binding warnings.
- **File-size cap** (G4 `title_style.rs`, D12 `per_side_separators.rs`): `ClipWarnState` lives in sibling `src/component/table/clip_warn.rs` so `mod.rs` stays well under cap.
- **PartialEq exclusion**: `clip_warn_state` excluded from custom `PartialEq for TableState` impl, mirroring the existing `cross_variant_warned_cols` precedent.

## Test plan

- [x] D3: 7 detection helper tests + 4 dedup-state tests + 1 snapshot test
- [x] D7: doc-test for snapshot recipe runs end-to-end against tempdir
- [x] D8: `cargo build --examples --all-features` builds both `drilldown` and refreshed `router`
- [x] `cargo nextest run --all-features` — all pass
- [x] `cargo test --all-features --doc` — all pass
- [x] `cargo clippy --all-features -- -D warnings` — clean
- [x] `cargo fmt --check` — clean
- [x] `cargo build --no-default-features` — clean
- [x] `./tools/audit/target/release/envision-audit all` — 8/9 baseline preserved

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)" 2>&1 | tail -3
```

Expected: PR URL returned. Note the PR number for the tracking-doc PR's reference.

### Step 5: CI watch

- [ ] Run `gh pr checks <PR_NUMBER>` periodically until all required checks complete.

If any check fails:
- Read the failure log: `gh run view <RUN_ID> --log-failed`
- Diagnose the root cause
- Fix in a follow-up signed commit on the same branch
- Push

Do not attempt to merge until all required checks pass.

### Step 6: Merge after approval

- [ ] After leadline approves and all CI is green:

```bash
gh pr merge <PR_NUMBER> --squash --delete-branch
```

- [ ] After merge, the impl is complete. Next: tracking-doc PR (NOT part of this plan; opened on a separate branch after this PR lands).

---

## Out of scope for this plan

- Tracking-doc PR closure (separate branch + PR after impl merges)
- Any new harness public API (decision table + recipe are pure docs)
- New `Constraint` types or column resize semantics
- Replacing `insta` in envision's own tests
- Tracing-output capture in tests (matches existing `cross_variant_warned_cols` precedent — internal-only field, warn is observability not behavior)

## Recovery patterns from prior cadences

- **`git commit -S` fails** → STOP. Ask the user. Never bypass with `--no-gpg-sign`.
- **`cargo fmt --check` drifts mid-task** → Run `cargo fmt`, stage, add a small follow-up signed commit (e.g., `fmt: cargo fmt drift after Task N`). Don't amend.
- **Snapshot test fails on first run** → Inspect the `.snap.new` file. If output is sensible, accept with `cargo insta review` or manual rename. If output is corrupted (panics, layout misalignment that suggests real bugs), STOP and diagnose.
- **`detect_clipped_columns` test surfaces a stronger negative assertion** → Update the test to use the stronger form, but keep the snapshot as the load-bearing render-path coverage.
- **Audit baseline regresses** → Identify the new gap, add a doc-test or accessor as needed, re-run audit. Don't merge until 8/9 is restored.
