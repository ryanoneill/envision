use crate::backend::CaptureBackend;
use crate::theme::Theme;
use ratatui::Terminal;

/// Creates a terminal and theme for rendering tests.
///
/// Returns a `(Terminal<CaptureBackend>, Theme)` tuple configured
/// with the given dimensions and the default theme.
///
/// # Example
///
/// ```rust
/// use envision::component::test_utils::setup_render;
///
/// let (mut terminal, theme) = setup_render(80, 24);
/// assert_eq!(terminal.backend().width(), 80);
/// ```
pub fn setup_render(width: u16, height: u16) -> (Terminal<CaptureBackend>, Theme) {
    let backend = CaptureBackend::new(width, height);
    let terminal = Terminal::new(backend).unwrap();
    (terminal, Theme::default())
}
