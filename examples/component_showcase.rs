//! Component Showcase - Demonstrating 18+ Envision components in a single application.
//!
//! This example shows how to compose multiple components together into a cohesive
//! application using The Elm Architecture (TEA) pattern. It demonstrates:
//!
//! - **Event dispatch**: Using `dispatch_event` to route events to the focused component
//! - **EventContext**: Using `EventContext::new().focused(true)` to pass focus state
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
//! - Sparkline (compact data trend)
//! - Gauge (visual measurement)
//! - Heatmap (2D color grid)
//! - Timeline (event/span visualization)
//! - CommandPalette (fuzzy finder)
//! - CodeBlock (syntax highlighting)
//!
//! Run with: `cargo run --example component_showcase --features full`

use envision::component::code_block::highlight::Language;
use envision::component::{
    ButtonState, CheckboxState, CodeBlock, CodeBlockState, Column, CommandPalette,
    CommandPaletteState, Component, Dialog, DialogMessage, DialogOutput, DialogState, FocusManager,
    Gauge, GaugeState, GaugeVariant, Heatmap, HeatmapState, InputFieldState, MenuItem, MenuOutput,
    MenuState, PaletteItem, ProgressBarState, RadioGroupState, SelectableListState, Sparkline,
    SparklineState, Spinner, SpinnerMessage, SpinnerState, Table, TableOutput, TableRow,
    TableState, Tabs, TabsState, Timeline, TimelineEvent, TimelineSpan, TimelineState, Toast,
    ToastMessage, ToastState,
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
    Heatmap,
    Timeline,
    CommandPalette,
    CodeBlock,
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
    fn cells(&self) -> Vec<Cell> {
        vec![
            Cell::new(&self.name),
            Cell::new(&self.role),
            Cell::new(&self.status),
        ]
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
    Viz,
}

impl std::fmt::Display for Panel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Panel::Form => write!(f, "Form"),
            Panel::Data => write!(f, "Data"),
            Panel::Status => write!(f, "Status"),
            Panel::Viz => write!(f, "Viz"),
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

    // Viz panel
    sparkline: SparklineState,
    gauge: GaugeState,
    heatmap: HeatmapState,
    timeline: TimelineState,
    command_palette: CommandPaletteState,
    code_block: CodeBlockState,

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
            FocusId::Heatmap,
            FocusId::Timeline,
            FocusId::CommandPalette,
            FocusId::CodeBlock,
        ]);
        focus.focus(&FocusId::Tabs);

        let tabs = TabsState::new(vec![Panel::Form, Panel::Data, Panel::Status, Panel::Viz]);

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

        // Viz panel components
        let sparkline = SparklineState::with_data(vec![
            2.0, 5.0, 8.0, 12.0, 7.0, 4.0, 9.0, 15.0, 11.0, 6.0, 3.0, 8.0, 10.0, 14.0, 9.0, 5.0,
        ])
        .with_title("Request Rate");

        let gauge = GaugeState::new(73.0, 100.0)
            .with_label("CPU Usage")
            .with_units("%")
            .with_variant(GaugeVariant::Full);

        let heatmap = HeatmapState::with_data(vec![
            vec![1.0, 3.0, 5.0, 2.0, 7.0],
            vec![4.0, 6.0, 2.0, 8.0, 3.0],
            vec![2.0, 1.0, 9.0, 4.0, 6.0],
        ])
        .with_row_labels(vec!["Mon".into(), "Tue".into(), "Wed".into()])
        .with_col_labels(vec![
            "00:00".into(),
            "06:00".into(),
            "12:00".into(),
            "18:00".into(),
            "24:00".into(),
        ])
        .with_title("Error Rate by Day/Hour");

        let timeline = TimelineState::new()
            .with_events(vec![
                TimelineEvent::new("e1", 100.0, "Deploy v2.1"),
                TimelineEvent::new("e2", 450.0, "Alert fired"),
                TimelineEvent::new("e3", 800.0, "Resolved"),
            ])
            .with_spans(vec![
                TimelineSpan::new("s1", 100.0, 300.0, "Build"),
                TimelineSpan::new("s2", 300.0, 700.0, "Test"),
                TimelineSpan::new("s3", 700.0, 900.0, "Deploy"),
            ])
            .with_view_range(0.0, 1000.0)
            .with_title("CI/CD Pipeline");

        let command_palette = CommandPaletteState::new(vec![
            PaletteItem::new("open", "Open File"),
            PaletteItem::new("save", "Save File"),
            PaletteItem::new("quit", "Quit Application"),
            PaletteItem::new("find", "Find in Files"),
            PaletteItem::new("replace", "Find and Replace"),
            PaletteItem::new("settings", "Open Settings"),
        ])
        .with_title("Command Palette")
        .with_visible(true);

        let code_block = CodeBlockState::new()
            .with_code(
                "fn main() {\n    let data = vec![1, 2, 3];\n    for item in &data {\n        println!(\"{item}\");\n    }\n}",
            )
            .with_language(Language::Rust)
            .with_line_numbers(true)
            .with_title("Example Code");

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
            sparkline,
            gauge,
            heatmap,
            timeline,
            command_palette,
            code_block,
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
                    if let Some(output) = Dialog::dispatch_event(
                        &mut state.dialog,
                        &event,
                        &EventContext::new().focused(true),
                    ) {
                        handle_dialog_output(state, output);
                    }
                    return Command::none();
                }

                // Route to focused component
                let focused = state.focus.focused().cloned();
                match focused {
                    Some(FocusId::Menu) => {
                        if let Some(MenuOutput::Selected(idx)) = Menu::dispatch_event(
                            &mut state.menu,
                            &event,
                            &EventContext::new().focused(true),
                        ) {
                            let label = state.menu.items()[idx].label();
                            push_toast(
                                &mut state.toast,
                                format!("Menu: {label} clicked"),
                                envision::component::ToastLevel::Info,
                            );
                        }
                    }
                    Some(FocusId::Tabs) => {
                        Tabs::dispatch_event(
                            &mut state.tabs,
                            &event,
                            &EventContext::new().focused(true),
                        );
                    }
                    Some(FocusId::Input) => {
                        InputField::dispatch_event(
                            &mut state.input,
                            &event,
                            &EventContext::new().focused(true),
                        );
                    }
                    Some(FocusId::Checkbox) => {
                        Checkbox::dispatch_event(
                            &mut state.checkbox,
                            &event,
                            &EventContext::new().focused(true),
                        );
                    }
                    Some(FocusId::Radio) => {
                        RadioGroup::<String>::dispatch_event(
                            &mut state.radio,
                            &event,
                            &EventContext::new().focused(true),
                        );
                    }
                    Some(FocusId::SubmitButton) => {
                        let output = Button::dispatch_event(
                            &mut state.submit_button,
                            &event,
                            &EventContext::new().focused(true),
                        );
                        if output.is_some() {
                            Dialog::update(&mut state.dialog, DialogMessage::Open);
                        }
                    }
                    Some(FocusId::List) => {
                        SelectableList::dispatch_event(
                            &mut state.list,
                            &event,
                            &EventContext::new().focused(true),
                        );
                    }
                    Some(FocusId::Table) => {
                        if let Some(TableOutput::Selected(row)) = Table::dispatch_event(
                            &mut state.table,
                            &event,
                            &EventContext::new().focused(true),
                        ) {
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
                    Some(FocusId::Heatmap) => {
                        Heatmap::dispatch_event(
                            &mut state.heatmap,
                            &event,
                            &EventContext::new().focused(true),
                        );
                    }
                    Some(FocusId::Timeline) => {
                        Timeline::dispatch_event(
                            &mut state.timeline,
                            &event,
                            &EventContext::new().focused(true),
                        );
                    }
                    Some(FocusId::CommandPalette) => {
                        CommandPalette::dispatch_event(
                            &mut state.command_palette,
                            &event,
                            &EventContext::new().focused(true),
                        );
                    }
                    Some(FocusId::CodeBlock) => {
                        CodeBlock::dispatch_event(
                            &mut state.code_block,
                            &event,
                            &EventContext::new().focused(true),
                        );
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
        envision::component::Menu::view(
            &state.menu,
            &mut RenderContext::new(frame, main_chunks[0], &theme),
        );

        // Tabs
        Tabs::view(
            &state.tabs,
            &mut RenderContext::new(frame, main_chunks[1], &theme),
        );

        // Content panel based on selected tab
        let content_area = main_chunks[2];
        match state.tabs.selected_item() {
            Some(Panel::Form) => render_form_panel(state, frame, content_area, &theme),
            Some(Panel::Data) => render_data_panel(state, frame, content_area, &theme),
            Some(Panel::Status) => render_status_panel(state, frame, content_area, &theme),
            Some(Panel::Viz) => render_viz_panel(state, frame, content_area, &theme),
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
            Dialog::view(
                &state.dialog,
                &mut RenderContext::new(frame, dialog_area, &theme),
            );
        }
    }

    fn handle_event(event: &Event) -> Option<Msg> {
        use envision::input::Key;

        if let Some(key) = event.as_key() {
            match key.code {
                Key::Char('q') | Key::Esc => Some(Msg::Quit),
                Key::Tab if key.modifiers.shift() => Some(Msg::FocusPrev),

                Key::Tab => Some(Msg::FocusNext),
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

    envision::component::InputField::view(
        &state.input,
        &mut RenderContext::new(frame, chunks[0], theme),
    );
    envision::component::Checkbox::view(
        &state.checkbox,
        &mut RenderContext::new(frame, chunks[1], theme),
    );
    envision::component::RadioGroup::view(
        &state.radio,
        &mut RenderContext::new(frame, chunks[2], theme),
    );
    envision::component::Button::view(
        &state.submit_button,
        &mut RenderContext::new(frame, chunks[3], theme),
    );
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
    envision::component::SelectableList::view(
        &state.list,
        &mut RenderContext::new(frame, list_inner, theme),
    );

    // Table
    Table::view(
        &state.table,
        &mut RenderContext::new(frame, chunks[1], theme),
    );
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

    envision::component::ProgressBar::view(
        &state.progress,
        &mut RenderContext::new(frame, chunks[0], theme),
    );
    Spinner::view(
        &state.spinner,
        &mut RenderContext::new(frame, chunks[1], theme),
    );
    Toast::view(
        &state.toast,
        &mut RenderContext::new(frame, chunks[2], theme),
    );
}

fn render_viz_panel(state: &State, frame: &mut Frame, area: Rect, theme: &Theme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .title("Visualization Panel");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Top row: Sparkline + Gauge | Bottom row: Heatmap + Timeline + CodeBlock
    let rows = Layout::vertical([
        Constraint::Length(5), // Sparkline + Gauge
        Constraint::Length(7), // Heatmap
        Constraint::Length(8), // Timeline
        Constraint::Min(6),    // CommandPalette + CodeBlock
    ])
    .split(inner);

    // Row 1: Sparkline (left) + Gauge (right)
    let top_cols =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(rows[0]);
    Sparkline::view(
        &state.sparkline,
        &mut RenderContext::new(frame, top_cols[0], theme),
    );
    Gauge::view(
        &state.gauge,
        &mut RenderContext::new(frame, top_cols[1], theme),
    );

    // Row 2: Heatmap (full width)
    Heatmap::view(
        &state.heatmap,
        &mut RenderContext::new(frame, rows[1], theme),
    );

    // Row 3: Timeline (full width)
    Timeline::view(
        &state.timeline,
        &mut RenderContext::new(frame, rows[2], theme),
    );

    // Row 4: CommandPalette (left) + CodeBlock (right)
    let bottom_cols =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).split(rows[3]);
    CommandPalette::view(
        &state.command_palette,
        &mut RenderContext::new(frame, bottom_cols[0], theme),
    );
    CodeBlock::view(
        &state.code_block,
        &mut RenderContext::new(frame, bottom_cols[1], theme),
    );
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Focus synchronization is now handled via EventContext in view() and handle_event().
fn sync_focus(_state: &mut State) {
    // No-op: focused/disabled state is passed via EventContext, not stored in component state.
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
    let mut vt = Runtime::<ShowcaseApp, _>::virtual_builder(80, 40).build()?;

    println!("=== Component Showcase ===\n");
    println!("Demonstrating 18 Envision components with simplified event routing.\n");

    // Initial render (Form panel)
    vt.tick()?;
    println!("--- Initial State (Form Panel) ---");
    println!("{}\n", vt.display_ansi());

    // Navigate menu via ComponentEvent
    vt.dispatch(Msg::ComponentEvent(Event::key(envision::input::Key::Right)));
    vt.dispatch(Msg::ComponentEvent(Event::key(envision::input::Key::Enter)));

    // Switch focus to Tabs, then navigate
    vt.dispatch(Msg::FocusNext);
    vt.dispatch(Msg::ComponentEvent(Event::key(envision::input::Key::Right)));
    vt.tick()?;
    println!("--- Data Panel ---");
    println!("{}\n", vt.display_ansi());

    // Focus the list, navigate it
    vt.dispatch(Msg::FocusNext); // -> Input
    vt.dispatch(Msg::FocusNext); // -> Checkbox
    vt.dispatch(Msg::FocusNext); // -> Radio
    vt.dispatch(Msg::FocusNext); // -> SubmitButton
    vt.dispatch(Msg::FocusNext); // -> List
    vt.dispatch(Msg::ComponentEvent(Event::key(envision::input::Key::Down)));
    vt.dispatch(Msg::ComponentEvent(Event::key(envision::input::Key::Down)));

    // Focus the table, select a row
    vt.dispatch(Msg::FocusNext); // -> Table
    vt.dispatch(Msg::ComponentEvent(Event::key(envision::input::Key::Down)));
    vt.dispatch(Msg::ComponentEvent(Event::key(envision::input::Key::Enter)));
    vt.tick()?;
    println!("--- Data Panel (after navigation) ---");
    println!("{}\n", vt.display_ansi());

    // Switch to Status panel
    vt.dispatch(Msg::FocusNext); // -> Progress
    vt.dispatch(Msg::FocusNext); // -> Menu (wraps)
    vt.dispatch(Msg::FocusNext); // -> Tabs
    vt.dispatch(Msg::ComponentEvent(Event::key(envision::input::Key::Right)));
    vt.dispatch(Msg::SpinnerTick);
    vt.dispatch(Msg::ToastTick);
    vt.tick()?;
    println!("--- Status Panel ---");
    println!("{}\n", vt.display_ansi());

    // Go back to Form and submit
    vt.dispatch(Msg::ComponentEvent(Event::key(envision::input::Key::Left)));
    vt.dispatch(Msg::ComponentEvent(Event::key(envision::input::Key::Left)));

    // Focus the submit button and press it
    vt.dispatch(Msg::FocusNext); // -> Input
    vt.dispatch(Msg::FocusNext); // -> Checkbox
    vt.dispatch(Msg::FocusNext); // -> Radio
    vt.dispatch(Msg::FocusNext); // -> SubmitButton
    vt.dispatch(Msg::ComponentEvent(Event::key(envision::input::Key::Enter)));
    vt.tick()?;
    println!("--- Form Panel with Dialog Overlay ---");
    println!("{}\n", vt.display_ansi());

    // Confirm dialog via dispatch_event (Tab to OK, Enter to confirm)
    vt.dispatch(Msg::ComponentEvent(Event::key(envision::input::Key::Tab)));
    vt.dispatch(Msg::ComponentEvent(Event::key(envision::input::Key::Enter)));
    vt.tick()?;
    println!("--- After Submission (toast notification) ---");
    println!("{}\n", vt.display_ansi());

    // Switch to Viz panel to show new components
    vt.dispatch(Msg::FocusNext); // -> List
    vt.dispatch(Msg::FocusNext); // -> Table
    vt.dispatch(Msg::FocusNext); // -> Progress
    vt.dispatch(Msg::FocusNext); // -> Heatmap
    vt.dispatch(Msg::FocusNext); // -> Timeline
    vt.dispatch(Msg::FocusNext); // -> CommandPalette
    vt.dispatch(Msg::FocusNext); // -> CodeBlock
    vt.dispatch(Msg::FocusNext); // -> Menu (wraps)
    vt.dispatch(Msg::FocusNext); // -> Tabs
    // Navigate right three times to reach Viz tab
    vt.dispatch(Msg::ComponentEvent(Event::key(envision::input::Key::Right)));
    vt.dispatch(Msg::ComponentEvent(Event::key(envision::input::Key::Right)));
    vt.dispatch(Msg::ComponentEvent(Event::key(envision::input::Key::Right)));
    vt.tick()?;
    println!("--- Viz Panel (Sparkline, Gauge, Heatmap, Timeline, CommandPalette, CodeBlock) ---");
    println!("{}\n", vt.display_ansi());

    println!("=== Showcase Complete ===");
    println!("This example demonstrated Menu, Tabs, InputField, Checkbox,");
    println!("RadioGroup, Button, SelectableList, Table, ProgressBar,");
    println!("Spinner, Toast, Dialog, Sparkline, Gauge, Heatmap,");
    println!("Timeline, CommandPalette, and CodeBlock components working together.");
    println!("\nKey patterns shown:");
    println!("  - Msg enum: 6 variants (was 30+)");
    println!("  - sync_focus: no turbofish needed");
    println!("  - dispatch_event: routes events to focused component directly");

    Ok(())
}
