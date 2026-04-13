use super::*;

#[test]
fn test_render_empty() {
    let state = LogViewerState::new();
    let (mut terminal, theme) = test_utils::setup_render(60, 15);
    terminal
        .draw(|frame| {
            LogViewer::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();
}

#[test]
fn test_render_with_entries() {
    let state = sample_state();
    let (mut terminal, theme) = test_utils::setup_render(60, 15);
    terminal
        .draw(|frame| {
            LogViewer::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();
}

#[test]
fn test_render_focused() {
    let state = focused_state();
    let (mut terminal, theme) = test_utils::setup_render(60, 15);
    terminal
        .draw(|frame| {
            LogViewer::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();
}

#[test]
fn test_render_search_focused() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    LogViewer::update(&mut state, LogViewerMessage::SearchInput('c'));
    LogViewer::update(&mut state, LogViewerMessage::SearchInput('o'));
    let (mut terminal, theme) = test_utils::setup_render(60, 15);
    terminal
        .draw(|frame| {
            LogViewer::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();
}

#[test]
fn test_render_disabled() {
    let state = LogViewerState::new();
    let (mut terminal, theme) = test_utils::setup_render(60, 15);
    terminal
        .draw(|frame| {
            LogViewer::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).disabled(true),
            );
        })
        .unwrap();
}

#[test]
fn test_render_with_title() {
    let mut state = LogViewerState::new().with_title("Application Log");
    state.push_info("entry 1");
    state.push_error("entry 2");
    let (mut terminal, theme) = test_utils::setup_render(60, 15);
    terminal
        .draw(|frame| {
            LogViewer::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();
}

#[test]
fn test_render_with_timestamps() {
    let mut state = LogViewerState::new().with_show_timestamps(true);
    state.push_info_with_timestamp("entry 1", "12:00:00");
    state.push_error_with_timestamp("entry 2", "12:00:01");
    let (mut terminal, theme) = test_utils::setup_render(60, 15);
    terminal
        .draw(|frame| {
            LogViewer::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();
}

#[test]
fn test_render_small_area() {
    let state = sample_state();
    let (mut terminal, theme) = test_utils::setup_render(60, 2);
    terminal
        .draw(|frame| {
            LogViewer::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();
}
