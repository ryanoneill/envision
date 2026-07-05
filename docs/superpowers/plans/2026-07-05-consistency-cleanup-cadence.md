# Consistency-cleanup cadence (v0.17.0 pre-release, Cadence A) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close two trust-eroding findings from the 2026-07-05 audit before cutting v0.17.0 — the `selected_value` / `selected_item` / `active_tab` accessor divergence across six components (Unit 1) and the `tokio::sync::mpsc::Sender` dep-leakage at the AppHarness surface (Unit 2). All ten breaking renames land in one impl PR with three logically-separated commits + verification + push.

**Architecture:** Three independent code-shaped units bundled into one impl PR:
- Unit 1 (selected_item consistency sweep): six components — dropdown, select, heatmap, tab_bar, data_grid, table — get their alias accessors deleted or semantically distinct accessors renamed to `selected_item()` / `value_at_selection()`. Internal component-body call sites migrate in the same commit; tautology alias-equivalence tests get DELETED (not renamed).
- Unit 2 (MessageSender<M> newtype + Position cosmetic): new sibling module `src/harness/message_sender.rs` with `MessageSender<M: Send + 'static>` + `MessageSendError<T>` + `TrySendError<T>` + passthrough methods + `into_inner()` escape hatch. `AppHarness::message_sender()` returns the newtype. `virtual_terminal.rs:147` swaps `ratatui::layout::Position` for `crate::layout::Position` (zero runtime change).
- Unit 3 (CHANGELOG + MIGRATION.md updates): the same shape as prior cadences — new sub-sub-sections under `[Unreleased]`; two migration tables under `## v0.16.x to v0.17.0`; audit findings #6 and #8 removed from the Known Deferred Findings block.

**Tech Stack:** Rust (edition 2024, MSRV 1.85), ratatui 0.29, tokio 1.x, cargo-nextest, insta snapshots.

## Global Constraints

Every task's requirements implicitly include this section. Copy verbatim from the spec:

- Signed commits required. If `git commit -S` fails, STOP and ask the user — never bypass with `--no-gpg-sign`.
- Files must stay under 1000 lines.
- No clippy warnings on either default features (tracing OFF) or `--all-features`: `cargo clippy --all-features -- -D warnings` must be clean.
- No dead-code or unused-import warnings on any feature combination.
- `cargo doc --no-deps --all-features` must produce zero intra-doc-link warnings.
- `cargo build --no-default-features` must pass AND `cargo test --no-default-features --no-run` must pass (D8 lesson — the `--no-run` form catches example-gating drift the `build` form misses).
- Audit scorecard 9/9 preserved (this cadence doesn't touch scorecard items; goal is "unchanged").
- **`MessageSender<M>` is parameterized on `M: Send + 'static`, NOT on `A: App`** (adversarial review S1 correction — avoid propagating envision's `App` bound onto downstream helper functions).
- **Heatmap rename is `value_at_selection()`, NOT `selected_cell_value()`** (adversarial review S2 correction — `value_` prefix sorts distinctly from `selected_*` in IDE autocomplete).
- **Table IS in scope for Unit 1** (adversarial review M3 correction — Table has byte-identical divergence to data_grid; leaving it out makes the "consistency sweep" framing dishonest).
- **Grep gates use callsite forms** (`\.selected_row(` etc.), NOT token-boundary matches (adversarial review M1+M2 correction — token-boundary grep collides with private fields and downstream example state).
- **Tautology alias-equivalence tests are DELETED, not renamed** (adversarial review A2 — renaming them to `assert_eq!(state.selected_item(), state.selected_item())` produces genuinely tautological tests).
- **`assert_impl_all!` compile-time gate** on `MessageSender<M>` for `Send + Sync + Clone` (adversarial review A4). `static_assertions` crate is NOT currently in dev-dependencies — implementer writes an inline `fn _assert_send_sync_clone<T: Send + Sync + Clone>() {}` shim instead of adding the dep.

---

## Pre-execution gotchas (read once before Task 1)

- **File sizes are comfortable.** All 6 target component files are between 483-889 lines; adding + deleting won't approach the 1000-line cap. No sibling-file split needed.
- **`static_assertions` NOT in Cargo.toml dev-dependencies** — Task 2's compile-time Send/Sync/Clone gate uses an inline shim, not the crate. Do NOT add the dep.
- **`selected_row` is BOTH a method AND a private field on `DataGridState`.** The method is deleted (public API); the private field is renamed to `selected_row_index` (breaks grep collision with the deleted method name; internal-only rename). Both changes land in the same commit.
- **`selected_row` on `Table::state`: check for the field name pattern.** Table's private field storing the selected row index MAY also be named `selected_row` (~15 refs). Verify at impl time and apply the same `selected_row_index` rename if so. If Table's field has a different name (e.g., `selected` or `current`), leave it alone.
- **Internal `state.selected_value()` call sites inside component `view()` bodies MUST migrate in the same commit** as the alias deletion, or the crate fails to compile mid-task. Enumerated: `select/mod.rs:526,547`, `dropdown/mod.rs:693,722,729`. Re-verify at impl time via `grep -n 'state\.selected_value\b\|self\.selected_value\b' src/component/{select,dropdown}/mod.rs`.
- **Tautology alias-equivalence tests location shift.** After the alias deletions in dropdown/select/data_grid, the exact line numbers of the tautology tests (dropdown/tests.rs:770, select/tests.rs:423, data_grid/tests.rs:87) may shift if earlier deletions in the SAME file moved code up. Locate via `grep -n 'assert_eq!(.*selected_item(),.*selected_row()\|assert_eq!(.*selected_value(),.*selected_item()' src/component/*/tests.rs` at impl time.
- **`Position` re-export at `src/layout/mod.rs:49`** is `pub use ratatui::layout::Position;` — verified. Changing `virtual_terminal.rs:147`'s return type from `Vec<ratatui::layout::Position>` to `Vec<crate::layout::Position>` is truly zero-migration (same underlying type).
- **`AppHarness::message_sender()` internal caller inside the AppHarness itself:** the body is `self.runtime.message_sender()` — that call reaches into `Runtime`, whose method signature returns `tokio::sync::mpsc::Sender<A::Message>`. The `Runtime::message_sender` internal method stays as-is (private-ish path); ONLY the public `AppHarness::message_sender` surface returns the wrapper. Do NOT wrap at the Runtime level.
- **Prelude re-export IS unconditional** (verified: `TestHarness` at `src/lib.rs:476` has no `#[cfg]` guard). `MessageSender` goes into the same unconditional re-export line, no feature-gate needed.
- **Runtime library docs auto-generated** — the crate-root `//!` block at `src/lib.rs:1-141` doesn't mention specific accessor names; safe to change accessor names without doc changes there.

## File structure

Files created:
- `src/harness/message_sender.rs` — new sibling module housing `MessageSender<M>`, `MessageSendError<T>`, `TrySendError<T>`, and their tests. (Task 2, Step 1.)

Files modified:
- `src/component/dropdown/mod.rs` — delete `selected_value()` alias at :251 + migrate internal callers at :693,722,729. (Task 1, Steps 1-2.)
- `src/component/select/mod.rs` — delete `selected_value()` alias at :239 + migrate internal callers at :526,547. (Task 1, Steps 3-4.)
- `src/component/heatmap/mod.rs` — rename `selected_value()` at :466 → `value_at_selection()`. (Task 1, Step 5.)
- `src/component/tab_bar/mod.rs` — delete `selected()` alias at :322; rename `active_tab()` at :336 → `selected_item()`; rename `active_tab_mut()` at :351 → `selected_item_mut()`. (Task 1, Step 6.)
- `src/component/data_grid/state.rs` — delete `selected()` alias at :143 and `selected_row()` alias at :167; rename private field `selected_row: Option<usize>` → `selected_row_index`. (Task 1, Step 7.)
- `src/component/data_grid/mod.rs` — update ~15 internal references to the renamed field (if the field name appears there; verify at impl time). (Task 1, Step 7.)
- `src/component/table/state.rs` — delete `selected()` alias at :270 and `selected_row()` alias at :295; rename private field if present. (Task 1, Step 8.)
- `src/component/dropdown/tests.rs`, `src/component/select/tests.rs`, `src/component/heatmap/tests.rs`, `src/component/tab_bar/tests.rs`, `src/component/data_grid/tests.rs`, `src/component/table/tests.rs` — migrate call-site references; DELETE tautology alias-equivalence tests. (Task 1, Step 9.)
- `src/harness/mod.rs` — declare `mod message_sender;` + `pub use message_sender::{MessageSender, MessageSendError, TrySendError};`. (Task 2, Step 2.)
- `src/lib.rs:408` — extend crate-root harness re-export to include the three new types. (Task 2, Step 3.)
- `src/lib.rs:476` — extend prelude harness re-export to include `MessageSender`. (Task 2, Step 3.)
- `src/harness/app_harness/mod.rs:264` — change return type from raw tokio Sender to `MessageSender<A::Message>`. (Task 2, Step 4.)
- `src/app/runtime/virtual_terminal.rs:147` — change return type from `Vec<ratatui::layout::Position>` to `Vec<crate::layout::Position>`. (Task 2, Step 5.)
- `CHANGELOG.md` — new sub-sub-sections under `[Unreleased]` for Breaking Changes + Added; trim Known Deferred Findings block. (Task 3, Step 1.)
- `MIGRATION.md` — append two new subsections under `## v0.16.x to v0.17.0`. (Task 3, Step 2.)

---

## Task 1: Unit 1 — Consistency sweep across 6 components

**Files:**
- Modify: `src/component/dropdown/mod.rs`
- Modify: `src/component/select/mod.rs`
- Modify: `src/component/heatmap/mod.rs`
- Modify: `src/component/tab_bar/mod.rs`
- Modify: `src/component/data_grid/state.rs`
- Modify: `src/component/data_grid/mod.rs`
- Modify: `src/component/table/state.rs`
- Modify: `src/component/data_grid/tests.rs`, `src/component/select/tests.rs`, `src/component/dropdown/tests.rs`, `src/component/heatmap/tests.rs`, `src/component/tab_bar/tests.rs`, `src/component/table/tests.rs`
- Possibly modify: `examples/` if any example calls a deleted accessor (verify via grep at impl time)

**Interfaces:**
- Consumes: nothing from earlier tasks; this is Task 1.
- Produces: the 10 breaking-rename surface for MIGRATION.md v0.16→v0.17 (Task 3 renders the table). Also produces the `selected_item()` / `selected_index()` / `value_at_selection()` canonical shape used by Task 3's CHANGELOG narrative.

### Step 1: Delete `dropdown::selected_value()` alias + docstring

- [ ] Open `src/component/dropdown/mod.rs`. Locate the `selected_value` method at approximately line 251:

```rust
    /// Returns the value of the currently selected option.
    ///
    /// # Example
    /// ```rust
    /// # use envision::component::DropdownState;
    /// let mut state = DropdownState::new(vec![("a".into(), "Alpha".into())]);
    /// state.set_selected(Some(0));
    /// assert_eq!(state.selected_value(), Some("a"));
    /// ```
    pub fn selected_value(&self) -> Option<&str> {
        self.selected_item()
    }
```

Delete the entire block (docstring + method). If a preceding blank line exists between it and `selected_item()`, leave one blank line separating adjacent methods (matches Rust style).

- [ ] Save.

### Step 2: Migrate internal `state.selected_value()` callers in `dropdown/mod.rs`

- [ ] Locate the internal callers via:

```bash
grep -n 'state\.selected_value\|self\.selected_value' src/component/dropdown/mod.rs
```

Expected: three hits at approximately lines 693, 722, 729 (verify actual line numbers post-Step 1's deletion — line numbers may shift up).

- [ ] For each hit, replace `state.selected_value()` → `state.selected_item()` (or `self.selected_value()` → `self.selected_item()`, whichever form appears). Semantics unchanged — the two methods returned the same value.

- [ ] Verify:

```bash
grep -n '\.selected_value(' src/component/dropdown/mod.rs
```

Expected: zero hits.

- [ ] Compile check:

```bash
cargo check --all-features 2>&1 | tail -5
```

Expected: clean (or errors only in test files, which Step 9 handles).

### Step 3: Delete `select::selected_value()` alias

- [ ] Open `src/component/select/mod.rs`. Locate the `selected_value` method at approximately line 239:

```rust
    /// Returns the value of the currently selected option.
    ///
    /// # Example
    /// ```rust
    /// # use envision::component::SelectState;
    /// let mut state = SelectState::new(vec![("a".into(), "Alpha".into())]);
    /// state.set_selected(Some(0));
    /// assert_eq!(state.selected_value(), Some("a"));
    /// ```
    pub fn selected_value(&self) -> Option<&str> {
        self.selected_item()
    }
```

Delete the entire block (docstring + method).

- [ ] Save.

### Step 4: Migrate internal `state.selected_value()` callers in `select/mod.rs`

- [ ] Locate the internal callers via:

```bash
grep -n 'state\.selected_value\|self\.selected_value' src/component/select/mod.rs
```

Expected: two hits at approximately lines 526, 547 (line numbers may shift).

- [ ] For each hit, replace with `.selected_item()`.

- [ ] Verify:

```bash
grep -n '\.selected_value(' src/component/select/mod.rs
```

Expected: zero hits.

- [ ] Compile check:

```bash
cargo check --all-features 2>&1 | tail -5
```

Expected: clean (or errors only in tests).

### Step 5: Rename `heatmap::selected_value()` → `value_at_selection()`

- [ ] Open `src/component/heatmap/mod.rs`. Locate the `selected_value` method at approximately line 466:

```rust
    /// Returns the value at the currently selected cell.
    ///
    /// # Example
    /// ```rust
    /// use envision::component::HeatmapState;
    ///
    /// let state = HeatmapState::with_data(vec![vec![7.5, 3.2]]);
    /// assert_eq!(state.selected_value(), Some(7.5));
    /// ```
    pub fn selected_value(&self) -> Option<f64> {
        let (r, c) = self.selected()?;
        self.get(r, c)
    }
```

Replace with:

```rust
    /// Returns the data value at the currently selected cell, or `None` if
    /// no cell is selected.
    ///
    /// This is distinct from [`selected`](Self::selected), which returns
    /// the `(row, col)` coordinate pair — `value_at_selection()` reads that
    /// coordinate out of the underlying data grid.
    ///
    /// Renamed from `selected_value()` in v0.17.0 to disambiguate from the
    /// collection-selection pattern used by other components (see
    /// MIGRATION.md `v0.16.x to v0.17.0`).
    ///
    /// # Example
    /// ```rust
    /// use envision::component::HeatmapState;
    ///
    /// let state = HeatmapState::with_data(vec![vec![7.5, 3.2]]);
    /// assert_eq!(state.value_at_selection(), Some(7.5));
    /// ```
    pub fn value_at_selection(&self) -> Option<f64> {
        let (r, c) = self.selected()?;
        self.get(r, c)
    }
```

Method body is unchanged. Docstring updated to (a) rename the referenced method in the example, (b) cross-link `selected()` as the coordinate-pair companion, (c) note the rename with a MIGRATION.md pointer.

- [ ] Save.

- [ ] Verify internal callers:

```bash
grep -n '\.selected_value(' src/component/heatmap/mod.rs
```

Expected: zero hits (heatmap doesn't have internal `selected_value` callers per grep; test-file callers are Step 9).

### Step 6: Migrate `tab_bar` — delete `selected()` alias; rename `active_tab*` → `selected_item*`

- [ ] Open `src/component/tab_bar/mod.rs`. Locate `selected()` at approximately line 322:

```rust
    /// Returns the selected tab index, or `None` if the tab bar is empty.
    ///
    /// This is the getter counterpart to [`set_selected`](Self::set_selected).
    ///
    /// # Example
    /// ```rust
    /// use envision::component::{Tab, TabBarState};
    ///
    /// let mut state = TabBarState::new(vec![
    ///     Tab::new("a", "A"),
    ///     Tab::new("b", "B"),
    /// ]);
    /// assert_eq!(state.selected(), Some(0));
    ///
    /// state.set_selected(Some(1));
    /// assert_eq!(state.selected(), Some(1));
    ///
    /// state.set_selected(None);
    /// assert_eq!(state.selected(), None);
    /// ```
    pub fn selected(&self) -> Option<usize> {
        self.active
    }
```

Delete the entire block (docstring + method). This is a literal alias for `selected_index()` — deleting it forces consumers to standardize on the canonical name.

- [ ] Locate `active_tab()` at approximately line 336:

```rust
    /// Returns the currently active tab, or `None` if empty.
    ///
    /// # Example
    /// ```rust
    /// use envision::component::{Tab, TabBarState};
    ///
    /// let state = TabBarState::new(vec![Tab::new("a", "Alpha")]);
    /// assert_eq!(state.active_tab().unwrap().label(), "Alpha");
    /// ```
    pub fn active_tab(&self) -> Option<&Tab> {
        self.tabs.get(self.active?)
    }
```

Replace with:

```rust
    /// Returns the currently selected tab, or `None` if empty.
    ///
    /// Renamed from `active_tab()` in v0.17.0 for consistency with the
    /// `selected_item()` accessor pattern used by other components (see
    /// MIGRATION.md `v0.16.x to v0.17.0`).
    ///
    /// # Example
    /// ```rust
    /// use envision::component::{Tab, TabBarState};
    ///
    /// let state = TabBarState::new(vec![Tab::new("a", "Alpha")]);
    /// assert_eq!(state.selected_item().unwrap().label(), "Alpha");
    /// ```
    pub fn selected_item(&self) -> Option<&Tab> {
        self.tabs.get(self.active?)
    }
```

- [ ] Locate `active_tab_mut()` at approximately line 351:

```rust
    /// Returns a mutable reference to the currently active tab, or `None` if empty.
    ///
    /// # Example
    /// ```rust
    /// use envision::component::{Tab, TabBarState};
    ///
    /// let mut state = TabBarState::new(vec![Tab::new("a", "Alpha")]);
    /// if let Some(tab) = state.active_tab_mut() {
    ///     tab.set_label("Alpha!".into());
    /// }
    /// assert_eq!(state.active_tab().unwrap().label(), "Alpha!");
    /// ```
    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.tabs.get_mut(self.active?)
    }
```

Replace with:

```rust
    /// Returns a mutable reference to the currently selected tab, or `None` if empty.
    ///
    /// Renamed from `active_tab_mut()` in v0.17.0 (see MIGRATION.md
    /// `v0.16.x to v0.17.0`).
    ///
    /// # Example
    /// ```rust
    /// use envision::component::{Tab, TabBarState};
    ///
    /// let mut state = TabBarState::new(vec![Tab::new("a", "Alpha")]);
    /// if let Some(tab) = state.selected_item_mut() {
    ///     tab.set_label("Alpha!".into());
    /// }
    /// assert_eq!(state.selected_item().unwrap().label(), "Alpha!");
    /// ```
    pub fn selected_item_mut(&mut self) -> Option<&mut Tab> {
        self.tabs.get_mut(self.active?)
    }
```

- [ ] Save.

- [ ] Verify internal callers in `tab_bar/mod.rs`:

```bash
grep -n '\.selected()\|\.active_tab(\|\.active_tab_mut(' src/component/tab_bar/mod.rs
```

Expected: zero hits (all uses should be in tests, which Step 9 handles).

### Step 7: `data_grid` — delete `selected()` + `selected_row()` aliases; rename private field

- [ ] Open `src/component/data_grid/state.rs`. Locate `selected()` at approximately line 143:

```rust
    /// Alias for [`selected_index()`](Self::selected_index).
    ///
    /// # Example
    /// ```rust
    /// # ...
    /// assert_eq!(state.selected(), Some(0));
    /// ```
    pub fn selected(&self) -> Option<usize> {
        self.selected_index()
    }
```

Delete the entire block. This is a literal alias.

- [ ] Locate `selected_row()` at approximately line 167:

```rust
    /// Returns a reference to the currently selected row.
    ///
    /// # Example
    /// ```rust
    /// # ...
    /// assert_eq!(state.selected_row().unwrap().name, "Alice");
    /// ```
    pub fn selected_row(&self) -> Option<&T> {
        self.selected_row.and_then(|i| self.rows.get(i))
    }
```

Delete the entire block. Body accesses `self.selected_row` (the private field). After field rename in the next sub-step, this method would have referred to the old field name; deleting it avoids the compile break entirely.

- [ ] Rename the private field. Locate the struct definition at approximately line 30-45 (top of `DataGridState<T>`):

```rust
pub struct DataGridState<T> {
    // ...
    selected_row: Option<usize>,
    // ...
}
```

Change field name to `selected_row_index`:

```rust
pub struct DataGridState<T> {
    // ...
    selected_row_index: Option<usize>,
    // ...
}
```

- [ ] Update all references to the renamed field within `src/component/data_grid/state.rs`. Locate via:

```bash
grep -n '\bselected_row\b' src/component/data_grid/state.rs
```

Expected: ~10-15 hits (constructor bodies, `set_selected`, `selected_index()` returning `self.selected_row`, etc.).

For each hit, replace `self.selected_row` → `self.selected_row_index`.

- [ ] Verify:

```bash
grep -n '\bselected_row\b' src/component/data_grid/state.rs
```

Expected: zero hits (or hits only in comments/docstrings — inspect and update those too if they reference the old field name).

- [ ] Open `src/component/data_grid/mod.rs`. Check for references to the renamed field:

```bash
grep -n '\bselected_row\b' src/component/data_grid/mod.rs
```

Expected: several hits (~5-10) — replace each with `selected_row_index`.

- [ ] Save both files.

- [ ] Compile check:

```bash
cargo check --all-features 2>&1 | tail -10
```

Expected: clean (or errors only in tests).

### Step 8: `table` — delete `selected()` + `selected_row()` aliases

- [ ] Open `src/component/table/state.rs`. Locate `selected()` at approximately line 270:

```rust
    /// Alias for [`selected_index()`](Self::selected_index).
    ///
    /// # Example
    /// ```rust
    /// # ...
    /// assert_eq!(state.selected(), Some(0));
    /// ```
    pub fn selected(&self) -> Option<usize> {
        self.selected_index()
    }
```

Delete the entire block.

- [ ] Locate `selected_row()` at approximately line 295:

```rust
    /// Returns a reference to the currently selected row.
    ///
    /// # Example
    /// ```rust
    /// # ...
    /// assert_eq!(state.selected_row().unwrap().name, "Alice");
    /// ```
    pub fn selected_row(&self) -> Option<&T> {
        // body accessing self.selected_row or similar
    }
```

Delete the entire block.

- [ ] Check for a private field named `selected_row` on `TableState<T>`:

```bash
grep -n 'selected_row:' src/component/table/state.rs | head -3
```

If a private field named `selected_row: Option<usize>` exists, apply the same rename as data_grid (Step 7):

- Rename field to `selected_row_index`
- Update all internal references (`self.selected_row` → `self.selected_row_index`)

If no such field exists (Table's selection storage may have a different name like `selected` or use a delegated state), leave the field alone.

- [ ] Save.

- [ ] Compile check:

```bash
cargo check --all-features 2>&1 | tail -10
```

Expected: clean (or errors only in tests).

### Step 9: Migrate + delete tests

- [ ] Migrate call-site references in all 6 test files. Systematic grep:

```bash
grep -n '\.selected_value(\|\.active_tab(\|\.active_tab_mut(\|\.selected_row(' \
  src/component/dropdown/tests.rs \
  src/component/select/tests.rs \
  src/component/heatmap/tests.rs \
  src/component/tab_bar/tests.rs \
  src/component/data_grid/tests.rs \
  src/component/table/tests.rs
```

For each hit, apply the rename per the spec's per-component table:
- `.selected_value()` → `.selected_item()` (dropdown, select) or `.value_at_selection()` (heatmap)
- `.active_tab()` → `.selected_item()` (tab_bar)
- `.active_tab_mut()` → `.selected_item_mut()` (tab_bar)
- `.selected_row()` → `.selected_item()` (data_grid, table)

Also grep for `.selected()` inside these 6 test files (returned `Option<usize>` — was an alias for `.selected_index()` in tab_bar/data_grid/table):

```bash
grep -n '\.selected()' src/component/tab_bar/tests.rs src/component/data_grid/tests.rs src/component/table/tests.rs
```

For each hit, replace `.selected()` → `.selected_index()`.

**Exception (heatmap):** heatmap's `.selected()` returns `Option<(usize, usize)>` — a genuine coordinate accessor, NOT an alias. Do NOT rename `.selected()` calls in `heatmap/tests.rs`.

- [ ] **Delete tautology alias-equivalence tests.** Locate the ~4 tests that assert an alias returns the same value as its canonical counterpart:

```bash
grep -nB2 -A5 'assert_eq!(.*selected_item(),.*selected_row()\|assert_eq!(.*selected_value(),.*selected_item()\|assert_eq!(.*active_tab(),.*selected_item()\|assert_eq!(.*selected(),.*selected_index()' \
  src/component/*/tests.rs
```

Expected: ~4-6 hits across dropdown/select/data_grid/table/tab_bar tests.

For each hit, locate the containing `#[test] fn ...` block and DELETE the entire test function (including the `#[test]` attribute, function signature, body, and closing brace).

Do NOT rename these tests. After the alias is gone, they'd become `assert_eq!(x, x)` — a genuinely tautological assertion with no coverage value.

- [ ] Save all test files.

- [ ] Run the affected component tests:

```bash
cargo nextest run --all-features -E 'test(dropdown) or test(select) or test(heatmap) or test(tab_bar) or test(data_grid) or test(table)' 2>&1 | tail -15
```

Expected: all pass.

### Step 10: Grep-verify the callsite forms are clean

- [ ] Run the callsite-form grep gates (per spec's Success Criteria section):

```bash
echo "--- .selected_value( ---"
grep -rn '\.selected_value(' src/ tests/ examples/
echo "--- .active_tab( ---"
grep -rn '\.active_tab(' src/ tests/ examples/
echo "--- .active_tab_mut( ---"
grep -rn '\.active_tab_mut(' src/ tests/ examples/
echo "--- .selected_row( ---"
grep -rn '\.selected_row(' src/ tests/ examples/
```

Expected: zero hits from all four (except possibly CHANGELOG.md / MIGRATION.md prose from Task 3 — those files aren't grepped here since we scope to `src/` `tests/` `examples/`).

If ANY hits remain in code (not comment/doc), locate and migrate them.

Note: the `.selected()` gate is NOT run here — it has legitimate hits (heatmap coordinate accessor + 15+ other components with the alias not in this scope per adversarial review A3). Task 4's verification gauntlet uses a narrower grep specific to the 6 in-scope components.

### Step 11: Verify data_grid + table private field renames took

- [ ] If Table has a `selected_row` field (Step 8 decision point):

```bash
grep -n '\bselected_row\b' src/component/table/state.rs src/component/table/mod.rs
```

Expected: zero hits (or hits only in comments/docstrings that don't reference the old field name).

- [ ] For data_grid (always applies):

```bash
grep -n '\bselected_row\b' src/component/data_grid/state.rs src/component/data_grid/mod.rs
```

Expected: zero hits (or comment/doc references, which should be updated to `selected_row_index`).

### Step 12: Full-suite verification for Task 1

- [ ] Run:

```bash
cargo fmt --check
```

Expected: no output (clean).

- [ ] Run:

```bash
cargo clippy --all-features -- -D warnings 2>&1 | tail -10
```

Expected: no warnings.

- [ ] Run:

```bash
cargo nextest run --all-features 2>&1 | tail -10
```

Expected: all tests pass. Test count may DECREASE slightly (~4-6) due to tautology-test deletions.

- [ ] Run:

```bash
cargo test --all-features --doc 2>&1 | tail -10
```

Expected: all doc tests pass. New doc tests were added on `heatmap::value_at_selection` + `tab_bar::selected_item` + `selected_item_mut` docstrings.

- [ ] Run:

```bash
cargo build --no-default-features 2>&1 | tail -3
```

Expected: clean.

- [ ] Run:

```bash
cargo test --no-default-features --no-run 2>&1 | tail -5
```

Expected: clean.

- [ ] Run:

```bash
cargo doc --no-deps --all-features 2>&1 | grep -iE "warning|error" | head -10
```

Expected: no intra-doc-link warnings from the changed files (Task 1 doesn't touch any files that generate intra-doc warnings per the release-readiness cadence's baseline).

### Step 13: Commit Task 1

- [ ] Stage:

```bash
git add src/component/dropdown/mod.rs src/component/select/mod.rs src/component/heatmap/mod.rs src/component/tab_bar/mod.rs src/component/data_grid/mod.rs src/component/data_grid/state.rs src/component/table/state.rs src/component/dropdown/tests.rs src/component/select/tests.rs src/component/heatmap/tests.rs src/component/tab_bar/tests.rs src/component/data_grid/tests.rs src/component/table/tests.rs
```

- [ ] Commit:

```bash
git commit -S -m "$(cat <<'EOF'
consistency-sweep: canonical selected_item() across 6 components

Close audit finding #6 (selected_value / selected_item / active_tab
divergence). 10 breaking renames total; delete-outright pre-1.0
pattern matching D5/D14/G7/D12/D3/D8/resource_gauge/FileSortDirection
precedent.

Per-component changes:
- dropdown: delete selected_value() alias; keep selected_item() ->
  Option<&str>. Internal callers at :693, :722, :729 migrated.
- select: same as dropdown. Internal callers at :526, :547 migrated.
- heatmap: rename selected_value() -> value_at_selection() (returns
  data value at cursor; NOT a selection accessor). Docstring
  cross-links `selected()` as the coordinate-pair companion.
- tab_bar: delete selected() alias; rename active_tab() ->
  selected_item(); rename active_tab_mut() -> selected_item_mut().
- data_grid: delete selected() alias and selected_row() alias; keep
  selected_index() + selected_item() + set_selected() +
  selected_column(). Rename private field selected_row ->
  selected_row_index to break the grep collision with the deleted
  method name (internal-only, zero downstream impact).
- table: delete selected() alias and selected_row() alias; keep
  selected_index() + selected_item() + set_selected(). Same field
  rename if applicable.

Tautology alias-equivalence tests DELETED (not renamed):
- data_grid/tests.rs tautology test
- select/tests.rs tautology test
- dropdown/tests.rs tautology test
- Plus any table/tab_bar equivalents. After the aliases are gone,
  these tests would be assert_eq!(x, x) — no coverage value.

Grep-verified via callsite forms (\.selected_value(, \.active_tab(,
\.active_tab_mut(, \.selected_row() -- zero hits across src/ tests/
examples/. Token-boundary grep intentionally NOT used because it
collides with private fields (data_grid's selected_row_index field
still contains the substring "selected_row") and downstream example
state (chat_client.rs has its own active_tab: usize field).

Verification: cargo fmt / cargo clippy --all-features -- -D warnings
/ cargo nextest run --all-features / cargo test --doc / cargo build
--no-default-features / cargo test --no-default-features --no-run all
clean. Test count decreased by ~4-6 due to tautology-test deletions.

Migration table for MIGRATION.md v0.16→v0.17 populated in Task 3.
EOF
)"
```

- [ ] Verify signature:

```bash
git log --show-signature -1 HEAD 2>&1 | head -5
```

Expected: `Good signature from "Ryan O'Neill ..."`.

---

## Task 2: Unit 2 — `MessageSender<M>` newtype + Position cosmetic

**Files:**
- Create: `src/harness/message_sender.rs`
- Modify: `src/harness/mod.rs`
- Modify: `src/lib.rs` (extend two re-export lines)
- Modify: `src/harness/app_harness/mod.rs:264`
- Modify: `src/app/runtime/virtual_terminal.rs:147`

**Interfaces:**
- Consumes: nothing from Task 1 (independent code path).
- Produces: `envision::harness::{MessageSender, MessageSendError, TrySendError}` public API + `envision::prelude::MessageSender`. `AppHarness::message_sender()` returns `MessageSender<A::Message>` instead of raw tokio Sender. `crate::layout::Position` used in `virtual_terminal.rs:147`.

### Step 1: Create `src/harness/message_sender.rs`

- [ ] Create the new file with the following content:

```rust
//! `MessageSender<M>` — first-party wrapper around the async message channel
//! that carries messages into an [`AppHarness`](crate::harness::AppHarness).
//!
//! Hides the underlying `tokio::sync::mpsc::Sender<M>` so envision consumers
//! don't need `tokio` as a direct dependency to use the message-injection
//! surface. Full tokio Sender semantics are preserved through passthrough
//! methods (`send`, `try_send`, `is_closed`, `capacity`, `max_capacity`)
//! plus an explicit [`into_inner`](MessageSender::into_inner) escape hatch
//! for the small number of consumers who need tokio-specific functionality
//! (`reserve`, `send_timeout`, `same_channel`, `downgrade`, `closed()` future).

use tokio::sync::mpsc;

/// Hands the caller a way to inject messages into the AppHarness's Runtime
/// asynchronously — from subscription callbacks, spawned tasks, or any other
/// non-App-loop code path.
///
/// Wraps `tokio::sync::mpsc::Sender<M>` so envision consumers don't need
/// `tokio` as a direct dependency to use `AppHarness::message_sender()`.
/// The Sender's semantics are preserved (bounded, cloneable, `send` returns
/// `Result` on receiver-dropped) and its non-mutating query surface
/// (`is_closed`, `capacity`, `max_capacity`) is passed through. Consumers
/// needing tokio-specific functionality beyond what's exposed can call
/// [`into_inner`](Self::into_inner) as an explicit escape hatch.
///
/// # Type parameter
///
/// `MessageSender<M>` is parameterized on the message type `M`, not on an
/// App-typed generic. This means portable helper functions like
/// `fn spawn_watcher<M: Send + 'static>(sender: MessageSender<M>) { ... }`
/// work without depending on envision's `App` trait.
///
/// # Example
///
/// ```rust,no_run
/// use envision::prelude::*;
///
/// async fn ingest<M: Send + 'static>(sender: MessageSender<M>, msg: M) {
///     sender.send(msg).await.expect("harness still alive");
/// }
/// ```
pub struct MessageSender<M> {
    inner: mpsc::Sender<M>,
}

impl<M> MessageSender<M> {
    /// Wraps the given tokio Sender. Internal constructor — external
    /// consumers acquire a `MessageSender<M>` via
    /// [`AppHarness::message_sender()`](crate::harness::AppHarness::message_sender).
    pub(crate) fn new(inner: mpsc::Sender<M>) -> Self {
        Self { inner }
    }

    /// Sends a message into the AppHarness. Returns an error only when the
    /// AppHarness (and hence the receiver) has been dropped.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use envision::prelude::*;
    ///
    /// async fn example<M: Send + 'static>(sender: MessageSender<M>, msg: M) {
    ///     sender.send(msg).await.expect("harness still alive");
    /// }
    /// ```
    pub async fn send(&self, msg: M) -> Result<(), MessageSendError<M>> {
        self.inner.send(msg).await.map_err(|e| MessageSendError(e.0))
    }

    /// Attempts to send a message without waiting. Returns an error if the
    /// channel is full or the AppHarness has been dropped. The message is
    /// returned inside the error variant when send fails, so the caller can
    /// retry or handle it.
    pub fn try_send(&self, msg: M) -> Result<(), TrySendError<M>> {
        self.inner.try_send(msg).map_err(TrySendError::from_tokio)
    }

    /// Returns `true` if the receiver end of the channel has been dropped.
    ///
    /// Useful for `spawn_watcher`-style loops that want to exit before
    /// wasting work on messages that would fail to send.
    pub fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }

    /// Returns the current available capacity of the channel — the number
    /// of messages that can be enqueued without blocking or hitting a
    /// `TrySendError::Full`.
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Returns the maximum capacity of the channel (the bound configured
    /// at AppHarness construction).
    pub fn max_capacity(&self) -> usize {
        self.inner.max_capacity()
    }

    /// Explicit escape hatch: consumes the wrapper and returns the underlying
    /// `tokio::sync::mpsc::Sender<M>` for consumers who need tokio-specific
    /// functionality (`reserve`, `send_timeout`, `same_channel`, `downgrade`,
    /// or a `closed()` Future) that this wrapper deliberately doesn't expose
    /// to keep the default surface minimal.
    ///
    /// Using this method re-couples your code to the tokio dep; it's an
    /// escape hatch by design, not a routine call.
    pub fn into_inner(self) -> mpsc::Sender<M> {
        self.inner
    }
}

impl<M> Clone for MessageSender<M> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<M> std::fmt::Debug for MessageSender<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageSender").finish_non_exhaustive()
    }
}

/// Error returned by [`MessageSender::send`] when the AppHarness receiver
/// has been dropped. Carries the message back so the caller can inspect it.
#[derive(Debug)]
pub struct MessageSendError<T>(pub T);

impl<T> std::fmt::Display for MessageSendError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "message sender: receiver dropped")
    }
}

impl<T: std::fmt::Debug> std::error::Error for MessageSendError<T> {}

/// Error returned by [`MessageSender::try_send`].
///
/// Preserves tokio's `Full` / `Closed` distinction so consumers can
/// retry-on-full or exit-on-closed with a match arm.
#[derive(Debug)]
pub enum TrySendError<T> {
    /// Channel is full; the message was NOT sent. Caller may retry later
    /// once the AppHarness has drained.
    Full(T),
    /// AppHarness receiver has been dropped; the message was NOT sent and
    /// will never succeed on retry.
    Closed(T),
}

impl<T> TrySendError<T> {
    /// Extracts the message from either variant.
    pub fn into_inner(self) -> T {
        match self {
            Self::Full(t) | Self::Closed(t) => t,
        }
    }

    /// Internal converter from tokio's `TrySendError<T>` — decouples the
    /// call sites from tokio's error path.
    fn from_tokio(err: mpsc::error::TrySendError<T>) -> Self {
        match err {
            mpsc::error::TrySendError::Full(t) => TrySendError::Full(t),
            mpsc::error::TrySendError::Closed(t) => TrySendError::Closed(t),
        }
    }
}

impl<T> std::fmt::Display for TrySendError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Full(_) => write!(f, "message sender: channel full"),
            Self::Closed(_) => write!(f, "message sender: receiver dropped"),
        }
    }
}

impl<T: std::fmt::Debug> std::error::Error for TrySendError<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    // Compile-time gate that MessageSender<M> is Send + Sync + Clone when
    // M is Send. Uses an inline shim rather than the `static_assertions`
    // crate (not in dev-dependencies).
    fn _assert_send_sync_clone<T: Send + Sync + Clone>() {}

    fn _compile_assertions() {
        _assert_send_sync_clone::<MessageSender<u32>>();
        _assert_send_sync_clone::<MessageSender<String>>();
    }

    #[tokio::test]
    async fn test_send_round_trip() {
        let (tx, mut rx) = mpsc::channel::<u32>(16);
        let sender = MessageSender::new(tx);

        sender.send(42).await.expect("send succeeds");

        let received = rx.recv().await;
        assert_eq!(received, Some(42));
    }

    #[tokio::test]
    async fn test_try_send_full() {
        // Channel with capacity 1; fill it, then try_send should return Full.
        let (tx, _rx) = mpsc::channel::<u32>(1);
        let sender = MessageSender::new(tx);

        // Fill the buffer without draining.
        sender.try_send(1).expect("first try_send succeeds");
        // Second try_send should return Full (buffer full).
        let err = sender.try_send(2).expect_err("second try_send is Full");
        assert!(matches!(err, TrySendError::Full(2)));
    }

    #[tokio::test]
    async fn test_send_closed() {
        let (tx, rx) = mpsc::channel::<u32>(16);
        let sender = MessageSender::new(tx);

        drop(rx);

        let err = sender.send(99).await.expect_err("send fails");
        assert_eq!(err.0, 99);
    }

    #[tokio::test]
    async fn test_try_send_closed() {
        let (tx, rx) = mpsc::channel::<u32>(16);
        let sender = MessageSender::new(tx);

        drop(rx);

        let err = sender.try_send(99).expect_err("try_send fails");
        assert!(matches!(err, TrySendError::Closed(99)));
    }

    #[tokio::test]
    async fn test_is_closed() {
        let (tx, rx) = mpsc::channel::<u32>(16);
        let sender = MessageSender::new(tx);

        assert!(!sender.is_closed());
        drop(rx);
        assert!(sender.is_closed());
    }

    #[tokio::test]
    async fn test_capacity_and_max_capacity() {
        let (tx, _rx) = mpsc::channel::<u32>(4);
        let sender = MessageSender::new(tx);

        assert_eq!(sender.max_capacity(), 4);
        assert_eq!(sender.capacity(), 4);

        // After filling one slot, capacity decreases.
        sender.try_send(1).expect("first try_send");
        assert_eq!(sender.capacity(), 3);
        assert_eq!(sender.max_capacity(), 4); // unchanged
    }

    #[tokio::test]
    async fn test_clone_shares_channel() {
        let (tx, mut rx) = mpsc::channel::<u32>(16);
        let sender = MessageSender::new(tx);
        let cloned = sender.clone();

        sender.send(1).await.expect("original send");
        cloned.send(2).await.expect("cloned send");

        assert_eq!(rx.recv().await, Some(1));
        assert_eq!(rx.recv().await, Some(2));
    }

    #[tokio::test]
    async fn test_into_inner_escape_hatch() {
        let (tx, mut rx) = mpsc::channel::<u32>(16);
        let sender = MessageSender::new(tx);

        let inner: mpsc::Sender<u32> = sender.into_inner();
        inner.send(42).await.expect("inner tokio send succeeds");

        assert_eq!(rx.recv().await, Some(42));
    }

    #[test]
    fn test_try_send_error_into_inner() {
        let err = TrySendError::Full(42u32);
        assert_eq!(err.into_inner(), 42);

        let err = TrySendError::Closed(99u32);
        assert_eq!(err.into_inner(), 99);
    }

    #[test]
    fn test_message_send_error_message_recovered() {
        let err = MessageSendError(42u32);
        assert_eq!(err.0, 42);
    }

    #[test]
    fn test_display_impls() {
        let msg_err: MessageSendError<u32> = MessageSendError(42);
        assert_eq!(msg_err.to_string(), "message sender: receiver dropped");

        let full: TrySendError<u32> = TrySendError::Full(1);
        assert_eq!(full.to_string(), "message sender: channel full");

        let closed: TrySendError<u32> = TrySendError::Closed(2);
        assert_eq!(closed.to_string(), "message sender: receiver dropped");
    }
}
```

- [ ] Save. File is ~250 lines including tests — comfortably under the 1000-line cap.

### Step 2: Update `src/harness/mod.rs`

- [ ] Open `src/harness/mod.rs`. Locate the top-of-file module declarations. Add:

```rust
mod message_sender;

pub use message_sender::{MessageSender, MessageSendError, TrySendError};
```

Place next to (or alongside) the existing sibling module declarations. If the file uses a `mod app_harness;` line and a `pub use app_harness::AppHarness;` re-export, follow the same pattern.

- [ ] Save.

- [ ] Compile check:

```bash
cargo check --all-features 2>&1 | tail -5
```

Expected: clean.

### Step 3: Update `src/lib.rs` re-exports

- [ ] Open `src/lib.rs`. Locate line 408 (crate-root harness re-export):

```rust
pub use harness::{AppHarness, Assertion, Snapshot, TestHarness};
```

Extend to:

```rust
pub use harness::{
    AppHarness, Assertion, MessageSendError, MessageSender, Snapshot, TestHarness,
    TrySendError,
};
```

Names in alphabetical order for consistency with the existing style.

- [ ] Locate line 476 (prelude harness re-export):

```rust
pub use crate::harness::{AppHarness, TestHarness};
```

Extend to:

```rust
pub use crate::harness::{AppHarness, MessageSender, TestHarness};
```

- [ ] Save.

- [ ] Compile check:

```bash
cargo check --all-features 2>&1 | tail -5
```

Expected: clean.

- [ ] Verify the type resolves via all three paths:

```bash
cargo test --all-features -p envision --doc message_sender 2>&1 | tail -5
```

Expected: the `MessageSender` doc-tests compile and run. If not, verify the re-export chain is complete.

### Step 4: Update `AppHarness::message_sender()` return type

- [ ] Open `src/harness/app_harness/mod.rs`. Locate line 264 (the current `message_sender` method):

```rust
    pub fn message_sender(&self) -> tokio::sync::mpsc::Sender<A::Message> {
        self.runtime.message_sender()
    }
```

Replace with:

```rust
    /// Returns a [`MessageSender<A::Message>`](crate::harness::MessageSender) for
    /// injecting messages into this AppHarness's Runtime from subscription
    /// callbacks, spawned tasks, or any other non-App-loop code path.
    ///
    /// Returns the envision-native newtype rather than the raw
    /// `tokio::sync::mpsc::Sender<A::Message>` so consumers don't need `tokio`
    /// as a direct dependency. Use
    /// [`MessageSender::into_inner`](crate::harness::MessageSender::into_inner)
    /// as an explicit escape hatch if you need tokio-specific functionality.
    pub fn message_sender(&self) -> crate::harness::MessageSender<A::Message> {
        crate::harness::MessageSender::new(self.runtime.message_sender())
    }
```

The docstring is new; the method body just wraps the inner Runtime sender. No behavior change beyond the return type.

- [ ] Save.

- [ ] Compile check:

```bash
cargo check --all-features 2>&1 | tail -5
```

Expected: clean.

### Step 5: Update `virtual_terminal.rs` Position return type

- [ ] Open `src/app/runtime/virtual_terminal.rs`. Locate line 147:

```rust
    pub fn find_text(&self, needle: &str) -> Vec<ratatui::layout::Position> {
```

Replace with:

```rust
    pub fn find_text(&self, needle: &str) -> Vec<crate::layout::Position> {
```

Zero runtime change: `crate::layout::Position` is `pub use ratatui::layout::Position;` at `src/layout/mod.rs:49`. Same underlying type; changes only the source-visible path.

- [ ] Save.

### Step 6: Full-suite verification for Task 2

- [ ] Run:

```bash
cargo fmt --check
```

Expected: clean.

- [ ] Run:

```bash
cargo clippy --all-features -- -D warnings 2>&1 | tail -10
```

Expected: no warnings.

- [ ] Run the MessageSender unit tests + full suite:

```bash
cargo nextest run --all-features -E 'test(message_sender)' 2>&1 | tail -15
```

Expected: all 12 unit tests pass (round_trip / try_send_full / send_closed / try_send_closed / is_closed / capacity_and_max_capacity / clone_shares_channel / into_inner_escape_hatch / try_send_error_into_inner / message_send_error_message_recovered / display_impls / _compile_assertions).

- [ ] Run:

```bash
cargo nextest run --all-features 2>&1 | tail -10
```

Expected: full suite passes.

- [ ] Run:

```bash
cargo test --all-features --doc 2>&1 | tail -10
```

Expected: doc tests pass (including the new MessageSender examples on the AppHarness accessor + on MessageSender::send).

- [ ] Run:

```bash
cargo build --no-default-features 2>&1 | tail -3
```

Expected: clean.

- [ ] Run:

```bash
cargo test --no-default-features --no-run 2>&1 | tail -5
```

Expected: clean.

- [ ] Run:

```bash
cargo doc --no-deps --all-features 2>&1 | grep -iE "warning|error" | head -10
```

Expected: zero intra-doc-link warnings.

### Step 7: Grep-verify dep-leakage reduced

- [ ] Confirm `tokio::sync::mpsc::Sender` no longer appears in public API surfaces:

```bash
grep -rn 'pub fn.*tokio::sync::mpsc::Sender' src/
```

Expected: zero hits (only the internal `Runtime::message_sender` may have this — that's private-ish, not a public API break).

- [ ] Confirm `ratatui::layout::Position` no longer appears in public API surfaces:

```bash
grep -rn 'pub fn.*ratatui::layout::Position' src/
```

Expected: zero hits.

### Step 8: Commit Task 2

- [ ] Stage:

```bash
git add src/harness/message_sender.rs src/harness/mod.rs src/lib.rs src/harness/app_harness/mod.rs src/app/runtime/virtual_terminal.rs
```

- [ ] Commit:

```bash
git commit -S -m "$(cat <<'EOF'
message-sender: newtype wraps tokio::sync::mpsc::Sender (audit finding #8)

Close audit finding #8 (dep-leakage in AppHarness surface) with a
first-party MessageSender<M> newtype. Also lands one small cosmetic:
virtual_terminal.rs:147 swaps ratatui::layout::Position for
crate::layout::Position (same underlying type via re-export at
src/layout/mod.rs:49).

New src/harness/message_sender.rs (~250 lines including tests):

- `MessageSender<M: Send + 'static>` (parameterized on message type,
  NOT on `A: App` — avoids propagating envision's App bound onto
  every downstream helper function).
- Passthrough surface: send() (async, returns Result<(),
  MessageSendError<M>>), try_send() (returns Result<(),
  TrySendError<M>> with Full/Closed variants preserved), is_closed(),
  capacity(), max_capacity().
- Escape hatch: into_inner() consumes and returns the underlying
  tokio Sender for consumers who need reserve/send_timeout/
  same_channel/downgrade/closed() future.
- First-party error types: MessageSendError<T>(T) carries the
  message back on receiver-drop; TrySendError<T>::{Full,Closed}
  preserves tokio's semantic distinction.
- Impls: Clone (shares channel), Debug (non-exhaustive), std::error
  chain on error types.
- 12 unit tests: send round trip, try_send full, send/try_send
  closed, is_closed, capacity/max_capacity, clone shares channel,
  into_inner escape hatch, error accessors, Display impls, plus a
  compile-time _assert_send_sync_clone<MessageSender<_>>() shim
  (static_assertions crate NOT in dev-deps; inline shim used
  instead).

Re-exports:
- src/harness/mod.rs: mod + pub use message_sender::{MessageSender,
  MessageSendError, TrySendError}
- src/lib.rs:408: crate-root harness re-export extended
- src/lib.rs:476: prelude harness re-export extended (unconditional
  — matches TestHarness precedent)

AppHarness::message_sender() at :264 now returns
crate::harness::MessageSender<A::Message> instead of raw
tokio::sync::mpsc::Sender<A::Message>. Body wraps the internal
Runtime sender. Docstring cross-links MessageSender::into_inner as
the escape hatch.

Position cosmetic: virtual_terminal.rs:147 pub fn find_text signature
changes from Vec<ratatui::layout::Position> to
Vec<crate::layout::Position>. Zero runtime change (crate::layout::
Position IS ratatui::layout::Position via re-export). Removes the
last direct ratatui:: path from the public API surface in this file.

Grep verified: `grep -rn 'pub fn.*tokio::sync::mpsc::Sender' src/`
and `grep -rn 'pub fn.*ratatui::layout::Position' src/` return zero
hits.

Verification: cargo fmt / cargo clippy --all-features -- -D warnings
/ cargo nextest run --all-features / cargo test --doc / cargo build
--no-default-features / cargo test --no-default-features --no-run all
clean. Migration table for MIGRATION.md v0.16→v0.17 populated in
Task 3.
EOF
)"
```

- [ ] Verify signature.

---

## Task 3: Unit 3 — CHANGELOG + MIGRATION.md updates

**Files:**
- Modify: `CHANGELOG.md`
- Modify: `MIGRATION.md`

**Interfaces:**
- Consumes: Task 1's 10 breaking renames + Task 2's `MessageSender<M>` newtype + AppHarness signature change. Task 3 renders the migration tables that document them.
- Produces: consumer-facing narrative for both units. Task 4's verification gauntlet reads these files as part of the full audit.

### Step 1: Update `CHANGELOG.md` under `[Unreleased]`

- [ ] Open `CHANGELOG.md`. Locate the `## [Unreleased]` block at approximately line 8.

- [ ] Under `### Breaking Changes`, append two new `#### ` sub-sub-sections at the bottom of the Breaking Changes list (after the existing entries like `#### App::init takes args; RuntimeBuilder split` and `#### FileSortDirection removed`):

```markdown
#### Component selection accessors unified on `selected_item()`

Six components had divergent accessor shapes for "which is selected?" — literal aliases (`selected_value()` / `selected_row()` / `selected()`), semantic outliers (`active_tab()` on tab_bar), and type-incoherent overloads (heatmap's `selected_value() -> f64` returning the data value at coordinates, not the selection itself). Unified on canonical `selected_item()` returning `Option<&T>` where the concept fits; renamed the semantic outliers to disambiguate:

- `dropdown::selected_value()` — deleted; use `selected_item() -> Option<&str>`
- `select::selected_value()` — deleted; use `selected_item() -> Option<&str>`
- `heatmap::selected_value() -> Option<f64>` — renamed to `value_at_selection() -> Option<f64>` (returns data value at cursor coords; not a selection accessor)
- `tab_bar::selected()` — deleted; use `selected_index()`
- `tab_bar::active_tab() -> Option<&Tab>` — renamed to `selected_item() -> Option<&Tab>`
- `tab_bar::active_tab_mut() -> Option<&mut Tab>` — renamed to `selected_item_mut() -> Option<&mut Tab>`
- `data_grid::selected()` — deleted; use `selected_index()`
- `data_grid::selected_row()` — deleted; use `selected_item()`
- `table::selected()` — deleted; use `selected_index()`
- `table::selected_row()` — deleted; use `selected_item()`

Closes audit finding #6 (`selected_value` / `selected_item` / `active_tab` divergence). See `MIGRATION.md#v016x-to-v0170` for the full migration table.

#### `MessageSender<M>` replaces `tokio::sync::mpsc::Sender` in AppHarness surface

`AppHarness::message_sender()` now returns a first-party `MessageSender<A::Message>` newtype instead of raw `tokio::sync::mpsc::Sender<A::Message>`. Consumers no longer need `tokio` as a direct dependency to use the accessor. Full tokio Sender semantics preserved through passthrough methods; an explicit `MessageSender::into_inner()` escape hatch is available for consumers needing `reserve` / `send_timeout` / `same_channel` / `downgrade` / `closed()` future.

Closes audit finding #8 (dependency leakage on AppHarness surface). See `MIGRATION.md#v016x-to-v0170` for the migration table.
```

- [ ] Under `### Added`, append at the bottom (after the existing entries):

```markdown
#### `MessageSender<M>` + `MessageSendError<T>` + `TrySendError<T>` (this cadence, Unit 2)

New first-party types in `envision::harness` (re-exported at crate root and prelude):

- `MessageSender<M: Send + 'static>` — newtype wrapper around `tokio::sync::mpsc::Sender<M>`. Full passthrough surface: `send()`, `try_send()`, `is_closed()`, `capacity()`, `max_capacity()`, plus `into_inner()` explicit escape hatch. `Clone + Debug + Send + Sync` (when `M: Send`).
- `MessageSendError<T>(pub T)` — returned by `send()` when the receiver has been dropped. Carries the message back so the caller can inspect it. Implements `Debug + Display + Error`.
- `TrySendError<T>::{Full(T), Closed(T)}` — returned by `try_send()`. Preserves tokio's Full/Closed distinction so consumers can retry-on-full or exit-on-closed with a match arm. Includes `into_inner(self) -> T` extractor.
```

- [ ] Under `### Known Deferred Findings`, REMOVE the entries for findings #6 and #8 (this cadence closes both). The section currently reads approximately:

```markdown
### Known Deferred Findings

The 2026-07-04 audit (Fable) surfaced two API incoherences deliberately deferred beyond v0.17.0. Both are tracked as follow-up cadences and will be addressed in v0.18.0 or later:

- **`selected_value` / `selected_item` / `active_tab` accessor shape divergence** across `dropdown`, `select`, `heatmap`, `tab_bar`, and `data_grid`. [...]
- **Dependency leakage in 8 public signatures** [...]

Both are tracked as follow-up cadences and will be addressed in v0.18.0 or later.
```

Replace with:

```markdown
### Known Deferred Findings

The 2026-07-04 audit surfaced items that remain deferred beyond v0.17.0:

- **`selected()` alias on ~15 additional components** (accordion, tabs, radio_group, searchable_list, selectable_list, tree, menu, box_plot, loading_list, alert_panel, and others) — these components have `selected() -> Option<usize>` as a literal alias for `selected_index()`, similar to the aliases just removed from tab_bar/data_grid/table in this release but without the additional divergence that motivated their inclusion this round. Scheduled for **Cadence D** (v0.18+): finish the consistency sweep across the remaining alias sites.

If audit findings #6 (selected_value/selected_item/active_tab across 5 originally-flagged components) and #8 (dependency leakage in 8 public signatures) previously appeared here, they were closed by the v0.17.0 consistency-cleanup cadence (see Breaking Changes / Added subsections above).
```

- [ ] Save.

### Step 2: Update `MIGRATION.md` under `## v0.16.x to v0.17.0`

- [ ] Open `MIGRATION.md`. Locate the `## v0.16.x to v0.17.0` section (approximately line 3).

- [ ] Append two new subsections at the BOTTOM of the v0.16→v0.17 section, AFTER the existing FileSortDirection and ResourceGaugeState migration tables from the release-readiness cadence:

```markdown
### Component selection accessors unified on `selected_item()`

Six components had divergent shapes for "which is selected?"; unified on canonical `selected_item()` returning `Option<&T>` where the concept fits. Semantic outliers renamed to disambiguate.

| Component | Old | New |
|---|---|---|
| `dropdown` | `state.selected_value()` | `state.selected_item()` (was already the canonical name; the alias is what's deleted) |
| `select` | `state.selected_value()` | `state.selected_item()` (same shape) |
| `heatmap` | `state.selected_value() -> Option<f64>` (returns data value at cursor coordinates) | `state.value_at_selection() -> Option<f64>` (renamed — the `value_` prefix disambiguates from selection-accessor pattern; the coordinate accessor `state.selected() -> Option<(usize, usize)>` is unchanged) |
| `tab_bar` | `state.active_tab() -> Option<&Tab>` | `state.selected_item() -> Option<&Tab>` |
| `tab_bar` | `state.active_tab_mut() -> Option<&mut Tab>` | `state.selected_item_mut() -> Option<&mut Tab>` |
| `tab_bar` | `state.selected() -> Option<usize>` (was: literal alias for `selected_index()`) | `state.selected_index()` |
| `data_grid` | `state.selected()` (was: literal alias for `selected_index()`) | `state.selected_index()` |
| `data_grid` | `state.selected_row()` (was: literal alias for `selected_item()`) | `state.selected_item()` |
| `table` | `state.selected()` (was: literal alias for `selected_index()`) | `state.selected_index()` |
| `table` | `state.selected_row()` (was: literal alias for `selected_item()`) | `state.selected_item()` |

Grep hint for consumers: search your codebase for `\.selected_value(`, `\.active_tab(`, `\.active_tab_mut(`, `\.selected_row(`, and `\.selected(` (the last only on tab_bar / data_grid / table state) — every hit needs to migrate to the new form per the table above.

### `MessageSender<M>` wraps `tokio::sync::mpsc::Sender<M>`

`AppHarness::message_sender()` now returns a first-party `MessageSender<M>` newtype instead of raw `tokio::sync::mpsc::Sender<M>`. Consumer code changes are limited to type spellings; call sites are identical.

| Old | New |
|---|---|
| `let sender: tokio::sync::mpsc::Sender<MyMsg> = harness.message_sender();` | `let sender: envision::MessageSender<MyMsg> = harness.message_sender();` (or use `envision::prelude::MessageSender`) |
| `sender.send(msg).await` — returns `Result<(), tokio::sync::mpsc::error::SendError<MyMsg>>` | `sender.send(msg).await` — returns `Result<(), envision::MessageSendError<MyMsg>>` |
| `sender.try_send(msg)` — returns `Result<(), tokio::sync::mpsc::error::TrySendError<MyMsg>>` | `sender.try_send(msg)` — returns `Result<(), envision::TrySendError<MyMsg>>` |
| `sender.is_closed()` — still available | `sender.is_closed()` — still available (passthrough) |
| `sender.capacity()` — still available | `sender.capacity()` — still available (passthrough) |
| `sender.max_capacity()` — still available | `sender.max_capacity()` — still available (passthrough) |
| `sender.reserve()`, `.send_timeout()`, `.same_channel()`, `.downgrade()`, `.closed()` future | `let tokio_sender = sender.into_inner();` — explicit escape hatch that re-couples to tokio |

`MessageSender<M>` is `Clone + Debug + Send + Sync` (when `M: Send`). Its `send`/`try_send` methods have the same semantics as tokio's Sender (bounded channel, receiver-dropped errors carry the message back).

**Generic parameter note:** `MessageSender<M>` is parameterized on the message type `M`, not on an App type. This means portable helper functions like `fn spawn_watcher<M: Send + 'static>(sender: MessageSender<M>) { ... }` work without depending on envision's `App` trait.
```

- [ ] Save.

### Step 3: Commit Task 3

- [ ] Stage:

```bash
git add CHANGELOG.md MIGRATION.md
```

- [ ] Commit:

```bash
git commit -S -m "$(cat <<'EOF'
CHANGELOG + MIGRATION.md: consistency-cleanup cadence entries

Add Breaking Changes + Added sub-sub-sections to CHANGELOG's
[Unreleased] block for the consistency-cleanup cadence (Task 1 +
Task 2). Trim Known Deferred Findings to remove findings #6 and #8
(closed by this cadence) and preserve the Cadence D commitment for
the remaining `selected()` alias sites on ~15 other components.

Append two new migration tables to MIGRATION.md v0.16→v0.17:

1. Component selection accessors unified on selected_item() —
   10-row table covering the dropdown/select/heatmap/tab_bar/
   data_grid/table renames, plus a grep hint for consumers.

2. MessageSender<M> wraps tokio::sync::mpsc::Sender<M> — 7-row
   table covering send/try_send/is_closed/capacity/max_capacity
   passthroughs + the escape-hatch pattern for tokio-specific
   functionality (reserve, send_timeout, same_channel, downgrade,
   closed() future).

Cross-references: CHANGELOG entries link to MIGRATION.md#v016x-
to-v0170. MIGRATION.md tables cross-reference the affected APIs.

No code changes; documentation-only commit. Verified via
grep for orphaned migration table refs and CHANGELOG structure.
EOF
)"
```

- [ ] Verify signature.

---

## Task 4: Verification gauntlet + audit scorecard preservation

**Files:** none directly — this is a full-suite verification pass on the accumulated state after Tasks 1-3.

**Interfaces:**
- Consumes: three signed commits from Tasks 1-3.
- Produces: verification-clean report suitable for the impl PR body.

### Step 1: Full verification gauntlet

- [ ] Run each command and confirm the expected output:

```bash
cargo fmt --check
```
Expected: no output (clean).

```bash
cargo clippy --all-features -- -D warnings 2>&1 | tail -10
```
Expected: no warnings (`Finished` line only).

```bash
cargo nextest run --all-features 2>&1 | tail -10
```
Expected: all tests pass. Full pass count (unit + integration) should be within a few tests of the pre-cadence baseline (~10K), with a small decrease from the tautology-test deletions.

```bash
cargo test --all-features --doc 2>&1 | tail -10
```
Expected: all doc tests pass. Doc-test count increased by ~5-10 due to new `MessageSender` + `heatmap::value_at_selection` + `tab_bar::selected_item*` examples.

```bash
cargo build --no-default-features 2>&1 | tail -3
```
Expected: `Finished` (clean).

```bash
cargo test --no-default-features --no-run 2>&1 | tail -5
```
Expected: `Finished` (clean). Catches example-gating drift (D8 lesson).

```bash
cargo build --examples --all-features 2>&1 | tail -3
```
Expected: `Finished` (clean).

```bash
cargo doc --no-deps --all-features 2>&1 | grep -iE "warning|error" | head -10
```
Expected: zero intra-doc-link warnings.

```bash
./tools/audit/target/release/envision-audit all 2>&1 | grep -iE "scorecard|baseline|PASS|FAIL" | head -15
```
Expected: 9/9 scorecard PASS, all baseline checks PASS. No regressions from the pre-cadence state.

### Step 2: Grep-verify final callsite-form gates

- [ ] Run the four callsite-form grep gates:

```bash
echo "--- .selected_value( ---"
grep -rn '\.selected_value(' src/ tests/ examples/
echo "--- .active_tab( ---"
grep -rn '\.active_tab(' src/ tests/ examples/
echo "--- .active_tab_mut( ---"
grep -rn '\.active_tab_mut(' src/ tests/ examples/
echo "--- .selected_row( ---"
grep -rn '\.selected_row(' src/ tests/ examples/
```

Expected: zero hits from all four (CHANGELOG.md and MIGRATION.md are NOT in these paths; migration-table prose there is expected and not caught by this grep).

- [ ] Run the .selected() gate scoped to the 6 in-scope components:

```bash
echo "--- .selected() in 6 in-scope components ---"
grep -rn '\.selected()' src/component/dropdown/ src/component/select/ src/component/heatmap/ src/component/tab_bar/ src/component/data_grid/ src/component/table/
```

Expected: hits ONLY in `src/component/heatmap/` (heatmap's `.selected() -> Option<(usize, usize)>` is a genuine coordinate accessor, NOT an alias — retained). Zero hits in the other five components.

- [ ] Confirm dep-leakage residual:

```bash
grep -rn 'pub fn.*tokio::sync::mpsc::Sender' src/
```

Expected: zero hits.

```bash
grep -rn 'pub fn.*ratatui::layout::Position' src/
```

Expected: zero hits.

### Step 3: No commit for Task 4

Task 4 is verification-only. Tasks 1-3 produce the three signed commits; Task 4 confirms they compose cleanly. No commit here. If any verification fails, backtrack to the offending task and fix.

---

## Task 5: Push impl branch + open impl PR

**Files:** none — mechanical push + `gh pr create`.

### Step 1: Confirm branch state

- [ ] Run:

```bash
git log --oneline -5
```

Expected (in order, most recent first):
- Task 3 (CHANGELOG + MIGRATION) commit
- Task 2 (message-sender) commit
- Task 1 (consistency-sweep) commit
- Whatever the branch parent was — should be at or near current `main`

If any commit is missing or misordered, STOP and reconcile.

### Step 2: Merge latest `origin/main` into the impl branch

- [ ] Run:

```bash
git fetch origin main
git merge origin/main --no-ff -S -m "Merge origin/main into consistency-cleanup-impl"
```

If merge conflicts (unlikely since the release-readiness cadence just landed and no other work has), resolve them (CHANGELOG.md is the most likely conflict candidate; both cadences append to `[Unreleased]`).

If signing the merge commit fails, STOP and ask the user.

### Step 3: Push impl branch

- [ ] Run:

```bash
git push -u origin consistency-cleanup-impl
```

### Step 4: Open impl PR

- [ ] Run:

```bash
gh pr create --title "Impl: consistency-cleanup cadence (v0.17.0 pre-release, Cadence A)" --body "$(cat <<'EOF'
## Summary

Closes two of the top trust-eroding findings from the 2026-07-05 audit before cutting v0.17.0. Both are breaking; both bundle cleanly into the same `MIGRATION.md` `v0.16.x to v0.17.0` section established by the release-readiness cadence.

- **Unit 1 — Consistency sweep** on `selected_item()` across 6 components (dropdown, select, heatmap, tab_bar, data_grid, table). Delete literal aliases; rename semantically distinct `heatmap::selected_value()` → `value_at_selection()`; rename `tab_bar::active_tab*()` → `selected_item*()`. 10 breaking renames total. Tautology alias-equivalence tests DELETED. Private `data_grid::selected_row` field renamed to `selected_row_index` to break grep collision.
- **Unit 2 — `MessageSender<M>` newtype** wraps `tokio::sync::mpsc::Sender<A::Message>` at `AppHarness::message_sender()`. Parameterized on message type `M: Send + 'static` (not App type — avoids envision-specific trait bound propagation). Full passthrough surface + `into_inner()` explicit escape hatch. First-party `MessageSendError<T>` + `TrySendError<T>::{Full,Closed}` error types. Plus one cosmetic: `virtual_terminal.rs:147` swaps `ratatui::layout::Position` for `crate::layout::Position` (zero runtime change).
- **Unit 3 — CHANGELOG + MIGRATION.md updates**. Findings #6 and #8 come OUT of Known Deferred Findings (this cadence closes both). Two new migration tables under v0.16→v0.17. Cadence D committed as follow-up for the ~15 other components with `selected()` alias.

## Principled deviations from plan

If any surfaced during impl, list them here. Task 1 and Task 2 are mechanical enough that deviations are unlikely; if any occurred (e.g., Table's `selected_row` field turned out to have a different name), the specific difference goes here.

## Spec / plan

- Spec: `docs/superpowers/specs/2026-07-05-consistency-cleanup-cadence-design.md` (PR #506)
- Plan: `docs/superpowers/plans/2026-07-05-consistency-cleanup-cadence.md` (PR TBD — plan PR)

## Design decisions from brainstorm + adversarial review

- Bundle #1 (consistency) + #2 (dep-leakage) as ONE cadence — both breaking, both fit the same MIGRATION.md v0.16→v0.17 section.
- Canonical accessor: `selected_item()` across all 6 in-scope components (Table added per adversarial review M3 — byte-identical divergence to data_grid).
- Heatmap rename: `value_at_selection()` (S2 — the `value_` prefix sorts distinctly from `selected_*` in IDE autocomplete).
- `MessageSender<M>` parameterized on message type, NOT on App (S1 — avoids envision-specific `App` trait bound propagation).
- `MessageSender` API expanded to include `is_closed()` + `capacity()` + `max_capacity()` passthroughs + `into_inner()` escape hatch (M4 — original draft dropped half of tokio Sender's API surface without replacement).
- Delete outright, no `#[deprecated]` — matches D5/D14/G7/D12/D3/D8/resource_gauge/FileSortDirection pre-1.0 precedent.
- Grep gates use callsite forms (M1+M2 — token-boundary grep collides with private fields and downstream example state).
- Tautology alias-equivalence tests DELETED not renamed (A2).

## Test plan

- [x] Unit 1: 10 breaking renames verified via callsite-form grep gates (zero hits in `src/` `tests/` `examples/`).
- [x] Unit 1: tautology alias-equivalence tests deleted; ~4-6 fewer tests but no coverage loss (the deletions were assertions of `x == x` after alias removal).
- [x] Unit 2: `MessageSender<M>` compile-time `Send + Sync + Clone` gate via inline shim.
- [x] Unit 2: 12 unit tests for `MessageSender` (send round trip, try_send full, send/try_send closed, is_closed, capacity, clone, into_inner, error variants).
- [x] Unit 2: `AppHarness::message_sender()` return type wraps tokio Sender.
- [x] Unit 2: `virtual_terminal.rs:147` uses `crate::layout::Position` (zero runtime change, verified via `grep 'pub fn.*ratatui::layout::Position' src/` returning zero hits).
- [x] `cargo fmt --check` — clean
- [x] `cargo clippy --all-features -- -D warnings` — clean
- [x] `cargo nextest run --all-features` — all pass
- [x] `cargo test --all-features --doc` — all pass (new MessageSender + heatmap + tab_bar doc-tests)
- [x] `cargo build --no-default-features` — clean
- [x] `cargo test --no-default-features --no-run` — clean (D8 lesson)
- [x] `cargo build --examples --all-features` — clean
- [x] `cargo doc --no-deps --all-features` — zero intra-doc-link warnings
- [x] `./tools/audit/target/release/envision-audit all` — 9/9 scorecard preserved

## Next steps after this PR merges

- Tracking-doc PR (short) marking audit findings #6 and #8 CLOSED. Checks in a small verification record at `docs/audits/2026-07-05-post-consistency-cleanup.md`.
- Cadence B (doc-hygiene split of CHANGELOG.md + MIGRATION.md > 1000 lines). Separate brainstorm.
- v0.17.0 release via `/release minor`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

Expected: PR URL returned.

### Step 5: CI watch

- [ ] Run `gh pr checks <PR_NUMBER>` periodically until all required checks complete. Coverage's tarpaulin timeout is a known flake — retrigger with `gh run rerun <RUN_ID> --failed` if it fires.

- [ ] If required checks pass, the impl PR is ready for review and merge.

- [ ] If required checks fail, diagnose the root cause and fix in a follow-up signed commit on the same branch.

Do not attempt to merge until required checks pass.

### Step 6: Merge after approval

- [ ] After the tracking-doc PR is opened and approved:

```bash
gh pr merge <PR_NUMBER> --squash --delete-branch
```

- [ ] After merge, the impl is complete. Next: tracking-doc PR (NOT part of this plan; opened on a separate branch after this PR lands).

---

## Out of scope for this plan

- Tracking-doc PR closure (separate branch + PR after impl merges — marks audit findings #6 and #8 resolved in a new `docs/audits/2026-07-05-post-consistency-cleanup.md` verification record).
- Cadence B (doc-hygiene split of CHANGELOG.md and MIGRATION.md over 1000-line human cap) — separate brainstorm.
- Cadence D (`selected()` alias removal from the ~15 other components: accordion, tabs, radio_group, searchable_list, selectable_list, tree, menu, box_plot, loading_list, alert_panel, etc.) — deferred to v0.18+ per spec.
- `/release minor` for v0.17.0 — dispatched after tracking-doc + any pending cadence B work land.

## Recovery patterns from prior cadences

- **`git commit -S` fails** → STOP. Ask the user. Never bypass with `--no-gpg-sign`.
- **`cargo fmt --check` drifts mid-task** → Run `cargo fmt`, stage, add a small follow-up signed commit (e.g., `fmt: cargo fmt drift after Task N`). Don't amend.
- **`data_grid::selected_row` field rename breaks internal callers** → grep for `\bself\.selected_row\b` inside `src/component/data_grid/` and migrate to `self.selected_row_index`. The struct field is private, so this is contained.
- **Table's private field is NOT named `selected_row`** → leave it alone; Step 8's rename is conditional. Just delete the two alias methods.
- **`MessageSender` unit tests fail with `channel full`** → tokio channels of size 1 may need a small yield to fill. If the `test_try_send_full` test needs adjustment, use `channel(2)` and fill both slots — the intent is Full-variant coverage.
- **`static_assertions` NOT in dev-deps** → the plan says use an inline shim; do NOT add the dep. If the shim fails to catch Send/Sync issues at compile time, that's evidence the shim is written wrong — investigate.
- **Coverage CI check flakes with tarpaulin timeout after tests pass** → `gh run rerun <RUN_ID> --failed`. Coverage is not required for merge per branch protection.
- **README doctest fails after `MessageSender` re-export** → README uses `envision::prelude::*`; verify `MessageSender` is in the prelude line at `src/lib.rs:476`. If a README code block references `MessageSender`, verify it compiles under `--features full` (which is default).

## Reference

- Spec: `docs/superpowers/specs/2026-07-05-consistency-cleanup-cadence-design.md` (PR #506; commits `4a27aab` + `912b268`).
- 2026-07-05 audit report (in-session `/audit`): `A-` (3.81 GPA). Findings #1 (consistency) and #2 (dep-leakage) in the trust-eroders list closed by this cadence.
- 2026-07-04 Fable audit: `A-` (3.62 GPA). Findings #6 and #8 originally deferred; closed here.
- 2026-07-05 Fable re-audit: `A` (3.91 GPA). Verified deferred findings as honestly documented.
- Precedent for delete-outright pre-1.0: D5 paragraph→line rename, D14 same, resource_gauge::new deletion, FileSortDirection deletion (release-readiness cadence). Pattern is outright deletion + MIGRATION.md table + no `#[deprecated]` shim.
- Cadence pattern: brainstorm → spec PR → plan PR → impl PR → tracking-doc PR (established across 12+ prior cadences).
- CLAUDE.md project rules: PRs required; signed commits; squash-merge; merge `origin/main` before push; files under 1000 lines; no clippy warnings; no TODOs without tracking doc.
