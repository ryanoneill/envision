use super::render::{contrasting_fg, format_value, truncate_str};
use super::*;
use crate::component::test_utils;
use crate::input::Event;

// =============================================================================
// Construction
// =============================================================================

#[test]
fn test_new_creates_empty_grid() {
    let state = HeatmapState::new(3, 5);
    assert_eq!(state.rows(), 3);
    assert_eq!(state.cols(), 5);
    assert_eq!(state.get(0, 0), Some(0.0));
    assert_eq!(state.get(2, 4), Some(0.0));
    assert_eq!(state.selected(), Some((0, 0)));
}

#[test]
fn test_new_zero_rows() {
    let state = HeatmapState::new(0, 5);
    assert_eq!(state.rows(), 0);
    assert_eq!(state.cols(), 0);
    assert_eq!(state.selected(), None);
}

#[test]
fn test_new_zero_cols() {
    let state = HeatmapState::new(3, 0);
    assert_eq!(state.rows(), 3);
    assert_eq!(state.cols(), 0);
    assert_eq!(state.selected(), None);
}

#[test]
fn test_with_data() {
    let data = vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]];
    let state = HeatmapState::with_data(data);
    assert_eq!(state.rows(), 2);
    assert_eq!(state.cols(), 3);
    assert_eq!(state.get(0, 0), Some(1.0));
    assert_eq!(state.get(1, 2), Some(6.0));
    assert_eq!(state.selected(), Some((0, 0)));
}

#[test]
fn test_with_data_empty() {
    let state = HeatmapState::with_data(vec![]);
    assert_eq!(state.rows(), 0);
    assert_eq!(state.cols(), 0);
    assert_eq!(state.selected(), None);
}

#[test]
fn test_with_data_empty_rows() {
    let state = HeatmapState::with_data(vec![vec![], vec![]]);
    assert_eq!(state.rows(), 2);
    assert_eq!(state.cols(), 0);
    assert_eq!(state.selected(), None);
}

#[test]
fn test_default() {
    let state = HeatmapState::default();
    assert_eq!(state.rows(), 0);
    assert_eq!(state.cols(), 0);
    assert!(!state.focused);
    assert!(!state.disabled);
    assert!(!state.show_values);
    assert_eq!(state.title(), None);
    assert_eq!(state.color_scale(), &HeatmapColorScale::default());
}

// =============================================================================
// Builder methods
// =============================================================================

#[test]
fn test_with_row_labels() {
    let state = HeatmapState::new(2, 2).with_row_labels(vec!["R1".into(), "R2".into()]);
    assert_eq!(state.row_labels(), &["R1", "R2"]);
}

#[test]
fn test_with_col_labels() {
    let state = HeatmapState::new(2, 3).with_col_labels(vec!["A".into(), "B".into(), "C".into()]);
    assert_eq!(state.col_labels(), &["A", "B", "C"]);
}

#[test]
fn test_with_color_scale() {
    let state = HeatmapState::new(2, 2).with_color_scale(HeatmapColorScale::BlueToRed);
    assert_eq!(state.color_scale(), &HeatmapColorScale::BlueToRed);
}

#[test]
fn test_with_range() {
    let state = HeatmapState::new(2, 2).with_range(0.0, 100.0);
    assert_eq!(state.effective_min(), 0.0);
    assert_eq!(state.effective_max(), 100.0);
}

#[test]
fn test_with_show_values() {
    let state = HeatmapState::new(2, 2).with_show_values(true);
    assert!(state.show_values());
}

#[test]
fn test_with_title() {
    let state = HeatmapState::new(2, 2).with_title("Test Heatmap");
    assert_eq!(state.title(), Some("Test Heatmap"));
}

#[test]
fn test_with_disabled() {
    let state = HeatmapState::new(2, 2).with_disabled(true);
    assert!(state.is_disabled());
}

// =============================================================================
// Data operations
// =============================================================================

#[test]
fn test_set_and_get() {
    let mut state = HeatmapState::new(3, 3);
    state.set(1, 2, 42.0);
    assert_eq!(state.get(1, 2), Some(42.0));
    assert_eq!(state.get(0, 0), Some(0.0));
}

#[test]
fn test_set_out_of_bounds() {
    let mut state = HeatmapState::new(2, 2);
    state.set(5, 5, 99.0);
    // Should not panic; no effect
    assert_eq!(state.get(5, 5), None);
}

#[test]
fn test_get_out_of_bounds() {
    let state = HeatmapState::new(2, 2);
    assert_eq!(state.get(10, 0), None);
    assert_eq!(state.get(0, 10), None);
}

#[test]
fn test_clear_data() {
    let mut state = HeatmapState::with_data(vec![vec![1.0, 2.0], vec![3.0, 4.0]]);
    state.update(HeatmapMessage::Clear);
    assert_eq!(state.get(0, 0), Some(0.0));
    assert_eq!(state.get(1, 1), Some(0.0));
    // Grid dimensions preserved
    assert_eq!(state.rows(), 2);
    assert_eq!(state.cols(), 2);
}

// =============================================================================
// Color mapping
// =============================================================================

#[test]
fn test_green_to_red_at_zero() {
    let color = value_to_color(0.0, 0.0, 1.0, &HeatmapColorScale::GreenToRed);
    assert_eq!(color, Color::Rgb(0, 255, 0));
}

#[test]
fn test_green_to_red_at_half() {
    let color = value_to_color(0.5, 0.0, 1.0, &HeatmapColorScale::GreenToRed);
    assert_eq!(color, Color::Rgb(255, 255, 0)); // yellow
}

#[test]
fn test_green_to_red_at_one() {
    let color = value_to_color(1.0, 0.0, 1.0, &HeatmapColorScale::GreenToRed);
    assert_eq!(color, Color::Rgb(255, 0, 0));
}

#[test]
fn test_blue_to_red_at_zero() {
    let color = value_to_color(0.0, 0.0, 1.0, &HeatmapColorScale::BlueToRed);
    assert_eq!(color, Color::Rgb(0, 0, 255));
}

#[test]
fn test_blue_to_red_at_half() {
    let color = value_to_color(0.5, 0.0, 1.0, &HeatmapColorScale::BlueToRed);
    assert_eq!(color, Color::Rgb(255, 0, 255)); // magenta
}

#[test]
fn test_blue_to_red_at_one() {
    let color = value_to_color(1.0, 0.0, 1.0, &HeatmapColorScale::BlueToRed);
    assert_eq!(color, Color::Rgb(255, 0, 0));
}

#[test]
fn test_cool_to_warm_at_zero() {
    let color = value_to_color(0.0, 0.0, 1.0, &HeatmapColorScale::CoolToWarm);
    assert_eq!(color, Color::Rgb(0, 0, 200));
}

#[test]
fn test_cool_to_warm_at_half() {
    let color = value_to_color(0.5, 0.0, 1.0, &HeatmapColorScale::CoolToWarm);
    assert_eq!(color, Color::Rgb(200, 200, 200)); // gray
}

#[test]
fn test_cool_to_warm_at_one() {
    let color = value_to_color(1.0, 0.0, 1.0, &HeatmapColorScale::CoolToWarm);
    assert_eq!(color, Color::Rgb(200, 200, 0));
}

#[test]
fn test_intensity_at_zero() {
    let color = value_to_color(0.0, 0.0, 1.0, &HeatmapColorScale::Intensity(Color::Cyan));
    // Cyan = (0, 255, 255) at 20% brightness
    assert_eq!(color, Color::Rgb(0, 51, 51));
}

#[test]
fn test_intensity_at_one() {
    let color = value_to_color(1.0, 0.0, 1.0, &HeatmapColorScale::Intensity(Color::Cyan));
    // Cyan = (0, 255, 255) at full brightness
    assert_eq!(color, Color::Rgb(0, 255, 255));
}

#[test]
fn test_value_to_color_equal_min_max() {
    // When min == max, t should be 0.5
    let color = value_to_color(5.0, 5.0, 5.0, &HeatmapColorScale::GreenToRed);
    // t=0.5 => yellow
    assert_eq!(color, Color::Rgb(255, 255, 0));
}

#[test]
fn test_value_to_color_clamped_above() {
    let color = value_to_color(2.0, 0.0, 1.0, &HeatmapColorScale::GreenToRed);
    // Value above max is clamped to 1.0 => red
    assert_eq!(color, Color::Rgb(255, 0, 0));
}

#[test]
fn test_value_to_color_clamped_below() {
    let color = value_to_color(-1.0, 0.0, 1.0, &HeatmapColorScale::GreenToRed);
    // Value below min is clamped to 0.0 => green
    assert_eq!(color, Color::Rgb(0, 255, 0));
}

#[test]
fn test_intensity_with_rgb_color() {
    let color = value_to_color(
        1.0,
        0.0,
        1.0,
        &HeatmapColorScale::Intensity(Color::Rgb(100, 200, 50)),
    );
    assert_eq!(color, Color::Rgb(100, 200, 50));
}

// =============================================================================
// Navigation
// =============================================================================

fn focused_3x3() -> HeatmapState {
    let mut state = HeatmapState::with_data(vec![
        vec![1.0, 2.0, 3.0],
        vec![4.0, 5.0, 6.0],
        vec![7.0, 8.0, 9.0],
    ]);
    state.set_focused(true);
    state
}

#[test]
fn test_select_down() {
    let mut state = focused_3x3();
    let output = state.update(HeatmapMessage::SelectDown);
    assert_eq!(state.selected(), Some((1, 0)));
    assert_eq!(
        output,
        Some(HeatmapOutput::SelectionChanged { row: 1, col: 0 })
    );
}

#[test]
fn test_select_up_at_top() {
    let mut state = focused_3x3();
    let output = state.update(HeatmapMessage::SelectUp);
    assert_eq!(state.selected(), Some((0, 0)));
    assert_eq!(output, None); // Can't go up further
}

#[test]
fn test_select_right() {
    let mut state = focused_3x3();
    let output = state.update(HeatmapMessage::SelectRight);
    assert_eq!(state.selected(), Some((0, 1)));
    assert_eq!(
        output,
        Some(HeatmapOutput::SelectionChanged { row: 0, col: 1 })
    );
}

#[test]
fn test_select_left_at_left_edge() {
    let mut state = focused_3x3();
    let output = state.update(HeatmapMessage::SelectLeft);
    assert_eq!(state.selected(), Some((0, 0)));
    assert_eq!(output, None);
}

#[test]
fn test_select_down_at_bottom() {
    let mut state = focused_3x3();
    state.update(HeatmapMessage::SelectDown);
    state.update(HeatmapMessage::SelectDown);
    let output = state.update(HeatmapMessage::SelectDown);
    assert_eq!(state.selected(), Some((2, 0)));
    assert_eq!(output, None); // At bottom, can't go further
}

#[test]
fn test_select_right_at_right_edge() {
    let mut state = focused_3x3();
    state.update(HeatmapMessage::SelectRight);
    state.update(HeatmapMessage::SelectRight);
    let output = state.update(HeatmapMessage::SelectRight);
    assert_eq!(state.selected(), Some((0, 2)));
    assert_eq!(output, None);
}

#[test]
fn test_navigation_full_traversal() {
    let mut state = focused_3x3();
    // Go to (1, 1)
    state.update(HeatmapMessage::SelectDown);
    state.update(HeatmapMessage::SelectRight);
    assert_eq!(state.selected(), Some((1, 1)));
    assert_eq!(state.selected_value(), Some(5.0));

    // Go up
    state.update(HeatmapMessage::SelectUp);
    assert_eq!(state.selected(), Some((0, 1)));
    assert_eq!(state.selected_value(), Some(2.0));

    // Go left
    state.update(HeatmapMessage::SelectLeft);
    assert_eq!(state.selected(), Some((0, 0)));
    assert_eq!(state.selected_value(), Some(1.0));
}

// =============================================================================
// Selection
// =============================================================================

#[test]
fn test_selected_value() {
    let state = HeatmapState::with_data(vec![vec![42.0, 7.5]]);
    assert_eq!(state.selected_value(), Some(42.0));
}

#[test]
fn test_selected_value_empty() {
    let state = HeatmapState::default();
    assert_eq!(state.selected_value(), None);
}

// =============================================================================
// Range: auto vs manual
// =============================================================================

#[test]
fn test_effective_range_auto() {
    let state = HeatmapState::with_data(vec![vec![5.0, 10.0], vec![15.0, 20.0]]);
    assert_eq!(state.effective_min(), 5.0);
    assert_eq!(state.effective_max(), 20.0);
}

#[test]
fn test_effective_range_manual() {
    let state = HeatmapState::new(2, 2).with_range(0.0, 100.0);
    assert_eq!(state.effective_min(), 0.0);
    assert_eq!(state.effective_max(), 100.0);
}

#[test]
fn test_effective_range_empty() {
    let state = HeatmapState::default();
    assert_eq!(state.effective_min(), 0.0);
    assert_eq!(state.effective_max(), 0.0);
}

// =============================================================================
// Event mapping
// =============================================================================

#[test]
fn test_arrow_up_maps_to_select_up() {
    let state = focused_3x3();
    let msg = Heatmap::handle_event(&state, &Event::key(KeyCode::Up));
    assert_eq!(msg, Some(HeatmapMessage::SelectUp));
}

#[test]
fn test_arrow_down_maps_to_select_down() {
    let state = focused_3x3();
    let msg = Heatmap::handle_event(&state, &Event::key(KeyCode::Down));
    assert_eq!(msg, Some(HeatmapMessage::SelectDown));
}

#[test]
fn test_arrow_left_maps_to_select_left() {
    let state = focused_3x3();
    let msg = Heatmap::handle_event(&state, &Event::key(KeyCode::Left));
    assert_eq!(msg, Some(HeatmapMessage::SelectLeft));
}

#[test]
fn test_arrow_right_maps_to_select_right() {
    let state = focused_3x3();
    let msg = Heatmap::handle_event(&state, &Event::key(KeyCode::Right));
    assert_eq!(msg, Some(HeatmapMessage::SelectRight));
}

#[test]
fn test_hjkl_keys() {
    let state = focused_3x3();
    assert_eq!(
        Heatmap::handle_event(&state, &Event::char('k')),
        Some(HeatmapMessage::SelectUp)
    );
    assert_eq!(
        Heatmap::handle_event(&state, &Event::char('j')),
        Some(HeatmapMessage::SelectDown)
    );
    assert_eq!(
        Heatmap::handle_event(&state, &Event::char('h')),
        Some(HeatmapMessage::SelectLeft)
    );
    assert_eq!(
        Heatmap::handle_event(&state, &Event::char('l')),
        Some(HeatmapMessage::SelectRight)
    );
}

#[test]
fn test_enter_emits_cell_selected() {
    let mut state = focused_3x3();
    let output = state.dispatch_event(&Event::key(KeyCode::Enter));
    assert_eq!(
        output,
        Some(HeatmapOutput::CellSelected {
            row: 0,
            col: 0,
            value: 1.0,
        })
    );
}

// =============================================================================
// Disabled state
// =============================================================================

#[test]
fn test_disabled_ignores_events() {
    let mut state = focused_3x3();
    state.set_disabled(true);
    let msg = Heatmap::handle_event(&state, &Event::key(KeyCode::Down));
    assert_eq!(msg, None);
}

#[test]
fn test_unfocused_ignores_events() {
    let state = HeatmapState::with_data(vec![vec![1.0]]);
    let msg = Heatmap::handle_event(&state, &Event::key(KeyCode::Down));
    assert_eq!(msg, None);
}

// =============================================================================
// Instance methods
// =============================================================================

#[test]
fn test_instance_handle_event() {
    let state = focused_3x3();
    let msg = state.handle_event(&Event::key(KeyCode::Down));
    assert_eq!(msg, Some(HeatmapMessage::SelectDown));
}

#[test]
fn test_instance_update() {
    let mut state = focused_3x3();
    let output = state.update(HeatmapMessage::SelectDown);
    assert_eq!(
        output,
        Some(HeatmapOutput::SelectionChanged { row: 1, col: 0 })
    );
}

#[test]
fn test_instance_dispatch_event() {
    let mut state = focused_3x3();
    let output = state.dispatch_event(&Event::key(KeyCode::Down));
    assert_eq!(
        output,
        Some(HeatmapOutput::SelectionChanged { row: 1, col: 0 })
    );
}

// =============================================================================
// Message handling
// =============================================================================

#[test]
fn test_set_data_message() {
    let mut state = focused_3x3();
    state.update(HeatmapMessage::SetData(vec![vec![10.0, 20.0]]));
    assert_eq!(state.rows(), 1);
    assert_eq!(state.cols(), 2);
    assert_eq!(state.get(0, 0), Some(10.0));
    // Selection clamped to new bounds
    assert_eq!(state.selected(), Some((0, 0)));
}

#[test]
fn test_set_data_empty() {
    let mut state = focused_3x3();
    state.update(HeatmapMessage::SetData(vec![]));
    assert_eq!(state.rows(), 0);
    assert_eq!(state.selected(), None);
}

#[test]
fn test_set_cell_message() {
    let mut state = HeatmapState::new(2, 2);
    state.set_focused(true);
    // Navigate to (1, 1) first
    state.update(HeatmapMessage::SelectDown);
    state.update(HeatmapMessage::SelectRight);
    // Now SetCell at a different position -- this should set the value
    state.update(HeatmapMessage::SetCell {
        row: 0,
        col: 0,
        value: 99.0,
    });
    assert_eq!(state.get(0, 0), Some(99.0));
}

#[test]
fn test_set_row_labels_message() {
    let mut state = HeatmapState::new(2, 2);
    state.update(HeatmapMessage::SetRowLabels(vec!["A".into(), "B".into()]));
    assert_eq!(state.row_labels(), &["A", "B"]);
}

#[test]
fn test_set_col_labels_message() {
    let mut state = HeatmapState::new(2, 2);
    state.update(HeatmapMessage::SetColLabels(vec!["X".into(), "Y".into()]));
    assert_eq!(state.col_labels(), &["X", "Y"]);
}

#[test]
fn test_set_color_scale_message() {
    let mut state = HeatmapState::new(2, 2);
    state.update(HeatmapMessage::SetColorScale(HeatmapColorScale::CoolToWarm));
    assert_eq!(state.color_scale(), &HeatmapColorScale::CoolToWarm);
}

#[test]
fn test_set_range_message() {
    let mut state = HeatmapState::new(2, 2);
    state.update(HeatmapMessage::SetRange(Some(-10.0), Some(10.0)));
    assert_eq!(state.effective_min(), -10.0);
    assert_eq!(state.effective_max(), 10.0);
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn test_1x1_grid() {
    let mut state = HeatmapState::with_data(vec![vec![42.0]]);
    state.set_focused(true);
    assert_eq!(state.selected(), Some((0, 0)));
    assert_eq!(state.selected_value(), Some(42.0));

    // Navigation should produce None (nowhere to go)
    let output = state.update(HeatmapMessage::SelectUp);
    assert_eq!(output, None);
    let output = state.update(HeatmapMessage::SelectDown);
    assert_eq!(output, None);
    let output = state.update(HeatmapMessage::SelectLeft);
    assert_eq!(output, None);
    let output = state.update(HeatmapMessage::SelectRight);
    assert_eq!(output, None);
}

#[test]
fn test_empty_grid_navigation() {
    let mut state = HeatmapState::default();
    state.set_focused(true);
    let output = state.update(HeatmapMessage::SelectDown);
    assert_eq!(output, None);
}

#[test]
fn test_uneven_row_lengths() {
    let state = HeatmapState::with_data(vec![
        vec![1.0, 2.0, 3.0],
        vec![4.0, 5.0], // shorter row
    ]);
    assert_eq!(state.get(1, 2), None); // beyond row 1's length
    assert_eq!(state.get(1, 1), Some(5.0));
}

#[test]
fn test_uneven_row_navigation() {
    let mut state = HeatmapState::with_data(vec![
        vec![1.0, 2.0, 3.0],
        vec![4.0, 5.0], // shorter row
    ]);
    state.set_focused(true);
    // Navigate to column 2 in row 0
    state.update(HeatmapMessage::SelectRight);
    state.update(HeatmapMessage::SelectRight);
    assert_eq!(state.selected(), Some((0, 2)));

    // Move down -- column should clamp to row 1's max
    state.update(HeatmapMessage::SelectDown);
    assert_eq!(state.selected(), Some((1, 1))); // clamped to col 1
}

// =============================================================================
// Focus and disabled
// =============================================================================

#[test]
fn test_focus_methods() {
    let mut state = HeatmapState::new(2, 2);
    assert!(!state.is_focused());
    state.set_focused(true);
    assert!(state.is_focused());
    state.set_focused(false);
    assert!(!state.is_focused());
}

#[test]
fn test_disabled_methods() {
    let mut state = HeatmapState::new(2, 2);
    assert!(!state.is_disabled());
    state.set_disabled(true);
    assert!(state.is_disabled());
    state.set_disabled(false);
    assert!(!state.is_disabled());
}

// =============================================================================
// Rendering
// =============================================================================

#[test]
fn test_render_empty() {
    let state = HeatmapState::default();
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Heatmap::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_small_grid() {
    let state = HeatmapState::with_data(vec![vec![0.0, 0.5, 1.0], vec![0.3, 0.7, 0.9]]);
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Heatmap::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_with_labels() {
    let state = HeatmapState::with_data(vec![vec![0.0, 0.5, 1.0], vec![0.3, 0.7, 0.9]])
        .with_row_labels(vec!["AM".into(), "PM".into()])
        .with_col_labels(vec!["Mon".into(), "Tue".into(), "Wed".into()])
        .with_title("Schedule");
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Heatmap::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_with_values() {
    let state = HeatmapState::with_data(vec![vec![1.5, 2.7], vec![3.1, 4.9]])
        .with_show_values(true)
        .with_title("Values");
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Heatmap::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_disabled() {
    let state = HeatmapState::with_data(vec![vec![1.0, 2.0]]).with_disabled(true);
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Heatmap::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_focused_with_selection() {
    let mut state = HeatmapState::with_data(vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]]);
    state.set_focused(true);
    state.update(HeatmapMessage::SelectDown);
    state.update(HeatmapMessage::SelectRight);
    // Selected cell is (1, 1)
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Heatmap::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_small_area() {
    let state = HeatmapState::with_data(vec![vec![1.0]]);
    let (mut terminal, theme) = test_utils::setup_render(5, 2);
    terminal
        .draw(|frame| {
            Heatmap::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

// =============================================================================
// Helper function tests
// =============================================================================

#[test]
fn test_truncate_str_fits() {
    assert_eq!(truncate_str("abc", 5), "abc");
}

#[test]
fn test_truncate_str_exact() {
    assert_eq!(truncate_str("abc", 3), "abc");
}

#[test]
fn test_truncate_str_too_long() {
    assert_eq!(truncate_str("abcdef", 3), "abc");
}

#[test]
fn test_truncate_str_zero_width() {
    assert_eq!(truncate_str("abc", 0), "");
}

#[test]
fn test_format_value_wide() {
    let s = format_value(3.75, 6);
    assert_eq!(s, "3.8");
}

#[test]
fn test_format_value_narrow() {
    let s = format_value(42.0, 3);
    assert_eq!(s, "42");
}

#[test]
fn test_format_value_zero_width() {
    let s = format_value(1.0, 0);
    assert_eq!(s, "");
}

#[test]
fn test_contrasting_fg_dark_bg() {
    assert_eq!(contrasting_fg(Color::Rgb(0, 0, 0)), Color::White);
}

#[test]
fn test_contrasting_fg_light_bg() {
    assert_eq!(contrasting_fg(Color::Rgb(255, 255, 255)), Color::Black);
}

#[test]
fn test_contrasting_fg_dark_gray() {
    assert_eq!(contrasting_fg(Color::DarkGray), Color::White);
}

// =============================================================================
// Annotation
// =============================================================================

#[test]
fn test_annotation_emitted() {
    use crate::annotation::with_annotations;
    let state = HeatmapState::with_data(vec![vec![1.0, 2.0]]);
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                Heatmap::view(&state, frame, frame.area(), &theme, &ViewContext::default());
            })
            .unwrap();
    });
    assert!(registry.get_by_id("heatmap").is_some());
}

// =============================================================================
// Color scale equality
// =============================================================================

#[test]
fn test_color_scale_default_is_green_to_red() {
    assert_eq!(HeatmapColorScale::default(), HeatmapColorScale::GreenToRed);
}

#[test]
fn test_color_scale_partial_eq() {
    assert_eq!(HeatmapColorScale::BlueToRed, HeatmapColorScale::BlueToRed);
    assert_ne!(HeatmapColorScale::BlueToRed, HeatmapColorScale::GreenToRed);
    assert_eq!(
        HeatmapColorScale::Intensity(Color::Red),
        HeatmapColorScale::Intensity(Color::Red)
    );
    assert_ne!(
        HeatmapColorScale::Intensity(Color::Red),
        HeatmapColorScale::Intensity(Color::Blue)
    );
}
