# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/ryanoneill/envision/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/ryanoneill/envision/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/ryanoneill/envision/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/ryanoneill/envision/releases/tag/v0.1.0
