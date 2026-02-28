use super::*;

// ========== AccordionPanel Tests ==========

#[test]
fn test_panel_new() {
    let panel = AccordionPanel::new("Title", "Content");
    assert_eq!(panel.title(), "Title");
    assert_eq!(panel.content(), "Content");
    assert!(!panel.is_expanded());
}

#[test]
fn test_panel_expanded_builder() {
    let panel = AccordionPanel::new("Title", "Content").expanded();
    assert!(panel.is_expanded());
}

#[test]
fn test_panel_accessors() {
    let panel = AccordionPanel::new("My Title", "My Content");
    assert_eq!(panel.title(), "My Title");
    assert_eq!(panel.content(), "My Content");
    assert!(!panel.is_expanded());
}

#[test]
fn test_panel_clone() {
    let panel = AccordionPanel::new("Title", "Content").expanded();
    let cloned = panel.clone();
    assert_eq!(cloned.title(), "Title");
    assert!(cloned.is_expanded());
}

// ========== State Creation Tests ==========

#[test]
fn test_new() {
    let panels = vec![
        AccordionPanel::new("A", "Content A"),
        AccordionPanel::new("B", "Content B"),
    ];
    let state = AccordionState::new(panels);
    assert_eq!(state.len(), 2);
    assert_eq!(state.focused_index(), 0);
    assert!(!state.is_disabled());
}

#[test]
fn test_from_pairs() {
    let state = AccordionState::from_pairs(vec![("A", "Content A"), ("B", "Content B")]);
    assert_eq!(state.len(), 2);
    assert_eq!(state.panels()[0].title(), "A");
    assert_eq!(state.panels()[1].content(), "Content B");
}

#[test]
fn test_default() {
    let state = AccordionState::default();
    assert!(state.is_empty());
    assert_eq!(state.len(), 0);
}

#[test]
fn test_new_empty() {
    let state = AccordionState::new(Vec::new());
    assert!(state.is_empty());
    assert_eq!(state.focused_index(), 0);
}

// ========== Accessor Tests ==========

#[test]
fn test_panels() {
    let state = AccordionState::from_pairs(vec![("A", "1"), ("B", "2")]);
    assert_eq!(state.panels().len(), 2);
    assert_eq!(state.panels()[0].title(), "A");
}

#[test]
fn test_len() {
    let state = AccordionState::from_pairs(vec![("A", "1"), ("B", "2"), ("C", "3")]);
    assert_eq!(state.len(), 3);
}

#[test]
fn test_is_empty() {
    let empty = AccordionState::default();
    assert!(empty.is_empty());

    let not_empty = AccordionState::from_pairs(vec![("A", "1")]);
    assert!(!not_empty.is_empty());
}

#[test]
fn test_focused_index() {
    let state = AccordionState::from_pairs(vec![("A", "1"), ("B", "2")]);
    assert_eq!(state.focused_index(), 0);
}

#[test]
fn test_focused_panel() {
    let state = AccordionState::from_pairs(vec![("A", "1"), ("B", "2")]);
    assert_eq!(state.focused_panel().unwrap().title(), "A");

    let empty = AccordionState::default();
    assert!(empty.focused_panel().is_none());
}

#[test]
fn test_is_disabled() {
    let mut state = AccordionState::default();
    assert!(!state.is_disabled());
    state.set_disabled(true);
    assert!(state.is_disabled());
}

// ========== Mutator Tests ==========

#[test]
fn test_set_panels() {
    let mut state = AccordionState::from_pairs(vec![("A", "1")]);
    state.set_panels(vec![
        AccordionPanel::new("X", "10"),
        AccordionPanel::new("Y", "20"),
    ]);
    assert_eq!(state.len(), 2);
    assert_eq!(state.panels()[0].title(), "X");
}

#[test]
fn test_add_panel() {
    let mut state = AccordionState::from_pairs(vec![("A", "1")]);
    state.add_panel(AccordionPanel::new("B", "2"));
    assert_eq!(state.len(), 2);
    assert_eq!(state.panels()[1].title(), "B");
}

#[test]
fn test_set_disabled() {
    let mut state = AccordionState::default();
    state.set_disabled(true);
    assert!(state.is_disabled());
    state.set_disabled(false);
    assert!(!state.is_disabled());
}

// ========== Query Method Tests ==========

#[test]
fn test_expanded_count() {
    let panels = vec![
        AccordionPanel::new("A", "1").expanded(),
        AccordionPanel::new("B", "2"),
        AccordionPanel::new("C", "3").expanded(),
    ];
    let state = AccordionState::new(panels);
    assert_eq!(state.expanded_count(), 2);
}

#[test]
fn test_is_any_expanded() {
    let none_expanded = AccordionState::from_pairs(vec![("A", "1"), ("B", "2")]);
    assert!(!none_expanded.is_any_expanded());

    let some_expanded = AccordionState::new(vec![
        AccordionPanel::new("A", "1"),
        AccordionPanel::new("B", "2").expanded(),
    ]);
    assert!(some_expanded.is_any_expanded());
}

#[test]
fn test_is_all_expanded() {
    let all_expanded = AccordionState::new(vec![
        AccordionPanel::new("A", "1").expanded(),
        AccordionPanel::new("B", "2").expanded(),
    ]);
    assert!(all_expanded.is_all_expanded());

    let partial = AccordionState::new(vec![
        AccordionPanel::new("A", "1").expanded(),
        AccordionPanel::new("B", "2"),
    ]);
    assert!(!partial.is_all_expanded());

    let empty = AccordionState::default();
    assert!(!empty.is_all_expanded());
}

// ========== Navigation Tests ==========

#[test]
fn test_next() {
    let mut state = AccordionState::from_pairs(vec![("A", "1"), ("B", "2"), ("C", "3")]);
    assert_eq!(state.focused_index(), 0);

    Accordion::update(&mut state, AccordionMessage::Next);
    assert_eq!(state.focused_index(), 1);

    Accordion::update(&mut state, AccordionMessage::Next);
    assert_eq!(state.focused_index(), 2);
}

#[test]
fn test_previous() {
    let mut state = AccordionState::from_pairs(vec![("A", "1"), ("B", "2"), ("C", "3")]);
    Accordion::update(&mut state, AccordionMessage::Next);
    Accordion::update(&mut state, AccordionMessage::Next);
    assert_eq!(state.focused_index(), 2);

    Accordion::update(&mut state, AccordionMessage::Previous);
    assert_eq!(state.focused_index(), 1);

    Accordion::update(&mut state, AccordionMessage::Previous);
    assert_eq!(state.focused_index(), 0);
}

#[test]
fn test_next_wraps() {
    let mut state = AccordionState::from_pairs(vec![("A", "1"), ("B", "2")]);
    Accordion::update(&mut state, AccordionMessage::Next);
    assert_eq!(state.focused_index(), 1);

    Accordion::update(&mut state, AccordionMessage::Next);
    assert_eq!(state.focused_index(), 0); // Wrapped
}

#[test]
fn test_previous_wraps() {
    let mut state = AccordionState::from_pairs(vec![("A", "1"), ("B", "2")]);
    assert_eq!(state.focused_index(), 0);

    Accordion::update(&mut state, AccordionMessage::Previous);
    assert_eq!(state.focused_index(), 1); // Wrapped to end
}

#[test]
fn test_first() {
    let mut state = AccordionState::from_pairs(vec![("A", "1"), ("B", "2"), ("C", "3")]);
    Accordion::update(&mut state, AccordionMessage::Next);
    Accordion::update(&mut state, AccordionMessage::Next);
    assert_eq!(state.focused_index(), 2);

    let output = Accordion::update(&mut state, AccordionMessage::First);
    assert_eq!(state.focused_index(), 0);
    assert_eq!(output, Some(AccordionOutput::FocusChanged(0)));
}

#[test]
fn test_last() {
    let mut state = AccordionState::from_pairs(vec![("A", "1"), ("B", "2"), ("C", "3")]);
    assert_eq!(state.focused_index(), 0);

    let output = Accordion::update(&mut state, AccordionMessage::Last);
    assert_eq!(state.focused_index(), 2);
    assert_eq!(output, Some(AccordionOutput::FocusChanged(2)));
}

#[test]
fn test_navigation_empty() {
    let mut state = AccordionState::default();

    let output = Accordion::update(&mut state, AccordionMessage::Next);
    assert_eq!(output, None);

    let output = Accordion::update(&mut state, AccordionMessage::Previous);
    assert_eq!(output, None);

    let output = Accordion::update(&mut state, AccordionMessage::First);
    assert_eq!(output, None);

    let output = Accordion::update(&mut state, AccordionMessage::Last);
    assert_eq!(output, None);
}

#[test]
fn test_navigation_returns_focus_changed() {
    let mut state = AccordionState::from_pairs(vec![("A", "1"), ("B", "2")]);

    let output = Accordion::update(&mut state, AccordionMessage::Next);
    assert_eq!(output, Some(AccordionOutput::FocusChanged(1)));
}

// ========== Toggle/Expand/Collapse Tests ==========

#[test]
fn test_toggle() {
    let mut state = AccordionState::from_pairs(vec![("A", "1")]);
    assert!(!state.panels()[0].is_expanded());

    Accordion::update(&mut state, AccordionMessage::Toggle);
    assert!(state.panels()[0].is_expanded());

    Accordion::update(&mut state, AccordionMessage::Toggle);
    assert!(!state.panels()[0].is_expanded());
}

#[test]
fn test_toggle_returns_expanded() {
    let mut state = AccordionState::from_pairs(vec![("A", "1")]);
    let output = Accordion::update(&mut state, AccordionMessage::Toggle);
    assert_eq!(output, Some(AccordionOutput::Expanded(0)));
}

#[test]
fn test_toggle_returns_collapsed() {
    let mut state = AccordionState::new(vec![AccordionPanel::new("A", "1").expanded()]);
    let output = Accordion::update(&mut state, AccordionMessage::Toggle);
    assert_eq!(output, Some(AccordionOutput::Collapsed(0)));
}

#[test]
fn test_expand() {
    let mut state = AccordionState::from_pairs(vec![("A", "1")]);
    let output = Accordion::update(&mut state, AccordionMessage::Expand);
    assert_eq!(output, Some(AccordionOutput::Expanded(0)));
    assert!(state.panels()[0].is_expanded());
}

#[test]
fn test_expand_already_expanded() {
    let mut state = AccordionState::new(vec![AccordionPanel::new("A", "1").expanded()]);
    let output = Accordion::update(&mut state, AccordionMessage::Expand);
    assert_eq!(output, None);
}

#[test]
fn test_collapse() {
    let mut state = AccordionState::new(vec![AccordionPanel::new("A", "1").expanded()]);
    let output = Accordion::update(&mut state, AccordionMessage::Collapse);
    assert_eq!(output, Some(AccordionOutput::Collapsed(0)));
    assert!(!state.panels()[0].is_expanded());
}

#[test]
fn test_collapse_already_collapsed() {
    let mut state = AccordionState::from_pairs(vec![("A", "1")]);
    let output = Accordion::update(&mut state, AccordionMessage::Collapse);
    assert_eq!(output, None);
}

#[test]
fn test_toggle_index() {
    let mut state = AccordionState::from_pairs(vec![("A", "1"), ("B", "2")]);

    let output = Accordion::update(&mut state, AccordionMessage::ToggleIndex(1));
    assert_eq!(output, Some(AccordionOutput::Expanded(1)));
    assert!(state.panels()[1].is_expanded());

    let output = Accordion::update(&mut state, AccordionMessage::ToggleIndex(1));
    assert_eq!(output, Some(AccordionOutput::Collapsed(1)));
    assert!(!state.panels()[1].is_expanded());
}

#[test]
fn test_toggle_index_out_of_bounds() {
    let mut state = AccordionState::from_pairs(vec![("A", "1")]);
    let output = Accordion::update(&mut state, AccordionMessage::ToggleIndex(5));
    assert_eq!(output, None);
}

// ========== Bulk Operations Tests ==========

#[test]
fn test_expand_all() {
    let mut state = AccordionState::from_pairs(vec![("A", "1"), ("B", "2"), ("C", "3")]);
    assert_eq!(state.expanded_count(), 0);

    let output = Accordion::update(&mut state, AccordionMessage::ExpandAll);
    assert!(output.is_some());
    assert_eq!(state.expanded_count(), 3);
    assert!(state.is_all_expanded());
}

#[test]
fn test_collapse_all() {
    let mut state = AccordionState::new(vec![
        AccordionPanel::new("A", "1").expanded(),
        AccordionPanel::new("B", "2").expanded(),
    ]);
    assert_eq!(state.expanded_count(), 2);

    let output = Accordion::update(&mut state, AccordionMessage::CollapseAll);
    assert!(output.is_some());
    assert_eq!(state.expanded_count(), 0);
    assert!(!state.is_any_expanded());
}

#[test]
fn test_expand_all_already_expanded() {
    let mut state = AccordionState::new(vec![
        AccordionPanel::new("A", "1").expanded(),
        AccordionPanel::new("B", "2").expanded(),
    ]);
    let output = Accordion::update(&mut state, AccordionMessage::ExpandAll);
    assert_eq!(output, None);
}

#[test]
fn test_collapse_all_already_collapsed() {
    let mut state = AccordionState::from_pairs(vec![("A", "1"), ("B", "2")]);
    let output = Accordion::update(&mut state, AccordionMessage::CollapseAll);
    assert_eq!(output, None);
}

// ========== Disabled State Tests ==========

#[test]
fn test_disabled_ignores_messages() {
    let mut state = AccordionState::from_pairs(vec![("A", "1"), ("B", "2")]);
    state.set_disabled(true);

    let output = Accordion::update(&mut state, AccordionMessage::Toggle);
    assert_eq!(output, None);
    assert!(!state.panels()[0].is_expanded());

    let output = Accordion::update(&mut state, AccordionMessage::Next);
    assert_eq!(output, None);
    assert_eq!(state.focused_index(), 0);
}

#[test]
fn test_disabling_preserves_state() {
    let mut state = AccordionState::new(vec![AccordionPanel::new("A", "1").expanded()]);
    assert!(state.panels()[0].is_expanded());

    state.set_disabled(true);
    assert!(state.panels()[0].is_expanded()); // Still expanded
}

// ========== Focus Tests ==========

#[test]
fn test_focusable_is_focused() {
    let state = AccordionState::default();
    assert!(!Accordion::is_focused(&state));
}

#[test]
fn test_focusable_set_focused() {
    let mut state = AccordionState::default();
    Accordion::set_focused(&mut state, true);
    assert!(Accordion::is_focused(&state));
}

#[test]
fn test_focus_blur() {
    let mut state = AccordionState::default();

    Accordion::focus(&mut state);
    assert!(Accordion::is_focused(&state));

    Accordion::blur(&mut state);
    assert!(!Accordion::is_focused(&state));
}

// ========== View Tests ==========

#[test]
fn test_view_empty() {
    use crate::backend::CaptureBackend;
    use ratatui::Terminal;

    let state = AccordionState::default();
    let theme = Theme::default();

    let backend = CaptureBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            Accordion::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    // Should render without error
    let _ = terminal.backend().to_string();
}

#[test]
fn test_view_collapsed() {
    use crate::backend::CaptureBackend;
    use ratatui::Terminal;

    let state = AccordionState::from_pairs(vec![("Section 1", "Content 1")]);
    let theme = Theme::default();

    let backend = CaptureBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            Accordion::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("▶")); // Collapsed indicator
    assert!(output.contains("Section 1"));
}

#[test]
fn test_view_expanded() {
    use crate::backend::CaptureBackend;
    use ratatui::Terminal;

    let state = AccordionState::new(vec![
        AccordionPanel::new("Section 1", "Content 1").expanded()
    ]);
    let theme = Theme::default();

    let backend = CaptureBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            Accordion::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("▼")); // Expanded indicator
    assert!(output.contains("Section 1"));
    assert!(output.contains("Content 1"));
}

#[test]
fn test_view_mixed() {
    use crate::backend::CaptureBackend;
    use ratatui::Terminal;

    let state = AccordionState::new(vec![
        AccordionPanel::new("Expanded", "Expanded content").expanded(),
        AccordionPanel::new("Collapsed", "Collapsed content"),
    ]);
    let theme = Theme::default();

    let backend = CaptureBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            Accordion::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("Expanded"));
    assert!(output.contains("Collapsed"));
    assert!(output.contains("Expanded content"));
}

#[test]
fn test_view_focused_highlight() {
    use crate::backend::CaptureBackend;
    use ratatui::Terminal;

    let mut state = AccordionState::from_pairs(vec![("A", "1"), ("B", "2")]);
    Accordion::focus(&mut state);
    let theme = Theme::default();

    let backend = CaptureBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            Accordion::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    // Should render without error (we can't easily check color in text)
    let output = terminal.backend().to_string();
    assert!(output.contains("A"));
}

#[test]
fn test_view_long_content() {
    use crate::backend::CaptureBackend;
    use ratatui::Terminal;

    let state = AccordionState::new(vec![AccordionPanel::new(
        "Multi-line",
        "Line 1\nLine 2\nLine 3",
    )
    .expanded()]);
    let theme = Theme::default();

    let backend = CaptureBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            Accordion::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("Multi-line"));
    assert!(output.contains("Line 1"));
}

// ========== Integration Tests ==========

#[test]
fn test_clone() {
    let state = AccordionState::new(vec![AccordionPanel::new("A", "1").expanded()]);
    let cloned = state.clone();
    assert_eq!(cloned.len(), 1);
    assert!(cloned.panels()[0].is_expanded());
}

#[test]
fn test_init() {
    let state = Accordion::init();
    assert!(state.is_empty());
    assert!(!Accordion::is_focused(&state));
}

#[test]
fn test_full_workflow() {
    let mut state = AccordionState::from_pairs(vec![
        ("Section 1", "Content 1"),
        ("Section 2", "Content 2"),
        ("Section 3", "Content 3"),
    ]);
    Accordion::focus(&mut state);

    // Initially no panels expanded
    assert_eq!(state.expanded_count(), 0);

    // Toggle first panel
    let output = Accordion::update(&mut state, AccordionMessage::Toggle);
    assert_eq!(output, Some(AccordionOutput::Expanded(0)));
    assert_eq!(state.expanded_count(), 1);

    // Navigate to next and toggle
    Accordion::update(&mut state, AccordionMessage::Next);
    assert_eq!(state.focused_index(), 1);
    Accordion::update(&mut state, AccordionMessage::Toggle);
    assert_eq!(state.expanded_count(), 2);

    // Both panels 0 and 1 are expanded (multi-expand)
    assert!(state.panels()[0].is_expanded());
    assert!(state.panels()[1].is_expanded());
    assert!(!state.panels()[2].is_expanded());

    // Collapse all
    Accordion::update(&mut state, AccordionMessage::CollapseAll);
    assert_eq!(state.expanded_count(), 0);

    // Expand all
    Accordion::update(&mut state, AccordionMessage::ExpandAll);
    assert!(state.is_all_expanded());
}
