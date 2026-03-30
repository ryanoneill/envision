#![cfg(feature = "full")]
//! Integration tests for new components added in Iterations 2-6 and the
//! Claude Code component suite. Each test exercises the full
//! event -> message -> update -> view cycle through the public API.

use envision::component::code_block::highlight::Language;
use envision::component::{
    // Observability
    AlertMetric,
    AlertPanel,
    AlertPanelMessage,
    AlertPanelOutput,
    AlertPanelState,
    AlertState,
    AlertThreshold,
    // General purpose
    Calendar,
    CalendarMessage,
    CalendarOutput,
    CalendarState,
    // Visualization
    Chart,
    ChartKind,
    ChartMessage,
    ChartOutput,
    ChartState,
    CodeBlock,
    CodeBlockMessage,
    CodeBlockState,
    CommandPalette,
    CommandPaletteMessage,
    CommandPaletteOutput,
    CommandPaletteState,
    // Traits
    Component,
    // Claude Code components
    ConversationMessage,
    ConversationRole,
    ConversationView,
    ConversationViewMessage,
    ConversationViewState,
    DataSeries,
    EventLevel,
    EventStream,
    EventStreamMessage,
    EventStreamState,
    FlameGraph,
    FlameGraphMessage,
    FlameGraphOutput,
    FlameGraphState,
    FlameNode,
    Focusable,
    Gauge,
    GaugeMessage,
    GaugeState,
    GaugeVariant,
    Heatmap,
    HeatmapMessage,
    HeatmapOutput,
    HeatmapState,
    HelpPanel,
    HelpPanelMessage,
    HelpPanelState,
    Histogram,
    HistogramMessage,
    HistogramState,
    KeyBinding,
    KeyBindingGroup,
    MessageBlock,
    Paginator,
    PaginatorMessage,
    PaginatorOutput,
    PaginatorState,
    PaletteItem,
    Slider,
    SliderMessage,
    SliderOutput,
    SliderState,
    SpanNode,
    SpanTree,
    SpanTreeMessage,
    SpanTreeOutput,
    SpanTreeState,
    Sparkline,
    SparklineMessage,
    SparklineState,
    Switch,
    SwitchMessage,
    SwitchOutput,
    SwitchState,
    Tab,
    TabBar,
    TabBarMessage,
    TabBarOutput,
    TabBarState,
    TerminalOutput,
    TerminalOutputMessage,
    TerminalOutputOutput,
    TerminalOutputState,
    ThresholdLine,
    ThresholdZone,
    Timeline,
    TimelineEvent,
    TimelineMessage,
    TimelineSpan,
    TimelineState,
};
use envision::CaptureBackend;
use ratatui::prelude::*;
use ratatui::Terminal;

// ============================================================================
// 1. Sparkline: push data, verify view updates
// ============================================================================

#[test]
fn test_sparkline_push_data_and_bounded_eviction() {
    let mut state = SparklineState::with_data(vec![1, 2, 3]);
    assert_eq!(state.len(), 3);
    assert_eq!(state.data(), &[1, 2, 3]);

    // Push a new data point
    Sparkline::update(&mut state, SparklineMessage::Push(4));
    assert_eq!(state.len(), 4);
    assert_eq!(state.data()[3], 4);
    assert_eq!(state.last(), Some(4));

    // Push with bounded capacity, should evict oldest
    Sparkline::update(&mut state, SparklineMessage::PushBounded(9, 3));
    assert_eq!(state.len(), 3);
    assert_eq!(state.data(), &[3, 4, 9]);
    assert_eq!(state.min(), Some(3));
    assert_eq!(state.max(), Some(9));

    // Clear all data
    Sparkline::update(&mut state, SparklineMessage::Clear);
    assert!(state.is_empty());
    assert_eq!(state.min(), None);

    // Replace data
    Sparkline::update(&mut state, SparklineMessage::SetData(vec![10, 20, 30]));
    assert_eq!(state.len(), 3);
    assert_eq!(state.last(), Some(30));

    // Verify rendering does not panic
    let backend = CaptureBackend::new(60, 5);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = envision::Theme::default();
    terminal
        .draw(|frame| {
            Sparkline::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

// ============================================================================
// 2. Gauge: set value, verify threshold colors change
// ============================================================================

#[test]
fn test_gauge_threshold_color_transitions() {
    let mut state = GaugeState::new(20.0, 100.0).with_units("%");

    // 20% -> Green (below 70%)
    assert_eq!(state.current_color(), Color::Green);
    assert_eq!(state.display_percentage(), 20);

    // Move to 75% -> Yellow (at/above 70%, below 90%)
    Gauge::update(&mut state, GaugeMessage::SetValue(75.0));
    assert_eq!(state.value(), 75.0);
    assert_eq!(state.current_color(), Color::Yellow);

    // Move to 95% -> Red (at/above 90%)
    Gauge::update(&mut state, GaugeMessage::SetValue(95.0));
    assert_eq!(state.current_color(), Color::Red);
    assert_eq!(state.display_percentage(), 95);

    // Move back to 50% -> Green
    Gauge::update(&mut state, GaugeMessage::SetValue(50.0));
    assert_eq!(state.current_color(), Color::Green);

    // Verify rendering in both variants
    for variant in [GaugeVariant::Full, GaugeVariant::Line] {
        let state = GaugeState::new(60.0, 100.0).with_variant(variant);
        let backend = CaptureBackend::new(40, 3);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = envision::Theme::default();
        terminal
            .draw(|frame| {
                Gauge::view(&state, frame, frame.area(), &theme);
            })
            .unwrap();
    }
}

// ============================================================================
// 3. Histogram: push data, verify bin computation
// ============================================================================

#[test]
fn test_histogram_push_data_and_bin_computation() {
    let mut state = HistogramState::with_data(vec![1.0, 2.0, 3.0, 4.0, 5.0])
        .with_bin_count(5)
        .with_range(1.0, 5.0);

    assert_eq!(state.data().len(), 5);
    assert_eq!(state.bin_count(), 5);

    // Compute bins and verify total count
    let bins = state.compute_bins();
    assert_eq!(bins.len(), 5);
    let total: usize = bins.iter().map(|(_, _, c)| c).sum();
    assert_eq!(total, 5);

    // Push more data via message
    Histogram::update(&mut state, HistogramMessage::PushData(3.5));
    assert_eq!(state.data().len(), 6);

    // Push a batch
    Histogram::update(&mut state, HistogramMessage::PushDataBatch(vec![2.5, 4.5]));
    assert_eq!(state.data().len(), 8);

    // Recompute bins after adding data
    let bins = state.compute_bins();
    let total: usize = bins.iter().map(|(_, _, c)| c).sum();
    assert_eq!(total, 8);

    // Change bin count
    Histogram::update(&mut state, HistogramMessage::SetBinCount(10));
    assert_eq!(state.bin_count(), 10);
    let bins = state.compute_bins();
    assert_eq!(bins.len(), 10);

    // Clear data
    Histogram::update(&mut state, HistogramMessage::Clear);
    assert!(state.data().is_empty());
}

// ============================================================================
// 4. Heatmap: set cells, navigate, verify selection
// ============================================================================

#[test]
fn test_heatmap_cell_navigation_and_selection() {
    let mut state = HeatmapState::new(3, 4);
    state.set_focused(true);

    // Initial selection at (0, 0)
    assert_eq!(state.selected(), Some((0, 0)));
    assert_eq!(state.get(0, 0), Some(0.0));

    // Set some cells
    state.set(0, 0, 1.0);
    state.set(1, 2, 0.5);
    state.set(2, 3, 0.8);
    assert_eq!(state.get(0, 0), Some(1.0));
    assert_eq!(state.get(1, 2), Some(0.5));
    assert_eq!(state.get(2, 3), Some(0.8));

    // Navigate down
    let output = Heatmap::update(&mut state, HeatmapMessage::SelectDown);
    assert_eq!(
        output,
        Some(HeatmapOutput::SelectionChanged { row: 1, col: 0 })
    );
    assert_eq!(state.selected(), Some((1, 0)));

    // Navigate right twice
    Heatmap::update(&mut state, HeatmapMessage::SelectRight);
    Heatmap::update(&mut state, HeatmapMessage::SelectRight);
    assert_eq!(state.selected(), Some((1, 2)));

    // Navigate down again
    Heatmap::update(&mut state, HeatmapMessage::SelectDown);
    assert_eq!(state.selected(), Some((2, 2)));

    // Navigate right
    Heatmap::update(&mut state, HeatmapMessage::SelectRight);
    assert_eq!(state.selected(), Some((2, 3)));

    // Navigate down at boundary - should stay at row 2
    Heatmap::update(&mut state, HeatmapMessage::SelectDown);
    assert_eq!(state.selected(), Some((2, 3)));

    // Set data via message
    Heatmap::update(
        &mut state,
        HeatmapMessage::SetCell {
            row: 0,
            col: 1,
            value: 0.75,
        },
    );
    assert_eq!(state.get(0, 1), Some(0.75));

    // Render to confirm no panic
    let backend = CaptureBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = envision::Theme::default();
    terminal
        .draw(|frame| {
            Heatmap::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

// ============================================================================
// 5. Chart (enhanced): create area/scatter chart, add threshold lines
// ============================================================================

#[test]
fn test_chart_area_scatter_with_thresholds() {
    // Create an area chart with a data series
    let series = vec![
        DataSeries::new("CPU", vec![45.0, 52.0, 48.0, 60.0, 55.0]).with_color(Color::Cyan),
        DataSeries::new("Memory", vec![30.0, 35.0, 40.0, 38.0, 42.0]).with_color(Color::Green),
    ];
    let mut state = ChartState::area(series)
        .with_title("System Metrics")
        .with_x_label("Time")
        .with_y_label("Usage %");

    assert_eq!(state.kind(), &ChartKind::Area);
    assert_eq!(state.series().len(), 2);
    assert_eq!(state.title(), Some("System Metrics"));

    // Add threshold lines
    let threshold = ThresholdLine::new(80.0, "Warning", Color::Yellow);
    Chart::update(&mut state, ChartMessage::AddThreshold(threshold));
    assert_eq!(state.thresholds().len(), 1);

    // Add a second threshold
    Chart::update(
        &mut state,
        ChartMessage::AddThreshold(ThresholdLine::new(95.0, "Critical", Color::Red)),
    );
    assert_eq!(state.thresholds().len(), 2);

    // Set Y range
    Chart::update(&mut state, ChartMessage::SetYRange(Some(0.0), Some(100.0)));

    // Cycle active series
    let output = Chart::update(&mut state, ChartMessage::NextSeries);
    assert_eq!(output, Some(ChartOutput::ActiveSeriesChanged(1)));

    let output = Chart::update(&mut state, ChartMessage::NextSeries);
    assert_eq!(output, Some(ChartOutput::ActiveSeriesChanged(0)));

    // Create a scatter chart
    let scatter_state = ChartState::scatter(vec![DataSeries::new(
        "Points",
        vec![10.0, 25.0, 15.0, 30.0, 20.0],
    )]);
    assert_eq!(scatter_state.kind(), &ChartKind::Scatter);

    // Render both charts
    let backend = CaptureBackend::new(60, 15);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = envision::Theme::default();
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

// ============================================================================
// 6. Timeline: add events/spans, zoom in/out, verify view range
// ============================================================================

#[test]
fn test_timeline_events_spans_and_zoom() {
    let mut state = TimelineState::new()
        .with_events(vec![
            TimelineEvent::new("e1", 100.0, "Start"),
            TimelineEvent::new("e2", 500.0, "Deploy"),
        ])
        .with_spans(vec![TimelineSpan::new("s1", 200.0, 800.0, "request-1")]);
    state.set_focused(true);

    assert_eq!(state.events().len(), 2);
    assert_eq!(state.spans().len(), 1);

    // Default view range
    let (start, end) = state.view_range();
    assert_eq!(start, 0.0);
    assert_eq!(end, 1000.0);

    // Add another event via message
    let new_event = TimelineEvent::new("e3", 750.0, "Rollback");
    Timeline::update(&mut state, TimelineMessage::AddEvent(new_event));
    assert_eq!(state.events().len(), 3);

    // Add another span
    let new_span = TimelineSpan::new("s2", 600.0, 900.0, "request-2");
    Timeline::update(&mut state, TimelineMessage::AddSpan(new_span));
    assert_eq!(state.spans().len(), 2);

    // Zoom in - should narrow the view range
    let initial_range = state.view_range();
    let initial_width = initial_range.1 - initial_range.0;

    Timeline::update(&mut state, TimelineMessage::ZoomIn);
    let after_zoom = state.view_range();
    let zoomed_width = after_zoom.1 - after_zoom.0;
    assert!(
        zoomed_width < initial_width,
        "Zoom in should narrow the view: {} < {}",
        zoomed_width,
        initial_width
    );

    // Zoom out - should widen again
    Timeline::update(&mut state, TimelineMessage::ZoomOut);
    let after_zoomout = state.view_range();
    let zoomout_width = after_zoomout.1 - after_zoomout.0;
    assert!(
        zoomout_width > zoomed_width,
        "Zoom out should widen: {} > {}",
        zoomout_width,
        zoomed_width
    );

    // Render
    let backend = CaptureBackend::new(80, 10);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = envision::Theme::default();
    terminal
        .draw(|frame| {
            Timeline::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

// ============================================================================
// 7. SpanTree: create tree, expand/collapse, navigate
// ============================================================================

#[test]
fn test_span_tree_expand_collapse_navigate() {
    let root = SpanNode::new("root", "frontend/request", 0.0, 1000.0)
        .with_child(
            SpanNode::new("db", "db/query", 100.0, 400.0).with_child(SpanNode::new(
                "serialize",
                "db/serialize",
                120.0,
                180.0,
            )),
        )
        .with_child(SpanNode::new("cache", "cache/lookup", 400.0, 600.0));

    let mut state = SpanTreeState::new(vec![root]);
    state.set_focused(true);

    assert_eq!(state.roots().len(), 1);
    assert_eq!(state.global_start(), 0.0);
    assert_eq!(state.global_end(), 1000.0);
    assert_eq!(state.selected_index(), Some(0));

    // Navigate down to first child (db/query)
    let output = SpanTree::update(&mut state, SpanTreeMessage::SelectDown);
    assert_eq!(output, Some(SpanTreeOutput::Selected("db".into())));
    assert_eq!(state.selected_index(), Some(1));

    // Navigate down to second child (db/serialize)
    let output = SpanTree::update(&mut state, SpanTreeMessage::SelectDown);
    assert_eq!(output, Some(SpanTreeOutput::Selected("serialize".into())));
    assert_eq!(state.selected_index(), Some(2));

    // Navigate back up to db
    SpanTree::update(&mut state, SpanTreeMessage::SelectUp);
    assert_eq!(state.selected_index(), Some(1));

    // Collapse db (should hide serialize child)
    let output = SpanTree::update(&mut state, SpanTreeMessage::Collapse);
    assert_eq!(output, Some(SpanTreeOutput::Collapsed("db".into())));

    // Expand it again
    let output = SpanTree::update(&mut state, SpanTreeMessage::Expand);
    assert_eq!(output, Some(SpanTreeOutput::Expanded("db".into())));

    // Collapse all
    SpanTree::update(&mut state, SpanTreeMessage::CollapseAll);

    // Expand all
    SpanTree::update(&mut state, SpanTreeMessage::ExpandAll);

    // Render
    let backend = CaptureBackend::new(80, 15);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = envision::Theme::default();
    terminal
        .draw(|frame| {
            SpanTree::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

// ============================================================================
// 8. FlameGraph: set root, zoom in/out, search
// ============================================================================

#[test]
fn test_flame_graph_navigation_zoom_search() {
    let root = FlameNode::new("main()", 500)
        .with_child(
            FlameNode::new("compute()", 300)
                .with_child(FlameNode::new("sort()", 200))
                .with_child(FlameNode::new("filter()", 100)),
        )
        .with_child(FlameNode::new("io()", 100));

    let mut state = FlameGraphState::with_root(root);
    state.set_focused(true);

    assert!(state.root().is_some());
    assert_eq!(state.root().unwrap().label(), "main()");
    assert_eq!(state.selected_depth(), 0);
    assert_eq!(state.selected_index(), 0);

    // Navigate down into children
    let output = FlameGraph::update(&mut state, FlameGraphMessage::SelectDown);
    assert!(matches!(
        output,
        Some(FlameGraphOutput::FrameSelected { .. })
    ));
    assert_eq!(state.selected_depth(), 1);

    // Navigate right to sibling
    FlameGraph::update(&mut state, FlameGraphMessage::SelectRight);
    assert_eq!(state.selected_index(), 1);

    // Navigate left back
    FlameGraph::update(&mut state, FlameGraphMessage::SelectLeft);
    assert_eq!(state.selected_index(), 0);

    // Navigate up to parent
    FlameGraph::update(&mut state, FlameGraphMessage::SelectUp);
    assert_eq!(state.selected_depth(), 0);

    // Navigate down to compute() and zoom in
    FlameGraph::update(&mut state, FlameGraphMessage::SelectDown);
    let output = FlameGraph::update(&mut state, FlameGraphMessage::ZoomIn);
    assert!(matches!(output, Some(FlameGraphOutput::ZoomedIn(_))));
    assert!(!state.zoom_stack().is_empty());

    // Zoom out
    let output = FlameGraph::update(&mut state, FlameGraphMessage::ZoomOut);
    assert_eq!(output, Some(FlameGraphOutput::ZoomedOut));
    assert!(state.zoom_stack().is_empty());

    // Set search query
    FlameGraph::update(&mut state, FlameGraphMessage::SetSearch("sort".to_string()));
    assert_eq!(state.search_query(), "sort");

    // Clear search
    FlameGraph::update(&mut state, FlameGraphMessage::ClearSearch);
    assert_eq!(state.search_query(), "");

    // Render
    let backend = CaptureBackend::new(60, 15);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = envision::Theme::default();
    terminal
        .draw(|frame| {
            FlameGraph::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

// ============================================================================
// 9. EventStream: push events, filter by level, verify visible count
// ============================================================================

#[test]
fn test_event_stream_push_and_level_filter() {
    let mut state = EventStreamState::new().with_title("System Events");
    state.set_focused(true);

    // Push events at different levels
    state.push_event(EventLevel::Trace, "Trace message");
    state.push_event(EventLevel::Debug, "Debug message");
    state.push_event(EventLevel::Info, "Request received");
    state.push_event(EventLevel::Warning, "Slow query detected");
    state.push_event(EventLevel::Error, "Connection timeout");
    state.push_event_with_fields(
        EventLevel::Info,
        "User login",
        vec![
            ("user".into(), "alice".into()),
            ("method".into(), "oauth".into()),
        ],
    );

    assert_eq!(state.event_count(), 6);
    assert_eq!(state.visible_events().len(), 6);

    // Filter to Warning and above
    EventStream::update(
        &mut state,
        EventStreamMessage::SetLevelFilter(Some(EventLevel::Warning)),
    );
    assert_eq!(state.visible_events().len(), 2); // Warning + Error

    // Filter to Info and above
    EventStream::update(
        &mut state,
        EventStreamMessage::SetLevelFilter(Some(EventLevel::Info)),
    );
    assert_eq!(state.visible_events().len(), 4); // Info + Warning + Error + Info(login)

    // Clear filter - all visible again
    EventStream::update(&mut state, EventStreamMessage::SetLevelFilter(None));
    assert_eq!(state.visible_events().len(), 6);

    // Text filter
    EventStream::update(
        &mut state,
        EventStreamMessage::SetFilter("query".to_string()),
    );
    assert_eq!(state.visible_events().len(), 1); // "Slow query detected"

    // Clear events
    EventStream::update(&mut state, EventStreamMessage::Clear);
    assert_eq!(state.event_count(), 0);
}

// ============================================================================
// 10. AlertPanel: update metrics, verify state transitions
// ============================================================================

#[test]
fn test_alert_panel_state_transitions() {
    let metrics = vec![
        AlertMetric::new("cpu", "CPU Usage", AlertThreshold::new(70.0, 90.0))
            .with_units("%")
            .with_value(45.0),
        AlertMetric::new("mem", "Memory", AlertThreshold::new(80.0, 95.0))
            .with_units("%")
            .with_value(50.0),
        AlertMetric::new("disk", "Disk", AlertThreshold::new(85.0, 95.0))
            .with_units("%")
            .with_value(60.0),
    ];

    let mut state = AlertPanelState::new().with_metrics(metrics).with_columns(2);
    state.set_focused(true);

    // All start as OK
    assert_eq!(state.ok_count(), 3);
    assert_eq!(state.warning_count(), 0);
    assert_eq!(state.critical_count(), 0);

    // Update CPU to 75% -> Warning (threshold: warn=70, crit=90)
    let output = AlertPanel::update(
        &mut state,
        AlertPanelMessage::UpdateMetric {
            id: "cpu".into(),
            value: 75.0,
        },
    );
    assert!(matches!(
        output,
        Some(AlertPanelOutput::StateChanged {
            id,
            old: AlertState::Ok,
            new_state: AlertState::Warning,
            ..
        }) if id == "cpu"
    ));
    assert_eq!(state.ok_count(), 2);
    assert_eq!(state.warning_count(), 1);

    // Update CPU to 95% -> Critical
    let output = AlertPanel::update(
        &mut state,
        AlertPanelMessage::UpdateMetric {
            id: "cpu".into(),
            value: 95.0,
        },
    );
    assert!(matches!(
        output,
        Some(AlertPanelOutput::StateChanged {
            new_state: AlertState::Critical,
            ..
        })
    ));
    assert_eq!(state.critical_count(), 1);

    // Update CPU back down to 50% -> OK
    let output = AlertPanel::update(
        &mut state,
        AlertPanelMessage::UpdateMetric {
            id: "cpu".into(),
            value: 50.0,
        },
    );
    assert!(matches!(
        output,
        Some(AlertPanelOutput::StateChanged {
            new_state: AlertState::Ok,
            ..
        })
    ));
    assert_eq!(state.ok_count(), 3);
    assert_eq!(state.critical_count(), 0);

    // Navigate selection
    AlertPanel::update(&mut state, AlertPanelMessage::SelectNext);
    AlertPanel::update(&mut state, AlertPanelMessage::SelectNext);

    // Select and verify output
    let output = AlertPanel::update(&mut state, AlertPanelMessage::Select);
    assert!(matches!(output, Some(AlertPanelOutput::MetricSelected(_))));

    // Render
    let backend = CaptureBackend::new(60, 20);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = envision::Theme::default();
    terminal
        .draw(|frame| {
            AlertPanel::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

// ============================================================================
// 11. CommandPalette: type query, verify filtering, select item
// ============================================================================

#[test]
fn test_command_palette_filter_and_select() {
    let items = vec![
        PaletteItem::new("open", "Open File").with_shortcut("Ctrl+O"),
        PaletteItem::new("save", "Save File").with_shortcut("Ctrl+S"),
        PaletteItem::new("save_as", "Save As").with_shortcut("Ctrl+Shift+S"),
        PaletteItem::new("quit", "Quit Application").with_shortcut("Ctrl+Q"),
        PaletteItem::new("find", "Find in File").with_shortcut("Ctrl+F"),
    ];

    let mut state = CommandPaletteState::new(items);
    state.set_focused(true);
    state.set_visible(true);

    assert!(state.is_visible());
    assert_eq!(state.items().len(), 5);
    assert_eq!(state.filtered_items().len(), 5);
    assert_eq!(state.query(), "");

    // Type "save" to filter
    for c in "save".chars() {
        CommandPalette::update(&mut state, CommandPaletteMessage::TypeChar(c));
    }
    assert_eq!(state.query(), "save");
    // "Save File" and "Save As" should match
    assert!(state.filtered_items().len() >= 2);

    // Navigate down to second result
    CommandPalette::update(&mut state, CommandPaletteMessage::SelectNext);

    // Confirm selection
    let output = CommandPalette::update(&mut state, CommandPaletteMessage::Confirm);
    assert!(matches!(output, Some(CommandPaletteOutput::Selected(_))));
    assert!(!state.is_visible()); // Palette hides after selection

    // Show again, clear query
    CommandPalette::update(&mut state, CommandPaletteMessage::Show);
    assert!(state.is_visible());
    CommandPalette::update(&mut state, CommandPaletteMessage::ClearQuery);
    assert_eq!(state.query(), "");
    assert_eq!(state.filtered_items().len(), 5);

    // Dismiss
    CommandPalette::update(&mut state, CommandPaletteMessage::Dismiss);
    assert!(!state.is_visible());
}

// ============================================================================
// 12. Calendar: navigate months, select date
// ============================================================================

#[test]
fn test_calendar_navigation_and_selection() {
    let mut state = CalendarState::new(2026, 3).with_selected_day(15);
    Calendar::focus(&mut state);

    assert_eq!(state.year(), 2026);
    assert_eq!(state.month(), 3);
    assert_eq!(state.month_name(), "March");
    assert_eq!(state.selected_day(), Some(15));

    // Navigate to next month
    let output = Calendar::update(&mut state, CalendarMessage::NextMonth);
    assert_eq!(output, Some(CalendarOutput::MonthChanged(2026, 4)));
    assert_eq!(state.month(), 4);
    assert_eq!(state.month_name(), "April");

    // Navigate to previous month (back to March)
    let output = Calendar::update(&mut state, CalendarMessage::PrevMonth);
    assert_eq!(output, Some(CalendarOutput::MonthChanged(2026, 3)));
    assert_eq!(state.month(), 3);

    // Navigate to next year
    let output = Calendar::update(&mut state, CalendarMessage::NextYear);
    assert_eq!(output, Some(CalendarOutput::MonthChanged(2027, 3)));
    assert_eq!(state.year(), 2027);

    // Select a specific day
    Calendar::update(&mut state, CalendarMessage::SelectDay(20));
    assert_eq!(state.selected_day(), Some(20));

    // Confirm selection
    let output = Calendar::update(&mut state, CalendarMessage::ConfirmSelection);
    assert_eq!(output, Some(CalendarOutput::DateSelected(2027, 3, 20)));

    // Add an event marker
    Calendar::update(
        &mut state,
        CalendarMessage::AddEvent {
            year: 2027,
            month: 3,
            day: 20,
            color: Color::Green,
        },
    );
    assert!(state.has_event(2027, 3, 20));

    // Navigate days
    Calendar::update(&mut state, CalendarMessage::SelectNextDay);
    assert_eq!(state.selected_day(), Some(21));
    Calendar::update(&mut state, CalendarMessage::SelectPrevDay);
    assert_eq!(state.selected_day(), Some(20));

    // Navigate weeks
    Calendar::update(&mut state, CalendarMessage::SelectNextWeek);
    assert_eq!(state.selected_day(), Some(27));
    Calendar::update(&mut state, CalendarMessage::SelectPrevWeek);
    assert_eq!(state.selected_day(), Some(20));

    // Render
    let backend = CaptureBackend::new(30, 10);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = envision::Theme::default();
    terminal
        .draw(|frame| {
            Calendar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

// ============================================================================
// 13. Slider: increment/decrement, verify clamping
// ============================================================================

#[test]
fn test_slider_increment_decrement_clamping() {
    let mut state = SliderState::new(0.0, 10.0).with_step(2.0);
    assert_eq!(state.value(), 0.0);
    assert_eq!(state.min(), 0.0);
    assert_eq!(state.max(), 10.0);
    assert_eq!(state.step(), 2.0);

    // Increment by step
    let output = Slider::update(&mut state, SliderMessage::Increment);
    assert_eq!(output, Some(SliderOutput::ValueChanged(2.0)));
    assert_eq!(state.value(), 2.0);

    // Increment more
    Slider::update(&mut state, SliderMessage::Increment);
    Slider::update(&mut state, SliderMessage::Increment);
    assert_eq!(state.value(), 6.0);

    // IncrementPage (step * 10 = 20.0, clamped to max)
    Slider::update(&mut state, SliderMessage::IncrementPage);
    assert_eq!(state.value(), 10.0); // clamped

    // Decrement from max
    let output = Slider::update(&mut state, SliderMessage::Decrement);
    assert_eq!(output, Some(SliderOutput::ValueChanged(8.0)));

    // Decrement past zero is clamped
    Slider::update(&mut state, SliderMessage::SetValue(-5.0));
    assert_eq!(state.value(), 0.0);

    // SetMax
    Slider::update(&mut state, SliderMessage::SetMax);
    assert_eq!(state.value(), 10.0);

    // SetMin
    Slider::update(&mut state, SliderMessage::SetMin);
    assert_eq!(state.value(), 0.0);

    // Percentage at min
    assert!((state.percentage() - 0.0).abs() < f64::EPSILON);

    // Percentage at max
    Slider::update(&mut state, SliderMessage::SetMax);
    assert!((state.percentage() - 1.0).abs() < f64::EPSILON);

    // Disabled slider returns None
    state.set_disabled(true);
    let output = Slider::update(&mut state, SliderMessage::Increment);
    assert_eq!(output, None);
}

// ============================================================================
// 14. Switch: toggle, verify output
// ============================================================================

#[test]
fn test_switch_toggle_workflow() {
    let mut state = SwitchState::new().with_label("Dark Mode");
    assert!(!state.is_on());
    assert_eq!(state.label(), Some("Dark Mode"));

    // Toggle on
    let output = Switch::update(&mut state, SwitchMessage::Toggle);
    assert_eq!(output, Some(SwitchOutput::On));
    assert!(state.is_on());

    // Toggle off
    let output = Switch::update(&mut state, SwitchMessage::Toggle);
    assert_eq!(output, Some(SwitchOutput::Off));
    assert!(!state.is_on());

    // Set directly to on
    let output = Switch::update(&mut state, SwitchMessage::SetOn(true));
    assert_eq!(output, Some(SwitchOutput::Toggled(true)));
    assert!(state.is_on());

    // Set directly to off
    let output = Switch::update(&mut state, SwitchMessage::SetOn(false));
    assert_eq!(output, Some(SwitchOutput::Toggled(false)));
    assert!(!state.is_on());

    // Disabled switch does not toggle
    state.set_disabled(true);
    let output = Switch::update(&mut state, SwitchMessage::Toggle);
    assert_eq!(output, None);
    assert!(!state.is_on());

    // Re-enable and verify toggle works again
    state.set_disabled(false);
    let output = Switch::update(&mut state, SwitchMessage::Toggle);
    assert_eq!(output, Some(SwitchOutput::On));
    assert!(state.is_on());

    // Render
    let backend = CaptureBackend::new(20, 1);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = envision::Theme::default();
    terminal
        .draw(|frame| {
            Switch::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

// ============================================================================
// 15. Paginator: navigate pages, verify boundary behavior
// ============================================================================

#[test]
fn test_paginator_boundary_navigation() {
    let mut state = PaginatorState::new(5);
    assert_eq!(state.current_page(), 0);
    assert_eq!(state.display_page(), 1);
    assert_eq!(state.total_pages(), 5);

    // Navigate to next page
    let output = Paginator::update(&mut state, PaginatorMessage::NextPage);
    assert_eq!(output, Some(PaginatorOutput::PageChanged(1)));
    assert_eq!(state.current_page(), 1);

    // Navigate to last page
    let output = Paginator::update(&mut state, PaginatorMessage::LastPage);
    assert_eq!(output, Some(PaginatorOutput::PageChanged(4)));
    assert_eq!(state.current_page(), 4);
    assert_eq!(state.display_page(), 5);

    // NextPage at last page - should stay at last page
    let output = Paginator::update(&mut state, PaginatorMessage::NextPage);
    assert_eq!(output, None);
    assert_eq!(state.current_page(), 4);

    // Navigate to first page
    let output = Paginator::update(&mut state, PaginatorMessage::FirstPage);
    assert_eq!(output, Some(PaginatorOutput::PageChanged(0)));
    assert_eq!(state.current_page(), 0);

    // PrevPage at first page - should stay at first page
    let output = Paginator::update(&mut state, PaginatorMessage::PrevPage);
    assert_eq!(output, None);
    assert_eq!(state.current_page(), 0);

    // GoToPage
    let output = Paginator::update(&mut state, PaginatorMessage::GoToPage(3));
    assert_eq!(output, Some(PaginatorOutput::PageChanged(3)));
    assert_eq!(state.current_page(), 3);

    // Update total pages
    Paginator::update(&mut state, PaginatorMessage::SetTotalPages(10));
    assert_eq!(state.total_pages(), 10);
    assert_eq!(state.current_page(), 3); // should stay on page 3

    // Navigate to last of the new total
    Paginator::update(&mut state, PaginatorMessage::LastPage);
    assert_eq!(state.current_page(), 9);

    // Render
    let backend = CaptureBackend::new(40, 1);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = envision::Theme::default();
    terminal
        .draw(|frame| {
            Paginator::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

// ============================================================================
// 16. HelpPanel: scroll through bindings
// ============================================================================

#[test]
fn test_help_panel_scroll_and_groups() {
    let state = HelpPanelState::new().with_groups(vec![
        KeyBindingGroup::new(
            "Navigation",
            vec![
                KeyBinding::new("Up/k", "Move up"),
                KeyBinding::new("Down/j", "Move down"),
                KeyBinding::new("PageUp", "Page up"),
                KeyBinding::new("PageDown", "Page down"),
                KeyBinding::new("Home", "Go to top"),
                KeyBinding::new("End", "Go to bottom"),
            ],
        ),
        KeyBindingGroup::new(
            "Actions",
            vec![
                KeyBinding::new("Enter", "Select item"),
                KeyBinding::new("Space", "Toggle"),
                KeyBinding::new("Tab", "Next field"),
                KeyBinding::new("q/Esc", "Quit"),
            ],
        ),
        KeyBindingGroup::new(
            "Search",
            vec![
                KeyBinding::new("/", "Start search"),
                KeyBinding::new("n", "Next result"),
                KeyBinding::new("N", "Previous result"),
            ],
        ),
    ]);

    assert_eq!(state.groups().len(), 3);
    assert_eq!(state.title(), Some("Help"));
    assert!(state.is_visible());

    // Scroll down and up
    let mut state = state;
    HelpPanel::update(&mut state, HelpPanelMessage::ScrollDown);
    HelpPanel::update(&mut state, HelpPanelMessage::ScrollDown);
    HelpPanel::update(&mut state, HelpPanelMessage::ScrollUp);

    // Page down
    HelpPanel::update(&mut state, HelpPanelMessage::PageDown(5));

    // Home
    HelpPanel::update(&mut state, HelpPanelMessage::Home);

    // End
    HelpPanel::update(&mut state, HelpPanelMessage::End);

    // Add a new group
    HelpPanel::update(
        &mut state,
        HelpPanelMessage::AddGroup(KeyBindingGroup::new(
            "Advanced",
            vec![KeyBinding::new("Ctrl+R", "Reload configuration")],
        )),
    );
    assert_eq!(state.groups().len(), 4);

    // Render
    let backend = CaptureBackend::new(50, 20);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = envision::Theme::default();
    terminal
        .draw(|frame| {
            HelpPanel::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

// ============================================================================
// 17. CodeBlock: set code, change language, scroll
// ============================================================================

#[test]
fn test_code_block_set_code_language_scroll() {
    let mut state = CodeBlockState::new()
        .with_code("fn main() {\n    println!(\"Hello, world!\");\n}\n")
        .with_language(Language::Rust)
        .with_title("main.rs")
        .with_line_numbers(true);

    assert_eq!(
        state.code(),
        "fn main() {\n    println!(\"Hello, world!\");\n}\n"
    );
    assert_eq!(state.language(), &Language::Rust);
    assert_eq!(state.title(), Some("main.rs"));
    assert!(state.show_line_numbers());
    assert_eq!(state.scroll_offset(), 0);

    // Change code content
    CodeBlock::update(
        &mut state,
        CodeBlockMessage::SetCode("def hello():\n    print(\"Hello\")\n\nhello()\n".to_string()),
    );
    assert_eq!(
        state.code(),
        "def hello():\n    print(\"Hello\")\n\nhello()\n"
    );

    // Change language
    CodeBlock::update(&mut state, CodeBlockMessage::SetLanguage(Language::Python));
    assert_eq!(state.language(), &Language::Python);

    // Scroll down
    CodeBlock::update(&mut state, CodeBlockMessage::ScrollDown);

    // Scroll to end
    CodeBlock::update(&mut state, CodeBlockMessage::End);

    // Scroll to beginning
    CodeBlock::update(&mut state, CodeBlockMessage::Home);
    assert_eq!(state.scroll_offset(), 0);

    // Toggle line numbers
    CodeBlock::update(&mut state, CodeBlockMessage::ToggleLineNumbers);
    assert!(!state.show_line_numbers());

    // Highlight a line
    CodeBlock::update(&mut state, CodeBlockMessage::HighlightLine(2));
    // Unhighlight
    CodeBlock::update(&mut state, CodeBlockMessage::UnhighlightLine(2));

    // Render
    let backend = CaptureBackend::new(60, 10);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = envision::Theme::default();
    terminal
        .draw(|frame| {
            CodeBlock::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

// ============================================================================
// 18. TerminalOutput: append lines, verify auto-scroll, check ANSI parsing
// ============================================================================

#[test]
fn test_terminal_output_append_and_auto_scroll() {
    let mut state = TerminalOutputState::new()
        .with_title("Build Output")
        .with_auto_scroll(true)
        .with_max_lines(100);

    assert!(state.auto_scroll());
    assert_eq!(state.line_count(), 0);

    // Push lines
    state.push_line("Compiling envision v0.7.0");
    state.push_line("\x1b[32m   Finished\x1b[0m in 2.5s");
    state.push_line("\x1b[31merror[E0308]\x1b[0m: mismatched types");

    assert_eq!(state.line_count(), 3);
    assert_eq!(state.lines()[0], "Compiling envision v0.7.0");
    assert_eq!(state.lines()[1], "\x1b[32m   Finished\x1b[0m in 2.5s");

    // Push via message
    let output = TerminalOutput::update(
        &mut state,
        TerminalOutputMessage::PushLine("warning: unused variable".to_string()),
    );
    assert!(matches!(output, Some(TerminalOutputOutput::LineAdded(4))));
    assert_eq!(state.line_count(), 4);

    // Push multiple lines
    TerminalOutput::update(
        &mut state,
        TerminalOutputMessage::PushLines(vec!["line 5".to_string(), "line 6".to_string()]),
    );
    assert_eq!(state.line_count(), 6);

    // Toggle auto-scroll
    let output = TerminalOutput::update(&mut state, TerminalOutputMessage::ToggleAutoScroll);
    assert!(matches!(
        output,
        Some(TerminalOutputOutput::AutoScrollToggled(false))
    ));
    assert!(!state.auto_scroll());

    // Toggle line numbers
    let output = TerminalOutput::update(&mut state, TerminalOutputMessage::ToggleLineNumbers);
    assert!(matches!(
        output,
        Some(TerminalOutputOutput::LineNumbersToggled(true))
    ));

    // Set running and exit code
    TerminalOutput::update(&mut state, TerminalOutputMessage::SetRunning(true));
    TerminalOutput::update(&mut state, TerminalOutputMessage::SetExitCode(Some(0)));

    // Clear
    let output = TerminalOutput::update(&mut state, TerminalOutputMessage::Clear);
    assert!(matches!(output, Some(TerminalOutputOutput::Cleared)));
    assert_eq!(state.line_count(), 0);

    // Verify ANSI parsing works
    let segments = envision::component::parse_ansi("\x1b[32mgreen\x1b[0m normal");
    assert!(!segments.is_empty());

    // Render
    state.push_line("Test line");
    let backend = CaptureBackend::new(60, 10);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = envision::Theme::default();
    terminal
        .draw(|frame| {
            TerminalOutput::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

// ============================================================================
// 19. ConversationView: push messages, toggle blocks, scroll
// ============================================================================

#[test]
fn test_conversation_view_messages_and_collapse() {
    let mut state = ConversationViewState::new().with_title("Chat Session");
    assert_eq!(state.message_count(), 0);
    assert!(state.auto_scroll());

    // Push messages using helper methods
    state.push_user("Hello, can you help me?");
    assert_eq!(state.message_count(), 1);
    assert_eq!(state.messages()[0].role(), ConversationRole::User);

    state.push_assistant("Of course! How can I help?");
    assert_eq!(state.message_count(), 2);
    assert_eq!(state.messages()[1].role(), ConversationRole::Assistant);

    state.push_system("System: model context loaded");
    assert_eq!(state.message_count(), 3);
    assert_eq!(state.messages()[2].role(), ConversationRole::System);

    // Push a structured message with code blocks
    state.push_message(ConversationMessage::with_blocks(
        ConversationRole::Assistant,
        vec![
            MessageBlock::text("Here is the fix:"),
            MessageBlock::code("fn main() {}", Some("rust")),
        ],
    ));
    assert_eq!(state.message_count(), 4);
    assert_eq!(state.messages()[3].blocks().len(), 2);

    // Push tool result
    state.push_tool("Search results: 5 items found");
    assert_eq!(state.message_count(), 5);
    assert_eq!(state.messages()[4].role(), ConversationRole::Tool);

    // Toggle collapse on a named block key
    assert!(!state.is_collapsed("thinking"));
    state.toggle_collapse("thinking");
    assert!(state.is_collapsed("thinking"));
    state.toggle_collapse("thinking");
    assert!(!state.is_collapsed("thinking"));

    // Scroll via messages
    ConversationView::update(&mut state, ConversationViewMessage::ScrollToTop);
    ConversationView::update(&mut state, ConversationViewMessage::ScrollToBottom);
    ConversationView::update(&mut state, ConversationViewMessage::ScrollUp);
    ConversationView::update(&mut state, ConversationViewMessage::ScrollDown);

    // Max messages cap
    let mut capped_state = ConversationViewState::new().with_max_messages(3);
    for i in 0..5 {
        capped_state.push_user(format!("Message {}", i));
    }
    assert_eq!(capped_state.message_count(), 3);

    // Render
    let backend = CaptureBackend::new(60, 20);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = envision::Theme::default();
    terminal
        .draw(|frame| {
            ConversationView::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

// ============================================================================
// 20. TabBar: add/close/navigate tabs
// ============================================================================

#[test]
fn test_tab_bar_add_close_navigate() {
    let tabs = vec![
        Tab::new("file1", "main.rs"),
        Tab::new("file2", "lib.rs").with_modified(true),
        Tab::new("file3", "test.rs").with_closable(true),
    ];
    let mut state = TabBarState::new(tabs);
    TabBar::set_focused(&mut state, true);

    assert_eq!(state.len(), 3);
    assert_eq!(state.active_index(), Some(0));
    assert_eq!(state.active_tab().map(|t| t.label()), Some("main.rs"));

    // Navigate to next tab
    let output = TabBar::update(&mut state, TabBarMessage::NextTab);
    assert_eq!(output, Some(TabBarOutput::TabSelected(1)));
    assert_eq!(state.active_index(), Some(1));
    assert_eq!(state.active_tab().map(|t| t.label()), Some("lib.rs"));
    assert!(state.active_tab().map(|t| t.modified()).unwrap_or(false));

    // Navigate to last tab
    TabBar::update(&mut state, TabBarMessage::NextTab);
    assert_eq!(state.active_index(), Some(2));
    assert_eq!(state.active_tab().map(|t| t.label()), Some("test.rs"));

    // NextTab at last position does not wrap - stays at last
    let output = TabBar::update(&mut state, TabBarMessage::NextTab);
    assert_eq!(output, None);
    assert_eq!(state.active_index(), Some(2));

    // PrevTab at first position does not wrap - stays at first
    TabBar::update(&mut state, TabBarMessage::First);
    let output = TabBar::update(&mut state, TabBarMessage::PrevTab);
    assert_eq!(output, None);
    assert_eq!(state.active_index(), Some(0));

    // Navigate to last tab for further testing
    TabBar::update(&mut state, TabBarMessage::Last);

    // Add a new tab
    let output = TabBar::update(
        &mut state,
        TabBarMessage::AddTab(Tab::new("file4", "config.toml").with_closable(true)),
    );
    assert_eq!(output, Some(TabBarOutput::TabAdded(3)));
    assert_eq!(state.len(), 4);
    assert_eq!(state.active_index(), Some(3)); // new tab becomes active

    // Close the closable tab at index 2 (test.rs)
    let output = TabBar::update(&mut state, TabBarMessage::CloseTab(2));
    assert_eq!(output, Some(TabBarOutput::TabClosed(2)));
    assert_eq!(state.len(), 3);

    // Jump to first
    TabBar::update(&mut state, TabBarMessage::First);
    assert_eq!(state.active_index(), Some(0));

    // Jump to last
    TabBar::update(&mut state, TabBarMessage::Last);
    assert_eq!(state.active_index(), Some(2));

    // Select by index
    let output = TabBar::update(&mut state, TabBarMessage::SelectTab(1));
    assert_eq!(output, Some(TabBarOutput::TabSelected(1)));
    assert_eq!(state.active_index(), Some(1));

    // Render
    let backend = CaptureBackend::new(60, 3);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = envision::Theme::default();
    terminal
        .draw(|frame| {
            TabBar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

// ============================================================================
// 21. Gauge with custom thresholds
// ============================================================================

#[test]
fn test_gauge_custom_thresholds() {
    let state = GaugeState::new(30.0, 100.0).with_thresholds(vec![
        ThresholdZone {
            above: 0.0,
            color: Color::Blue,
        },
        ThresholdZone {
            above: 0.5,
            color: Color::Cyan,
        },
        ThresholdZone {
            above: 0.8,
            color: Color::Magenta,
        },
    ]);

    // 30% -> Blue (above 0.0, below 0.5)
    assert_eq!(state.current_color(), Color::Blue);

    let mut state = state;
    Gauge::update(&mut state, GaugeMessage::SetValue(55.0));
    // 55% -> Cyan (above 0.5, below 0.8)
    assert_eq!(state.current_color(), Color::Cyan);

    Gauge::update(&mut state, GaugeMessage::SetValue(85.0));
    // 85% -> Magenta (above 0.8)
    assert_eq!(state.current_color(), Color::Magenta);

    // Test SetMax
    Gauge::update(&mut state, GaugeMessage::SetMax(200.0));
    assert_eq!(state.max(), 200.0);
    // 85 / 200 = 42.5%, should be Blue again
    assert_eq!(state.current_color(), Color::Blue);
}

// ============================================================================
// 22. Heatmap with labels and data replacement
// ============================================================================

#[test]
fn test_heatmap_labels_and_data_replacement() {
    let data = vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]];
    let mut state = HeatmapState::with_data(data)
        .with_row_labels(vec!["Row A".into(), "Row B".into()])
        .with_col_labels(vec!["Mon".into(), "Tue".into(), "Wed".into()]);

    assert_eq!(state.rows(), 2);
    assert_eq!(state.cols(), 3);
    assert_eq!(state.row_labels(), &["Row A", "Row B"]);
    assert_eq!(state.col_labels(), &["Mon", "Tue", "Wed"]);
    assert_eq!(state.get(1, 2), Some(6.0));

    // Replace all data via message
    Heatmap::update(
        &mut state,
        HeatmapMessage::SetData(vec![vec![10.0, 20.0], vec![30.0, 40.0], vec![50.0, 60.0]]),
    );
    assert_eq!(state.rows(), 3);
    assert_eq!(state.cols(), 2);
    assert_eq!(state.get(2, 1), Some(60.0));

    // Clear zeros all cells but preserves dimensions
    Heatmap::update(&mut state, HeatmapMessage::Clear);
    assert_eq!(state.rows(), 3);
    assert_eq!(state.cols(), 2);
    assert_eq!(state.get(0, 0), Some(0.0));
    assert_eq!(state.get(2, 1), Some(0.0));
}

// ============================================================================
// 23. Chart series cycling with multiple series
// ============================================================================

#[test]
fn test_chart_multi_series_cycling() {
    let state = ChartState::line(vec![
        DataSeries::new("Series A", vec![1.0, 2.0]),
        DataSeries::new("Series B", vec![3.0, 4.0]),
        DataSeries::new("Series C", vec![5.0, 6.0]),
    ]);

    let mut state = state;

    // Initially on series 0
    let output = Chart::update(&mut state, ChartMessage::NextSeries);
    assert_eq!(output, Some(ChartOutput::ActiveSeriesChanged(1)));

    let output = Chart::update(&mut state, ChartMessage::NextSeries);
    assert_eq!(output, Some(ChartOutput::ActiveSeriesChanged(2)));

    // Wrap around
    let output = Chart::update(&mut state, ChartMessage::NextSeries);
    assert_eq!(output, Some(ChartOutput::ActiveSeriesChanged(0)));

    // PrevSeries
    let output = Chart::update(&mut state, ChartMessage::PrevSeries);
    assert_eq!(output, Some(ChartOutput::ActiveSeriesChanged(2)));
}

// ============================================================================
// 24. Calendar month-boundary wrapping
// ============================================================================

#[test]
fn test_calendar_month_boundary_wrapping() {
    // Start at December 2026
    let mut state = CalendarState::new(2026, 12);
    Calendar::focus(&mut state);

    // Next month wraps to January 2027
    let output = Calendar::update(&mut state, CalendarMessage::NextMonth);
    assert_eq!(output, Some(CalendarOutput::MonthChanged(2027, 1)));
    assert_eq!(state.year(), 2027);
    assert_eq!(state.month(), 1);

    // Previous month goes back to December 2026
    let output = Calendar::update(&mut state, CalendarMessage::PrevMonth);
    assert_eq!(output, Some(CalendarOutput::MonthChanged(2026, 12)));
    assert_eq!(state.year(), 2026);
    assert_eq!(state.month(), 12);

    // January -> previous month -> December of previous year
    let mut state = CalendarState::new(2026, 1);
    Calendar::focus(&mut state);
    let output = Calendar::update(&mut state, CalendarMessage::PrevMonth);
    assert_eq!(output, Some(CalendarOutput::MonthChanged(2025, 12)));
}

// ============================================================================
// 25. EventStream with structured fields
// ============================================================================

#[test]
fn test_event_stream_structured_fields_and_text_filter() {
    let mut state = EventStreamState::new();
    state.set_focused(true);

    state.push_event_with_fields(
        EventLevel::Info,
        "GET /api/users",
        vec![
            ("method".into(), "GET".into()),
            ("path".into(), "/api/users".into()),
            ("status".into(), "200".into()),
        ],
    );
    state.push_event_with_fields(
        EventLevel::Warning,
        "POST /api/orders",
        vec![
            ("method".into(), "POST".into()),
            ("path".into(), "/api/orders".into()),
            ("status".into(), "500".into()),
        ],
    );
    state.push_event_with_fields(
        EventLevel::Info,
        "GET /api/products",
        vec![
            ("method".into(), "GET".into()),
            ("path".into(), "/api/products".into()),
            ("status".into(), "200".into()),
        ],
    );

    assert_eq!(state.event_count(), 3);

    // Filter by text "orders"
    EventStream::update(
        &mut state,
        EventStreamMessage::SetFilter("orders".to_string()),
    );
    assert_eq!(state.visible_events().len(), 1);

    // Filter by text "GET"
    EventStream::update(&mut state, EventStreamMessage::SetFilter("GET".to_string()));
    assert_eq!(state.visible_events().len(), 2);

    // Clear filter
    EventStream::update(&mut state, EventStreamMessage::SetFilter(String::new()));
    assert_eq!(state.visible_events().len(), 3);
}

// ============================================================================
// 26. Slider with dispatch_event
// ============================================================================

#[test]
fn test_slider_dispatch_event_keyboard() {
    let mut state = SliderState::new(0.0, 100.0).with_step(5.0);
    state.set_focused(true);

    // Right arrow should increment
    let event = envision::input::Event::key(crossterm::event::KeyCode::Right);
    Slider::dispatch_event(&mut state, &event);
    assert_eq!(state.value(), 5.0);

    // Left arrow should decrement
    let event = envision::input::Event::key(crossterm::event::KeyCode::Left);
    Slider::dispatch_event(&mut state, &event);
    assert_eq!(state.value(), 0.0);

    // Home should go to min
    Slider::update(&mut state, SliderMessage::SetValue(50.0));
    let event = envision::input::Event::key(crossterm::event::KeyCode::Home);
    Slider::dispatch_event(&mut state, &event);
    assert_eq!(state.value(), 0.0);

    // End should go to max
    let event = envision::input::Event::key(crossterm::event::KeyCode::End);
    Slider::dispatch_event(&mut state, &event);
    assert_eq!(state.value(), 100.0);
}

// ============================================================================
// 27. Zero-size rendering for new components
// ============================================================================

fn assert_view_zero_size<F>(name: &str, render_fn: F)
where
    F: FnOnce(&mut ratatui::Frame, Rect, &envision::Theme),
{
    let backend = CaptureBackend::new(0, 0);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = envision::Theme::default();
    terminal
        .draw(|frame| {
            let area = Rect::default();
            render_fn(frame, area, &theme);
        })
        .unwrap_or_else(|e| panic!("{} panicked on zero-size area: {}", name, e));
}

#[test]
fn test_new_components_handle_zero_size_area() {
    // Sparkline
    assert_view_zero_size("Sparkline", |frame, area, theme| {
        let state = SparklineState::with_data(vec![1, 2, 3]);
        Sparkline::view(&state, frame, area, theme);
    });

    // Gauge
    assert_view_zero_size("Gauge", |frame, area, theme| {
        let state = GaugeState::new(50.0, 100.0);
        Gauge::view(&state, frame, area, theme);
    });

    // Calendar
    assert_view_zero_size("Calendar", |frame, area, theme| {
        let state = CalendarState::new(2026, 3);
        Calendar::view(&state, frame, area, theme);
    });

    // Slider
    assert_view_zero_size("Slider", |frame, area, theme| {
        let state = SliderState::new(0.0, 100.0);
        Slider::view(&state, frame, area, theme);
    });

    // Switch
    assert_view_zero_size("Switch", |frame, area, theme| {
        let state = SwitchState::new();
        Switch::view(&state, frame, area, theme);
    });

    // Paginator
    assert_view_zero_size("Paginator", |frame, area, theme| {
        let state = PaginatorState::new(5);
        Paginator::view(&state, frame, area, theme);
    });

    // CodeBlock
    assert_view_zero_size("CodeBlock", |frame, area, theme| {
        let state = CodeBlockState::new().with_code("fn main() {}");
        CodeBlock::view(&state, frame, area, theme);
    });

    // TabBar
    assert_view_zero_size("TabBar", |frame, area, theme| {
        let state = TabBarState::new(vec![Tab::new("a", "Tab A")]);
        TabBar::view(&state, frame, area, theme);
    });
}

// ============================================================================
// 28. SpanTree with deep nesting
// ============================================================================

#[test]
fn test_span_tree_deep_nesting() {
    let root = SpanNode::new("root", "service", 0.0, 1000.0).with_child(
        SpanNode::new("l1", "handler", 10.0, 900.0).with_child(
            SpanNode::new("l2", "middleware", 20.0, 880.0)
                .with_child(SpanNode::new("l3", "db-query", 30.0, 500.0))
                .with_child(SpanNode::new("l4", "cache-set", 510.0, 600.0)),
        ),
    );

    let mut state = SpanTreeState::new(vec![root]);
    state.set_focused(true);

    // Selected at root
    assert_eq!(state.selected_index(), Some(0));

    // Navigate all the way down
    SpanTree::update(&mut state, SpanTreeMessage::SelectDown);
    assert_eq!(state.selected_index(), Some(1)); // handler
    SpanTree::update(&mut state, SpanTreeMessage::SelectDown);
    assert_eq!(state.selected_index(), Some(2)); // middleware
    SpanTree::update(&mut state, SpanTreeMessage::SelectDown);
    assert_eq!(state.selected_index(), Some(3)); // db-query
    SpanTree::update(&mut state, SpanTreeMessage::SelectDown);
    assert_eq!(state.selected_index(), Some(4)); // cache-set

    // Navigate back up
    SpanTree::update(&mut state, SpanTreeMessage::SelectUp);
    assert_eq!(state.selected_index(), Some(3)); // db-query
}

// ============================================================================
// 29. Paginator from items
// ============================================================================

#[test]
fn test_paginator_from_items_calculation() {
    let state = PaginatorState::from_items(247, 25);
    assert_eq!(state.total_pages(), 10);
    assert_eq!(state.total_items(), 247);
    assert_eq!(state.page_size(), 25);
    assert_eq!(state.current_page(), 0);

    // Exact division
    let state = PaginatorState::from_items(100, 10);
    assert_eq!(state.total_pages(), 10);

    // Single item
    let state = PaginatorState::from_items(1, 10);
    assert_eq!(state.total_pages(), 1);
}

// ============================================================================
// 30. Full workflow: dashboard with multiple new components
// ============================================================================

#[test]
fn test_dashboard_workflow_with_mixed_new_components() {
    // Simulate a monitoring dashboard that uses several new components together

    // CPU gauge
    let mut cpu_gauge = GaugeState::new(0.0, 100.0)
        .with_units("%")
        .with_title("CPU");

    // Memory sparkline
    let mut mem_sparkline = SparklineState::new().with_title("Memory");

    // Alert panel
    let mut alerts = AlertPanelState::new().with_metrics(vec![
        AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0))
            .with_units("%")
            .with_value(0.0),
        AlertMetric::new("mem", "Memory", AlertThreshold::new(80.0, 95.0))
            .with_units("%")
            .with_value(0.0),
    ]);

    // Paginator for log pages
    let mut paginator = PaginatorState::new(1);

    // Simulate metric updates over time
    let cpu_values = [25.0, 45.0, 60.0, 72.0, 85.0, 91.0, 78.0, 55.0];
    let mem_values = [40u64, 42, 45, 50, 55, 60, 62, 65];

    for (i, (&cpu, &mem)) in cpu_values.iter().zip(mem_values.iter()).enumerate() {
        // Update gauge
        Gauge::update(&mut cpu_gauge, GaugeMessage::SetValue(cpu));
        assert_eq!(cpu_gauge.value(), cpu);

        // Push sparkline data
        Sparkline::update(&mut mem_sparkline, SparklineMessage::Push(mem));
        assert_eq!(mem_sparkline.len(), i + 1);

        // Update alert metric
        AlertPanel::update(
            &mut alerts,
            AlertPanelMessage::UpdateMetric {
                id: "cpu".into(),
                value: cpu,
            },
        );
        AlertPanel::update(
            &mut alerts,
            AlertPanelMessage::UpdateMetric {
                id: "mem".into(),
                value: mem as f64,
            },
        );
    }

    // After all updates, verify final states
    assert_eq!(cpu_gauge.value(), 55.0);
    assert_eq!(cpu_gauge.current_color(), Color::Green); // 55% is below 70%
    assert_eq!(mem_sparkline.len(), 8);
    assert_eq!(mem_sparkline.last(), Some(65));

    // All metrics should be OK at these values
    assert_eq!(alerts.ok_count(), 2);

    // Update paginator for new data
    Paginator::update(&mut paginator, PaginatorMessage::SetTotalPages(5));
    Paginator::update(&mut paginator, PaginatorMessage::NextPage);
    assert_eq!(paginator.current_page(), 1);

    // Render all components
    let backend = CaptureBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = envision::Theme::default();
    terminal
        .draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(10),
                    Constraint::Length(1),
                ])
                .split(frame.area());

            Gauge::view(&cpu_gauge, frame, chunks[0], &theme);
            Sparkline::view(&mem_sparkline, frame, chunks[1], &theme);
            AlertPanel::view(&alerts, frame, chunks[2], &theme);
            Paginator::view(&paginator, frame, chunks[3], &theme);
        })
        .unwrap();
}
