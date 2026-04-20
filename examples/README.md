# Envision Examples

A catalog of runnable examples for every component and feature in the Envision TUI framework.

Run any example with:

```sh
cargo run --example <name>
```

---

## Input Components

| Component | Example | Description |
|-----------|---------|-------------|
| Button | [button.rs](button.rs) | Clickable button with keyboard activation and focus styling |
| Checkbox | [checkbox.rs](checkbox.rs) | Toggleable checkbox with keyboard activation |
| Dropdown | [dropdown.rs](dropdown.rs) | Drop-down selector with filterable options |
| InputField | [input_field.rs](input_field.rs) | Text input field with cursor navigation and editing |
| LineInput | [line_input.rs](line_input.rs) | Single-line text input for prompts and forms |
| NumberInput | [number_input.rs](number_input.rs) | Numeric input with increment/decrement and range validation |
| RadioGroup | [radio_group.rs](radio_group.rs) | Single-selection radio button group |
| Select | [select.rs](select.rs) | Inline selection widget with keyboard navigation |
| Slider | [slider.rs](slider.rs) | Adjustable slider for numeric value selection |
| Switch | [switch.rs](switch.rs) | Toggle switch for boolean on/off state |
| TextArea | [text_area.rs](text_area.rs) | Multi-line text editor with cursor and scrolling |
| Form | [form.rs](form.rs) | Compound form with multiple field types and validation |

## Data Components

| Component | Example | Description |
|-----------|---------|-------------|
| SelectableList | [selectable_list.rs](selectable_list.rs) | Scrollable list with keyboard-driven selection |
| SearchableList | [searchable_list.rs](searchable_list.rs) | Filterable list with built-in search input |
| LoadingList | [loading_list.rs](loading_list.rs) | List with per-item loading states for async data |
| Table | [table.rs](table.rs) | Sortable, scrollable table with column headers |
| DataGrid | [data_grid.rs](data_grid.rs) | Spreadsheet-like grid with cell navigation and editing |
| Tree | [tree.rs](tree.rs) | Hierarchical tree view with expand/collapse |

## Display Components

| Component | Example | Description |
|-----------|---------|-------------|
| BigText | [big_text.rs](big_text.rs) | Large ASCII-art text rendering |
| Calendar | [calendar.rs](calendar.rs) | Month calendar with date selection and navigation |
| Canvas | [canvas.rs](canvas.rs) | Freeform drawing canvas with shapes and markers |
| CodeBlock | [code_block.rs](code_block.rs) | Syntax-highlighted code display with line numbers |
| Collapsible | [collapsible.rs](collapsible.rs) | Expandable/collapsible content section |
| Divider | [divider.rs](divider.rs) | Horizontal or vertical separator line |
| Gauge | [gauge.rs](gauge.rs) | Progress gauge with threshold zones and variants |
| HelpPanel | [help_panel.rs](help_panel.rs) | Grouped key binding reference panel |
| KeyHints | [key_hints.rs](key_hints.rs) | Compact key hint bar for contextual shortcuts |
| MarkdownRenderer | [markdown_renderer.rs](markdown_renderer.rs) | Rich markdown rendering with headings, lists, and code |
| MultiProgress | [multi_progress.rs](multi_progress.rs) | Multiple concurrent progress bars with status tracking |
| Paginator | [paginator.rs](paginator.rs) | Page navigation with customizable page size |
| ProgressBar | [progress_bar.rs](progress_bar.rs) | Animated progress bar with ETA formatting |
| ScrollView | [scroll_view.rs](scroll_view.rs) | Generic scrollable viewport for arbitrary content |
| ScrollableText | [scrollable_text.rs](scrollable_text.rs) | Scrollable multi-line text display |
| Sparkline | [sparkline.rs](sparkline.rs) | Compact inline data trend visualization |
| Spinner | [spinner.rs](spinner.rs) | Animated loading spinner with multiple styles |
| StatusBar | [status_bar.rs](status_bar.rs) | Application status bar with configurable sections |
| StatusLog | [status_log.rs](status_log.rs) | Scrollable log of timestamped status entries |
| StyledText | [styled_text.rs](styled_text.rs) | Rich text with inline styling and formatting |
| TerminalOutput | [terminal_output.rs](terminal_output.rs) | ANSI-aware terminal output display |
| TitleCard | [title_card.rs](title_card.rs) | Decorative title card for section headers |
| Toast | [toast.rs](toast.rs) | Temporary notification popup with auto-dismiss |
| UsageDisplay | [usage_display.rs](usage_display.rs) | Resource usage metrics display with layouts |

## Visualization Components

| Component | Example | Description |
|-----------|---------|-------------|
| BoxPlot | [box_plot.rs](box_plot.rs) | Statistical box-and-whisker plot |
| Chart | [chart.rs](chart.rs) | Line, bar, and area charts with axes and legends |
| Chart (Enhanced) | [chart_enhanced.rs](chart_enhanced.rs) | Advanced chart with thresholds and multiple series |
| Heatmap | [heatmap.rs](heatmap.rs) | Color-scaled grid heatmap for matrix data |
| Histogram | [histogram.rs](histogram.rs) | Frequency distribution histogram |
| Treemap | [treemap.rs](treemap.rs) | Hierarchical treemap for proportional data |

## Observability Components

| Component | Example | Description |
|-----------|---------|-------------|
| AlertPanel | [alert_panel.rs](alert_panel.rs) | Alert dashboard with thresholds and metric states |
| EventStream | [event_stream.rs](event_stream.rs) | Live event feed with level filtering |
| FlameGraph | [flame_graph.rs](flame_graph.rs) | Interactive flame graph for profiling data |
| LogCorrelation | [log_correlation.rs](log_correlation.rs) | Multi-stream correlated log viewer |
| LogViewer | [log_viewer.rs](log_viewer.rs) | Searchable, filterable log viewer |
| MetricsDashboard | [metrics_dashboard.rs](metrics_dashboard.rs) | Multi-metric monitoring dashboard |
| SpanTree | [span_tree.rs](span_tree.rs) | Distributed tracing span tree visualization |
| Timeline | [timeline.rs](timeline.rs) | Event and span timeline with selection |

## Navigation Components

| Component | Example | Description |
|-----------|---------|-------------|
| Accordion | [accordion.rs](accordion.rs) | Expandable accordion panels with single/multi mode |
| Breadcrumb | [breadcrumb.rs](breadcrumb.rs) | Breadcrumb trail for hierarchical navigation |
| CommandPalette | [command_palette.rs](command_palette.rs) | Fuzzy-search command palette overlay |
| Menu | [menu.rs](menu.rs) | Vertical menu with keyboard navigation |
| Router | [router.rs](router.rs) | Client-side route management for multi-view apps |
| StepIndicator | [step_indicator.rs](step_indicator.rs) | Multi-step wizard progress indicator |
| TabBar | [tab_bar.rs](tab_bar.rs) | Horizontal tab bar with closeable tabs |
| Tabs | [tabs.rs](tabs.rs) | Generic typed tab component with selection |

## Overlay Components

| Component | Example | Description |
|-----------|---------|-------------|
| ConfirmDialog | [confirm_dialog.rs](confirm_dialog.rs) | Yes/No confirmation dialog overlay |
| Dialog | [dialog.rs](dialog.rs) | Modal dialog with configurable buttons |
| Tooltip | [tooltip.rs](tooltip.rs) | Contextual tooltip with positioning options |

## Code and AI Components

| Component | Example | Description |
|-----------|---------|-------------|
| ChatView | [chat_view.rs](chat_view.rs) | Chat interface with input field and message history |
| ConversationView | [conversation_view.rs](conversation_view.rs) | Read-only AI conversation display with structured blocks |
| Diagram | [diagram.rs](diagram.rs) | Graph visualization with layout algorithms |
| DiffViewer | [diff_viewer.rs](diff_viewer.rs) | Side-by-side and unified diff viewer |
| FileBrowser | [file_browser.rs](file_browser.rs) | File system browser with directory navigation |
| PaneLayout | [pane_layout.rs](pane_layout.rs) | Resizable multi-pane layout manager |
| SplitPanel | [split_panel.rs](split_panel.rs) | Two-pane split view with adjustable divider |

## Application Demos

| Example | Description |
|---------|-------------|
| [annotations.rs](annotations.rs) | Demonstrates the annotation and accessibility system |
| [async_counter.rs](async_counter.rs) | Async counter showcasing Command::spawn for background tasks |
| [beautiful_dashboard.rs](beautiful_dashboard.rs) | Polished dashboard combining multiple components |
| [capture_backend.rs](capture_backend.rs) | Headless rendering with CaptureBackend for testing |
| [chat_app.rs](chat_app.rs) | Full chat application with markdown rendering |
| [chat_markdown_demo.rs](chat_markdown_demo.rs) | Markdown rendering inside chat messages |
| [component_showcase.rs](component_showcase.rs) | Gallery of all components on a single screen |
| [counter_app.rs](counter_app.rs) | Minimal TEA counter to get started |
| [dashboard_demo.rs](dashboard_demo.rs) | Metrics dashboard with live-updating data |
| [production_app.rs](production_app.rs) | Production-grade app structure with error handling |
| [styling_showcase.rs](styling_showcase.rs) | Theme and styling demonstration |
| [test_harness.rs](test_harness.rs) | AppHarness for headless integration testing |
| [themed_app.rs](themed_app.rs) | Theme switching between default and Nord themes |
