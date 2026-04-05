#![cfg(feature = "full")]
//! Integration tests for new components added in Iterations 2-6 and the
//! Claude Code component suite. Each test exercises the full
//! event -> message -> update -> view cycle through the public API.
//!
//! Tests 1-14 live here. Tests 15-30 are in integration_new_components_2.rs.

use envision::CaptureBackend;
use envision::ViewContext;
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
    CommandPalette,
    CommandPaletteMessage,
    CommandPaletteOutput,
    CommandPaletteState,
    // Traits
    Component,
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
    Gauge,
    GaugeMessage,
    GaugeState,
    GaugeVariant,
    Heatmap,
    HeatmapMessage,
    HeatmapOutput,
    HeatmapState,
    Histogram,
    HistogramMessage,
    HistogramState,
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
    ThresholdLine,
    Timeline,
    TimelineEvent,
    TimelineMessage,
    TimelineSpan,
    TimelineState,
};
use ratatui::Terminal;
use ratatui::prelude::*;

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
            Sparkline::view(&state, frame, frame.area(), &theme, &ViewContext::default());
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
                Gauge::view(&state, frame, frame.area(), &theme, &ViewContext::default());
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
            Heatmap::view(&state, frame, frame.area(), &theme, &ViewContext::default());
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
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
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
            Timeline::view(&state, frame, frame.area(), &theme, &ViewContext::default());
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
            SpanTree::view(&state, frame, frame.area(), &theme, &ViewContext::default());
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
            FlameGraph::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

// ============================================================================
// 9. EventStream: push events, filter by level, verify visible count
// ============================================================================

#[test]
fn test_event_stream_push_and_level_filter() {
    let mut state = EventStreamState::new().with_title("System Events");

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
            AlertPanel::view(&state, frame, frame.area(), &theme, &ViewContext::default());
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
            Calendar::view(&state, frame, frame.area(), &theme, &ViewContext::default());
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

    // Toggle back on
    let output = Switch::update(&mut state, SwitchMessage::Toggle);
    assert_eq!(output, Some(SwitchOutput::On));
    assert!(state.is_on());

    // Render
    let backend = CaptureBackend::new(20, 1);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = envision::Theme::default();
    terminal
        .draw(|frame| {
            Switch::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}
