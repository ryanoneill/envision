use super::*;
use crate::component::test_utils;
use crate::input::KeyCode;

fn sample_groups() -> Vec<KeyBindingGroup> {
    vec![
        KeyBindingGroup::new(
            "Navigation",
            vec![
                KeyBinding::new("Up/k", "Move up"),
                KeyBinding::new("Down/j", "Move down"),
                KeyBinding::new("PgUp", "Page up"),
                KeyBinding::new("PgDn", "Page down"),
            ],
        ),
        KeyBindingGroup::new(
            "Actions",
            vec![
                KeyBinding::new("Enter", "Select item"),
                KeyBinding::new("Space", "Toggle"),
                KeyBinding::new("q/Esc", "Quit"),
            ],
        ),
    ]
}

fn focused_state() -> HelpPanelState {
    HelpPanelState::new()
}

fn focused_state_with_groups() -> HelpPanelState {
    HelpPanelState::new().with_groups(sample_groups())
}

// =============================================================================
// KeyBinding constructors
// =============================================================================

#[test]
fn test_key_binding_new() {
    let binding = KeyBinding::new("Ctrl+S", "Save file");
    assert_eq!(binding.key(), "Ctrl+S");
    assert_eq!(binding.description(), "Save file");
}

#[test]
fn test_key_binding_with_string_types() {
    let binding = KeyBinding::new(String::from("Enter"), String::from("Confirm"));
    assert_eq!(binding.key(), "Enter");
    assert_eq!(binding.description(), "Confirm");
}

// =============================================================================
// KeyBindingGroup constructors
// =============================================================================

#[test]
fn test_key_binding_group_new() {
    let group = KeyBindingGroup::new(
        "Navigation",
        vec![
            KeyBinding::new("Up", "Move up"),
            KeyBinding::new("Down", "Move down"),
        ],
    );
    assert_eq!(group.title(), "Navigation");
    assert_eq!(group.bindings().len(), 2);
}

#[test]
fn test_key_binding_group_empty() {
    let group = KeyBindingGroup::new("Empty", vec![]);
    assert_eq!(group.title(), "Empty");
    assert!(group.bindings().is_empty());
}

// =============================================================================
// Construction
// =============================================================================

#[test]
fn test_new() {
    let state = HelpPanelState::new();
    assert!(state.groups().is_empty());
    assert_eq!(state.title(), Some("Help"));

    assert!(state.is_visible());
}

#[test]
fn test_default() {
    let state = HelpPanelState::default();
    assert!(state.groups().is_empty());
    // Default doesn't set title
    assert_eq!(state.title(), None);
}

#[test]
fn test_with_groups() {
    let state = HelpPanelState::new().with_groups(sample_groups());
    assert_eq!(state.groups().len(), 2);
    assert_eq!(state.groups()[0].title(), "Navigation");
    assert_eq!(state.groups()[1].title(), "Actions");
}

#[test]
fn test_with_title() {
    let state = HelpPanelState::new().with_title("Keybindings");
    // Title is always "Help"
    assert_eq!(state.title(), Some("Help"));
}

#[test]
fn test_init() {
    let state = HelpPanel::init();
    assert!(state.groups().is_empty());
    assert_eq!(state.title(), Some("Help"));
    assert!(state.is_visible());
}

// =============================================================================
// Group management
// =============================================================================

#[test]
fn test_add_group() {
    let mut state = HelpPanelState::new();
    state.add_group(KeyBindingGroup::new(
        "Navigation",
        vec![KeyBinding::new("Up", "Move up")],
    ));
    assert_eq!(state.groups().len(), 1);
    assert_eq!(state.groups()[0].title(), "Navigation");
}

#[test]
fn test_set_groups() {
    let mut state = HelpPanelState::new();
    state.set_groups(sample_groups());
    assert_eq!(state.groups().len(), 2);
}

#[test]
fn test_set_groups_resets_scroll() {
    let mut state = HelpPanelState::new().with_groups(sample_groups());
    // Scroll down
    state.update(HelpPanelMessage::ScrollDown);
    assert!(state.scroll_offset() > 0 || state.total_lines() == 0);
    // Set new groups resets scroll
    state.set_groups(sample_groups());
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_clear() {
    let mut state = HelpPanelState::new().with_groups(sample_groups());
    assert!(!state.groups().is_empty());
    state.clear();
    assert!(state.groups().is_empty());
    assert_eq!(state.total_lines(), 0);
}

#[test]
fn test_total_lines() {
    let state = HelpPanelState::new().with_groups(sample_groups());
    // Navigation: title(1) + separator(1) + bindings(4) + blank(1) = 7
    // Actions: title(1) + separator(1) + bindings(3) = 5
    assert_eq!(state.total_lines(), 12);
}

#[test]
fn test_total_lines_empty() {
    let state = HelpPanelState::new();
    assert_eq!(state.total_lines(), 0);
}

#[test]
fn test_total_lines_single_group() {
    let state = HelpPanelState::new().with_groups(vec![KeyBindingGroup::new(
        "Only",
        vec![KeyBinding::new("a", "do a"), KeyBinding::new("b", "do b")],
    )]);
    // title(1) + separator(1) + bindings(2) = 4 (no trailing blank for last group)
    assert_eq!(state.total_lines(), 4);
}

// =============================================================================
// Scroll operations
// =============================================================================

#[test]
fn test_scroll_down() {
    let mut state = focused_state_with_groups();
    state.update(HelpPanelMessage::ScrollDown);
    assert_eq!(state.scroll_offset(), 1);
}

#[test]
fn test_scroll_up() {
    let mut state = focused_state_with_groups();
    state.update(HelpPanelMessage::ScrollDown);
    state.update(HelpPanelMessage::ScrollDown);
    state.update(HelpPanelMessage::ScrollUp);
    assert_eq!(state.scroll_offset(), 1);
}

#[test]
fn test_scroll_up_at_top() {
    let mut state = focused_state_with_groups();
    state.update(HelpPanelMessage::ScrollUp);
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_page_down() {
    let mut state = focused_state_with_groups();
    state.update(HelpPanelMessage::PageDown(5));
    assert_eq!(state.scroll_offset(), 5);
}

#[test]
fn test_page_up() {
    let mut state = focused_state_with_groups();
    state.update(HelpPanelMessage::PageDown(8));
    state.update(HelpPanelMessage::PageUp(3));
    assert_eq!(state.scroll_offset(), 5);
}

#[test]
fn test_page_up_at_top() {
    let mut state = focused_state_with_groups();
    state.update(HelpPanelMessage::PageUp(10));
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_home() {
    let mut state = focused_state_with_groups();
    state.update(HelpPanelMessage::ScrollDown);
    state.update(HelpPanelMessage::ScrollDown);
    state.update(HelpPanelMessage::Home);
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_home_already_at_top() {
    let mut state = focused_state_with_groups();
    state.update(HelpPanelMessage::Home);
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_end() {
    let mut state = focused_state_with_groups();
    state.update(HelpPanelMessage::End);
    // With 12 total lines and viewport_height 0 (no render), max_offset = 12
    assert_eq!(state.scroll_offset(), 12);
}

// =============================================================================
// SetGroups / AddGroup messages
// =============================================================================

#[test]
fn test_set_groups_message() {
    let mut state = HelpPanelState::new();
    state.update(HelpPanelMessage::SetGroups(sample_groups()));
    assert_eq!(state.groups().len(), 2);
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_add_group_message() {
    let mut state = HelpPanelState::new();
    state.update(HelpPanelMessage::AddGroup(KeyBindingGroup::new(
        "Test",
        vec![KeyBinding::new("t", "test")],
    )));
    assert_eq!(state.groups().len(), 1);
}

// =============================================================================
// Update returns None (display-only)
// =============================================================================

#[test]
fn test_update_returns_none() {
    let mut state = focused_state_with_groups();
    assert_eq!(state.update(HelpPanelMessage::ScrollDown), None);
    assert_eq!(state.update(HelpPanelMessage::ScrollUp), None);
    assert_eq!(state.update(HelpPanelMessage::PageDown(5)), None);
    assert_eq!(state.update(HelpPanelMessage::PageUp(5)), None);
    assert_eq!(state.update(HelpPanelMessage::Home), None);
    assert_eq!(state.update(HelpPanelMessage::End), None);
    assert_eq!(state.update(HelpPanelMessage::SetGroups(vec![])), None);
    assert_eq!(
        state.update(HelpPanelMessage::AddGroup(KeyBindingGroup::new(
            "X",
            vec![]
        ))),
        None
    );
}

// =============================================================================
// Event handling
// =============================================================================

#[test]
fn test_handle_event_up() {
    let state = focused_state();
    assert_eq!(
        HelpPanel::handle_event(
            &state,
            &Event::key(KeyCode::Up),
            &EventContext::new().focused(true)
        ),
        Some(HelpPanelMessage::ScrollUp)
    );
}

#[test]
fn test_handle_event_down() {
    let state = focused_state();
    assert_eq!(
        HelpPanel::handle_event(
            &state,
            &Event::key(KeyCode::Down),
            &EventContext::new().focused(true)
        ),
        Some(HelpPanelMessage::ScrollDown)
    );
}

#[test]
fn test_handle_event_k_j() {
    let state = focused_state();
    assert_eq!(
        HelpPanel::handle_event(
            &state,
            &Event::char('k'),
            &EventContext::new().focused(true)
        ),
        Some(HelpPanelMessage::ScrollUp)
    );
    assert_eq!(
        HelpPanel::handle_event(
            &state,
            &Event::char('j'),
            &EventContext::new().focused(true)
        ),
        Some(HelpPanelMessage::ScrollDown)
    );
}

#[test]
fn test_handle_event_page_up_down() {
    let state = focused_state();
    assert_eq!(
        HelpPanel::handle_event(
            &state,
            &Event::key(KeyCode::PageUp),
            &EventContext::new().focused(true)
        ),
        Some(HelpPanelMessage::PageUp(10))
    );
    assert_eq!(
        HelpPanel::handle_event(
            &state,
            &Event::key(KeyCode::PageDown),
            &EventContext::new().focused(true)
        ),
        Some(HelpPanelMessage::PageDown(10))
    );
}

#[test]
fn test_handle_event_ctrl_u_d() {
    let state = focused_state();
    assert_eq!(
        HelpPanel::handle_event(
            &state,
            &Event::ctrl('u'),
            &EventContext::new().focused(true)
        ),
        Some(HelpPanelMessage::PageUp(10))
    );
    assert_eq!(
        HelpPanel::handle_event(
            &state,
            &Event::ctrl('d'),
            &EventContext::new().focused(true)
        ),
        Some(HelpPanelMessage::PageDown(10))
    );
}

#[test]
fn test_handle_event_home_end() {
    let state = focused_state();
    assert_eq!(
        HelpPanel::handle_event(
            &state,
            &Event::key(KeyCode::Home),
            &EventContext::new().focused(true)
        ),
        Some(HelpPanelMessage::Home)
    );
    assert_eq!(
        HelpPanel::handle_event(
            &state,
            &Event::key(KeyCode::End),
            &EventContext::new().focused(true)
        ),
        Some(HelpPanelMessage::End)
    );
}

#[test]
#[allow(non_snake_case)]
fn test_handle_event_g_and_G() {
    let state = focused_state();
    assert_eq!(
        HelpPanel::handle_event(
            &state,
            &Event::char('g'),
            &EventContext::new().focused(true)
        ),
        Some(HelpPanelMessage::Home)
    );
    assert_eq!(
        HelpPanel::handle_event(
            &state,
            &Event::key_with(KeyCode::Char('G'), KeyModifiers::SHIFT),
            &EventContext::new().focused(true)
        ),
        Some(HelpPanelMessage::End)
    );
}

#[test]
fn test_handle_event_unrecognized() {
    let state = focused_state();
    assert_eq!(
        HelpPanel::handle_event(
            &state,
            &Event::char('x'),
            &EventContext::new().focused(true)
        ),
        None
    );
}

// =============================================================================
// Disabled and unfocused guards
// =============================================================================

#[test]
fn test_disabled_ignores_events() {
    let state = focused_state();
    assert_eq!(
        HelpPanel::handle_event(
            &state,
            &Event::key(KeyCode::Up),
            &EventContext::new().focused(true).disabled(true)
        ),
        None
    );
}

#[test]
fn test_unfocused_ignores_events() {
    let state = HelpPanelState::new();
    assert_eq!(
        HelpPanel::handle_event(&state, &Event::key(KeyCode::Up), &EventContext::default()),
        None
    );
}

// =============================================================================
// Instance methods
// =============================================================================

#[test]
fn test_instance_update() {
    let mut state = focused_state_with_groups();
    let output = state.update(HelpPanelMessage::ScrollDown);
    assert_eq!(output, None);
    assert_eq!(state.scroll_offset(), 1);
}

// =============================================================================
// Toggleable trait
// =============================================================================

#[test]
fn test_toggleable_trait() {
    let mut state = HelpPanel::init();
    assert!(HelpPanel::is_visible(&state));

    HelpPanel::hide(&mut state);
    assert!(!HelpPanel::is_visible(&state));

    HelpPanel::show(&mut state);
    assert!(HelpPanel::is_visible(&state));

    HelpPanel::toggle(&mut state);
    assert!(!HelpPanel::is_visible(&state));

    HelpPanel::toggle(&mut state);
    assert!(HelpPanel::is_visible(&state));
}

// =============================================================================
// State accessors
// =============================================================================

#[test]
fn test_set_title() {
    let mut state = HelpPanelState::new();
    state.set_title(Some("Custom".to_string()));
    assert_eq!(state.title(), Some("Custom"));
    state.set_title(None);
    assert_eq!(state.title(), None);
}

#[test]
fn test_set_visible() {
    let mut state = HelpPanelState::new();
    state.set_visible(false);
    assert!(!state.is_visible());
    state.set_visible(true);
    assert!(state.is_visible());
}

// =============================================================================
// Snapshot tests
// =============================================================================

#[test]
fn test_view_empty() {
    let state = HelpPanelState::new();
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            HelpPanel::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_groups() {
    let state = HelpPanelState::new().with_groups(sample_groups());
    let (mut terminal, theme) = test_utils::setup_render(40, 16);
    terminal
        .draw(|frame| {
            HelpPanel::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_focused() {
    let state = focused_state_with_groups();
    let (mut terminal, theme) = test_utils::setup_render(40, 16);
    terminal
        .draw(|frame| {
            HelpPanel::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_disabled() {
    let state = HelpPanelState::new().with_groups(sample_groups());
    let (mut terminal, theme) = test_utils::setup_render(40, 16);
    terminal
        .draw(|frame| {
            HelpPanel::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme)
                    .focused(true)
                    .disabled(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_scrolled() {
    let mut state = HelpPanelState::new().with_groups(sample_groups());
    // Scroll down a few lines
    state.update(HelpPanelMessage::ScrollDown);
    state.update(HelpPanelMessage::ScrollDown);
    state.update(HelpPanelMessage::ScrollDown);
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            HelpPanel::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
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
    use crate::annotation::{WidgetType, with_annotations};
    let state = HelpPanelState::new().with_groups(sample_groups());
    let (mut terminal, theme) = test_utils::setup_render(40, 16);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                HelpPanel::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.find_by_type(&WidgetType::HelpPanel);
    assert_eq!(regions.len(), 1);
    assert!(!regions[0].annotation.focused);
    assert!(!regions[0].annotation.disabled);
}

#[test]
fn test_annotation_focused() {
    use crate::annotation::{WidgetType, with_annotations};
    let state = focused_state_with_groups();
    let (mut terminal, theme) = test_utils::setup_render(40, 16);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                HelpPanel::view(
                    &state,
                    &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
                );
            })
            .unwrap();
    });
    let regions = registry.find_by_type(&WidgetType::HelpPanel);
    assert_eq!(regions.len(), 1);
    assert!(regions[0].annotation.focused);
}
