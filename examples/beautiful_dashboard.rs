//! Beautiful Dashboard - Showcasing modern TUI aesthetics with Envision.
//!
//! This example demonstrates how to build a visually stunning terminal
//! dashboard using Envision's component library and Catppuccin Mocha theme.
//!
//! Features demonstrated:
//! - Catppuccin Mocha theme with rounded borders
//! - Menu navigation with Unicode indicators
//! - Real-time chart with simulated data
//! - Metric cards with sparkline trends
//! - Progress bars with labels
//! - Contextual key hints
//! - Toast notifications
//! - Focus management across panels
//!
//! Run with: cargo run --example beautiful_dashboard --features full

use envision::component::{
    Chart, ChartState, Component, DataSeries, FocusManager, KeyHint, KeyHints, KeyHintsLayout,
    KeyHintsState, Menu, MenuItem, MenuMessage, MenuState, MetricWidget, MetricsDashboard,
    MetricsDashboardMessage, MetricsDashboardState, ProgressBar, ProgressBarState, Toast,
    ToastMessage, ToastState,
};
use envision::prelude::*;
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Padding};

// =============================================================================
// Theme colors - Catppuccin Mocha palette for direct styling
// =============================================================================

use envision::theme::{
    CATPPUCCIN_BASE, CATPPUCCIN_GREEN, CATPPUCCIN_LAVENDER, CATPPUCCIN_MAUVE, CATPPUCCIN_OVERLAY0,
    CATPPUCCIN_SAPPHIRE, CATPPUCCIN_SURFACE2,
};

// =============================================================================
// Focus management
// =============================================================================

#[derive(Clone, Debug, PartialEq, Eq)]
enum Panel {
    Navigation,
    Chart,
    Metrics,
}

// =============================================================================
// Application state
// =============================================================================

struct DashboardApp;

#[derive(Clone)]
struct State {
    // Component states
    menu: MenuState,
    chart: ChartState,
    metrics: MetricsDashboardState,
    progress_cpu: ProgressBarState,
    progress_mem: ProgressBarState,
    progress_disk: ProgressBarState,
    hints: KeyHintsState,
    toasts: ToastState,

    // Navigation
    focus: FocusManager<Panel>,

    // Simulated data
    tick_count: u64,
    cpu_history: Vec<f64>,
    mem_history: Vec<f64>,
}

impl Default for State {
    fn default() -> Self {
        // Navigation menu
        let menu = MenuState::new(vec![
            MenuItem::new("Dashboard"),
            MenuItem::new("Processes"),
            MenuItem::new("Network"),
            MenuItem::new("Alerts"),
        ]);

        // Chart with two data series
        let cpu_data: Vec<f64> = vec![
            23.0, 45.0, 67.0, 34.0, 56.0, 78.0, 45.0, 32.0, 54.0, 76.0, 43.0, 65.0, 87.0, 54.0,
            32.0, 67.0, 89.0, 45.0, 56.0, 34.0,
        ];
        let mem_data: Vec<f64> = vec![
            45.0, 46.0, 47.0, 48.0, 49.0, 50.0, 51.0, 52.0, 53.0, 54.0, 55.0, 56.0, 57.0, 58.0,
            59.0, 60.0, 61.0, 62.0, 63.0, 64.0,
        ];

        let chart = ChartState::line(vec![
            DataSeries::new("CPU %", cpu_data.clone()).with_color(CATPPUCCIN_SAPPHIRE),
            DataSeries::new("Memory %", mem_data.clone()).with_color(CATPPUCCIN_MAUVE),
        ])
        .with_title("System Performance")
        .with_y_label("%")
        .with_max_display_points(20);

        // Metrics dashboard
        let metrics = MetricsDashboardState::new(
            vec![
                MetricWidget::counter("Req/s", 1247),
                MetricWidget::text("Latency", "42ms"),
                MetricWidget::text("Uptime", "99.97%"),
                MetricWidget::text("Errors", "0.03%"),
            ],
            4,
        );

        // Progress bars
        let mut progress_cpu = ProgressBarState::with_progress(0.67);
        progress_cpu.set_label(Some("CPU".to_string()));
        let mut progress_mem = ProgressBarState::with_progress(0.54);
        progress_mem.set_label(Some("Memory".to_string()));
        let mut progress_disk = ProgressBarState::with_progress(0.82);
        progress_disk.set_label(Some("Disk".to_string()));

        // Key hints
        let hints = KeyHintsState::with_hints(vec![
            KeyHint::new("Tab", "Focus"),
            KeyHint::new("\u{2190}\u{2192}", "Navigate"),
            KeyHint::new("\u{2191}\u{2193}", "Select"),
            KeyHint::new("Enter", "Confirm"),
            KeyHint::new("q", "Quit"),
        ])
        .with_layout(KeyHintsLayout::Spaced);

        // Toasts
        let mut toasts = ToastState::with_duration(5000);
        toasts.success("System monitor started");

        // Focus manager
        let focus =
            FocusManager::with_initial_focus(vec![Panel::Navigation, Panel::Chart, Panel::Metrics]);

        Self {
            menu,
            chart,
            metrics,
            progress_cpu,
            progress_mem,
            progress_disk,
            hints,
            toasts,
            focus,
            tick_count: 0,
            cpu_history: cpu_data,
            mem_history: mem_data,
        }
    }
}

// =============================================================================
// Messages
// =============================================================================

#[derive(Clone, Debug)]
enum Msg {
    Tick,
    FocusNext,
    FocusPrev,
    Left,
    Right,
    Up,
    Down,
    Select,
    ChartNextSeries,
    Quit,
}

// =============================================================================
// App implementation
// =============================================================================

impl App for DashboardApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        (State::default(), Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Tick => {
                state.tick_count += 1;

                // Simulate CPU data
                let new_cpu = 30.0
                    + 40.0 * (state.tick_count as f64 * 0.1).sin()
                    + 10.0 * (state.tick_count as f64 * 0.3).cos();
                state.cpu_history.push(new_cpu.clamp(0.0, 100.0));
                if state.cpu_history.len() > 20 {
                    state.cpu_history.remove(0);
                }

                // Simulate memory data (slowly rising)
                let new_mem = 50.0
                    + (state.tick_count as f64 * 0.05).sin() * 15.0
                    + state.tick_count as f64 * 0.1;
                let new_mem = new_mem.clamp(0.0, 100.0);
                state.mem_history.push(new_mem);
                if state.mem_history.len() > 20 {
                    state.mem_history.remove(0);
                }

                // Update chart series
                if let Some(series) = state.chart.get_series_mut(0) {
                    *series = DataSeries::new("CPU %", state.cpu_history.clone())
                        .with_color(CATPPUCCIN_SAPPHIRE);
                }
                if let Some(series) = state.chart.get_series_mut(1) {
                    *series = DataSeries::new("Memory %", state.mem_history.clone())
                        .with_color(CATPPUCCIN_MAUVE);
                }

                // Update progress bars
                let cpu_pct = new_cpu as f32 / 100.0;
                state.progress_cpu.set_progress(cpu_pct);
                state.progress_mem.set_progress(new_mem as f32 / 100.0);

                // Slowly fill disk
                let disk = 0.82 + (state.tick_count as f32 * 0.001);
                state.progress_disk.set_progress(disk.min(0.99));

                // Update metric counters
                if let Some(w) = state.metrics.widget_mut(0) {
                    w.increment(((new_cpu * 0.5) as i64).max(1));
                }

                // Tick toasts
                Toast::update(&mut state.toasts, ToastMessage::Tick(100));

                // Trigger alerts occasionally
                if state.tick_count % 50 == 0 && state.tick_count > 0 {
                    state.toasts.warning("High CPU usage detected");
                }
            }
            Msg::FocusNext => {
                state.focus.focus_next();
                sync_focus(state);
            }
            Msg::FocusPrev => {
                state.focus.focus_prev();
                sync_focus(state);
            }
            Msg::Left => match state.focus.focused() {
                Some(Panel::Navigation) => {
                    Menu::update(&mut state.menu, MenuMessage::Left);
                }
                Some(Panel::Metrics) => {
                    MetricsDashboard::update(&mut state.metrics, MetricsDashboardMessage::Left);
                }
                _ => {}
            },
            Msg::Right => match state.focus.focused() {
                Some(Panel::Navigation) => {
                    Menu::update(&mut state.menu, MenuMessage::Right);
                }
                Some(Panel::Metrics) => {
                    MetricsDashboard::update(&mut state.metrics, MetricsDashboardMessage::Right);
                }
                _ => {}
            },
            Msg::Up => {
                if state.focus.is_focused(&Panel::Metrics) {
                    MetricsDashboard::update(&mut state.metrics, MetricsDashboardMessage::Up);
                }
            }
            Msg::Down => {
                if state.focus.is_focused(&Panel::Metrics) {
                    MetricsDashboard::update(&mut state.metrics, MetricsDashboardMessage::Down);
                }
            }
            Msg::Select => {
                if state.focus.is_focused(&Panel::Navigation) {
                    if let Some(idx) = state.menu.selected_index() {
                        let name = state.menu.items()[idx].label().to_string();
                        state.toasts.info(format!("Navigated to {}", name));
                    }
                }
            }
            Msg::ChartNextSeries => {
                // Toggle active series via chart message
                let event = Event::key(KeyCode::Tab);
                Chart::dispatch_event(&mut state.chart, &event, &ViewContext::default());
            }
            Msg::Quit => {
                return Command::quit();
            }
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::catppuccin_mocha();
        let area = frame.area();

        // Clear background
        let bg_block = Block::default().style(Style::default().bg(CATPPUCCIN_BASE));
        frame.render_widget(bg_block, area);

        // Main layout
        let main = Layout::vertical([
            Constraint::Length(3), // Title bar
            Constraint::Length(3), // Navigation menu
            Constraint::Min(8),    // Content area
            Constraint::Length(3), // Progress bars
            Constraint::Length(3), // Key hints
        ])
        .split(area);

        // ── Title Bar ──
        render_title_bar(state, frame, main[0], &theme);

        // ── Navigation Menu ──
        render_navigation(state, frame, main[1], &theme);

        // ── Content Area (Chart + Metrics) ──
        render_content(state, frame, main[2], &theme);

        // ── Progress Bars ──
        render_progress(state, frame, main[3], &theme);

        // ── Key Hints ──
        KeyHints::view(
            &state.hints,
            frame,
            main[4],
            &theme,
            &ViewContext::default(),
        );

        // ── Toast overlay (renders on top) ──
        Toast::view(&state.toasts, frame, area, &theme, &ViewContext::default());
    }

    fn handle_event(event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => Some(Msg::Quit),
                KeyCode::Tab => Some(Msg::FocusNext),
                KeyCode::BackTab => Some(Msg::FocusPrev),
                KeyCode::Left | KeyCode::Char('h') => Some(Msg::Left),
                KeyCode::Right | KeyCode::Char('l') => Some(Msg::Right),
                KeyCode::Up | KeyCode::Char('k') => Some(Msg::Up),
                KeyCode::Down | KeyCode::Char('j') => Some(Msg::Down),
                KeyCode::Enter => Some(Msg::Select),
                KeyCode::Char('s') => Some(Msg::ChartNextSeries),
                _ => None,
            }
        } else {
            None
        }
    }
}

// =============================================================================
// View helpers
// =============================================================================

fn sync_focus(_state: &mut State) {
    // No-op: focused/disabled state is passed via ViewContext, not stored in component state.
}

fn render_title_bar(state: &State, frame: &mut Frame, area: Rect, _theme: &Theme) {
    let tick_indicator = if state.tick_count % 4 == 0 {
        "\u{25cf}"
    } else if state.tick_count % 4 == 1 {
        "\u{25cb}"
    } else if state.tick_count % 4 == 2 {
        "\u{25cf}"
    } else {
        "\u{25cb}"
    };

    let title = Line::from(vec![
        Span::styled(
            " \u{2728} Envision System Monitor ",
            Style::default()
                .fg(CATPPUCCIN_LAVENDER)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(tick_indicator, Style::default().fg(CATPPUCCIN_GREEN)),
    ]);

    let status = format!(" tick: {} ", state.tick_count);
    let title_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(CATPPUCCIN_SURFACE2))
        .padding(Padding::horizontal(1))
        .title(title)
        .title_alignment(Alignment::Left)
        .title_bottom(
            Line::from(vec![Span::styled(
                status,
                Style::default().fg(CATPPUCCIN_OVERLAY0),
            )])
            .alignment(Alignment::Right),
        );

    frame.render_widget(title_block, area);
}

fn render_navigation(state: &State, frame: &mut Frame, area: Rect, theme: &Theme) {
    let is_focused = state.focus.is_focused(&Panel::Navigation);

    let border_color = if is_focused {
        CATPPUCCIN_LAVENDER
    } else {
        CATPPUCCIN_SURFACE2
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color).bg(CATPPUCCIN_BASE))
        .title(Span::styled(
            " Navigation ",
            Style::default()
                .fg(if is_focused {
                    CATPPUCCIN_LAVENDER
                } else {
                    CATPPUCCIN_OVERLAY0
                })
                .add_modifier(Modifier::BOLD),
        ))
        .padding(Padding::horizontal(1));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Render menu inside the block
    Menu::view(&state.menu, frame, inner, theme, &ViewContext::default());
}

fn render_content(state: &State, frame: &mut Frame, area: Rect, theme: &Theme) {
    let content = Layout::vertical([
        Constraint::Min(6),    // Chart
        Constraint::Length(5), // Metrics
    ])
    .split(area);

    // ── Chart Panel ──
    let chart_focused = state.focus.is_focused(&Panel::Chart);
    let chart_border = if chart_focused {
        CATPPUCCIN_LAVENDER
    } else {
        CATPPUCCIN_SURFACE2
    };

    let chart_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(chart_border).bg(CATPPUCCIN_BASE))
        .title(Span::styled(
            " CPU & Memory ",
            Style::default()
                .fg(if chart_focused {
                    CATPPUCCIN_LAVENDER
                } else {
                    CATPPUCCIN_OVERLAY0
                })
                .add_modifier(Modifier::BOLD),
        ))
        .padding(Padding::horizontal(1));

    let chart_inner = chart_block.inner(content[0]);
    frame.render_widget(chart_block, content[0]);
    Chart::view(
        &state.chart,
        frame,
        chart_inner,
        theme,
        &ViewContext::default(),
    );

    // ── Metrics Panel ──
    let metrics_focused = state.focus.is_focused(&Panel::Metrics);
    let metrics_border = if metrics_focused {
        CATPPUCCIN_LAVENDER
    } else {
        CATPPUCCIN_SURFACE2
    };

    let metrics_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(metrics_border).bg(CATPPUCCIN_BASE))
        .title(Span::styled(
            " Key Metrics ",
            Style::default()
                .fg(if metrics_focused {
                    CATPPUCCIN_LAVENDER
                } else {
                    CATPPUCCIN_OVERLAY0
                })
                .add_modifier(Modifier::BOLD),
        ))
        .padding(Padding::horizontal(1));

    let metrics_inner = metrics_block.inner(content[1]);
    frame.render_widget(metrics_block, content[1]);
    MetricsDashboard::view(
        &state.metrics,
        frame,
        metrics_inner,
        theme,
        &ViewContext::default(),
    );
}

fn render_progress(state: &State, frame: &mut Frame, area: Rect, theme: &Theme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(CATPPUCCIN_SURFACE2).bg(CATPPUCCIN_BASE))
        .title(Span::styled(
            " Resources ",
            Style::default()
                .fg(CATPPUCCIN_OVERLAY0)
                .add_modifier(Modifier::BOLD),
        ))
        .padding(Padding::horizontal(1));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let cols = Layout::horizontal([
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
    ])
    .split(inner);

    ProgressBar::view(
        &state.progress_cpu,
        frame,
        cols[0],
        theme,
        &ViewContext::default(),
    );
    ProgressBar::view(
        &state.progress_mem,
        frame,
        cols[1],
        theme,
        &ViewContext::default(),
    );
    ProgressBar::view(
        &state.progress_disk,
        frame,
        cols[2],
        theme,
        &ViewContext::default(),
    );
}

// =============================================================================
// Main
// =============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<DashboardApp, _>::virtual_terminal(80, 30)?;

    println!("=== Beautiful Dashboard ===\n");
    println!("Demonstrating Envision + Catppuccin Mocha theme.\n");

    // Initial render
    vt.tick()?;
    println!("Initial state:");
    println!("{}\n", vt.display_ansi());

    // Simulate several ticks of live data
    for _ in 0..5 {
        vt.dispatch(Msg::Tick);
    }
    vt.tick()?;
    println!("After 5 ticks (live data updates):");
    println!("{}\n", vt.display_ansi());

    // Focus navigation demo
    vt.dispatch(Msg::FocusNext);
    vt.tick()?;
    println!("Focus moved to Chart panel:");
    println!("{}\n", vt.display_ansi());

    vt.dispatch(Msg::FocusNext);
    vt.tick()?;
    println!("Focus moved to Metrics panel:");
    println!("{}\n", vt.display_ansi());

    // Menu navigation
    vt.dispatch(Msg::FocusPrev);
    vt.dispatch(Msg::FocusPrev);
    vt.dispatch(Msg::Right);
    vt.dispatch(Msg::Select);
    vt.tick()?;
    println!("Menu navigation (selected Processes):");
    println!("{}\n", vt.display_ansi());

    // More ticks
    for _ in 0..20 {
        vt.dispatch(Msg::Tick);
    }
    vt.tick()?;
    println!("After 25 ticks (data evolved):");
    println!("{}\n", vt.display_ansi());

    println!("=== Dashboard Demo Complete ===");
    println!("\nThe Catppuccin Mocha theme provides:");
    println!("  \u{2022} Lavender (#B4BEFE) focus borders for clear panel indication");
    println!("  \u{2022} Mauve (#CBA6F7) selection highlighting");
    println!("  \u{2022} Sapphire (#74C7EC) and Peach (#FAB387) data series");
    println!("  \u{2022} Rounded borders (BorderType::Rounded) for modern aesthetics");
    println!("  \u{2022} Consistent padding for breathing room");

    Ok(())
}
