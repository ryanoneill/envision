use super::*;

// ==================== BreadcrumbSegment Tests ====================

#[test]
fn test_segment_new() {
    let segment = BreadcrumbSegment::new("Home");
    assert_eq!(segment.label(), "Home");
    assert_eq!(segment.data(), None);
}

#[test]
fn test_segment_with_data() {
    let segment = BreadcrumbSegment::new("Products").with_data("/products");
    assert_eq!(segment.label(), "Products");
    assert_eq!(segment.data(), Some("/products"));
}

#[test]
fn test_segment_accessors() {
    let segment = BreadcrumbSegment::new("Test").with_data("data");
    assert_eq!(segment.label(), "Test");
    assert_eq!(segment.data(), Some("data"));
}

#[test]
fn test_segment_clone() {
    let segment = BreadcrumbSegment::new("Clone").with_data("data");
    let cloned = segment.clone();
    assert_eq!(segment, cloned);
}

#[test]
fn test_segment_eq() {
    let seg1 = BreadcrumbSegment::new("Test").with_data("data");
    let seg2 = BreadcrumbSegment::new("Test").with_data("data");
    let seg3 = BreadcrumbSegment::new("Other");
    assert_eq!(seg1, seg2);
    assert_ne!(seg1, seg3);
}

// ==================== State Creation Tests ====================

#[test]
fn test_new() {
    let segments = vec![
        BreadcrumbSegment::new("Home"),
        BreadcrumbSegment::new("Products"),
    ];
    let state = BreadcrumbState::new(segments);
    assert_eq!(state.len(), 2);
    assert_eq!(state.focused_index(), 0);
}

#[test]
fn test_from_labels() {
    let state = BreadcrumbState::from_labels(vec!["Home", "Products", "Item"]);
    assert_eq!(state.len(), 3);
    assert_eq!(state.segments()[0].label(), "Home");
    assert_eq!(state.segments()[1].label(), "Products");
    assert_eq!(state.segments()[2].label(), "Item");
}

#[test]
fn test_from_path() {
    let state = BreadcrumbState::from_path("home/user/documents", "/");
    assert_eq!(state.len(), 3);
    assert_eq!(state.segments()[0].label(), "home");
    assert_eq!(state.segments()[1].label(), "user");
    assert_eq!(state.segments()[2].label(), "documents");
}

#[test]
fn test_from_path_with_leading_separator() {
    let state = BreadcrumbState::from_path("/home/user", "/");
    assert_eq!(state.len(), 2);
    assert_eq!(state.segments()[0].label(), "home");
}

#[test]
fn test_default() {
    let state = BreadcrumbState::default();
    assert!(state.is_empty());
    assert_eq!(state.separator(), " > ");
    assert_eq!(state.max_visible(), None);
}

#[test]
fn test_new_empty() {
    let state = BreadcrumbState::new(vec![]);
    assert!(state.is_empty());
    assert_eq!(state.len(), 0);
}

// ==================== Accessor Tests ====================

#[test]
fn test_segments() {
    let state = BreadcrumbState::from_labels(vec!["A", "B"]);
    let segments = state.segments();
    assert_eq!(segments.len(), 2);
}

#[test]
fn test_len() {
    let state = BreadcrumbState::from_labels(vec!["A", "B", "C"]);
    assert_eq!(state.len(), 3);
}

#[test]
fn test_is_empty() {
    let empty = BreadcrumbState::default();
    let non_empty = BreadcrumbState::from_labels(vec!["A"]);
    assert!(empty.is_empty());
    assert!(!non_empty.is_empty());
}

#[test]
fn test_focused_index() {
    let state = BreadcrumbState::from_labels(vec!["A", "B"]);
    assert_eq!(state.focused_index(), 0);
}

#[test]
fn test_focused_segment() {
    let state = BreadcrumbState::from_labels(vec!["Home", "Products"]);
    assert_eq!(state.focused_segment().unwrap().label(), "Home");
}

#[test]
fn test_focused_segment_empty() {
    let state = BreadcrumbState::default();
    assert!(state.focused_segment().is_none());
}

#[test]
fn test_is_disabled() {
    let mut state = BreadcrumbState::default();
    assert!(!state.is_disabled());
    state.set_disabled(true);
    assert!(state.is_disabled());
}

#[test]
fn test_separator() {
    let state = BreadcrumbState::default();
    assert_eq!(state.separator(), " > ");
}

#[test]
fn test_max_visible() {
    let mut state = BreadcrumbState::default();
    assert_eq!(state.max_visible(), None);
    state.set_max_visible(Some(3));
    assert_eq!(state.max_visible(), Some(3));
}

#[test]
fn test_current() {
    let state = BreadcrumbState::from_labels(vec!["Home", "Products", "Item"]);
    assert_eq!(state.current().unwrap().label(), "Item");
}

#[test]
fn test_current_empty() {
    let state = BreadcrumbState::default();
    assert!(state.current().is_none());
}

// ==================== Mutator Tests ====================

#[test]
fn test_set_segments() {
    let mut state = BreadcrumbState::from_labels(vec!["A"]);
    state.set_segments(vec![
        BreadcrumbSegment::new("X"),
        BreadcrumbSegment::new("Y"),
    ]);
    assert_eq!(state.len(), 2);
    assert_eq!(state.segments()[0].label(), "X");
    assert_eq!(state.focused_index(), 0);
}

#[test]
fn test_push() {
    let mut state = BreadcrumbState::from_labels(vec!["Home"]);
    state.push(BreadcrumbSegment::new("Products"));
    assert_eq!(state.len(), 2);
    assert_eq!(state.segments()[1].label(), "Products");
}

#[test]
fn test_pop() {
    let mut state = BreadcrumbState::from_labels(vec!["Home", "Products", "Item"]);
    let popped = state.pop();
    assert_eq!(popped.unwrap().label(), "Item");
    assert_eq!(state.len(), 2);
}

#[test]
fn test_pop_adjusts_focus() {
    let mut state = BreadcrumbState::from_labels(vec!["Home", "Products"]);
    state.focused_index = 1;
    state.pop();
    assert_eq!(state.focused_index(), 0);
}

#[test]
fn test_pop_empty() {
    let mut state = BreadcrumbState::default();
    assert!(state.pop().is_none());
}

#[test]
fn test_set_separator() {
    let mut state = BreadcrumbState::default();
    state.set_separator(" / ");
    assert_eq!(state.separator(), " / ");
}

#[test]
fn test_set_max_visible() {
    let mut state = BreadcrumbState::default();
    state.set_max_visible(Some(5));
    assert_eq!(state.max_visible(), Some(5));
    state.set_max_visible(None);
    assert_eq!(state.max_visible(), None);
}

#[test]
fn test_set_disabled() {
    let mut state = BreadcrumbState::default();
    state.set_disabled(true);
    assert!(state.is_disabled());
    state.set_disabled(false);
    assert!(!state.is_disabled());
}

// ==================== Truncation Tests ====================

#[test]
fn test_is_truncated_false() {
    let mut state = BreadcrumbState::from_labels(vec!["A", "B", "C"]);
    state.set_max_visible(Some(5));
    assert!(!state.is_truncated());
}

#[test]
fn test_is_truncated_true() {
    let mut state = BreadcrumbState::from_labels(vec!["A", "B", "C", "D", "E"]);
    state.set_max_visible(Some(3));
    assert!(state.is_truncated());
}

#[test]
fn test_is_truncated_no_max() {
    let state = BreadcrumbState::from_labels(vec!["A", "B", "C", "D", "E"]);
    assert!(!state.is_truncated());
}

#[test]
fn test_visible_segments() {
    let mut state = BreadcrumbState::from_labels(vec!["A", "B", "C", "D", "E"]);
    state.set_max_visible(Some(3));
    let visible = state.visible_segments();
    assert_eq!(visible.len(), 3);
    assert_eq!(visible[0].label(), "C");
    assert_eq!(visible[1].label(), "D");
    assert_eq!(visible[2].label(), "E");
}

#[test]
fn test_visible_segments_no_truncation() {
    let state = BreadcrumbState::from_labels(vec!["A", "B", "C"]);
    let visible = state.visible_segments();
    assert_eq!(visible.len(), 3);
    assert_eq!(visible[0].label(), "A");
}

#[test]
fn test_truncation_shows_last_n() {
    let mut state =
        BreadcrumbState::from_labels(vec!["Root", "Level1", "Level2", "Level3", "Current"]);
    state.set_max_visible(Some(3));
    let visible = state.visible_segments();
    assert_eq!(visible[0].label(), "Level2");
    assert_eq!(visible[1].label(), "Level3");
    assert_eq!(visible[2].label(), "Current");
}

// ==================== Navigation Tests ====================

#[test]
fn test_left() {
    let mut state = BreadcrumbState::from_labels(vec!["A", "B", "C"]);
    state.focused_index = 2;
    let output = Breadcrumb::update(&mut state, BreadcrumbMessage::Left);
    assert_eq!(output, Some(BreadcrumbOutput::FocusChanged(1)));
    assert_eq!(state.focused_index(), 1);
}

#[test]
fn test_right() {
    let mut state = BreadcrumbState::from_labels(vec!["A", "B", "C"]);
    let output = Breadcrumb::update(&mut state, BreadcrumbMessage::Right);
    assert_eq!(output, Some(BreadcrumbOutput::FocusChanged(1)));
    assert_eq!(state.focused_index(), 1);
}

#[test]
fn test_left_at_start() {
    let mut state = BreadcrumbState::from_labels(vec!["A", "B", "C"]);
    let output = Breadcrumb::update(&mut state, BreadcrumbMessage::Left);
    assert_eq!(output, None);
    assert_eq!(state.focused_index(), 0);
}

#[test]
fn test_right_at_end() {
    let mut state = BreadcrumbState::from_labels(vec!["A", "B", "C"]);
    state.focused_index = 2;
    let output = Breadcrumb::update(&mut state, BreadcrumbMessage::Right);
    assert_eq!(output, None);
    assert_eq!(state.focused_index(), 2);
}

#[test]
fn test_first() {
    let mut state = BreadcrumbState::from_labels(vec!["A", "B", "C"]);
    state.focused_index = 2;
    let output = Breadcrumb::update(&mut state, BreadcrumbMessage::First);
    assert_eq!(output, Some(BreadcrumbOutput::FocusChanged(0)));
    assert_eq!(state.focused_index(), 0);
}

#[test]
fn test_first_already_at_first() {
    let mut state = BreadcrumbState::from_labels(vec!["A", "B", "C"]);
    let output = Breadcrumb::update(&mut state, BreadcrumbMessage::First);
    assert_eq!(output, None);
}

#[test]
fn test_last() {
    let mut state = BreadcrumbState::from_labels(vec!["A", "B", "C"]);
    let output = Breadcrumb::update(&mut state, BreadcrumbMessage::Last);
    assert_eq!(output, Some(BreadcrumbOutput::FocusChanged(2)));
    assert_eq!(state.focused_index(), 2);
}

#[test]
fn test_last_already_at_last() {
    let mut state = BreadcrumbState::from_labels(vec!["A", "B", "C"]);
    state.focused_index = 2;
    let output = Breadcrumb::update(&mut state, BreadcrumbMessage::Last);
    assert_eq!(output, None);
}

#[test]
fn test_navigation_empty() {
    let mut state = BreadcrumbState::default();
    assert_eq!(
        Breadcrumb::update(&mut state, BreadcrumbMessage::Left),
        None
    );
    assert_eq!(
        Breadcrumb::update(&mut state, BreadcrumbMessage::Right),
        None
    );
    assert_eq!(
        Breadcrumb::update(&mut state, BreadcrumbMessage::First),
        None
    );
    assert_eq!(
        Breadcrumb::update(&mut state, BreadcrumbMessage::Last),
        None
    );
}

#[test]
fn test_navigation_returns_focus_changed() {
    let mut state = BreadcrumbState::from_labels(vec!["A", "B"]);
    let output = Breadcrumb::update(&mut state, BreadcrumbMessage::Right);
    assert!(matches!(output, Some(BreadcrumbOutput::FocusChanged(_))));
}

// ==================== Selection Tests ====================

#[test]
fn test_select() {
    let mut state = BreadcrumbState::from_labels(vec!["A", "B", "C"]);
    state.focused_index = 1;
    let output = Breadcrumb::update(&mut state, BreadcrumbMessage::Select);
    assert_eq!(output, Some(BreadcrumbOutput::Selected(1)));
}

#[test]
fn test_select_returns_selected() {
    let mut state = BreadcrumbState::from_labels(vec!["A", "B"]);
    let output = Breadcrumb::update(&mut state, BreadcrumbMessage::Select);
    assert!(matches!(output, Some(BreadcrumbOutput::Selected(_))));
}

#[test]
fn test_select_index() {
    let mut state = BreadcrumbState::from_labels(vec!["A", "B", "C"]);
    let output = Breadcrumb::update(&mut state, BreadcrumbMessage::SelectIndex(2));
    assert_eq!(output, Some(BreadcrumbOutput::Selected(2)));
}

#[test]
fn test_select_index_out_of_bounds() {
    let mut state = BreadcrumbState::from_labels(vec!["A", "B"]);
    let output = Breadcrumb::update(&mut state, BreadcrumbMessage::SelectIndex(5));
    assert_eq!(output, None);
}

#[test]
fn test_select_empty() {
    let mut state = BreadcrumbState::default();
    let output = Breadcrumb::update(&mut state, BreadcrumbMessage::Select);
    assert_eq!(output, None);
}

// ==================== Disabled State Tests ====================

#[test]
fn test_disabled_ignores_messages() {
    let mut state = BreadcrumbState::from_labels(vec!["A", "B", "C"]);
    state.set_disabled(true);

    assert_eq!(
        Breadcrumb::update(&mut state, BreadcrumbMessage::Right),
        None
    );
    assert_eq!(
        Breadcrumb::update(&mut state, BreadcrumbMessage::Left),
        None
    );
    assert_eq!(
        Breadcrumb::update(&mut state, BreadcrumbMessage::Select),
        None
    );
    assert_eq!(state.focused_index(), 0);
}

#[test]
fn test_disabling_preserves_state() {
    let mut state = BreadcrumbState::from_labels(vec!["A", "B", "C"]);
    state.focused_index = 1;
    state.set_disabled(true);
    assert_eq!(state.focused_index(), 1);
    assert_eq!(state.len(), 3);
}

// ==================== Focus Tests ====================

#[test]
fn test_focusable_is_focused() {
    let mut state = BreadcrumbState::default();
    assert!(!Breadcrumb::is_focused(&state));
    state.focused = true;
    assert!(Breadcrumb::is_focused(&state));
}

#[test]
fn test_focusable_set_focused() {
    let mut state = BreadcrumbState::default();
    Breadcrumb::set_focused(&mut state, true);
    assert!(state.focused);
    Breadcrumb::set_focused(&mut state, false);
    assert!(!state.focused);
}

#[test]
fn test_focus_blur() {
    let mut state = BreadcrumbState::default();
    Breadcrumb::focus(&mut state);
    assert!(Breadcrumb::is_focused(&state));
    Breadcrumb::blur(&mut state);
    assert!(!Breadcrumb::is_focused(&state));
}

// ==================== View Tests ====================

#[test]
fn test_view_empty() {
    use crate::backend::CaptureBackend;
    use crate::theme::Theme;
    use ratatui::Terminal;

    let state = BreadcrumbState::default();

    let backend = CaptureBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            Breadcrumb::view(&state, frame, frame.area(), &Theme::default());
        })
        .unwrap();

    let output = terminal.backend().to_string();
    // Empty state renders nothing
    assert!(output.trim().is_empty());
}

#[test]
fn test_view_single() {
    use crate::backend::CaptureBackend;
    use crate::theme::Theme;
    use ratatui::Terminal;

    let state = BreadcrumbState::from_labels(vec!["Home"]);

    let backend = CaptureBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            Breadcrumb::view(&state, frame, frame.area(), &Theme::default());
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("Home"));
}

#[test]
fn test_view_multiple() {
    use crate::backend::CaptureBackend;
    use crate::theme::Theme;
    use ratatui::Terminal;

    let state = BreadcrumbState::from_labels(vec!["Home", "Products", "Item"]);

    let backend = CaptureBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            Breadcrumb::view(&state, frame, frame.area(), &Theme::default());
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("Home"));
    assert!(output.contains(">"));
    assert!(output.contains("Products"));
    assert!(output.contains("Item"));
}

#[test]
fn test_view_focused_highlight() {
    use crate::backend::CaptureBackend;
    use crate::theme::Theme;
    use ratatui::Terminal;

    let mut state = BreadcrumbState::from_labels(vec!["Home", "Products"]);
    state.focused = true;
    state.focused_index = 1;

    let backend = CaptureBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            Breadcrumb::view(&state, frame, frame.area(), &Theme::default());
        })
        .unwrap();

    let output = terminal.backend().to_string();
    // Just verify it renders - style testing would need ANSI output
    assert!(output.contains("Products"));
}

#[test]
fn test_view_truncated() {
    use crate::backend::CaptureBackend;
    use crate::theme::Theme;
    use ratatui::Terminal;

    let mut state =
        BreadcrumbState::from_labels(vec!["Root", "Level1", "Level2", "Level3", "Current"]);
    state.set_max_visible(Some(3));

    let backend = CaptureBackend::new(60, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            Breadcrumb::view(&state, frame, frame.area(), &Theme::default());
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("…")); // Ellipsis for truncation
    assert!(output.contains("Level2"));
    assert!(output.contains("Current"));
    assert!(!output.contains("Root")); // Truncated
}

#[test]
fn test_view_custom_separator() {
    use crate::backend::CaptureBackend;
    use crate::theme::Theme;
    use ratatui::Terminal;

    let mut state = BreadcrumbState::from_labels(vec!["Home", "Docs"]);
    state.set_separator(" / ");

    let backend = CaptureBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            Breadcrumb::view(&state, frame, frame.area(), &Theme::default());
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains(" / "));
}

#[test]
fn test_view_disabled() {
    use crate::backend::CaptureBackend;
    use crate::theme::Theme;
    use ratatui::Terminal;

    let mut state = BreadcrumbState::from_labels(vec!["Home", "Products"]);
    state.set_disabled(true);

    let backend = CaptureBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            Breadcrumb::view(&state, frame, frame.area(), &Theme::default());
        })
        .unwrap();

    let output = terminal.backend().to_string();
    // Just verify it renders - disabled style would need ANSI output
    assert!(output.contains("Home"));
}

// ==================== Integration Tests ====================

#[test]
fn test_clone() {
    let state = BreadcrumbState::from_labels(vec!["A", "B"]);
    let cloned = state.clone();
    assert_eq!(state.len(), cloned.len());
    assert_eq!(state.separator(), cloned.separator());
}

#[test]
fn test_init() {
    let state = Breadcrumb::init();
    assert!(state.is_empty());
    assert_eq!(state.separator(), " > ");
}

#[test]
fn test_full_workflow() {
    let mut state = BreadcrumbState::from_labels(vec!["Home", "Products", "Electronics"]);
    Breadcrumb::set_focused(&mut state, true);

    // Navigate right twice
    Breadcrumb::update(&mut state, BreadcrumbMessage::Right);
    Breadcrumb::update(&mut state, BreadcrumbMessage::Right);
    assert_eq!(state.focused_index(), 2);

    // Select the current segment
    let output = Breadcrumb::update(&mut state, BreadcrumbMessage::Select);
    assert_eq!(output, Some(BreadcrumbOutput::Selected(2)));

    // Navigate back
    Breadcrumb::update(&mut state, BreadcrumbMessage::First);
    assert_eq!(state.focused_index(), 0);

    // Push a new segment
    state.push(BreadcrumbSegment::new("Item"));
    assert_eq!(state.len(), 4);

    // Pop a segment
    state.pop();
    assert_eq!(state.len(), 3);
}
