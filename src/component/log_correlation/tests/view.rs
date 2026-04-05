use super::*;

// =============================================================================
// Rendering
// =============================================================================

#[test]
fn test_render_empty() {
    let state = LogCorrelationState::new();
    let (mut terminal, theme) = test_utils::setup_render(80, 20);
    terminal
        .draw(|frame| {
            LogCorrelation::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().focused(true),
            );
        })
        .unwrap();
}

#[test]
fn test_render_two_streams() {
    let state = two_stream_state();
    let (mut terminal, theme) = test_utils::setup_render(80, 20);
    terminal
        .draw(|frame| {
            LogCorrelation::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().focused(true),
            );
        })
        .unwrap();
}

#[test]
fn test_render_focused() {
    let state = focused_state();
    let (mut terminal, theme) = test_utils::setup_render(80, 20);
    terminal
        .draw(|frame| {
            LogCorrelation::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().focused(true),
            );
        })
        .unwrap();
}

#[test]
fn test_render_disabled() {
    let state = two_stream_state();
    let (mut terminal, theme) = test_utils::setup_render(80, 20);
    terminal
        .draw(|frame| {
            LogCorrelation::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().disabled(true),
            );
        })
        .unwrap();
}

#[test]
fn test_render_with_title() {
    let state = LogCorrelationState::new()
        .with_title("Log Correlation")
        .with_streams(vec![
            LogStream::new("API").with_entry(CorrelationEntry::new(
                1.0,
                CorrelationLevel::Info,
                "ok",
            )),
            LogStream::new("DB").with_entry(CorrelationEntry::new(
                1.0,
                CorrelationLevel::Info,
                "ok",
            )),
        ]);
    let (mut terminal, theme) = test_utils::setup_render(80, 20);
    terminal
        .draw(|frame| {
            LogCorrelation::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().focused(true),
            );
        })
        .unwrap();
}

#[test]
fn test_render_with_filter() {
    let mut state = two_stream_state();
    state.streams[0].filter = "Query".to_string();
    let (mut terminal, theme) = test_utils::setup_render(80, 20);
    terminal
        .draw(|frame| {
            LogCorrelation::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().focused(true),
            );
        })
        .unwrap();
}

#[test]
fn test_render_small_area() {
    let state = two_stream_state();
    let (mut terminal, theme) = test_utils::setup_render(80, 2);
    terminal
        .draw(|frame| {
            LogCorrelation::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().focused(true),
            );
        })
        .unwrap();
}

#[test]
fn test_render_narrow_area() {
    let state = two_stream_state();
    let (mut terminal, theme) = test_utils::setup_render(20, 10);
    terminal
        .draw(|frame| {
            LogCorrelation::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().focused(true),
            );
        })
        .unwrap();
}
