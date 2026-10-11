//! CaptureBackend models terminal wide-glyph semantics (spec § 5).
//!
//! Rule numbers refer to the numbered rules in spec § 5:
//! 1. an owner write sets its continuation cells to reset blanks (current frame);
//! 2. a blank write into a continuation column leaves the owner intact and
//!    records the written style;
//! 3. a non-blank write into a continuation column blanks the owner (keeping
//!    its style);
//! 4. a narrow write over an owner blanks the former continuation cells.

use std::num::NonZeroU16;

use envision::backend::{Backend, SerializableColor};
use envision::ratatui::Terminal;
use envision::ratatui::buffer::{Cell, CellDiffOption};
use envision::ratatui::style::{Color, Style};
use envision::ratatui::widgets::{Block, List, ListItem, ListState, Paragraph};
use envision::{CaptureBackend, EnhancedCell};

fn cell(sym: &'static str) -> Cell {
    Cell::new(sym)
}

fn draw(backend: &mut CaptureBackend, cells: &[(u16, u16, Cell)]) {
    let Ok(()) = backend.draw(cells.iter().map(|(x, y, c)| (*x, *y, c)));
}

/// Ends the current frame, so the next draw is stamped with a new frame number.
fn next_frame(backend: &mut CaptureBackend) {
    let Ok(()) = backend.flush();
}

// ---- 11a: unit level, partial cell lists exactly as a diff would send ----

// Rule 1: the continuation cell is a reset blank stamped with the owner's frame.
#[test]
fn owner_blanks_its_continuation_cells() {
    let mut b = CaptureBackend::new(4, 1);
    draw(&mut b, &[(1, 0, cell("x"))]);
    next_frame(&mut b);
    let frame = b.current_frame();
    draw(&mut b, &[(0, 0, cell("中"))]);
    assert_eq!(b.row_content(0), "中   ");
    assert_eq!(b.cell(1, 0).unwrap().last_modified_frame(), frame);
}

// Rule 4: with a flush between draws, rule 1 alone would leave column 1 at the
// second draw's frame, so the third draw's frame proves rule 4 ran.
#[test]
fn narrow_over_owner_blanks_former_continuation() {
    let mut b = CaptureBackend::new(4, 1);
    draw(&mut b, &[(1, 0, cell("x"))]);
    next_frame(&mut b);
    draw(&mut b, &[(0, 0, cell("中"))]);
    next_frame(&mut b);
    let frame = b.current_frame();
    draw(&mut b, &[(0, 0, cell("a"))]); // 0.30 diff sends only column 0
    assert_eq!(b.row_content(0), "a   ");
    assert_eq!(b.cell(1, 0).unwrap().last_modified_frame(), frame);
}

// Rule 3: the blanked owner is stamped with the frame of the overwriting draw.
#[test]
fn non_blank_into_continuation_blanks_owner() {
    let mut b = CaptureBackend::new(4, 1);
    draw(&mut b, &[(0, 0, cell("中"))]);
    next_frame(&mut b);
    let frame = b.current_frame();
    draw(&mut b, &[(1, 0, cell("z"))]);
    assert_eq!(b.row_content(0), " z  ");
    assert_eq!(b.cell(0, 0).unwrap().last_modified_frame(), frame);
}

// Rule 2: the owner survives and the continuation records the written style.
#[test]
fn blank_into_continuation_keeps_owner_and_records_style() {
    let mut b = CaptureBackend::new(4, 1);
    let mut trailing = cell(" ");
    trailing.set_style(Style::new().bg(Color::Blue));
    draw(&mut b, &[(0, 0, cell("✔️")), (1, 0, trailing)]); // VS16 trailing re-emit
    assert_eq!(b.row_content(0), "✔️   ");
    assert_eq!(b.cell(1, 0).unwrap().bg(), SerializableColor::Blue);
}

// Spec § 4c: widths follow ratatui, not unicode-width.
#[test]
fn width_follows_ratatui_for_divergent_symbols() {
    let mut b = CaptureBackend::new(4, 1);
    draw(&mut b, &[(1, 0, cell("x"))]);
    draw(&mut b, &[(0, 0, cell("ﾊﾟ"))]); // ratatui width 2
    assert_eq!(b.cell(1, 0).unwrap().symbol(), " ");
}

// Rule 3: the blanked owner keeps its style.
#[test]
fn blanked_owner_keeps_its_style() {
    let mut b = CaptureBackend::new(4, 1);
    let mut owner = cell("中");
    owner.set_style(Style::new().fg(Color::Red));
    draw(&mut b, &[(0, 0, owner)]);
    draw(&mut b, &[(1, 0, cell("z"))]);
    let blanked = b.cell(0, 0).unwrap();
    assert_eq!(blanked.symbol(), " ");
    assert_eq!(blanked.fg(), SerializableColor::Red);
}

#[test]
fn control_byte_draw_does_not_panic() {
    let mut b = CaptureBackend::new(4, 1);
    draw(
        &mut b,
        &[(0, 0, cell("\n")), (1, 0, cell("\t")), (2, 0, cell("\x1b"))],
    );
}

// Spec § 5 / invariant: a wide glyph at the last column is clamped to the row;
// the stale cell it replaces is overwritten, not left behind.
#[test]
fn wide_glyph_at_last_column_is_clamped() {
    let mut b = CaptureBackend::new(3, 1);
    draw(&mut b, &[(2, 0, cell("x"))]);
    draw(&mut b, &[(2, 0, cell("中"))]);
    assert_eq!(b.row_content(0), "  中");
}

// Spec § 4a/§ 5 hostile input: a captured ForcedWidth(65535) only decides which
// in-bounds cells to blank. Rules 1, 3 and 4 must stay inside the row and never
// touch other rows.
#[test]
fn hostile_forced_width_draw_is_inert() {
    let hostile = || {
        let mut c = cell("x");
        c.set_diff_option(CellDiffOption::ForcedWidth(NonZeroU16::new(65535).unwrap()));
        c
    };

    // Rules 1 then 3: owner at column 0, then a non-blank write at column 1.
    let mut b = CaptureBackend::new(4, 2);
    draw(&mut b, &[(0, 1, cell("q"))]);
    draw(&mut b, &[(0, 0, hostile())]);
    draw(&mut b, &[(1, 0, cell("y"))]);
    assert_eq!(b.row_content(0), " y  ");
    assert_eq!(b.row_content(1), "q   ");

    // Rules 1 then 4: owner at column 0, then a narrow write over it.
    let mut b = CaptureBackend::new(4, 2);
    draw(&mut b, &[(0, 1, cell("q"))]);
    draw(&mut b, &[(0, 0, hostile())]);
    draw(&mut b, &[(0, 0, cell("a"))]);
    assert_eq!(b.row_content(0), "a   ");
    assert_eq!(b.row_content(1), "q   ");
}

// ---- 11b: multi-frame through Terminal ----

#[test]
fn cjk_wide_to_narrow_across_frames() {
    let mut t = Terminal::new(CaptureBackend::new(4, 1)).unwrap();
    t.draw(|f| f.render_widget(Paragraph::new(" x"), f.area()))
        .unwrap();
    t.draw(|f| f.render_widget(Paragraph::new("中"), f.area()))
        .unwrap();
    t.draw(|f| f.render_widget(Paragraph::new("a"), f.area()))
        .unwrap();
    assert_eq!(t.backend().row_content(0), "a   ");
}

// Spec § 5 test list: VS16 emoji wide -> narrow.
#[test]
fn vs16_emoji_wide_to_narrow_across_frames() {
    let mut t = Terminal::new(CaptureBackend::new(4, 1)).unwrap();
    t.draw(|f| f.render_widget(Paragraph::new("✔️"), f.area()))
        .unwrap();
    t.draw(|f| f.render_widget(Paragraph::new("a"), f.area()))
        .unwrap();
    assert_eq!(t.backend().row_content(0), "a   ");
    // The VS16 draw stamps frame 0, Terminal::draw flushes to frame 1, and rule 4
    // restamps column 1 in the "a" draw (without it the column stays at 0).
    assert_eq!(t.backend().cell(1, 0).unwrap().last_modified_frame(), 1);
}

// Rule 2: narrow -> VS16 where the old trailing column is not blank.
#[test]
fn vs16_emoji_survives_over_non_blank_trailing_column() {
    let mut t = Terminal::new(CaptureBackend::new(4, 1)).unwrap();
    t.draw(|f| f.render_widget(Paragraph::new("ok"), f.area()))
        .unwrap();
    t.draw(|f| f.render_widget(Paragraph::new("✔️"), f.area()))
        .unwrap();
    assert_eq!(t.backend().row_content(0), "✔️   ");
}

// Rule 2 in a styled Block: the continuation cell carries the block's bg.
// The border takes column 0, so the emoji sits at x = 1 and its trailing cell at x = 2.
#[test]
fn vs16_emoji_survives_in_styled_block() {
    let mut t = Terminal::new(CaptureBackend::new(6, 3)).unwrap();
    let block = Block::bordered().style(Style::new().bg(Color::Blue));
    t.draw(|f| f.render_widget(Paragraph::new("ok").block(block.clone()), f.area()))
        .unwrap();
    t.draw(|f| f.render_widget(Paragraph::new("✔️").block(block), f.area()))
        .unwrap();
    assert!(t.backend().row_content(1).contains("✔️"));
    assert_eq!(t.backend().cell(1, 1).unwrap().symbol(), "✔️");
    assert_eq!(
        t.backend().cell(2, 1).unwrap().bg(),
        SerializableColor::Blue
    );
}

// Rule 2 in a highlighted List row: no highlight symbol is set, so the emoji
// sits at x = 0 and its trailing cell at x = 1 carries the highlight bg.
#[test]
fn vs16_emoji_survives_in_highlighted_list_row() {
    let mut t = Terminal::new(CaptureBackend::new(8, 2)).unwrap();
    let mut state = ListState::default().with_selected(Some(0));
    for text in ["ok", "✔️"] {
        let list = List::new([ListItem::new(text)]).highlight_style(Style::new().bg(Color::Yellow));
        t.draw(|f| f.render_stateful_widget(list, f.area(), &mut state))
            .unwrap();
    }
    assert!(t.backend().row_content(0).contains("✔️"));
    assert_eq!(t.backend().cell(0, 0).unwrap().symbol(), "✔️");
    assert_eq!(
        t.backend().cell(1, 0).unwrap().bg(),
        SerializableColor::Yellow
    );
}

// Rule 1: even inside a themed area the continuation is a *reset* blank, not
// the owner's style, stamped with the frame of the draw. Column 1 is seeded with
// a styled non-blank cell first so a no-op would be caught.
#[test]
fn themed_cjk_continuation_is_reset_blank() {
    let mut t = Terminal::new(CaptureBackend::new(4, 1)).unwrap();
    let seed = Style::new().bg(Color::Red).fg(Color::Green);
    t.draw(|f| f.render_widget(Paragraph::new(" x").style(seed), f.area()))
        .unwrap();
    assert_eq!(t.backend().cell(1, 0).unwrap().symbol(), "x");

    let frame = t.backend().current_frame(); // Terminal::draw flushes after drawing
    let style = Style::new().bg(Color::Blue);
    t.draw(|f| f.render_widget(Paragraph::new("中").style(style), f.area()))
        .unwrap();

    let continuation = t.backend().cell(1, 0).unwrap();
    assert_eq!(continuation.symbol(), " ");
    assert_eq!(continuation.fg(), SerializableColor::Reset);
    assert_eq!(continuation.bg(), SerializableColor::Reset);
    assert_eq!(continuation.modifiers(), EnhancedCell::new().modifiers());
    assert_eq!(continuation.last_modified_frame(), frame);
}
