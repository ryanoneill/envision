use super::*;

// --- Construction tests ---

#[test]
fn test_new() {
    let state = SparklineState::new();
    assert!(state.is_empty());
    assert_eq!(state.len(), 0);
    assert_eq!(state.title(), None);
    assert_eq!(state.direction(), &SparklineDirection::LeftToRight);
    assert_eq!(state.color(), None);
    assert_eq!(state.max_display_points(), None);
}

#[test]
fn test_default() {
    let state = SparklineState::default();
    assert!(state.is_empty());
    assert_eq!(state.direction(), &SparklineDirection::LeftToRight);
}

#[test]
fn test_default_matches_init() {
    let default_state = SparklineState::default();
    let init_state = Sparkline::init();

    assert_eq!(default_state.data(), init_state.data());
    assert_eq!(default_state.title(), init_state.title());
    assert_eq!(default_state.direction(), init_state.direction());
    assert_eq!(default_state.color(), init_state.color());
    assert_eq!(
        default_state.max_display_points(),
        init_state.max_display_points()
    );
}

#[test]
fn test_with_data() {
    let state = SparklineState::with_data(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    assert_eq!(state.data(), &[1.0, 2.0, 3.0, 4.0, 5.0]);
    assert_eq!(state.len(), 5);
    assert!(!state.is_empty());
}

#[test]
fn test_with_title() {
    let state = SparklineState::new().with_title("CPU Usage");
    assert_eq!(state.title(), Some("CPU Usage"));
}

#[test]
fn test_with_direction() {
    let state = SparklineState::new().with_direction(SparklineDirection::RightToLeft);
    assert_eq!(state.direction(), &SparklineDirection::RightToLeft);
}

#[test]
fn test_with_color() {
    let state = SparklineState::new().with_color(Color::Green);
    assert_eq!(state.color(), Some(Color::Green));
}

#[test]
fn test_with_max_display_points() {
    let state = SparklineState::new().with_max_display_points(10);
    assert_eq!(state.max_display_points(), Some(10));
}

#[test]
fn test_builder_chaining() {
    let state = SparklineState::with_data(vec![1.0, 2.0, 3.0])
        .with_title("Metrics")
        .with_direction(SparklineDirection::RightToLeft)
        .with_max_display_points(50)
        .with_color(Color::Cyan);

    assert_eq!(state.data(), &[1.0, 2.0, 3.0]);
    assert_eq!(state.title(), Some("Metrics"));
    assert_eq!(state.direction(), &SparklineDirection::RightToLeft);
    assert_eq!(state.max_display_points(), Some(50));
    assert_eq!(state.color(), Some(Color::Cyan));
}

// --- Direction tests ---

#[test]
fn test_direction_default() {
    let dir = SparklineDirection::default();
    assert_eq!(dir, SparklineDirection::LeftToRight);
}

#[test]
fn test_direction_clone() {
    let dir = SparklineDirection::RightToLeft;
    let cloned = dir.clone();
    assert_eq!(dir, cloned);
}

#[test]
fn test_direction_into_render_direction() {
    let ltr: RenderDirection = SparklineDirection::LeftToRight.into();
    assert_eq!(ltr, RenderDirection::LeftToRight);

    let rtl: RenderDirection = SparklineDirection::RightToLeft.into();
    assert_eq!(rtl, RenderDirection::RightToLeft);
}

// --- Data operation tests ---

#[test]
fn test_push() {
    let mut state = SparklineState::new();
    state.push(10.0);
    state.push(20.0);
    state.push(30.0);
    assert_eq!(state.data(), &[10.0, 20.0, 30.0]);
    assert_eq!(state.len(), 3);
}

#[test]
fn test_push_bounded_within_limit() {
    let mut state = SparklineState::new();
    state.push_bounded(1.0, 5);
    state.push_bounded(2.0, 5);
    state.push_bounded(3.0, 5);
    assert_eq!(state.data(), &[1.0, 2.0, 3.0]);
    assert_eq!(state.len(), 3);
}

#[test]
fn test_push_bounded_exceeds_limit() {
    let mut state = SparklineState::with_data(vec![1.0, 2.0, 3.0]);
    state.push_bounded(4.0, 3);
    assert_eq!(state.data(), &[2.0, 3.0, 4.0]);
    assert_eq!(state.len(), 3);
}

#[test]
fn test_push_bounded_max_one() {
    let mut state = SparklineState::with_data(vec![1.0, 2.0, 3.0]);
    state.push_bounded(99.0, 1);
    assert_eq!(state.data(), &[99.0]);
}

#[test]
fn test_clear() {
    let mut state = SparklineState::with_data(vec![1.0, 2.0, 3.0]);
    state.clear();
    assert!(state.is_empty());
    assert_eq!(state.len(), 0);
}

#[test]
fn test_len() {
    let state = SparklineState::with_data(vec![1.0, 2.0, 3.0, 4.0]);
    assert_eq!(state.len(), 4);
}

#[test]
fn test_is_empty_true() {
    let state = SparklineState::new();
    assert!(state.is_empty());
}

#[test]
fn test_is_empty_false() {
    let state = SparklineState::with_data(vec![1.0]);
    assert!(!state.is_empty());
}

#[test]
fn test_last() {
    let state = SparklineState::with_data(vec![1.0, 2.0, 3.0]);
    assert_eq!(state.last(), Some(3.0));
}

#[test]
fn test_last_empty() {
    let state = SparklineState::new();
    assert_eq!(state.last(), None);
}

#[test]
fn test_min() {
    let state = SparklineState::with_data(vec![5.0, 3.0, 7.0, 1.0, 9.0]);
    assert_eq!(state.min(), Some(1.0));
}

#[test]
fn test_min_empty() {
    let state = SparklineState::new();
    assert_eq!(state.min(), None);
}

#[test]
fn test_max() {
    let state = SparklineState::with_data(vec![5.0, 3.0, 7.0, 1.0, 9.0]);
    assert_eq!(state.max(), Some(9.0));
}

#[test]
fn test_max_empty() {
    let state = SparklineState::new();
    assert_eq!(state.max(), None);
}

#[test]
fn test_min_max_single_element() {
    let state = SparklineState::with_data(vec![42.0]);
    assert_eq!(state.min(), Some(42.0));
    assert_eq!(state.max(), Some(42.0));
}
// --- Update/message tests ---

#[test]
fn test_update_push() {
    let mut state = SparklineState::new();
    let output = Sparkline::update(&mut state, SparklineMessage::Push(42.0));
    assert_eq!(output, None);
    assert_eq!(state.data(), &[42.0]);
}

#[test]
fn test_update_push_bounded() {
    let mut state = SparklineState::with_data(vec![1.0, 2.0, 3.0]);
    let output = Sparkline::update(&mut state, SparklineMessage::PushBounded(4.0, 3));
    assert_eq!(output, None);
    assert_eq!(state.data(), &[2.0, 3.0, 4.0]);
}

#[test]
fn test_update_set_data() {
    let mut state = SparklineState::with_data(vec![1.0, 2.0, 3.0]);
    let output = Sparkline::update(&mut state, SparklineMessage::SetData(vec![10.0, 20.0]));
    assert_eq!(output, None);
    assert_eq!(state.data(), &[10.0, 20.0]);
}

#[test]
fn test_update_clear() {
    let mut state = SparklineState::with_data(vec![1.0, 2.0, 3.0]);
    let output = Sparkline::update(&mut state, SparklineMessage::Clear);
    assert_eq!(output, None);
    assert!(state.is_empty());
}

#[test]
fn test_update_set_max_display_points() {
    let mut state = SparklineState::new();
    let output = Sparkline::update(&mut state, SparklineMessage::SetMaxDisplayPoints(Some(10)));
    assert_eq!(output, None);
    assert_eq!(state.max_display_points(), Some(10));
}

#[test]
fn test_update_set_max_display_points_none() {
    let mut state = SparklineState::new().with_max_display_points(10);
    Sparkline::update(&mut state, SparklineMessage::SetMaxDisplayPoints(None));
    assert_eq!(state.max_display_points(), None);
}

// --- Instance method tests ---

#[test]
fn test_instance_update() {
    let mut state = SparklineState::new();
    let output = state.update(SparklineMessage::Push(42.0));
    assert_eq!(output, None);
    assert_eq!(state.last(), Some(42.0));
}

// --- Component trait tests ---

#[test]
fn test_init() {
    let state = Sparkline::init();
    assert!(state.is_empty());
    assert_eq!(state.direction(), &SparklineDirection::LeftToRight);
}

// --- View/snapshot tests ---

#[test]
fn test_view_empty() {
    let state = SparklineState::new();
    let (mut terminal, theme) = crate::component::test_utils::setup_render(20, 3);

    terminal
        .draw(|frame| {
            Sparkline::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_data() {
    let state = SparklineState::with_data(vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(20, 1);

    terminal
        .draw(|frame| {
            Sparkline::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_title() {
    let state =
        SparklineState::with_data(vec![1.0, 3.0, 5.0, 7.0, 5.0, 3.0, 1.0]).with_title("CPU");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(20, 3);

    terminal
        .draw(|frame| {
            Sparkline::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}
#[test]
fn test_view_right_to_left() {
    let state = SparklineState::with_data(vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0])
        .with_direction(SparklineDirection::RightToLeft);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(20, 1);

    terminal
        .draw(|frame| {
            Sparkline::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_max_display_points() {
    let state = SparklineState::with_data(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0])
        .with_max_display_points(5);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(20, 1);

    terminal
        .draw(|frame| {
            Sparkline::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_color() {
    let state = SparklineState::with_data(vec![1.0, 3.0, 5.0, 7.0]).with_color(Color::Red);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(20, 1);

    terminal
        .draw(|frame| {
            Sparkline::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

// --- Annotation test ---

#[test]
fn test_annotation_emitted() {
    use crate::annotation::{WidgetType, with_annotations};

    let state = SparklineState::with_data(vec![1.0, 2.0, 3.0]).with_title("Metrics");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 3);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                Sparkline::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.find_by_type(&WidgetType::Sparkline);
    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].annotation.label, Some("Metrics".to_string()));
}

#[test]
fn test_annotation_emitted_no_title() {
    use crate::annotation::{WidgetType, with_annotations};

    let state = SparklineState::with_data(vec![1.0, 2.0, 3.0]);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 1);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                Sparkline::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.find_by_type(&WidgetType::Sparkline);
    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].annotation.label, Some("".to_string()));
}

#[test]
fn view_chrome_owned_no_outer_border() {
    let state = SparklineState::with_data(vec![1.0, 5.0, 3.0, 8.0, 2.0]).with_title("CPU");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(20, 5);
    terminal
        .draw(|frame| {
            Sparkline::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).chrome_owned(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}
