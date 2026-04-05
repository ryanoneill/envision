//! Dashboard Demo — interactive multi-component dashboard with live styling.
//!
//! Combines Chart, MultiProgress, StatusLog, Toast, and StatusBar in a single
//! interactive dashboard. Simulates a CI/CD pipeline with build tasks, metrics,
//! and notifications.
//!
//! Controls:
//!   Ctrl+T      Cycle through themes
//!   Space       Start/restart simulated build tasks
//!   1           Add info toast
//!   2           Add success toast
//!   3           Add warning toast
//!   4           Add error toast
//!   Up/Down     Scroll status log
//!   Esc         Quit
//!
//! Run with: cargo run --example dashboard_demo --features full

use std::time::Duration;

use envision::app::UnboundedChannelSubscription;
use envision::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

// ---------------------------------------------------------------------------
// Theme cycling
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum ActiveTheme {
    #[default]
    Default,
    Nord,
    Dracula,
    SolarizedDark,
    GruvboxDark,
    CatppuccinMocha,
}

impl ActiveTheme {
    fn name(&self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Nord => "Nord",
            Self::Dracula => "Dracula",
            Self::SolarizedDark => "Solarized Dark",
            Self::GruvboxDark => "Gruvbox Dark",
            Self::CatppuccinMocha => "Catppuccin Mocha",
        }
    }

    fn next(&self) -> Self {
        match self {
            Self::Default => Self::Nord,
            Self::Nord => Self::Dracula,
            Self::Dracula => Self::SolarizedDark,
            Self::SolarizedDark => Self::GruvboxDark,
            Self::GruvboxDark => Self::CatppuccinMocha,
            Self::CatppuccinMocha => Self::Default,
        }
    }

    fn theme(&self) -> Theme {
        match self {
            Self::Default => Theme::default(),
            Self::Nord => Theme::nord(),
            Self::Dracula => Theme::dracula(),
            Self::SolarizedDark => Theme::solarized_dark(),
            Self::GruvboxDark => Theme::gruvbox_dark(),
            Self::CatppuccinMocha => Theme::catppuccin_mocha(),
        }
    }
}

// ---------------------------------------------------------------------------
// Build task names
// ---------------------------------------------------------------------------

const BUILD_TASKS: &[(&str, &str)] = &[
    ("lint", "Lint & Format"),
    ("test", "Unit Tests"),
    ("build", "Build Release"),
    ("docker", "Docker Image"),
    ("deploy", "Deploy to Staging"),
];

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct State {
    active_theme: ActiveTheme,
    chart: ChartState,
    multi_progress: MultiProgressState,
    status_log: StatusLogState,
    toasts: ToastState,
    status_bar: StatusBarState,
    build_running: bool,
    build_count: u64,
}

impl Default for State {
    fn default() -> Self {
        // Chart: build times over last 8 runs
        let build_series = DataSeries::new(
            "Build Time (s)",
            vec![45.0, 52.0, 48.0, 41.0, 55.0, 39.0, 44.0, 50.0],
        )
        .with_color(Color::Cyan);
        let test_series = DataSeries::new(
            "Test Time (s)",
            vec![22.0, 28.0, 25.0, 20.0, 30.0, 18.0, 24.0, 26.0],
        )
        .with_color(Color::Green);
        let chart = ChartState::line(vec![build_series, test_series])
            .with_title("Pipeline History")
            .with_legend(true);

        // Multi-progress: build tasks
        let mut multi_progress = MultiProgressState::new()
            .with_title("Build Pipeline")
            .with_percentages(true);
        for (id, label) in BUILD_TASKS {
            multi_progress.add(*id, *label);
        }

        // Status log
        let mut status_log = StatusLogState::new()
            .with_title("Activity")
            .with_max_entries(30);
        status_log.info("Dashboard initialized");
        status_log.info("Press Space to start a build");

        // Toasts
        let mut toasts = ToastState::with_duration(4000);
        toasts.set_max_visible(4);

        // Status bar
        let mut status_bar = StatusBarState::new();
        status_bar.push_left(StatusBarItem::new("IDLE").with_style(StatusBarStyle::Muted));
        status_bar.push_center(StatusBarItem::new("CI/CD Dashboard"));
        status_bar.push_right(StatusBarItem::counter().with_label("Builds"));
        status_bar.push_right(StatusBarItem::elapsed_time().with_style(StatusBarStyle::Muted));

        Self {
            active_theme: ActiveTheme::default(),
            chart,
            multi_progress,
            status_log,
            toasts,
            status_bar,
            build_running: false,
            build_count: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Messages
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
enum Msg {
    CycleTheme,
    StartBuild,
    TaskStarted(String),
    TaskProgress(String, f32),
    TaskCompleted(String),
    BuildDone,
    AddToast(ToastLevel),
    Log(StatusLogMessage),
    Tick,
    Quit,
}

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

struct DashboardApp;

impl App for DashboardApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        (State::default(), Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::CycleTheme => {
                state.active_theme = state.active_theme.next();
                state
                    .status_log
                    .info(format!("Theme: {}", state.active_theme.name()));
            }
            Msg::StartBuild => {
                if state.build_running {
                    state.toasts.warning("Build already in progress");
                    return Command::none();
                }
                state.build_running = true;
                state.build_count += 1;
                state
                    .status_log
                    .info(format!("Build #{} started", state.build_count));
                state.toasts.info("Build pipeline started");

                // Reset progress items
                for (id, label) in BUILD_TASKS {
                    state.multi_progress.remove(id);
                    state.multi_progress.add(*id, *label);
                }

                // Update status bar
                StatusBar::update(
                    &mut state.status_bar,
                    StatusBarMessage::SetLeftItems(vec![
                        StatusBarItem::new("BUILDING").with_style(StatusBarStyle::Warning),
                    ]),
                );
                StatusBar::update(
                    &mut state.status_bar,
                    StatusBarMessage::SetCounter {
                        section: Section::Right,
                        index: 0,
                        value: state.build_count,
                    },
                );

                // Spawn background simulation
                // Use channel-based approach; this path is only hit in DashboardApp
                // which is not used directly. DashboardChannelApp intercepts StartBuild.
                return Command::none();
            }
            Msg::TaskStarted(id) => {
                MultiProgress::update(
                    &mut state.multi_progress,
                    MultiProgressMessage::SetStatus {
                        id: id.clone(),
                        status: ProgressItemStatus::Active,
                    },
                );
                if let Some(item) = state.multi_progress.find(&id) {
                    state.status_log.info(format!("Started: {}", item.label()));
                }
            }
            Msg::TaskProgress(id, progress) => {
                MultiProgress::update(
                    &mut state.multi_progress,
                    MultiProgressMessage::SetProgress { id, progress },
                );
            }
            Msg::TaskCompleted(id) => {
                if let Some(item) = state.multi_progress.find(&id) {
                    state
                        .status_log
                        .success(format!("Completed: {}", item.label()));
                }
                MultiProgress::update(
                    &mut state.multi_progress,
                    MultiProgressMessage::Complete(id),
                );
            }
            Msg::BuildDone => {
                state.build_running = false;
                state.toasts.success("Build pipeline completed!");
                state.status_log.success(format!(
                    "Build #{} finished successfully",
                    state.build_count
                ));

                // Add new data point to chart
                let build_time = 38.0 + (state.build_count as f64 * 3.0) % 20.0;
                let test_time = 18.0 + (state.build_count as f64 * 2.0) % 15.0;
                if let Some(series) = state.chart.series_mut().first_mut() {
                    series.push_bounded(build_time, 12);
                }
                if let Some(series) = state.chart.series_mut().get_mut(1) {
                    series.push_bounded(test_time, 12);
                }

                // Update status bar
                StatusBar::update(
                    &mut state.status_bar,
                    StatusBarMessage::SetLeftItems(vec![
                        StatusBarItem::new("IDLE").with_style(StatusBarStyle::Success),
                    ]),
                );
            }
            Msg::AddToast(level) => match level {
                ToastLevel::Info => {
                    state.toasts.info("Informational notification");
                }
                ToastLevel::Success => {
                    state.toasts.success("Operation successful!");
                }
                ToastLevel::Warning => {
                    state.toasts.warning("Warning: check configuration");
                }
                ToastLevel::Error => {
                    state.toasts.error("Error: connection timeout");
                }
            },
            Msg::Log(m) => {
                StatusLog::update(&mut state.status_log, m);
            }
            Msg::Tick => {
                // Advance toast timers
                Toast::update(&mut state.toasts, ToastMessage::Tick(250));
                // Advance status bar timer
                StatusBar::update(&mut state.status_bar, StatusBarMessage::Tick(250));
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = state.active_theme.theme();
        let area = frame.area();

        // Background
        frame.render_widget(Block::default().style(theme.normal_style()), area);

        let main_chunks = Layout::vertical([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Content
            Constraint::Length(1), // Status bar
            Constraint::Length(1), // Key hints
        ])
        .split(area);

        // Header
        render_header(state, frame, main_chunks[0], &theme);

        // Content: left (chart + progress) | right (log)
        let content_chunks =
            Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)])
                .split(main_chunks[1]);

        // Left column: chart on top, progress below
        let left_chunks =
            Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(content_chunks[0]);

        Chart::view(
            &state.chart,
            frame,
            left_chunks[0],
            &theme,
            &ViewContext::default(),
        );
        MultiProgress::view(
            &state.multi_progress,
            frame,
            left_chunks[1],
            &theme,
            &ViewContext::default(),
        );

        // Right column: status log
        StatusLog::view(
            &state.status_log,
            frame,
            content_chunks[1],
            &theme,
            &ViewContext::default(),
        );

        // Toast overlay (renders on top of everything)
        Toast::view(&state.toasts, frame, area, &theme, &ViewContext::default());

        // Status bar
        StatusBar::view(
            &state.status_bar,
            frame,
            main_chunks[2],
            &theme,
            &ViewContext::default(),
        );

        // Key hints
        render_key_hints(frame, main_chunks[3], &theme);
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            if key.code == KeyCode::Char('t') && key.modifiers.contains(KeyModifiers::CONTROL) {
                return Some(Msg::CycleTheme);
            }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Some(Msg::Quit),
                KeyCode::Char(' ') => return Some(Msg::StartBuild),
                KeyCode::Char('1') => return Some(Msg::AddToast(ToastLevel::Info)),
                KeyCode::Char('2') => return Some(Msg::AddToast(ToastLevel::Success)),
                KeyCode::Char('3') => return Some(Msg::AddToast(ToastLevel::Warning)),
                KeyCode::Char('4') => return Some(Msg::AddToast(ToastLevel::Error)),
                _ => {}
            }
        }
        // Delegate scroll to status log
        StatusLog::handle_event(&state.status_log, event, &ViewContext::new().focused(true))
            .map(Msg::Log)
    }

    fn on_tick(state: &State) -> Option<Msg> {
        // Use on_tick for toast/timer advancement
        let _ = state;
        Some(Msg::Tick)
    }
}

// ---------------------------------------------------------------------------
// Background build simulation
// ---------------------------------------------------------------------------

/// Spawns a background task that sends granular progress updates via a channel.
fn spawn_build_worker(tx: tokio::sync::mpsc::UnboundedSender<Msg>) {
    tokio::spawn(async move {
        for (id, _label) in BUILD_TASKS {
            // Start task
            if tx.send(Msg::TaskStarted(id.to_string())).is_err() {
                return;
            }

            // Simulate progress in steps
            for step in 1..=5 {
                tokio::time::sleep(Duration::from_millis(200)).await;
                let progress = step as f32 / 5.0;
                if tx
                    .send(Msg::TaskProgress(id.to_string(), progress))
                    .is_err()
                {
                    return;
                }
            }

            // Complete task
            if tx.send(Msg::TaskCompleted(id.to_string())).is_err() {
                return;
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        let _ = tx.send(Msg::BuildDone);
    });
}

// ---------------------------------------------------------------------------
// Rendering helpers
// ---------------------------------------------------------------------------

fn render_header(state: &State, frame: &mut Frame, area: Rect, theme: &Theme) {
    let status_indicator = if state.build_running {
        Span::styled(" BUILDING ", theme.warning_style())
    } else {
        Span::styled(" IDLE ", theme.success_style())
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " Dashboard Demo ",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        status_indicator,
        Span::raw(" | Theme: "),
        Span::styled(state.active_theme.name(), theme.focused_bold_style()),
        Span::raw(format!(" | Builds: {} ", state.build_count)),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme.focused_border_style()),
    );
    frame.render_widget(header, area);
}

fn render_key_hints(frame: &mut Frame, area: Rect, theme: &Theme) {
    let hints = Paragraph::new(Line::from(vec![
        Span::styled("[Ctrl+T]", theme.info_style()),
        Span::raw(" Theme  "),
        Span::styled("[Space]", theme.info_style()),
        Span::raw(" Build  "),
        Span::styled("[1-4]", theme.info_style()),
        Span::raw(" Toasts  "),
        Span::styled("[Up/Dn]", theme.info_style()),
        Span::raw(" Scroll  "),
        Span::styled("[Esc]", theme.error_style()),
        Span::raw(" Quit"),
    ]))
    .alignment(Alignment::Center)
    .style(theme.normal_style());
    frame.render_widget(hints, area);
}

// ---------------------------------------------------------------------------
// Main — with channel subscription for build updates
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> envision::Result<()> {
    // We override the StartBuild handling to use the channel-based approach
    // by creating a custom wrapper that intercepts the StartBuild command.

    let (state, _) = <DashboardApp as App>::init();
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Msg>();

    let mut runtime = TerminalRuntime::<DashboardChannelApp>::new_terminal_with_state(
        ChannelState {
            inner: state,
            build_tx: tx,
        },
        Command::none(),
    )?;

    runtime.subscribe(UnboundedChannelSubscription::new(rx));

    let _final_state = runtime.run_terminal().await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Channel-aware wrapper app
// ---------------------------------------------------------------------------

/// Wraps the dashboard state with a channel sender for build updates.
#[derive(Clone)]
struct ChannelState {
    inner: State,
    build_tx: tokio::sync::mpsc::UnboundedSender<Msg>,
}

struct DashboardChannelApp;

impl App for DashboardChannelApp {
    type State = ChannelState;
    type Message = Msg;

    fn init() -> (ChannelState, Command<Msg>) {
        // Not used — we use with_state constructor
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        (
            ChannelState {
                inner: State::default(),
                build_tx: tx,
            },
            Command::none(),
        )
    }

    fn update(state: &mut ChannelState, msg: Msg) -> Command<Msg> {
        match &msg {
            Msg::StartBuild if !state.inner.build_running => {
                // Intercept to spawn the channel-based worker
                state.inner.build_running = true;
                state.inner.build_count += 1;
                state
                    .inner
                    .status_log
                    .info(format!("Build #{} started", state.inner.build_count));
                state.inner.toasts.info("Build pipeline started");

                // Reset progress items
                for (id, label) in BUILD_TASKS {
                    state.inner.multi_progress.remove(id);
                    state.inner.multi_progress.add(*id, *label);
                }

                StatusBar::update(
                    &mut state.inner.status_bar,
                    StatusBarMessage::SetLeftItems(vec![
                        StatusBarItem::new("BUILDING").with_style(StatusBarStyle::Warning),
                    ]),
                );
                StatusBar::update(
                    &mut state.inner.status_bar,
                    StatusBarMessage::SetCounter {
                        section: Section::Right,
                        index: 0,
                        value: state.inner.build_count,
                    },
                );

                spawn_build_worker(state.build_tx.clone());
                Command::none()
            }
            _ => {
                // Delegate everything else to the inner app logic
                DashboardApp::update(&mut state.inner, msg)
            }
        }
    }

    fn view(state: &ChannelState, frame: &mut Frame) {
        DashboardApp::view(&state.inner, frame);
    }

    fn handle_event_with_state(state: &ChannelState, event: &Event) -> Option<Msg> {
        DashboardApp::handle_event_with_state(&state.inner, event)
    }

    fn on_tick(_state: &ChannelState) -> Option<Msg> {
        Some(Msg::Tick)
    }
}
