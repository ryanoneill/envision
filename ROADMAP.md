# Envision Roadmap

## Current State (v0.7.0)

Envision has **72 components**, **5000+ tests**, and comprehensive API consistency built on The Elm Architecture (TEA). The original roadmap from v0.5.0 has been substantially completed through six major iterations of development.

### Completed Iterations

#### Iteration 1: Virtual Scrolling Infrastructure -- COMPLETE

`ScrollState` core module landed with offset tracking, visible range calculation, selection tracking, scrollbar rendering helpers, and full test suite. Retrofitted across offset-based components (ScrollableText, LogViewer, ChatView) and selection-based components (SelectableList, Table, LoadingList, DataGrid, SearchableList, Tree). Scrollbar theme integration completed.

#### Iteration 2: Data Visualization Primitives -- COMPLETE

- **Sparkline** -- compact inline data trend display wrapping ratatui's Sparkline widget
- **Gauge** -- ratio/percentage display with Full and Line variants, threshold zones, units
- **Canvas** -- general-purpose drawing surface with primitives (Line, Rectangle, Circle, Points, Label)
- **Enhanced Chart** -- area chart, scatter plot, stacked/grouped bar modes, threshold lines, shared Y-axis

#### Iteration 3: New General-Purpose Components -- MOSTLY COMPLETE

**Completed:**
- **Slider** -- numeric range selection with keyboard control
- **Switch** -- on/off toggle, visually distinct from checkbox
- **NumberInput** -- numeric input with validation, increment/decrement, bounds
- **Calendar** -- month view with date selection and event markers
- **BigText** -- large pixel text rendering for dashboard hero numbers
- **Divider** -- horizontal/vertical separator with optional label
- **Paginator** -- page navigation indicator
- **CommandPalette** -- searchable, fuzzy-filtered action/item picker
- **HelpPanel** -- auto-generated keybinding display
- **Collapsible** -- single expandable/collapsible section with header

**Not yet implemented:**
- DatePicker (date selection combining Calendar with input)
- ScrollView / Viewport (generic scrollable container for arbitrary content)

#### Iteration 4: Statistical Visualization -- MOSTLY COMPLETE

**Completed:**
- **Histogram** -- binned frequency distribution display
- **Heatmap** -- 2D color-intensity grid with row/column labels, color scales, keyboard navigation

**Not yet implemented:**
- ScatterPlot (first-class scatter with labeled points, clusters, trend lines)
- BoxPlot (statistical distribution: median, quartiles, outliers)

#### Iteration 5: Observability Components -- COMPLETE

- **Timeline** -- horizontal timeline with events/spans, zoom, pan, selection
- **SpanTree** -- hierarchical tree with timing bars (Gantt-style trace visualization)
- **FlameGraph** -- interactive flame graph with zoom, search, color-by modes
- **EventStream** -- real-time filterable event feed with structured fields
- **AlertPanel** -- metric display with threshold states (OK/Warning/Critical)

#### Iteration 6: Advanced Visualization -- MOSTLY COMPLETE

**Completed:**
- **Treemap** -- nested rectangles showing hierarchical proportions
- **DiffViewer** -- side-by-side and unified diff display with hunk navigation
- **DependencyGraph** -- service/component relationship visualization with directed graph
- **LogCorrelation** -- side-by-side time-aligned log streams with synchronized scrolling

**Not yet implemented:**
- Sankey Diagram (flow visualization between categories)

#### Additional Components (Beyond Original Roadmap)

These components were added outside the original iteration plan, including the "Claude Code" family of components built for AI-assisted TUI development:

- **CodeBlock** -- syntax-highlighted code display with line numbers, scrolling, 10+ languages
- **TerminalOutput** -- ANSI-aware terminal output rendering
- **MarkdownRenderer** -- Markdown rendering with headings, lists, code blocks, emphasis
- **ConversationView** -- chat-style conversation display (user/assistant messages)
- **Accordion** -- multi-section expandable/collapsible container
- **Breadcrumb** -- hierarchical navigation path indicator
- **Dropdown** -- single-selection dropdown menu
- **KeyHints** -- compact keybinding hints display
- **MultiProgress** -- multiple concurrent progress tracking
- **Select** -- enhanced selection component
- **StatusBar** -- application status bar
- **StatusLog** -- timestamped status message log
- **StepIndicator** -- multi-step process progress (wizard-style)
- **StyledText** -- rich text with inline styling
- **TabBar** -- enhanced tab navigation bar
- **TitleCard** -- titled content card with borders
- **Tooltip** -- contextual tooltip overlays
- **UsageDisplay** -- resource usage visualization
- **Router** -- view routing/navigation management
- **ConfirmDialog** -- simplified confirmation dialog

---

## What's Next

### Remaining Items from Iterations 3-6

These components from the original roadmap have not yet been implemented:

| Component | Original Iteration | Description |
|-----------|-------------------|-------------|
| DatePicker | Iteration 3 | Date selection combining Calendar with input field |
| ScrollView / Viewport | Iteration 3 | Generic scrollable container for arbitrary widget content |
| ScatterPlot | Iteration 4 | First-class scatter plot with labeled points, clusters, trend lines |
| BoxPlot | Iteration 4 | Statistical distribution visualization (median, quartiles, outliers) |
| Sankey Diagram | Iteration 6 | Flow visualization between categories showing magnitude |

### Iteration 7: Existing Component Enhancements

These improve existing components. Can be tackled incrementally as needed:

- **LogViewer**: regex search, structured field columns, follow mode indicator, context lines, timestamp filtering, search history
- **Chart**: animation/transitions, legend toggle, tooltip on selection, export to clipboard
- **Table**: multi-column sort, custom comparators, column resizing, row grouping, row expansion
- **MetricsDashboard**: configurable thresholds, units display, visual gauge bars, comparison/delta display, error states
- **DataGrid**: cell validation hooks, read-only columns, column hiding/reordering, multi-cell selection, undo/redo
- **TextArea / InputField**: syntax highlighting, line numbers, find/replace

### Infrastructure

- **ratatui 0.30 support** -- Support both ratatui 0.29 and 0.30 via feature flags ([Issue #126](https://github.com/ryanoneill/envision/issues/126))
- **Crate split** -- Consider splitting into envision (core) and envision-components crates ([Issue #124](https://github.com/ryanoneill/envision/issues/124))

---

## Component Count

| Category | Count |
|----------|-------|
| Input | 12 (InputField, Checkbox, RadioGroup, Button, Slider, Switch, NumberInput, Select, Dropdown, LineInput, TextArea, Form) |
| Data | 4 (SelectableList, Table, DataGrid, LoadingList) |
| Display | 20 (ProgressBar, Spinner, Chart, Sparkline, Gauge, BigText, Divider, Paginator, Calendar, Heatmap, Histogram, CodeBlock, TerminalOutput, MarkdownRenderer, StyledText, StatusBar, StatusLog, MultiProgress, StepIndicator, UsageDisplay) |
| Navigation | 10 (Menu, Tabs, TabBar, Breadcrumb, CommandPalette, HelpPanel, KeyHints, Router, Tree, FileBrowser) |
| Overlay | 5 (Dialog, ConfirmDialog, Toast, Tooltip, TitleCard) |
| Compound | 20 (SearchableList, SplitPanel, PaneLayout, Form, LogViewer, ChatView, ConversationView, MetricsDashboard, Canvas, Timeline, SpanTree, FlameGraph, EventStream, AlertPanel, Treemap, DiffViewer, DependencyGraph, LogCorrelation, Accordion, Collapsible) |
| Infrastructure | 1 (FocusManager) |
| **Total** | **72** |
