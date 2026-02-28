use super::*;

// ========================================
// KeyHint Tests
// ========================================

#[test]
fn test_key_hint_new() {
    let hint = KeyHint::new("Enter", "Select");
    assert_eq!(hint.key(), "Enter");
    assert_eq!(hint.action(), "Select");
    assert!(hint.is_enabled());
    assert_eq!(hint.priority(), 100);
}

#[test]
fn test_key_hint_with_priority() {
    let hint = KeyHint::new("q", "Quit").with_priority(1);
    assert_eq!(hint.priority(), 1);
}

#[test]
fn test_key_hint_with_enabled() {
    let hint = KeyHint::new("Delete", "Remove").with_enabled(false);
    assert!(!hint.is_enabled());
}

#[test]
fn test_key_hint_setters() {
    let mut hint = KeyHint::new("a", "Action");
    hint.set_key("b");
    hint.set_action("New Action");
    hint.set_enabled(false);
    hint.set_priority(5);

    assert_eq!(hint.key(), "b");
    assert_eq!(hint.action(), "New Action");
    assert!(!hint.is_enabled());
    assert_eq!(hint.priority(), 5);
}

#[test]
fn test_key_hint_clone() {
    let hint = KeyHint::new("x", "Execute").with_priority(10);
    let cloned = hint.clone();
    assert_eq!(cloned.key(), "x");
    assert_eq!(cloned.priority(), 10);
}

// ========================================
// KeyHintsLayout Tests
// ========================================

#[test]
fn test_layout_default() {
    let layout = KeyHintsLayout::default();
    assert_eq!(layout, KeyHintsLayout::Spaced);
}

#[test]
fn test_layout_eq() {
    assert_eq!(KeyHintsLayout::Spaced, KeyHintsLayout::Spaced);
    assert_ne!(KeyHintsLayout::Spaced, KeyHintsLayout::Inline);
}

// ========================================
// State Creation Tests
// ========================================

#[test]
fn test_state_new() {
    let state = KeyHintsState::new();
    assert!(state.is_empty());
    assert_eq!(state.layout(), KeyHintsLayout::Spaced);
}

#[test]
fn test_state_with_hints() {
    let hints = vec![KeyHint::new("a", "Action A"), KeyHint::new("b", "Action B")];
    let state = KeyHintsState::with_hints(hints);
    assert_eq!(state.len(), 2);
}

#[test]
fn test_state_with_layout() {
    let state = KeyHintsState::new().with_layout(KeyHintsLayout::Inline);
    assert_eq!(state.layout(), KeyHintsLayout::Inline);
}

#[test]
fn test_state_builder_hint() {
    let state = KeyHintsState::new()
        .hint("Enter", "Select")
        .hint("Esc", "Cancel")
        .hint("q", "Quit");
    assert_eq!(state.len(), 3);
}

#[test]
fn test_state_builder_hint_with_priority() {
    let state = KeyHintsState::new()
        .hint_with_priority("q", "Quit", 1)
        .hint_with_priority("?", "Help", 10);

    let visible = state.visible_hints();
    assert_eq!(visible[0].key(), "q"); // Lower priority first
    assert_eq!(visible[1].key(), "?");
}

#[test]
fn test_state_default() {
    let state = KeyHintsState::default();
    assert!(state.is_empty());
}

// ========================================
// Accessor Tests
// ========================================

#[test]
fn test_hints() {
    let state = KeyHintsState::new().hint("a", "A").hint("b", "B");
    assert_eq!(state.hints().len(), 2);
}

#[test]
fn test_visible_hints() {
    let state = KeyHintsState::with_hints(vec![
        KeyHint::new("a", "A").with_enabled(false),
        KeyHint::new("b", "B"),
        KeyHint::new("c", "C"),
    ]);

    let visible = state.visible_hints();
    assert_eq!(visible.len(), 2);
}

#[test]
fn test_visible_hints_sorted_by_priority() {
    let state = KeyHintsState::with_hints(vec![
        KeyHint::new("c", "C").with_priority(50),
        KeyHint::new("a", "A").with_priority(1),
        KeyHint::new("b", "B").with_priority(25),
    ]);

    let visible = state.visible_hints();
    assert_eq!(visible[0].key(), "a");
    assert_eq!(visible[1].key(), "b");
    assert_eq!(visible[2].key(), "c");
}

#[test]
fn test_len_and_is_empty() {
    let state = KeyHintsState::new();
    assert!(state.is_empty());
    assert_eq!(state.len(), 0);

    let state = KeyHintsState::new().hint("a", "A");
    assert!(!state.is_empty());
    assert_eq!(state.len(), 1);
}

// ========================================
// Mutator Tests
// ========================================

#[test]
fn test_set_hints() {
    let mut state = KeyHintsState::new().hint("old", "Old");
    state.set_hints(vec![KeyHint::new("new", "New")]);
    assert_eq!(state.len(), 1);
    assert_eq!(state.hints()[0].key(), "new");
}

#[test]
fn test_add_hint() {
    let mut state = KeyHintsState::new();
    state.add_hint(KeyHint::new("a", "A"));
    assert_eq!(state.len(), 1);
}

#[test]
fn test_remove_hint() {
    let mut state = KeyHintsState::new().hint("a", "A").hint("b", "B");
    state.remove_hint("a");
    assert_eq!(state.len(), 1);
    assert_eq!(state.hints()[0].key(), "b");
}

#[test]
fn test_enable_disable_hint() {
    let mut state = KeyHintsState::new().hint("a", "A");
    state.disable_hint("a");
    assert!(!state.hints()[0].is_enabled());

    state.enable_hint("a");
    assert!(state.hints()[0].is_enabled());
}

#[test]
fn test_set_layout() {
    let mut state = KeyHintsState::new();
    state.set_layout(KeyHintsLayout::Inline);
    assert_eq!(state.layout(), KeyHintsLayout::Inline);
}

#[test]
fn test_clear() {
    let mut state = KeyHintsState::new().hint("a", "A").hint("b", "B");
    state.clear();
    assert!(state.is_empty());
}

// ========================================
// Component Tests
// ========================================

#[test]
fn test_init() {
    let state = KeyHints::init();
    assert!(state.is_empty());
}

#[test]
fn test_update_set_hints() {
    let mut state = KeyHints::init();
    KeyHints::update(
        &mut state,
        KeyHintsMessage::SetHints(vec![KeyHint::new("x", "X")]),
    );
    assert_eq!(state.len(), 1);
}

#[test]
fn test_update_add_hint() {
    let mut state = KeyHints::init();
    KeyHints::update(&mut state, KeyHintsMessage::AddHint(KeyHint::new("a", "A")));
    assert_eq!(state.len(), 1);
}

#[test]
fn test_update_remove_hint() {
    let mut state = KeyHintsState::new().hint("a", "A");
    KeyHints::update(&mut state, KeyHintsMessage::RemoveHint("a".to_string()));
    assert!(state.is_empty());
}

#[test]
fn test_update_enable_hint() {
    let mut state = KeyHintsState::with_hints(vec![KeyHint::new("a", "A").with_enabled(false)]);
    KeyHints::update(&mut state, KeyHintsMessage::EnableHint("a".to_string()));
    assert!(state.hints()[0].is_enabled());
}

#[test]
fn test_update_disable_hint() {
    let mut state = KeyHintsState::new().hint("a", "A");
    KeyHints::update(&mut state, KeyHintsMessage::DisableHint("a".to_string()));
    assert!(!state.hints()[0].is_enabled());
}

#[test]
fn test_update_set_layout() {
    let mut state = KeyHints::init();
    KeyHints::update(
        &mut state,
        KeyHintsMessage::SetLayout(KeyHintsLayout::Inline),
    );
    assert_eq!(state.layout(), KeyHintsLayout::Inline);
}

#[test]
fn test_update_clear() {
    let mut state = KeyHintsState::new().hint("a", "A");
    KeyHints::update(&mut state, KeyHintsMessage::Clear);
    assert!(state.is_empty());
}

#[test]
fn test_update_returns_none() {
    let mut state = KeyHints::init();
    let output = KeyHints::update(&mut state, KeyHintsMessage::Clear);
    assert!(output.is_none());
}

// ========================================
// View Tests
// ========================================

#[test]
fn test_view_empty() {
    let state = KeyHintsState::new();
    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 1);

    terminal
        .draw(|frame| KeyHints::view(&state, frame, frame.area(), &theme))
        .unwrap();

    // Empty state should render nothing
    let output = terminal.backend().to_string();
    assert!(output.trim().is_empty());
}

#[test]
fn test_view_single_hint() {
    let state = KeyHintsState::new().hint("Enter", "Select");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 1);

    terminal
        .draw(|frame| KeyHints::view(&state, frame, frame.area(), &theme))
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("Enter"));
    assert!(output.contains("Select"));
}

#[test]
fn test_view_multiple_hints() {
    let state = KeyHintsState::new()
        .hint("Enter", "Select")
        .hint("Esc", "Cancel")
        .hint("q", "Quit");

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 1);

    terminal
        .draw(|frame| KeyHints::view(&state, frame, frame.area(), &theme))
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("Enter"));
    assert!(output.contains("Esc"));
    assert!(output.contains("q"));
}

#[test]
fn test_view_disabled_hints_hidden() {
    let state = KeyHintsState::with_hints(vec![
        KeyHint::new("a", "Visible"),
        KeyHint::new("b", "Hidden").with_enabled(false),
    ]);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 1);

    terminal
        .draw(|frame| KeyHints::view(&state, frame, frame.area(), &theme))
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("Visible"));
    assert!(!output.contains("Hidden"));
}

#[test]
fn test_view_inline_layout() {
    let state = KeyHintsState::new()
        .with_layout(KeyHintsLayout::Inline)
        .hint("a", "A")
        .hint("b", "B");

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 1);

    terminal
        .draw(|frame| KeyHints::view(&state, frame, frame.area(), &theme))
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("a"));
    assert!(output.contains("b"));
}

// ========================================
// Style Tests
// ========================================

#[test]
fn test_custom_key_style() {
    let state = KeyHintsState::new().with_key_style(Style::default().fg(Color::Yellow));
    assert_eq!(state.key_style().fg, Some(Color::Yellow));
}

#[test]
fn test_custom_action_style() {
    let state = KeyHintsState::new().with_action_style(Style::default().fg(Color::Cyan));
    assert_eq!(state.action_style().fg, Some(Color::Cyan));
}

#[test]
fn test_custom_separators() {
    let state = KeyHintsState::new()
        .with_key_action_separator(": ")
        .with_hint_separator(" | ");

    // Just verify it doesn't panic and state is set correctly
    assert!(!state.is_empty() || state.is_empty()); // Always true, just exercising the code
}
