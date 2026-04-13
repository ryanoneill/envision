# Envision

[![CI](https://github.com/ryanoneill/envision/actions/workflows/ci.yml/badge.svg)](https://github.com/ryanoneill/envision/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/ryanoneill/envision/graph/badge.svg)](https://codecov.io/gh/ryanoneill/envision)
[![docs.rs](https://docs.rs/envision/badge.svg)](https://docs.rs/envision)
[![Crates.io](https://img.shields.io/crates/v/envision.svg)](https://crates.io/crates/envision)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A [ratatui](https://github.com/ratatui/ratatui) framework for collaborative TUI development with headless testing support.

## Features

- **Component Library** - 73 ready-to-use UI components following TEA pattern
- **Headless Testing** - Render your TUI without a terminal using `CaptureBackend`
- **TEA Architecture** - The Elm Architecture pattern with `App`, `Runtime`, and `Command`
- **Async Runtime** - Full async support with subscriptions, timers, and async commands
- **Widget Annotations** - Attach semantic metadata to widgets for testing and accessibility
- **Test Harness** - Fluent assertions and snapshot testing for TUI applications
- **Input Simulation** - Programmatically simulate keyboard and mouse events
- **Multiple Output Formats** - Plain text, ANSI-colored, and JSON output

## Quick Start

Add envision to your project:

```bash
cargo add envision
```

### Basic Capture Backend

```rust
use envision::backend::CaptureBackend;
use ratatui::Terminal;
use ratatui::widgets::Paragraph;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a headless terminal
    let backend = CaptureBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;

    // Render something
    terminal.draw(|frame| {
        frame.render_widget(Paragraph::new("Hello, Envision!"), frame.area());
    })?;

    // Capture the output
    println!("{}", terminal.backend());
    Ok(())
}
```

### TEA Architecture

```rust
use envision::prelude::*;

struct MyApp;

#[derive(Default, Clone)]
struct State {
    count: i32,
}

#[derive(Clone)]
enum Msg {
    Increment,
    Decrement,
}

impl App for MyApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        (State::default(), Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Increment => state.count += 1,
            Msg::Decrement => state.count -= 1,
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let text = format!("Count: {}", state.count);
        frame.render_widget(Paragraph::new(text), frame.area());
    }
}
```

### Testing with Runtime

```rust
use envision::prelude::*;
use ratatui::widgets::Paragraph;

// Given the MyApp defined above:
#[test]
fn test_my_app() {
    let mut runtime = Runtime::<MyApp>::virtual_terminal(80, 24).unwrap();

    runtime.dispatch(Msg::Increment);
    runtime.dispatch(Msg::Increment);
    runtime.render().unwrap();

    assert!(runtime.contains_text("Count: 2"));
}
```

### Test Harness for Custom Rendering

```rust
use envision::harness::TestHarness;
use ratatui::widgets::Paragraph;

let mut harness = TestHarness::new(80, 24);

harness.render(|frame| {
    frame.render_widget(Paragraph::new("Hello!"), frame.area());
});

harness.assert_contains("Hello!");
```

## Examples

Run the examples to see envision in action:

```bash
# Basic capture backend usage
cargo run --example capture_backend

# TEA architecture with sync runtime
cargo run --example counter_app

# Async runtime with subscriptions
cargo run --example async_counter

# Test harness and assertions
cargo run --example test_harness

# Widget annotations
cargo run --example annotations

# Theme switching with components
cargo run --example themed_app

# Multi-component showcase with focus management
cargo run --example component_showcase
```

## Components

Envision provides a comprehensive library of 73 reusable UI components, all following the TEA (The Elm Architecture) pattern with `Component` and `Toggleable` traits.

### Input Components

| Component | Description |
|-----------|-------------|
| `Button` | Clickable button with keyboard activation |
| `Checkbox` | Toggleable checkbox with label |
| `Dropdown` | Searchable/filterable select with type-to-filter |
| `InputField` | Single-line text input with cursor navigation |
| `LineInput` | Single-line input with visual wrapping, history, undo/redo |
| `RadioGroup` | Single-selection radio button group |
| `Select` | Dropdown selection widget |
| `TextArea` | Multi-line text editor with scrolling |

### Display Components

| Component | Description |
|-----------|-------------|
| `KeyHints` | Keyboard shortcut display bar |
| `MultiProgress` | Concurrent progress indicators for batch operations |
| `ProgressBar` | Visual progress indicator with percentage |
| `ScrollableText` | Scrollable read-only text display with CJK support |
| `Spinner` | Animated loading indicator (multiple styles) |
| `StatusBar` | Application status bar with sections, timers, counters |
| `StatusLog` | Scrolling status messages with severity levels |
| `StepIndicator` | Multi-step workflow progress display |
| `StyledText` | Rich text display with inline styling |
| `TitleCard` | Centered title display with subtitle and prefix/suffix |
| `Toast` | Non-modal notification system with auto-dismiss |
| `Tooltip` | Contextual information overlay |

### Navigation Components

| Component | Description |
|-----------|-------------|
| `Accordion` | Collapsible panel container |
| `Breadcrumb` | Navigation breadcrumb trail |
| `Menu` | Keyboard-navigable menu with shortcuts |
| `Router` | Multi-screen navigation with history |
| `StepIndicator` | Pipeline/workflow visualization with per-step styles |
| `Tabs` | Horizontal tab navigation |
| `TabBar` | Tab bar with closeable tabs and overflow |

### Data Components

| Component | Description |
|-----------|-------------|
| `LoadingList` | List with per-item loading and error states |
| `SelectableList` | Scrollable list with keyboard navigation |
| `Table` | Data table with sorting and selection |
| `Tree` | Hierarchical tree view with expand/collapse |

### Display Components

| Component | Description |
|-----------|-------------|
| `BigText` | Large block-character text rendering |
| `Calendar` | Month calendar with event markers |
| `Canvas` | General-purpose drawing surface with shape primitives |
| `CodeBlock` | Syntax-highlighted code display |
| `Collapsible` | Expandable/collapsible content panel |
| `Divider` | Horizontal or vertical separator |
| `Gauge` | Ratio and measurement display with thresholds |
| `HelpPanel` | Keyboard shortcut reference panel |
| `KeyHints` | Contextual keyboard shortcut bar |
| `MultiProgress` | Multiple concurrent progress trackers |
| `Paginator` | Page navigation indicators |
| `ProgressBar` | Progress display with ETA and rate |
| `ScrollView` | Scrollable container for arbitrary content |
| `ScrollableText` | Scrollable multi-line text display |
| `Sparkline` | Inline trend visualization |
| `Spinner` | Animated loading indicator (multiple styles) |
| `StatusBar` | Application status bar with sections |
| `StatusLog` | Timestamped status message log |
| `StyledText` | Rich text display with styled content |
| `TerminalOutput` | ANSI-capable terminal output display |
| `TitleCard` | Styled title with optional subtitle |
| `Toast` | Timed notification messages |
| `UsageDisplay` | Resource usage metrics display |

### Overlay Components

| Component | Description |
|-----------|-------------|
| `ConfirmDialog` | Preset confirmation dialog with Yes/No buttons |
| `Dialog` | Modal dialog overlay with custom buttons |
| `Tooltip` | Positioned tooltip with auto-dismiss |

### Compound Components

| Component | Description |
|-----------|-------------|
| `AlertPanel` | Alert metrics dashboard with sparklines |
| `BoxPlot` | Statistical box-and-whisker plots |
| `Chart` | Line, bar, area, and scatter charts with annotations |
| `CommandPalette` | Fuzzy-searchable command palette overlay |
| `ConversationView` | AI conversation display with role colors and markdown |
| `DataGrid` | Editable data table with cell navigation |
| `DependencyGraph` | Node-edge dependency visualization |
| `DiffViewer` | Side-by-side and unified diff display |
| `EventStream` | Real-time event log with levels and timestamps |
| `FileBrowser` | File system browser with pluggable backend |
| `FlameGraph` | Hierarchical flame graph visualization |
| `Form` | Multi-field form with validation |
| `Heatmap` | 2D color-mapped data visualization |
| `Histogram` | Distribution visualization with adaptive binning |
| `LogCorrelation` | Multi-stream synchronized log viewer |
| `LogViewer` | Filterable log display with search |
| `MarkdownRenderer` | Markdown text rendering (headings, bold, code, lists) |
| `MetricsDashboard` | Dashboard with charts, counters, and gauges |
| `PaneLayout` | Resizable split-pane layouts |
| `SearchableList` | Filterable list with search input |
| `SpanTree` | Hierarchical span/trace tree |
| `SplitPanel` | Resizable dual-panel layout |
| `Timeline` | Time-based event and span visualization |
| `Treemap` | Proportional area-based data visualization |

### Utility

| Component | Description |
|-----------|-------------|
| `FocusManager` | Keyboard focus coordination |
| `AppShell` | Consistent header/content/footer layout splits |

### Component Example

```rust
use envision::component::{Button, ButtonMessage, ButtonOutput, ButtonState, Component};

// Initialize state
let mut state = ButtonState::new("Submit");

// Handle messages
let output = Button::update(&mut state, ButtonMessage::Press);
assert_eq!(output, Some(ButtonOutput::Pressed));
```

## Architecture

Envision follows The Elm Architecture (TEA) pattern:

```
┌─────────────────────────────────────────────────────────┐
│                    Application                           │
│                                                          │
│   ┌─────────┐     ┌────────┐     ┌──────────────────┐   │
│   │  State  │────▶│  View  │────▶│  Terminal/Frame  │   │
│   └─────────┘     └────────┘     └──────────────────┘   │
│        ▲                                                 │
│        │                                                 │
│   ┌─────────┐     ┌────────────────────┐                │
│   │ Update  │◀────│  Message/Events    │                │
│   └─────────┘     └────────────────────┘                │
│        │                    ▲                            │
│        ▼                    │                            │
│   ┌─────────┐     ┌────────────────────┐                │
│   │ Command │────▶│  Effect Handler    │                │
│   └─────────┘     └────────────────────┘                │
└─────────────────────────────────────────────────────────┘
```

## Modules

| Module | Description |
|--------|-------------|
| `component` | 73 reusable UI components with `Component`, `Toggleable` traits |
| `backend` | `CaptureBackend` for headless rendering |
| `app` | TEA architecture: `App`, `Runtime`, `Command`, subscriptions |
| `harness` | `TestHarness` and `AppHarness` for testing |
| `annotation` | Widget annotations with `Annotate` and `AnnotationRegistry` |
| `input` | Input simulation with `EventQueue` |
| `overlay` | Modal overlay system with `Overlay` trait and `OverlayStack` |
| `theme` | Theming with `Theme` for consistent styling across components |
| `adapter` | `DualBackend` for simultaneous real + capture rendering |

## Minimum Supported Rust Version

The minimum supported Rust version is **1.85** (edition 2024).

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
