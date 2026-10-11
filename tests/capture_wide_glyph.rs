//! CaptureBackend models terminal wide-glyph semantics (spec § 5).

use envision::CaptureBackend;
use envision::backend::Backend;
use envision::ratatui::Terminal;
use envision::ratatui::buffer::Cell;
use envision::ratatui::style::{Color, Style};
use envision::ratatui::widgets::{Block, List, ListItem, ListState, Paragraph};

fn cell(sym: &str) -> Cell {
    Cell::new(sym)
}

fn draw(backend: &mut CaptureBackend, cells: &[(u16, u16, Cell)]) {
    let Ok(()) = backend.draw(cells.iter().map(|(x, y, c)| (*x, *y, c)));
}

// ---- 11a: unit level, partial cell lists exactly as a diff would send ----

#[test]
fn owner_blanks_its_continuation_cells() {
    let mut b = CaptureBackend::new(4, 1);
    draw(&mut b, &[(1, 0, cell("x"))]);
    draw(&mut b, &[(0, 0, cell("中"))]);
    assert_eq!(b.row_content(0), "中  ");
}

#[test]
fn narrow_over_owner_blanks_former_continuation() {
    let mut b = CaptureBackend::new(4, 1);
    draw(&mut b, &[(1, 0, cell("x"))]);
    draw(&mut b, &[(0, 0, cell("中"))]);
    draw(&mut b, &[(0, 0, cell("a"))]); // 0.30 diff sends only column 0
    assert_eq!(b.row_content(0), "a   ");
}

#[test]
fn non_blank_into_continuation_blanks_owner() {
    let mut b = CaptureBackend::new(4, 1);
    draw(&mut b, &[(0, 0, cell("中"))]);
    draw(&mut b, &[(1, 0, cell("z"))]);
    assert_eq!(b.row_content(0), " z  ");
}

#[test]
fn blank_into_continuation_keeps_owner_and_records_style() {
    let mut b = CaptureBackend::new(4, 1);
    let mut trailing = cell(" ");
    trailing.set_style(Style::new().bg(Color::Blue));
    draw(&mut b, &[(0, 0, cell("✔️")), (1, 0, trailing)]); // VS16 trailing re-emit
    assert_eq!(b.row_content(0), "✔️  ");
    assert_eq!(
        b.cell(1, 0).unwrap().bg(),
        envision::backend::SerializableColor::Blue
    );
}

#[test]
fn width_follows_ratatui_for_divergent_symbols() {
    let mut b = CaptureBackend::new(4, 1);
    draw(&mut b, &[(1, 0, cell("x"))]);
    draw(&mut b, &[(0, 0, cell("ﾊﾟ"))]); // ratatui width 2
    assert_eq!(b.cell(1, 0).unwrap().symbol(), " ");
}

#[test]
fn blanked_owner_keeps_its_style() {
    let mut b = CaptureBackend::new(4, 1);
    let mut owner = cell("中");
    owner.set_style(Style::new().fg(Color::Red));
    draw(&mut b, &[(0, 0, owner)]);
    draw(&mut b, &[(1, 0, cell("z"))]);
    let blanked = b.cell(0, 0).unwrap();
    assert_eq!(blanked.symbol(), " ");
    assert_eq!(blanked.fg(), envision::backend::SerializableColor::Red);
}

#[test]
fn control_byte_draw_does_not_panic() {
    let mut b = CaptureBackend::new(4, 1);
    draw(
        &mut b,
        &[(0, 0, cell("\n")), (1, 0, cell("\t")), (2, 0, cell("\x1b"))],
    );
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

#[test]
fn vs16_emoji_survives_over_non_blank_trailing_column() {
    let mut t = Terminal::new(CaptureBackend::new(4, 1)).unwrap();
    t.draw(|f| f.render_widget(Paragraph::new("ok"), f.area()))
        .unwrap();
    t.draw(|f| f.render_widget(Paragraph::new("✔️"), f.area()))
        .unwrap();
    assert!(t.backend().row_content(0).starts_with("✔️"));
}

#[test]
fn vs16_emoji_survives_in_styled_block() {
    let mut t = Terminal::new(CaptureBackend::new(6, 3)).unwrap();
    let block = Block::bordered().style(Style::new().bg(Color::Blue));
    t.draw(|f| f.render_widget(Paragraph::new("ok").block(block.clone()), f.area()))
        .unwrap();
    t.draw(|f| f.render_widget(Paragraph::new("✔️").block(block), f.area()))
        .unwrap();
    assert!(t.backend().row_content(1).contains("✔️"));
}

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
}

#[test]
fn themed_cjk_continuation_is_reset_blank() {
    let mut t = Terminal::new(CaptureBackend::new(4, 1)).unwrap();
    let style = Style::new().bg(Color::Blue);
    t.draw(|f| f.render_widget(Paragraph::new("中").style(style), f.area()))
        .unwrap();
    let continuation = t.backend().cell(1, 0).unwrap();
    assert_eq!(continuation.symbol(), " ");
}

#[test]
fn wide_glyph_at_last_column_is_clamped() {
    let mut b = CaptureBackend::new(3, 1);
    draw(&mut b, &[(2, 0, cell("中"))]);
    assert_eq!(b.row_content(0).chars().count(), 3);
}
