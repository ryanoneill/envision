# Choosing the Right Component

Envision has 74 components. This guide helps you find the right one.

## I want to show a list of things

| Use case | Component | Why |
|----------|-----------|-----|
| Simple scrollable list with selection | [`SelectableList`] | Basic list with vim/arrow navigation |
| List with search/filter | [`SearchableList`] | Combines a text filter with `SelectableList` |
| List with per-item loading/error states | [`LoadingList`] | Each item has Pending/Ready/Error state |
| Hierarchical/nested items | [`Tree`] | Expand/collapse with indentation |
| Tabular data with columns | [`Table`] | Sortable columns, row selection |
| Editable tabular data | [`DataGrid`] | Cell-level editing and navigation |
| Fuzzy-searchable action picker | [`CommandPalette`] | Overlay-style, like VS Code's Ctrl+P |

## I want text input

| Use case | Component | Why |
|----------|-----------|-----|
| Single-line text | [`LineInput`] | History, clipboard, selection |
| Single-line with field label | [`InputField`] | `LineInput` + border + label |
| Multi-line text editing | [`TextArea`] | Undo/redo, line numbers, selection |
| Number with min/max/step | [`NumberInput`] | Validates numeric input with range |
| Dropdown with search | [`Dropdown`] | Type to filter, then select |
| Dropdown without search | [`Select`] | Simple pick-one-from-list |

## I want to show data visualization

| Use case | Component | Why |
|----------|-----------|-----|
| Line, bar, area, or scatter chart | [`Chart`] | Multi-series, annotations, error bars |
| Distribution/histogram | [`Histogram`] | Adaptive binning (Sturges/Scott/Freedman-Diaconis) |
| Heatmap / 2D grid | [`Heatmap`] | Color scales (Viridis, Inferno, diverging) |
| Box-and-whisker plots | [`BoxPlot`] | Statistical summaries with outliers |
| Inline trend line | [`Sparkline`] | Compact, fits in a status bar |
| Flame graph | [`FlameGraph`] | Hierarchical profiling data |
| Graph / topology visualization | [`Diagram`] | Sugiyama hierarchical and force-directed layout, spatial navigation, edge following, node search, clusters, minimap, viewport pan/zoom, edge styles, node shapes, status indicators |
| Treemap | [`Treemap`] | Proportional area visualization |
| Timeline / Gantt | [`Timeline`] | Events and spans on a time axis |
| Span/trace tree | [`SpanTree`] | Distributed tracing hierarchies |

## I want navigation / layout

| Use case | Component | Why |
|----------|-----------|-----|
| Tab switching (simple) | [`Tabs`] | Horizontal tabs, generic type |
| Tab switching (rich — closable, icons) | [`TabBar`] | Editor-style with overflow scrolling |
| Menu with keyboard shortcuts | [`Menu`] | Nested items with enabled/disabled |
| Breadcrumb path | [`Breadcrumb`] | Navigate hierarchical paths |
| Step/wizard progress | [`StepIndicator`] | Pipeline visualization with per-step styles |
| Multi-screen routing | [`Router`] | Navigation with history stack |
| Page indicators | [`Paginator`] | Dots/numbers/compact styles |
| Resizable split pane | [`SplitPanel`] | Two panels with draggable divider |
| Resizable multi-pane | [`PaneLayout`] | N panes with proportional sizing |
| Accordion panels | [`Accordion`] | Expand/collapse sections |

## I want to show status / progress

| Use case | Component | Why |
|----------|-----------|-----|
| Progress bar | [`ProgressBar`] | With ETA, rate, label |
| Multiple progress bars | [`MultiProgress`] | Track concurrent tasks |
| Gauge / meter | [`Gauge`] | Ratio display with thresholds |
| Spinner | [`Spinner`] | Multiple animation styles |
| Status bar | [`StatusBar`] | Bottom bar with left/center/right sections |
| Status log | [`StatusLog`] | Timestamped messages |
| Toast notifications | [`Toast`] | Timed popups with levels |

## I want to display content

| Use case | Component | Why |
|----------|-----------|-----|
| Scrollable text | [`ScrollableText`] | Read-only, scrollable |
| Rich/styled text | [`StyledText`] | Inline styles and colors |
| Markdown | [`MarkdownRenderer`] | Headings, bold, code, lists (requires `markdown` feature) |
| Code with syntax hints | [`CodeBlock`] | Line numbers, highlight lines |
| Diff view | [`DiffViewer`] | Side-by-side or unified |
| Large block text | [`BigText`] | Block-character rendering |
| Title with subtitle | [`TitleCard`] | Decorative header |
| AI conversation | [`ConversationView`] | Role colors, code blocks, streaming |
| Terminal output | [`TerminalOutput`] | ANSI-capable output display |
| Resource usage | [`UsageDisplay`] | CPU/memory/disk metrics |

## I want overlays / dialogs

| Use case | Component | Why |
|----------|-----------|-----|
| Yes/No confirmation | [`ConfirmDialog`] | Preset buttons |
| Custom dialog | [`Dialog`] | Custom buttons and content |
| Tooltip | [`Tooltip`] | Positioned, auto-dismiss |
| Command palette | [`CommandPalette`] | Fuzzy search overlay |
| Help panel | [`HelpPanel`] | Keyboard shortcut reference |

## I want logs / events

| Use case | Component | Why |
|----------|-----------|-----|
| Filterable log viewer | [`LogViewer`] | Search, regex, follow mode |
| Real-time event stream | [`EventStream`] | Levels, timestamps, source |
| Multi-stream correlation | [`LogCorrelation`] | Synchronized scroll across streams |
| Alert dashboard | [`AlertPanel`] | Metrics with thresholds and sparklines |
| Metrics dashboard | [`MetricsDashboard`] | Charts, counters, gauges in a grid |

## I want to manage focus

| Component | Purpose |
|-----------|---------|
| [`FocusManager`] | Coordinates Tab/Shift+Tab focus cycling across components |

Pass `focused: true/false` via `RenderContext` and `EventContext` to
each component. `FocusManager` tracks which component is focused;
your `view()` reads the focus state and passes it through.

## Still not sure?

- **`SelectableList` vs `SearchableList`**: If users need to filter by typing, use `SearchableList`. Otherwise `SelectableList`.
- **`Select` vs `Dropdown`**: `Select` is a basic pick-list. `Dropdown` adds type-to-filter.
- **`Tabs` vs `TabBar`**: `Tabs` is minimal (label + selection). `TabBar` adds close buttons, icons, modified indicators, overflow scrolling.
- **`Table` vs `DataGrid`**: `Table` is read-only with sorting. `DataGrid` adds cell-level editing.
- **`ConversationView` vs `ScrollableText`**: Use `ConversationView` for multi-role chat with structured message blocks. Use `ScrollableText` for simple read-only text.
