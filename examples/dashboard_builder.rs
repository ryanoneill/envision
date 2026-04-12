#![allow(
    clippy::collapsible_if,
    clippy::single_match,
    clippy::type_complexity,
    clippy::collapsible_match
)]
// =============================================================================
// Dashboard Builder — Reference Application #2
// =============================================================================
//
// A real-time infrastructure monitoring dashboard demonstrating:
//
// - **MetricsDashboard**: Grid of counter/gauge/status widgets
// - **AlertPanel**: Threshold-based alerting with sparklines
// - **Sparkline**: Compact inline data trends
// - **Gauge**: CPU/Memory/Disk visual meters
// - **Heatmap**: Error rate by hour and day
// - **Tabs**: Switch between dashboard views
// - **StatusBar**: Live update counters
// - **Tick subscriptions**: Simulated real-time data
//
// Run: cargo run --example dashboard_builder --features full
//
// Layout:
// ┌─ Overview ─┬─ Alerts ─┬─ Heatmap ─────────────────┐
// │            │          │                            │
// │ Metrics    │ Alert    │  Error Rate by Hour × Day  │
// │ Dashboard  │ Panel    │  ██░░██░░██░░██░░██░░██░░  │
// │            │          │                            │
// ├────────────┴──────────┴───────────────────────────┤
// │ ▁▂▃▄▅▆▇█ CPU   ████░░░░ Mem   ██████░░ Disk      │
// ├───────────────────────────────────────────────────┤
// │ Updated: 42 │ Alerts: 1 WARN │ Tab: switch        │
// └───────────────────────────────────────────────────┘

use envision::prelude::*;

// ---------------------------------------------------------------------------
// Tab identifiers
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
enum DashTab {
    Overview,
    Alerts,
    Heatmap,
}

// ---------------------------------------------------------------------------
// App message
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
enum Msg {
    Metrics(MetricsDashboardMessage),
    Alert(AlertPanelMessage),
    Heatmap(HeatmapMessage),
    Tab(TabsMessage),
    Tick,
    Quit,
}

// ---------------------------------------------------------------------------
// App state
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct State {
    metrics: MetricsDashboardState,
    alerts: AlertPanelState,
    heatmap: HeatmapState,
    tabs: TabsState<DashTab>,
    status: StatusBarState,
    // Gauges rendered separately in the bottom strip
    cpu_gauge: GaugeState,
    mem_gauge: GaugeState,
    disk_gauge: GaugeState,
    cpu_spark: SparklineState,
    tick_count: u64,
}

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

struct Dashboard;

impl App for Dashboard {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        // Metrics dashboard widgets
        let widgets = vec![
            MetricWidget::counter("Requests/s", 1250),
            MetricWidget::counter("Active Conns", 89),
            MetricWidget::gauge("CPU", 45, 100),
            MetricWidget::gauge("Memory", 62, 100),
            MetricWidget::status("API", true),
            MetricWidget::status("Database", true),
            MetricWidget::text("Region", "us-east-1"),
            MetricWidget::counter("Errors (5m)", 3),
        ];
        let metrics = MetricsDashboardState::new(widgets, 4).with_title("Infrastructure");

        // Alert panel
        let alerts = AlertPanelState::new()
            .with_metrics(vec![
                AlertMetric::new("cpu", "CPU Usage", AlertThreshold::new(70.0, 90.0))
                    .with_value(45.0),
                AlertMetric::new("mem", "Memory", AlertThreshold::new(80.0, 95.0)).with_value(62.0),
                AlertMetric::new("disk", "Disk I/O", AlertThreshold::new(50.0, 80.0))
                    .with_units("MB/s")
                    .with_value(28.0),
                AlertMetric::new("errors", "Error Rate", AlertThreshold::new(1.0, 5.0))
                    .with_value(0.3),
            ])
            .with_columns(2)
            .with_title("Alerts")
            .with_show_sparklines(true);

        // Heatmap: error rate by hour (rows) × day (cols)
        let hours: Vec<String> = (0..6).map(|h| format!("{:02}h", h * 4)).collect();
        let days: Vec<String> = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let heatmap_data = vec![
            vec![0.1, 0.2, 0.3, 0.1, 0.4, 0.0, 0.1],
            vec![0.5, 0.3, 0.8, 0.2, 0.6, 0.1, 0.2],
            vec![1.2, 0.9, 2.1, 0.7, 1.5, 0.3, 0.4],
            vec![0.8, 0.6, 1.4, 0.5, 1.1, 0.2, 0.3],
            vec![0.3, 0.2, 0.5, 0.2, 0.4, 0.1, 0.1],
            vec![0.1, 0.1, 0.2, 0.1, 0.2, 0.0, 0.1],
        ];
        let heatmap = HeatmapState::with_data(heatmap_data)
            .with_row_labels(hours)
            .with_col_labels(days)
            .with_title("Error Rate (%) by Hour × Day")
            .with_show_values(true);

        // Tabs for switching views
        let tabs = TabsState::new(vec![DashTab::Overview, DashTab::Alerts, DashTab::Heatmap]);

        // Inline gauges
        let cpu_gauge = GaugeState::new(45.0, 100.0)
            .with_label("CPU")
            .with_variant(GaugeVariant::Line);
        let mem_gauge = GaugeState::new(62.0, 100.0)
            .with_label("Memory")
            .with_variant(GaugeVariant::Line);
        let disk_gauge = GaugeState::new(38.0, 100.0)
            .with_label("Disk")
            .with_variant(GaugeVariant::Line);

        let cpu_spark =
            SparklineState::with_data(vec![30.0, 35.0, 42.0, 38.0, 45.0, 50.0, 48.0, 45.0]);

        // Status bar
        let mut status = StatusBarState::new();
        status.set_left(vec![StatusBarItem::new("Updated: 0")]);
        status.set_center(vec![StatusBarItem::new("Alerts: 0 OK")]);
        status.set_right(vec![StatusBarItem::new("Tab/←→: switch │ q: quit")]);

        let state = State {
            metrics,
            alerts,
            heatmap,
            tabs,
            status,
            cpu_gauge,
            mem_gauge,
            disk_gauge,
            cpu_spark,
            tick_count: 0,
        };

        (state, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Metrics(m) => {
                MetricsDashboard::update(&mut state.metrics, m);
            }
            Msg::Alert(m) => {
                if let Some(output) = AlertPanel::update(&mut state.alerts, m) {
                    match output {
                        AlertPanelOutput::StateChanged {
                            id,
                            old: _,
                            new_state,
                        } => {
                            // Update status bar with alert state change
                            let alert_summary = format!(
                                "Alerts: {} OK, {} WARN, {} CRIT",
                                state.alerts.ok_count(),
                                state.alerts.warning_count(),
                                state.alerts.critical_count(),
                            );
                            state
                                .status
                                .set_center(vec![StatusBarItem::new(alert_summary)]);
                            let _ = (id, new_state); // suppress warnings
                        }
                        _ => {}
                    }
                }
            }
            Msg::Heatmap(m) => {
                Heatmap::update(&mut state.heatmap, m);
            }
            Msg::Tab(m) => {
                Tabs::<DashTab>::update(&mut state.tabs, m);
            }
            Msg::Tick => {
                state.tick_count += 1;
                simulate_data_update(state);
                state.status.set_left(vec![StatusBarItem::new(format!(
                    "Updated: {}",
                    state.tick_count
                ))]);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();

        // Layout: tabs + main content + gauge strip + status bar
        let chunks = Layout::vertical([
            Constraint::Length(1), // Tab bar
            Constraint::Min(0),    // Main content
            Constraint::Length(1), // Gauge strip
            Constraint::Length(1), // Status bar
        ])
        .split(area);

        // Tab bar
        Tabs::<DashTab>::view(
            &state.tabs,
            &mut RenderContext::new(frame, chunks[0], &theme).focused(true),
        );

        // Main content based on active tab
        let active_tab = state.tabs.selected_item();
        match active_tab {
            Some(DashTab::Overview) => {
                MetricsDashboard::view(
                    &state.metrics,
                    &mut RenderContext::new(frame, chunks[1], &theme).focused(true),
                );
            }
            Some(DashTab::Alerts) => {
                AlertPanel::view(
                    &state.alerts,
                    &mut RenderContext::new(frame, chunks[1], &theme).focused(true),
                );
            }
            Some(DashTab::Heatmap) => {
                Heatmap::view(
                    &state.heatmap,
                    &mut RenderContext::new(frame, chunks[1], &theme).focused(true),
                );
            }
            None => {
                MetricsDashboard::view(
                    &state.metrics,
                    &mut RenderContext::new(frame, chunks[1], &theme),
                );
            }
        }

        // Gauge strip: three inline gauges side by side
        let gauge_chunks = Layout::horizontal([
            Constraint::Length(4), // Sparkline
            Constraint::Percentage(32),
            Constraint::Percentage(32),
            Constraint::Percentage(32),
        ])
        .split(chunks[2]);

        Sparkline::view(
            &state.cpu_spark,
            &mut RenderContext::new(frame, gauge_chunks[0], &theme),
        );
        Gauge::view(
            &state.cpu_gauge,
            &mut RenderContext::new(frame, gauge_chunks[1], &theme),
        );
        Gauge::view(
            &state.mem_gauge,
            &mut RenderContext::new(frame, gauge_chunks[2], &theme),
        );
        Gauge::view(
            &state.disk_gauge,
            &mut RenderContext::new(frame, gauge_chunks[3], &theme),
        );

        // Status bar
        StatusBar::view(
            &state.status,
            &mut RenderContext::new(frame, chunks[3], &theme),
        );
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        let key = event.as_key()?;

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Some(Msg::Quit),
            KeyCode::Left | KeyCode::Char('h') => Some(Msg::Tab(TabsMessage::Left)),
            KeyCode::Right | KeyCode::Char('l') => Some(Msg::Tab(TabsMessage::Right)),
            _ => {
                // Route to active tab's component
                let active_tab = state.tabs.selected_item();
                match active_tab {
                    Some(DashTab::Overview) => MetricsDashboard::handle_event(
                        &state.metrics,
                        event,
                        &EventContext::default(),
                    )
                    .map(Msg::Metrics),
                    Some(DashTab::Alerts) => {
                        AlertPanel::handle_event(&state.alerts, event, &EventContext::default())
                            .map(Msg::Alert)
                    }
                    Some(DashTab::Heatmap) => {
                        Heatmap::handle_event(&state.heatmap, event, &EventContext::default())
                            .map(Msg::Heatmap)
                    }
                    None => None,
                }
            }
        }
    }

    fn handle_event(_event: &Event) -> Option<Msg> {
        None
    }

    fn on_tick(_state: &State) -> Option<Msg> {
        Some(Msg::Tick)
    }
}

// ---------------------------------------------------------------------------
// Simulated data updates
// ---------------------------------------------------------------------------

fn simulate_data_update(state: &mut State) {
    let t = state.tick_count;

    // Simulate varying CPU (40-80 range with some noise)
    let cpu = 45.0 + 20.0 * ((t as f64 * 0.3).sin()) + (t as f64 * 0.7).cos() * 10.0;
    let cpu = cpu.clamp(5.0, 98.0);

    // Update gauges
    state.cpu_gauge.set_value(cpu);
    state
        .mem_gauge
        .set_value(60.0 + (t as f64 * 0.1).sin() * 15.0);
    state
        .disk_gauge
        .set_value(35.0 + (t as f64 * 0.05).sin() * 10.0);

    // Update sparkline with CPU values
    Sparkline::update(&mut state.cpu_spark, SparklineMessage::Push(cpu));

    // Update alert metrics
    state.alerts.update_metric("cpu", cpu);
    state
        .alerts
        .update_metric("mem", 60.0 + (t as f64 * 0.1).sin() * 15.0);
    state
        .alerts
        .update_metric("disk", 28.0 + (t as f64 * 0.2).cos() * 8.0);
    state
        .alerts
        .update_metric("errors", 0.3 + (t as f64 * 0.05).sin().abs() * 2.0);

    // Update metrics dashboard widgets by index
    // (Rough edge: no update-by-label API — must track indices manually)
    // Index 0: Requests/s, 1: Active Conns, 2: CPU gauge, 3: Memory gauge
    let requests = 1250 + (t as i64 * 7) % 200;
    if let Some(w) = state.metrics.widget_mut(0) {
        w.set_counter_value(requests);
    }
    if let Some(w) = state.metrics.widget_mut(1) {
        w.set_counter_value(89 + (t as i64 % 20));
    }
    if let Some(w) = state.metrics.widget_mut(2) {
        w.set_gauge_value(cpu as u64);
    }
    if let Some(w) = state.metrics.widget_mut(3) {
        w.set_gauge_value((60.0 + (t as f64 * 0.1).sin() * 15.0) as u64);
    }
}

// ---------------------------------------------------------------------------
// Tab display
// ---------------------------------------------------------------------------

impl std::fmt::Display for DashTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DashTab::Overview => write!(f, "Overview"),
            DashTab::Alerts => write!(f, "Alerts"),
            DashTab::Heatmap => write!(f, "Heatmap"),
        }
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() -> envision::Result<()> {
    let mut vt = Runtime::<Dashboard, _>::virtual_terminal(100, 30)?;

    // Run a few ticks to populate data
    for _ in 0..10 {
        vt.tick()?;
    }

    println!("Dashboard Builder — Reference Application");
    println!("==========================================");
    println!();
    println!("{}", vt.display());
    println!();
    println!("This demonstrates:");
    println!("  - MetricsDashboard with 8 widgets in 4-column grid");
    println!("  - AlertPanel with threshold monitoring + sparklines");
    println!("  - Heatmap for error rate visualization");
    println!("  - Tabs for view switching");
    println!("  - Inline Gauge + Sparkline strip");
    println!("  - StatusBar with live update counter");
    println!("  - Tick-driven simulated data stream");

    Ok(())
}
