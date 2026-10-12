# PR 5 — Inline-viewport correctness and opt-in `scrolling-regions` Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:**
- `CaptureBackend` and `DualBackend` handle `append_lines` correctly; `DualBackend` keeps its capture cursor in sync with the primary's.
- An opt-in envision `scrolling-regions` feature adds `scroll_region_up/down` with `TestBackend` semantics.

**Architecture:**
- **Helpers.** New `src/backend/capture/scroll.rs` holds the row-shifting helpers: shift cells, reset vacated rows, move the per-row owner flags, stamp the frame.
- **`append_lines`** is always compiled and uses the upward shift.
- **Scroll methods** are compiled only under `cfg(feature = "scrolling-regions")`, which forwards to `ratatui/scrolling-regions`.
- **`DualBackend`** forwards everything, primary first.

**Tech Stack:** Rust 1.88, ratatui 0.30.2 (`ratatui-core` 0.1.2 `Backend` trait), proptest.

**Spec:** `docs/superpowers/specs/2026-10-12-v018-programme-completion-design.md` § PR 5. Its "Implementation notes" subsection is binding.

## Global Constraints

- **Feature:** `scrolling-regions = ["ratatui/scrolling-regions"]` in `[features]`, **not** in `default` or `full`.
- **Degenerate input:** inverted regions (`start > end`), empty regions and `n == 0` are no-ops that touch nothing. They never panic.
- **After a scroll:** the cursor is unchanged. Every cell in the region's rows is stamped with the current frame. Vacated rows become `EnhancedCell::new()`. `row_may_hold_owner` entries move with their rows, and vacated rows get `false`.
- **`append_lines` follows `TestBackend`** (`ratatui-core-0.1.2/src/backend/test.rs:326-361`). Overflow scrolls the whole buffer up; rows that leave the top are discarded, since there is no scrollback. The cursor ends at `((x+1).min(w-1), (y+n).min(h-1))`.
- **`DualBackend::append_lines`** in order:
  1. `primary.get_cursor_position()?` (fails before anything changes);
  2. set the capture cursor;
  3. `primary.append_lines(n)?`;
  4. `let Ok(()) = capture.append_lines(n)`.
- **Code limits:** no `unsafe`, no warnings (also clippy with the feature **off**), files under 1000 lines (`capture/mod.rs` is ~735; new code goes in `capture/scroll.rs`).
- **Commits** are signed, with the `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` trailer. Branch `v018-scroll-regions`.

## Review Focus

1. **Scrolling a row containing a wide glyph keeps rule 3 working.** A non-blank write into its continuation column after the scroll still blanks the owner. Task 3 test.
2. **`append_lines` with `n` near `u16::MAX`** saturates and never panics or overflows indices (mirrors `TestBackend`'s test at `test.rs:931`). Task 2 test.
3. **A 0-height or 0-width backend** makes every method a no-op without panicking. Task 2 and Task 3 tests.
4. **`DualBackend` primary error.** If `get_cursor_position` fails, neither half is modified. Task 2 test.
5. **Feature-off build.** `cargo clippy --all-targets --features serialization,full -- -D warnings` stays clean, with no dead-code warnings from scroll helpers. Task 3 Step 5.

---

### Task 1: Row-shift helpers

**Files:**
- Create: `src/backend/capture/scroll.rs`
- Modify: `src/backend/capture/mod.rs` (add `mod scroll;`)
- Test: `src/backend/capture/scroll.rs` (`#[cfg(test)] mod tests`)

**Interfaces — Produces:**
- `pub(super) fn shift_rows_up(&mut self, start: u16, end: u16, n: u16)`. Always compiled.
- `#[cfg(feature = "scrolling-regions")] pub(super) fn shift_rows_down(&mut self, start: u16, end: u16, n: u16)`.

- [ ] **Step 1: Write failing unit tests** in `scroll.rs`:

```rust
#[cfg(test)]
mod tests {
    use crate::backend::CaptureBackend;
    use ratatui::backend::Backend;
    use ratatui::buffer::Cell;

    /// Rows "aa…", "bb…", … drawn through the public `Backend::draw`.
    fn grid(w: u16, h: u16) -> CaptureBackend {
        let mut b = CaptureBackend::new(w, h);
        let mut cells = Vec::new();
        for y in 0..h {
            let symbol = char::from(b'a' + y as u8).to_string();
            for x in 0..w {
                let mut cell = Cell::default();
                cell.set_symbol(&symbol);
                cells.push((x, y, cell));
            }
        }
        let Ok(()) = b.draw(cells.iter().map(|(x, y, c)| (*x, *y, c)));
        b
    }

    fn rows(b: &CaptureBackend) -> Vec<String> {
        (0..b.height()).map(|y| b.row_content(y)).collect()
    }

    #[test]
    fn shift_up_moves_rows_and_blanks_bottom() {
        let mut b = grid(2, 4);
        let Ok(()) = b.flush();
        b.shift_rows_up(1, 4, 1);
        assert_eq!(rows(&b), ["aa", "cc", "dd", "  "]);
        assert!((0..2).all(|x| b.cell(x, 1).unwrap().last_modified_frame() == b.current_frame()));
        assert_eq!(b.cell(0, 0).unwrap().last_modified_frame(), 0, "row outside region untouched");
    }

    #[test]
    fn shift_up_by_region_height_or_more_clears_region() {
        let mut b = grid(2, 3);
        b.shift_rows_up(0, 3, 9);
        assert_eq!(rows(&b), ["  ", "  ", "  "]);
    }

    #[test]
    fn degenerate_inputs_are_noops() {
        for (s, e, n) in [(2, 1, 1), (1, 1, 1), (0, 3, 0), (5, 9, 1)] {
            let mut b = grid(2, 3);
            let Ok(()) = b.flush();
            b.shift_rows_up(s, e, n);
            assert_eq!(rows(&b), ["aa", "bb", "cc"], "({s},{e},{n})");
            assert!(b.cells().iter().all(|c| c.last_modified_frame() == 0));
        }
    }

    #[test]
    fn zero_sized_backend_is_noop() {
        let mut b = CaptureBackend::new(0, 0);
        b.shift_rows_up(0, 5, 3);
    }
}
```

- [ ] **Step 2: Run, expect RED** (`no method shift_rows_up`).

- [ ] **Step 3: Implement** in `scroll.rs`:

```rust
//! Row shifting for `append_lines` and scrolling regions (spec PR 5).

use super::CaptureBackend;
use crate::backend::cell::EnhancedCell;

impl CaptureBackend {
    /// Shifts rows `[start, end)` up by `n`. Vacated bottom rows become reset
    /// blanks; every cell in the region is stamped with the current frame;
    /// per-row owner flags move with their rows. Degenerate input (inverted or
    /// empty region, `n == 0`) is a no-op.
    pub(super) fn shift_rows_up(&mut self, start: u16, end: u16, n: u16) {
        let Some((s, e, by)) = self.region_bounds(start, end, n) else { return };
        let w = usize::from(self.width);
        self.cells[s * w..e * w].rotate_left(by * w);
        self.cells[(e - by) * w..e * w].fill_with(EnhancedCell::new);
        self.row_may_hold_owner[s..e].rotate_left(by);
        self.row_may_hold_owner[e - by..e].fill(false);
        self.stamp_rows(s, e);
    }

    /// Mirror of [`shift_rows_up`](Self::shift_rows_up): rows move down and
    /// vacated top rows become reset blanks.
    #[cfg(feature = "scrolling-regions")]
    pub(super) fn shift_rows_down(&mut self, start: u16, end: u16, n: u16) {
        let Some((s, e, by)) = self.region_bounds(start, end, n) else { return };
        let w = usize::from(self.width);
        self.cells[s * w..e * w].rotate_right(by * w);
        self.cells[s * w..(s + by) * w].fill_with(EnhancedCell::new);
        self.row_may_hold_owner[s..e].rotate_right(by);
        self.row_may_hold_owner[s..s + by].fill(false);
        self.stamp_rows(s, e);
    }

    /// Clamped `(start_row, end_row, shift)` in rows, or `None` for no-ops.
    fn region_bounds(&self, start: u16, end: u16, n: u16) -> Option<(usize, usize, usize)> {
        if start > end || n == 0 || self.width == 0 {
            return None;
        }
        let (s, e) = (usize::from(start.min(self.height)), usize::from(end.min(self.height)));
        (s < e).then(|| (s, e, usize::from(n).min(e - s)))
    }

    fn stamp_rows(&mut self, s: usize, e: usize) {
        let w = usize::from(self.width);
        let frame = self.current_frame;
        for cell in &mut self.cells[s * w..e * w] {
            cell.set_last_modified_frame(frame);
        }
    }
}
```

Add `mod scroll;` in `capture/mod.rs` next to `mod wide;`. Check the vacated cells: `fill_with(EnhancedCell::new)` creates cells with frame 0, and `stamp_rows` then stamps the whole region, vacated rows included. The tests assert this.

- [ ] **Step 4: GREEN.** Run `cargo nextest run --all-features -E 'test(/capture::scroll/)'`, then `cargo build --features serialization,full` (feature off), which must have no dead-code warning.
- [ ] **Step 5: Commit** with signature: "CaptureBackend: row-shift helpers for append_lines and scroll regions".

---

### Task 2: `append_lines` for `CaptureBackend` and `DualBackend`

**Files:**
- Modify: `src/backend/capture/mod.rs` (`impl Backend`: add `append_lines`)
- Modify: `src/adapter/dual/mod.rs` (`impl Backend`: add `append_lines`)
- Test: `tests/capture_append_lines.rs` (create)

**Interfaces:**
- Consumes: `shift_rows_up` (Task 1).
- Produces: `Backend::append_lines` on both types.

- [ ] **Step 1: Failing tests** in `tests/capture_append_lines.rs`:

```rust
//! append_lines mirrors ratatui's TestBackend (spec PR 5).
use envision::adapter::DualBackend;
use envision::backend::Backend;
use envision::ratatui::backend::TestBackend;
use envision::ratatui::layout::Position;
use envision::CaptureBackend;

fn cursor_after(w: u16, h: u16, start: Position, n: u16) -> (Position, Position) {
    let mut c = CaptureBackend::new(w, h);
    let mut t = TestBackend::new(w, h);
    let Ok(()) = c.set_cursor_position(start);
    let Ok(()) = t.set_cursor_position(start);
    let Ok(()) = c.append_lines(n);
    let Ok(()) = t.append_lines(n);
    (c.cursor_position(), t.get_cursor_position().unwrap())
}

#[test]
fn cursor_matches_test_backend() {
    for (start, n) in [((0, 0), 1), ((3, 1), 2), ((4, 4), 3), ((0, 2), 0), ((9, 9), 1)] {
        let (c, t) = cursor_after(5, 5, Position::new(start.0, start.1), n);
        assert_eq!(c, t, "start {start:?} n {n}");
    }
}

fn grid(w: u16, h: u16) -> CaptureBackend {
    let mut b = CaptureBackend::new(w, h);
    let mut cells = Vec::new();
    for y in 0..h {
        let symbol = char::from(b'a' + y as u8).to_string();
        for x in 0..w {
            let mut cell = envision::ratatui::buffer::Cell::default();
            cell.set_symbol(&symbol);
            cells.push((x, y, cell));
        }
    }
    let Ok(()) = b.draw(cells.iter().map(|(x, y, c)| (*x, *y, c)));
    b
}

#[test]
fn overflow_scrolls_buffer_up_and_blanks_bottom() {
    let mut c = grid(3, 3); // rows "aaa", "bbb", "ccc"
    let Ok(()) = c.set_cursor_position(Position::new(0, 2));
    let Ok(()) = c.append_lines(1);
    assert_eq!(c.row_content(0), "bbb");
    assert_eq!(c.row_content(2), "   ");
}

#[test]
fn huge_line_count_saturates() {
    let mut c = CaptureBackend::new(4, 3);
    let Ok(()) = c.append_lines(u16::MAX);
    assert_eq!(c.cursor_position().y, 2);
}

#[test]
fn zero_sized_backend_is_noop() {
    let mut c = CaptureBackend::new(0, 0);
    let Ok(()) = c.append_lines(5);
}

#[test]
fn dual_syncs_capture_cursor_from_primary() {
    let mut primary = CaptureBackend::new(4, 3);
    let Ok(()) = primary.set_cursor_position(Position::new(0, 2));
    let capture = CaptureBackend::new(4, 3); // cursor at origin: stale
    let mut dual = DualBackend::new(primary, capture);
    let Ok(()) = dual.append_lines(1);
    assert_eq!(dual.primary().cursor_position(), dual.capture().cursor_position());
}
```

Add a `DualBackend` error-ordering test using a primary whose `get_cursor_position` returns `Err`. Reuse the `NoSize`-style pattern from `tests/backend_error_types.rs`, with its own fixture. Assert the capture half is unchanged (cursor and cells).

- [ ] **Step 2: RED.** The cursor/content tests fail because the trait default `append_lines` is a no-op.
- [ ] **Step 3: Implement.**

`CaptureBackend` (in `impl Backend`):

```rust
    fn append_lines(&mut self, n: u16) -> Result<(), Infallible> {
        // Mirrors ratatui's TestBackend (ratatui-core 0.1.2 backend/test.rs) minus
        // scrollback: rows leaving the top are discarded. The x+1 cursor step is
        // TestBackend's convention (crossterm keeps x in raw mode); mirrored so
        // captures match ratatui's reference backend.
        if self.width == 0 || self.height == 0 {
            return Ok(());
        }
        let Position { x, y } = self.cursor_position;
        let max_y = self.height - 1;
        let lines_after_cursor = max_y.saturating_sub(y);
        if n > lines_after_cursor {
            self.shift_rows_up(0, self.height, n - lines_after_cursor);
        }
        self.cursor_position = Position::new(
            x.saturating_add(1).min(self.width - 1),
            y.saturating_add(n).min(max_y),
        );
        Ok(())
    }
```

`DualBackend` (in `impl<P: Backend> Backend`):

```rust
    /// Forwards to both halves. Reads the primary's cursor first (a real
    /// terminal round-trip for crossterm) and syncs the capture half to it, so
    /// both halves scroll from the same position; a failed read leaves both
    /// halves untouched.
    fn append_lines(&mut self, n: u16) -> Result<(), P::Error> {
        let position = self.primary.get_cursor_position()?;
        let Ok(()) = self.capture.set_cursor_position(position);
        self.primary.append_lines(n)?;
        let Ok(()) = self.capture.append_lines(n);
        Ok(())
    }
```

- [ ] **Step 4: GREEN** on `tests/capture_append_lines.rs` and the full suite. Inline-viewport behaviour has no existing envision tests or snapshots (PR 5 specialist), so nothing else should change.
- [ ] **Step 5: Commit** with signature: "append_lines for CaptureBackend and DualBackend (with capture cursor sync)".

---

### Task 3: Opt-in `scrolling-regions` feature

**Files:**
- Modify: `Cargo.toml` (`[features]`)
- Modify: `src/backend/capture/mod.rs`, `src/adapter/dual/mod.rs` (cfg'd trait methods)
- Modify, adding cfg'd methods to in-repo backends:
  - `tests/backend_errors.rs:30`
  - `tests/backend_error_types.rs:30`
  - `tests/trybuild_backend/foreign_error.rs:26`
  - `src/error.rs` doctests (~109, ~147)
  - the `capture_append_lines.rs` error fixture
- Test: `tests/scroll_regions.rs` (create, `#![cfg(feature = "scrolling-regions")]`)

- [ ] **Step 1: Add the feature.** In `[features]`, add `scrolling-regions = ["ratatui/scrolling-regions"]`. Do not add it to `default` or `full`. Run `cargo tree -e features --features scrolling-regions -i ratatui-core | head` to confirm, and confirm `Cargo.lock` is unchanged.

- [ ] **Step 2: Failing tests** in `tests/scroll_regions.rs`:
  - **Interior region:** `scroll_region_up(1..3, 1)` on rows `a,b,c,d` gives `a,c, ,d`. `scroll_region_down(1..3, 1)` gives `a, ,b,d`.
  - **Region at row 0:** the top row is discarded.
  - **`n` ≥ height:** the region is cleared.
  - **Degenerate:** `scroll_region_up(3..1, 1)` (written as `std::ops::Range { start: 3, end: 1 }`), `1..1` and `n = 0` change nothing and stamp nothing.
  - **Cursor:** unchanged after each scroll.
  - **Frame stamps:** after a `flush()`, the region's rows carry the current frame and rows outside it do not.
  - **Owner flags (Review Focus 1):**
    1. draw `中` at (0,1) with a 4×4 backend;
    2. `scroll_region_up(0..4, 1)`, so `中` is now at (0,0);
    3. draw `z` at (1,0);
    4. assert (0,0) is blanked (rule 3) and the row reads `" z  "`.
  - **`DualBackend`:** both scroll methods reach both halves; check each half's rows.
  - **Oracle, proptest:**
    1. generate a width and height of 1..8, rows of **width-1 ASCII** symbols with a random `fg` from a small palette, a region with `start ≤ end ≤ h+2`, `n` in 0..10, and up or down;
    2. apply the same draws and scroll to `CaptureBackend` and `ratatui::backend::TestBackend`;
    3. for every (x,y), `EnhancedCell::from_ratatui_cell(&test_backend.buffer()[(x, y)], 0)` must equal the capture cell in **symbol, fg, bg and modifiers**. Compare via the accessors; frames are ignored.
  - **Self-relative oracle:** with wide glyphs allowed, post-state row `r` equals pre-state row `σ(r)` (the shift mapping) or is all reset blanks.
  - **Integration (spec PR 5 Tests):**
    1. `Terminal::with_options(DualBackend::new(CaptureBackend::new(20, 10), CaptureBackend::new(20, 10)), TerminalOptions { viewport: Viewport::Inline(3) })`;
    2. `insert_before(1, |buf| Paragraph::new("hello").render(buf.area, buf))`;
    3. assert that the capture half contains "hello".

    Put this test **outside** the cfg (in `tests/capture_append_lines.rs`) so it runs both without and with the feature.

- [ ] **Step 3: RED** with `cargo nextest run --features scrolling-regions,serialization,full --test scroll_regions`. Without the impls this fails to compile with E0046, which is the RED evidence. Record it.

- [ ] **Step 4: Implement**, in `impl Backend for CaptureBackend`:

```rust
    #[cfg(feature = "scrolling-regions")]
    fn scroll_region_up(&mut self, region: std::ops::Range<u16>, n: u16) -> Result<(), Infallible> {
        self.shift_rows_up(region.start, region.end, n);
        Ok(())
    }

    #[cfg(feature = "scrolling-regions")]
    fn scroll_region_down(&mut self, region: std::ops::Range<u16>, n: u16) -> Result<(), Infallible> {
        self.shift_rows_down(region.start, region.end, n);
        Ok(())
    }
```

In `DualBackend`:

```rust
    #[cfg(feature = "scrolling-regions")]
    fn scroll_region_up(&mut self, region: std::ops::Range<u16>, n: u16) -> Result<(), P::Error> {
        self.primary.scroll_region_up(region.clone(), n)?;
        let Ok(()) = self.capture.scroll_region_up(region, n);
        Ok(())
    }
```

`scroll_region_down` is the same, using `scroll_region_down`.

In the **in-repo backends**, add `#[cfg(feature = "scrolling-regions")]` no-op impls returning `Ok(())`, in their error type. In the `src/error.rs` doctests, add them as hidden `# ` lines with the same cfg attribute.

**Verify that cfg works in doctests and in trybuild fixtures:** run `cargo test --doc --features scrolling-regions,serialization,full error` and `cargo nextest run --features scrolling-regions,serialization,full --test trybuild_pass`, and the same without the feature. If rustdoc does not pass crate features into doctests, STOP and report. The controller will rule; do not delete the doctests. Also run `cargo +nightly test --doc --features scrolling-regions,serialization,full error`, and confirm the `compile_fail,E0277` doctest still fails only with E0277.

Update rustdoc: `CaptureBackend` and `DualBackend` type docs gain the line "`scroll_region_up`/`scroll_region_down` require the `scrolling-regions` feature."

- [ ] **Step 5: GREEN, both ways:**

```bash
cargo nextest run --all-features --profile ci                                   # feature on (via all-features)
cargo nextest run --features serialization,full --profile ci                    # feature off
cargo nextest run --no-default-features --profile ci
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --all-targets --features serialization,full -- -D warnings         # feature-off clippy (Review Focus 5)
cargo test --doc --all-features && cargo test --doc --features serialization,full
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
wc -l src/backend/capture/*.rs src/adapter/dual/mod.rs                          # each < 1000
```

- [ ] **Step 6: Commit** with signature: "Opt-in scrolling-regions feature: CaptureBackend/DualBackend scroll_region_up/down".

---

### Task 4: Docs, PR, merge

- [ ] **Step 1: CHANGELOG `[Unreleased]`.**
  - **Added:** the `scrolling-regions` feature; `append_lines` support for `CaptureBackend`/`DualBackend`.
  - **Fixed:** `DualBackend` previously dropped `append_lines`, so inline-viewport output was missing on the real terminal.
  - Replace #524's "`scrolling-regions` is not yet supported" text in two places:
    - the bold "Supported ratatui features" paragraph under `#### ratatui re-exports` (~`CHANGELOG.md:77`);
    - the Known Deferred Findings bullet (~`:115`).

    The new text: supported via envision's opt-in feature. Enabling `ratatui/scrolling-regions` directly without `envision/scrolling-regions` still fails with E0046; the fix is to enable envision's feature. With the feature, inline `insert_before` uses scroll regions, which means less flicker, but it errors on legacy non-VT Windows consoles (crossterm `ScrollUpInRegion` → `Unsupported`).
- [ ] **Step 2: MIGRATION.**
  - § 7 (~`MIGRATION.md:239`): same content as the CHANGELOG. Remove "as was already the case in v0.17".
  - § 3: a recipe for backend authors, showing cfg'd `scroll_region_up`/`down` behind a forwarded feature. Note that a **direct** `ratatui-termion`/`-termwiz`/`-termina` dependency needs that crate's own `scrolling-regions` feature when `envision/scrolling-regions` is on.
  - § 6: inline-viewport captures now reflect `append_lines`.
- [ ] **Step 3:** Run `cargo test --doc --all-features` and make a signed commit.
- [ ] **Step 4:** Merge `origin/main`, push, open a PR (with the `🤖 Generated with [Claude Code](https://claude.com/claude-code)` footer), get an independent review, and squash-merge once green.
