//! Component Showcase - Demonstrating 12+ Envision components in a single application.
//!
//! This example shows how to compose multiple components together into a cohesive
//! application using The Elm Architecture (TEA) pattern. It demonstrates:
//!
//! - **State composition**: Embedding multiple component states in one App state
//! - **Message routing**: Mapping keyboard events to the right component
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
    Button, ButtonState, Checkbox, CheckboxMessage, CheckboxState, Column, Dialog, DialogMessage,
    DialogOutput, DialogState, FocusManager, Focusable, InputField, InputFieldMessage,
    InputFieldState, Menu, MenuItem, MenuMessage, MenuOutput, MenuState, ProgressBar,
    ProgressBarState, RadioGroup, RadioGroupMessage, RadioGroupState, SelectableList,
    SelectableListMessage, SelectableListState, Spinner, SpinnerMessage, SpinnerState, Table,
    TableMessage, TableOutput, TableRow, TableState, Tabs, TabsMessage, TabsState, Toast,
    ToastMessage, ToastState, Toggleable,
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
        // Start with tabs focused
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
// Messages
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
enum Msg {
    // Navigation
    FocusNext,
    FocusPrev,
    TabLeft,
    TabRight,

    // Menu
    MenuRight,
    MenuLeft,
    MenuSelect,

    // Form
    InputChar(char),
    InputBackspace,
    CheckboxToggle,
    RadioDown,
    RadioUp,
    Submit,

    // Data
    ListDown,
    ListUp,
    ListSelect,
    TableDown,
    TableUp,
    TableSelect,

    // Status
    ProgressIncrease,
    ProgressDecrease,
    SpinnerTick,

    // Dialog
    DialogConfirm,
    DialogClose,
    DialogFocusNext,

    // Toast
    ToastDismiss,
    ToastTick,

    // App
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
        // If dialog is visible, only handle dialog messages
        if Dialog::is_visible(&state.dialog) {
            match msg {
                Msg::DialogConfirm => {
                    if let Some(output) = Dialog::update(&mut state.dialog, DialogMessage::Press) {
                        match output {
                            DialogOutput::ButtonPressed(id) if id == "ok" => {
                                state.submission_count += 1;
                                let count = state.submission_count;
                                Toast::update(
                                    &mut state.toast,
                                    ToastMessage::Push {
                                        message: format!("Form submitted! (#{count})"),
                                        level: envision::component::ToastLevel::Success,
                                        duration_ms: Some(3000),
                                    },
                                );
                            }
                            _ => {}
                        }
                    }
                }
                Msg::DialogClose => {
                    Dialog::update(&mut state.dialog, DialogMessage::Close);
                }
                Msg::DialogFocusNext => {
                    Dialog::update(&mut state.dialog, DialogMessage::FocusNext);
                }
                Msg::Quit => return Command::quit(),
                _ => {}
            }
            return Command::none();
        }

        match msg {
            // Navigation
            Msg::FocusNext => {
                state.focus.focus_next();
                sync_focus(state);
            }
            Msg::FocusPrev => {
                state.focus.focus_prev();
                sync_focus(state);
            }
            Msg::TabLeft => {
                Tabs::update(&mut state.tabs, TabsMessage::Left);
            }
            Msg::TabRight => {
                Tabs::update(&mut state.tabs, TabsMessage::Right);
            }

            // Menu
            Msg::MenuRight => {
                Menu::update(&mut state.menu, MenuMessage::Right);
            }
            Msg::MenuLeft => {
                Menu::update(&mut state.menu, MenuMessage::Left);
            }
            Msg::MenuSelect => {
                if let Some(MenuOutput::Selected(idx)) =
                    Menu::update(&mut state.menu, MenuMessage::Select)
                {
                    let label = state.menu.items()[idx].label();
                    Toast::update(
                        &mut state.toast,
                        ToastMessage::Push {
                            message: format!("Menu: {label} clicked"),
                            level: envision::component::ToastLevel::Info,
                            duration_ms: Some(2000),
                        },
                    );
                }
            }

            // Form
            Msg::InputChar(c) => {
                InputField::update(&mut state.input, InputFieldMessage::Insert(c));
            }
            Msg::InputBackspace => {
                InputField::update(&mut state.input, InputFieldMessage::Backspace);
            }
            Msg::CheckboxToggle => {
                Checkbox::update(&mut state.checkbox, CheckboxMessage::Toggle);
            }
            Msg::RadioDown => {
                RadioGroup::update(&mut state.radio, RadioGroupMessage::Down);
            }
            Msg::RadioUp => {
                RadioGroup::update(&mut state.radio, RadioGroupMessage::Up);
            }
            Msg::Submit => {
                Dialog::update(&mut state.dialog, DialogMessage::Open);
            }

            // Data
            Msg::ListDown => {
                SelectableList::<String>::update(
                    &mut state.list,
                    SelectableListMessage::Down,
                );
            }
            Msg::ListUp => {
                SelectableList::<String>::update(
                    &mut state.list,
                    SelectableListMessage::Up,
                );
            }
            Msg::ListSelect => {
                SelectableList::<String>::update(
                    &mut state.list,
                    SelectableListMessage::Select,
                );
            }
            Msg::TableDown => {
                Table::<UserRow>::update(&mut state.table, TableMessage::Down);
            }
            Msg::TableUp => {
                Table::<UserRow>::update(&mut state.table, TableMessage::Up);
            }
            Msg::TableSelect => {
                if let Some(TableOutput::Selected(row)) =
                    Table::<UserRow>::update(&mut state.table, TableMessage::Select)
                {
                    Toast::update(
                        &mut state.toast,
                        ToastMessage::Push {
                            message: format!("Selected user: {}", row.name),
                            level: envision::component::ToastLevel::Info,
                            duration_ms: Some(2000),
                        },
                    );
                }
            }

            // Status
            Msg::ProgressIncrease => {
                let current = state.progress.progress();
                state.progress.set_progress((current + 0.1).min(1.0));
            }
            Msg::ProgressDecrease => {
                let current = state.progress.progress();
                state.progress.set_progress((current - 0.1).max(0.0));
            }
            Msg::SpinnerTick => {
                Spinner::update(&mut state.spinner, SpinnerMessage::Tick);
            }

            // Toast
            Msg::ToastDismiss => {
                if let Some(item) = state.toast.toasts().first() {
                    let id = item.id();
                    Toast::update(&mut state.toast, ToastMessage::Dismiss(id));
                }
            }
            Msg::ToastTick => {
                Toast::update(&mut state.toast, ToastMessage::Tick(100));
            }

            // Dialog (handled above when visible)
            Msg::DialogConfirm | Msg::DialogClose | Msg::DialogFocusNext => {}

            Msg::Quit => return Command::quit(),
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
        Menu::view(&state.menu, frame, main_chunks[0], &theme);

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
        if Dialog::is_visible(&state.dialog) {
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

                // Tab navigation
                KeyCode::Left => Some(Msg::TabLeft),
                KeyCode::Right => Some(Msg::TabRight),

                // Component-specific
                KeyCode::Up => Some(Msg::RadioUp),
                KeyCode::Down => Some(Msg::RadioDown),
                KeyCode::Enter => Some(Msg::Submit),
                KeyCode::Char(' ') => Some(Msg::CheckboxToggle),
                KeyCode::Char(c) => Some(Msg::InputChar(c)),
                KeyCode::Backspace => Some(Msg::InputBackspace),

                _ => None,
            }
        } else {
            None
        }
    }
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

    InputField::view(&state.input, frame, chunks[0], theme);
    Checkbox::view(&state.checkbox, frame, chunks[1], theme);
    RadioGroup::view(&state.radio, frame, chunks[2], theme);
    Button::view(&state.submit_button, frame, chunks[3], theme);
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
    SelectableList::view(&state.list, frame, list_inner, theme);

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

    ProgressBar::view(&state.progress, frame, chunks[0], theme);
    Spinner::view(&state.spinner, frame, chunks[1], theme);
    Toast::view(&state.toast, frame, chunks[2], theme);
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Synchronizes focus state across all components based on the FocusManager.
fn sync_focus(state: &mut State) {
    let focused = state.focus.focused().cloned();

    Menu::set_focused(&mut state.menu, focused == Some(FocusId::Menu));
    Tabs::set_focused(&mut state.tabs, focused == Some(FocusId::Tabs));
    InputField::set_focused(&mut state.input, focused == Some(FocusId::Input));
    Checkbox::set_focused(&mut state.checkbox, focused == Some(FocusId::Checkbox));
    RadioGroup::set_focused(&mut state.radio, focused == Some(FocusId::Radio));
    Button::set_focused(&mut state.submit_button, focused == Some(FocusId::SubmitButton));
    SelectableList::<String>::set_focused(&mut state.list, focused == Some(FocusId::List));
    Table::<UserRow>::set_focused(&mut state.table, focused == Some(FocusId::Table));
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
    println!("Demonstrating 12 Envision components in a single application.\n");

    // Initial render (Form panel)
    vt.tick()?;
    println!("--- Initial State (Form Panel) ---");
    println!("{}\n", vt.display_ansi());

    // Navigate menu
    vt.dispatch(Msg::MenuRight);
    vt.dispatch(Msg::MenuSelect);

    // Switch to Data panel
    vt.dispatch(Msg::TabRight);
    vt.tick()?;
    println!("--- Data Panel ---");
    println!("{}\n", vt.display_ansi());

    // Navigate list and table
    vt.dispatch(Msg::ListDown);
    vt.dispatch(Msg::ListDown);
    vt.dispatch(Msg::ListUp);
    vt.dispatch(Msg::ListSelect);
    vt.dispatch(Msg::TableDown);
    vt.dispatch(Msg::TableDown);
    vt.dispatch(Msg::TableUp);
    vt.dispatch(Msg::TableSelect);
    vt.tick()?;
    println!("--- Data Panel (after navigation) ---");
    println!("{}\n", vt.display_ansi());

    // Switch to Status panel
    vt.dispatch(Msg::TabRight);
    vt.dispatch(Msg::SpinnerTick);
    vt.dispatch(Msg::ProgressIncrease);
    vt.dispatch(Msg::ProgressIncrease);
    vt.dispatch(Msg::ProgressDecrease);
    vt.dispatch(Msg::ToastTick);
    vt.tick()?;
    println!("--- Status Panel ---");
    println!("{}\n", vt.display_ansi());

    // Go back to Form panel and submit
    vt.dispatch(Msg::TabLeft);
    vt.dispatch(Msg::TabLeft);
    vt.dispatch(Msg::MenuLeft);
    vt.dispatch(Msg::Submit);
    vt.tick()?;
    println!("--- Form Panel with Dialog Overlay ---");
    println!("{}\n", vt.display_ansi());

    // Confirm dialog
    vt.dispatch(Msg::DialogFocusNext); // Focus OK button
    vt.dispatch(Msg::DialogConfirm);
    vt.tick()?;
    println!("--- After Submission (toast notification) ---");
    println!("{}\n", vt.display_ansi());

    // Dismiss toast
    vt.dispatch(Msg::ToastDismiss);

    // Close dialog if open
    vt.dispatch(Msg::DialogClose);
    vt.tick()?;

    println!("=== Showcase Complete ===");
    println!("This example demonstrated Menu, Tabs, InputField, Checkbox,");
    println!("RadioGroup, Button, SelectableList, Table, ProgressBar,");
    println!("Spinner, Toast, and Dialog components working together.");

    Ok(())
}
