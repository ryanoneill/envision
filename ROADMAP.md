# Envision Roadmap Plan

## Context

Envision is at v0.7.0 with 42 components, 5000+ tests, and excellent API consistency. The previous roadmap (based on v0.5.0 audit) is ~90% complete and has been removed. This roadmap establishes the next phase of development focused on three goals:

1. **Virtual scrolling infrastructure** — the single biggest gap for production use with real data sizes
2. **Data visualization components** — making envision the best TUI framework for charts, graphs, and analytical displays
3. **Novel observability components** — things no TUI framework offers, positioning envision as uniquely powerful for building monitoring, tracing, and log analysis tools

The roadmap is sequenced by dependencies: foundational infrastructure first, then primitives that build on it, then compound components that compose primitives.

---

## Iteration 1: Virtual Scrolling Infrastructure

**Why first**: Every list, table, tree, and log component currently allocates O(n) items per frame. This blocks all observability use cases (large logs, long trace spans, high-cardinality metrics). This must land before new data-heavy components are built.

### 1.1 ScrollState Core

New module `src/scroll/mod.rs` with a `ScrollState` struct that provides:

- Scroll offset tracking with clamping
- Visible range calculation (`visible_range() -> Range<usize>`)
- Selection tracking (`ensure_visible(index)` to keep selection in viewport)
- Scroll operations (`scroll_up`, `scroll_down`, `page_up`, `page_down`, `scroll_to_start`, `scroll_to_end`)
- Query methods (`can_scroll`, `at_start`, `at_end`, `max_offset`)
- ratatui `ScrollbarState` generation for scrollbar rendering
- `render_scrollbar()` and `render_scrollbar_inside_border()` helpers
- Serialization support behind feature flag
- Full unit test suite

**Files**: `src/scroll/mod.rs` (new), `src/scroll/tests.rs` (new), `src/lib.rs` (re-exports)

### 1.2 Retrofit Offset-Based Components

These already have `scroll_offset: usize` and use `.skip().take()` — lowest risk migration:

- **ScrollableText** — replace `scroll_offset` with `ScrollState`, add scrollbar
- **LogViewer** — replace `scroll_offset` with `ScrollState`, add scrollbar
- **ChatView** — replace `scroll_offset` with `ScrollState`, integrate with auto_scroll

### 1.3 Retrofit Selection-Based Components

These currently create ListItem/Row for ALL items — highest performance impact:

- **SelectableList** — add `ScrollState`, render only `visible_range()` items, adjust ListState selection relative to window
- **Table** — same pattern with TableState
- **LoadingList** — same pattern as SelectableList
- **DataGrid** — same pattern as Table
- **SearchableList** — update render.rs to use ScrollState

### 1.4 Retrofit Tree

- Add `ScrollState` to `TreeState<T>`
- Render only lines in visible range from flattened output
- Use `ensure_visible(selected_index)` for selection tracking

### 1.5 Scrollbar Theme Integration

- Add scrollbar style methods to `Theme` (thumb color, track color)
- Ensure consistent scrollbar appearance across all retrofitted components

---

## Iteration 2: Data Visualization Primitives

**Why second**: These are the building blocks for all visualization components. ratatui already has underlying widgets (Sparkline, Gauge, Canvas) that envision doesn't yet wrap. Low effort, high value.

### 2.1 Sparkline Component

Compact inline data trend display (1-2 rows tall). Wraps ratatui's `Sparkline` widget. Distinct from Chart — designed for embedding in dashboards, status bars, and table cells.

- State: `Vec<u64>` data, optional label, max display points, direction (L→R or R→L)
- Push/clear data, bounded push with max length
- Display-only (no focus/interaction needed)

### 2.2 Gauge Component

Ratio/percentage display with visual fill bar and centered label. Wraps ratatui's `Gauge` and `LineGauge`. Distinct from ProgressBar — shows ratios and measurements, not task progress.

- Two variants: full Gauge (block fill) and LineGauge (single-line compact)
- Configurable threshold zones (green/yellow/red) with custom breakpoints
- Units display ("75% CPU", "3.2 GB / 8 GB")
- Optional label, color customization per zone

### 2.3 Canvas Component

General-purpose drawing surface wrapping ratatui's `Canvas`. Enables custom visualizations without building new components.

- Drawing primitives: Line, Rectangle, Circle, Points, Label
- Coordinate system with x/y bounds
- Custom `Shape` trait support for user-defined shapes
- Focusable for pan/zoom interaction
- Foundation for Heatmap, ScatterPlot, FlameGraph, etc.

### 2.4 Enhanced Chart

Extend existing Chart component with missing capabilities:

- **Area chart** mode (filled line chart)
- **Stacked bar** and **grouped bar** modes
- **Scatter plot** mode with labeled points
- **Threshold/reference lines** (horizontal markers at configurable values)
- **Shared Y-axis** for multi-series line charts (currently stacks vertically)
- Manual min/max scaling (currently always auto-scales)
- Data point interaction (navigate to individual points, emit selection)
- Use ratatui's full `Chart` widget with `Axis` for proper axes rendering

---

## Iteration 3: New General-Purpose Components

**Why third**: Fills the gaps that make envision feel incomplete compared to other frameworks. These are all broadly useful, not observability-specific.

### Input Components

**3.1 Slider** — Numeric range selection with keyboard control. Horizontal/vertical orientation, configurable min/max/step, visual track and thumb.

**3.2 Switch/Toggle** — On/off toggle, visually distinct from checkbox. Animated slide between states.

**3.3 NumberInput** — Numeric input with validation, increment/decrement (Up/Down arrows), optional min/max bounds, step size.

**3.4 DatePicker** — Date selection combining a Calendar widget with input. Month navigation, day selection, configurable date format.

### Display Components

**3.5 Calendar** — Month view with date selection and event markers. Wraps ratatui's `Calendar` widget with envision's Component pattern.

**3.6 BigText / Digits** — Large pixel text rendering for dashboard hero numbers. Think: big counter displays, clocks, KPI values.

**3.7 Divider / Rule** — Horizontal or vertical separator line with optional label. Simple but universally needed for layout.

**3.8 Paginator** — Page navigation indicator ("Page 3 of 12", "Showing 51-100 of 2,847"). Works with virtual scrolling.

### Navigation Components

**3.9 CommandPalette** — Searchable, fuzzy-filtered action/item picker. The most impactful UX pattern in modern tools (k9s, Helix, VS Code). Input field + filtered list + preview area. Generic over action/item type.

**3.10 HelpPanel** — Auto-generated keybinding display from a list of (key, description) pairs. Overlay or inline. Every serious TUI app needs this.

### Container Components

**3.11 ScrollView / Viewport** — Generic scrollable container for arbitrary widget content. The missing layout primitive — wraps any content in a scrollable viewport with virtual scrolling.

**3.12 Collapsible** — Single expandable/collapsible section with header. Distinct from Accordion (which is multi-section). More composable.

---

## Iteration 4: Statistical Visualization

**Why fourth**: Builds on Canvas and enhanced Chart from Iteration 2. These are the analytical primitives that make envision uniquely powerful for data-heavy applications.

### 4.1 Histogram

Binned frequency distribution display. Distinct from bar chart — automatic binning of continuous data, bucket labels, configurable bin count/width.

Use cases: latency distribution, request size distribution, error frequency.

### 4.2 Heatmap

2D color-intensity grid. **No TUI framework has this as a reusable widget.**

- Row and column labels
- Configurable color scale (sequential, diverging)
- Cell value display on hover/selection
- Keyboard navigation between cells

Use cases: GitHub-style contribution graphs, correlation matrices, error rate by hour x day, latency percentiles.

### 4.3 ScatterPlot

First-class scatter plot with labeled points, optional clusters, and trend lines. Builds on Canvas.

Use cases: correlation analysis, anomaly detection, latency vs throughput.

### 4.4 BoxPlot

Statistical distribution visualization showing median, quartiles, outliers. Multiple box plots side-by-side for comparison.

Use cases: P50/P95/P99 latency comparison across services, response time distributions.

---

## Iteration 5: Observability Components

**Why fifth**: These are the novel, differentiating components. They compose primitives from Iterations 1-4 into compound components purpose-built for observability — but designed as general-purpose building blocks.

### 5.1 Timeline

Horizontal timeline with events/spans plotted along a time axis. Zoomable, pannable, with event markers.

- Time-axis rendering with auto-scaling (ms/s/min/hr)
- Event markers with severity/category coloring
- Span bars (start time + duration)
- Zoom in/out, scroll left/right
- Selection and detail display

Use cases: distributed trace visualization (like Jaeger UI), deployment timelines, incident timelines, CI/CD pipelines.

### 5.2 SpanTree

Hierarchical tree with horizontal timing bars — essentially a Gantt chart for trace spans. Combines Tree navigation with Timeline rendering.

- Tree structure with expand/collapse
- Duration bars aligned to a shared time axis
- Color-coded by service/status
- Selection emits span details

Use cases: distributed tracing (the primary Jaeger/Zipkin view), profiler output, task dependency visualization.

### 5.3 FlameGraph

Interactive flame graph rendering in the terminal. **No TUI framework offers this as a composable widget.**

- Stack frame rendering with width proportional to time/samples
- Zoom into subtrees
- Search/highlight frames matching a pattern
- Color by: package, self-time, frequency

Use cases: CPU profiling, memory allocation analysis, call stack visualization.

### 5.4 EventStream

Real-time filterable event feed with severity coloring and structured fields. More structured than LogViewer — each event has typed key-value fields, not just text.

- Structured event display (timestamp, level, fields as columns)
- Dynamic column visibility
- Real-time append with rate indicator
- Pattern detection (highlight repeated events, group bursts)
- Virtual scrolling from day one

Use cases: structured logging, audit trails, real-time event monitoring.

### 5.5 AlertPanel

Metric display with configurable threshold states (OK/Warning/Critical/Unknown). Visual state transitions with history.

- Metric value with threshold zones
- State badge (colored indicator)
- Sparkline history
- Time-in-state tracking
- Configurable thresholds per metric

Use cases: SLI/SLO dashboards, infrastructure health panels, alerting displays.

---

## Iteration 6: Advanced Visualization

**Why sixth**: These are complex compound components that build on everything before them. High effort, high differentiation.

### 6.1 Treemap

Nested rectangles showing hierarchical proportions. Builds on Canvas.

Use cases: disk usage, memory allocation by module, request volume by service/endpoint.

### 6.2 Sankey Diagram

Flow visualization between categories. Shows magnitude of flow between nodes.

Use cases: request routing visualization, data pipeline flows, traffic distribution.

### 6.3 DiffViewer

Side-by-side or unified diff display with hunk navigation. Every git TUI builds this ad-hoc — none offer it as a reusable widget.

- Unified and side-by-side modes
- Hunk navigation (next/prev change)
- Line-level highlighting (added/removed/modified)
- Virtual scrolling for large diffs

### 6.4 DependencyGraph

Service/component relationship visualization. Directed graph with nodes and edges.

- Node layout (force-directed or hierarchical)
- Edge rendering with direction indicators
- Node status coloring (healthy/degraded/down)
- Selection and detail display
- Zoom and pan

Use cases: service mesh topology, dependency health, architecture visualization.

### 6.5 LogCorrelation

Side-by-side time-aligned log streams from multiple sources. Filter independently but scroll in sync.

- Multiple log panes (2-4 sources)
- Time-synchronized scrolling
- Independent filtering per pane
- Cross-pane search highlighting
- Shared timeline ruler

Use cases: distributed system debugging, multi-service log analysis.

---

## Iteration 7: Existing Component Enhancements

**Why last**: These improve existing components but aren't blocking new work. Can be interleaved with other iterations as needed.

### 7.1 LogViewer Enhancements
- Regex search (not just substring)
- Structured field columns (key=value parsing)
- Follow mode indicator (tail -f style)
- Context lines around search matches
- Timestamp/date-range filtering
- Search history

### 7.2 Chart Enhancements
- Animation/transitions between data updates
- Legend toggle (click to show/hide series)
- Tooltip on data point selection
- Export data to clipboard

### 7.3 Table Enhancements
- Multi-column sort
- Custom sort comparators (numeric, date, etc.)
- Column resizing via keyboard
- Row grouping/aggregation
- Row expansion (detail view)

### 7.4 MetricsDashboard Enhancements
- Configurable threshold breakpoints (not hardcoded 70/90%)
- Units display on metric values
- Visual gauge bars (not just text)
- Comparison/delta display
- Error/degraded widget states
- Timestamped history points

### 7.5 DataGrid Enhancements
- Cell validation hooks
- Read-only columns
- Column hiding/reordering
- Multi-cell selection
- Undo/redo for edits

### 7.6 TextArea / InputField Enhancements
- Syntax highlighting (for code input)
- Line numbers
- Find/replace

---

## Dependency Graph

```
Iteration 1 (Virtual Scrolling)
    |
    +---> Iteration 2 (Viz Primitives: Sparkline, Gauge, Canvas, Enhanced Chart)
    |         |
    |         +---> Iteration 4 (Statistical: Histogram, Heatmap, ScatterPlot, BoxPlot)
    |         |         |
    |         |         +---> Iteration 6 (Advanced: Treemap, Sankey, DependencyGraph)
    |         |
    |         +---> Iteration 5 (Observability: Timeline, SpanTree, FlameGraph, EventStream, AlertPanel)
    |                   |
    |                   +---> Iteration 6 (Advanced: LogCorrelation, DiffViewer)
    |
    +---> Iteration 3 (General-Purpose: CommandPalette, Slider, Calendar, etc.)
    |
    +---> Iteration 7 (Existing Component Enhancements) [can be interleaved anytime]
```

## Component Count Projection

| Category | Current | After Roadmap |
|----------|---------|---------------|
| Input | 8 | 12 (+Slider, Switch, NumberInput, DatePicker) |
| Data | 4 | 4 (enhanced, not new) |
| Display | 10 | 17 (+Sparkline, Gauge, BigText, Divider, Paginator, Calendar, Heatmap) |
| Navigation | 6 | 8 (+CommandPalette, HelpPanel) |
| Overlay | 3 | 3 |
| Compound | 9 | 21 (+Canvas, Histogram, ScatterPlot, BoxPlot, Timeline, SpanTree, FlameGraph, EventStream, AlertPanel, Treemap, Sankey, DiffViewer, DependencyGraph, LogCorrelation, ScrollView, Collapsible) |
| Infrastructure | 1 | 2 (+ScrollState) |
| **Total** | **42** | **67** |

## Implementation Notes

- Each new component follows the established pattern: `src/component/<name>/mod.rs` + `tests.rs`
- Feature flags: new visualization components under `visualization-components`, new observability components under `observability-components`, both included in `full`
- Every component gets: unit tests, snapshot tests, at least one example, doc tests
- Virtual scrolling is integrated via composition (embed `ScrollState` in component state), not inheritance
- Canvas-based components (Heatmap, ScatterPlot, FlameGraph, Treemap, Sankey, DependencyGraph) share the Canvas primitive
- All new components support the existing trait system (Focusable, Disableable, Toggleable where appropriate)
