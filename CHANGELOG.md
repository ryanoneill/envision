# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.14.0] - 2026-04-12

### Breaking

- **Crossterm event types replaced with envision-owned types.**
  `KeyCode` is now `Key`, `KeyModifiers` is now `Modifiers`, and
  `KeyEvent`/`MouseEvent` are envision-defined structs. Letter keys
  are normalized to lowercase; use `raw_char` for text input.
  `BackTab` is replaced by `Tab` with `modifiers.shift()`.
  See MIGRATION.md for the upgrade path.
- **`KeyEvent` field `key` renamed to `code`** for readability.
  `key.key` becomes `key.code` in all match expressions. The type
  is still `Key`.
- **`TerminalEventSubscription` handler now receives `Event`**
  instead of `crossterm::event::Event`. The subscription converts
  crossterm events internally before invoking the handler.
- **`Component::view` signature changed** from
  `(state, frame, area, theme, ctx)` to `(state, ctx)`. The new
  `ctx: &mut RenderContext<'_, '_>` bundles `frame`, `area`, `theme`,
  `focused`, and `disabled` into a single value. This is a large,
  cascading breakage: every component, every custom component
  implementation, every direct call to `Component::view`, and every
  test that renders a component through the public API must be
  updated. See MIGRATION.md for the upgrade path.
- **`ViewContext` renamed to `EventContext`.** It is now used only by
  `Component::handle_event`, not by `Component::view`. The fields and
  builder methods are unchanged.
- **`Component::handle_event` signature changed** from
  `(state, event, ctx: &ViewContext)` to
  `(state, event, ctx: &EventContext)`. Pure type rename.
- **`Component::traced_view` signature changed** to match the new
  `view` signature: `(state, ctx: &mut RenderContext<'_, '_>)`.
- **`Component::dispatch_event` now takes `&EventContext`** (not
  `&ViewContext`). Pure type rename.
- **`ChartGrid::render` signature changed** to take
  `&mut RenderContext<'_, '_>` as the sole non-`self` argument
  (previously took `frame`, `area`, `theme`, `ctx`).
- **`Router::view` signature changed** to take
  `(state, ctx: &mut RenderContext<'_, '_>)`.
- **`ConversationView::view_from` signature changed** to take
  `(source, state, ctx: &mut RenderContext<'_, '_>)`.

### Breaking (Worker module)

- **`ProgressSender` is now generic**: `ProgressSender<P>` where `P` is
  any `Send + 'static` type. The old `send_percentage()` and
  `send_status()` convenience methods are removed. Use
  `sender.send(WorkerProgress::new(0.5, None))` for percentage+string,
  or define your own progress enum for richer updates.

### Added

- `Command::subscribe(BoxedSubscription<M>)` for registering
  subscriptions dynamically from within `update()`. This unblocks the
  worker module for on-demand background tasks — return
  `Command::subscribe(sub)` from `update()` and the runtime registers
  the subscription on the next command processing cycle.
- `ProgressSender::new(tx)` public constructor for creating a
  `ProgressSender` outside of `WorkerBuilder` (testing, channel
  bridging).
- `ProgressSender::try_send()` for non-blocking fire-and-forget
  progress updates. Use for high-frequency ticks where dropping one is
  better than applying backpressure.
- `RenderContext<'frame, 'buf>` type in `envision::component`
  that bundles `frame`, `area`, `theme`, `focused`, and `disabled`.
  Provides builder methods (`focused`, `disabled`), sub-area reborrow
  (`with_area`), a `render_widget` convenience method, and an
  `event_context()` slice method for calls that need an
  `EventContext`.
- `EventContext` type (renamed from `ViewContext`) in
  `envision::component`, now used exclusively by
  `Component::handle_event`. `From<&RenderContext<'_, '_>>` impl is
  provided so `(&render_ctx).into()` yields an `EventContext`.

## [0.13.1] - 2026-04-11

### Added

- Memory allocation benchmarks for `SelectableList`, `Table`, and `Tree`
  rendering, plus state creation scaling (1000/5000/10000 items). Uses
  a counting global allocator wrapper isolated to the bench binary.
- `focus_manager` example demonstrating focus coordination across
  multiple components with Tab/Shift+Tab navigation. Brings example
  coverage to 73/73 (100%).
- Missing accessor getters: `FlameGraphState::search()`,
  `TabBarState::active()`, `TooltipState::duration()`, and
  `TreemapNode::color()` to match their existing setters.
- `PartialEq` derives on `DependencyGraphState`, `FlameGraphState`,
  `SpanTreeState`, and `FocusManager`.
- ~265 new doc tests across 21 components, raising overall doc test
  coverage from 63.6% to 84.4%.
- Terminal escape sequence handling section in `SECURITY.md`,
  documenting that envision components render through ratatui widgets
  (safe against escape injection) and noting the exception for direct
  `Span::raw()` usage with unsanitized input.

### Fixed

- `KeyEvent::new(Key::Char('G'))` now normalizes uppercase letters
  to lowercase with SHIFT modifier, matching the behavior of
  `KeyEvent::char()` and `from_crossterm_key`. Previously, test code
  using `Event::key(Key::Char('G'))` produced events that could never
  occur from real terminal input.
- `ConversationView` markdown rendering also recolors spans with
  `fg: None` (in addition to spans with the theme default foreground),
  defending against future markdown renderer changes.

### Changed

- Refactored render helper functions in `alert_panel`, `chart`,
  `code_block`, `dependency_graph`, `flame_graph`, `heatmap`, and
  `log_correlation` to use parameter structs instead of long argument
  lists. Removed all 10 `#[allow(clippy::too_many_arguments)]`
  suppressions from component code.
- Removed 6 stale clippy suppressions in `code_block`, `gauge`,
  `progress_bar`, `slider`, and `number_input` that were no longer
  necessary. Zero clippy suppressions remain in component code.
- Split `chart/tests.rs` (1115 → 416 + 712 lines) into two files
  to stay under the 1000-line limit.
- Split `line_input/mod.rs` (1109 → 769 lines) by extracting
  `update.rs` for the message-handling logic.
- Split `tree/mod.rs` (1041 → 948 lines) by extracting `render.rs`
  for the rendering helpers.

### Documentation

- Added `# Errors` and `# Panics` sections to public methods with
  non-obvious failure conditions in `App::init`, `Command::save_state`,
  and `ProgressSender::send_status`/`send_percentage`.

### Internal

- Audit tool (`tools/audit/`) now correctly detects: feature flag cfg
  gates, benchmark parameterization, manual `impl Trait for` blocks
  (Debug/Clone/Default/PartialEq), and example coverage via prelude
  imports. Five false positive detection issues fixed.

## [0.13.0] - 2026-04-09

### Added

- Per-role style overrides in `ConversationView` via
  `ConversationViewState::with_role_style(role, style)`,
  `set_role_style()`, `role_style_override()`, and `clear_role_style()`.
  Allows customizing the color/style for specific roles (User, Assistant,
  etc.) without affecting others. Falls back to the default
  `ConversationRole::color()` when no override is set.

- `AppShell` layout helper for consistent `(header, content, footer)`
  splits. Construct once at app init with
  `AppShell::new().header(Constraint::Length(4)).footer(Constraint::Length(1))`,
  then call `.split(area)` from views and overlays to get the same rects
  without duplicating layout constants. Returns `AppRegions` with
  `header`, `content`, and `footer` fields. Header and footer are
  optional — unconfigured regions produce zero-height rects.

- `StepIndicatorState::with_show_border(bool)`, `show_border()`, and
  `set_show_border(bool)` for opting out of the border box. When the
  border is disabled, `StepIndicator` becomes usable as an inline
  breadcrumb in a single-row area. Defaults to `true` so existing
  callers see no change. Matches the naming convention of
  `StyledTextState::with_show_border`. Note: when the border is
  hidden, the state's title is not rendered (the title is drawn as
  part of the border block).

- `DistributionMap` convenience API for visualizing distribution
  evolution over time via `Heatmap`.

- `ChartGrid` convenience component for multi-chart dashboard layouts
  with automatic grid sizing.

- Structured semantic output for `CaptureBackend` for AI consumption,
  including cell-level style metadata and widget annotation data.

- Point annotations and text callouts for `Chart` component via
  `ChartAnnotation` and `with_annotation()`.

- Time-axis support with custom X-axis labels for `Chart` via
  `with_x_labels()`.

- Multi-series bar chart support with grouped and stacked modes via
  `BarMode::Grouped` and `BarMode::Stacked`.

- Error bars and confidence intervals for `Chart` line/scatter series.

- Viridis, Inferno, and Plasma perceptual color scales for `Heatmap`.

- Diverging color scales (BlueWhiteRed, RedWhiteBlue) for `Heatmap`.

- Adaptive binning for `Histogram` via `BinMethod::Sturges`,
  `BinMethod::Scott`, and `BinMethod::FreedmanDiaconis`.

- Area chart fill below the curve for `Chart`.

- XY-pair support for `Chart` `DataSeries`, allowing explicit
  (x, y) coordinates instead of implicit index-based x values.

- Optional grid lines for `Chart` via `with_grid()`.

- Categorical labels for bar chart x-axis.

- Expanded chart color palette from 8 to 20 Tableau-inspired colors.

### Breaking

- `Sparkline` data type changed from `u64` to `f64` for scientific
  and ML use cases. Update calls to `SparklineState::new()` and
  `with_data()` to use `f64` values.

### Changed

- Chart rendering polish: improved legend placement, axis alignment,
  number formatting, and palette cycling.

- Split `chart/tests.rs` into two files to stay under the 1000-line
  limit. No behavioral changes.

### Fixed

- `ConversationView` now honors role colors (User=green, Assistant=blue,
  etc.) when markdown rendering is enabled. Previously, the role style was
  discarded in the markdown branch of `format_text_block`, causing all body
  text to render in the terminal default foreground regardless of role.
  Markdown-specific styling (bold, inline code) is preserved where set.

### Documentation

- Added doc tests to 12 under-documented components (key_hints, router,
  status_bar, spinner, radio_group, tabs, usage_display, text_area,
  input_field, box_plot, breadcrumb, code_block), bringing doc test
  coverage from 63.6% to 74.3%.

- Added `# Errors` and `# Panics` documentation sections to public
  methods with non-obvious failure conditions.

## [0.12.0] - 2026-04-05

### Breaking

- **`Focusable` and `Disableable` traits deleted.** Focus and disabled
  state is now exclusively controlled via `ViewContext` passed to
  `handle_event()` and `view()`. Components no longer store
  `focused: bool` or `disabled: bool` fields. Remove all calls to
  `set_focused()`, `is_focused()`, `set_disabled()`, `is_disabled()`.

- **`Component::handle_event` signature changed** from
  `(state, event)` to `(state, event, ctx: &ViewContext)`. Same for
  `dispatch_event`. Pass `&ViewContext::new().focused(true)` for
  focused components, `&ViewContext::default()` for unfocused.

- **`ChatView` component deleted.** Use `ConversationView` instead.
  ChatView examples (`chat_app`, `chat_markdown_demo`, `chat_view`)
  also removed.

- **Instance method bridges deleted.** `state.handle_event(&event)`
  and `state.dispatch_event(&event)` no longer exist. Use the static
  `Component::handle_event(&state, &event, &ctx)` form instead.
  `state.update(msg)` is preserved.

- **`#[non_exhaustive]` removed from all Output enums.** Downstream
  `match` is now exhaustive — the compiler will tell you about new
  variants on upgrade.

- **Edition 2024 and MSRV 1.85.** Bumped from edition 2021 / MSRV 1.81.

### Added

- **`MessageSource` trait**: `ConversationView::view_from()` renders
  from external message stores without mirroring into state.

- **`ConversationView::set_status()`**: Status line inside the border
  for transient info like rate-limit backoff.

- **`Language::Hcl`**: HCL/Terraform syntax highlighting in CodeBlock.

- **Horizontal scroll in CodeBlock**: `ScrollLeft`/`ScrollRight`
  messages, Left/Right/h/l key bindings. Long lines render with
  character offset instead of wrapping.

- **`MetricsDashboard::widget_by_label()`**: Look up metrics by name.

- **`LogViewerState::push_entry()` now public.**

- **`render_diff()` and `render_to_string()` test utilities** for
  visual regression testing.

- **Chart rendering improvements**: Braille line rendering, axis tick
  marks, log scale, LTTB downsampling, vertical reference lines,
  cursor/crosshair for interactive exploration.

- **`FileEntry`, `FileSortField`, `Language`** re-exported in prelude.

### Fixed

- **StatusBar overflow**: Center section truncates with ellipsis when
  content exceeds terminal width, preserving left and right sections.

- **`view_with_focus` and `view_with_disabled` removed** (dead code,
  superseded by ViewContext in v0.10.0).

- **Calendar panic replaced** with safe fallback for invalid months.

### Internal

- **cargo-nextest** for CI test execution (Windows: 21min → 9min).
- 4 oversized test files split into submodules.
- 15,137 tests, zero clippy warnings, zero unsafe code.
- Audit tool fixed: type-level doc coverage correctly reports 100%.

## [0.11.0] - 2026-04-04

### Breaking

- **`TextAreaState::with_value()` is now a chainable builder**: Takes
  `mut self` instead of being a standalone constructor. Migrate from
  `TextAreaState::with_value("...")` to
  `TextAreaState::new().with_value("...")`.

- **`TextAreaState::with_placeholder()` is now a chainable builder**:
  Same change. Use `TextAreaState::new().with_placeholder("...")`.

- **`SplitPanelState::with_ratio()` is now a chainable builder**: Takes
  `mut self` instead of `(orientation, ratio)`. Migrate from
  `SplitPanelState::with_ratio(orientation, 0.3)` to
  `SplitPanelState::new(orientation).with_ratio(0.3)`.

### Added

- **Reference Application: Log Explorer** (`examples/log_explorer.rs`):
  Multi-pane log analysis tool demonstrating LogViewer, EventStream,
  SplitPanel, CommandPalette, FocusManager, and StatusBar.

- **Reference Application: Dashboard Builder**
  (`examples/dashboard_builder.rs`): Metrics dashboard demonstrating
  MetricsDashboard, AlertPanel, Heatmap, Tabs, Gauge, and Sparkline.

- **Reference Application: Chat Client** (`examples/chat_client.rs`):
  AI chat client demonstrating ConversationView, MessageHandle, TabBar,
  TextArea, CommandPalette, and markdown rendering.

- **Reference Application: File Manager** (`examples/file_manager.rs`):
  File manager demonstrating FileBrowser, CodeBlock, DiffViewer,
  SplitPanel, Breadcrumb, and CommandPalette.

- **Prelude re-exports**: `FileEntry`, `FileSortField`,
  `FileSortDirection`, `SelectionMode` (from FileBrowser) and `Language`
  (from CodeBlock) are now available via the prelude.

- **`MetricsDashboardState::widget_by_label()`**: Look up a metric
  widget by its label string instead of tracking indices.

- **`MetricsDashboardState::widget_by_label_mut()`**: Mutable version
  for updating metrics by label.

- **`LogViewerState::push_entry()` is now public**: General-purpose
  method for adding entries with a specific level and optional timestamp.

### Fixed

- **StatusBar right section truncation**: When left + center + right
  sections exceeded terminal width, the right section was silently
  clipped. Now the center section is truncated first (with ellipsis "…")
  to preserve left and right sections.

- **Virtual terminal command dispatch**: Reference app examples now use
  `vt.dispatch()` instead of `App::update()` for proper
  `Command::message()` cascading on `tick()`.

## [0.10.3] - 2026-03-31

### Fixed

- **Scrollbar off-by-one**: ConversationView now reserves 1 column for
  the scrollbar when content is scrollable, preventing the last character
  of every line from being hidden behind the scrollbar track.

- **Code spans are atomic in markdown wrap**: Inline code spans
  (backtick-wrapped like ``us-east5``) are never broken across lines.
  Previously ``us-east5`` could split as ``us-`` / ``east5``.

## [0.10.2] - 2026-03-31

### Fixed

- **render_markdown wraps text preserving styling**: Paragraphs, list
  items, and blockquotes now word-wrap at the given width inside the
  markdown renderer itself, preserving bold/italic/code styling across
  wrapped lines. Previously wrapping was only in the caller and lost
  styling on overflow lines.

- **Removed plain-text fallback in ConversationView**: Since
  render_markdown now wraps correctly, the fallback that joined spans
  to a String and re-wrapped unstyled is no longer needed.

## [0.10.1] - 2026-03-31

### Fixed

- **ViewContext Phase 2**: All 74 components now read `ctx.focused` /
  `ctx.disabled` for rendering decisions instead of `state.focused` /
  `state.disabled`. ViewContext is no longer dead code — parents can
  control visual focus/disabled at render time.

- **ConversationView word-wrap uses unicode display width**: `wrap_lines()`
  now uses `UnicodeWidthStr::width()` and `UnicodeWidthChar::width()` for
  correct CJK, emoji, and multi-byte character handling.

- **Markdown + indent width accounting**: Markdown rendering passes
  `width - indent_display_width` to `render_markdown()`. Lines that still
  overflow fall back to plain-text wrapping.

- **ViewContext precedence documented**: `ctx` is authoritative for
  rendering; `state` is authoritative for behavior (event handling).

## [0.10.0] - 2026-03-31

### Breaking

- **`Component::view()` now takes `&ViewContext`**: The view signature
  changed from `(state, frame, area, theme)` to
  `(state, frame, area, theme, ctx)`. Pass `&ViewContext::default()` to
  migrate existing code. `ViewContext` carries render-time state
  (focused, disabled) as a parameter, separating it from persistent
  component state.

- **`ConversationRole` no longer implements `Copy`**: The `Custom(String)`
  variant requires heap allocation. Use `.clone()` where `Copy` was
  relied upon. `role()` now returns `&ConversationRole`.

- **`ToastOutput::Expired(u64)` → `Expired(Vec<u64>)`**: Reports all
  expired toast IDs per tick instead of just the first.

- **`MessageBlock::ToolUse` field changes**: `input` is now
  `Option<String>`. Constructor `tool_use(name)` takes only the name;
  use `.with_input()` / `.with_output()` builders for optional fields.

### Added

- **`ViewContext`** struct with `focused` and `disabled` fields, builder
  pattern (`ViewContext::new().focused(true)`). Passed to all component
  `view()` functions as the architectural foundation for focus-as-render-
  parameter (Phase 1).

- **`MessageHandle`** for stable streaming identity. `push_message()`
  returns a handle; `update_by_handle(handle, f)` finds the message by
  ID regardless of intervening pushes or evictions.

- **`ConversationRole::Custom(String)`** for user-defined roles beyond
  User/Assistant/System/Tool.

- **`MessageBlock::tool_use()` builder pattern** with `.with_input()`
  and `.with_output()` for optional tool data.

- **ConversationView word-wrap**: All block types (text, thinking,
  tool_use, error) now word-wrap at terminal width instead of only
  splitting on `\n`.

- **ConversationView markdown rendering**: `with_markdown(true)` enables
  markdown rendering for text blocks (behind `markdown` feature flag),
  reusing the existing `render_markdown()` pipeline.

- **Conditional indent**: ConversationView drops the 2-char indent when
  `show_role_labels` is false, reclaiming 2 columns per line.

- **`examples/README.md`** component catalog: Categorized listing of all
  74 components with descriptions and example links.

### Fixed

- **`total_display_lines` no longer hardcodes width 80**: Uses
  `last_known_width` for scroll content length estimation.

## [0.9.0] - 2026-03-30

### Breaking

- **`#[non_exhaustive]` on all Output enums**: All 56 component Output
  enums are now `#[non_exhaustive]`. Match expressions on Output types
  must include a `_ => {}` wildcard arm. This prevents future variant
  additions from being semver-breaking.

- **`MultiProgressOutput::Selected(usize)`**: New variant emitted when
  Enter is pressed on a focused item. Code exhaustively matching on
  `MultiProgressOutput` will need updating.

### Added

- **`Focusable::view_with_focus()`**: Render with focus temporarily
  overridden without cloning state. Requires `&mut State`; intended for
  testing and non-TEA contexts.

- **`Disableable::view_with_disabled()`**: Same pattern for disabled state.

- **Safe mutation APIs**: `update_item()`, `push_item()`, `remove_item()`
  on SelectableList and SearchableList that maintain filter indices.
  `update_tab()` on TabBar, `update_root()` on Tree,
  `update_last_message()` and `update_message()` on ConversationView.
  All no-op on out-of-bounds indices instead of panicking.

- **Tier 1 `_mut()` accessors**: Direct mutable collection access on
  simple data containers (Timeline, Histogram, Heatmap, HelpPanel,
  AlertPanel, DependencyGraph, SpanTree, FlameGraph, EventStream).

- **`ConversationMessage::set_blocks()`** and **`blocks_mut()`** for
  streaming LLM output — update blocks in place without rebuilding.

- **`MultiProgressState::selected()`**, **`selected_item()`**,
  **`set_selected()`**: Proper selection tracking independent of
  scroll viewport.

- **50+ missing setters**: `set_placeholder()`, `set_max_*()`,
  `set_color()`, `set_show_*()`, `set_orientation()`, `set_title()`
  on 26 components.

### Changed

- **`MultiProgress` selection model**: Up/Down now moves a dedicated
  `selected` index (with viewport tracking) instead of scroll_offset.
  `Selected(usize)` emits the actual selected item index.

- **`update_item()` refilters**: SelectableList and SearchableList
  `update_item()` now re-applies the active filter after mutation,
  maintaining filtered_indices consistency.

- **`set_max_*` eviction adjusts scroll**: All `set_max_events()`,
  `set_max_messages()`, `set_max_entries()`, `set_max_lines()` methods
  now update scroll state after evicting items.

### Removed

- **`ConversationViewState::messages_mut()`**: Replaced by safe
  `update_message()` and `update_last_message()` which preserve
  scroll position and collapsed block state.

### Fixed

- **StatusLog `set_max_entries()`** now evicts oldest entries when
  reducing capacity (was a no-op before).

## [0.8.0] - 2026-03-30

### Added

- **32 new components** bringing the total from 42 to 74:
  - **Data Visualization**: Sparkline, Gauge, Canvas, Histogram, Heatmap,
    BoxPlot, Treemap
  - **Observability**: Timeline, SpanTree, FlameGraph, EventStream,
    AlertPanel, LogCorrelation, DependencyGraph
  - **General Purpose**: Switch, HelpPanel, Divider, Collapsible, Slider,
    Paginator, NumberInput, BigText, Calendar, CommandPalette, ScrollView
  - **Claude Code Suite**: CodeBlock (syntax highlighting), TerminalOutput
    (ANSI colors), MarkdownRenderer, ConversationView, TabBar, UsageDisplay

- **Virtual scrolling infrastructure**: `ScrollState` struct with
  `visible_range()`, `ensure_visible()`, and `render_scrollbar()`.
  Retrofitted into all 9 scrollable components (ScrollableText, LogViewer,
  ChatView, SelectableList, Table, LoadingList, DataGrid, SearchableList,
  Tree) with scrollbar indicators.

- **Enhanced Chart**: Area charts, scatter plots, threshold/reference lines,
  manual Y-axis scaling via `with_y_range()`, shared axes for multi-series.

- **Enhanced LogViewer**: Regex search (behind `regex` feature flag),
  follow mode with auto-scroll, search history with deduplication.

- **Enhanced Table**: Multi-column sort with priority indicators, custom
  sort comparators (`numeric_comparator()`, `date_comparator()`), column
  resizing via keyboard.

- **Enhanced MetricsDashboard**: Configurable warning/critical thresholds
  per widget, units display, visual gauge bars, comparison/delta display.

- **Enhanced DataGrid**: Read-only columns via `Column::with_editable()`,
  column hiding via `HideColumn`/`ShowColumn`/`ToggleColumn` messages,
  navigation skips hidden columns.

- **Enhanced TextArea**: Line number display, text search with match
  navigation (StartSearch, NextMatch, PrevMatch, ClearSearch).

- **Serialization expanded** to most component State types behind the
  `serialization` feature flag.

- **Integration tests** for all new components (30 tests covering full
  event→message→update→view cycle).

- **87 examples** covering most components with progressive complexity.

- **`regex` feature flag** for LogViewer regex search support (included
  in `full`).

- **`test_utils::setup_render()`** now public via the `test-utils` feature
  flag for downstream crate testing.

### Changed

- **Compound components re-exported from crate root**: All 23 compound
  components now accessible via `envision::ComponentName` (previously
  required `envision::component::ComponentName`).

- Component files exceeding 1000 lines split into submodules: tab_bar,
  table, loading_list, chart, integration tests.

- Builder methods added to button, checkbox, data_grid, event_stream
  for consistency.

- Doc test coverage improved from 51.9% to ~60% with 150+ new doc tests.

- Snapshot test coverage expanded to all components.

## [0.7.0] - 2026-03-09

### Added

- **ChatView markdown rendering**: Parse and render markdown in chat messages
  when the `markdown` feature is enabled. Supports headings, bold, italic,
  strikethrough, inline code, fenced code blocks, bullet/numbered lists,
  horizontal rules, and links. Role-specific colors are preserved through
  the rendering pipeline via `StyledContent::render_lines_styled()`.
  - `ChatViewState::with_markdown()` / `set_markdown_enabled()` /
    `markdown_enabled()` for opt-in markdown support
  - `StyledContent::from_blocks()` constructor for pre-built block vectors
  - `StyledContent::render_lines_styled()` for caller-provided base style

- **`Command::spawn()`** for fire-and-forget async tasks that don't produce
  messages. Useful for logging, analytics, or background cleanup.

- **Command inspection methods** for testing: `is_none()`, `is_quit()`,
  `is_batch()`, `is_async()` allow tests to verify command types without
  executing them.

- **`App::init()` default implementation** — `init()` now has a default that
  panics with a descriptive message. Applications using `with_state`
  constructors no longer need to implement `init()`.

- **`EnvisionError::Other(BoxedError)` variant** — catch-all error variant
  for wrapping arbitrary errors that don't fit the structured categories.
  Includes `EnvisionError::other()` convenience constructor.

- **`LineInputState::visual_rows_at_width()`** — calculates the number of
  visual rows a line input would occupy at a given width, useful for
  dynamic layout sizing.

- **Tracing instrumentation widened** — `tracing` spans now cover runtime
  event loops, command handler task spawning, subscription registration,
  and async message processing (previously limited to `dispatch_event`
  and `view`).

- **Standalone examples** for components that previously lacked them:
  `confirm_dialog`, `file_browser`, `pane_layout`, `step_indicator`,
  `styled_text`. Five existing examples converted to interactive terminal
  mode.

- **`FileBrowserState` Debug impl** expanded to include all fields.

- **CompactString rationale** documented in `EnhancedCell` module docs,
  explaining the inline-storage optimization for terminal cell buffers.

### Changed

- **Breaking**: `TerminalHook` type widened from
  `Arc<dyn Fn() -> io::Result<()> + Send + Sync>` to
  `Arc<dyn Fn() -> envision::Result<()> + Send + Sync>`. Lifecycle hooks
  (`on_setup`, `on_teardown`, `on_setup_once`, `on_teardown_once`) now
  accept and return `envision::Result` instead of `io::Result`.
  See `MIGRATION.md` for upgrade guide.

- **Breaking**: `SearchableList` matcher function type (`MatcherFn`) now
  requires `Send + Sync` bounds. Closures passed to `with_matcher()` must
  be thread-safe.

- ChatView component implementation extracted to `component_impl.rs`
  submodule to stay within the 1000-line file limit.

### Fixed

- Clippy `io_other_error` lint compatibility with Rust 1.94.
- Re-export gaps for subscription types and runtime aliases from crate root.

## [0.6.0] - 2026-03-08

### Added

- **New components**:
  - **ScrollableText**: Read-only scrollable text display with keyboard
    navigation, optional title, and CJK-aware wrapping
  - **TitleCard**: Centered title display with subtitle, prefix/suffix text,
    configurable styles, and optional border
  - **LineInput**: Single-line text input with visual wrapping, command
    history, undo/redo, text selection, and `max_length` constraint
  - **StepIndicator**: Navigation component showing progress through
    multi-step workflows
  - **StyledText**: Display component for rich text with inline styling
  - **ConfirmDialog**: Preset confirmation dialog with Yes/No buttons
  - **PaneLayout**: Compound component for resizable split-pane layouts
  - **FileBrowser**: Compound component with pluggable filesystem backend

- **Runtime and lifecycle**:
  - **`EnvisionError` custom error type** with `Io`, `Render`, `Config`,
    `Subscription` variants — all public API methods now return
    `envision::Result<T>` instead of `std::io::Result<T>`
  - **`Command::request_cancel_token()`** for cooperative shutdown — allows
    applications to obtain the runtime's `CancellationToken` for cancelling
    background tasks
  - **`VirtualRuntime<A>` and `TerminalRuntime<A>` type aliases** — hide the
    backend generic parameter from user code
  - **`with_state` constructors** on Runtime — bypass `App::init()` with
    pre-built state for CLI-style configuration
  - **`run_terminal()` returns final state** — access application state after
    the TUI exits
  - **`RuntimeConfig` lifecycle hooks** — `on_setup` / `on_teardown` and
    `on_setup_once` / `on_teardown_once` for terminal setup/cleanup
  - **`UnboundedChannelSubscription`** for non-blocking message forwarding
    from external producers
  - **Subscription polling fix** — subscriptions now spawn forwarding tasks
    instead of storing unpolled streams
  - **Component tracing** — `dispatch_event` emits tracing spans when the
    `tracing` feature is enabled

- **Traits and API improvements**:
  - **`Disableable` trait** implemented on all 34 components with
    `is_disabled()` / `set_disabled()` / `with_disabled()` convenience methods
  - **`selected()` getter** as alias for `selected_index()` on all selection
    components
  - **`set_selected(Option<usize>)` standardized** across all components
  - **ChatView role styles**: `set_role_style()` / `with_role_style()` for
    per-role color customization
  - **ProgressBar ETA and rate display** with `set_eta()` / `set_rate()`
  - **`with_visible()` builder** on Dialog and Tooltip
  - **`Default` implementations** for SelectState, StatusBarState,
    StyledTextState
  - **`PartialEq`** on FormState and FileBrowserState
  - **`# Errors` doc sections** on all `Result`-returning public functions

- **Annotation system**:
  - All components emit semantic annotations via `Annotate` /
    `AnnotateContainer` when rendered inside `with_annotations()`
  - New `WidgetType` variants: Spinner, Toast, Tooltip, Accordion,
    Breadcrumb, LoadingList, KeyHints, MultiProgress, StatusLog, TitleCard,
    LineInput, Dropdown, ScrollableText, Form, SplitPanel, SearchableList,
    RadioGroup

- **Testing infrastructure**:
  - **`test-utils` feature flag** — makes `AppHarness` async methods
    (`advance_time`, `wait_for`, etc.) available to integration tests and
    downstream crates
  - Integration, stress, and async test suites
  - Property-based testing expanded to cover 10 additional components
  - ChatView snapshot tests (7 scenarios)
  - Compound component snapshot tests for all compound components
  - Doc test coverage improvements across ~20 components
  - Cross-references between related types in documentation

- **Re-exports**: All subscription types and functions now re-exported from
  crate root (`envision::ChannelSubscription`, `envision::tick`, etc.)

- **Serialization** for ScrollableText, TitleCard, and LineInput state types
  (behind `serialization` feature flag)

- **Worker module** for background task abstraction

- **Examples**: `scrollable_text`, `title_card`, `line_input`, `chat_app`,
  `production_app`, `step_indicator`, `styled_text`, `confirm_dialog`,
  `pane_layout`, `file_browser`, `tree`, `dropdown`, `text_area`,
  `data_grid`, plus 10 additional component examples

- **Migration guide** (`MIGRATION.md`) covering v0.1.0 through v0.6.0
  upgrade paths

- **Audit tool** (`tools/audit/`) for automated library quality assessment

### Changed

- **Breaking**: All public API methods return `envision::Result<T>` instead
  of `std::io::Result<T>` — see `MIGRATION.md` for upgrade guide
- Runtime module split into submodules for maintainability
- `view()` allocations reduced in Tree and SelectableList
- Files exceeding 1000-line limit split into submodules (TextArea,
  FileBrowser tests)

### Fixed

- Clipboard heap corruption on Windows via process-global singleton
- `on_setup` hook not called in `terminal_with_state_and_config`
- Overflow in ScrollableText and char boundary bug in TextArea
- Production example updated to use `on_setup_once` / `on_teardown_once`

## [0.5.0] - 2026-03-02

### Added

- **Consistent `selected_item()` accessor** on all selection-based components
  (RadioGroup, Tabs, Table, Dropdown, Select, Tree, DataGrid)
- **Consistent `set_selected()` setter** on all selection-based components
  (SelectableList, SearchableList, DataGrid, Tree, MetricsDashboard)
- **Disabled state** on all Focusable components with `is_disabled()`/`set_disabled()`
- **`with_disabled()` builder** on all components supporting disabled state
- **`with_placeholder()` builder** on Dropdown and Select
- **`serialization` feature flag**: serde/serde_json are now optional
  dependencies behind a default feature, allowing users to opt out
- **Feature flags** for component groups: `input-components`, `data-components`,
  `display-components`, `navigation-components`, `overlay-components`,
  `compound-components`
- **Overlay/Modal system**: Runtime-owned `OverlayStack` for layered UI
  - `Overlay` trait for custom overlay implementations
  - `OverlayAction` for overlay lifecycle management
  - Priority-based rendering with dismiss support
- **`handle_event` and `dispatch_event`** on `Component` trait
  - All 18 interactive components support event-to-message mapping
  - `dispatch_event` combines `handle_event` + `update` in one call
  - Instance methods on all State types eliminate turbofish syntax
- **`handle_event_with_state`** for state-aware event handling patterns
- **`cell_at()` convenience method** on Runtime, TestHarness, and AppHarness
- **`Command::future()` alias** for ergonomic async command creation
- **`snapshot()` method** on AppHarness for capturing test snapshots
- **Insta snapshot testing** for all component `view()` functions
- **Integration tests** for multi-component workflows
- **Edge case tests** for large datasets, emoji, and Unicode
- **Component showcase example** demonstrating 12 components with
  `dispatch_event` and instance methods

### Changed

- **Selection API standardized**: `selected()` removed in favor of `selected_item()`,
  `set_selected_index()` renamed to `set_selected()`, `selected_row_index()`
  renamed to `selected_index()`
- **Cursor API standardized**: `set_cursor()` renamed to `set_cursor_position()`
  on InputField and TextArea
- Consistent `{Component}Message` / `{Component}Output` naming across all
  components
- Unified navigation variant naming (Up/Down/Left/Right) across components
- `selected_index()` returns `Option<usize>` consistently across all components
- Display-only components use `()` for Output instead of empty enums
- `SearchableListState` matcher changed from `Box<dyn Fn>` to `Arc<dyn Fn>`
  for proper `Clone` support
- **Unified Runtime**: Sync `Runtime` replaced with single async `Runtime`
  - `Runtime::new_terminal()` for interactive use
  - `Runtime::virtual_terminal()` for programmatic control
  - `AsyncTestHarness` renamed to `AppHarness`
- **`App::Message` requires `Send + 'static`** for async compatibility
- **`State: Clone` no longer required** on `App` and `Component` traits
- **Slimmed prelude**: exports only essential framework types
- Extracted shared runtime and command handler logic
- All tests extracted from source files into separate `tests.rs` modules
- Removed `step()` and all deprecated methods, consolidated on `tick()`
- `Runtime::terminal()` renamed to `new_terminal()`
- `Runtime::inner_terminal()` renamed to `terminal()`
- `SimulatedEvent` renamed to `Event`

### Removed

- Broken `Command::clone()` implementation (Command contains `Box<dyn FnOnce>`)
- `step()` method and all deprecated API methods

### Fixed

- `Router::init()` no longer panics on empty route configuration
- Theme style methods return correct fg/bg colors
- Race conditions in tick/interval cancellation tests on Windows
- `SearchableListState::clone()` now correctly preserves the matcher function
  (previously silently set it to `None`)

## [0.4.1] - 2026-01-15

### Fixed

- Pin `insta` to 0.3.10 for MSRV 1.81 compatibility

## [0.4.0] - 2026-01-10

### Added

- **Theme system**: Consistent styling across all components
  - `Theme` struct with style helpers (`normal_style`, `focused_style`,
    `disabled_style`, `focused_border_style`)
  - Built-in themes: Default and Nord
  - `themed_app` example demonstrating theme usage
  - All component `view()` functions accept `&Theme`
- **KeyHints Component**: Keyboard shortcut display bar
  - Configurable key-action pairs
  - Horizontal layout with separator
- **StatusLog Component**: Scrolling message log
  - Severity levels with timestamps
  - Auto-scroll to latest entry
  - Configurable max entries
- **MultiProgress Component**: Concurrent progress tracking
  - Multiple named progress bars
  - Individual item states (running, complete, error)
  - Keyboard navigation between items
- **Router Component**: Multi-screen navigation
  - Named routes with push/pop/replace operations
  - Route history tracking
  - Back navigation support
- **LoadingList Component**: Lists with per-item loading states
  - Items can be Loading, Ready, or Error
  - Visual indicators for each state
  - Keyboard navigation
- **Virtual terminal API**: `Runtime::virtual_terminal()` for programmatic use
  - Event injection via `vt.send()`
  - Display inspection via `vt.display()`
  - Tick-based frame advance
- **Event type rename**: `SimulatedEvent` renamed to `Event`
- **StatusBar enhancements**: Dynamic content with timers, counters, heartbeat

### Changed

- All component `view()` signatures now accept `&Theme` parameter
- Updated README with full component documentation
- Module documentation updated for new Runtime API
- Updated examples to use virtual terminal API

### Fixed

- `tick_cancellation` test race condition on Windows
- `interval_immediate_cancellation` test race condition on Windows

## [0.3.0] - 2026-01-02

### Added

- **FocusManager**: Keyboard focus coordination between multiple components
  - Tracks focused component by index
  - Supports focus cycling (next/previous)
  - Wrap-around navigation

- **Button Component**: Clickable button with keyboard activation
  - Press/release states
  - Focusable with visual feedback
  - Customizable label

- **Checkbox Component**: Toggleable checkbox with keyboard support
  - Checked/unchecked states
  - Toggle on Space/Enter
  - Customizable label

- **RadioGroup Component**: Single-selection radio button group
  - Keyboard navigation between options
  - Selection change events
  - Horizontal or vertical layout support

- **ProgressBar Component**: Visual progress indicator
  - Configurable min/max/current values
  - Percentage display option
  - Customizable styling

- **Spinner Component**: Animated loading indicator
  - Multiple built-in styles (Dots, Line, Braille, Blocks, Bounce)
  - Tick-based animation
  - Optional label

- **TextArea Component**: Multi-line text editing
  - Full cursor navigation (arrows, Home, End, Ctrl+arrows)
  - Line-based editing with word wrap
  - Insert/delete operations
  - Scrolling for large content

- **Tabs Component**: Horizontal tab navigation
  - Keyboard navigation (Left/Right)
  - Tab selection events
  - Customizable tab labels

- **Table Component**: Generic data table with sorting
  - Column definitions with headers
  - Row selection with keyboard navigation
  - Sortable columns (ascending/descending)
  - Customizable column widths

- **Dialog Component**: Modal dialog overlay
  - Configurable title, message, and buttons
  - Button focus navigation
  - Preset dialogs: alert, confirm
  - Implements Toggleable trait

- **Toast Component**: Non-modal notification system
  - Severity levels: Info, Success, Warning, Error
  - Auto-dismiss with configurable duration
  - Stacking with max visible limit
  - Tick-based expiration

- **Menu Component**: Keyboard-navigable menu
  - Hierarchical menu items with separators
  - Keyboard shortcuts display
  - Disabled item support
  - Selection events

- **Select Component**: Dropdown selection widget
  - Single selection from options list
  - Keyboard navigation when open
  - Implements Toggleable trait

- **Dropdown Component**: Enhanced searchable Select
  - Type-to-filter functionality
  - Filterable option list
  - Clear selection support
  - Combines InputField with Select behavior

- **StatusBar Component**: Application status bar
  - Multiple sections (left, center, right alignment)
  - Configurable styles per item
  - Mode indicators and status display

- **Tree Component**: Hierarchical tree view
  - Expand/collapse nodes
  - Keyboard navigation (Up, Down, Left, Right)
  - Selection tracking
  - Arbitrary depth support

- **Accordion Component**: Collapsible panel container
  - Multiple panels with headers
  - Single or multiple expansion modes
  - Keyboard navigation between panels
  - Expand/collapse all support

- **Breadcrumb Component**: Navigation breadcrumb trail
  - Clickable path segments
  - Keyboard navigation
  - Customizable separator
  - Home segment support

- **Tooltip Component**: Contextual information overlay
  - Configurable position (Above, Below, Left, Right)
  - Automatic fallback positioning
  - Optional auto-hide with duration
  - Customizable colors (fg, bg, border)

## [0.2.0] - 2025-12-31

### Added

- **Component System**: TEA-style composable UI components
  - `Component` trait for defining reusable components
  - `Focusable` trait for keyboard focus management
  - `Toggleable` trait for visibility toggling

- **SelectableList Component**: Generic scrollable list with selection
  - Keyboard navigation (Up, Down, Home, End, PageUp, PageDown)
  - Selection tracking with change events
  - Customizable rendering via `Display` trait
  - Focusable with visual feedback

- **InputField Component**: Text input with cursor navigation
  - Character insertion and deletion
  - Cursor movement (Left, Right, Home, End, word jumps)
  - Word-level deletion (Ctrl+Backspace, Ctrl+Delete)
  - Placeholder text support
  - Unicode support
  - Submit handling

## [0.1.0] - 2025-12-29

### Added

- **CaptureBackend**: Headless ratatui backend for testing
  - Captures rendered frames as inspectable data
  - Frame history with configurable depth
  - Multiple output formats: Plain, ANSI, JSON, JsonPretty
  - Cell-level access for detailed assertions

- **TEA Architecture**: The Elm Architecture pattern for TUI apps
  - `App` trait for defining application logic
  - `Runtime` for synchronous applications
  - `AsyncRuntime` for async applications with tokio
  - `Command` type for side effects and async operations

- **Subscriptions**: Reactive event streams
  - `TickSubscription` for periodic updates
  - `TimerSubscription` for delayed events
  - `TerminalEventSubscription` for keyboard/mouse input
  - `IntervalImmediateSubscription` for immediate-then-periodic ticks
  - Combinators: `filter`, `throttle`, `debounce`, `take`

- **Widget Annotations**: Semantic metadata for widgets
  - `Annotate` wrapper widget
  - `AnnotationRegistry` for tracking widget regions
  - Built-in widget types: Button, Input, List, Table, etc.
  - Custom widget type support
  - Interactive and focus state tracking

- **Test Harness**: Fluent testing interface
  - `TestHarness` for synchronous testing
  - `AsyncTestHarness` for async testing
  - `Assertion` enum with composable assertions
  - `Snapshot` for snapshot testing
  - Input simulation: keyboard, mouse, clipboard

- **DualBackend**: Adapter for simultaneous rendering
  - Renders to both a real terminal and CaptureBackend
  - Useful for visual debugging while testing

[Unreleased]: https://github.com/ryanoneill/envision/compare/v0.12.0...HEAD
[0.12.0]: https://github.com/ryanoneill/envision/compare/v0.11.0...v0.12.0
[0.11.0]: https://github.com/ryanoneill/envision/compare/v0.10.3...v0.11.0
[0.10.3]: https://github.com/ryanoneill/envision/compare/v0.10.2...v0.10.3
[0.10.2]: https://github.com/ryanoneill/envision/compare/v0.10.1...v0.10.2
[0.10.1]: https://github.com/ryanoneill/envision/compare/v0.10.0...v0.10.1
[0.10.0]: https://github.com/ryanoneill/envision/compare/v0.9.0...v0.10.0
[0.9.0]: https://github.com/ryanoneill/envision/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/ryanoneill/envision/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/ryanoneill/envision/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/ryanoneill/envision/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/ryanoneill/envision/compare/v0.4.1...v0.5.0
[0.4.1]: https://github.com/ryanoneill/envision/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/ryanoneill/envision/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/ryanoneill/envision/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/ryanoneill/envision/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/ryanoneill/envision/releases/tag/v0.1.0
