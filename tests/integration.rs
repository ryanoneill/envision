//! Integration tests exercising multi-component workflows through the public API.

use envision::{
    Checkbox, CheckboxMessage, CheckboxOutput, CheckboxState, Component, Dialog, DialogButton,
    DialogMessage, DialogOutput, DialogState, FocusManager, Focusable, InputField, InputFieldState,
    InputMessage, InputOutput, ListMessage, ListOutput, RadioGroup, RadioGroupState, RadioMessage,
    RadioOutput, SelectableList, SelectableListState, TabMessage, TabOutput, Tabs, TabsState,
    Toggleable,
};

#[test]
fn test_focus_manager_tab_navigation() {
    #[derive(Clone, PartialEq, Debug)]
    enum Field {
        Username,
        Password,
        Submit,
    }

    let mut focus =
        FocusManager::with_initial_focus(vec![Field::Username, Field::Password, Field::Submit]);

    // Initially focused on first item
    assert_eq!(focus.focused(), Some(&Field::Username));

    // focus_next cycles through all fields
    focus.focus_next();
    assert_eq!(focus.focused(), Some(&Field::Password));

    focus.focus_next();
    assert_eq!(focus.focused(), Some(&Field::Submit));

    // Wraps around to the beginning
    focus.focus_next();
    assert_eq!(focus.focused(), Some(&Field::Username));

    // focus_prev reverses direction
    focus.focus_prev();
    assert_eq!(focus.focused(), Some(&Field::Submit));

    focus.focus_prev();
    assert_eq!(focus.focused(), Some(&Field::Password));

    focus.focus_prev();
    assert_eq!(focus.focused(), Some(&Field::Username));

    // Wraps backward
    focus.focus_prev();
    assert_eq!(focus.focused(), Some(&Field::Submit));

    // blur clears focus
    focus.blur();
    assert_eq!(focus.focused(), None);
    assert!(!focus.is_focused(&Field::Username));
    assert!(!focus.is_focused(&Field::Password));
    assert!(!focus.is_focused(&Field::Submit));

    // focus_first and focus_last jump
    focus.focus_first();
    assert_eq!(focus.focused(), Some(&Field::Username));

    focus.focus_last();
    assert_eq!(focus.focused(), Some(&Field::Submit));
}

#[test]
fn test_dialog_confirm_workflow() {
    let mut state = DialogState::confirm("Delete?", "This cannot be undone.");
    assert!(!Dialog::is_visible(&state));

    // Open the dialog
    Dialog::update(&mut state, DialogMessage::Open);
    assert!(Dialog::is_visible(&state));

    // confirm() sets primary to index 1 (OK), so focused_button starts at 1
    assert_eq!(state.focused_button(), 1);

    // Navigate to Cancel (index 0)
    Dialog::update(&mut state, DialogMessage::FocusPrev);
    assert_eq!(state.focused_button(), 0);

    // Press Cancel
    let output = Dialog::update(&mut state, DialogMessage::Press);
    assert_eq!(output, Some(DialogOutput::ButtonPressed("cancel".into())));
    assert!(!Dialog::is_visible(&state));

    // Re-open the dialog — focus should reset to primary (OK)
    Dialog::update(&mut state, DialogMessage::Open);
    assert!(Dialog::is_visible(&state));
    assert_eq!(state.focused_button(), 1);

    // Press OK
    let output = Dialog::update(&mut state, DialogMessage::Press);
    assert_eq!(output, Some(DialogOutput::ButtonPressed("ok".into())));
    assert!(!Dialog::is_visible(&state));
}

#[test]
fn test_input_field_type_and_submit() {
    let mut state = InputFieldState::new();
    InputField::set_focused(&mut state, true);

    // Type "John"
    InputField::update(&mut state, InputMessage::Insert('J'));
    InputField::update(&mut state, InputMessage::Insert('o'));
    InputField::update(&mut state, InputMessage::Insert('h'));
    InputField::update(&mut state, InputMessage::Insert('n'));

    assert_eq!(state.value(), "John");
    assert_eq!(state.cursor_position(), 4);

    // Submit
    let output = InputField::update(&mut state, InputMessage::Submit);
    assert_eq!(output, Some(InputOutput::Submitted("John".into())));
}

#[test]
fn test_selectable_list_navigation_200_items() {
    let items: Vec<String> = (0..200).map(|i| format!("Item {}", i)).collect();
    let mut state = SelectableListState::new(items);

    // Should start at index 0
    assert_eq!(state.selected_index(), Some(0));

    // Navigate down 199 times to reach the last item
    for _ in 0..199 {
        SelectableList::<String>::update(&mut state, ListMessage::Down);
    }
    assert_eq!(state.selected_index(), Some(199));
    assert_eq!(state.selected_item(), Some(&"Item 199".to_string()));

    // Down at last should stay at last (no wrapping in SelectableList)
    let output = SelectableList::<String>::update(&mut state, ListMessage::Down);
    assert_eq!(output, None);
    assert_eq!(state.selected_index(), Some(199));

    // PageUp with page size 50
    SelectableList::<String>::update(&mut state, ListMessage::PageUp(50));
    assert_eq!(state.selected_index(), Some(149));

    // First
    SelectableList::<String>::update(&mut state, ListMessage::First);
    assert_eq!(state.selected_index(), Some(0));

    // Last
    SelectableList::<String>::update(&mut state, ListMessage::Last);
    assert_eq!(state.selected_index(), Some(199));

    // Select at last position
    let output = SelectableList::<String>::update(&mut state, ListMessage::Select);
    assert_eq!(output, Some(ListOutput::Selected("Item 199".to_string())));
}

#[test]
fn test_tabs_and_radio_group_independent_selection() {
    let mut tabs = TabsState::new(vec![
        "Home".to_string(),
        "Settings".to_string(),
        "Help".to_string(),
        "About".to_string(),
    ]);
    let mut radio = RadioGroupState::new(vec![
        "Option A".to_string(),
        "Option B".to_string(),
        "Option C".to_string(),
        "Option D".to_string(),
    ]);

    // Initially both at index 0
    assert_eq!(tabs.selected_index(), Some(0));
    assert_eq!(radio.selected_index(), Some(0));

    // Navigate tabs to index 2
    Tabs::<String>::update(&mut tabs, TabMessage::Right);
    Tabs::<String>::update(&mut tabs, TabMessage::Right);
    assert_eq!(tabs.selected_index(), Some(2));

    // Radio should still be at 0
    assert_eq!(radio.selected_index(), Some(0));

    // Navigate radio to index 3
    RadioGroup::<String>::update(&mut radio, RadioMessage::Down);
    RadioGroup::<String>::update(&mut radio, RadioMessage::Down);
    RadioGroup::<String>::update(&mut radio, RadioMessage::Down);
    assert_eq!(radio.selected_index(), Some(3));

    // Tabs should still be at 2
    assert_eq!(tabs.selected_index(), Some(2));

    // Confirm each
    let tab_output = Tabs::<String>::update(&mut tabs, TabMessage::Confirm);
    assert_eq!(tab_output, Some(TabOutput::Confirmed("Help".to_string())));

    let radio_output = RadioGroup::<String>::update(&mut radio, RadioMessage::Confirm);
    assert_eq!(
        radio_output,
        Some(RadioOutput::Confirmed("Option D".to_string()))
    );
}

#[test]
fn test_dialog_three_button_full_cycle() {
    let buttons = vec![
        DialogButton::new("save", "Save"),
        DialogButton::new("discard", "Discard"),
        DialogButton::new("cancel", "Cancel"),
    ];
    let mut state = DialogState::with_primary("Unsaved Changes", "Save your work?", buttons, 0);

    // Open
    Dialog::update(&mut state, DialogMessage::Open);
    assert!(Dialog::is_visible(&state));
    assert_eq!(state.focused_button(), 0); // Primary is Save

    // FocusNext cycles through all 3
    Dialog::update(&mut state, DialogMessage::FocusNext);
    assert_eq!(state.focused_button(), 1); // Discard

    Dialog::update(&mut state, DialogMessage::FocusNext);
    assert_eq!(state.focused_button(), 2); // Cancel

    // FocusNext wraps to Save
    Dialog::update(&mut state, DialogMessage::FocusNext);
    assert_eq!(state.focused_button(), 0); // Save (wrapped)

    // Press Save
    let output = Dialog::update(&mut state, DialogMessage::Press);
    assert_eq!(output, Some(DialogOutput::ButtonPressed("save".into())));
    assert!(!Dialog::is_visible(&state));

    // Re-open, navigate to Discard, press
    Dialog::update(&mut state, DialogMessage::Open);
    assert_eq!(state.focused_button(), 0); // Resets to primary
    Dialog::update(&mut state, DialogMessage::FocusNext);
    let output = Dialog::update(&mut state, DialogMessage::Press);
    assert_eq!(output, Some(DialogOutput::ButtonPressed("discard".into())));

    // Re-open, navigate to Cancel, press
    Dialog::update(&mut state, DialogMessage::Open);
    Dialog::update(&mut state, DialogMessage::FocusNext);
    Dialog::update(&mut state, DialogMessage::FocusNext);
    let output = Dialog::update(&mut state, DialogMessage::Press);
    assert_eq!(output, Some(DialogOutput::ButtonPressed("cancel".into())));
}

#[test]
fn test_checkbox_toggle_sequence() {
    let mut cb1 = CheckboxState::new("Accept Terms");
    let mut cb2 = CheckboxState::new("Subscribe");
    let mut cb3 = CheckboxState::new("Remember Me");

    // All start unchecked
    assert!(!cb1.is_checked());
    assert!(!cb2.is_checked());
    assert!(!cb3.is_checked());

    // Toggle cb1 on
    let output = Checkbox::update(&mut cb1, CheckboxMessage::Toggle);
    assert_eq!(output, Some(CheckboxOutput::Toggled(true)));
    assert!(cb1.is_checked());

    // cb2 and cb3 should be independent
    assert!(!cb2.is_checked());
    assert!(!cb3.is_checked());

    // Toggle cb2 on
    Checkbox::update(&mut cb2, CheckboxMessage::Toggle);
    assert!(cb2.is_checked());
    assert!(cb1.is_checked()); // cb1 still checked

    // Toggle cb1 off
    let output = Checkbox::update(&mut cb1, CheckboxMessage::Toggle);
    assert_eq!(output, Some(CheckboxOutput::Toggled(false)));
    assert!(!cb1.is_checked());
    assert!(cb2.is_checked()); // cb2 still checked

    // Disabled checkbox returns None
    cb3.set_disabled(true);
    let output = Checkbox::update(&mut cb3, CheckboxMessage::Toggle);
    assert_eq!(output, None);
    assert!(!cb3.is_checked()); // Unchanged
}
