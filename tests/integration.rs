#![cfg(feature = "full")]
//! Integration tests exercising multi-component workflows through the public API.
use envision::ViewContext;

use envision::component::{
    SearchableList, SearchableListMessage, SearchableListOutput, SearchableListState,
};
use envision::{
    Accordion, AccordionPanel, AccordionState, App, AppHarness, Breadcrumb, BreadcrumbMessage,
    BreadcrumbOutput, BreadcrumbSegment, BreadcrumbState, Button, ButtonOutput, ButtonState,
    CaptureBackend, Checkbox, CheckboxMessage, CheckboxOutput, CheckboxState, Column, Command,
    Component, Dialog, DialogButton, DialogMessage, DialogOutput, DialogState, Dropdown,
    DropdownState, Event, FocusManager, InputField, InputFieldMessage, InputFieldOutput,
    InputFieldState, KeyHint, KeyHints, KeyHintsState, LineInput, LineInputState, LoadingList,
    LoadingListState, Menu, MenuItem, MenuState, MultiProgress, MultiProgressState, ProgressBar,
    ProgressBarState, RadioGroup, RadioGroupMessage, RadioGroupOutput, RadioGroupState,
    ScrollableText, ScrollableTextState, Select, SelectState, SelectableList,
    SelectableListMessage, SelectableListOutput, SelectableListState, Spinner, SpinnerState,
    StatusBar, StatusBarState, StatusLog, StatusLogState, Table, TableRow, TableState, Tabs,
    TabsMessage, TabsOutput, TabsState, TextArea, TextAreaState, Theme, TitleCard, TitleCardState,
    Toast, ToastState, Toggleable, Tooltip, TooltipState, Tree, TreeNode, TreeState,
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
    let cb3 = CheckboxState::new("Remember Me");

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

    // Type "Alice" into the InputField via dispatch_event
    let events = [
        Event::char('A'),
        Event::char('l'),
        Event::char('i'),
        Event::char('c'),
        Event::char('e'),
    ];
    for event in &events {
        InputField::dispatch_event(&mut input, event, &ViewContext::new().focused(true));
    }
    assert_eq!(input.value(), "Alice");
    assert_eq!(input.cursor_position(), 5);

    // Tab to Checkbox: advance FocusManager, blur InputField, focus Checkbox
    focus.focus_next();
    assert_eq!(focus.focused(), Some(&FormField::AcceptTerms));

    // Verify InputField is no longer focused and Checkbox is

    // Toggle checkbox via dispatch_event (Space key)
    let space_event = Event::char(' ');
    let output = Checkbox::dispatch_event(
        &mut checkbox,
        &space_event,
        &ViewContext::new().focused(true),
    );
    assert_eq!(output, Some(CheckboxOutput::Toggled(true)));
    assert!(checkbox.is_checked());

    // Tab to Button: advance FocusManager, blur Checkbox, focus Button
    focus.focus_next();
    assert_eq!(focus.focused(), Some(&FormField::Submit));

    // Verify Checkbox is no longer focused and Button is

    // Press Enter on Button via dispatch_event
    let enter_event = Event::key(crossterm::event::KeyCode::Enter);
    let output =
        Button::dispatch_event(&mut button, &enter_event, &ViewContext::new().focused(true));
    assert_eq!(output, Some(ButtonOutput::Pressed));

    // Final state verification: all components retain their state after the workflow
    assert_eq!(input.value(), "Alice");
    assert!(checkbox.is_checked());
}

// ---------------------------------------------------------------------------
// Test 2: Stress test with 10,000 items
// ---------------------------------------------------------------------------

#[test]
fn test_selectable_list_stress_10000_items() {
    let items: Vec<String> = (0..10_000).map(|i| format!("Item {}", i)).collect();
    let mut state = SelectableListState::new(items);

    // Should start at index 0
    assert_eq!(state.selected_index(), Some(0));

    // Send 100 Down events via dispatch_event
    let down_event = Event::key(crossterm::event::KeyCode::Down);
    for _ in 0..100 {
        SelectableList::<String>::dispatch_event(
            &mut state,
            &down_event,
            &ViewContext::new().focused(true),
        );
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
        Button::view(&state, frame, area, theme, &ViewContext::default());
    });

    // Checkbox
    assert_view_zero_size("Checkbox", |frame, area, theme| {
        let state = CheckboxState::new("Check");
        Checkbox::view(&state, frame, area, theme, &ViewContext::default());
    });

    // InputField
    assert_view_zero_size("InputField", |frame, area, theme| {
        let state = InputFieldState::new();
        InputField::view(&state, frame, area, theme, &ViewContext::default());
    });

    // SelectableList
    assert_view_zero_size("SelectableList", |frame, area, theme| {
        let state = SelectableListState::new(vec!["A".to_string(), "B".to_string()]);
        SelectableList::<String>::view(&state, frame, area, theme, &ViewContext::default());
    });

    // RadioGroup
    assert_view_zero_size("RadioGroup", |frame, area, theme| {
        let state = RadioGroupState::new(vec!["X".to_string(), "Y".to_string()]);
        RadioGroup::<String>::view(&state, frame, area, theme, &ViewContext::default());
    });

    // Tabs
    assert_view_zero_size("Tabs", |frame, area, theme| {
        let state = TabsState::new(vec!["Tab1".to_string(), "Tab2".to_string()]);
        Tabs::<String>::view(&state, frame, area, theme, &ViewContext::default());
    });

    // Table
    assert_view_zero_size("Table", |frame, area, theme| {
        let rows = vec![SimpleRow {
            name: "row1".into(),
        }];
        let columns = vec![Column::new("Name", Constraint::Length(10))];
        let state = TableState::new(rows, columns);
        Table::<SimpleRow>::view(&state, frame, area, theme, &ViewContext::default());
    });

    // Tree
    assert_view_zero_size("Tree", |frame, area, theme| {
        let root = TreeNode::new("root", "root_data".to_string());
        let state = TreeState::new(vec![root]);
        Tree::<String>::view(&state, frame, area, theme, &ViewContext::default());
    });

    // Accordion
    assert_view_zero_size("Accordion", |frame, area, theme| {
        let panel = AccordionPanel::new("Panel", "Content");
        let state = AccordionState::new(vec![panel]);
        Accordion::view(&state, frame, area, theme, &ViewContext::default());
    });

    // Dialog
    assert_view_zero_size("Dialog", |frame, area, theme| {
        let mut state = DialogState::confirm("Title", "Body");
        Dialog::update(&mut state, DialogMessage::Open);
        Dialog::view(&state, frame, area, theme, &ViewContext::default());
    });

    // Menu
    assert_view_zero_size("Menu", |frame, area, theme| {
        let items = vec![MenuItem::new("File"), MenuItem::new("Edit")];
        let state = MenuState::new(items);
        Menu::view(&state, frame, area, theme, &ViewContext::default());
    });

    // Dropdown
    assert_view_zero_size("Dropdown", |frame, area, theme| {
        let state = DropdownState::new(vec!["Option 1", "Option 2"]);
        Dropdown::view(&state, frame, area, theme, &ViewContext::default());
    });

    // Select
    assert_view_zero_size("Select", |frame, area, theme| {
        let state = SelectState::new(vec!["Opt A", "Opt B"]);
        Select::view(&state, frame, area, theme, &ViewContext::default());
    });

    // TextArea
    assert_view_zero_size("TextArea", |frame, area, theme| {
        let state = TextAreaState::new();
        TextArea::view(&state, frame, area, theme, &ViewContext::default());
    });

    // ProgressBar
    assert_view_zero_size("ProgressBar", |frame, area, theme| {
        let state = ProgressBarState::new();
        ProgressBar::view(&state, frame, area, theme, &ViewContext::default());
    });

    // Spinner
    assert_view_zero_size("Spinner", |frame, area, theme| {
        let state = SpinnerState::new();
        Spinner::view(&state, frame, area, theme, &ViewContext::default());
    });

    // Toast
    assert_view_zero_size("Toast", |frame, area, theme| {
        let state = ToastState::new();
        Toast::view(&state, frame, area, theme, &ViewContext::default());
    });

    // Tooltip
    assert_view_zero_size("Tooltip", |frame, area, theme| {
        let state = TooltipState::new("Tip content");
        Tooltip::view(&state, frame, area, theme, &ViewContext::default());
    });

    // StatusBar
    assert_view_zero_size("StatusBar", |frame, area, theme| {
        let state = StatusBarState::new();
        StatusBar::view(&state, frame, area, theme, &ViewContext::default());
    });

    // StatusLog
    assert_view_zero_size("StatusLog", |frame, area, theme| {
        let state = StatusLogState::new();
        StatusLog::view(&state, frame, area, theme, &ViewContext::default());
    });

    // Breadcrumb
    assert_view_zero_size("Breadcrumb", |frame, area, theme| {
        let segments = vec![
            BreadcrumbSegment::new("Home"),
            BreadcrumbSegment::new("Settings"),
        ];
        let state = BreadcrumbState::new(segments);
        Breadcrumb::view(&state, frame, area, theme, &ViewContext::default());
    });

    // KeyHints
    assert_view_zero_size("KeyHints", |frame, area, theme| {
        let state = KeyHintsState::with_hints(vec![KeyHint::new("q", "Quit")]);
        KeyHints::view(&state, frame, area, theme, &ViewContext::default());
    });

    // MultiProgress
    assert_view_zero_size("MultiProgress", |frame, area, theme| {
        let state = MultiProgressState::new();
        MultiProgress::view(&state, frame, area, theme, &ViewContext::default());
    });

    // LoadingList
    assert_view_zero_size("LoadingList", |frame, area, theme| {
        let state: LoadingListState<String> = LoadingListState::new();
        LoadingList::<String>::view(&state, frame, area, theme, &ViewContext::default());
    });

    // ScrollableText
    assert_view_zero_size("ScrollableText", |frame, area, theme| {
        let state = ScrollableTextState::new();
        ScrollableText::view(&state, frame, area, theme, &ViewContext::default());
    });

    // TitleCard
    assert_view_zero_size("TitleCard", |frame, area, theme| {
        let state = TitleCardState::new("Title");
        TitleCard::view(&state, frame, area, theme, &ViewContext::default());
    });

    // LineInput
    assert_view_zero_size("LineInput", |frame, area, theme| {
        let state = LineInputState::new();
        LineInput::view(&state, frame, area, theme, &ViewContext::default());
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

// ---------------------------------------------------------------------------
// Test 5: Settings panel workflow with tabs, input, radio, checkbox
// ---------------------------------------------------------------------------

#[test]
fn test_settings_panel_with_tabs_and_components() {
    #[derive(Clone, PartialEq, Debug)]
    enum Panel {
        General,
        Appearance,
        Keybinds,
    }

    let mut focus =
        FocusManager::with_initial_focus(vec![Panel::General, Panel::Appearance, Panel::Keybinds]);

    // Components for each tab
    let mut tabs = TabsState::new(vec![
        "General".to_string(),
        "Appearance".to_string(),
        "Keybinds".to_string(),
    ]);
    let mut input = InputFieldState::new(); // General tab
    let mut radio = RadioGroupState::new(vec![
        // Appearance tab
        "Light".to_string(),
        "Dark".to_string(),
        "System".to_string(),
    ]);
    let mut checkbox = CheckboxState::new("Vim Mode"); // Keybinds tab

    // Start on General tab, type a name
    assert_eq!(focus.focused(), Some(&Panel::General));
    for c in "MyApp".chars() {
        InputField::update(&mut input, InputFieldMessage::Insert(c));
    }
    assert_eq!(input.value(), "MyApp");

    // Switch to Appearance tab
    Tabs::<String>::update(&mut tabs, TabsMessage::Right);
    assert_eq!(tabs.selected_index(), Some(1));
    focus.focus_next();
    // Select "Dark" theme
    RadioGroup::<String>::update(&mut radio, RadioGroupMessage::Down);
    let output = RadioGroup::<String>::update(&mut radio, RadioGroupMessage::Confirm);
    assert_eq!(
        output,
        Some(RadioGroupOutput::Confirmed("Dark".to_string()))
    );
    assert_eq!(radio.selected_index(), Some(1));

    // Switch to Keybinds tab
    Tabs::<String>::update(&mut tabs, TabsMessage::Right);
    focus.focus_next();
    // Toggle vim mode
    let output = Checkbox::update(&mut checkbox, CheckboxMessage::Toggle);
    assert_eq!(output, Some(CheckboxOutput::Toggled(true)));
    assert!(checkbox.is_checked());

    // Switch back to General tab — verify state preserved
    Tabs::<String>::update(&mut tabs, TabsMessage::Left);
    Tabs::<String>::update(&mut tabs, TabsMessage::Left);
    assert_eq!(tabs.selected_index(), Some(0));
    focus.focus_first();

    // All states preserved
    assert_eq!(input.value(), "MyApp");
    assert_eq!(radio.selected_index(), Some(1));
    assert!(checkbox.is_checked());
}

// ---------------------------------------------------------------------------
// Test 6: Master-detail with dialog confirmation
// ---------------------------------------------------------------------------

#[test]
fn test_master_detail_with_dialog_confirmation() {
    let mut list = SelectableListState::new(vec![
        "Document A".to_string(),
        "Document B".to_string(),
        "Document C".to_string(),
        "Document D".to_string(),
        "Document E".to_string(),
    ]);
    // Navigate to item 2
    SelectableList::<String>::update(&mut list, SelectableListMessage::Down);
    SelectableList::<String>::update(&mut list, SelectableListMessage::Down);
    assert_eq!(list.selected_index(), Some(2));
    assert_eq!(list.selected_item(), Some(&"Document C".to_string()));

    // Open a confirmation dialog for deletion
    let mut dialog = DialogState::confirm("Delete?", "Delete Document C permanently?");
    Dialog::update(&mut dialog, DialogMessage::Open);
    assert!(Dialog::is_visible(&dialog));
    // Navigate to Cancel and press — should close dialog without change
    Dialog::update(&mut dialog, DialogMessage::FocusPrev);
    let output = Dialog::update(&mut dialog, DialogMessage::Press);
    assert_eq!(output, Some(DialogOutput::ButtonPressed("cancel".into())));
    assert!(!Dialog::is_visible(&dialog));

    // List still has 5 items, selection preserved
    assert_eq!(list.items().len(), 5);
    assert_eq!(list.selected_index(), Some(2));

    // Re-open dialog, this time confirm
    Dialog::update(&mut dialog, DialogMessage::Open);
    assert_eq!(dialog.focused_button(), 1); // OK is primary
    let output = Dialog::update(&mut dialog, DialogMessage::Press);
    assert_eq!(output, Some(DialogOutput::ButtonPressed("ok".into())));

    // Simulate deletion: rebuild list without item 2
    let remaining: Vec<String> = list
        .items()
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != 2)
        .map(|(_, item)| item.clone())
        .collect();
    let mut list = SelectableListState::new(remaining);
    assert_eq!(list.items().len(), 4);
    // Selection should be at the new index 2 (was "Document D")
    SelectableList::<String>::update(&mut list, SelectableListMessage::Down);
    SelectableList::<String>::update(&mut list, SelectableListMessage::Down);
    assert_eq!(list.selected_item(), Some(&"Document D".to_string()));
}

// ---------------------------------------------------------------------------
// Test 7: Breadcrumb navigation coordinated with tabs
// ---------------------------------------------------------------------------

#[test]
fn test_breadcrumb_tab_navigation_coordination() {
    let mut tabs = TabsState::new(vec![
        "Dashboard".to_string(),
        "Settings".to_string(),
        "Help".to_string(),
    ]);

    // Breadcrumb reflects current location
    let mut breadcrumb = BreadcrumbState::new(vec![
        BreadcrumbSegment::new("Home"),
        BreadcrumbSegment::new("Dashboard"),
    ]);

    // Initially on Dashboard
    assert_eq!(tabs.selected_index(), Some(0));
    assert_eq!(breadcrumb.segments().len(), 2);

    // Switch tab to Settings (Right emits SelectionChanged with new index)
    let output = Tabs::<String>::update(&mut tabs, TabsMessage::Right);
    assert_eq!(output, Some(TabsOutput::SelectionChanged(1)));
    assert_eq!(tabs.selected_index(), Some(1));

    // Update breadcrumb to reflect
    breadcrumb = BreadcrumbState::new(vec![
        BreadcrumbSegment::new("Home"),
        BreadcrumbSegment::new("Settings"),
    ]);

    // Navigate breadcrumb to "Home" (first segment)
    Breadcrumb::update(&mut breadcrumb, BreadcrumbMessage::First);
    let output = Breadcrumb::update(&mut breadcrumb, BreadcrumbMessage::Select);
    assert_eq!(output, Some(BreadcrumbOutput::Selected(0)));

    // This would trigger tab reset to Dashboard in a real app
    Tabs::<String>::update(&mut tabs, TabsMessage::Left);
    assert_eq!(tabs.selected_index(), Some(0));

    breadcrumb = BreadcrumbState::new(vec![
        BreadcrumbSegment::new("Home"),
        BreadcrumbSegment::new("Dashboard"),
    ]);
    assert_eq!(breadcrumb.segments().len(), 2);
}

// ---------------------------------------------------------------------------
// Test 8: SearchableList filter, navigate, select, clear workflow
// ---------------------------------------------------------------------------

#[test]
fn test_searchable_list_filter_and_select_workflow() {
    let items = vec![
        "Apple".to_string(),
        "Apricot".to_string(),
        "Banana".to_string(),
        "Blueberry".to_string(),
        "Cherry".to_string(),
        "Cranberry".to_string(),
        "Date".to_string(),
        "Elderberry".to_string(),
        "Fig".to_string(),
        "Grape".to_string(),
    ];
    let mut state = SearchableListState::new(items.clone());
    // Verify all items visible initially
    assert_eq!(state.items().len(), 10);
    assert_eq!(state.filtered_items().len(), 10);

    // Type "Apri" to filter (case-insensitive substring)
    for c in "Apri".chars() {
        SearchableList::<String>::update(&mut state, SearchableListMessage::FilterChar(c));
    }

    // Should filter to "Apricot" only
    assert_eq!(state.filtered_items().len(), 1);
    assert_eq!(state.filter_text(), "Apri");

    // Select — should return "Apricot"
    let output = SearchableList::<String>::update(&mut state, SearchableListMessage::Select);
    assert_eq!(
        output,
        Some(SearchableListOutput::Selected("Apricot".to_string()))
    );

    // Clear filter — all items restored
    SearchableList::<String>::update(&mut state, SearchableListMessage::FilterClear);
    assert_eq!(state.filtered_items().len(), 10);
    assert_eq!(state.filter_text(), "");
}
