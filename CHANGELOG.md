# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/ryanoneill/envision/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/ryanoneill/envision/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/ryanoneill/envision/releases/tag/v0.1.0
