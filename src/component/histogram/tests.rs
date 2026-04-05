use super::*;
use crate::component::test_utils;

// =============================================================================
// Construction
// =============================================================================

#[test]
fn test_new() {
    let state = HistogramState::new();
    assert!(state.data().is_empty());
    assert_eq!(state.bin_count(), 10);
    assert_eq!(state.title(), None);
    assert_eq!(state.x_label(), None);
    assert_eq!(state.y_label(), None);
    assert_eq!(state.color(), None);
    assert!(!state.show_counts());
}

#[test]
fn test_default() {
    let state = HistogramState::default();
    assert!(state.data().is_empty());
    assert_eq!(state.bin_count(), 10);
}

#[test]
fn test_with_data() {
    let state = HistogramState::with_data(vec![1.0, 2.0, 3.0]);
    assert_eq!(state.data(), &[1.0, 2.0, 3.0]);
    assert_eq!(state.bin_count(), 10);
}

#[test]
fn test_with_bin_count() {
    let state = HistogramState::new().with_bin_count(5);
    assert_eq!(state.bin_count(), 5);
}

#[test]
fn test_with_bin_count_zero_clamped() {
    let state = HistogramState::new().with_bin_count(0);
    assert_eq!(state.bin_count(), 1);
}

#[test]
fn test_with_range() {
    let state = HistogramState::new().with_range(0.0, 100.0);
    assert_eq!(state.effective_min(), 0.0);
    assert_eq!(state.effective_max(), 100.0);
}

#[test]
fn test_with_title() {
    let state = HistogramState::new().with_title("Test Histogram");
    assert_eq!(state.title(), Some("Test Histogram"));
}

#[test]
fn test_with_x_label() {
    let state = HistogramState::new().with_x_label("Values");
    assert_eq!(state.x_label(), Some("Values"));
}

#[test]
fn test_with_y_label() {
    let state = HistogramState::new().with_y_label("Frequency");
    assert_eq!(state.y_label(), Some("Frequency"));
}

#[test]
fn test_with_color() {
    let state = HistogramState::new().with_color(Color::Red);
    assert_eq!(state.color(), Some(Color::Red));
}

#[test]
fn test_with_show_counts() {
    let state = HistogramState::new().with_show_counts(true);
    assert!(state.show_counts());
}

// =============================================================================
// Data operations
// =============================================================================

#[test]
fn test_push() {
    let mut state = HistogramState::new();
    state.push(42.0);
    assert_eq!(state.data(), &[42.0]);
    state.push(43.0);
    assert_eq!(state.data(), &[42.0, 43.0]);
}

#[test]
fn test_push_batch() {
    let mut state = HistogramState::new();
    state.push_batch(&[1.0, 2.0, 3.0]);
    assert_eq!(state.data(), &[1.0, 2.0, 3.0]);
    state.push_batch(&[4.0, 5.0]);
    assert_eq!(state.data(), &[1.0, 2.0, 3.0, 4.0, 5.0]);
}

#[test]
fn test_clear() {
    let mut state = HistogramState::with_data(vec![1.0, 2.0, 3.0]);
    state.clear();
    assert!(state.data().is_empty());
}

#[test]
fn test_set_bin_count() {
    let mut state = HistogramState::new();
    state.set_bin_count(20);
    assert_eq!(state.bin_count(), 20);
}

#[test]
fn test_set_bin_count_zero_clamped() {
    let mut state = HistogramState::new();
    state.set_bin_count(0);
    assert_eq!(state.bin_count(), 1);
}

// =============================================================================
// Range: auto vs manual
// =============================================================================

#[test]
fn test_effective_min_auto() {
    let state = HistogramState::with_data(vec![5.0, 10.0, 15.0]);
    assert_eq!(state.effective_min(), 5.0);
}

#[test]
fn test_effective_max_auto() {
    let state = HistogramState::with_data(vec![5.0, 10.0, 15.0]);
    assert_eq!(state.effective_max(), 15.0);
}

#[test]
fn test_effective_min_manual() {
    let state = HistogramState::with_data(vec![5.0, 10.0]).with_range(0.0, 20.0);
    assert_eq!(state.effective_min(), 0.0);
}

#[test]
fn test_effective_max_manual() {
    let state = HistogramState::with_data(vec![5.0, 10.0]).with_range(0.0, 20.0);
    assert_eq!(state.effective_max(), 20.0);
}

#[test]
fn test_effective_min_empty_no_manual() {
    let state = HistogramState::new();
    assert_eq!(state.effective_min(), 0.0);
}

#[test]
fn test_effective_max_empty_no_manual() {
    let state = HistogramState::new();
    assert_eq!(state.effective_max(), 0.0);
}

// =============================================================================
// Binning: compute_bins with known data
// =============================================================================

#[test]
fn test_compute_bins_uniform() {
    // 5 data points uniformly distributed in 5 bins from 1.0 to 5.0
    let state = HistogramState::with_data(vec![1.0, 2.0, 3.0, 4.0, 5.0])
        .with_bin_count(5)
        .with_range(0.0, 5.0);
    let bins = state.compute_bins();
    assert_eq!(bins.len(), 5);
    // Total count should equal data length
    let total: usize = bins.iter().map(|(_, _, c)| *c).sum();
    assert_eq!(total, 5);
}

#[test]
fn test_compute_bins_all_in_one() {
    // All values in the range [0, 1), bin count = 3, range [0, 3)
    let state = HistogramState::with_data(vec![0.1, 0.2, 0.3, 0.4, 0.5])
        .with_bin_count(3)
        .with_range(0.0, 3.0);
    let bins = state.compute_bins();
    assert_eq!(bins.len(), 3);
    assert_eq!(bins[0].2, 5); // All in first bin
    assert_eq!(bins[1].2, 0);
    assert_eq!(bins[2].2, 0);
}

#[test]
fn test_compute_bins_max_value_in_last_bin() {
    // The maximum value should be included in the last bin
    let state = HistogramState::with_data(vec![0.0, 5.0, 10.0])
        .with_bin_count(2)
        .with_range(0.0, 10.0);
    let bins = state.compute_bins();
    assert_eq!(bins.len(), 2);
    // 0.0 => bin 0, 5.0 => bin 1, 10.0 => bin 1 (clamped to last)
    assert_eq!(bins[0].2, 1);
    assert_eq!(bins[1].2, 2);
}

#[test]
fn test_compute_bins_bin_edges() {
    let state = HistogramState::with_data(vec![1.0, 2.0, 3.0])
        .with_bin_count(3)
        .with_range(1.0, 4.0);
    let bins = state.compute_bins();
    assert_eq!(bins.len(), 3);
    // Verify edges
    assert!((bins[0].0 - 1.0).abs() < f64::EPSILON);
    assert!((bins[0].1 - 2.0).abs() < f64::EPSILON);
    assert!((bins[1].0 - 2.0).abs() < f64::EPSILON);
    assert!((bins[1].1 - 3.0).abs() < f64::EPSILON);
    assert!((bins[2].0 - 3.0).abs() < f64::EPSILON);
    assert!((bins[2].1 - 4.0).abs() < f64::EPSILON);
}

// =============================================================================
// Binning edge cases
// =============================================================================

#[test]
fn test_compute_bins_empty_data() {
    let state = HistogramState::new().with_bin_count(5);
    let bins = state.compute_bins();
    assert_eq!(bins.len(), 5);
    for (_, _, count) in &bins {
        assert_eq!(*count, 0);
    }
}

#[test]
fn test_compute_bins_empty_data_with_range() {
    let state = HistogramState::new()
        .with_bin_count(3)
        .with_range(0.0, 30.0);
    let bins = state.compute_bins();
    assert_eq!(bins.len(), 3);
    assert!((bins[0].0 - 0.0).abs() < f64::EPSILON);
    assert!((bins[0].1 - 10.0).abs() < f64::EPSILON);
    for (_, _, count) in &bins {
        assert_eq!(*count, 0);
    }
}

#[test]
fn test_compute_bins_single_point() {
    let state = HistogramState::with_data(vec![5.0]).with_bin_count(3);
    let bins = state.compute_bins();
    // All same value means zero range => single bin
    assert_eq!(bins.len(), 1);
    assert_eq!(bins[0].2, 1);
}

#[test]
fn test_compute_bins_all_same_value() {
    let state = HistogramState::with_data(vec![7.0, 7.0, 7.0, 7.0]).with_bin_count(5);
    let bins = state.compute_bins();
    // Zero range => single bin with all data
    assert_eq!(bins.len(), 1);
    assert_eq!(bins[0].2, 4);
    // Bin should span around the value
    assert!((bins[0].0 - 6.5).abs() < f64::EPSILON);
    assert!((bins[0].1 - 7.5).abs() < f64::EPSILON);
}

#[test]
fn test_compute_bins_negative_values() {
    let state = HistogramState::with_data(vec![-5.0, -3.0, -1.0, 1.0, 3.0, 5.0]).with_bin_count(5);
    let bins = state.compute_bins();
    assert_eq!(bins.len(), 5);
    let total: usize = bins.iter().map(|(_, _, c)| *c).sum();
    assert_eq!(total, 6);
    // Verify range covers negative to positive
    assert!(bins[0].0 < 0.0);
    assert!(bins[4].1 > 0.0);
}

#[test]
fn test_compute_bins_bin_count_one() {
    let state = HistogramState::with_data(vec![1.0, 2.0, 3.0, 4.0, 5.0]).with_bin_count(1);
    let bins = state.compute_bins();
    assert_eq!(bins.len(), 1);
    assert_eq!(bins[0].2, 5);
}

#[test]
fn test_compute_bins_auto_range() {
    // Without manual range, should use data min/max
    let state = HistogramState::with_data(vec![10.0, 20.0, 30.0]).with_bin_count(3);
    let bins = state.compute_bins();
    assert_eq!(bins.len(), 3);
    assert!((bins[0].0 - 10.0).abs() < f64::EPSILON);
    let total: usize = bins.iter().map(|(_, _, c)| *c).sum();
    assert_eq!(total, 3);
}

// =============================================================================
// Message handling (update)
// =============================================================================

#[test]
fn test_update_set_data() {
    let mut state = HistogramState::new();
    Histogram::update(&mut state, HistogramMessage::SetData(vec![1.0, 2.0]));
    assert_eq!(state.data(), &[1.0, 2.0]);
}

#[test]
fn test_update_push_data() {
    let mut state = HistogramState::with_data(vec![1.0]);
    Histogram::update(&mut state, HistogramMessage::PushData(2.0));
    assert_eq!(state.data(), &[1.0, 2.0]);
}

#[test]
fn test_update_push_data_batch() {
    let mut state = HistogramState::new();
    Histogram::update(
        &mut state,
        HistogramMessage::PushDataBatch(vec![1.0, 2.0, 3.0]),
    );
    assert_eq!(state.data(), &[1.0, 2.0, 3.0]);
}

#[test]
fn test_update_clear() {
    let mut state = HistogramState::with_data(vec![1.0, 2.0]);
    Histogram::update(&mut state, HistogramMessage::Clear);
    assert!(state.data().is_empty());
}

#[test]
fn test_update_set_bin_count() {
    let mut state = HistogramState::new();
    Histogram::update(&mut state, HistogramMessage::SetBinCount(20));
    assert_eq!(state.bin_count(), 20);
}

#[test]
fn test_update_set_bin_count_zero_clamped() {
    let mut state = HistogramState::new();
    Histogram::update(&mut state, HistogramMessage::SetBinCount(0));
    assert_eq!(state.bin_count(), 1);
}

#[test]
fn test_update_set_range() {
    let mut state = HistogramState::new();
    Histogram::update(
        &mut state,
        HistogramMessage::SetRange(Some(0.0), Some(100.0)),
    );
    assert_eq!(state.effective_min(), 0.0);
    assert_eq!(state.effective_max(), 100.0);
}

#[test]
fn test_update_returns_none() {
    let mut state = HistogramState::new();
    let output = Histogram::update(&mut state, HistogramMessage::Clear);
    assert!(output.is_none());
}

// =============================================================================
// Instance methods
// =============================================================================

#[test]
fn test_instance_update() {
    let mut state = HistogramState::new();
    let output = state.update(HistogramMessage::PushData(1.0));
    assert!(output.is_none());
    assert_eq!(state.data(), &[1.0]);
}

// =============================================================================
// PartialEq
// =============================================================================

#[test]
fn test_partial_eq() {
    let state1 = HistogramState::with_data(vec![1.0, 2.0, 3.0]).with_bin_count(5);
    let state2 = HistogramState::with_data(vec![1.0, 2.0, 3.0]).with_bin_count(5);
    assert_eq!(state1, state2);
}

#[test]
fn test_partial_eq_different_data() {
    let state1 = HistogramState::with_data(vec![1.0, 2.0]);
    let state2 = HistogramState::with_data(vec![3.0, 4.0]);
    assert_ne!(state1, state2);
}

// =============================================================================
// View rendering
// =============================================================================

#[test]
fn test_render_empty() {
    let state = HistogramState::new();
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Histogram::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_with_data() {
    let state =
        HistogramState::with_data(vec![1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5, 5.0, 5.0, 5.0])
            .with_bin_count(5)
            .with_title("Test Histogram");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Histogram::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_with_labels() {
    let state = HistogramState::with_data(vec![10.0, 20.0, 30.0, 40.0, 50.0])
        .with_bin_count(5)
        .with_title("Latency")
        .with_x_label("ms")
        .with_y_label("Count");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Histogram::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_with_show_counts() {
    let state = HistogramState::with_data(vec![1.0, 1.0, 2.0, 3.0, 3.0, 3.0])
        .with_bin_count(3)
        .with_show_counts(true);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Histogram::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_disabled() {
    let state = HistogramState::with_data(vec![1.0, 2.0, 3.0]).with_bin_count(3);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Histogram::view(
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
fn test_render_focused() {
    let state =
        HistogramState::with_data(vec![1.0, 2.0, 3.0, 4.0, 5.0]).with_title("Focused Histogram");

    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Histogram::view(
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
    let state = HistogramState::with_data(vec![1.0, 2.0, 3.0]);
    let (mut terminal, theme) = test_utils::setup_render(60, 2);
    terminal
        .draw(|frame| {
            Histogram::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_with_color() {
    let state = HistogramState::with_data(vec![1.0, 2.0, 3.0, 4.0])
        .with_bin_count(4)
        .with_color(Color::Green);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Histogram::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

// =============================================================================
// Snapshot tests
// =============================================================================

#[test]
fn test_snapshot_empty() {
    let state = HistogramState::new();
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Histogram::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_with_data() {
    let state =
        HistogramState::with_data(vec![1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5, 5.0, 5.0, 5.0])
            .with_bin_count(5)
            .with_title("Value Distribution");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Histogram::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_with_title() {
    let state = HistogramState::with_data(vec![10.0, 20.0, 30.0, 40.0, 50.0])
        .with_bin_count(5)
        .with_title("Response Times");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Histogram::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_disabled() {
    let state = HistogramState::with_data(vec![1.0, 2.0, 3.0, 4.0, 5.0])
        .with_bin_count(5)
        .with_title("Disabled Histogram");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Histogram::view(
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

// =============================================================================
// Annotation tests
// =============================================================================

#[test]
fn test_annotation_emitted() {
    use crate::annotation::with_annotations;
    let state = HistogramState::with_data(vec![1.0, 2.0, 3.0]);
    let (mut terminal, theme) = test_utils::setup_render(40, 15);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                Histogram::view(&state, frame, frame.area(), &theme, &ViewContext::default());
            })
            .unwrap();
    });
    assert!(registry.get_by_id("histogram").is_some());
}
