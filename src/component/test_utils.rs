use crate::backend::CaptureBackend;
use crate::theme::Theme;
use ratatui::Terminal;

pub(crate) fn setup_render(width: u16, height: u16) -> (Terminal<CaptureBackend>, Theme) {
    let backend = CaptureBackend::new(width, height);
    let terminal = Terminal::new(backend).unwrap();
    (terminal, Theme::default())
}
