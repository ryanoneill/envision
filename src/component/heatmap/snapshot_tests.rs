use super::*;
use crate::component::test_utils;

#[test]
fn test_snapshot_empty() {
    let state = HeatmapState::default();
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Heatmap::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_small_grid() {
    let state = HeatmapState::with_data(vec![vec![0.0, 0.5, 1.0], vec![0.3, 0.7, 0.9]]);
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Heatmap::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_with_labels() {
    let state = HeatmapState::with_data(vec![vec![0.0, 0.5, 1.0], vec![0.3, 0.7, 0.9]])
        .with_row_labels(vec!["AM".into(), "PM".into()])
        .with_col_labels(vec!["Mon".into(), "Tue".into(), "Wed".into()])
        .with_title("Schedule");
    let (mut terminal, theme) = test_utils::setup_render(50, 10);
    terminal
        .draw(|frame| {
            Heatmap::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_with_values() {
    let state = HeatmapState::with_data(vec![vec![1.5, 2.7], vec![3.1, 4.9]])
        .with_show_values(true)
        .with_title("Values");
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Heatmap::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_focused() {
    let mut state = HeatmapState::with_data(vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]])
        .with_title("Focused Heatmap");
    state.set_focused(true);
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Heatmap::view(
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
fn test_snapshot_focused_with_selection() {
    let mut state = HeatmapState::with_data(vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]]);
    state.set_focused(true);
    state.update(HeatmapMessage::SelectDown);
    state.update(HeatmapMessage::SelectRight);
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Heatmap::view(
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
fn test_snapshot_disabled() {
    let state = HeatmapState::with_data(vec![vec![1.0, 2.0], vec![3.0, 4.0]])
        .with_disabled(true)
        .with_title("Disabled");
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Heatmap::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().disabled(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_blue_to_red_scale() {
    let state = HeatmapState::with_data(vec![vec![0.0, 0.25, 0.5, 0.75, 1.0]])
        .with_color_scale(HeatmapColorScale::BlueToRed)
        .with_range(0.0, 1.0);
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Heatmap::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_1x1_grid() {
    let state = HeatmapState::with_data(vec![vec![0.5]]);
    let (mut terminal, theme) = test_utils::setup_render(20, 6);
    terminal
        .draw(|frame| {
            Heatmap::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}
