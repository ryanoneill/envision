use super::*;
use crate::component::test_utils;

#[test]
fn test_with_bounds() {
    let s = DataSeries::new("p50", vec![10.0, 20.0]).with_bounds(vec![5.0, 15.0], vec![15.0, 25.0]);
    assert_eq!(s.lower_bound(), Some([5.0, 15.0].as_slice()));
    assert_eq!(s.upper_bound(), Some([15.0, 25.0].as_slice()));
}

#[test]
fn test_bounds_none_by_default() {
    let s = DataSeries::new("Test", vec![1.0]);
    assert_eq!(s.upper_bound(), None);
    assert_eq!(s.lower_bound(), None);
}

#[test]
fn test_upper_bound_affects_effective_max() {
    let s = DataSeries::new("p50", vec![10.0, 20.0]).with_upper_bound(vec![50.0, 60.0]);
    let state = ChartState::line(vec![s]);
    assert_eq!(state.effective_max(), 60.0);
}

#[test]
fn test_lower_bound_affects_effective_min() {
    let s = DataSeries::new("p50", vec![10.0, 20.0]).with_lower_bound(vec![-5.0, 5.0]);
    let state = ChartState::line(vec![s]);
    assert_eq!(state.effective_min(), -5.0);
}

#[test]
fn test_manual_y_range_overrides_bounds() {
    let s = DataSeries::new("p50", vec![50.0]).with_bounds(vec![0.0], vec![100.0]);
    let state = ChartState::line(vec![s]).with_y_range(25.0, 75.0);
    assert_eq!(state.effective_min(), 25.0);
    assert_eq!(state.effective_max(), 75.0);
}

#[test]
fn test_render_with_bounds_shows_shading() {
    let s = DataSeries::new("p50", vec![50.0, 60.0, 55.0, 70.0, 65.0]).with_bounds(
        vec![30.0, 40.0, 35.0, 50.0, 45.0],
        vec![70.0, 80.0, 75.0, 90.0, 85.0],
    );
    let state = ChartState::line(vec![s]);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    let output = terminal.backend().to_string();
    assert!(
        output.contains('\u{2591}'),
        "Chart with error bands should contain shade fill"
    );
}

#[test]
fn test_render_without_bounds_no_shading() {
    let s = DataSeries::new("plain", vec![50.0, 60.0, 55.0, 70.0, 65.0]);
    let state = ChartState::line(vec![s]);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    let output = terminal.backend().to_string();
    assert!(
        !output.contains('\u{2591}'),
        "Line chart without bounds should not contain shade fill"
    );
}

#[test]
fn test_render_with_only_upper_bound() {
    let s = DataSeries::new("p50", vec![50.0, 60.0, 55.0, 70.0, 65.0])
        .with_upper_bound(vec![60.0, 70.0, 65.0, 80.0, 75.0]);
    let state = ChartState::line(vec![s]);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    let output = terminal.backend().to_string();
    assert!(
        output.contains('\u{2591}'),
        "Chart with only upper bound should contain shade fill"
    );
}

#[test]
fn test_render_bounds_disabled() {
    let s = DataSeries::new("p50", vec![50.0, 60.0, 55.0])
        .with_bounds(vec![40.0, 50.0, 45.0], vec![60.0, 70.0, 65.0]);
    let state = ChartState::line(vec![s]);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(
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
fn test_bounds_clone_and_eq() {
    let a = DataSeries::new("p50", vec![10.0]).with_bounds(vec![5.0], vec![15.0]);
    let b = a.clone();
    assert_eq!(a, b);
    let c = DataSeries::new("p50", vec![10.0]);
    assert_ne!(a, c);
}
