//! Tests for the per-row ownership flag (`row_may_hold_owner`) that lets
//! `write_cell` skip the left scan on rows that cannot contain a wide owner.

use std::num::NonZeroU16;

use ratatui::backend::{Backend, ClearType};
use ratatui::buffer::Cell;

use super::super::CaptureBackend;
use crate::backend::cell::SerializableDiffOption;

fn draw(b: &mut CaptureBackend, cells: &[(u16, u16, &'static str)]) {
    let cells: Vec<(u16, u16, Cell)> = cells
        .iter()
        .map(|&(x, y, s)| (x, y, Cell::new(s)))
        .collect();
    let Ok(()) = b.draw(cells.iter().map(|(x, y, c)| (*x, *y, c)));
}

/// The flag's invariant: every row that holds an owner (a cell whose
/// `owner_width() > 1`; blank cells never qualify, ruling R13) is flagged.
/// (A flagged row without an owner is allowed: it only costs a scan.)
fn assert_flags_conservative(b: &CaptureBackend) {
    assert_eq!(b.row_may_hold_owner.len(), usize::from(b.height));
    for y in 0..b.height {
        let holds_owner = (0..b.width).any(|x| b.cells[b.index_of(x, y)].owner_width() > 1);
        if holds_owner {
            assert!(
                b.row_may_hold_owner[usize::from(y)],
                "row {y} holds an owner but is not flagged"
            );
        }
    }
}

fn flags(b: &CaptureBackend) -> Vec<bool> {
    b.row_may_hold_owner.clone()
}

#[test]
fn constructors_size_the_flags_to_the_height() {
    let b = CaptureBackend::new(4, 3);
    assert_eq!(flags(&b), vec![false; 3]);
    let b = CaptureBackend::with_history(4, 5, 2);
    assert_eq!(flags(&b), vec![false; 5]);
}

#[test]
fn draw_flags_only_rows_given_an_owner() {
    let mut b = CaptureBackend::new(4, 3);
    draw(&mut b, &[(0, 0, "a"), (0, 1, "中"), (0, 2, "b")]);
    assert_eq!(flags(&b), vec![false, true, false]);
    assert_flags_conservative(&b);
}

#[test]
fn forced_width_owner_flags_its_row() {
    let mut b = CaptureBackend::new(4, 1);
    let mut wide = Cell::new("W");
    wide.set_diff_option(ratatui::buffer::CellDiffOption::ForcedWidth(
        NonZeroU16::new(2).unwrap(),
    ));
    let Ok(()) = b.draw([(0u16, 0u16, &wide)].into_iter());
    assert_eq!(flags(&b), vec![true]);
}

// Test (1): after clear(), the row is unflagged and a sparse write into the
// formerly-owned row blanks nothing.
#[test]
fn clear_unflags_and_sparse_write_is_correct() {
    let mut b = CaptureBackend::new(4, 2);
    draw(&mut b, &[(0, 0, "中"), (0, 1, "中")]);
    let Ok(()) = b.clear();
    assert_eq!(flags(&b), vec![false, false]);
    draw(&mut b, &[(1, 0, "z"), (3, 0, "q")]);
    assert_eq!(b.row_content(0), " z q");
    assert_flags_conservative(&b);
}

// Test (2): in a flagged row, narrow writes elsewhere leave the owner alone,
// and a non-blank write into its continuation still blanks it.
#[test]
fn narrow_writes_in_an_owned_row_blank_correctly() {
    let mut b = CaptureBackend::new(6, 1);
    draw(&mut b, &[(2, 0, "中"), (5, 0, "b")]);
    draw(&mut b, &[(0, 0, "a"), (4, 0, "c")]); // left of / right of the glyph
    assert_eq!(b.row_content(0), "a 中 cb");
    draw(&mut b, &[(3, 0, "z")]); // into the continuation
    assert_eq!(b.row_content(0), "a  zcb");
    assert_flags_conservative(&b);
}

// Test (3): every public mutation path keeps the invariant.

#[test]
fn cell_mut_flags_the_row_it_exposes() {
    let mut b = CaptureBackend::new(4, 2);
    if let Some(cell) = b.cell_mut(0, 1) {
        cell.set_symbol("中");
    }
    assert_eq!(flags(&b), vec![false, true]);
    assert_flags_conservative(&b);
    draw(&mut b, &[(1, 1, "z")]);
    assert_eq!(b.row_content(1), " z  ", "owner set via cell_mut is found");
}

#[test]
fn cell_mut_forced_width_owner_is_found() {
    let mut b = CaptureBackend::new(4, 1);
    if let Some(cell) = b.cell_mut(0, 0) {
        cell.set_symbol("W");
        cell.set_diff_option(SerializableDiffOption::ForcedWidth(
            NonZeroU16::new(3).unwrap(),
        ));
    }
    assert_flags_conservative(&b);
    draw(&mut b, &[(2, 0, "z")]);
    assert_eq!(b.row_content(0), "  z ");
}

#[test]
fn cell_mut_out_of_bounds_flags_nothing() {
    let mut b = CaptureBackend::new(4, 2);
    assert!(b.cell_mut(4, 0).is_none());
    assert!(b.cell_mut(0, 2).is_none());
    assert_eq!(flags(&b), vec![false, false]);
}

/// Builds a 4x3 backend with an owner at column 0 of every row and the cursor
/// at `(cursor_x, cursor_y)`.
fn owned_rows(cursor_x: u16, cursor_y: u16) -> CaptureBackend {
    let mut b = CaptureBackend::new(4, 3);
    draw(&mut b, &[(0, 0, "中"), (0, 1, "中"), (0, 2, "中")]);
    let Ok(()) = b.set_cursor_position((cursor_x, cursor_y));
    b
}

#[test]
fn clear_region_all_unflags_every_row() {
    let mut b = owned_rows(1, 1);
    let Ok(()) = b.clear_region(ClearType::All);
    assert_eq!(flags(&b), vec![false, false, false]);
    assert_flags_conservative(&b);
}

#[test]
fn clear_region_current_line_unflags_only_that_row() {
    let mut b = owned_rows(2, 1);
    let Ok(()) = b.clear_region(ClearType::CurrentLine);
    assert_eq!(flags(&b), vec![true, false, true]);
    assert_flags_conservative(&b);
}

#[test]
fn clear_region_after_cursor_unflags_only_whole_rows() {
    // Cursor mid-row 0: row 0 is only partly cleared, so its owner survives.
    let mut b = owned_rows(2, 0);
    let Ok(()) = b.clear_region(ClearType::AfterCursor);
    assert_eq!(flags(&b), vec![true, false, false]);
    assert_flags_conservative(&b);
    draw(&mut b, &[(1, 0, "z")]);
    assert_eq!(b.row_content(0), " z  ", "surviving owner is still found");

    // Cursor at a row start: that row is cleared whole.
    let mut b = owned_rows(0, 1);
    let Ok(()) = b.clear_region(ClearType::AfterCursor);
    assert_eq!(flags(&b), vec![true, false, false]);
}

#[test]
fn clear_region_before_cursor_unflags_only_whole_rows() {
    // Cursor at (1, 2): rows 0 and 1 are cleared whole; row 2 keeps column 1+.
    let mut b = owned_rows(1, 2);
    let Ok(()) = b.clear_region(ClearType::BeforeCursor);
    assert_eq!(flags(&b), vec![false, false, true]);
    assert_flags_conservative(&b);
}

#[test]
fn clear_region_until_new_line_keeps_partly_cleared_row() {
    let mut b = owned_rows(1, 1);
    let Ok(()) = b.clear_region(ClearType::UntilNewLine);
    assert_eq!(flags(&b), vec![true, true, true]);
    assert_flags_conservative(&b);
    draw(&mut b, &[(1, 1, "z")]);
    assert_eq!(b.row_content(1), " z  ");

    let mut b = owned_rows(0, 1);
    let Ok(()) = b.clear_region(ClearType::UntilNewLine);
    assert_eq!(flags(&b), vec![true, false, true]);
}

#[test]
fn clone_carries_the_flags() {
    let mut b = CaptureBackend::new(4, 1);
    draw(&mut b, &[(0, 0, "中")]);
    let mut copy = b.clone();
    assert_eq!(flags(&copy), vec![true]);
    draw(&mut copy, &[(1, 0, "z")]);
    assert_eq!(copy.row_content(0), " z  ");
}

#[test]
fn snapshot_does_not_carry_the_flags() {
    let mut b = CaptureBackend::with_history(4, 1, 2);
    draw(&mut b, &[(0, 0, "中")]);
    let snapshot = b.snapshot();
    assert_eq!(snapshot.cells.len(), 4);
    #[cfg(feature = "serialization")]
    {
        let json = serde_json::to_string(&snapshot).unwrap();
        assert!(!json.contains("row_may_hold_owner"));
    }
}
