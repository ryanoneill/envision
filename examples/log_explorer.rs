#![allow(
    clippy::collapsible_if,
    clippy::single_match,
    clippy::type_complexity,
    clippy::collapsible_match
)]
// =============================================================================
// Log Explorer — Reference Application #1
// =============================================================================
//
// A multi-pane log exploration tool demonstrating envision component integration:
//
// - **SplitPanel**: Side-by-side LogViewer + EventStream
// - **FocusManager**: Tab-key focus cycling between panes
// - **ViewContext**: Parent controls focus rendering per component
// - **CommandPalette**: Ctrl+P for action picker (overlay)
// - **StatusBar**: Live counters and mode indicators
// - **Async data**: Tick-driven simulated log stream
// - **Message routing**: Multiple components through single update()
//
// Run: cargo run --example log_explorer --features full
//
// Layout:
// ┌─ Log Viewer ──────────┬─ Event Stream ────────┐
// │ 12:00:01 INF Request   │ 12:00:01 INF api ...  │
// │ 12:00:02 WRN Slow      │ 12:00:02 WRN db ...   │
// │ 12:00:03 ERR Timeout   │ 12:00:03 ERR ...      │
// ├────────────────────────┴───────────────────────┤
// │ Logs: 42 │ Events: 18 │ Focus: LogViewer │ F1  │
// └────────────────────────────────────────────────┘

use envision::prelude::*;

// ---------------------------------------------------------------------------
// Focus identifiers
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Pane {
    LogViewer,
    EventStream,
}

// ---------------------------------------------------------------------------
// App message — wraps all child component messages
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
enum Msg {
    // Routed child messages
    Log(LogViewerMessage),
    Event(EventStreamMessage),
    Palette(CommandPaletteMessage),
    #[allow(dead_code)]
    Status(StatusBarMessage),
    Split(SplitPanelMessage),

    // App-level actions
    Tick,
    TogglePalette,
    FocusNext,
    ClearAll,
    Quit,

    // Simulated data
    SimulateLog,
    SimulateEvent,
}

// ---------------------------------------------------------------------------
// App state
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct State {
    log: LogViewerState,
    events: EventStreamState,
    palette: CommandPaletteState,
    status: StatusBarState,
    split: SplitPanelState,
    focus: FocusManager<Pane>,
    tick_count: u64,
    log_counter: u64,
    event_counter: u64,
}

// ---------------------------------------------------------------------------
// App implementation
// ---------------------------------------------------------------------------

struct LogExplorer;

impl App for LogExplorer {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        // Build command palette actions
        let palette_items = vec![
            PaletteItem::new("clear", "Clear All Logs")
                .with_shortcut("Ctrl+L")
                .with_category("Actions"),
            PaletteItem::new("focus_log", "Focus Log Viewer")
                .with_shortcut("1")
                .with_category("Navigation"),
            PaletteItem::new("focus_events", "Focus Event Stream")
                .with_shortcut("2")
                .with_category("Navigation"),
            PaletteItem::new("quit", "Quit Application")
                .with_shortcut("q")
                .with_category("Actions"),
        ];

        // Build status bar with sections
        let mut status = StatusBarState::new();
        status.set_left(vec![
            StatusBarItem::new("Logs: 0"),
            StatusBarItem::new("Events: 0"),
        ]);
        status.set_center(vec![StatusBarItem::new("Focus: LogViewer")]);
        status.set_right(vec![StatusBarItem::new(
            "Tab: switch │ Ctrl+P: commands │ q: quit",
        )]);

        let state = State {
            log: LogViewerState::new()
                .with_title("Log Viewer")
                .with_follow(true)
                .with_max_entries(500),
            events: EventStreamState::new()
                .with_title("Event Stream")
                .with_max_events(500)
                .with_auto_scroll(true),
            palette: CommandPaletteState::new(palette_items)
                .with_title("Command Palette")
                .with_placeholder("Type to search actions..."),
            status,
            split: SplitPanelState::with_ratio(SplitOrientation::Vertical, 0.5)
                .with_bounds(0.25, 0.75),
            focus: FocusManager::with_initial_focus(vec![Pane::LogViewer, Pane::EventStream]),
            tick_count: 0,
            log_counter: 0,
            event_counter: 0,
        };

        (state, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            // -- Child message routing --
            Msg::Log(m) => {
                if let Some(output) = LogViewer::update(&mut state.log, m) {
                    match output {
                        LogViewerOutput::SearchChanged(text) => {
                            // Update status bar with current search
                            if text.is_empty() {
                                state.status.set_center(vec![StatusBarItem::new(format!(
                                    "Focus: {:?}",
                                    state.focus.focused().unwrap_or(&Pane::LogViewer)
                                ))]);
                            } else {
                                state.status.set_center(vec![StatusBarItem::new(format!(
                                    "Search: {}",
                                    text
                                ))]);
                            }
                        }
                        _ => {}
                    }
                }
            }

            Msg::Event(m) => {
                if let Some(_output) = EventStream::update(&mut state.events, m) {
                    // Could react to filter changes, event additions, etc.
                }
            }

            Msg::Palette(m) => {
                if let Some(output) = CommandPalette::update(&mut state.palette, m) {
                    match output {
                        CommandPaletteOutput::Selected(item) => match item.id.as_str() {
                            "clear" => return Self::update(state, Msg::ClearAll),
                            "focus_log" => {
                                state.focus.focus(&Pane::LogViewer);
                            }
                            "focus_events" => {
                                state.focus.focus(&Pane::EventStream);
                            }
                            "quit" => return Command::quit(),
                            _ => {}
                        },
                        CommandPaletteOutput::Dismissed => {}
                        _ => {}
                    }
                }
            }

            Msg::Status(m) => {
                StatusBar::update(&mut state.status, m);
            }

            Msg::Split(m) => {
                if let Some(output) = SplitPanel::update(&mut state.split, m) {
                    match output {
                        SplitPanelOutput::FocusedFirst => {
                            state.focus.focus(&Pane::LogViewer);
                        }
                        SplitPanelOutput::FocusedSecond => {
                            state.focus.focus(&Pane::EventStream);
                        }
                        _ => {}
                    }
                }
            }

            // -- App-level actions --
            Msg::Tick => {
                state.tick_count += 1;
                // Simulate incoming data every few ticks
                if state.tick_count % 3 == 0 {
                    return Command::message(Msg::SimulateLog);
                }
                if state.tick_count % 5 == 0 {
                    return Command::message(Msg::SimulateEvent);
                }
            }

            Msg::TogglePalette => {
                if state.palette.is_visible() {
                    state.palette.dismiss();
                } else {
                    state.palette.show();
                }
            }

            Msg::FocusNext => {
                state.focus.focus_next();
                update_focus_status(state);
            }

            Msg::ClearAll => {
                state.log.clear();
                state.events.clear();
                state.log_counter = 0;
                state.event_counter = 0;
                update_counters(state);
            }

            Msg::Quit => return Command::quit(),

            // -- Simulated data --
            Msg::SimulateLog => {
                state.log_counter += 1;
                let (msg_text, level) = simulated_log_entry(state.log_counter);
                match level {
                    StatusLogLevel::Info => {
                        state.log.push_info(&msg_text);
                    }
                    StatusLogLevel::Warning => {
                        state.log.push_warning(&msg_text);
                    }
                    StatusLogLevel::Error => {
                        state.log.push_error(&msg_text);
                    }
                    StatusLogLevel::Success => {
                        state.log.push_success(&msg_text);
                    }
                }
                update_counters(state);
            }

            Msg::SimulateEvent => {
                state.event_counter += 1;
                let (msg_text, level, fields) = simulated_event(state.event_counter);
                let _id = state
                    .events
                    .push_event_with_fields(level, &msg_text, fields);
                update_counters(state);
            }
        }

        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();

        // Layout: main content area + status bar
        let chunks = Layout::vertical([
            Constraint::Min(0),    // Main content (split panel)
            Constraint::Length(1), // Status bar
        ])
        .split(frame.area());

        // Get the two pane areas from SplitPanel
        let (left_area, right_area) = state.split.layout(chunks[0]);

        // Determine focus for ViewContext
        let log_focused = state.focus.is_focused(&Pane::LogViewer);
        let event_focused = state.focus.is_focused(&Pane::EventStream);

        // Render components with ViewContext carrying focus state
        LogViewer::view(
            &state.log,
            frame,
            left_area,
            &theme,
            &ViewContext::new().focused(log_focused),
        );

        EventStream::view(
            &state.events,
            frame,
            right_area,
            &theme,
            &ViewContext::new().focused(event_focused),
        );

        // Status bar (never focused)
        StatusBar::view(
            &state.status,
            frame,
            chunks[1],
            &theme,
            &ViewContext::default(),
        );

        // Command palette renders last (overlay)
        if state.palette.is_visible() {
            CommandPalette::view(
                &state.palette,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().focused(true),
            );
        }
    }

    fn handle_event(_event: &Event) -> Option<Msg> {
        None // We use handle_event_with_state for focus-aware routing
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        let key = event.as_key()?;

        // Command palette gets priority when visible
        if state.palette.is_visible() {
            return CommandPalette::handle_event(&state.palette, event).map(Msg::Palette);
        }

        // Global shortcuts (always active)
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Some(Msg::Quit),
            KeyCode::Tab => return Some(Msg::FocusNext),
            KeyCode::Char('p') if ctrl => return Some(Msg::TogglePalette),
            KeyCode::Char('l') if ctrl => return Some(Msg::ClearAll),
            KeyCode::Char('1') => {
                return Some(Msg::Log(LogViewerMessage::ScrollToTop));
            }
            KeyCode::Char('2') => {
                return Some(Msg::Event(EventStreamMessage::ScrollToTop));
            }
            _ => {}
        }

        // Route to focused component
        if state.focus.is_focused(&Pane::LogViewer) {
            if let Some(msg) = LogViewer::handle_event(&state.log, event) {
                return Some(Msg::Log(msg));
            }
        }

        if state.focus.is_focused(&Pane::EventStream) {
            if let Some(msg) = EventStream::handle_event(&state.events, event) {
                return Some(Msg::Event(msg));
            }
        }

        // SplitPanel resize (Shift+Left/Right)
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        if shift {
            match key.code {
                KeyCode::Left => return Some(Msg::Split(SplitPanelMessage::ShrinkFirst)),
                KeyCode::Right => return Some(Msg::Split(SplitPanelMessage::GrowFirst)),
                _ => {}
            }
        }

        None
    }

    fn on_tick(_state: &State) -> Option<Msg> {
        Some(Msg::Tick)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn update_counters(state: &mut State) {
    state.status.set_left(vec![
        StatusBarItem::new(format!("Logs: {}", state.log_counter)),
        StatusBarItem::new(format!("Events: {}", state.event_counter)),
    ]);
}

fn update_focus_status(state: &mut State) {
    let focus_name = match state.focus.focused() {
        Some(Pane::LogViewer) => "LogViewer",
        Some(Pane::EventStream) => "EventStream",
        None => "None",
    };
    state
        .status
        .set_center(vec![StatusBarItem::new(format!("Focus: {}", focus_name))]);
}

fn simulated_log_entry(counter: u64) -> (String, StatusLogLevel) {
    let entries = [
        ("GET /api/users - 200 OK (12ms)", StatusLogLevel::Info),
        (
            "POST /api/orders - 201 Created (45ms)",
            StatusLogLevel::Info,
        ),
        (
            "Database query took 250ms (threshold: 100ms)",
            StatusLogLevel::Warning,
        ),
        ("GET /api/products - 200 OK (8ms)", StatusLogLevel::Info),
        (
            "Connection pool exhausted, waiting for release",
            StatusLogLevel::Error,
        ),
        ("Cache hit for user:1234 (0.5ms)", StatusLogLevel::Success),
        (
            "Rate limit approaching: 980/1000 requests",
            StatusLogLevel::Warning,
        ),
        ("GET /health - 200 OK (1ms)", StatusLogLevel::Info),
        ("SSL certificate expires in 7 days", StatusLogLevel::Warning),
        ("Worker process restarted after OOM", StatusLogLevel::Error),
    ];

    let (msg, level) = entries[(counter as usize) % entries.len()];
    (msg.to_string(), level)
}

fn simulated_event(counter: u64) -> (String, EventLevel, Vec<(String, String)>) {
    let events: Vec<(&str, EventLevel, Vec<(&str, &str)>)> = vec![
        (
            "Request received",
            EventLevel::Info,
            vec![("path", "/api/users"), ("method", "GET")],
        ),
        (
            "Database query",
            EventLevel::Debug,
            vec![("table", "users"), ("duration_ms", "12")],
        ),
        (
            "Cache miss",
            EventLevel::Warning,
            vec![("key", "session:abc"), ("ttl", "expired")],
        ),
        (
            "Response sent",
            EventLevel::Info,
            vec![("status", "200"), ("bytes", "4096")],
        ),
        (
            "Connection timeout",
            EventLevel::Error,
            vec![("host", "db-replica-2"), ("timeout_ms", "5000")],
        ),
        (
            "Auth token validated",
            EventLevel::Debug,
            vec![("user_id", "1234"), ("scope", "read")],
        ),
    ];

    let (msg, level, fields) = &events[(counter as usize) % events.len()];
    let string_fields: Vec<(String, String)> = fields
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    (msg.to_string(), level.clone(), string_fields)
}

// ---------------------------------------------------------------------------
// Entry point — virtual terminal for demonstration
// ---------------------------------------------------------------------------

fn main() -> envision::Result<()> {
    let mut vt = Runtime::<LogExplorer, _>::virtual_terminal(100, 30)?;

    // Simulate some initial data
    for _ in 0..8 {
        vt.tick()?;
    }

    println!("Log Explorer — Reference Application");
    println!("=====================================");
    println!();
    println!("{}", vt.display());
    println!();
    println!("This demonstrates:");
    println!("  - SplitPanel with LogViewer + EventStream");
    println!("  - FocusManager + ViewContext for focus routing");
    println!("  - CommandPalette overlay (Ctrl+P)");
    println!("  - StatusBar with live counters");
    println!("  - Async simulated data via on_tick");
    println!("  - Message routing from 5 components");

    Ok(())
}
