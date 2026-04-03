//! Round-trip serialization tests for component State types.
//!
//! Verifies that all State types can be serialized to JSON and deserialized
//! back to equivalent values.

#![cfg(feature = "serialization")]

use envision::component::*;

// =============================================================================
// Helper
// =============================================================================

fn round_trip<T>(value: &T) -> T
where
    T: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug,
{
    let json = serde_json::to_string(value).expect("serialize failed");
    serde_json::from_str(&json).expect("deserialize failed")
}

// =============================================================================
// Simple State types (no generics)
// =============================================================================

#[test]
fn test_button_state_round_trip() {
    let state = ButtonState::new("Submit");
    let restored = round_trip(&state);
    assert_eq!(restored.label(), "Submit");
}

#[test]
fn test_checkbox_state_round_trip() {
    let mut state = CheckboxState::new("Accept");
    Checkbox::update(&mut state, CheckboxMessage::Toggle);
    let restored = round_trip(&state);
    assert_eq!(restored.label(), "Accept");
    assert!(restored.is_checked());
}

#[test]
fn test_input_field_state_round_trip() {
    let state = InputFieldState::with_value("hello world");
    let restored = round_trip(&state);
    assert_eq!(restored.value(), "hello world");
}

#[test]
fn test_text_area_state_round_trip() {
    let state = TextAreaState::new().with_value("line 1\nline 2\nline 3");
    let restored = round_trip(&state);
    assert_eq!(restored.value(), "line 1\nline 2\nline 3");
    assert_eq!(restored.line_count(), 3);
}

#[test]
fn test_progress_bar_state_round_trip() {
    let mut state = ProgressBarState::new();
    ProgressBar::update(&mut state, ProgressBarMessage::SetProgress(0.75));
    let restored = round_trip(&state);
    assert!((restored.progress() - 0.75).abs() < f32::EPSILON);
}

#[test]
fn test_select_state_round_trip() {
    let state = SelectState::new(vec!["A", "B", "C"]);
    let restored = round_trip(&state);
    assert_eq!(restored.options().len(), 3);
}

#[test]
fn test_dropdown_state_round_trip() {
    let state = DropdownState::new(vec!["Red", "Green", "Blue"]);
    let restored = round_trip(&state);
    assert_eq!(restored.options().len(), 3);
}

#[test]
fn test_dialog_state_round_trip() {
    let state = DialogState::new(
        "Title",
        "Message",
        vec![
            DialogButton::new("ok", "OK"),
            DialogButton::new("cancel", "Cancel"),
        ],
    );
    let restored: DialogState = round_trip(&state);
    assert_eq!(restored.title(), "Title");
    assert_eq!(restored.message(), "Message");
}

#[test]
fn test_menu_state_round_trip() {
    let state = MenuState::new(vec![
        MenuItem::new("File"),
        MenuItem::new("Edit"),
        MenuItem::new("View"),
    ]);
    let restored = round_trip(&state);
    assert_eq!(restored.items().len(), 3);
}

#[test]
fn test_spinner_state_round_trip() {
    let state = SpinnerState::new();
    let restored = round_trip(&state);
    assert!(restored.is_spinning());
}

#[test]
fn test_accordion_state_round_trip() {
    let state = AccordionState::new(vec![
        AccordionPanel::new("Section 1", "Content 1"),
        AccordionPanel::new("Section 2", "Content 2"),
    ]);
    let restored = round_trip(&state);
    assert_eq!(restored.panels().len(), 2);
}

#[test]
fn test_breadcrumb_state_round_trip() {
    let state = BreadcrumbState::new(vec![
        BreadcrumbSegment::new("Home"),
        BreadcrumbSegment::new("Products"),
    ]);
    let restored = round_trip(&state);
    assert_eq!(restored.segments().len(), 2);
}

#[test]
fn test_key_hints_state_round_trip() {
    let state = KeyHintsState::new()
        .hint("Enter", "Select")
        .hint("Esc", "Cancel");
    let restored = round_trip(&state);
    assert_eq!(restored.hints().len(), 2);
}

#[test]
fn test_multi_progress_state_round_trip() {
    let mut state = MultiProgressState::new();
    state.add("task1", "Download");
    let restored = round_trip(&state);
    assert_eq!(restored.items().len(), 1);
}

#[test]
fn test_status_bar_state_round_trip() {
    let mut state = StatusBarState::new();
    state.set_left(vec![StatusBarItem::new("Ready")]);
    let restored = round_trip(&state);
    assert_eq!(restored.left().len(), 1);
}

#[test]
fn test_status_log_state_round_trip() {
    let mut state = StatusLogState::new();
    state.info("Started");
    state.error("Failed");
    let restored = round_trip(&state);
    assert_eq!(restored.entries().len(), 2);
}

#[test]
fn test_toast_state_round_trip() {
    let mut state = ToastState::new();
    ToastState::info(&mut state, "Hello");
    let restored = round_trip(&state);
    assert_eq!(restored.toasts().len(), 1);
}

#[test]
fn test_tooltip_state_round_trip() {
    let state = TooltipState::new("Helpful tip");
    let restored = round_trip(&state);
    assert_eq!(restored.content(), "Helpful tip");
}

#[test]
fn test_scrollable_text_state_round_trip() {
    let state = ScrollableTextState::new()
        .with_content("Hello, world!")
        .with_title("Preview");
    let restored = round_trip(&state);
    assert_eq!(restored.content(), "Hello, world!");
    assert_eq!(restored.title(), Some("Preview"));
}

#[test]
fn test_title_card_state_round_trip() {
    let state = TitleCardState::new("My App")
        .with_subtitle("v1.0")
        .with_prefix("\u{1f680} ");
    let restored = round_trip(&state);
    assert_eq!(restored.title(), "My App");
    assert_eq!(restored.subtitle(), Some("v1.0"));
    assert_eq!(restored.prefix(), Some("\u{1f680} "));
}

#[test]
fn test_line_input_state_round_trip() {
    let state = LineInputState::with_value("hello world");
    let restored = round_trip(&state);
    assert_eq!(restored.value(), "hello world");
}

// =============================================================================
// Generic State types
// =============================================================================

#[test]
fn test_selectable_list_state_round_trip() {
    let state = SelectableListState::new(vec!["Alpha".to_string(), "Beta".to_string()]);
    let restored: SelectableListState<String> = round_trip(&state);
    assert_eq!(restored.items().len(), 2);
}

#[test]
fn test_radio_group_state_round_trip() {
    let state = RadioGroupState::new(vec!["Option A".to_string(), "Option B".to_string()]);
    let restored: RadioGroupState<String> = round_trip(&state);
    assert_eq!(restored.options().len(), 2);
}

#[test]
fn test_tabs_state_round_trip() {
    let state = TabsState::new(vec!["Tab 1".to_string(), "Tab 2".to_string()]);
    let restored: TabsState<String> = round_trip(&state);
    assert_eq!(restored.tabs().len(), 2);
}

#[test]
fn test_tree_state_round_trip() {
    let mut root = TreeNode::new("Root", 0);
    root.add_child(TreeNode::new("Child 1", 1));
    root.add_child(TreeNode::new("Child 2", 2));
    let state = TreeState::new(vec![root]);
    let restored: TreeState<i32> = round_trip(&state);
    assert_eq!(restored.roots().len(), 1);
}

#[test]
fn test_table_state_round_trip() {
    // TableRow is a trait, so we test with a serializable row type
    // Table columns lose width (Constraint) on round-trip but preserve headers
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    struct TestRow {
        name: String,
        value: i32,
    }

    impl TableRow for TestRow {
        fn cells(&self) -> Vec<String> {
            vec![self.name.clone(), self.value.to_string()]
        }
    }

    let columns = vec![
        Column::new("Name", ratatui::layout::Constraint::Min(10)),
        Column::new("Value", ratatui::layout::Constraint::Min(5)),
    ];
    let rows = vec![
        TestRow {
            name: "Alice".into(),
            value: 42,
        },
        TestRow {
            name: "Bob".into(),
            value: 17,
        },
    ];
    let state = TableState::new(rows, columns);
    let json = serde_json::to_string(&state).expect("serialize");
    let restored: TableState<TestRow> = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored.rows().len(), 2);
}

#[test]
fn test_router_state_round_trip() {
    let mut state = RouterState::new("home".to_string());
    Router::update(&mut state, RouterMessage::Navigate("settings".to_string()));
    let restored: RouterState<String> = round_trip(&state);
    assert_eq!(restored.current(), "settings");
}

#[test]
fn test_loading_list_state_round_trip() {
    let state = LoadingListState::with_items(vec!["task1".to_string(), "task2".to_string()], |t| {
        t.clone()
    });
    let restored: LoadingListState<String> = round_trip(&state);
    assert_eq!(restored.items().len(), 2);
}

// =============================================================================
// JSON structure verification
// =============================================================================

#[test]
fn test_json_structure_is_clean() {
    let state = ButtonState::new("OK");
    let json = serde_json::to_value(&state).expect("serialize");
    // Verify the JSON has expected fields
    assert!(json.get("label").is_some());
    assert!(json.get("focused").is_some());
    assert!(json.get("disabled").is_some());
}

#[test]
fn test_skipped_fields_not_in_json() {
    let state = SelectableListState::new(vec!["a".to_string()]);
    let json = serde_json::to_value(&state).expect("serialize");
    // list_state should be skipped
    assert!(json.get("list_state").is_none());
    // items should be present
    assert!(json.get("items").is_some());
}
