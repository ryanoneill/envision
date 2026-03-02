#![cfg(feature = "full")]
//! Integration tests exercising multi-component workflows through the public API.

use envision::{
    Accordion, AccordionPanel, AccordionState, App, AppHarness, Breadcrumb, BreadcrumbSegment,
    BreadcrumbState, Button, ButtonOutput, ButtonState, CaptureBackend, Checkbox, CheckboxMessage,
    CheckboxOutput, CheckboxState, Column, Command, Component, Dialog, DialogButton, DialogMessage,
    DialogOutput, DialogState, Dropdown, DropdownState, Event, FocusManager, Focusable, InputField,
    InputFieldMessage, InputFieldOutput, InputFieldState, KeyHint, KeyHints, KeyHintsState,
    LoadingList, LoadingListState, Menu, MenuItem, MenuState, MultiProgress, MultiProgressState,
    ProgressBar, ProgressBarState, RadioGroup, RadioGroupMessage, RadioGroupOutput,
    RadioGroupState, Select, SelectState, SelectableList, SelectableListMessage,
    SelectableListOutput, SelectableListState, Spinner, SpinnerState, StatusBar, StatusBarState,
    StatusLog, StatusLogState, Table, TableRow, TableState, Tabs, TabsMessage, TabsOutput,
    TabsState, TextArea, TextAreaState, Theme, Toast, ToastState, Toggleable, Tooltip,
    TooltipState, Tree, TreeNode, TreeState,
};
use ratatui::prelude::*;
use ratatui::Terminal;

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
    InputField::update(&mut state, InputFieldMessage::Insert('J'));
    InputField::update(&mut state, InputFieldMessage::Insert('o'));
    InputField::update(&mut state, InputFieldMessage::Insert('h'));
    InputField::update(&mut state, InputFieldMessage::Insert('n'));

    assert_eq!(state.value(), "John");
    assert_eq!(state.cursor_position(), 4);

    // Submit
    let output = InputField::update(&mut state, InputFieldMessage::Submit);
    assert_eq!(output, Some(InputFieldOutput::Submitted("John".into())));
}

#[test]
fn test_selectable_list_navigation_200_items() {
    let items: Vec<String> = (0..200).map(|i| format!("Item {}", i)).collect();
    let mut state = SelectableListState::new(items);

    // Should start at index 0
    assert_eq!(state.selected_index(), Some(0));

    // Navigate down 199 times to reach the last item
    for _ in 0..199 {
        SelectableList::<String>::update(&mut state, SelectableListMessage::Down);
    }
    assert_eq!(state.selected_index(), Some(199));
    assert_eq!(state.selected_item(), Some(&"Item 199".to_string()));

    // Down at last should stay at last (no wrapping in SelectableList)
    let output = SelectableList::<String>::update(&mut state, SelectableListMessage::Down);
    assert_eq!(output, None);
    assert_eq!(state.selected_index(), Some(199));

    // PageUp with page size 50
    SelectableList::<String>::update(&mut state, SelectableListMessage::PageUp(50));
    assert_eq!(state.selected_index(), Some(149));

    // First
    SelectableList::<String>::update(&mut state, SelectableListMessage::First);
    assert_eq!(state.selected_index(), Some(0));

    // Last
    SelectableList::<String>::update(&mut state, SelectableListMessage::Last);
    assert_eq!(state.selected_index(), Some(199));

    // Select at last position
    let output = SelectableList::<String>::update(&mut state, SelectableListMessage::Select);
    assert_eq!(
        output,
        Some(SelectableListOutput::Selected("Item 199".to_string()))
    );
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
    Tabs::<String>::update(&mut tabs, TabsMessage::Right);
    Tabs::<String>::update(&mut tabs, TabsMessage::Right);
    assert_eq!(tabs.selected_index(), Some(2));

    // Radio should still be at 0
    assert_eq!(radio.selected_index(), Some(0));

    // Navigate radio to index 3
    RadioGroup::<String>::update(&mut radio, RadioGroupMessage::Down);
    RadioGroup::<String>::update(&mut radio, RadioGroupMessage::Down);
    RadioGroup::<String>::update(&mut radio, RadioGroupMessage::Down);
    assert_eq!(radio.selected_index(), Some(3));

    // Tabs should still be at 2
    assert_eq!(tabs.selected_index(), Some(2));

    // Confirm each
    let tab_output = Tabs::<String>::update(&mut tabs, TabsMessage::Confirm);
    assert_eq!(tab_output, Some(TabsOutput::Confirmed("Help".to_string())));

    let radio_output = RadioGroup::<String>::update(&mut radio, RadioGroupMessage::Confirm);
    assert_eq!(
        radio_output,
        Some(RadioGroupOutput::Confirmed("Option D".to_string()))
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

// ---------------------------------------------------------------------------
// Test 1: Form workflow with FocusManager
// ---------------------------------------------------------------------------

#[test]
fn test_form_workflow_with_focus_manager() {
    #[derive(Clone, PartialEq, Debug)]
    enum FormField {
        Name,
        AcceptTerms,
        Submit,
    }

    // Create a FocusManager with 3 components
    let mut focus = FocusManager::with_initial_focus(vec![
        FormField::Name,
        FormField::AcceptTerms,
        FormField::Submit,
    ]);

    // Create component states
    let mut input = InputFieldState::new();
    let mut checkbox = CheckboxState::new("Accept Terms");
    let mut button = ButtonState::new("Submit");

    // Initially focused on Name
    assert_eq!(focus.focused(), Some(&FormField::Name));

    // Set InputField focused since FocusManager says Name is focused
    InputField::set_focused(&mut input, true);
    assert!(InputField::is_focused(&input));

    // Type "Alice" into the InputField via dispatch_event
    let events = [
        Event::char('A'),
        Event::char('l'),
        Event::char('i'),
        Event::char('c'),
        Event::char('e'),
    ];
    for event in &events {
        InputField::dispatch_event(&mut input, event);
    }
    assert_eq!(input.value(), "Alice");
    assert_eq!(input.cursor_position(), 5);

    // Tab to Checkbox: advance FocusManager, blur InputField, focus Checkbox
    focus.focus_next();
    assert_eq!(focus.focused(), Some(&FormField::AcceptTerms));
    InputField::set_focused(&mut input, false);
    Checkbox::set_focused(&mut checkbox, true);

    // Verify InputField is no longer focused and Checkbox is
    assert!(!InputField::is_focused(&input));
    assert!(Checkbox::is_focused(&checkbox));

    // Toggle checkbox via dispatch_event (Space key)
    let space_event = Event::char(' ');
    let output = Checkbox::dispatch_event(&mut checkbox, &space_event);
    assert_eq!(output, Some(CheckboxOutput::Toggled(true)));
    assert!(checkbox.is_checked());

    // Tab to Button: advance FocusManager, blur Checkbox, focus Button
    focus.focus_next();
    assert_eq!(focus.focused(), Some(&FormField::Submit));
    Checkbox::set_focused(&mut checkbox, false);
    Button::set_focused(&mut button, true);

    // Verify Checkbox is no longer focused and Button is
    assert!(!Checkbox::is_focused(&checkbox));
    assert!(Button::is_focused(&button));

    // Press Enter on Button via dispatch_event
    let enter_event = Event::key(crossterm::event::KeyCode::Enter);
    let output = Button::dispatch_event(&mut button, &enter_event);
    assert_eq!(output, Some(ButtonOutput::Pressed));

    // Final state verification: all components retain their state after the workflow
    assert_eq!(input.value(), "Alice");
    assert!(checkbox.is_checked());
    assert!(Button::is_focused(&button));
}

// ---------------------------------------------------------------------------
// Test 2: Stress test with 10,000 items
// ---------------------------------------------------------------------------

#[test]
fn test_selectable_list_stress_10000_items() {
    let items: Vec<String> = (0..10_000).map(|i| format!("Item {}", i)).collect();
    let mut state = SelectableListState::new(items);
    state.set_focused(true);

    // Should start at index 0
    assert_eq!(state.selected_index(), Some(0));

    // Send 100 Down events via dispatch_event
    let down_event = Event::key(crossterm::event::KeyCode::Down);
    for _ in 0..100 {
        state.dispatch_event(&down_event);
    }
    assert_eq!(state.selected_index(), Some(100));

    // Send PageDown(1000) via update to jump by 1000
    SelectableList::<String>::update(&mut state, SelectableListMessage::PageDown(1000));
    assert_eq!(state.selected_index(), Some(1100));

    // Send Last to jump to the end
    SelectableList::<String>::update(&mut state, SelectableListMessage::Last);
    assert_eq!(state.selected_index(), Some(9999));
    assert_eq!(state.selected_item(), Some(&"Item 9999".to_string()));

    // Send First to jump to the beginning
    SelectableList::<String>::update(&mut state, SelectableListMessage::First);
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(state.selected_item(), Some(&"Item 0".to_string()));
}

// ---------------------------------------------------------------------------
// Test 3: Zero-size area rendering
// ---------------------------------------------------------------------------

/// Helper: creates a 0x0 terminal and renders the given component view without panicking.
fn assert_view_zero_size<F>(name: &str, render_fn: F)
where
    F: FnOnce(&mut Frame, Rect, &Theme),
{
    let backend = CaptureBackend::new(0, 0);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = Theme::default();
    terminal
        .draw(|frame| {
            let area = Rect::default(); // 0x0 area
            render_fn(frame, area, &theme);
        })
        .unwrap_or_else(|e| panic!("{} panicked on zero-size area: {}", name, e));
}

#[derive(Clone)]
struct SimpleRow {
    name: String,
}

impl TableRow for SimpleRow {
    fn cells(&self) -> Vec<String> {
        vec![self.name.clone()]
    }
}

#[test]
fn test_components_handle_zero_size_area() {
    // Button
    assert_view_zero_size("Button", |frame, area, theme| {
        let state = ButtonState::new("Click");
        Button::view(&state, frame, area, theme);
    });

    // Checkbox
    assert_view_zero_size("Checkbox", |frame, area, theme| {
        let state = CheckboxState::new("Check");
        Checkbox::view(&state, frame, area, theme);
    });

    // InputField
    assert_view_zero_size("InputField", |frame, area, theme| {
        let state = InputFieldState::new();
        InputField::view(&state, frame, area, theme);
    });

    // SelectableList
    assert_view_zero_size("SelectableList", |frame, area, theme| {
        let state = SelectableListState::new(vec!["A".to_string(), "B".to_string()]);
        SelectableList::<String>::view(&state, frame, area, theme);
    });

    // RadioGroup
    assert_view_zero_size("RadioGroup", |frame, area, theme| {
        let state = RadioGroupState::new(vec!["X".to_string(), "Y".to_string()]);
        RadioGroup::<String>::view(&state, frame, area, theme);
    });

    // Tabs
    assert_view_zero_size("Tabs", |frame, area, theme| {
        let state = TabsState::new(vec!["Tab1".to_string(), "Tab2".to_string()]);
        Tabs::<String>::view(&state, frame, area, theme);
    });

    // Table
    assert_view_zero_size("Table", |frame, area, theme| {
        let rows = vec![SimpleRow {
            name: "row1".into(),
        }];
        let columns = vec![Column::new("Name", Constraint::Length(10))];
        let state = TableState::new(rows, columns);
        Table::<SimpleRow>::view(&state, frame, area, theme);
    });

    // Tree
    assert_view_zero_size("Tree", |frame, area, theme| {
        let root = TreeNode::new("root", "root_data".to_string());
        let state = TreeState::new(vec![root]);
        Tree::<String>::view(&state, frame, area, theme);
    });

    // Accordion
    assert_view_zero_size("Accordion", |frame, area, theme| {
        let panel = AccordionPanel::new("Panel", "Content");
        let state = AccordionState::new(vec![panel]);
        Accordion::view(&state, frame, area, theme);
    });

    // Dialog
    assert_view_zero_size("Dialog", |frame, area, theme| {
        let mut state = DialogState::confirm("Title", "Body");
        Dialog::update(&mut state, DialogMessage::Open);
        Dialog::view(&state, frame, area, theme);
    });

    // Menu
    assert_view_zero_size("Menu", |frame, area, theme| {
        let items = vec![MenuItem::new("File"), MenuItem::new("Edit")];
        let state = MenuState::new(items);
        Menu::view(&state, frame, area, theme);
    });

    // Dropdown
    assert_view_zero_size("Dropdown", |frame, area, theme| {
        let state = DropdownState::new(vec!["Option 1", "Option 2"]);
        Dropdown::view(&state, frame, area, theme);
    });

    // Select
    assert_view_zero_size("Select", |frame, area, theme| {
        let state = SelectState::new(vec!["Opt A", "Opt B"]);
        Select::view(&state, frame, area, theme);
    });

    // TextArea
    assert_view_zero_size("TextArea", |frame, area, theme| {
        let state = TextAreaState::new();
        TextArea::view(&state, frame, area, theme);
    });

    // ProgressBar
    assert_view_zero_size("ProgressBar", |frame, area, theme| {
        let state = ProgressBarState::new();
        ProgressBar::view(&state, frame, area, theme);
    });

    // Spinner
    assert_view_zero_size("Spinner", |frame, area, theme| {
        let state = SpinnerState::new();
        Spinner::view(&state, frame, area, theme);
    });

    // Toast
    assert_view_zero_size("Toast", |frame, area, theme| {
        let state = ToastState::new();
        Toast::view(&state, frame, area, theme);
    });

    // Tooltip
    assert_view_zero_size("Tooltip", |frame, area, theme| {
        let state = TooltipState::new("Tip content");
        Tooltip::view(&state, frame, area, theme);
    });

    // StatusBar
    assert_view_zero_size("StatusBar", |frame, area, theme| {
        let state = StatusBarState::new();
        StatusBar::view(&state, frame, area, theme);
    });

    // StatusLog
    assert_view_zero_size("StatusLog", |frame, area, theme| {
        let state = StatusLogState::new();
        StatusLog::view(&state, frame, area, theme);
    });

    // Breadcrumb
    assert_view_zero_size("Breadcrumb", |frame, area, theme| {
        let segments = vec![
            BreadcrumbSegment::new("Home"),
            BreadcrumbSegment::new("Settings"),
        ];
        let state = BreadcrumbState::new(segments);
        Breadcrumb::view(&state, frame, area, theme);
    });

    // KeyHints
    assert_view_zero_size("KeyHints", |frame, area, theme| {
        let state = KeyHintsState::with_hints(vec![KeyHint::new("q", "Quit")]);
        KeyHints::view(&state, frame, area, theme);
    });

    // MultiProgress
    assert_view_zero_size("MultiProgress", |frame, area, theme| {
        let state = MultiProgressState::new();
        MultiProgress::view(&state, frame, area, theme);
    });

    // LoadingList
    assert_view_zero_size("LoadingList", |frame, area, theme| {
        let state: LoadingListState<String> = LoadingListState::new();
        LoadingList::<String>::view(&state, frame, area, theme);
    });
}

// ---------------------------------------------------------------------------
// Test 4: AppHarness counter workflow
// ---------------------------------------------------------------------------

struct CounterApp;

#[derive(Clone, Default)]
struct CounterState {
    count: i32,
}

#[derive(Clone, Debug)]
enum CounterMsg {
    Increment,
    Decrement,
}

impl App for CounterApp {
    type State = CounterState;
    type Message = CounterMsg;

    fn init() -> (Self::State, Command<Self::Message>) {
        (CounterState::default(), Command::none())
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
        match msg {
            CounterMsg::Increment => state.count += 1,
            CounterMsg::Decrement => state.count -= 1,
        }
        Command::none()
    }

    fn view(state: &Self::State, frame: &mut ratatui::Frame) {
        let text = format!("Count: {}", state.count);
        frame.render_widget(ratatui::widgets::Paragraph::new(text), frame.area());
    }
}

#[test]
fn test_app_harness_counter_workflow() {
    let mut harness = AppHarness::<CounterApp>::new(40, 10).unwrap();

    // Dispatch increment 5 times
    for _ in 0..5 {
        harness.dispatch(CounterMsg::Increment);
    }
    assert_eq!(harness.state().count, 5);

    // Render and verify display contains "Count: 5"
    harness.render().unwrap();
    harness.assert_contains("Count: 5");

    // Dispatch decrement twice
    harness.dispatch(CounterMsg::Decrement);
    harness.dispatch(CounterMsg::Decrement);
    assert_eq!(harness.state().count, 3);

    // Render and verify display contains "Count: 3"
    harness.render().unwrap();
    harness.assert_contains("Count: 3");

    // Also verify old value is no longer displayed
    harness.assert_not_contains("Count: 5");
}
