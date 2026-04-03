use super::*;
use crate::component::test_utils;

// =============================================================================
// Snapshot tests
// =============================================================================

#[test]
fn test_snapshot_default() {
    let state = SplitPanelState::default();
    let (mut terminal, theme) = test_utils::setup_render(70, 20);
    terminal
        .draw(|frame| {
            SplitPanel::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_vertical_focused_first() {
    let mut state = SplitPanelState::new(SplitOrientation::Vertical);
    SplitPanel::set_focused(&mut state, true);
    let (mut terminal, theme) = test_utils::setup_render(70, 20);
    terminal
        .draw(|frame| {
            SplitPanel::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().focused(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_vertical_focused_second() {
    let mut state = SplitPanelState::new(SplitOrientation::Vertical);
    SplitPanel::set_focused(&mut state, true);
    SplitPanel::update(&mut state, SplitPanelMessage::FocusOther);
    let (mut terminal, theme) = test_utils::setup_render(70, 20);
    terminal
        .draw(|frame| {
            SplitPanel::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().focused(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_horizontal() {
    let mut state = SplitPanelState::new(SplitOrientation::Horizontal);
    SplitPanel::set_focused(&mut state, true);
    let (mut terminal, theme) = test_utils::setup_render(70, 20);
    terminal
        .draw(|frame| {
            SplitPanel::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().focused(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_custom_ratio() {
    let mut state = SplitPanelState::new(SplitOrientation::Vertical).with_ratio(0.3);
    SplitPanel::set_focused(&mut state, true);
    let (mut terminal, theme) = test_utils::setup_render(70, 20);
    terminal
        .draw(|frame| {
            SplitPanel::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().focused(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_resized() {
    let mut state = SplitPanelState::new(SplitOrientation::Vertical);
    SplitPanel::set_focused(&mut state, true);
    // Grow the first pane twice
    SplitPanel::update(&mut state, SplitPanelMessage::GrowFirst);
    SplitPanel::update(&mut state, SplitPanelMessage::GrowFirst);
    let (mut terminal, theme) = test_utils::setup_render(70, 20);
    terminal
        .draw(|frame| {
            SplitPanel::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().focused(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}
