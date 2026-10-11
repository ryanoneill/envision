//! Terminal wide-glyph semantics for captured cells (spec § 5).
//!
//! Widths come from `EnhancedCell::covered_width` (ratatui's width via the
//! crate's single helper). No per-cell ownership state is stored: the owner of
//! a column is found by scanning left over blank cells, and only on rows whose
//! `row_may_hold_owner` flag is set.

use super::CaptureBackend;
use crate::backend::cell::EnhancedCell;

impl CaptureBackend {
    /// Writes `cell` at `(x, y)` (in bounds), applying spec § 5 rules 1–4.
    pub(super) fn write_cell(&mut self, x: u16, y: u16, mut cell: EnhancedCell) {
        let frame = self.current_frame;

        // Rules 2 and 3: a write into a continuation column.
        if !cell.is_blank_symbol()
            && self.row_may_hold_owner[usize::from(y)]
            && let Some(owner_x) = self.owner_of(x, y)
        {
            self.blank_owner_and_continuations(owner_x, y, frame); // rule 3
        }
        // (A blank write into a continuation column — rule 2 — is just a write:
        // the owner stays intact and the written style is recorded.)

        // Rule 4: replacing an owner blanks its former continuation cells.
        let idx = self.index_of(x, y);
        let old_width = self.cells[idx].covered_width();
        if old_width > 1 {
            self.blank_range(x.saturating_add(1), x.saturating_add(old_width), y, frame);
        }

        let new_width = cell.covered_width();
        cell.set_last_modified_frame(frame);
        self.cells[idx] = cell;

        // Rule 1: a new owner's continuation cells become reset blanks.
        if new_width > 1 {
            self.flag_row_may_hold_owner(y);
            let end = x.saturating_add(new_width).min(self.width);
            for cx in x.saturating_add(1)..end {
                let mut blank = EnhancedCell::new();
                blank.set_last_modified_frame(frame);
                let cidx = self.index_of(cx, y);
                self.cells[cidx] = blank;
            }
        }
    }

    /// Records that row `y` may now hold a wide-glyph owner.
    pub(super) fn flag_row_may_hold_owner(&mut self, y: u16) {
        self.row_may_hold_owner[usize::from(y)] = true;
    }

    /// Unflags every row lying wholly inside the reset cell range
    /// `[start, end)`. A row only partly reset keeps its flag, since an owner
    /// may survive outside the range.
    pub(super) fn unflag_rows_within(&mut self, start: usize, end: usize) {
        let width = usize::from(self.width);
        if width == 0 || start >= end {
            return;
        }
        let first_whole_row = start.div_ceil(width);
        let end_of_whole_rows = end / width;
        for row in first_whole_row..end_of_whole_rows {
            self.row_may_hold_owner[row] = false;
        }
    }

    /// The column of the wide glyph covering `(x, y)` as a continuation, if any.
    /// Scans left only across blank cells, so the cost is bounded by the blank
    /// run immediately to the left.
    fn owner_of(&self, x: u16, y: u16) -> Option<u16> {
        let mut candidate = x;
        while candidate > 0 {
            candidate -= 1;
            let c = &self.cells[self.index_of(candidate, y)];
            if !c.is_blank_symbol() {
                let reach = candidate.saturating_add(c.covered_width());
                return (reach > x).then_some(candidate);
            }
        }
        None
    }

    /// Rule 3: blank the owner and its continuation cells, keeping their style.
    fn blank_owner_and_continuations(&mut self, owner_x: u16, y: u16, frame: u64) {
        let width = self.cells[self.index_of(owner_x, y)].covered_width();
        self.blank_range(owner_x, owner_x.saturating_add(width), y, frame);
    }

    /// Sets the symbol of in-row cells `[from, to)` to `" "`, keeping style.
    fn blank_range(&mut self, from: u16, to: u16, y: u16, frame: u64) {
        for cx in from..to.min(self.width) {
            let idx = self.index_of(cx, y);
            self.cells[idx].set_symbol(" ");
            self.cells[idx].set_last_modified_frame(frame);
        }
    }
}

#[cfg(test)]
mod tests;
