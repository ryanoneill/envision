use super::*;
use crate::component::test_utils;

fn sample_items() -> Vec<PaletteItem> {
    vec![
        PaletteItem::new("open", "Open File").with_shortcut("Ctrl+O"),
        PaletteItem::new("save", "Save File").with_shortcut("Ctrl+S"),
        PaletteItem::new("close", "Close Tab").with_shortcut("Ctrl+W"),
        PaletteItem::new("settings", "Open Settings"),
        PaletteItem::new("quit", "Quit Application").with_shortcut("Ctrl+Q"),
    ]
}

fn active_state() -> CommandPaletteState {
    let mut state = CommandPaletteState::new(sample_items());
    state.set_focused(true);
    state.set_visible(true);
    state
}

// =============================================================================
// Construction and defaults
// =============================================================================

#[test]
fn test_new_creates_state_with_all_items() {
    let state = CommandPaletteState::new(sample_items());
    assert_eq!(state.items().len(), 5);
    assert_eq!(state.filtered_items().len(), 5);
    assert_eq!(state.filtered_count(), 5);
}

#[test]
fn test_new_selects_first_item() {
    let state = CommandPaletteState::new(sample_items());
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(state.selected_item().map(|i| i.id.as_str()), Some("open"));
}

#[test]
fn test_new_empty_has_no_selection() {
    let state = CommandPaletteState::new(vec![]);
    assert_eq!(state.selected_index(), None);
    assert_eq!(state.selected_item(), None);
    assert_eq!(state.items().len(), 0);
}

#[test]
fn test_default_state() {
    let state = CommandPaletteState::default();
    assert!(state.items().is_empty());
    assert!(state.filtered_items().is_empty());
    assert_eq!(state.query(), "");
    assert!(!state.is_focused());
    assert!(!state.is_disabled());
    assert!(!state.is_visible());
    assert_eq!(state.title(), Some("Command Palette"));
    assert_eq!(state.placeholder(), "Type to search...");
    assert_eq!(state.max_visible(), 10);
}

#[test]
fn test_new_starts_hidden() {
    let state = CommandPaletteState::new(sample_items());
    assert!(!state.is_visible());
}

// =============================================================================
// Builder methods
// =============================================================================

#[test]
fn test_with_title() {
    let state = CommandPaletteState::new(vec![]).with_title("Actions");
    assert_eq!(state.title(), Some("Actions"));
}

#[test]
fn test_with_placeholder() {
    let state = CommandPaletteState::new(vec![]).with_placeholder("Find a command...");
    assert_eq!(state.placeholder(), "Find a command...");
}

#[test]
fn test_with_max_visible() {
    let state = CommandPaletteState::new(vec![]).with_max_visible(5);
    assert_eq!(state.max_visible(), 5);
}

#[test]
fn test_with_visible() {
    let state = CommandPaletteState::new(vec![]).with_visible(true);
    assert!(state.is_visible());
}

#[test]
fn test_with_disabled() {
    let state = CommandPaletteState::new(vec![]).with_disabled(true);
    assert!(state.is_disabled());
}

// =============================================================================
// PaletteItem
// =============================================================================

#[test]
fn test_palette_item_new() {
    let item = PaletteItem::new("test", "Test Item");
    assert_eq!(item.id, "test");
    assert_eq!(item.label, "Test Item");
    assert!(item.description.is_none());
    assert!(item.shortcut.is_none());
    assert!(item.category.is_none());
}

#[test]
fn test_palette_item_with_description() {
    let item = PaletteItem::new("test", "Test").with_description("A test item");
    assert_eq!(item.description.as_deref(), Some("A test item"));
}

#[test]
fn test_palette_item_with_shortcut() {
    let item = PaletteItem::new("test", "Test").with_shortcut("Ctrl+T");
    assert_eq!(item.shortcut.as_deref(), Some("Ctrl+T"));
}

#[test]
fn test_palette_item_with_category() {
    let item = PaletteItem::new("test", "Test").with_category("File");
    assert_eq!(item.category.as_deref(), Some("File"));
}

#[test]
fn test_palette_item_builder_chain() {
    let item = PaletteItem::new("test", "Test")
        .with_description("desc")
        .with_shortcut("Ctrl+T")
        .with_category("Cat");
    assert_eq!(item.description.as_deref(), Some("desc"));
    assert_eq!(item.shortcut.as_deref(), Some("Ctrl+T"));
    assert_eq!(item.category.as_deref(), Some("Cat"));
}

// =============================================================================
// Fuzzy matching
// =============================================================================

#[test]
fn test_fuzzy_empty_query_matches_everything() {
    assert_eq!(fuzzy_score("", "anything"), Some(0));
}

#[test]
fn test_fuzzy_exact_prefix_match() {
    let score = fuzzy_score("open", "Open File");
    assert!(score.is_some());
    assert!(score.unwrap() >= 1000); // prefix match tier
}

#[test]
fn test_fuzzy_prefix_shorter_text_scores_lower_difference() {
    let long_score = fuzzy_score("open", "Open File Browser");
    let short_score = fuzzy_score("open", "Open File");
    // Both are prefix matches, different text lengths
    assert!(long_score.is_some());
    assert!(short_score.is_some());
}

#[test]
fn test_fuzzy_substring_match() {
    let score = fuzzy_score("file", "Open File");
    assert!(score.is_some());
    assert_eq!(score.unwrap(), 500); // substring tier
}

#[test]
fn test_fuzzy_case_insensitive() {
    let score1 = fuzzy_score("OPEN", "open file");
    let score2 = fuzzy_score("open", "Open File");
    assert!(score1.is_some());
    assert!(score2.is_some());
}

#[test]
fn test_fuzzy_chars_in_order() {
    let score = fuzzy_score("ofl", "Open File");
    assert!(score.is_some());
    // Should be a fuzzy match, not prefix or substring
    assert!(score.unwrap() < 500);
}

#[test]
fn test_fuzzy_no_match() {
    assert!(fuzzy_score("xyz", "Open File").is_none());
}

#[test]
fn test_fuzzy_out_of_order_no_match() {
    assert!(fuzzy_score("feo", "Open File").is_none());
}

#[test]
fn test_fuzzy_prefix_scores_higher_than_substring() {
    let prefix = fuzzy_score("open", "Open File").unwrap();
    let substring = fuzzy_score("file", "Open File").unwrap();
    assert!(prefix > substring);
}

#[test]
fn test_fuzzy_substring_scores_higher_than_fuzzy() {
    let substring = fuzzy_score("file", "Open File").unwrap();
    let fuzzy = fuzzy_score("ofl", "Open File").unwrap();
    assert!(substring > fuzzy);
}

// =============================================================================
// Query operations
// =============================================================================

#[test]
fn test_type_char_appends_to_query() {
    let mut state = active_state();
    CommandPalette::update(&mut state, CommandPaletteMessage::TypeChar('o'));
    assert_eq!(state.query(), "o");
    CommandPalette::update(&mut state, CommandPaletteMessage::TypeChar('p'));
    assert_eq!(state.query(), "op");
}

#[test]
fn test_type_char_returns_query_changed() {
    let mut state = active_state();
    let output = CommandPalette::update(&mut state, CommandPaletteMessage::TypeChar('x'));
    assert_eq!(
        output,
        Some(CommandPaletteOutput::QueryChanged("x".to_string()))
    );
}

#[test]
fn test_type_char_refilters() {
    let mut state = active_state();
    CommandPalette::update(&mut state, CommandPaletteMessage::TypeChar('q'));
    // Only "Quit Application" should match (prefix "q")
    assert_eq!(state.filtered_count(), 1);
    assert_eq!(state.filtered_items()[0].id, "quit");
}

#[test]
fn test_backspace_removes_last_char() {
    let mut state = active_state();
    CommandPalette::update(&mut state, CommandPaletteMessage::TypeChar('a'));
    CommandPalette::update(&mut state, CommandPaletteMessage::TypeChar('b'));
    CommandPalette::update(&mut state, CommandPaletteMessage::Backspace);
    assert_eq!(state.query(), "a");
}

#[test]
fn test_backspace_on_empty_returns_none() {
    let mut state = active_state();
    let output = CommandPalette::update(&mut state, CommandPaletteMessage::Backspace);
    assert_eq!(output, None);
}

#[test]
fn test_backspace_returns_query_changed() {
    let mut state = active_state();
    CommandPalette::update(&mut state, CommandPaletteMessage::TypeChar('a'));
    let output = CommandPalette::update(&mut state, CommandPaletteMessage::Backspace);
    assert_eq!(
        output,
        Some(CommandPaletteOutput::QueryChanged(String::new()))
    );
}

#[test]
fn test_clear_query() {
    let mut state = active_state();
    CommandPalette::update(&mut state, CommandPaletteMessage::TypeChar('a'));
    CommandPalette::update(&mut state, CommandPaletteMessage::TypeChar('b'));
    let output = CommandPalette::update(&mut state, CommandPaletteMessage::ClearQuery);
    assert_eq!(state.query(), "");
    assert_eq!(
        output,
        Some(CommandPaletteOutput::QueryChanged(String::new()))
    );
}

#[test]
fn test_clear_query_on_empty_returns_none() {
    let mut state = active_state();
    let output = CommandPalette::update(&mut state, CommandPaletteMessage::ClearQuery);
    assert_eq!(output, None);
}

#[test]
fn test_set_query() {
    let mut state = active_state();
    let output = CommandPalette::update(
        &mut state,
        CommandPaletteMessage::SetQuery("test".to_string()),
    );
    assert_eq!(state.query(), "test");
    assert_eq!(
        output,
        Some(CommandPaletteOutput::QueryChanged("test".to_string()))
    );
}

#[test]
fn test_query_change_resets_selection_to_first() {
    let mut state = active_state();
    CommandPalette::update(&mut state, CommandPaletteMessage::SelectNext);
    CommandPalette::update(&mut state, CommandPaletteMessage::SelectNext);
    assert_eq!(state.selected_index(), Some(2));

    CommandPalette::update(&mut state, CommandPaletteMessage::TypeChar('o'));
    assert_eq!(state.selected_index(), Some(0));
}

// =============================================================================
// Selection navigation
// =============================================================================

#[test]
fn test_select_next() {
    let mut state = active_state();
    assert_eq!(state.selected_index(), Some(0));
    CommandPalette::update(&mut state, CommandPaletteMessage::SelectNext);
    assert_eq!(state.selected_index(), Some(1));
}

#[test]
fn test_select_prev() {
    let mut state = active_state();
    CommandPalette::update(&mut state, CommandPaletteMessage::SelectNext);
    CommandPalette::update(&mut state, CommandPaletteMessage::SelectPrev);
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_select_next_wraps() {
    let mut state = active_state();
    for _ in 0..4 {
        CommandPalette::update(&mut state, CommandPaletteMessage::SelectNext);
    }
    assert_eq!(state.selected_index(), Some(4));
    CommandPalette::update(&mut state, CommandPaletteMessage::SelectNext);
    assert_eq!(state.selected_index(), Some(0)); // wraps around
}

#[test]
fn test_select_prev_wraps() {
    let mut state = active_state();
    assert_eq!(state.selected_index(), Some(0));
    CommandPalette::update(&mut state, CommandPaletteMessage::SelectPrev);
    assert_eq!(state.selected_index(), Some(4)); // wraps to last
}

#[test]
fn test_confirm_emits_selected() {
    let mut state = active_state();
    let output = CommandPalette::update(&mut state, CommandPaletteMessage::Confirm);
    assert!(matches!(output, Some(CommandPaletteOutput::Selected(_))));
    if let Some(CommandPaletteOutput::Selected(item)) = output {
        assert_eq!(item.id, "open");
    }
}

#[test]
fn test_confirm_hides_palette() {
    let mut state = active_state();
    CommandPalette::update(&mut state, CommandPaletteMessage::Confirm);
    assert!(!state.is_visible());
}

#[test]
fn test_confirm_clears_query() {
    let mut state = active_state();
    CommandPalette::update(&mut state, CommandPaletteMessage::TypeChar('s'));
    CommandPalette::update(&mut state, CommandPaletteMessage::Confirm);
    assert_eq!(state.query(), "");
}

#[test]
fn test_confirm_with_filter_selects_filtered_item() {
    let mut state = active_state();
    // Filter to items starting with "q" (Quit Application)
    CommandPalette::update(&mut state, CommandPaletteMessage::TypeChar('q'));
    let output = CommandPalette::update(&mut state, CommandPaletteMessage::Confirm);
    if let Some(CommandPaletteOutput::Selected(item)) = output {
        assert_eq!(item.id, "quit");
    } else {
        panic!("Expected Selected output");
    }
}

#[test]
fn test_confirm_empty_filtered_list_returns_none() {
    let mut state = active_state();
    CommandPalette::update(
        &mut state,
        CommandPaletteMessage::SetQuery("xyzxyz".to_string()),
    );
    assert_eq!(state.filtered_count(), 0);
    let output = CommandPalette::update(&mut state, CommandPaletteMessage::Confirm);
    assert_eq!(output, None);
}

#[test]
fn test_confirm_on_empty_items_returns_none() {
    let mut state = CommandPaletteState::new(vec![]);
    state.set_focused(true);
    state.set_visible(true);
    let output = CommandPalette::update(&mut state, CommandPaletteMessage::Confirm);
    assert_eq!(output, None);
}

// =============================================================================
// Dismiss
// =============================================================================

#[test]
fn test_dismiss_hides() {
    let mut state = active_state();
    CommandPalette::update(&mut state, CommandPaletteMessage::Dismiss);
    assert!(!state.is_visible());
}

#[test]
fn test_dismiss_clears_query() {
    let mut state = active_state();
    CommandPalette::update(&mut state, CommandPaletteMessage::TypeChar('a'));
    CommandPalette::update(&mut state, CommandPaletteMessage::Dismiss);
    assert_eq!(state.query(), "");
}

#[test]
fn test_dismiss_emits_dismissed() {
    let mut state = active_state();
    let output = CommandPalette::update(&mut state, CommandPaletteMessage::Dismiss);
    assert_eq!(output, Some(CommandPaletteOutput::Dismissed));
}

#[test]
fn test_dismiss_restores_full_item_list() {
    let mut state = active_state();
    CommandPalette::update(&mut state, CommandPaletteMessage::TypeChar('q'));
    assert_eq!(state.filtered_count(), 1);
    CommandPalette::update(&mut state, CommandPaletteMessage::Dismiss);
    assert_eq!(state.filtered_count(), 5);
}

// =============================================================================
// Show
// =============================================================================

#[test]
fn test_show_sets_visible() {
    let mut state = CommandPaletteState::new(sample_items());
    CommandPalette::update(&mut state, CommandPaletteMessage::Show);
    assert!(state.is_visible());
}

#[test]
fn test_show_sets_focused() {
    let mut state = CommandPaletteState::new(sample_items());
    CommandPalette::update(&mut state, CommandPaletteMessage::Show);
    assert!(state.is_focused());
}

// =============================================================================
// SetItems
// =============================================================================

#[test]
fn test_set_items_replaces_items() {
    let mut state = active_state();
    CommandPalette::update(
        &mut state,
        CommandPaletteMessage::SetItems(vec![
            PaletteItem::new("x", "X-Ray"),
            PaletteItem::new("y", "Yankee"),
        ]),
    );
    assert_eq!(state.items().len(), 2);
    assert_eq!(state.filtered_count(), 2);
}

#[test]
fn test_set_items_refilters_with_current_query() {
    let mut state = active_state();
    CommandPalette::update(&mut state, CommandPaletteMessage::TypeChar('z'));
    CommandPalette::update(
        &mut state,
        CommandPaletteMessage::SetItems(vec![
            PaletteItem::new("z", "Zebra"),
            PaletteItem::new("a", "Aardvark"),
        ]),
    );
    // "z" matches "Zebra" as prefix
    assert_eq!(state.filtered_count(), 1);
    assert_eq!(state.filtered_items()[0].id, "z");
}

#[test]
fn test_set_items_method() {
    let mut state = active_state();
    state.set_items(vec![PaletteItem::new("new", "New Item")]);
    assert_eq!(state.items().len(), 1);
}

// =============================================================================
// Disabled state
// =============================================================================

#[test]
fn test_disabled_ignores_all_messages() {
    let mut state = active_state();
    state.set_disabled(true);

    let output = CommandPalette::update(&mut state, CommandPaletteMessage::TypeChar('a'));
    assert_eq!(output, None);

    let output = CommandPalette::update(&mut state, CommandPaletteMessage::Confirm);
    assert_eq!(output, None);

    let output = CommandPalette::update(&mut state, CommandPaletteMessage::Dismiss);
    assert_eq!(output, None);
}

#[test]
fn test_disabled_ignores_events() {
    let mut state = active_state();
    state.set_disabled(true);
    let msg = CommandPalette::handle_event(&state, &Event::char('a'));
    assert_eq!(msg, None);
}

// =============================================================================
// Unfocused state
// =============================================================================

#[test]
fn test_unfocused_ignores_events() {
    let mut state = CommandPaletteState::new(sample_items());
    state.set_visible(true);
    // focused is false by default
    let msg = CommandPalette::handle_event(&state, &Event::char('a'));
    assert_eq!(msg, None);
}

// =============================================================================
// Hidden state
// =============================================================================

#[test]
fn test_hidden_ignores_events() {
    let mut state = CommandPaletteState::new(sample_items());
    state.set_focused(true);
    // visible is false by default
    let msg = CommandPalette::handle_event(&state, &Event::char('a'));
    assert_eq!(msg, None);
}

// =============================================================================
// Show/dismiss methods
// =============================================================================

#[test]
fn test_show_method() {
    let mut state = CommandPaletteState::new(sample_items());
    state.show();
    assert!(state.is_visible());
    assert!(state.is_focused());
}

#[test]
fn test_dismiss_method() {
    let mut state = active_state();
    CommandPalette::update(&mut state, CommandPaletteMessage::TypeChar('a'));
    state.dismiss();
    assert!(!state.is_visible());
    assert_eq!(state.query(), "");
    assert_eq!(state.filtered_count(), 5); // all items restored
}

// =============================================================================
// Instance methods
// =============================================================================

#[test]
fn test_instance_handle_event() {
    let state = active_state();
    let msg = state.handle_event(&Event::char('x'));
    assert_eq!(msg, Some(CommandPaletteMessage::TypeChar('x')));
}

#[test]
fn test_instance_update() {
    let mut state = active_state();
    let output = state.update(CommandPaletteMessage::TypeChar('a'));
    assert!(matches!(
        output,
        Some(CommandPaletteOutput::QueryChanged(_))
    ));
}

#[test]
fn test_instance_dispatch_event() {
    let mut state = active_state();
    let output = state.dispatch_event(&Event::char('a'));
    assert!(matches!(
        output,
        Some(CommandPaletteOutput::QueryChanged(_))
    ));
}

// =============================================================================
// Focusable trait
// =============================================================================

#[test]
fn test_focusable_trait() {
    let mut state = CommandPalette::init();
    assert!(!CommandPalette::is_focused(&state));

    CommandPalette::focus(&mut state);
    assert!(CommandPalette::is_focused(&state));

    CommandPalette::blur(&mut state);
    assert!(!CommandPalette::is_focused(&state));
}

// =============================================================================
// Disableable trait
// =============================================================================

#[test]
fn test_disableable_trait() {
    let mut state = CommandPalette::init();
    assert!(!CommandPalette::is_disabled(&state));

    CommandPalette::disable(&mut state);
    assert!(CommandPalette::is_disabled(&state));

    CommandPalette::enable(&mut state);
    assert!(!CommandPalette::is_disabled(&state));
}

// =============================================================================
// Toggleable trait
// =============================================================================

#[test]
fn test_toggleable_trait() {
    let mut state = CommandPalette::init();
    assert!(!CommandPalette::is_visible(&state));

    CommandPalette::show(&mut state);
    assert!(CommandPalette::is_visible(&state));

    CommandPalette::hide(&mut state);
    assert!(!CommandPalette::is_visible(&state));

    CommandPalette::toggle(&mut state);
    assert!(CommandPalette::is_visible(&state));
}

// =============================================================================
// PartialEq
// =============================================================================

#[test]
fn test_partial_eq() {
    let state1 = CommandPaletteState::new(sample_items());
    let state2 = CommandPaletteState::new(sample_items());
    assert_eq!(state1, state2);
}

#[test]
fn test_partial_eq_different_query() {
    let mut state1 = active_state();
    let state2 = active_state();
    CommandPalette::update(&mut state1, CommandPaletteMessage::TypeChar('a'));
    assert_ne!(state1, state2);
}

// =============================================================================
// Rendering
// =============================================================================

#[test]
fn test_render_hidden_is_noop() {
    let state = CommandPaletteState::new(sample_items());
    assert!(!state.is_visible());
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            CommandPalette::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    // Should render nothing (blank)
}

#[test]
fn test_render_visible() {
    let state = active_state();
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            CommandPalette::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_with_query() {
    let mut state = active_state();
    CommandPalette::update(&mut state, CommandPaletteMessage::TypeChar('o'));
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            CommandPalette::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_with_selection() {
    let mut state = active_state();
    CommandPalette::update(&mut state, CommandPaletteMessage::SelectNext);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            CommandPalette::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_no_matches() {
    let mut state = active_state();
    CommandPalette::update(
        &mut state,
        CommandPaletteMessage::SetQuery("xyzxyz".to_string()),
    );
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            CommandPalette::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_empty_items() {
    let mut state = CommandPaletteState::new(vec![]);
    state.set_focused(true);
    state.set_visible(true);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            CommandPalette::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn test_single_item() {
    let mut state = CommandPaletteState::new(vec![PaletteItem::new("only", "Only Item")]);
    state.set_focused(true);
    state.set_visible(true);

    assert_eq!(state.selected_index(), Some(0));
    CommandPalette::update(&mut state, CommandPaletteMessage::SelectNext);
    assert_eq!(state.selected_index(), Some(0)); // wraps to 0
    CommandPalette::update(&mut state, CommandPaletteMessage::SelectPrev);
    assert_eq!(state.selected_index(), Some(0)); // wraps to 0

    let output = CommandPalette::update(&mut state, CommandPaletteMessage::Confirm);
    if let Some(CommandPaletteOutput::Selected(item)) = output {
        assert_eq!(item.id, "only");
    } else {
        panic!("Expected Selected output");
    }
}

#[test]
fn test_all_items_filtered_out() {
    let mut state = active_state();
    CommandPalette::update(
        &mut state,
        CommandPaletteMessage::SetQuery("zzzzz".to_string()),
    );
    assert_eq!(state.filtered_count(), 0);
    assert_eq!(state.selected_index(), None);
    assert_eq!(state.selected_item(), None);

    // Navigation on empty list should be safe
    CommandPalette::update(&mut state, CommandPaletteMessage::SelectNext);
    CommandPalette::update(&mut state, CommandPaletteMessage::SelectPrev);
    assert_eq!(state.selected_index(), None);
}

#[test]
fn test_fuzzy_filter_sorts_by_score() {
    let items = vec![
        PaletteItem::new("zope", "Zope Framework"), // fuzzy match for "op"
        PaletteItem::new("open", "Open File"),      // prefix match for "op"
        PaletteItem::new("copy", "Copy to Clipboard"), // substring match for "op"
    ];
    let mut state = CommandPaletteState::new(items);
    state.set_focused(true);
    state.set_visible(true);
    CommandPalette::update(
        &mut state,
        CommandPaletteMessage::SetQuery("op".to_string()),
    );

    let filtered = state.filtered_items();
    // Open File should be first (prefix), then Copy to Clipboard (substring)
    assert!(!filtered.is_empty());
    assert_eq!(filtered[0].id, "open"); // prefix match
}

// =============================================================================
// Annotation
// =============================================================================

#[test]
fn test_annotation_emitted() {
    use crate::annotation::with_annotations;
    let state = active_state();
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                CommandPalette::view(&state, frame, frame.area(), &theme, &ViewContext::default());
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.find_by_id("command_palette");
    assert_eq!(regions.len(), 1);
}

#[test]
fn command_palette_state_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<CommandPaletteState>();
}
