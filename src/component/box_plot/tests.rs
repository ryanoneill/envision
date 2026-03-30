use super::*;
use crate::component::test_utils;

// =============================================================================
// BoxPlotData construction
// =============================================================================

#[test]
fn test_data_new() {
    let data = BoxPlotData::new("Test", 1.0, 2.0, 3.0, 4.0, 5.0);
    assert_eq!(data.label(), "Test");
    assert_eq!(data.min(), 1.0);
    assert_eq!(data.q1(), 2.0);
    assert_eq!(data.median(), 3.0);
    assert_eq!(data.q3(), 4.0);
    assert_eq!(data.max(), 5.0);
    assert!(data.outliers().is_empty());
    assert_eq!(data.color(), Color::Cyan);
}

#[test]
fn test_data_with_color() {
    let data = BoxPlotData::new("Test", 1.0, 2.0, 3.0, 4.0, 5.0).with_color(Color::Red);
    assert_eq!(data.color(), Color::Red);
}

#[test]
fn test_data_with_outliers() {
    let data = BoxPlotData::new("Test", 1.0, 2.0, 3.0, 4.0, 5.0).with_outliers(vec![0.1, 6.0, 7.0]);
    assert_eq!(data.outliers(), &[0.1, 6.0, 7.0]);
}

#[test]
fn test_data_iqr() {
    let data = BoxPlotData::new("Test", 0.0, 10.0, 20.0, 30.0, 40.0);
    assert_eq!(data.iqr(), 20.0);
}

#[test]
fn test_data_range() {
    let data = BoxPlotData::new("Test", 5.0, 10.0, 20.0, 30.0, 45.0);
    assert_eq!(data.range(), 40.0);
}

#[test]
fn test_data_overall_min_no_outliers() {
    let data = BoxPlotData::new("Test", 5.0, 10.0, 20.0, 30.0, 45.0);
    assert_eq!(data.overall_min(), 5.0);
}

#[test]
fn test_data_overall_min_with_outliers() {
    let data = BoxPlotData::new("Test", 5.0, 10.0, 20.0, 30.0, 45.0).with_outliers(vec![1.0, 60.0]);
    assert_eq!(data.overall_min(), 1.0);
}

#[test]
fn test_data_overall_max_no_outliers() {
    let data = BoxPlotData::new("Test", 5.0, 10.0, 20.0, 30.0, 45.0);
    assert_eq!(data.overall_max(), 45.0);
}

#[test]
fn test_data_overall_max_with_outliers() {
    let data = BoxPlotData::new("Test", 5.0, 10.0, 20.0, 30.0, 45.0).with_outliers(vec![1.0, 60.0]);
    assert_eq!(data.overall_max(), 60.0);
}

#[test]
fn test_data_set_label() {
    let mut data = BoxPlotData::new("Old", 1.0, 2.0, 3.0, 4.0, 5.0);
    data.set_label("New");
    assert_eq!(data.label(), "New");
}

#[test]
fn test_data_set_color() {
    let mut data = BoxPlotData::new("Test", 1.0, 2.0, 3.0, 4.0, 5.0);
    data.set_color(Color::Green);
    assert_eq!(data.color(), Color::Green);
}

#[test]
fn test_data_set_outliers() {
    let mut data = BoxPlotData::new("Test", 1.0, 2.0, 3.0, 4.0, 5.0);
    data.set_outliers(vec![0.5, 6.5]);
    assert_eq!(data.outliers(), &[0.5, 6.5]);
}

#[test]
fn test_data_add_outlier() {
    let mut data = BoxPlotData::new("Test", 1.0, 2.0, 3.0, 4.0, 5.0);
    data.add_outlier(0.5);
    data.add_outlier(6.5);
    assert_eq!(data.outliers(), &[0.5, 6.5]);
}

#[test]
fn test_data_partial_eq() {
    let data1 = BoxPlotData::new("A", 1.0, 2.0, 3.0, 4.0, 5.0);
    let data2 = BoxPlotData::new("A", 1.0, 2.0, 3.0, 4.0, 5.0);
    assert_eq!(data1, data2);
}

#[test]
fn test_data_partial_eq_different() {
    let data1 = BoxPlotData::new("A", 1.0, 2.0, 3.0, 4.0, 5.0);
    let data2 = BoxPlotData::new("B", 1.0, 2.0, 3.0, 4.0, 5.0);
    assert_ne!(data1, data2);
}

#[test]
fn test_data_clone() {
    let data = BoxPlotData::new("Test", 1.0, 2.0, 3.0, 4.0, 5.0)
        .with_color(Color::Red)
        .with_outliers(vec![0.1]);
    let cloned = data.clone();
    assert_eq!(data, cloned);
}

#[test]
fn test_data_debug() {
    let data = BoxPlotData::new("Test", 1.0, 2.0, 3.0, 4.0, 5.0);
    let debug_str = format!("{:?}", data);
    assert!(debug_str.contains("Test"));
}

// =============================================================================
// BoxPlotOrientation
// =============================================================================

#[test]
fn test_orientation_eq() {
    assert_eq!(BoxPlotOrientation::Vertical, BoxPlotOrientation::Vertical);
    assert_eq!(
        BoxPlotOrientation::Horizontal,
        BoxPlotOrientation::Horizontal
    );
    assert_ne!(BoxPlotOrientation::Vertical, BoxPlotOrientation::Horizontal);
}

#[test]
fn test_orientation_clone() {
    let o = BoxPlotOrientation::Vertical;
    let cloned = o.clone();
    assert_eq!(o, cloned);
}

#[test]
fn test_orientation_debug() {
    let debug_str = format!("{:?}", BoxPlotOrientation::Vertical);
    assert!(debug_str.contains("Vertical"));
}

// =============================================================================
// BoxPlotState construction
// =============================================================================

#[test]
fn test_state_default() {
    let state = BoxPlotState::default();
    assert!(state.datasets().is_empty());
    assert_eq!(state.title(), None);
    assert!(state.show_outliers());
    assert_eq!(state.orientation(), &BoxPlotOrientation::Vertical);
    assert_eq!(state.selected(), 0);
    assert!(!state.is_focused());
    assert!(!state.is_disabled());
}

#[test]
fn test_state_new() {
    let state = BoxPlotState::new(vec![
        BoxPlotData::new("A", 1.0, 2.0, 3.0, 4.0, 5.0),
        BoxPlotData::new("B", 6.0, 7.0, 8.0, 9.0, 10.0),
    ]);
    assert_eq!(state.datasets().len(), 2);
}

#[test]
fn test_state_with_title() {
    let state = BoxPlotState::default().with_title("My Plot");
    assert_eq!(state.title(), Some("My Plot"));
}

#[test]
fn test_state_with_show_outliers_false() {
    let state = BoxPlotState::default().with_show_outliers(false);
    assert!(!state.show_outliers());
}

#[test]
fn test_state_with_orientation_horizontal() {
    let state = BoxPlotState::default().with_orientation(BoxPlotOrientation::Horizontal);
    assert_eq!(state.orientation(), &BoxPlotOrientation::Horizontal);
}

#[test]
fn test_state_with_disabled() {
    let state = BoxPlotState::default().with_disabled(true);
    assert!(state.is_disabled());
}

// =============================================================================
// BoxPlotState accessors
// =============================================================================

#[test]
fn test_state_datasets_mut() {
    let mut state = BoxPlotState::new(vec![BoxPlotData::new("A", 1.0, 2.0, 3.0, 4.0, 5.0)]);
    state.datasets_mut()[0].set_label("Updated");
    assert_eq!(state.datasets()[0].label(), "Updated");
}

#[test]
fn test_state_get_dataset() {
    let state = BoxPlotState::new(vec![BoxPlotData::new("A", 1.0, 2.0, 3.0, 4.0, 5.0)]);
    assert!(state.get_dataset(0).is_some());
    assert!(state.get_dataset(1).is_none());
}

#[test]
fn test_state_get_dataset_mut() {
    let mut state = BoxPlotState::new(vec![BoxPlotData::new("A", 1.0, 2.0, 3.0, 4.0, 5.0)]);
    if let Some(d) = state.get_dataset_mut(0) {
        d.set_label("Modified");
    }
    assert_eq!(state.datasets()[0].label(), "Modified");
}

#[test]
fn test_state_set_title() {
    let mut state = BoxPlotState::default();
    state.set_title(Some("New Title".into()));
    assert_eq!(state.title(), Some("New Title"));
    state.set_title(None);
    assert_eq!(state.title(), None);
}

#[test]
fn test_state_set_show_outliers() {
    let mut state = BoxPlotState::default();
    assert!(state.show_outliers());
    state.set_show_outliers(false);
    assert!(!state.show_outliers());
}

#[test]
fn test_state_set_orientation() {
    let mut state = BoxPlotState::default();
    state.set_orientation(BoxPlotOrientation::Horizontal);
    assert_eq!(state.orientation(), &BoxPlotOrientation::Horizontal);
}

#[test]
fn test_state_set_selected() {
    let mut state = BoxPlotState::new(vec![
        BoxPlotData::new("A", 1.0, 2.0, 3.0, 4.0, 5.0),
        BoxPlotData::new("B", 6.0, 7.0, 8.0, 9.0, 10.0),
    ]);
    state.set_selected(1);
    assert_eq!(state.selected(), 1);
}

#[test]
fn test_state_set_selected_clamps_to_last() {
    let mut state = BoxPlotState::new(vec![BoxPlotData::new("A", 1.0, 2.0, 3.0, 4.0, 5.0)]);
    state.set_selected(10);
    assert_eq!(state.selected(), 0); // Only one item, clamped to index 0
}

#[test]
fn test_state_set_selected_empty() {
    let mut state = BoxPlotState::default();
    state.set_selected(5); // No-op on empty datasets
    assert_eq!(state.selected(), 0);
}

#[test]
fn test_state_dataset_count() {
    let state = BoxPlotState::new(vec![
        BoxPlotData::new("A", 1.0, 2.0, 3.0, 4.0, 5.0),
        BoxPlotData::new("B", 6.0, 7.0, 8.0, 9.0, 10.0),
    ]);
    assert_eq!(state.dataset_count(), 2);
}

#[test]
fn test_state_is_empty() {
    let state = BoxPlotState::default();
    assert!(state.is_empty());
}

#[test]
fn test_state_is_empty_false() {
    let state = BoxPlotState::new(vec![BoxPlotData::new("A", 1.0, 2.0, 3.0, 4.0, 5.0)]);
    assert!(!state.is_empty());
}

#[test]
fn test_state_add_dataset() {
    let mut state = BoxPlotState::default();
    state.add_dataset(BoxPlotData::new("A", 1.0, 2.0, 3.0, 4.0, 5.0));
    assert_eq!(state.dataset_count(), 1);
    state.add_dataset(BoxPlotData::new("B", 6.0, 7.0, 8.0, 9.0, 10.0));
    assert_eq!(state.dataset_count(), 2);
}

#[test]
fn test_state_clear_datasets() {
    let mut state = BoxPlotState::new(vec![
        BoxPlotData::new("A", 1.0, 2.0, 3.0, 4.0, 5.0),
        BoxPlotData::new("B", 6.0, 7.0, 8.0, 9.0, 10.0),
    ]);
    state.set_selected(1);
    state.clear_datasets();
    assert!(state.is_empty());
    assert_eq!(state.selected(), 0);
}

// =============================================================================
// Global min/max
// =============================================================================

#[test]
fn test_global_min_single_dataset() {
    let state = BoxPlotState::new(vec![BoxPlotData::new("A", 5.0, 10.0, 20.0, 30.0, 40.0)]);
    assert_eq!(state.global_min(), 5.0);
}

#[test]
fn test_global_max_single_dataset() {
    let state = BoxPlotState::new(vec![BoxPlotData::new("A", 5.0, 10.0, 20.0, 30.0, 40.0)]);
    assert_eq!(state.global_max(), 40.0);
}

#[test]
fn test_global_min_multiple_datasets() {
    let state = BoxPlotState::new(vec![
        BoxPlotData::new("A", 5.0, 10.0, 20.0, 30.0, 40.0),
        BoxPlotData::new("B", 8.0, 15.0, 25.0, 35.0, 50.0),
    ]);
    assert_eq!(state.global_min(), 5.0);
}

#[test]
fn test_global_max_multiple_datasets() {
    let state = BoxPlotState::new(vec![
        BoxPlotData::new("A", 5.0, 10.0, 20.0, 30.0, 40.0),
        BoxPlotData::new("B", 8.0, 15.0, 25.0, 35.0, 50.0),
    ]);
    assert_eq!(state.global_max(), 50.0);
}

#[test]
fn test_global_min_with_outliers_shown() {
    let state = BoxPlotState::new(vec![
        BoxPlotData::new("A", 5.0, 10.0, 20.0, 30.0, 40.0).with_outliers(vec![1.0, 60.0])
    ]);
    assert_eq!(state.global_min(), 1.0);
}

#[test]
fn test_global_max_with_outliers_shown() {
    let state = BoxPlotState::new(vec![
        BoxPlotData::new("A", 5.0, 10.0, 20.0, 30.0, 40.0).with_outliers(vec![1.0, 60.0])
    ]);
    assert_eq!(state.global_max(), 60.0);
}

#[test]
fn test_global_min_with_outliers_hidden() {
    let state = BoxPlotState::new(vec![
        BoxPlotData::new("A", 5.0, 10.0, 20.0, 30.0, 40.0).with_outliers(vec![1.0, 60.0])
    ])
    .with_show_outliers(false);
    assert_eq!(state.global_min(), 5.0);
}

#[test]
fn test_global_max_with_outliers_hidden() {
    let state = BoxPlotState::new(vec![
        BoxPlotData::new("A", 5.0, 10.0, 20.0, 30.0, 40.0).with_outliers(vec![1.0, 60.0])
    ])
    .with_show_outliers(false);
    assert_eq!(state.global_max(), 40.0);
}

#[test]
fn test_global_min_empty() {
    let state = BoxPlotState::default();
    assert_eq!(state.global_min(), 0.0);
}

#[test]
fn test_global_max_empty() {
    let state = BoxPlotState::default();
    assert_eq!(state.global_max(), 0.0);
}

// =============================================================================
// Focus and disabled
// =============================================================================

#[test]
fn test_state_focused() {
    let mut state = BoxPlotState::default();
    assert!(!state.is_focused());
    state.set_focused(true);
    assert!(state.is_focused());
    state.set_focused(false);
    assert!(!state.is_focused());
}

#[test]
fn test_state_disabled() {
    let mut state = BoxPlotState::default();
    assert!(!state.is_disabled());
    state.set_disabled(true);
    assert!(state.is_disabled());
    state.set_disabled(false);
    assert!(!state.is_disabled());
}

#[test]
fn test_focusable_trait() {
    let mut state = BoxPlotState::default();
    assert!(!BoxPlot::is_focused(&state));
    BoxPlot::set_focused(&mut state, true);
    assert!(BoxPlot::is_focused(&state));
    BoxPlot::blur(&mut state);
    assert!(!BoxPlot::is_focused(&state));
    BoxPlot::focus(&mut state);
    assert!(BoxPlot::is_focused(&state));
}

#[test]
fn test_disableable_trait() {
    let mut state = BoxPlotState::default();
    assert!(!BoxPlot::is_disabled(&state));
    BoxPlot::set_disabled(&mut state, true);
    assert!(BoxPlot::is_disabled(&state));
    BoxPlot::enable(&mut state);
    assert!(!BoxPlot::is_disabled(&state));
    BoxPlot::disable(&mut state);
    assert!(BoxPlot::is_disabled(&state));
}

// =============================================================================
// Component trait: init
// =============================================================================

#[test]
fn test_component_init() {
    let state = BoxPlot::init();
    assert!(state.is_empty());
    assert_eq!(state.title(), None);
    assert!(state.show_outliers());
    assert_eq!(state.orientation(), &BoxPlotOrientation::Vertical);
}

// =============================================================================
// Event handling
// =============================================================================

#[test]
fn test_handle_event_not_focused() {
    let state = BoxPlotState::default();
    let msg = BoxPlot::handle_event(&state, &Event::key(crate::input::KeyCode::Right));
    assert!(msg.is_none());
}

#[test]
fn test_handle_event_disabled() {
    let mut state = BoxPlotState::default();
    state.set_focused(true);
    state.set_disabled(true);
    let msg = BoxPlot::handle_event(&state, &Event::key(crate::input::KeyCode::Right));
    assert!(msg.is_none());
}

#[test]
fn test_handle_event_right_arrow() {
    let mut state = BoxPlotState::new(vec![
        BoxPlotData::new("A", 1.0, 2.0, 3.0, 4.0, 5.0),
        BoxPlotData::new("B", 6.0, 7.0, 8.0, 9.0, 10.0),
    ]);
    state.set_focused(true);
    let msg = BoxPlot::handle_event(&state, &Event::key(crate::input::KeyCode::Right));
    assert_eq!(msg, Some(BoxPlotMessage::NextDataset));
}

#[test]
fn test_handle_event_left_arrow() {
    let mut state = BoxPlotState::new(vec![
        BoxPlotData::new("A", 1.0, 2.0, 3.0, 4.0, 5.0),
        BoxPlotData::new("B", 6.0, 7.0, 8.0, 9.0, 10.0),
    ]);
    state.set_focused(true);
    let msg = BoxPlot::handle_event(&state, &Event::key(crate::input::KeyCode::Left));
    assert_eq!(msg, Some(BoxPlotMessage::PrevDataset));
}

#[test]
fn test_handle_event_l_key() {
    let mut state = BoxPlotState::default();
    state.set_focused(true);
    let msg = BoxPlot::handle_event(&state, &Event::char('l'));
    assert_eq!(msg, Some(BoxPlotMessage::NextDataset));
}

#[test]
fn test_handle_event_h_key() {
    let mut state = BoxPlotState::default();
    state.set_focused(true);
    let msg = BoxPlot::handle_event(&state, &Event::char('h'));
    assert_eq!(msg, Some(BoxPlotMessage::PrevDataset));
}

#[test]
fn test_handle_event_o_key() {
    let mut state = BoxPlotState::default();
    state.set_focused(true);
    let msg = BoxPlot::handle_event(&state, &Event::char('o'));
    assert_eq!(msg, Some(BoxPlotMessage::ToggleOutliers));
}

#[test]
fn test_handle_event_unhandled_key() {
    let mut state = BoxPlotState::default();
    state.set_focused(true);
    let msg = BoxPlot::handle_event(&state, &Event::char('z'));
    assert!(msg.is_none());
}

// =============================================================================
// Message handling (update)
// =============================================================================

#[test]
fn test_update_next_dataset() {
    let mut state = BoxPlotState::new(vec![
        BoxPlotData::new("A", 1.0, 2.0, 3.0, 4.0, 5.0),
        BoxPlotData::new("B", 6.0, 7.0, 8.0, 9.0, 10.0),
    ]);
    BoxPlot::update(&mut state, BoxPlotMessage::NextDataset);
    assert_eq!(state.selected(), 1);
}

#[test]
fn test_update_next_dataset_wraps() {
    let mut state = BoxPlotState::new(vec![
        BoxPlotData::new("A", 1.0, 2.0, 3.0, 4.0, 5.0),
        BoxPlotData::new("B", 6.0, 7.0, 8.0, 9.0, 10.0),
    ]);
    state.set_selected(1);
    BoxPlot::update(&mut state, BoxPlotMessage::NextDataset);
    assert_eq!(state.selected(), 0);
}

#[test]
fn test_update_next_dataset_empty() {
    let mut state = BoxPlotState::default();
    BoxPlot::update(&mut state, BoxPlotMessage::NextDataset);
    assert_eq!(state.selected(), 0);
}

#[test]
fn test_update_prev_dataset() {
    let mut state = BoxPlotState::new(vec![
        BoxPlotData::new("A", 1.0, 2.0, 3.0, 4.0, 5.0),
        BoxPlotData::new("B", 6.0, 7.0, 8.0, 9.0, 10.0),
    ]);
    state.set_selected(1);
    BoxPlot::update(&mut state, BoxPlotMessage::PrevDataset);
    assert_eq!(state.selected(), 0);
}

#[test]
fn test_update_prev_dataset_wraps() {
    let mut state = BoxPlotState::new(vec![
        BoxPlotData::new("A", 1.0, 2.0, 3.0, 4.0, 5.0),
        BoxPlotData::new("B", 6.0, 7.0, 8.0, 9.0, 10.0),
    ]);
    BoxPlot::update(&mut state, BoxPlotMessage::PrevDataset);
    assert_eq!(state.selected(), 1);
}

#[test]
fn test_update_prev_dataset_empty() {
    let mut state = BoxPlotState::default();
    BoxPlot::update(&mut state, BoxPlotMessage::PrevDataset);
    assert_eq!(state.selected(), 0);
}

#[test]
fn test_update_toggle_outliers() {
    let mut state = BoxPlotState::default();
    assert!(state.show_outliers());
    BoxPlot::update(&mut state, BoxPlotMessage::ToggleOutliers);
    assert!(!state.show_outliers());
    BoxPlot::update(&mut state, BoxPlotMessage::ToggleOutliers);
    assert!(state.show_outliers());
}

#[test]
fn test_update_set_datasets() {
    let mut state = BoxPlotState::new(vec![BoxPlotData::new("A", 1.0, 2.0, 3.0, 4.0, 5.0)]);
    state.set_selected(0);
    BoxPlot::update(
        &mut state,
        BoxPlotMessage::SetDatasets(vec![
            BoxPlotData::new("X", 10.0, 20.0, 30.0, 40.0, 50.0),
            BoxPlotData::new("Y", 15.0, 25.0, 35.0, 45.0, 55.0),
        ]),
    );
    assert_eq!(state.dataset_count(), 2);
    assert_eq!(state.selected(), 0); // Reset on SetDatasets
    assert_eq!(state.datasets()[0].label(), "X");
}

#[test]
fn test_update_add_dataset() {
    let mut state = BoxPlotState::default();
    BoxPlot::update(
        &mut state,
        BoxPlotMessage::AddDataset(BoxPlotData::new("A", 1.0, 2.0, 3.0, 4.0, 5.0)),
    );
    assert_eq!(state.dataset_count(), 1);
}

#[test]
fn test_update_clear_datasets() {
    let mut state = BoxPlotState::new(vec![BoxPlotData::new("A", 1.0, 2.0, 3.0, 4.0, 5.0)]);
    BoxPlot::update(&mut state, BoxPlotMessage::ClearDatasets);
    assert!(state.is_empty());
    assert_eq!(state.selected(), 0);
}

#[test]
fn test_update_set_orientation() {
    let mut state = BoxPlotState::default();
    BoxPlot::update(
        &mut state,
        BoxPlotMessage::SetOrientation(BoxPlotOrientation::Horizontal),
    );
    assert_eq!(state.orientation(), &BoxPlotOrientation::Horizontal);
}

#[test]
fn test_update_returns_none() {
    let mut state = BoxPlotState::default();
    let output = BoxPlot::update(&mut state, BoxPlotMessage::ToggleOutliers);
    assert!(output.is_none());
}

// =============================================================================
// Instance methods
// =============================================================================

#[test]
fn test_instance_handle_event() {
    let mut state = BoxPlotState::default();
    state.set_focused(true);
    let msg = state.handle_event(&Event::char('o'));
    assert_eq!(msg, Some(BoxPlotMessage::ToggleOutliers));
}

#[test]
fn test_instance_dispatch_event() {
    let mut state = BoxPlotState::new(vec![
        BoxPlotData::new("A", 1.0, 2.0, 3.0, 4.0, 5.0),
        BoxPlotData::new("B", 6.0, 7.0, 8.0, 9.0, 10.0),
    ]);
    state.set_focused(true);
    let output = state.dispatch_event(&Event::key(crate::input::KeyCode::Right));
    assert!(output.is_none());
    assert_eq!(state.selected(), 1);
}

#[test]
fn test_instance_update() {
    let mut state = BoxPlotState::default();
    let output = state.update(BoxPlotMessage::ToggleOutliers);
    assert!(output.is_none());
    assert!(!state.show_outliers());
}

// =============================================================================
// PartialEq
// =============================================================================

#[test]
fn test_state_partial_eq() {
    let state1 =
        BoxPlotState::new(vec![BoxPlotData::new("A", 1.0, 2.0, 3.0, 4.0, 5.0)]).with_title("Test");
    let state2 =
        BoxPlotState::new(vec![BoxPlotData::new("A", 1.0, 2.0, 3.0, 4.0, 5.0)]).with_title("Test");
    assert_eq!(state1, state2);
}

#[test]
fn test_state_partial_eq_different() {
    let state1 = BoxPlotState::new(vec![BoxPlotData::new("A", 1.0, 2.0, 3.0, 4.0, 5.0)]);
    let state2 = BoxPlotState::new(vec![BoxPlotData::new("B", 1.0, 2.0, 3.0, 4.0, 5.0)]);
    assert_ne!(state1, state2);
}

// =============================================================================
// Message PartialEq and Clone
// =============================================================================

#[test]
fn test_message_partial_eq() {
    assert_eq!(BoxPlotMessage::NextDataset, BoxPlotMessage::NextDataset);
    assert_eq!(BoxPlotMessage::PrevDataset, BoxPlotMessage::PrevDataset);
    assert_eq!(
        BoxPlotMessage::ToggleOutliers,
        BoxPlotMessage::ToggleOutliers
    );
    assert_ne!(BoxPlotMessage::NextDataset, BoxPlotMessage::PrevDataset);
}

#[test]
fn test_message_clone() {
    let msg = BoxPlotMessage::NextDataset;
    let cloned = msg.clone();
    assert_eq!(msg, cloned);
}

#[test]
fn test_message_debug() {
    let debug_str = format!("{:?}", BoxPlotMessage::NextDataset);
    assert!(debug_str.contains("NextDataset"));
}

// =============================================================================
// View rendering (vertical)
// =============================================================================

#[test]
fn test_render_empty() {
    let state = BoxPlotState::default();
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            BoxPlot::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_single_dataset() {
    let state = BoxPlotState::new(vec![BoxPlotData::new(
        "Latency", 10.0, 20.0, 30.0, 40.0, 50.0,
    )])
    .with_title("Box Plot");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            BoxPlot::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_multiple_datasets() {
    let state = BoxPlotState::new(vec![
        BoxPlotData::new("Service A", 10.0, 20.0, 30.0, 40.0, 50.0).with_color(Color::Cyan),
        BoxPlotData::new("Service B", 15.0, 25.0, 35.0, 45.0, 55.0).with_color(Color::Green),
        BoxPlotData::new("Service C", 5.0, 15.0, 25.0, 35.0, 45.0).with_color(Color::Yellow),
    ])
    .with_title("Latency Comparison");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            BoxPlot::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_with_outliers() {
    let state = BoxPlotState::new(vec![BoxPlotData::new(
        "Response", 10.0, 20.0, 30.0, 40.0, 50.0,
    )
    .with_outliers(vec![2.0, 65.0, 70.0])]);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            BoxPlot::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_without_outliers() {
    let state = BoxPlotState::new(vec![BoxPlotData::new(
        "Response", 10.0, 20.0, 30.0, 40.0, 50.0,
    )
    .with_outliers(vec![2.0, 65.0])])
    .with_show_outliers(false);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            BoxPlot::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_focused() {
    let mut state = BoxPlotState::new(vec![BoxPlotData::new("Test", 10.0, 20.0, 30.0, 40.0, 50.0)])
        .with_title("Focused Box Plot");
    state.set_focused(true);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            BoxPlot::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_disabled() {
    let state = BoxPlotState::new(vec![BoxPlotData::new("Test", 10.0, 20.0, 30.0, 40.0, 50.0)])
        .with_disabled(true);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            BoxPlot::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_small_area() {
    let state = BoxPlotState::new(vec![BoxPlotData::new("Test", 10.0, 20.0, 30.0, 40.0, 50.0)]);
    let (mut terminal, theme) = test_utils::setup_render(60, 2);
    terminal
        .draw(|frame| {
            BoxPlot::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_very_small_width() {
    let state = BoxPlotState::new(vec![BoxPlotData::new("Test", 10.0, 20.0, 30.0, 40.0, 50.0)]);
    let (mut terminal, theme) = test_utils::setup_render(4, 20);
    terminal
        .draw(|frame| {
            BoxPlot::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

// =============================================================================
// View rendering (horizontal)
// =============================================================================

#[test]
fn test_render_horizontal_single() {
    let state = BoxPlotState::new(vec![BoxPlotData::new(
        "Latency", 10.0, 20.0, 30.0, 40.0, 50.0,
    )])
    .with_orientation(BoxPlotOrientation::Horizontal)
    .with_title("Horizontal Box Plot");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            BoxPlot::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_horizontal_multiple() {
    let state = BoxPlotState::new(vec![
        BoxPlotData::new("SvcA", 10.0, 20.0, 30.0, 40.0, 50.0).with_color(Color::Cyan),
        BoxPlotData::new("SvcB", 15.0, 25.0, 35.0, 45.0, 55.0).with_color(Color::Green),
    ])
    .with_orientation(BoxPlotOrientation::Horizontal);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            BoxPlot::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_horizontal_with_outliers() {
    let state = BoxPlotState::new(vec![BoxPlotData::new(
        "Response", 10.0, 20.0, 30.0, 40.0, 50.0,
    )
    .with_outliers(vec![2.0, 65.0])])
    .with_orientation(BoxPlotOrientation::Horizontal);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            BoxPlot::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_horizontal_disabled() {
    let state = BoxPlotState::new(vec![BoxPlotData::new("Test", 10.0, 20.0, 30.0, 40.0, 50.0)])
        .with_orientation(BoxPlotOrientation::Horizontal)
        .with_disabled(true);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            BoxPlot::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

// =============================================================================
// Snapshot tests
// =============================================================================

#[test]
fn test_snapshot_empty() {
    let state = BoxPlotState::default().with_title("Empty Box Plot");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            BoxPlot::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_single_vertical() {
    let state = BoxPlotState::new(vec![BoxPlotData::new("API", 10.0, 20.0, 35.0, 45.0, 55.0)])
        .with_title("Latency Distribution");
    let (mut terminal, theme) = test_utils::setup_render(40, 20);
    terminal
        .draw(|frame| {
            BoxPlot::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_multiple_vertical() {
    let state = BoxPlotState::new(vec![
        BoxPlotData::new("SvcA", 10.0, 20.0, 30.0, 40.0, 50.0).with_color(Color::Cyan),
        BoxPlotData::new("SvcB", 15.0, 25.0, 35.0, 45.0, 55.0).with_color(Color::Green),
        BoxPlotData::new("SvcC", 5.0, 15.0, 25.0, 35.0, 60.0).with_color(Color::Yellow),
    ])
    .with_title("Service Comparison");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            BoxPlot::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_horizontal() {
    let state = BoxPlotState::new(vec![
        BoxPlotData::new("SvcA", 10.0, 20.0, 30.0, 40.0, 50.0),
        BoxPlotData::new("SvcB", 15.0, 25.0, 35.0, 45.0, 55.0),
    ])
    .with_orientation(BoxPlotOrientation::Horizontal)
    .with_title("Horizontal Comparison");
    let (mut terminal, theme) = test_utils::setup_render(60, 15);
    terminal
        .draw(|frame| {
            BoxPlot::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_with_outliers() {
    let state = BoxPlotState::new(vec![BoxPlotData::new(
        "Response", 10.0, 25.0, 35.0, 45.0, 55.0,
    )
    .with_outliers(vec![2.0, 70.0, 80.0])])
    .with_title("With Outliers");
    let (mut terminal, theme) = test_utils::setup_render(40, 20);
    terminal
        .draw(|frame| {
            BoxPlot::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_disabled() {
    let state = BoxPlotState::new(vec![BoxPlotData::new(
        "Disabled", 10.0, 20.0, 30.0, 40.0, 50.0,
    )])
    .with_title("Disabled Box Plot")
    .with_disabled(true);
    let (mut terminal, theme) = test_utils::setup_render(40, 20);
    terminal
        .draw(|frame| {
            BoxPlot::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

// =============================================================================
// Annotation tests
// =============================================================================

#[test]
fn test_annotation_emitted() {
    use crate::annotation::with_annotations;
    let state = BoxPlotState::new(vec![BoxPlotData::new("Test", 1.0, 2.0, 3.0, 4.0, 5.0)]);
    let (mut terminal, theme) = test_utils::setup_render(40, 15);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                BoxPlot::view(&state, frame, frame.area(), &theme);
            })
            .unwrap();
    });
    assert!(registry.get_by_id("box_plot").is_some());
}

#[test]
fn test_annotation_with_focus() {
    use crate::annotation::with_annotations;
    let mut state = BoxPlotState::new(vec![BoxPlotData::new("Test", 1.0, 2.0, 3.0, 4.0, 5.0)]);
    state.set_focused(true);
    let (mut terminal, theme) = test_utils::setup_render(40, 15);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                BoxPlot::view(&state, frame, frame.area(), &theme);
            })
            .unwrap();
    });
    let region = registry.get_by_id("box_plot").unwrap();
    assert!(region.annotation.focused);
}

#[test]
fn test_annotation_with_disabled() {
    use crate::annotation::with_annotations;
    let state = BoxPlotState::new(vec![BoxPlotData::new("Test", 1.0, 2.0, 3.0, 4.0, 5.0)])
        .with_disabled(true);
    let (mut terminal, theme) = test_utils::setup_render(40, 15);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                BoxPlot::view(&state, frame, frame.area(), &theme);
            })
            .unwrap();
    });
    let region = registry.get_by_id("box_plot").unwrap();
    assert!(region.annotation.disabled);
}

// =============================================================================
// Data with zero range
// =============================================================================

#[test]
fn test_render_zero_range() {
    // All five-number summary values are the same
    let state = BoxPlotState::new(vec![BoxPlotData::new(
        "Constant", 25.0, 25.0, 25.0, 25.0, 25.0,
    )]);
    let (mut terminal, theme) = test_utils::setup_render(40, 15);
    terminal
        .draw(|frame| {
            BoxPlot::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_data_zero_iqr() {
    let data = BoxPlotData::new("Test", 1.0, 5.0, 5.0, 5.0, 10.0);
    assert_eq!(data.iqr(), 0.0);
}

#[test]
fn test_data_zero_range() {
    let data = BoxPlotData::new("Test", 5.0, 5.0, 5.0, 5.0, 5.0);
    assert_eq!(data.range(), 0.0);
}
