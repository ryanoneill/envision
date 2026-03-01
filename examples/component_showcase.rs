//! Component Showcase - Demonstrating 12+ Envision components in a single application.
//!
//! This example shows how to compose multiple components together into a cohesive
//! application using The Elm Architecture (TEA) pattern. It demonstrates:
//!
//! - **Event dispatch**: Using `dispatch_event` to route events to the focused component
//! - **Instance methods**: Using `state.set_focused()` instead of `Component::set_focused()`
//! - **Simplified messages**: Global hotkeys only; component events dispatched directly
//! - **Focus management**: Using `FocusManager` to coordinate keyboard focus
//! - **Output handling**: Reacting to component outputs (selections, confirmations)
//! - **Theming**: Consistent styling across all components via `Theme`
//!
//! # Components Used
//!
//! - Menu (horizontal navigation bar)
//! - Tabs (panel switching)
//! - InputField (text entry)
//! - Checkbox (boolean toggle)
//! - RadioGroup (single selection from options)
//! - Button (form submission)
//! - SelectableList (item selection with scrolling)
//! - Table (data display with columns)
//! - ProgressBar (progress indicator)
//! - Spinner (loading animation)
//! - Toast (notification popups)
//! - Dialog (modal confirmation)
//!
//! Run with: `cargo run --example component_showcase`

use envision::component::{
    ButtonState, CheckboxState, Column, Component, Dialog, DialogMessage, DialogOutput,
    DialogState, FocusManager, InputFieldState, MenuItem, MenuOutput, MenuState, ProgressBarState,
    RadioGroupState, SelectableListState, Spinner, SpinnerMessage, SpinnerState, Table,
    TableOutput, TableRow, TableState, Tabs, TabsState, Toast, ToastMessage, ToastState,
};
use envision::prelude::*;
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

// ---------------------------------------------------------------------------
// Focus IDs for FocusManager
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
enum FocusId {
    Menu,
    Tabs,
    Input,
    Checkbox,
    Radio,
    SubmitButton,
    List,
    Table,
    Progress,
}

// ---------------------------------------------------------------------------
// Table row type
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct UserRow {
    name: String,
    role: String,
    status: String,
}

impl TableRow for UserRow {
    fn cells(&self) -> Vec<String> {
        vec![self.name.clone(), self.role.clone(), self.status.clone()]
    }
}

// ---------------------------------------------------------------------------
// Application State
// ---------------------------------------------------------------------------

struct ShowcaseApp;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Panel {
    Form,
    Data,
    Status,
}

impl std::fmt::Display for Panel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Panel::Form => write!(f, "Form"),
            Panel::Data => write!(f, "Data"),
            Panel::Status => write!(f, "Status"),
        }
    }
}

#[derive(Clone)]
struct State {
    // Layout
    focus: FocusManager<FocusId>,
    tabs: TabsState<Panel>,
    menu: MenuState,

    // Form panel
    input: InputFieldState,
    checkbox: CheckboxState,
    radio: RadioGroupState<String>,
    submit_button: ButtonState,

    // Data panel
    list: SelectableListState<String>,
    table: TableState<UserRow>,

    // Status panel
    progress: ProgressBarState,
    spinner: SpinnerState,
    toast: ToastState,

    // Dialog overlay
    dialog: DialogState,

    // Tracking
    submission_count: u32,
}

impl Default for State {
    fn default() -> Self {
        let mut focus = FocusManager::with_initial_focus(vec![
            FocusId::Menu,
            FocusId::Tabs,
            FocusId::Input,
            FocusId::Checkbox,
            FocusId::Radio,
            FocusId::SubmitButton,
            FocusId::List,
            FocusId::Table,
            FocusId::Progress,
        ]);
        focus.focus(&FocusId::Tabs);

        let tabs = TabsState::new(vec![Panel::Form, Panel::Data, Panel::Status]);

        let menu = MenuState::new(vec![
            MenuItem::new("File"),
            MenuItem::new("Edit"),
            MenuItem::new("Help"),
        ]);

        let mut input = InputFieldState::new();
        input.set_placeholder("Enter your name...");

        let checkbox = CheckboxState::new("Subscribe to newsletter");

        let radio = RadioGroupState::new(vec![
            "Standard".to_string(),
            "Premium".to_string(),
            "Enterprise".to_string(),
        ]);

        let submit_button = ButtonState::new("Submit");

        let list = SelectableListState::with_items(vec![
            "Alice Johnson".to_string(),
            "Bob Smith".to_string(),
            "Carol Williams".to_string(),
            "David Brown".to_string(),
            "Eve Davis".to_string(),
        ]);

        let rows = vec![
            UserRow {
                name: "Alice".to_string(),
                role: "Admin".to_string(),
                status: "Active".to_string(),
            },
            UserRow {
                name: "Bob".to_string(),
                role: "Editor".to_string(),
                status: "Active".to_string(),
            },
            UserRow {
                name: "Carol".to_string(),
                role: "Viewer".to_string(),
                status: "Inactive".to_string(),
            },
            UserRow {
                name: "David".to_string(),
                role: "Admin".to_string(),
                status: "Active".to_string(),
            },
        ];
        let columns = vec![
            Column::new("Name", Constraint::Length(15)),
            Column::new("Role", Constraint::Length(12)),
            Column::new("Status", Constraint::Length(10)),
        ];
        let table = TableState::new(rows, columns);

        let progress = ProgressBarState::with_progress(0.35);

        let mut spinner = SpinnerState::new();
        spinner.set_label(Some("Loading data...".to_string()));

        let toast = ToastState::with_max_visible(3);

        let dialog = DialogState::confirm("Confirm Submission", "Submit the form?");

        Self {
            focus,
            tabs,
            menu,
            input,
            checkbox,
            radio,
            submit_button,
            list,
            table,
            progress,
            spinner,
            toast,
            dialog,
            submission_count: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Messages — simplified with ComponentEvent routing
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
enum Msg {
    // Global navigation
    FocusNext,
    FocusPrev,

    // Component events — routed to the focused component via dispatch_event
    ComponentEvent(Event),

    // Timer-driven (not from keyboard)
    SpinnerTick,
    ToastTick,

    // App control
    Quit,
}

// ---------------------------------------------------------------------------
// App Implementation
// ---------------------------------------------------------------------------

impl App for ShowcaseApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        (State::default(), Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Quit => return Command::quit(),

            Msg::FocusNext => {
                state.focus.focus_next();
                sync_focus(state);
            }
            Msg::FocusPrev => {
                state.focus.focus_prev();
                sync_focus(state);
            }

            Msg::ComponentEvent(event) => {
                // If dialog is visible, route events to it
                if state.dialog.is_visible() {
                    if let Some(output) = state.dialog.dispatch_event(&event) {
                        handle_dialog_output(state, output);
                    }
                    return Command::none();
                }

                // Route to focused component
                let focused = state.focus.focused().cloned();
                match focused {
                    Some(FocusId::Menu) => {
                        if let Some(MenuOutput::Selected(idx)) = state.menu.dispatch_event(&event) {
                            let label = state.menu.items()[idx].label();
                            push_toast(
                                &mut state.toast,
                                format!("Menu: {label} clicked"),
                                envision::component::ToastLevel::Info,
                            );
                        }
                    }
                    Some(FocusId::Tabs) => {
                        state.tabs.dispatch_event(&event);
                    }
                    Some(FocusId::Input) => {
                        state.input.dispatch_event(&event);
                    }
                    Some(FocusId::Checkbox) => {
                        state.checkbox.dispatch_event(&event);
                    }
                    Some(FocusId::Radio) => {
                        state.radio.dispatch_event(&event);
                    }
                    Some(FocusId::SubmitButton) => {
                        if state.submit_button.dispatch_event(&event).is_some() {
                            Dialog::update(&mut state.dialog, DialogMessage::Open);
                        }
                    }
                    Some(FocusId::List) => {
                        state.list.dispatch_event(&event);
                    }
                    Some(FocusId::Table) => {
                        if let Some(TableOutput::Selected(row)) = state.table.dispatch_event(&event)
                        {
                            push_toast(
                                &mut state.toast,
                                format!("Selected user: {}", row.name),
                                envision::component::ToastLevel::Info,
                            );
                        }
                    }
                    Some(FocusId::Progress) => {
                        // Progress bar isn't interactive via keyboard;
                        // could extend to handle +/- keys here.
                    }
                    None => {}
                }
            }

            Msg::SpinnerTick => {
                Spinner::update(&mut state.spinner, SpinnerMessage::Tick);
            }
            Msg::ToastTick => {
                Toast::update(&mut state.toast, ToastMessage::Tick(100));
            }
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();

        // Main layout: menu bar, tabs, content, status line
        let main_chunks = Layout::vertical([
            Constraint::Length(1), // Menu bar
            Constraint::Length(3), // Tabs
            Constraint::Min(10),   // Content
            Constraint::Length(3), // Status / controls
        ])
        .split(area);

        // Menu bar
        envision::component::Menu::view(&state.menu, frame, main_chunks[0], &theme);

        // Tabs
        Tabs::view(&state.tabs, frame, main_chunks[1], &theme);

        // Content panel based on selected tab
        let content_area = main_chunks[2];
        match state.tabs.selected() {
            Some(Panel::Form) => render_form_panel(state, frame, content_area, &theme),
            Some(Panel::Data) => render_data_panel(state, frame, content_area, &theme),
            Some(Panel::Status) => render_status_panel(state, frame, content_area, &theme),
            None => {}
        }

        // Controls
        let controls = Paragraph::new(Line::from(vec![
            Span::styled("[Tab]", theme.info_style()),
            Span::raw(" Focus  "),
            Span::styled("[←→]", theme.info_style()),
            Span::raw(" Navigate  "),
            Span::styled("[Enter]", theme.info_style()),
            Span::raw(" Confirm  "),
            Span::styled("[Q]", theme.error_style()),
            Span::raw(" Quit"),
        ]))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.border_style())
                .title("Controls"),
        );
        frame.render_widget(controls, main_chunks[3]);

        // Dialog overlay (rendered last, on top)
        if state.dialog.is_visible() {
            let dialog_area = centered_rect(40, 8, area);
            Dialog::view(&state.dialog, frame, dialog_area, &theme);
        }
    }

    fn handle_event(event: &Event) -> Option<Msg> {
        use crossterm::event::KeyCode;

        if let Some(key) = event.as_key() {
            match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => Some(Msg::Quit),
                KeyCode::Tab => Some(Msg::FocusNext),
                KeyCode::BackTab => Some(Msg::FocusPrev),
                // All other keys are forwarded as ComponentEvent
                _ => Some(Msg::ComponentEvent(event.clone())),
            }
        } else {
            None
        }
    }
}

// ---------------------------------------------------------------------------
// Event Output Handlers
// ---------------------------------------------------------------------------

fn handle_dialog_output(state: &mut State, output: DialogOutput) {
    match output {
        DialogOutput::ButtonPressed(id) if id == "ok" => {
            state.submission_count += 1;
            let count = state.submission_count;
            push_toast(
                &mut state.toast,
                format!("Form submitted! (#{count})"),
                envision::component::ToastLevel::Success,
            );
        }
        DialogOutput::Closed => {}
        _ => {}
    }
}

fn push_toast(toast: &mut ToastState, message: String, level: envision::component::ToastLevel) {
    Toast::update(
        toast,
        ToastMessage::Push {
            message,
            level,
            duration_ms: Some(3000),
        },
    );
}

// ---------------------------------------------------------------------------
// Panel Rendering
// ---------------------------------------------------------------------------

fn render_form_panel(state: &State, frame: &mut Frame, area: Rect, theme: &Theme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .title("Form Panel");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::vertical([
        Constraint::Length(3), // Input
        Constraint::Length(1), // Checkbox
        Constraint::Length(5), // RadioGroup
        Constraint::Length(3), // Button
    ])
    .split(inner);

    envision::component::InputField::view(&state.input, frame, chunks[0], theme);
    envision::component::Checkbox::view(&state.checkbox, frame, chunks[1], theme);
    envision::component::RadioGroup::view(&state.radio, frame, chunks[2], theme);
    envision::component::Button::view(&state.submit_button, frame, chunks[3], theme);
}

fn render_data_panel(state: &State, frame: &mut Frame, area: Rect, theme: &Theme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .title("Data Panel");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).split(inner);

    // List with its own border
    let list_block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .title("Users");
    let list_inner = list_block.inner(chunks[0]);
    frame.render_widget(list_block, chunks[0]);
    envision::component::SelectableList::view(&state.list, frame, list_inner, theme);

    // Table
    Table::view(&state.table, frame, chunks[1], theme);
}

fn render_status_panel(state: &State, frame: &mut Frame, area: Rect, theme: &Theme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .title("Status Panel");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::vertical([
        Constraint::Length(3), // Progress
        Constraint::Length(1), // Spinner
        Constraint::Min(3),    // Toast area
    ])
    .split(inner);

    envision::component::ProgressBar::view(&state.progress, frame, chunks[0], theme);
    Spinner::view(&state.spinner, frame, chunks[1], theme);
    Toast::view(&state.toast, frame, chunks[2], theme);
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Synchronizes focus state across all components using instance methods.
/// No turbofish needed — the type is inferred from the receiver.
fn sync_focus(state: &mut State) {
    let focused = state.focus.focused().cloned();

    state.menu.set_focused(focused == Some(FocusId::Menu));
    state.tabs.set_focused(focused == Some(FocusId::Tabs));
    state.input.set_focused(focused == Some(FocusId::Input));
    state
        .checkbox
        .set_focused(focused == Some(FocusId::Checkbox));
    state.radio.set_focused(focused == Some(FocusId::Radio));
    state
        .submit_button
        .set_focused(focused == Some(FocusId::SubmitButton));
    state.list.set_focused(focused == Some(FocusId::List));
    state.table.set_focused(focused == Some(FocusId::Table));
}

/// Creates a centered rectangle for dialog overlays.
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<ShowcaseApp, _>::virtual_terminal(80, 30)?;

    println!("=== Component Showcase ===\n");
    println!("Demonstrating 12 Envision components with simplified event routing.\n");

    // Initial render (Form panel)
    vt.tick()?;
    println!("--- Initial State (Form Panel) ---");
    println!("{}\n", vt.display_ansi());

    // Navigate menu via ComponentEvent
    vt.dispatch(Msg::ComponentEvent(Event::key(
        crossterm::event::KeyCode::Right,
    )));
    vt.dispatch(Msg::ComponentEvent(Event::key(
        crossterm::event::KeyCode::Enter,
    )));

    // Switch focus to Tabs, then navigate
    vt.dispatch(Msg::FocusNext);
    vt.dispatch(Msg::ComponentEvent(Event::key(
        crossterm::event::KeyCode::Right,
    )));
    vt.tick()?;
    println!("--- Data Panel ---");
    println!("{}\n", vt.display_ansi());

    // Focus the list, navigate it
    vt.dispatch(Msg::FocusNext); // -> Input
    vt.dispatch(Msg::FocusNext); // -> Checkbox
    vt.dispatch(Msg::FocusNext); // -> Radio
    vt.dispatch(Msg::FocusNext); // -> SubmitButton
    vt.dispatch(Msg::FocusNext); // -> List
    vt.dispatch(Msg::ComponentEvent(Event::key(
        crossterm::event::KeyCode::Down,
    )));
    vt.dispatch(Msg::ComponentEvent(Event::key(
        crossterm::event::KeyCode::Down,
    )));

    // Focus the table, select a row
    vt.dispatch(Msg::FocusNext); // -> Table
    vt.dispatch(Msg::ComponentEvent(Event::key(
        crossterm::event::KeyCode::Down,
    )));
    vt.dispatch(Msg::ComponentEvent(Event::key(
        crossterm::event::KeyCode::Enter,
    )));
    vt.tick()?;
    println!("--- Data Panel (after navigation) ---");
    println!("{}\n", vt.display_ansi());

    // Switch to Status panel
    vt.dispatch(Msg::FocusNext); // -> Progress
    vt.dispatch(Msg::FocusNext); // -> Menu (wraps)
    vt.dispatch(Msg::FocusNext); // -> Tabs
    vt.dispatch(Msg::ComponentEvent(Event::key(
        crossterm::event::KeyCode::Right,
    )));
    vt.dispatch(Msg::SpinnerTick);
    vt.dispatch(Msg::ToastTick);
    vt.tick()?;
    println!("--- Status Panel ---");
    println!("{}\n", vt.display_ansi());

    // Go back to Form and submit
    vt.dispatch(Msg::ComponentEvent(Event::key(
        crossterm::event::KeyCode::Left,
    )));
    vt.dispatch(Msg::ComponentEvent(Event::key(
        crossterm::event::KeyCode::Left,
    )));

    // Focus the submit button and press it
    vt.dispatch(Msg::FocusNext); // -> Input
    vt.dispatch(Msg::FocusNext); // -> Checkbox
    vt.dispatch(Msg::FocusNext); // -> Radio
    vt.dispatch(Msg::FocusNext); // -> SubmitButton
    vt.dispatch(Msg::ComponentEvent(Event::key(
        crossterm::event::KeyCode::Enter,
    )));
    vt.tick()?;
    println!("--- Form Panel with Dialog Overlay ---");
    println!("{}\n", vt.display_ansi());

    // Confirm dialog via dispatch_event (Tab to OK, Enter to confirm)
    vt.dispatch(Msg::ComponentEvent(Event::key(
        crossterm::event::KeyCode::Tab,
    )));
    vt.dispatch(Msg::ComponentEvent(Event::key(
        crossterm::event::KeyCode::Enter,
    )));
    vt.tick()?;
    println!("--- After Submission (toast notification) ---");
    println!("{}\n", vt.display_ansi());

    println!("=== Showcase Complete ===");
    println!("This example demonstrated Menu, Tabs, InputField, Checkbox,");
    println!("RadioGroup, Button, SelectableList, Table, ProgressBar,");
    println!("Spinner, Toast, and Dialog components working together.");
    println!("\nKey improvements shown:");
    println!("  - Msg enum: 6 variants (was 30+)");
    println!("  - sync_focus: no turbofish needed");
    println!("  - dispatch_event: routes events to focused component directly");

    Ok(())
}
