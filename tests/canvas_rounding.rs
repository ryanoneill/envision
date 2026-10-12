//! Pins ratatui 0.30's canvas grid rounding (0.29 truncated: (1, 3)).
use envision::ratatui::symbols::Marker;
use envision::ratatui::widgets::canvas::{Context, Painter};

#[test]
fn braille_grid_rounds_to_nearest() {
    let mut ctx = Context::new(2, 2, [1.0, 2.0], [0.0, 2.0], Marker::Braille);
    let painter = Painter::from(&mut ctx);
    assert_eq!(painter.get_point(1.5, 1.0), Some((2, 4)));
}
