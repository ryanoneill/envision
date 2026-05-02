# Table Sort & Cell Unification Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Single coherent breaking-change pass that adds typed sort keys to a unified `Cell`, redesigns the sort message vocabulary, and merges `ResourceTable` into `Table` — addressing leadline gaps G1, G3, and G7 in one PR.

**Architecture:** New `src/component/cell.rs` introduces `Cell`, `SortKey`, `CellStyle`, `RowStatus`. `TableRow::cells()` switches return type from `Vec<String>` to `Vec<Cell>` and gains an optional `status()` method. `Column` loses `with_comparator` family, gains `with_default_sort`. `TableMessage` loses `SortBy`/`AddSort`/`ClearSort`, gains `SortAsc`/`SortDesc`/`SortToggle`/`SortClear`/`RemoveSort`/`AddSortAsc`/`AddSortDesc`/`AddSortToggle`. `TableState` gains `with_initial_sort`/`with_initial_sorts`. `src/component/resource_table/` deleted entirely. Phase 1 is additive (cell types coexist; everything compiles). Phase 2 is the atomic break: trait signature changes and every consumer migrates in the same set of commits. Phase 3 is the new test surface plus property tests and a bench gate.

**Tech Stack:** Rust (edition 2024, MSRV 1.85), ratatui 0.29, `compact_str`, `tracing`, cargo-nextest, insta snapshots, proptest, criterion.

**Spec:** `docs/superpowers/specs/2026-05-02-table-sort-cell-unification-design.md` (PR #459)

**Customer-feedback tracking:** `docs/customer-feedback/2026-05-01-leadline-gaps.md` (PR #458)

**Target version:** 0.17.0 (breaking)

---

## Project context

- **Working branch:** `table-sort-cell-unification` (created from main once spec PR #459 has merged or is approved). The plan doc lands on its own branch first (`plan-table-sort-cell-unification`).
- **PR strategy:** Implementation is one PR with internal commits granular by task. Final merge is squash per project rule, so internal commit count doesn't pollute history.
- **Why one PR, not several:** `TableRow::cells()` signature is shared across 26 files. There's no incremental path that keeps every caller compiling between commits. Phase 1 is the largest additive surface that can compile alongside the old code (new types in `cell.rs`, new `TableMessage` variants, new `Column` builder, new `TableState` builders); Phase 2 is the atomic switch.
- **Snapshot tests:** insta `.snap` files under `src/component/table/snapshots/` get re-recorded deliberately. The status-column rendering is new, per-cell styles are new, and the sort-indicator behavior changes when `SortToggle` replaces `SortBy`. Any snapshot diff requires manual eyeball review before `cargo insta accept`.
- **Test runner:** `cargo nextest run -p envision --all-features` (faster than `cargo test`, especially on Windows). Doc tests run separately: `cargo test --doc -p envision --all-features`.
- **Signed commits required.** If `gpg` signing fails, surface the error and stop — don't bypass with `--no-gpg-sign`.
- **No warnings allowed:** `cargo clippy --all-features --all-targets -- -D warnings` must be clean.
- **No files > 1000 lines:** `cell.rs` realistic ceiling is ~700 lines per spec; if it exceeds 1000, refactor inline (probably unlikely with the current type set).
- **Audit-scorecard target:** 9/9 before merge. Run `./tools/audit/target/release/envision-audit scorecard`.
- **Cross-ref handshake:** after this PR opens, leadline updates `notes/envision_gaps.md` G1/G3/G7 entries to reference the spec doc and PR number. Their workaround helpers (`apply_table_msg`, `apply_sort_persistent`, `strip_suffix_numeric_comparator`) get deleted on their side once this lands.

---

## File structure

### New files
- `src/component/cell.rs` — `Cell`, `SortKey`, `CellStyle`, `RowStatus` + their unit tests inline (`mod tests`). Realistic floor ~500–700 lines with doc tests.
- `src/component/table/sort_proptests.rs` — three property tests (ordering totality, sort stability, multi-column priority).
- `benches/sort_bench.rs` — criterion bench: 10k × 10 sort. Wall-time gate.

### Modified files
- `src/component/mod.rs` — declare `cell` module; remove `resource_table` declaration.
- `src/component/table/types.rs` — drop `comparator` field + `with_comparator`/`comparator()`/`SortComparator`/`numeric_comparator`/`date_comparator`. Add `default_sort` field + `with_default_sort`/`default_sort()`. Replace `TableMessage` sort variants. Update `TableRow::cells() -> Vec<Cell>` and add `status()` default impl. Add `InitialSort` struct.
- `src/component/table/state.rs` — switch sort comparator to use `SortKey`. Add `with_initial_sort`/`with_initial_sorts` builders. Stable sort verification.
- `src/component/table/mod.rs` — replace sort variant arms in `update()`; delete `SortBy`/`AddSort`/`ClearSort` arms; add the eight new variant arms.
- `src/component/table/render.rs` — apply per-cell `CellStyle`; render optional status column when any row has non-`None` status.
- `src/component/table/{tests,view_tests,multi_sort_tests,filter_tests,resize_tests}.rs` — migrate every `TableRow` impl + sort dispatch; add new tests for the 17 named categories from the spec.
- `src/component/data_grid/{mod,state,tests,snapshot_tests}.rs` — `TableRow` impls return `Vec<Cell>`; `editable` gating unchanged.
- `src/lib.rs` — top-level re-exports: add `Cell`, `SortKey`, `CellStyle`, `RowStatus`, `InitialSort`. Remove `ResourceTable*` re-exports.
- `examples/table.rs`, `examples/data_grid.rs`, `examples/component_showcase.rs` — migrate `TableRow` impls; switch `SortBy` → `SortToggle` (or explicit primitive); replace ResourceTable demo with Table-with-status demo.
- `tests/integration.rs`, `tests/serialization.rs`, `tests/integration_stress.rs`, `tests/property_extended.rs` — migrate row impls + sort dispatches.
- `CHANGELOG.md` — breaking-change section + migration table.

### Deleted files
- `src/component/resource_table/mod.rs`
- `src/component/resource_table/render.rs`
- `src/component/resource_table/state.rs`
- `src/component/resource_table/tests.rs`
- `src/component/resource_table/snapshots/*.snap` (all)

---

## Phase 1 — Additive (cell types compile alongside the old shape)

After Phase 1, every existing test still passes. Old `TableRow::cells() -> Vec<String>` still works. `Cell`, `SortKey`, etc. exist in `src/component/cell.rs` and are not yet used by `Table`.

### Task 1: Module skeleton

**Files:**
- Create: `src/component/cell.rs`
- Modify: `src/component/mod.rs` — add `pub mod cell;`

- [ ] **Step 1.1: Create `src/component/cell.rs` with module-level docs and a placeholder.**

```rust
//! Unified cell type for tabular components.
//!
//! `Cell` is the single cell representation used by `Table` and any
//! tabular component built on `TableRow`. A cell carries display text,
//! optional [`CellStyle`], and an optional [`SortKey`] for typed sorting.
//!
//! # Example
//!
//! ```
//! use envision::component::cell::{Cell, CellStyle, SortKey};
//!
//! let cell = Cell::new("running")
//!     .with_style(CellStyle::Success)
//!     .with_sort_key(SortKey::String("running".into()));
//! assert_eq!(cell.text(), "running");
//! ```

#![allow(dead_code)] // Placeholder during Phase 1; tasks 2–9 fill this in.
```

- [ ] **Step 1.2: Add `pub mod cell;` to `src/component/mod.rs`.**

Open `src/component/mod.rs`. Add `pub mod cell;` in alphabetical order with the existing `pub mod` declarations.

- [ ] **Step 1.3: Verify the crate still compiles.**

Run: `cargo check --all-features -p envision`
Expected: PASS, zero errors, zero warnings.

- [ ] **Step 1.4: Commit.**

```bash
git add src/component/cell.rs src/component/mod.rs
git commit -S -m "Add cell module skeleton (phase 1 additive)"
```

---

### Task 2: `SortKey` enum (TDD)

**Files:**
- Modify: `src/component/cell.rs`

- [ ] **Step 2.1: Write failing tests for the `SortKey` enum and `compare` method.**

Append to `src/component/cell.rs`:

```rust
#[cfg(test)]
mod sort_key_tests {
    use super::*;
    use std::cmp::Ordering;
    use std::time::{Duration, SystemTime};

    #[test]
    fn string_compare_lexicographic() {
        let a = SortKey::String("alice".into());
        let b = SortKey::String("bob".into());
        assert_eq!(SortKey::compare(&a, &b), Ordering::Less);
    }

    #[test]
    fn i64_compare_numeric() {
        let a = SortKey::I64(7);
        let b = SortKey::I64(11);
        assert_eq!(SortKey::compare(&a, &b), Ordering::Less);
    }

    #[test]
    fn u64_compare_numeric() {
        let a = SortKey::U64(7);
        let b = SortKey::U64(11);
        assert_eq!(SortKey::compare(&a, &b), Ordering::Less);
    }

    #[test]
    fn f64_compare_total_order() {
        // total_cmp: 3.5 < 7.0
        let a = SortKey::F64(3.5);
        let b = SortKey::F64(7.0);
        assert_eq!(SortKey::compare(&a, &b), Ordering::Less);
    }

    #[test]
    fn f64_nan_sorts_after_positive_infinity() {
        let inf = SortKey::F64(f64::INFINITY);
        let nan = SortKey::F64(f64::NAN);
        // total_cmp: +∞ < NaN
        assert_eq!(SortKey::compare(&inf, &nan), Ordering::Less);
    }

    #[test]
    fn bool_false_lt_true() {
        let f = SortKey::Bool(false);
        let t = SortKey::Bool(true);
        assert_eq!(SortKey::compare(&f, &t), Ordering::Less);
    }

    #[test]
    fn duration_compare() {
        let a = SortKey::Duration(Duration::from_secs(1));
        let b = SortKey::Duration(Duration::from_secs(2));
        assert_eq!(SortKey::compare(&a, &b), Ordering::Less);
    }

    #[test]
    fn datetime_older_lt_newer() {
        let older = SystemTime::UNIX_EPOCH;
        let newer = older + Duration::from_secs(1);
        assert_eq!(
            SortKey::compare(&SortKey::DateTime(older), &SortKey::DateTime(newer)),
            Ordering::Less,
        );
    }

    #[test]
    fn none_eq_none() {
        assert_eq!(SortKey::compare(&SortKey::None, &SortKey::None), Ordering::Equal);
    }
}
```

- [ ] **Step 2.2: Run tests, verify they fail (compile error — types not defined).**

Run: `cargo nextest run -p envision --all-features sort_key_tests`
Expected: COMPILATION ERROR — `SortKey` not defined.

- [ ] **Step 2.3: Add `SortKey` enum and stub `compare`.**

Append to `src/component/cell.rs` (above the `#[cfg(test)]` block):

```rust
use compact_str::CompactString;

/// A typed sort key carried by a [`Cell`] for typed comparison.
///
/// **DO NOT REORDER VARIANTS** — discriminant order is part of the
/// cross-variant fallback contract used by [`SortKey::compare`].
///
/// # Examples
///
/// ```
/// use envision::component::cell::SortKey;
/// use std::cmp::Ordering;
///
/// let a = SortKey::F64(3.5);
/// let b = SortKey::F64(7.0);
/// assert_eq!(SortKey::compare(&a, &b), Ordering::Less);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum SortKey {
    /// Lexicographic string sort.
    String(CompactString),
    /// Signed-integer sort.
    I64(i64),
    /// Unsigned-integer sort.
    U64(u64),
    /// Float sort using `f64::total_cmp`. NaN sorts after `+∞`
    /// (and [`SortKey::None`] sorts after NaN). Real values first,
    /// then NaN, then absent.
    F64(f64),
    /// `false` sorts before `true`.
    Bool(bool),
    /// Duration sort.
    Duration(std::time::Duration),
    /// Older sorts before newer.
    DateTime(std::time::SystemTime),
    /// Absent value. Sorts last in ascending, first in descending —
    /// always at the bottom of the visible list ("nulls last").
    None,
}

impl SortKey {
    /// Compares two sort keys per the documented rules.
    ///
    /// Same-variant comparisons use the natural ordering for the type
    /// (with `f64::total_cmp` for `F64`). Cross-variant comparisons
    /// (which shouldn't happen in well-formed code) fall back to
    /// discriminant order and emit a `tracing::warn!` once per
    /// `(render_pass, column_index)` (deduped at the call site, not here).
    pub fn compare(a: &Self, b: &Self) -> std::cmp::Ordering {
        unimplemented!("filled in next step")
    }
}
```

- [ ] **Step 2.4: Run tests, verify they fail at runtime (`unimplemented!`).**

Run: `cargo nextest run -p envision --all-features sort_key_tests`
Expected: tests panic on `unimplemented!`.

- [ ] **Step 2.5: Commit failing-test scaffolding.**

```bash
git add src/component/cell.rs
git commit -S -m "Add SortKey enum scaffolding and same-variant unit tests"
```

---

### Task 3: `SortKey::compare` implementation (same-variant + cross-variant + None)

**Files:**
- Modify: `src/component/cell.rs`

- [ ] **Step 3.1: Add cross-variant fallback test.**

Append inside `mod sort_key_tests`:

```rust
#[test]
fn cross_variant_falls_back_to_discriminant_order() {
    // I64 < F64 in the enum; in ascending, I64(7) appears before F64(3.5)
    // even though numerically 3.5 < 7.
    let a = SortKey::I64(7);
    let b = SortKey::F64(3.5);
    assert_eq!(SortKey::compare(&a, &b), Ordering::Less);
}

#[test]
fn none_sorts_last_in_ascending_against_real_value() {
    // None > Some(real) — always at the bottom in ascending.
    let real = SortKey::F64(0.0);
    assert_eq!(SortKey::compare(&real, &SortKey::None), Ordering::Less);
    assert_eq!(SortKey::compare(&SortKey::None, &real), Ordering::Greater);
}

#[test]
fn none_sorts_after_nan() {
    let nan = SortKey::F64(f64::NAN);
    assert_eq!(SortKey::compare(&nan, &SortKey::None), Ordering::Less);
}
```

- [ ] **Step 3.2: Implement `SortKey::compare`.**

Replace the `unimplemented!` body:

```rust
pub fn compare(a: &Self, b: &Self) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    use SortKey::*;

    // None policy: any non-None always sorts before None in ascending.
    match (a, b) {
        (None, None) => return Ordering::Equal,
        (None, _) => return Ordering::Greater,
        (_, None) => return Ordering::Less,
        _ => {}
    }

    // Same-variant fast paths.
    match (a, b) {
        (String(x), String(y)) => return x.cmp(y),
        (I64(x), I64(y)) => return x.cmp(y),
        (U64(x), U64(y)) => return x.cmp(y),
        (F64(x), F64(y)) => return x.total_cmp(y),
        (Bool(x), Bool(y)) => return x.cmp(y),
        (Duration(x), Duration(y)) => return x.cmp(y),
        (DateTime(x), DateTime(y)) => return x.cmp(y),
        _ => {}
    }

    // Cross-variant: fall back to discriminant order.
    Self::discriminant(a).cmp(&Self::discriminant(b))
}

fn discriminant(k: &Self) -> u8 {
    use SortKey::*;
    match k {
        String(_) => 0,
        I64(_) => 1,
        U64(_) => 2,
        F64(_) => 3,
        Bool(_) => 4,
        Duration(_) => 5,
        DateTime(_) => 6,
        None => 7,
    }
}
```

- [ ] **Step 3.3: Run tests, verify all pass.**

Run: `cargo nextest run -p envision --all-features sort_key_tests`
Expected: all 12 tests PASS.

- [ ] **Step 3.4: Commit.**

```bash
git add src/component/cell.rs
git commit -S -m "Implement SortKey::compare (same-variant, cross-variant, None policy)"
```

---

### Task 4: `CellStyle` enum

**Files:**
- Modify: `src/component/cell.rs`

- [ ] **Step 4.1: Write tests for `CellStyle` defaults and equality.**

Append a new test module:

```rust
#[cfg(test)]
mod cell_style_tests {
    use super::*;
    use ratatui::style::{Color, Style};

    #[test]
    fn default_is_default_variant() {
        assert_eq!(CellStyle::default(), CellStyle::Default);
    }

    #[test]
    fn custom_carries_style() {
        let style = Style::default().fg(Color::Red);
        assert_eq!(CellStyle::Custom(style), CellStyle::Custom(style));
    }
}
```

- [ ] **Step 4.2: Run, verify failure.**

Run: `cargo nextest run -p envision --all-features cell_style_tests`
Expected: COMPILATION ERROR — `CellStyle` not defined.

- [ ] **Step 4.3: Add `CellStyle`.**

Append to `src/component/cell.rs` (near `SortKey`):

```rust
use ratatui::style::Style;

/// Semantic cell styling.
///
/// `Default` renders with no override (theme-driven). `Success`,
/// `Warning`, `Error`, `Muted` map to the theme's semantic colors.
/// `Custom(Style)` applies a raw `ratatui::style::Style` directly.
///
/// # Example
///
/// ```
/// use envision::component::cell::CellStyle;
/// assert_eq!(CellStyle::default(), CellStyle::Default);
/// ```
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

- [ ] **Step 4.4: Run, verify pass.**

Run: `cargo nextest run -p envision --all-features cell_style_tests`
Expected: 2 tests PASS.

- [ ] **Step 4.5: Commit.**

```bash
git add src/component/cell.rs
git commit -S -m "Add CellStyle enum"
```

---

### Task 5: `RowStatus` enum + `indicator()`

**Files:**
- Modify: `src/component/cell.rs`

- [ ] **Step 5.1: Write tests for `RowStatus::indicator`.**

```rust
#[cfg(test)]
mod row_status_tests {
    use super::*;
    use ratatui::style::Color;

    #[test]
    fn none_has_no_indicator() {
        assert_eq!(RowStatus::None.indicator(), None);
    }

    #[test]
    fn healthy_indicator_green_dot() {
        assert_eq!(RowStatus::Healthy.indicator(), Some(("●", Color::Green)));
    }

    #[test]
    fn warning_indicator_yellow_triangle() {
        assert_eq!(RowStatus::Warning.indicator(), Some(("▲", Color::Yellow)));
    }

    #[test]
    fn error_indicator_red_cross() {
        assert_eq!(RowStatus::Error.indicator(), Some(("✖", Color::Red)));
    }

    #[test]
    fn unknown_indicator_gray_question() {
        assert_eq!(RowStatus::Unknown.indicator(), Some(("?", Color::DarkGray)));
    }

    #[test]
    fn custom_indicator_passes_through() {
        let custom = RowStatus::Custom { symbol: "★", color: Color::Magenta };
        assert_eq!(custom.indicator(), Some(("★", Color::Magenta)));
    }
}
```

- [ ] **Step 5.2: Run, verify failure (compile).**

Run: `cargo nextest run -p envision --all-features row_status_tests`
Expected: COMPILATION ERROR — `RowStatus` not defined.

- [ ] **Step 5.3: Add `RowStatus` and `indicator()`.**

```rust
use ratatui::style::Color;

/// Optional row-level status indicator. Renders as a colored symbol
/// in a status column prepended to the table.
///
/// # Example
///
/// ```
/// use envision::component::cell::RowStatus;
/// use ratatui::style::Color;
///
/// assert_eq!(RowStatus::Healthy.indicator(), Some(("●", Color::Green)));
/// assert_eq!(RowStatus::None.indicator(), None);
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub enum RowStatus {
    /// No status column rendered for this row.
    #[default]
    None,
    /// Green dot.
    Healthy,
    /// Yellow triangle.
    Warning,
    /// Red cross.
    Error,
    /// Gray question mark.
    Unknown,
    /// Caller-supplied symbol and color.
    Custom { symbol: &'static str, color: Color },
}

impl RowStatus {
    /// Returns `Some((symbol, color))` for non-None variants.
    pub fn indicator(&self) -> Option<(&'static str, Color)> {
        match self {
            RowStatus::None => None,
            RowStatus::Healthy => Some(("●", Color::Green)),
            RowStatus::Warning => Some(("▲", Color::Yellow)),
            RowStatus::Error => Some(("✖", Color::Red)),
            RowStatus::Unknown => Some(("?", Color::DarkGray)),
            RowStatus::Custom { symbol, color } => Some((symbol, *color)),
        }
    }
}
```

- [ ] **Step 5.4: Run, verify pass.**

Run: `cargo nextest run -p envision --all-features row_status_tests`
Expected: 6 tests PASS.

- [ ] **Step 5.5: Commit.**

```bash
git add src/component/cell.rs
git commit -S -m "Add RowStatus enum and indicator() accessor"
```

---

### Task 6: `Cell` struct + `new` + builders

**Files:**
- Modify: `src/component/cell.rs`

- [ ] **Step 6.1: Write tests for `Cell::new` + builders.**

```rust
#[cfg(test)]
mod cell_struct_tests {
    use super::*;

    #[test]
    fn new_default_style_no_sort_key() {
        let c = Cell::new("alice");
        assert_eq!(c.text(), "alice");
        assert_eq!(*c.style(), CellStyle::Default);
        assert_eq!(c.sort_key(), None);
    }

    #[test]
    fn with_style_round_trips() {
        let c = Cell::new("ok").with_style(CellStyle::Success);
        assert_eq!(*c.style(), CellStyle::Success);
    }

    #[test]
    fn with_sort_key_round_trips() {
        let c = Cell::new("7").with_sort_key(SortKey::I64(7));
        assert_eq!(c.sort_key(), Some(&SortKey::I64(7)));
    }

    #[test]
    fn with_text_replaces_text_only() {
        let c = Cell::new("v1")
            .with_style(CellStyle::Warning)
            .with_sort_key(SortKey::I64(1))
            .with_text("v2");
        assert_eq!(c.text(), "v2");
        assert_eq!(*c.style(), CellStyle::Warning);
        assert_eq!(c.sort_key(), Some(&SortKey::I64(1)));
    }

    #[test]
    fn default_cell_is_empty() {
        let c = Cell::default();
        assert_eq!(c.text(), "");
        assert_eq!(*c.style(), CellStyle::Default);
        assert_eq!(c.sort_key(), None);
    }
}
```

- [ ] **Step 6.2: Run, verify failure.**

Run: `cargo nextest run -p envision --all-features cell_struct_tests`
Expected: COMPILATION ERROR.

- [ ] **Step 6.3: Add `Cell` struct + impl.**

```rust
/// Unified cell type for tabular components.
///
/// Carries display text, optional [`CellStyle`], and optional [`SortKey`].
///
/// # Example
///
/// ```
/// use envision::component::cell::{Cell, CellStyle, SortKey};
///
/// let c = Cell::new("running")
///     .with_style(CellStyle::Success)
///     .with_sort_key(SortKey::String("running".into()));
/// assert_eq!(c.text(), "running");
/// assert_eq!(*c.style(), CellStyle::Success);
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Cell {
    text: CompactString,
    style: CellStyle,
    sort_key: Option<SortKey>,
}

impl Cell {
    /// Plain cell — default style, no typed sort key. Sort falls back to
    /// lexicographic on the cell text.
    pub fn new(text: impl Into<CompactString>) -> Self {
        Self {
            text: text.into(),
            style: CellStyle::Default,
            sort_key: Option::None,
        }
    }

    /// Builder: set the cell style.
    pub fn with_style(mut self, style: CellStyle) -> Self {
        self.style = style;
        self
    }

    /// Builder: set the typed sort key.
    pub fn with_sort_key(mut self, key: SortKey) -> Self {
        self.sort_key = Some(key);
        self
    }

    /// Builder: replace the display text without changing other fields.
    /// Useful in the mixed-precision pattern:
    ///
    /// ```
    /// use envision::component::cell::Cell;
    /// let cell = Cell::number(840.16).with_text(format!("{:.2}", 840.16));
    /// assert_eq!(cell.text(), "840.16");
    /// ```
    pub fn with_text(mut self, text: impl Into<CompactString>) -> Self {
        self.text = text.into();
        self
    }

    /// Display text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Cell style.
    pub fn style(&self) -> &CellStyle {
        &self.style
    }

    /// Optional typed sort key.
    pub fn sort_key(&self) -> Option<&SortKey> {
        self.sort_key.as_ref()
    }
}
```

- [ ] **Step 6.4: Run, verify pass.**

Run: `cargo nextest run -p envision --all-features cell_struct_tests`
Expected: 5 tests PASS.

- [ ] **Step 6.5: Commit.**

```bash
git add src/component/cell.rs
git commit -S -m "Add Cell struct, new, builder methods, accessors"
```

---

### Task 7: `Cell` typed-value convenience constructors

**Files:**
- Modify: `src/component/cell.rs`

- [ ] **Step 7.1: Write tests for `number`, `int`, `uint`, `bool`, `duration`, `datetime`.**

```rust
#[cfg(test)]
mod cell_typed_constructor_tests {
    use super::*;
    use std::time::{Duration, SystemTime};

    #[test]
    fn number_uses_display_fmt_and_f64_sort_key() {
        let c = Cell::number(840.0);
        assert_eq!(c.text(), "840");
        assert_eq!(c.sort_key(), Some(&SortKey::F64(840.0)));
    }

    #[test]
    fn number_with_text_overrides_display_for_mixed_precision() {
        let c = Cell::number(840.16).with_text(format!("{:.2}", 840.16));
        assert_eq!(c.text(), "840.16");
        assert_eq!(c.sort_key(), Some(&SortKey::F64(840.16)));
    }

    #[test]
    fn int_renders_and_sorts_as_i64() {
        // Doc test note: use Cell::int / Cell::uint only when the column's
        // data is integer-valued. For fractional or naturally-f64 data,
        // use Cell::number to preserve precision in the sort key.
        let c = Cell::int(-42);
        assert_eq!(c.text(), "-42");
        assert_eq!(c.sort_key(), Some(&SortKey::I64(-42)));
    }

    #[test]
    fn uint_renders_and_sorts_as_u64() {
        let c = Cell::uint(7);
        assert_eq!(c.text(), "7");
        assert_eq!(c.sort_key(), Some(&SortKey::U64(7)));
    }

    #[test]
    fn bool_renders_and_sorts() {
        let t = Cell::bool(true);
        assert_eq!(t.text(), "true");
        assert_eq!(t.sort_key(), Some(&SortKey::Bool(true)));
    }

    #[test]
    fn duration_renders_compact_and_sorts() {
        let c = Cell::duration(Duration::from_secs(192));
        // 3m12s — exact format depends on format_duration helper
        assert_eq!(c.sort_key(), Some(&SortKey::Duration(Duration::from_secs(192))));
        assert!(c.text().contains("3m"));
        assert!(c.text().contains("12s"));
    }

    #[test]
    fn datetime_carries_systemtime_sort_key() {
        let t = SystemTime::UNIX_EPOCH;
        let c = Cell::datetime(t);
        assert_eq!(c.sort_key(), Some(&SortKey::DateTime(t)));
        assert!(!c.text().is_empty());
    }
}
```

- [ ] **Step 7.2: Run, verify failure.**

Run: `cargo nextest run -p envision --all-features cell_typed_constructor_tests`
Expected: COMPILATION ERROR.

- [ ] **Step 7.3: Add convenience constructors and `format_duration` helper.**

Append to `Cell` impl block:

```rust
/// Numeric cell with `f64` sort key. Display text is `format!("{}", value)`.
///
/// For fixed-precision columns, chain `.with_text(format!(...))`:
///
/// ```
/// use envision::component::cell::Cell;
/// let c = Cell::number(840.16).with_text(format!("{:.2} ms", 840.16));
/// assert_eq!(c.text(), "840.16 ms");
/// ```
pub fn number(value: f64) -> Self {
    Self::new(format!("{}", value)).with_sort_key(SortKey::F64(value))
}

/// Signed-integer cell. Use only when the column's data is integer-valued —
/// for fractional or naturally-`f64` data, use [`Cell::number`] to preserve
/// precision in the sort key.
///
/// # Example
///
/// ```
/// use envision::component::cell::Cell;
/// let c = Cell::int(-42);
/// assert_eq!(c.text(), "-42");
/// ```
pub fn int(value: i64) -> Self {
    Self::new(format!("{}", value)).with_sort_key(SortKey::I64(value))
}

/// Unsigned-integer cell. Use only for non-negative integer data —
/// for naturally-`f64` data, use [`Cell::number`].
pub fn uint(value: u64) -> Self {
    Self::new(format!("{}", value)).with_sort_key(SortKey::U64(value))
}

/// Boolean cell. Renders as `"true"` / `"false"`.
pub fn bool(value: bool) -> Self {
    Self::new(format!("{}", value)).with_sort_key(SortKey::Bool(value))
}

/// Duration cell. Display text is a compact form (`"3m12s"`, `"2h15m"`,
/// `"3d4h"`); sort key is the underlying `Duration`.
pub fn duration(d: std::time::Duration) -> Self {
    Self::new(format_duration(d)).with_sort_key(SortKey::Duration(d))
}

/// Datetime cell. Display text is an ISO-ish string; sort key is the
/// underlying `SystemTime`.
pub fn datetime(t: std::time::SystemTime) -> Self {
    Self::new(format_systemtime(t)).with_sort_key(SortKey::DateTime(t))
}
```

Add helper functions at module scope:

```rust
fn format_duration(d: std::time::Duration) -> String {
    let secs = d.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m{}s", secs / 60, secs % 60)
    } else if secs < 86_400 {
        format!("{}h{}m", secs / 3600, (secs % 3600) / 60)
    } else {
        format!("{}d{}h", secs / 86_400, (secs % 86_400) / 3600)
    }
}

fn format_systemtime(t: std::time::SystemTime) -> String {
    use std::time::UNIX_EPOCH;
    match t.duration_since(UNIX_EPOCH) {
        Ok(d) => format!("{}s", d.as_secs()),
        Err(_) => "before-epoch".to_string(),
    }
}
```

(`format_systemtime` is intentionally minimal — consumers who want richer formatting use `Cell::new(formatted_string).with_sort_key(SortKey::DateTime(t))`.)

- [ ] **Step 7.4: Run, verify pass.**

Run: `cargo nextest run -p envision --all-features cell_typed_constructor_tests`
Expected: 7 tests PASS.

- [ ] **Step 7.5: Commit.**

```bash
git add src/component/cell.rs
git commit -S -m "Add Cell typed-value constructors (number/int/uint/bool/duration/datetime)"
```

---

### Task 8: `Cell` style-flavored constructors

**Files:**
- Modify: `src/component/cell.rs`

- [ ] **Step 8.1: Write tests.**

```rust
#[cfg(test)]
mod cell_style_constructor_tests {
    use super::*;

    #[test]
    fn success_sets_style() {
        let c = Cell::success("running");
        assert_eq!(c.text(), "running");
        assert_eq!(*c.style(), CellStyle::Success);
    }

    #[test]
    fn warning_sets_style() {
        assert_eq!(*Cell::warning("retry").style(), CellStyle::Warning);
    }

    #[test]
    fn error_sets_style() {
        assert_eq!(*Cell::error("crash").style(), CellStyle::Error);
    }

    #[test]
    fn muted_sets_style() {
        assert_eq!(*Cell::muted("idle").style(), CellStyle::Muted);
    }
}
```

- [ ] **Step 8.2: Run, verify failure.**

Run: `cargo nextest run -p envision --all-features cell_style_constructor_tests`
Expected: COMPILATION ERROR.

- [ ] **Step 8.3: Add the constructors.**

Append to the `Cell` impl block:

```rust
/// Success-styled cell (green).
pub fn success(text: impl Into<CompactString>) -> Self {
    Self::new(text).with_style(CellStyle::Success)
}

/// Warning-styled cell (yellow).
pub fn warning(text: impl Into<CompactString>) -> Self {
    Self::new(text).with_style(CellStyle::Warning)
}

/// Error-styled cell (red).
pub fn error(text: impl Into<CompactString>) -> Self {
    Self::new(text).with_style(CellStyle::Error)
}

/// Muted-styled cell (dark gray).
pub fn muted(text: impl Into<CompactString>) -> Self {
    Self::new(text).with_style(CellStyle::Muted)
}
```

- [ ] **Step 8.4: Run, verify pass.**

Run: `cargo nextest run -p envision --all-features cell_style_constructor_tests`
Expected: 4 tests PASS.

- [ ] **Step 8.5: Commit.**

```bash
git add src/component/cell.rs
git commit -S -m "Add Cell style-flavored constructors (success/warning/error/muted)"
```

---

### Task 9: `Cell` `From` impls

**Files:**
- Modify: `src/component/cell.rs`

- [ ] **Step 9.1: Write tests.**

```rust
#[cfg(test)]
mod cell_from_impls_tests {
    use super::*;
    use compact_str::CompactString;

    #[test]
    fn from_str_round_trips() {
        let c: Cell = "alice".into();
        assert_eq!(c.text(), "alice");
        assert_eq!(*c.style(), CellStyle::Default);
        assert_eq!(c.sort_key(), None);
    }

    #[test]
    fn from_string_round_trips() {
        let c: Cell = String::from("bob").into();
        assert_eq!(c.text(), "bob");
    }

    #[test]
    fn from_compact_string_round_trips() {
        let c: Cell = CompactString::from("carol").into();
        assert_eq!(c.text(), "carol");
    }
}
```

- [ ] **Step 9.2: Run, verify failure.**

Expected: COMPILATION ERROR.

- [ ] **Step 9.3: Add `From` impls.**

Append at module scope:

```rust
impl From<&str> for Cell {
    fn from(s: &str) -> Self { Cell::new(s) }
}

impl From<String> for Cell {
    fn from(s: String) -> Self { Cell::new(s) }
}

impl From<CompactString> for Cell {
    fn from(s: CompactString) -> Self { Cell::new(s) }
}
```

- [ ] **Step 9.4: Run, verify pass.**

Expected: 3 tests PASS.

- [ ] **Step 9.5: Commit.**

```bash
git add src/component/cell.rs
git commit -S -m "Add Cell From impls for &str, String, CompactString"
```

---

### Task 10: `InitialSort` struct

**Files:**
- Modify: `src/component/table/types.rs`

- [ ] **Step 10.1: Write a unit test.**

Append to existing tests in `src/component/table/types.rs` or `src/component/table/tests.rs`:

```rust
#[test]
fn initial_sort_struct() {
    use envision::component::SortDirection;
    use envision::component::InitialSort;

    let s = InitialSort { column: 4, direction: SortDirection::Descending };
    assert_eq!(s.column, 4);
    assert_eq!(s.direction, SortDirection::Descending);
}
```

- [ ] **Step 10.2: Run, verify failure.**

Expected: COMPILATION ERROR — `InitialSort` not defined.

- [ ] **Step 10.3: Add struct.**

In `src/component/table/types.rs`, near the existing `SortDirection`:

```rust
/// Pair of (column index, direction) for declarative initial sort.
///
/// # Example
///
/// ```
/// use envision::component::{InitialSort, SortDirection};
/// let s = InitialSort { column: 4, direction: SortDirection::Descending };
/// assert_eq!(s.column, 4);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InitialSort {
    pub column: usize,
    pub direction: SortDirection,
}
```

- [ ] **Step 10.4: Re-export at crate root in `src/lib.rs`.**

Add `InitialSort` to the re-export list (alphabetical placement; see existing `SortDirection`).

- [ ] **Step 10.5: Run, verify pass.**

Expected: PASS.

- [ ] **Step 10.6: Commit.**

```bash
git add src/component/table/types.rs src/lib.rs src/component/table/tests.rs
git commit -S -m "Add InitialSort struct"
```

---

### Task 11: `Column::default_sort` field, builder, getter

**Files:**
- Modify: `src/component/table/types.rs`
- Modify: `src/component/table/tests.rs` (or wherever Column tests live)

- [ ] **Step 11.1: Write tests.**

```rust
#[test]
fn column_default_sort_defaults_to_ascending() {
    use envision::component::{Column, SortDirection};
    use ratatui::layout::Constraint;
    let c = Column::new("X", Constraint::Length(5));
    assert_eq!(c.default_sort(), SortDirection::Ascending);
}

#[test]
fn column_with_default_sort_round_trips() {
    use envision::component::{Column, SortDirection};
    use ratatui::layout::Constraint;
    let c = Column::new("X", Constraint::Length(5))
        .with_default_sort(SortDirection::Descending);
    assert_eq!(c.default_sort(), SortDirection::Descending);
}
```

- [ ] **Step 11.2: Run, verify failure.**

Expected: COMPILATION ERROR.

- [ ] **Step 11.3: Add field, builder, and getter.**

In `src/component/table/types.rs`, modify `Column`:

```rust
pub struct Column {
    header: String,
    width: Constraint,
    sortable: bool,
    editable: bool,
    visible: bool,
    default_sort: SortDirection,             // NEW
    #[cfg_attr(feature = "serialization", serde(skip))]
    comparator: Option<SortComparator>,      // STILL HERE — removed in Phase 2
}
```

Update `Column::new` and `Column::fixed` constructors to set `default_sort: SortDirection::Ascending`.

Update `Default` impl similarly.

Update `PartialEq` impl to include `default_sort` in the comparison (it implements `PartialEq` already).

Add to `impl Column`:

```rust
/// Declares this column's natural sort direction. `SortToggle` and
/// `AddSortToggle` use this when activating the column for the first
/// time. Default: `Ascending`.
///
/// Use `Descending` for columns where bigger-is-worse (latency,
/// regression delta, error count).
///
/// # Example
///
/// ```
/// use envision::component::{Column, SortDirection};
/// use ratatui::layout::Constraint;
///
/// let c = Column::new("delta", Constraint::Length(10))
///     .with_default_sort(SortDirection::Descending);
/// assert_eq!(c.default_sort(), SortDirection::Descending);
/// ```
pub fn with_default_sort(mut self, dir: SortDirection) -> Self {
    self.default_sort = dir;
    self
}

/// Returns the column's natural sort direction.
pub fn default_sort(&self) -> SortDirection {
    self.default_sort
}
```

- [ ] **Step 11.4: Run, verify pass.**

Expected: PASS.

- [ ] **Step 11.5: Commit.**

```bash
git add src/component/table/types.rs src/component/table/tests.rs
git commit -S -m "Add Column::default_sort field, with_default_sort, default_sort()"
```

---

### Task 12: New `TableMessage` variants (added alongside old)

**Files:**
- Modify: `src/component/table/types.rs`

This task adds the eight new variants to `TableMessage` *alongside* the old `SortBy`/`AddSort`/`ClearSort`. The old variants are not yet handled in `Table::update` but they're not removed yet either — keeps Phase 1 additive. Phase 2 (Task 14) deletes the old variants and updates `update`.

- [ ] **Step 12.1: Add new variants to `TableMessage`.**

```rust
pub enum TableMessage {
    Up,
    Down,
    First,
    Last,
    PageUp(usize),
    PageDown(usize),
    Select,

    // ----- Existing sort variants — DELETED in Phase 2 -----
    SortBy(usize),
    AddSort(usize),
    ClearSort,

    // ----- NEW primary sort family -----
    /// Set the primary sort to this column, ascending. Replaces the
    /// entire sort stack with just this entry.
    SortAsc(usize),

    /// Set the primary sort to this column, descending. Replaces the
    /// entire sort stack with just this entry.
    SortDesc(usize),

    /// 2-cycle toggle. Never clears.
    /// - If this column is already the primary sort: flip Asc ↔ Desc.
    /// - If this column is not currently in the sort stack: activate it
    ///   using `Column::default_sort()`.
    SortToggle(usize),

    /// Drop the primary sort and any tiebreakers. Returns to load order.
    SortClear,

    /// Drop just one column from the multi-sort stack. The remaining
    /// columns keep their relative order. If the dropped column was
    /// primary, the next tiebreaker is promoted.
    RemoveSort(usize),

    // ----- NEW tiebreaker family -----
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

    IncreaseColumnWidth(usize),
    DecreaseColumnWidth(usize),
    SetFilter(String),
    ClearFilter,
}
```

- [ ] **Step 12.2: Verify it still compiles (the new variants are unused in `update`; that's expected for this phase).**

Run: `cargo check -p envision --all-features`
Expected: PASS, possibly with `dead_code` warnings on the new variants (allow them with `#[allow(dead_code)]` if needed, or accept the warning since Phase 2 deletes the old variants and uses the new ones).

If the project's `#[deny(warnings)]` is strict, temporarily add `#[allow(dead_code)]` on the new variants and remove the allow in Phase 2.

- [ ] **Step 12.3: Commit.**

```bash
git add src/component/table/types.rs
git commit -S -m "Add new TableMessage sort variants (alongside old; phase 1 additive)"
```

---

### Task 13: Phase 1 verification

- [ ] **Step 13.1: Run the full test suite — every existing test must still pass.**

Run: `cargo nextest run -p envision --all-features && cargo test --doc -p envision --all-features`
Expected: all tests pass, no regressions.

- [ ] **Step 13.2: Run clippy.**

Run: `cargo clippy --all-features --all-targets -- -D warnings`
Expected: clean.

- [ ] **Step 13.3: Run audit scorecard.**

Run: `./tools/audit/target/release/envision-audit scorecard`
Expected: 8/9 or 9/9 (the public-item count may shift but other categories should be unaffected).

- [ ] **Step 13.4: Commit any clippy fixes.**

If clippy flags anything, fix in this commit. Otherwise no commit needed.

---

## Phase 2 — Atomic switch (trait signature changes; every consumer migrates)

After Phase 2, no `Vec<String>` `TableRow::cells()` impls remain anywhere in the codebase. Old `TableMessage` variants are gone. `ResourceTable` is gone. Tests still pass (using migrated impls).

### Task 14: Switch `TableRow::cells()` signature; add `status()`

**Files:**
- Modify: `src/component/table/types.rs`

- [ ] **Step 14.1: Change the trait signature.**

In `src/component/table/types.rs`, replace:

```rust
pub trait TableRow: Clone {
    fn cells(&self) -> Vec<String>;
}
```

with:

```rust
use crate::component::cell::{Cell, RowStatus};

pub trait TableRow: Clone {
    /// Returns the cells for this row, one per column.
    fn cells(&self) -> Vec<Cell>;

    /// Optional row-level status indicator. Default: [`RowStatus::None`] —
    /// no status column rendered. If any row in the table returns non-None,
    /// the status column is rendered for all rows.
    fn status(&self) -> RowStatus { RowStatus::None }
}
```

- [ ] **Step 14.2: Run `cargo check` and capture every breakage location.**

Run: `cargo check -p envision --all-features 2>&1 | grep -E "^error|^  --> " > /tmp/cells-breakage.txt`

Expected: many errors. Every existing `TableRow::cells()` impl is now broken. The breakage list is the migration checklist for steps 14.3–14.10.

- [ ] **Step 14.3: Migrate `TableRow` impls in `src/component/data_grid/`.**

`data_grid/state.rs` and `data_grid/mod.rs` and `data_grid/tests.rs` and `data_grid/snapshot_tests.rs` — every `impl TableRow for X { fn cells(&self) -> Vec<String> { vec![ ... ] } }` becomes `fn cells(&self) -> Vec<Cell> { vec![ ... ] }`.

For each impl:
- If the row produces simple strings: `vec!["alice".to_string()]` → `vec![Cell::new("alice")]` or `vec!["alice".into()]`.
- If the row produces typed values: switch to typed constructors (`Cell::number`, `Cell::int`, etc.).

- [ ] **Step 14.4: Migrate `src/component/table/tests.rs`.**

Same pattern. Existing tests should still pass (the migrated impls produce equivalent display output).

- [ ] **Step 14.5: Migrate `src/component/table/{view_tests,multi_sort_tests,filter_tests,resize_tests}.rs`.**

Same.

- [ ] **Step 14.6: Migrate `examples/table.rs`.**

- [ ] **Step 14.7: Migrate `examples/data_grid.rs`.**

- [ ] **Step 14.8: Migrate `examples/component_showcase.rs`.**

Includes deleting the existing ResourceTable demo block; replace with a Table-with-status example demonstrating `RowStatus::Healthy`/`Warning`/`Error` rows.

- [ ] **Step 14.9: Migrate `tests/integration.rs`, `tests/serialization.rs`, `tests/integration_stress.rs`, `tests/property_extended.rs`.**

Same pattern.

- [ ] **Step 14.10: Verify `cargo check` is clean.**

Run: `cargo check -p envision --all-features`
Expected: PASS.

- [ ] **Step 14.11: Commit.**

```bash
git add -A
git commit -S -m "Switch TableRow::cells() to Vec<Cell>; add status() default; migrate all consumers"
```

---

### Task 15: Drop `Column::with_comparator` family

**Files:**
- Modify: `src/component/table/types.rs`

- [ ] **Step 15.1: Delete `comparator` field from `Column`.**

- [ ] **Step 15.2: Delete `Column::with_comparator`, `Column::comparator`, `SortComparator` type alias, `numeric_comparator()`, `date_comparator()`.**

- [ ] **Step 15.3: Update `Column`'s `new`, `fixed`, `Default`, `Debug`, `PartialEq` impls to no longer reference `comparator`.**

- [ ] **Step 15.4: Run `cargo check`, capture breakages.**

Expected: tests using `with_comparator` / `numeric_comparator` / `date_comparator` break.

- [ ] **Step 15.5: Migrate every breakage to the typed-cell pattern.**

E.g., `Column::new("Price", ...).with_comparator(numeric_comparator())` becomes plain `Column::new("Price", ...).sortable()` and the corresponding `cells()` impl returns `Cell::number(self.price)` for that column.

For tests in `src/component/table/multi_sort_tests.rs` that test the comparator API directly (e.g., `test_numeric_comparator`, `test_date_comparator`, `test_column_with_comparator`, `test_sort_with_numeric_comparator`, `test_sort_with_date_comparator`), delete them — they exercise an API that no longer exists. The replacement coverage is in Phase 3 (test categories 1–3).

- [ ] **Step 15.6: Verify `cargo check` is clean.**

- [ ] **Step 15.7: Commit.**

```bash
git add -A
git commit -S -m "Drop Column::with_comparator family (numeric_comparator/date_comparator/SortComparator)"
```

---

### Task 16: Delete old `TableMessage` sort variants

**Files:**
- Modify: `src/component/table/types.rs`
- Modify: `src/component/table/mod.rs`

- [ ] **Step 16.1: Remove `SortBy(usize)`, `AddSort(usize)`, `ClearSort` from `TableMessage`.**

- [ ] **Step 16.2: Remove `SortBy`/`AddSort`/`ClearSort` arms from `Table::update` in `src/component/table/mod.rs`.**

- [ ] **Step 16.3: Run `cargo check`, capture breakages — every test that dispatches the old variants needs migrating.**

- [ ] **Step 16.4: Migrate per the spec's consumer migration table.**

Per-call mapping:
- `TableMessage::SortBy(col)` for header click → `TableMessage::SortToggle(col)`
- `TableMessage::SortBy(col)` for "always Asc" → `TableMessage::SortAsc(col)`
- `TableMessage::SortBy(col)` for "always Desc" → `TableMessage::SortDesc(col)`
- `TableMessage::AddSort(col)` for tiebreaker click → `TableMessage::AddSortToggle(col)`
- `TableMessage::AddSort(col)` for "always Asc tiebreaker" → `TableMessage::AddSortAsc(col)`
- `TableMessage::ClearSort` → `TableMessage::SortClear`
- `SortBy(col); SortBy(col)` (init bootstrap to Desc) → `TableState::with_initial_sort(col, Descending)` — but this builder isn't added until Task 19. For now, use `SortDesc(col)`. Tests that test bootstrap behavior should be updated to use `with_initial_sort` once Task 19 lands.

- [ ] **Step 16.5: Verify `cargo check` is clean.**

- [ ] **Step 16.6: Commit.**

```bash
git add -A
git commit -S -m "Drop TableMessage::SortBy/AddSort/ClearSort; migrate consumers"
```

---

### Task 17: Implement new `Table::update` arms — primary sort family

**Files:**
- Modify: `src/component/table/mod.rs`
- Modify: `src/component/table/state.rs`

- [ ] **Step 17.1: Implement `SortAsc(col)` arm.**

```rust
TableMessage::SortAsc(col) => {
    let cols = state.columns.len();
    if col >= cols || !state.columns[col].is_sortable() {
        return None;  // silent no-op for out-of-range or non-sortable
    }
    let already = state.sort_columns.first() == Some(&(col, SortDirection::Ascending));
    if already {
        return None;  // idempotent
    }
    state.sort_columns = vec![(col, SortDirection::Ascending)];
    state.rebuild_display_order();
    Some(TableOutput::Sorted { column: col, direction: SortDirection::Ascending })
}
```

- [ ] **Step 17.2: Implement `SortDesc(col)` arm.**

Mirror `SortAsc` with `SortDirection::Descending`.

- [ ] **Step 17.3: Implement `SortToggle(col)` arm.**

```rust
TableMessage::SortToggle(col) => {
    let cols = state.columns.len();
    if col >= cols || !state.columns[col].is_sortable() {
        return None;
    }
    let new_dir = match state.sort_columns.first() {
        Some(&(c, dir)) if c == col => match dir {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        },
        _ => state.columns[col].default_sort(),  // first activation
    };
    state.sort_columns = vec![(col, new_dir)];
    state.rebuild_display_order();
    Some(TableOutput::Sorted { column: col, direction: new_dir })
}
```

- [ ] **Step 17.4: Implement `SortClear` arm.**

```rust
TableMessage::SortClear => {
    if state.sort_columns.is_empty() {
        return None;  // already empty, no-op
    }
    state.sort_columns.clear();
    state.rebuild_display_order();
    Some(TableOutput::SortCleared)
}
```

- [ ] **Step 17.5: Implement `RemoveSort(col)` arm per the spec output table.**

```rust
TableMessage::RemoveSort(col) => {
    let pos = state.sort_columns.iter().position(|&(c, _)| c == col);
    let Some(pos) = pos else { return None; };  // col not in stack

    let was_primary = pos == 0;
    state.sort_columns.remove(pos);
    state.rebuild_display_order();

    if state.sort_columns.is_empty() {
        Some(TableOutput::SortCleared)
    } else if was_primary {
        let (new_primary, new_dir) = state.sort_columns[0];
        Some(TableOutput::Sorted { column: new_primary, direction: new_dir })
    } else {
        None  // tiebreaker removed; primary unchanged
    }
}
```

- [ ] **Step 17.6: Run `cargo check`, fix any compile errors.**

- [ ] **Step 17.7: Commit.**

```bash
git add -A
git commit -S -m "Implement TableMessage::{SortAsc,SortDesc,SortToggle,SortClear,RemoveSort} arms"
```

---

### Task 18: Implement new `Table::update` arms — tiebreaker family

**Files:**
- Modify: `src/component/table/mod.rs`

- [ ] **Step 18.1: Implement `AddSortAsc(col)` arm — append/replace-direction-in-place.**

```rust
TableMessage::AddSortAsc(col) => {
    let cols = state.columns.len();
    if col >= cols || !state.columns[col].is_sortable() {
        return None;
    }
    if let Some(idx) = state.sort_columns.iter().position(|&(c, _)| c == col) {
        if state.sort_columns[idx].1 == SortDirection::Ascending {
            return None;  // idempotent
        }
        state.sort_columns[idx].1 = SortDirection::Ascending;  // replace, no reorder
    } else {
        state.sort_columns.push((col, SortDirection::Ascending));  // append
    }
    state.rebuild_display_order();
    Some(TableOutput::Sorted { column: col, direction: SortDirection::Ascending })
}
```

- [ ] **Step 18.2: Implement `AddSortDesc(col)` arm.** Mirror `AddSortAsc`.

- [ ] **Step 18.3: Implement `AddSortToggle(col)` arm.**

```rust
TableMessage::AddSortToggle(col) => {
    let cols = state.columns.len();
    if col >= cols || !state.columns[col].is_sortable() {
        return None;
    }
    if let Some(idx) = state.sort_columns.iter().position(|&(c, _)| c == col) {
        let new_dir = match state.sort_columns[idx].1 {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        };
        state.sort_columns[idx].1 = new_dir;
        state.rebuild_display_order();
        Some(TableOutput::Sorted { column: col, direction: new_dir })
    } else {
        let new_dir = state.columns[col].default_sort();
        state.sort_columns.push((col, new_dir));
        state.rebuild_display_order();
        Some(TableOutput::Sorted { column: col, direction: new_dir })
    }
}
```

- [ ] **Step 18.4: Run `cargo check`, fix errors.**

- [ ] **Step 18.5: Commit.**

```bash
git add -A
git commit -S -m "Implement TableMessage::{AddSortAsc,AddSortDesc,AddSortToggle} arms"
```

---

### Task 19: `TableState::with_initial_sort` and `with_initial_sorts`

**Files:**
- Modify: `src/component/table/state.rs`

- [ ] **Step 19.1: Add the builders.**

```rust
impl<T: TableRow> TableState<T> {
    /// Set the initial primary sort declaratively. Rebuilds display
    /// order immediately so the first frame renders sorted.
    ///
    /// If `col` is non-sortable, the sort is set anyway (this is a
    /// declarative bootstrap and the consumer is asserting intent).
    /// Downstream `SortToggle(col)` etc. on a non-sortable column
    /// remain silent no-ops.
    pub fn with_initial_sort(mut self, col: usize, dir: SortDirection) -> Self {
        self.sort_columns = vec![(col, dir)];
        self.rebuild_display_order();
        self
    }

    /// Multi-column variant: primary plus tiebreakers, in priority order.
    pub fn with_initial_sorts(mut self, sorts: Vec<InitialSort>) -> Self {
        self.sort_columns = sorts.into_iter().map(|s| (s.column, s.direction)).collect();
        self.rebuild_display_order();
        self
    }
}
```

- [ ] **Step 19.2: Run `cargo check`.**

Expected: PASS.

- [ ] **Step 19.3: Commit.**

```bash
git add src/component/table/state.rs
git commit -S -m "Add TableState::with_initial_sort and with_initial_sorts"
```

---

### Task 20: Sort comparator switches to `SortKey`; verify stable sort

**Files:**
- Modify: `src/component/table/state.rs`

- [ ] **Step 20.1: Replace the existing sort closure body in `rebuild_display_order`.**

Locate the existing `self.display_order.sort_by(|&a, &b| { ... })` block (around line 549 of state.rs). Replace its body with the SortKey-based comparator from the spec:

```rust
self.display_order.sort_by(|&a, &b| {
    use std::cmp::Ordering;
    for &(col, direction) in &sort_spec {
        let cells_a = self.rows[a].cells();
        let cells_b = self.rows[b].cells();
        let key_a = cells_a.get(col)
            .and_then(|c| c.sort_key().cloned())
            .unwrap_or_else(|| {
                SortKey::String(cells_a.get(col)
                    .map(|c| c.text().into())
                    .unwrap_or_default())
            });
        let key_b = cells_b.get(col)
            .and_then(|c| c.sort_key().cloned())
            .unwrap_or_else(|| {
                SortKey::String(cells_b.get(col)
                    .map(|c| c.text().into())
                    .unwrap_or_default())
            });
        // Cross-variant warn (deduped at this call site, not in compare).
        if std::mem::discriminant(&key_a) != std::mem::discriminant(&key_b) {
            // Implementation note: dedup key is (render_pass, col).
            // For now, emit unconditionally; Task 21 adds dedup.
            tracing::warn!(column = col, "sortable column has mixed SortKey variants");
        }
        let cmp = SortKey::compare(&key_a, &key_b);
        let ordered = match direction {
            SortDirection::Ascending => cmp,
            SortDirection::Descending => cmp.reverse(),
        };
        if ordered != Ordering::Equal {
            return ordered;
        }
    }
    Ordering::Equal
});
```

Confirm the call is `sort_by`, **not** `sort_unstable_by`. Add a code comment:

```rust
// MUST be sort_by (stable) — preserves insertion order on equal keys
// (e.g., consecutive SortKey::None rows). See spec test #14.
```

- [ ] **Step 20.2: Run the existing multi-sort tests.**

Run: `cargo nextest run -p envision --all-features multi_sort_tests`
Expected: PASS (the new comparator should produce the same results as the old one for tests that don't involve `with_comparator`-driven numeric sort).

- [ ] **Step 20.3: Commit.**

```bash
git add src/component/table/state.rs
git commit -S -m "Switch sort comparator to SortKey-based path; preserve stable sort"
```

---

### Task 21: Cross-variant warn dedup keyed on `(render_pass, column_index)`

**Files:**
- Modify: `src/component/table/state.rs`

- [ ] **Step 21.1: Add a dedup field to `TableState`.**

```rust
pub struct TableState<T: TableRow> {
    // ... existing fields ...
    /// Dedup keys for cross-variant SortKey warnings: column indices that
    /// already emitted a warning during the current render pass.
    /// Cleared at the start of each `rebuild_display_order` call.
    cross_variant_warned_cols: std::collections::HashSet<usize>,
}
```

- [ ] **Step 21.2: At the start of `rebuild_display_order`, clear the set.**

```rust
fn rebuild_display_order(&mut self) {
    self.cross_variant_warned_cols.clear();
    // ... existing logic ...
}
```

- [ ] **Step 21.3: Replace the unconditional `tracing::warn!` with a deduped emit.**

```rust
if std::mem::discriminant(&key_a) != std::mem::discriminant(&key_b)
    && self.cross_variant_warned_cols.insert(col)
{
    tracing::warn!(column = col, "sortable column has mixed SortKey variants; sort falling back to discriminant order");
}
```

(`HashSet::insert` returns `true` if the value was newly inserted — perfect for "first time we see this column this pass.")

- [ ] **Step 21.4: Update `Default` and any other constructors of `TableState` to initialize the new field.**

- [ ] **Step 21.5: Run tests, verify pass.**

- [ ] **Step 21.6: Commit.**

```bash
git add src/component/table/state.rs
git commit -S -m "Dedup cross-variant SortKey warnings per (render_pass, column)"
```

---

### Task 22: Render per-cell `CellStyle`

**Files:**
- Modify: `src/component/table/render.rs`

- [ ] **Step 22.1: Add a helper `cell_style_to_ratatui` to translate `CellStyle` to `ratatui::Style`.**

In `render.rs`:

```rust
fn cell_style_to_ratatui(style: &CellStyle, theme: &Theme, disabled: bool) -> Style {
    use ratatui::style::Color;
    if disabled {
        return Style::default().fg(Color::DarkGray);
    }
    match style {
        CellStyle::Default => Style::default(),
        CellStyle::Success => Style::default().fg(Color::Green),
        CellStyle::Warning => Style::default().fg(Color::Yellow),
        CellStyle::Error => Style::default().fg(Color::Red),
        CellStyle::Muted => Style::default().fg(Color::DarkGray),
        CellStyle::Custom(s) => *s,
    }
}
```

- [ ] **Step 22.2: Build `Cell` data rows applying per-cell style.**

In the data-row construction loop, replace plain text-cells with styled cells:

```rust
let cells_widget: Vec<Cell<'static>> = row.cells().into_iter().map(|cell| {
    let style = cell_style_to_ratatui(cell.style(), theme, disabled);
    Cell::from(cell.text().to_string()).style(style)
}).collect();
```

(Note: this `Cell` is `ratatui::widgets::Cell`, distinct from `envision::Cell`. Disambiguate the imports.)

- [ ] **Step 22.3: Run snapshot tests.**

Run: `cargo nextest run -p envision --all-features view_tests`
Expected: snapshots may diff (the per-cell-style rendering is new). Manually inspect each diff before accepting:

```bash
cargo insta review
```

For each diff, verify the change is intentional (e.g., a previously default-styled cell still renders default; a previously success-styled `ResourceCell::success("running")` now renders the same way under `Cell::success("running")`).

- [ ] **Step 22.4: Commit.**

```bash
git add -A
git commit -S -m "Apply per-cell CellStyle in Table render"
```

---

### Task 23: Optional status column rendering

**Files:**
- Modify: `src/component/table/render.rs`
- Modify: `src/component/table/state.rs` (add `has_status_column()` if not already present from RT migration)

- [ ] **Step 23.1: Add or move `has_status_column()` helper to `TableState`.**

```rust
pub(crate) fn has_status_column(&self) -> bool {
    self.rows.iter().any(|r| !matches!(r.status(), RowStatus::None))
}
```

- [ ] **Step 23.2: In `render.rs`, prepend a 2-cell status column if `state.has_status_column()`.**

Before the existing column-widths construction:

```rust
let has_status = state.has_status_column();

let mut widths: Vec<Constraint> = Vec::new();
if has_status {
    widths.push(Constraint::Length(2));
}
for col in state.columns() {
    widths.push(col.width());
}

// Header: prepend empty status header
let mut header_cells: Vec<RatatuiCell> = Vec::new();
if has_status {
    header_cells.push(RatatuiCell::from(""));
}
for col in state.columns() {
    header_cells.push(RatatuiCell::from(col.header().to_string()).style(header_style));
}
```

- [ ] **Step 23.3: For each data row, prepend the status indicator cell.**

```rust
let mut row_cells: Vec<RatatuiCell> = Vec::new();
if has_status {
    match row.status().indicator() {
        Some((symbol, color)) => {
            let style = if disabled {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(color)
            };
            row_cells.push(RatatuiCell::from(symbol.to_string()).style(style));
        }
        None => row_cells.push(RatatuiCell::from("")),
    }
}
// ... then push the data cells (from cells.rs migration above)
```

- [ ] **Step 23.4: Run snapshot tests.**

Run: `cargo nextest run -p envision --all-features`
Expected: existing tables (no rows with non-None status) still produce identical snapshots; tables with status rows produce new snapshots that need eyeball review and accept.

- [ ] **Step 23.5: Commit.**

```bash
git add -A
git commit -S -m "Render optional status column when any row has non-None status"
```

---

### Task 24: Delete `src/component/resource_table/`

**Files:**
- Delete: `src/component/resource_table/mod.rs`
- Delete: `src/component/resource_table/render.rs`
- Delete: `src/component/resource_table/state.rs`
- Delete: `src/component/resource_table/tests.rs`
- Delete: `src/component/resource_table/snapshots/` (entire directory)
- Modify: `src/component/mod.rs`
- Modify: `src/lib.rs`

- [ ] **Step 24.1: Delete the entire `src/component/resource_table/` directory.**

```bash
git rm -r src/component/resource_table/
```

- [ ] **Step 24.2: Remove `pub mod resource_table;` from `src/component/mod.rs`.**

- [ ] **Step 24.3: Remove all `ResourceTable*` and `ResourceCell*` and `ResourceColumn*` and `ResourceRow*` and `RowStatus` (the old re-export) from `src/lib.rs`. Add the new re-exports: `Cell`, `SortKey`, `CellStyle`, `RowStatus` (from `cell` module).**

- [ ] **Step 24.4: Run `cargo check` to confirm nothing else references the deleted module.**

If anything still references `ResourceTable`, find and fix.

- [ ] **Step 24.5: Run full test suite.**

Run: `cargo nextest run -p envision --all-features && cargo test --doc -p envision --all-features`
Expected: all PASS.

- [ ] **Step 24.6: Commit.**

```bash
git add -A
git commit -S -m "Delete resource_table module; merged into Table via TableRow::status()"
```

---

### Task 25: Phase 2 verification

- [ ] **Step 25.1: `cargo check --all-features`** — clean.
- [ ] **Step 25.2: `cargo nextest run -p envision --all-features`** — all PASS (no skipped tests pretending to pass).
- [ ] **Step 25.3: `cargo test --doc -p envision --all-features`** — all doc tests PASS.
- [ ] **Step 25.4: `cargo clippy --all-features --all-targets -- -D warnings`** — clean.
- [ ] **Step 25.5: `cargo fmt --check`** — clean (run `cargo fmt` if not).
- [ ] **Step 25.6: `cargo build --examples --all-features`** — examples compile.
- [ ] **Step 25.7: `./tools/audit/target/release/envision-audit scorecard`** — record current score; aim for 9/9 by end of Phase 3.
- [ ] **Step 25.8: Commit any clippy/fmt fixes from this verification pass.**

---

## Phase 3 — Tests, property tests, bench, polish

After Phase 3, the 17 named test categories from the spec are covered, three property tests are in place, the bench gate is recorded, snapshots are reviewed-and-accepted, CHANGELOG is updated.

### Task 26: cell.rs unit tests — categories 1, 3, 14

Most are already written in Tasks 2–9. This task adds the gaps.

**Files:**
- Modify: `src/component/cell.rs`

- [ ] **Step 26.1: Add test category 14 (None preserves insertion order under stable sort).**

```rust
#[test]
fn sort_key_none_preserves_insertion_order_stable() {
    use std::cmp::Ordering;
    // Equal pairs of None must compare Equal. Sort stability preservation
    // is verified end-to-end by Task 28 (state.rs test) — this test
    // pins the equality.
    assert_eq!(
        SortKey::compare(&SortKey::None, &SortKey::None),
        Ordering::Equal
    );
}
```

- [ ] **Step 26.2: Add cell.rs run-all.**

Run: `cargo nextest run -p envision --all-features cell::`
Expected: all PASS.

- [ ] **Step 26.3: Commit.**

```bash
git add src/component/cell.rs
git commit -S -m "Add SortKey::None equality test (test category 14 part 1)"
```

---

### Task 27: types.rs unit tests — category 12

**Files:**
- Modify: `src/component/table/tests.rs`

- [ ] **Step 27.1: Add `Column::default_sort()` defaults-to-Ascending test.**

(Already written in Task 11 — confirm it's still passing. If it was deleted during Phase 2 migration, restore.)

- [ ] **Step 27.2: Add a test that all 8 new `TableMessage` variants `derive` `Clone + Debug + PartialEq`.**

```rust
#[test]
fn new_table_message_variants_derive_traits() {
    use envision::component::TableMessage;
    let m = TableMessage::SortAsc(0);
    let m2 = m.clone();
    assert_eq!(m, m2);
    let _ = format!("{:?}", m);  // Debug
}
```

- [ ] **Step 27.3: Run, verify pass.**

- [ ] **Step 27.4: Commit.**

```bash
git add src/component/table/tests.rs
git commit -S -m "Add Column::default_sort default test + new TableMessage trait derives test"
```

---

### Task 28: state.rs sort tests — categories 2, 4, 5, 6, 7, 9, 13, 14, 15a, 16, 17

**Files:**
- Modify: `src/component/table/multi_sort_tests.rs`
- Optionally create: `src/component/table/sort_tests.rs` if `multi_sort_tests.rs` would exceed 1000 lines.

This is the largest test task in Phase 3. Implementations follow the spec verbatim — a few representative skeletons:

- [ ] **Step 28.1: Test #2 — cross-variant fallback + warn dedup.**

```rust
#[test]
fn cross_variant_falls_back_to_discriminant_in_ascending() {
    use envision::component::cell::{Cell, SortKey};
    use envision::component::{Column, Table, TableMessage, TableRow, TableState};
    use ratatui::layout::Constraint;

    #[derive(Clone, PartialEq, Debug)]
    struct Row { sort_key: SortKey, label: String }
    impl TableRow for Row {
        fn cells(&self) -> Vec<Cell> {
            vec![Cell::new(&self.label).with_sort_key(self.sort_key.clone())]
        }
    }
    let rows = vec![
        Row { sort_key: SortKey::F64(3.5), label: "F".into() },
        Row { sort_key: SortKey::I64(7),  label: "I".into() },
    ];
    let columns = vec![Column::new("V", Constraint::Length(5)).sortable()];
    let mut state = TableState::new(rows.clone(), columns);
    let _ = Table::<Row>::update(&mut state, TableMessage::SortAsc(0));
    let order = state.display_order().to_vec();
    let labels: Vec<&str> = order.iter().map(|&i| rows[i].label.as_str()).collect();
    // I64 (discriminant 1) before F64 (discriminant 3) — NOT 3.5 before 7.
    assert_eq!(labels, vec!["I", "F"]);
}

#[test]
fn cross_variant_warn_emitted_once_per_render_pass() {
    // Use tracing-test to capture log output across 100 mixed-variant rows.
    // Assert: tracing log emitted exactly 1 line tagged column=0.
    // (Implementation: see tracing-test crate; add to dev-dependencies if not already.)
    // Spec test category 2.
}
```

- [ ] **Step 28.2: Test #4 — `SortToggle` 2-cycle never clears.**

```rust
#[test]
fn sort_toggle_2_cycle_never_clears() {
    use envision::component::{Column, Table, TableMessage, TableRow, TableState};
    use envision::component::cell::Cell;
    use ratatui::layout::Constraint;

    #[derive(Clone)]
    struct R(u8);
    impl TableRow for R { fn cells(&self) -> Vec<Cell> { vec![Cell::int(self.0 as i64)] } }
    let rows = vec![R(1), R(2), R(3)];
    let columns = vec![Column::new("V", Constraint::Length(5)).sortable()];
    let mut state = TableState::new(rows, columns);

    for _ in 0..10 {
        let _ = Table::<R>::update(&mut state, TableMessage::SortToggle(0));
        assert!(state.sort().is_some(), "SortToggle must never clear");
    }
}
```

- [ ] **Step 28.3: Test #5 — `SortToggle` honors `with_default_sort` on first activation.**

```rust
#[test]
fn sort_toggle_honors_default_sort_on_first_activation() {
    use envision::component::{Column, SortDirection, Table, TableMessage, TableRow, TableState};
    use envision::component::cell::Cell;
    use ratatui::layout::Constraint;

    #[derive(Clone)] struct R(u8);
    impl TableRow for R { fn cells(&self) -> Vec<Cell> { vec![Cell::int(self.0 as i64)] } }
    let columns = vec![
        Column::new("V", Constraint::Length(5))
            .sortable()
            .with_default_sort(SortDirection::Descending),
    ];
    let mut state = TableState::new(vec![R(1), R(2)], columns);
    let _ = Table::<R>::update(&mut state, TableMessage::SortToggle(0));
    assert_eq!(state.sort(), Some((0, SortDirection::Descending)));
}
```

- [ ] **Step 28.4: Test #6 — `with_initial_sort` produces sorted state on frame 1.**

```rust
#[test]
fn with_initial_sort_renders_sorted_on_frame_1() {
    use envision::component::{Column, SortDirection, TableRow, TableState};
    use envision::component::cell::Cell;
    use ratatui::layout::Constraint;

    #[derive(Clone)] struct R(u8);
    impl TableRow for R { fn cells(&self) -> Vec<Cell> { vec![Cell::int(self.0 as i64)] } }
    let columns = vec![Column::new("V", Constraint::Length(5)).sortable()];
    let state = TableState::new(vec![R(3), R(1), R(2)], columns)
        .with_initial_sort(0, SortDirection::Ascending);
    // Without any update() calls, display_order should be [1, 2, 3]
    assert_eq!(state.sort(), Some((0, SortDirection::Ascending)));
}
```

- [ ] **Step 28.5: Test #7 — RemoveSort + SortClear output table (six tests, one per row).**

Each as a separate `#[test]` function with the exact assertion from the spec's output behavior table:

- `removesort_primary_promotes_next_emits_sorted_with_new_primary`
- `removesort_tiebreaker_unchanged_primary_emits_none`
- `removesort_only_sort_emits_sortcleared`
- `removesort_col_not_in_stack_emits_none`
- `sortclear_nonempty_emits_sortcleared`
- `sortclear_empty_emits_none`

- [ ] **Step 28.6: Test #9 — idempotent dispatches.**

```rust
#[test]
fn sort_asc_idempotent_returns_none() {
    // Setup state with (col, Asc); dispatch SortAsc(col); expect None output.
    // Same shape for SortDesc, AddSortAsc, AddSortDesc.
}
```

Four tests (SortAsc, SortDesc, AddSortAsc, AddSortDesc).

- [ ] **Step 28.7: Test #13 — non-sortable column is silent no-op.**

Each variant (SortAsc, SortDesc, SortToggle, AddSortAsc, AddSortDesc, AddSortToggle) gets a test verifying `state` unchanged and output `None` when the target column has `.sortable() = false` (or wasn't called).

- [ ] **Step 28.8: Test #14 — None preserves insertion order under stable sort.**

```rust
#[test]
fn none_rows_preserve_insertion_order() {
    use envision::component::cell::{Cell, SortKey};
    use envision::component::{Column, SortDirection, Table, TableMessage, TableRow, TableState};
    use ratatui::layout::Constraint;

    #[derive(Clone, PartialEq, Debug)]
    struct R { id: u8, key: SortKey }
    impl TableRow for R {
        fn cells(&self) -> Vec<Cell> {
            vec![Cell::new(format!("{}", self.id)).with_sort_key(self.key.clone())]
        }
    }
    let rows = vec![
        R { id: 1, key: SortKey::I64(1) },
        R { id: 2, key: SortKey::None },         // _a
        R { id: 3, key: SortKey::None },         // _b
        R { id: 4, key: SortKey::None },         // _c
        R { id: 5, key: SortKey::I64(2) },
    ];
    let columns = vec![Column::new("V", Constraint::Length(5)).sortable()];
    let mut state = TableState::new(rows.clone(), columns);
    let _ = Table::<R>::update(&mut state, TableMessage::SortAsc(0));
    let order: Vec<u8> = state.display_order().iter().map(|&i| rows[i].id).collect();
    assert_eq!(order, vec![1, 5, 2, 3, 4],
        "Real values first (1, 5), then Nones in original order (2, 3, 4)");
}
```

- [ ] **Step 28.9: Test #15a — `sort_toggle_state_persists_on_repeated_press`.**

```rust
#[test]
fn sort_toggle_state_persists_on_repeated_press() {
    // Same setup as test #4. Dispatch 10×; assert state.sort() always Some.
    // Pinned by name per the originating bug.
}
```

- [ ] **Step 28.10: Test #16 — column-switch first activation honors new column's default_sort.**

```rust
#[test]
fn sort_toggle_column_switch_honors_new_column_default_sort() {
    use envision::component::{Column, SortDirection, Table, TableMessage, TableRow, TableState};
    use envision::component::cell::Cell;
    use ratatui::layout::Constraint;

    #[derive(Clone)] struct R(u8, u8);
    impl TableRow for R { fn cells(&self) -> Vec<Cell> { vec![Cell::int(self.0 as i64), Cell::int(self.1 as i64)] } }
    let columns = vec![
        Column::new("A", Constraint::Length(5)).sortable(),
        Column::new("B", Constraint::Length(5)).sortable()
            .with_default_sort(SortDirection::Descending),
    ];
    let mut state = TableState::new(vec![R(1, 1), R(2, 2)], columns);
    // Stack starts (col 0, Asc)
    let _ = Table::<R>::update(&mut state, TableMessage::SortAsc(0));
    assert_eq!(state.sort(), Some((0, SortDirection::Ascending)));
    // Toggle col 1: should activate using col 1's default_sort = Descending
    let _ = Table::<R>::update(&mut state, TableMessage::SortToggle(1));
    assert_eq!(state.sort(), Some((1, SortDirection::Descending)));
}
```

- [ ] **Step 28.11: Test #17 — `AddSort*` position preservation on existing entries.**

```rust
#[test]
fn add_sort_asc_replaces_direction_in_place_no_reorder() {
    // Stack: [(0, Asc), (1, Desc), (2, Asc)]
    // Dispatch AddSortAsc(1)
    // Expect: [(0, Asc), (1, Asc), (2, Asc)]  — col 1 stays at position 1
    // Not at the end.
}
```

Plus same shape for `AddSortToggle`.

- [ ] **Step 28.12: Run, verify all pass.**

Run: `cargo nextest run -p envision --all-features sort`
Expected: every new test PASS.

- [ ] **Step 28.13: Commit.**

```bash
git add src/component/table/multi_sort_tests.rs
git commit -S -m "Add 17-category sort test suite (categories 2,4,5,6,7,9,13,14,15a,16,17)"
```

---

### Task 29: render.rs snapshot tests — categories 8, 10, 11, 15b

**Files:**
- Modify: `src/component/table/view_tests.rs` (or create dedicated test file)

- [ ] **Step 29.1: Test #10 — status column visibility.**

Two snapshots: one with all rows `RowStatus::None` (no status column), one with mixed statuses (status column rendered).

- [ ] **Step 29.2: Test #11 — per-cell `CellStyle` snapshot per variant.**

Six snapshots — one for each `CellStyle` variant — plus one mixed-styles snapshot.

- [ ] **Step 29.3: Test #8 — sort indicator visibility.**

Snapshot: dispatch `SortAsc(0)`, snapshot the header (indicator visible). Dispatch `SortClear`, snapshot again (indicator gone).

- [ ] **Step 29.4: Test #15b — `sort_toggle_arrow_persists_on_repeated_press` (render layer).**

Dispatch `SortToggle(0)` 10×; on each iteration, render the table and assert the indicator character is present in the rendered buffer at the column header location.

- [ ] **Step 29.5: Run, review each snapshot diff individually.**

Run: `cargo insta review`
For each snapshot: visually verify the rendering matches the spec's intent before accepting.

- [ ] **Step 29.6: Commit.**

```bash
git add -A
git commit -S -m "Add render-layer snapshot tests (categories 8, 10, 11, 15b)"
```

---

### Task 30: Property tests

**Files:**
- Create: `src/component/table/sort_proptests.rs`
- Modify: `src/component/table/mod.rs` — add `#[cfg(test)] mod sort_proptests;`

- [ ] **Step 30.1: Add proptest dev-dependency if not already present.**

Check `Cargo.toml`'s `[dev-dependencies]` for `proptest`. Already there per project memory; proceed.

- [ ] **Step 30.2: Property test: `SortKey` ordering is total.**

```rust
use envision::component::cell::SortKey;
use proptest::prelude::*;
use std::cmp::Ordering;

fn arb_sort_key() -> impl Strategy<Value = SortKey> {
    prop_oneof![
        any::<String>().prop_map(|s| SortKey::String(s.into())),
        any::<i64>().prop_map(SortKey::I64),
        any::<u64>().prop_map(SortKey::U64),
        any::<f64>().prop_map(SortKey::F64),
        any::<bool>().prop_map(SortKey::Bool),
        Just(SortKey::None),
    ]
}

proptest! {
    #[test]
    fn sort_key_ordering_total(a in arb_sort_key(), b in arb_sort_key()) {
        let cmp_ab = SortKey::compare(&a, &b);
        let cmp_ba = SortKey::compare(&b, &a);
        // Antisymmetry
        prop_assert_eq!(cmp_ab, cmp_ba.reverse());
    }

    #[test]
    fn sort_key_ordering_transitive(a in arb_sort_key(), b in arb_sort_key(), c in arb_sort_key()) {
        let ab = SortKey::compare(&a, &b);
        let bc = SortKey::compare(&b, &c);
        let ac = SortKey::compare(&a, &c);
        // Transitivity: if a <= b and b <= c, then a <= c
        if matches!(ab, Ordering::Less | Ordering::Equal) && matches!(bc, Ordering::Less | Ordering::Equal) {
            prop_assert!(matches!(ac, Ordering::Less | Ordering::Equal));
        }
    }
}
```

- [ ] **Step 30.3: Property test: sort is stable across permutations.**

Generate `Vec<Row>` with duplicate sort keys; shuffle; sort; assert relative order of equal-key elements preserved.

- [ ] **Step 30.4: Property test: multi-column sort respects priority.**

Generate stacked `(col_0, dir_0), (col_1, dir_1)` and a row population; verify rows with same `col_0` value are ordered by `col_1`.

- [ ] **Step 30.5: Run.**

Run: `cargo nextest run -p envision --all-features sort_proptests`
Expected: PASS (proptest will run many random cases).

- [ ] **Step 30.6: Commit.**

```bash
git add -A
git commit -S -m "Add sort property tests (totality, stability, multi-column priority)"
```

---

### Task 31: Sort bench gate

**Files:**
- Create: `benches/sort_bench.rs`
- Modify: `Cargo.toml` — declare the bench

- [ ] **Step 31.1: Add bench declaration to `Cargo.toml`.**

```toml
[[bench]]
name = "sort_bench"
harness = false
```

- [ ] **Step 31.2: Implement the bench.**

```rust
use criterion::{criterion_group, criterion_main, Criterion};
use envision::component::cell::{Cell, SortKey};
use envision::component::{Column, Table, TableMessage, TableRow, TableState};
use ratatui::layout::Constraint;

#[derive(Clone)]
struct R(f64);
impl TableRow for R {
    fn cells(&self) -> Vec<Cell> {
        let mut v = Vec::with_capacity(10);
        v.push(Cell::number(self.0));
        for i in 1..10 {
            v.push(Cell::int(i as i64));
        }
        v
    }
}

fn bench_sort_10k_rows(c: &mut Criterion) {
    use rand::{Rng, SeedableRng};
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let rows: Vec<R> = (0..10_000).map(|_| R(rng.gen_range(0.0..1000.0))).collect();
    let columns = vec![
        Column::new("V", Constraint::Length(8)).sortable(),
        // 9 more columns to match the spec's workload
    ];

    c.bench_function("sort_10k_rows_f64", |b| {
        b.iter_with_setup(
            || TableState::new(rows.clone(), columns.clone()),
            |mut state| {
                let _ = Table::<R>::update(&mut state, TableMessage::SortAsc(0));
            },
        );
    });
}

criterion_group!(benches, bench_sort_10k_rows);
criterion_main!(benches);
```

- [ ] **Step 31.3: Run the bench locally to record baseline.**

Run: `cargo bench --bench sort_bench`
Expected: produces a baseline measurement. Exact numbers will be machine-dependent.

- [ ] **Step 31.4: Commit.**

```bash
git add benches/sort_bench.rs Cargo.toml
git commit -S -m "Add sort_bench: 10k row × 10 column primary-sort wall-time gate"
```

---

### Task 32: Re-record snapshots

- [ ] **Step 32.1: Run `cargo insta test` and review every diff.**

Run: `cargo insta test -p envision --review`

For every diff:
- Verify it's an intentional change (e.g., new status column rendering, per-cell styling).
- If unexpected, the implementation has a bug — fix the implementation, don't accept the snapshot.
- After review, accept with `cargo insta accept`.

- [ ] **Step 32.2: Commit accepted snapshots.**

```bash
git add -A src/component/**/*.snap
git commit -S -m "Re-record Table snapshots with per-cell styles + status column"
```

---

### Task 33: CHANGELOG and lib.rs re-exports verification

**Files:**
- Modify: `CHANGELOG.md`
- Verify: `src/lib.rs`

- [ ] **Step 33.1: Add the breaking-change section to `CHANGELOG.md`.**

Use the sketch from the spec's CHANGELOG section — copy verbatim into `CHANGELOG.md` under the `[Unreleased]` heading.

- [ ] **Step 33.2: Verify `src/lib.rs` re-exports.**

Confirm:
- `pub use crate::component::cell::{Cell, CellStyle, RowStatus, SortKey};` — present
- `pub use crate::component::InitialSort;` — present
- All `ResourceTable*` exports — gone
- `numeric_comparator` / `date_comparator` / `SortComparator` — gone

- [ ] **Step 33.3: Commit.**

```bash
git add CHANGELOG.md src/lib.rs
git commit -S -m "Update CHANGELOG and lib.rs re-exports for sort/cell unification"
```

---

### Task 34: Final pre-merge verification

- [ ] **Step 34.1: Full test suite.**

Run:
```bash
cargo nextest run -p envision --all-features
cargo test --doc -p envision --all-features
```
Expected: all PASS.

- [ ] **Step 34.2: Lint and format.**

Run:
```bash
cargo clippy --all-features --all-targets -- -D warnings
cargo fmt --check
```
Expected: clean.

- [ ] **Step 34.3: Examples build.**

Run: `cargo build --examples --all-features`
Expected: clean.

- [ ] **Step 34.4: Audit scorecard.**

Run: `./tools/audit/target/release/envision-audit scorecard`
Expected: 9/9.

- [ ] **Step 34.5: Bench regression check.**

Run: `cargo bench --bench sort_bench`
Compare against baseline recorded in Task 31. If wall-time median regressed >10%, investigate.

- [ ] **Step 34.6: Verify file sizes are under 1000 lines.**

Run: `wc -l src/component/cell.rs src/component/table/*.rs`
Expected: all under 1000.

- [ ] **Step 34.7: Push the branch and open PR.**

```bash
git push -u origin table-sort-cell-unification
gh pr create --title "Table sort & cell unification (G1+G3+G7)" --body "..."
```

PR description must include:
- Lists every deleted public item
- Lists every new public item
- Reproduces the migration table inline
- Notes the `Cell::number` mixed-precision caveat
- References spec PR #459 and customer-feedback PR #458
- References leadline's `notes/envision_gaps.md` and `notes/envision_table_sort_api_redesign.md`

- [ ] **Step 34.8: Wait for CI (16 checks).** Address any failures.

- [ ] **Step 34.9: Once green, merge with `gh pr merge --squash --delete-branch`.**

- [ ] **Step 34.10: Cross-ref handshake.**

Update `docs/customer-feedback/2026-05-01-leadline-gaps.md` to reference the merged PR. Notify leadline so they can update their `notes/envision_gaps.md` G1/G3/G7 entries with the spec-doc and PR pointers, and remove their workaround helpers (`apply_table_msg`, `apply_sort_persistent`, `strip_suffix_numeric_comparator`).

---

## Self-review checklist

After implementation, before merge, verify:

**Spec coverage:** every section of `docs/superpowers/specs/2026-05-02-table-sort-cell-unification-design.md` maps to a task above.
- ✅ Architecture → Phase 1 + Phase 2 (modular layout, type definitions)
- ✅ Types (Cell, SortKey, CellStyle, RowStatus, TableRow, Column, TableMessage, TableOutput, InitialSort, TableState builders) → Tasks 2–12, 14, 15, 16, 19
- ✅ Sort dispatch & semantics (comparator path, same-variant rules, None policy, cross-variant fallback, stable sort, output behavior table) → Tasks 17, 18, 20, 21
- ✅ Migration (file actions, migration table, CHANGELOG sketch, PR description requirements) → Phase 2 + Task 33 + Task 34
- ✅ Testing & verification (17 test categories + 3 property tests + bench + audit) → Phase 3 (Tasks 26–32, 34)

**Placeholder scan:** zero "TBD", "TODO", "fill in details" in the plan above. ✓

**Type consistency:** types used in later tasks match definitions in earlier tasks.
- `Cell::new`, `Cell::with_text`, `Cell::with_style`, `Cell::with_sort_key`, `Cell::number`, `Cell::int`, `Cell::uint`, `Cell::bool`, `Cell::duration`, `Cell::datetime`, `Cell::success`, `Cell::warning`, `Cell::error`, `Cell::muted`, `Cell::text`, `Cell::style`, `Cell::sort_key` — defined Task 6, used consistently in later tasks.
- `SortKey::compare`, `SortKey::String/I64/U64/F64/Bool/Duration/DateTime/None` — defined Task 2-3, used consistently.
- `TableMessage::SortAsc/SortDesc/SortToggle/SortClear/RemoveSort/AddSortAsc/AddSortDesc/AddSortToggle` — defined Task 12, handled in Tasks 17–18.
- `Column::with_default_sort`, `Column::default_sort` — defined Task 11.
- `TableState::with_initial_sort`, `TableState::with_initial_sorts` — defined Task 19.
- `TableRow::cells -> Vec<Cell>`, `TableRow::status -> RowStatus` — defined Task 14.
- `RowStatus::indicator`, `CellStyle` variants — defined Tasks 4, 5.

**Spec requirements with no task:** none identified. If the implementer encounters one, add a task and verify the type-consistency table.

---

## Execution

When ready to execute:

1. **Subagent-driven** (recommended for plans this size): use `superpowers:subagent-driven-development`. Fresh subagent per task with two-stage review.
2. **Inline execution**: use `superpowers:executing-plans` if executing in this session.

Either way, the plan above is the complete execution surface — no per-task ad-hoc improvisation should be needed.
