# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.0] - 2026-03-07

### Added

- **Utility module** (`envision::util`): `truncate_to_width` and
  `wrapped_line_count` with full CJK and emoji support
- **ScrollableText component**: Read-only scrollable text display with
  keyboard navigation, optional title, and CJK-aware wrapping
- **TitleCard component**: Centered title display with subtitle,
  prefix/suffix text, configurable styles, and optional border
- **LineInput component**: Single-line text input with visual wrapping,
  command history, undo/redo, text selection, and `max_length` constraint
- **ChatView role styles**: `set_role_style()` / `with_role_style()` for
  per-role color customization in ChatView
- **Annotation wiring**: All components emit semantic annotations via
  `Annotate` / `AnnotateContainer` when rendered inside `with_annotations()`
- **WidgetType variants**: Added Spinner, Toast, Tooltip, Accordion,
  Breadcrumb, LoadingList, KeyHints, MultiProgress, StatusLog, TitleCard,
  LineInput, Dropdown, ScrollableText, Form, SplitPanel, SearchableList,
  RadioGroup
- **Serialization** for ScrollableText, TitleCard, and LineInput state types
  (behind `serialization` feature flag)
- **`with_visible()` builder** on Dialog and Tooltip
- **ChatView snapshot tests** (7 scenarios: empty, single message,
  multi-role, focused, disabled, custom role styles, scrolled)
- **Compound component snapshot tests** for all 7 compound components
  (Chart, ChatView, DataGrid, Form, LogViewer, MetricsDashboard, SplitPanel)
- **Integration tests** for zero-size rendering of ScrollableText, TitleCard,
  and LineInput
- **Doc test coverage** improvements for Dropdown, TextArea, Select, Table,
  and Column
- **Examples**: `scrollable_text`, `title_card`, `line_input`, `chat_app`
  (ChatView + LineInput composition)
- **Migration guide** (`MIGRATION.md`) covering v0.1.0 through v0.5.0
  upgrade paths

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

[Unreleased]: https://github.com/ryanoneill/envision/compare/v0.6.0...HEAD
[0.6.0]: https://github.com/ryanoneill/envision/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/ryanoneill/envision/compare/v0.4.1...v0.5.0
[0.4.1]: https://github.com/ryanoneill/envision/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/ryanoneill/envision/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/ryanoneill/envision/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/ryanoneill/envision/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/ryanoneill/envision/releases/tag/v0.1.0
