# Envision

[![CI](https://github.com/ryanoneill/envision/actions/workflows/ci.yml/badge.svg)](https://github.com/ryanoneill/envision/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/ryanoneill/envision/graph/badge.svg)](https://codecov.io/gh/ryanoneill/envision)
[![docs.rs](https://docs.rs/envision/badge.svg)](https://docs.rs/envision)
[![Crates.io](https://img.shields.io/crates/v/envision.svg)](https://crates.io/crates/envision)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A [ratatui](https://github.com/ratatui/ratatui) framework for collaborative TUI development with headless testing support.

## Features

- **Component Library** - 20+ ready-to-use UI components following TEA pattern
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

// Create a headless terminal
let backend = CaptureBackend::new(80, 24);
let mut terminal = Terminal::new(backend)?;

// Render something
terminal.draw(|frame| {
    frame.render_widget(Paragraph::new("Hello, Envision!"), frame.area());
})?;

// Capture the output
println!("{}", terminal.backend());
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

#[test]
fn test_my_app() {
    // Use Runtime::headless for testing App implementations
    let mut runtime = Runtime::<MyApp, _>::headless(80, 24).unwrap();

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
```

## Components

Envision provides a comprehensive library of reusable UI components, all following the TEA (The Elm Architecture) pattern with `Component`, `Focusable`, and `Toggleable` traits.

### Input Components

| Component | Description |
|-----------|-------------|
| `InputField` | Single-line text input with cursor navigation |
| `TextArea` | Multi-line text editor with scrolling |
| `Checkbox` | Toggleable checkbox with label |
| `RadioGroup` | Single-selection radio button group |
| `Select` | Dropdown selection widget |
| `Dropdown` | Searchable/filterable select with type-to-filter |

### Display Components

| Component | Description |
|-----------|-------------|
| `ProgressBar` | Visual progress indicator with percentage |
| `Spinner` | Animated loading indicator (multiple styles) |
| `Toast` | Non-modal notification system with auto-dismiss |
| `Tooltip` | Contextual information overlay |
| `StatusBar` | Application status bar with sections |

### Navigation Components

| Component | Description |
|-----------|-------------|
| `Tabs` | Horizontal tab navigation |
| `Menu` | Keyboard-navigable menu with shortcuts |
| `Breadcrumb` | Navigation breadcrumb trail |
| `Tree` | Hierarchical tree view with expand/collapse |

### Layout Components

| Component | Description |
|-----------|-------------|
| `Dialog` | Modal dialog overlay with buttons |
| `Accordion` | Collapsible panel container |
| `Table` | Data table with sorting and selection |
| `SelectableList` | Scrollable list with keyboard navigation |

### Utility

| Component | Description |
|-----------|-------------|
| `Button` | Clickable button with keyboard activation |
| `FocusManager` | Keyboard focus coordination |

### Component Example

```rust
use envision::component::{Button, ButtonMessage, ButtonState, Component, Focusable};

// Initialize state
let mut state = ButtonState::new("Submit");

// Handle messages
Button::update(&mut state, ButtonMessage::Press);

// Focus management
Button::focus(&mut state);
assert!(Button::is_focused(&state));
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
| `component` | 20+ reusable UI components with `Component`, `Focusable`, `Toggleable` traits |
| `backend` | `CaptureBackend` for headless rendering |
| `app` | TEA architecture: `App`, `Runtime`, `Command`, subscriptions |
| `harness` | `TestHarness` and `AsyncTestHarness` for testing |
| `annotation` | Widget annotations with `Annotate` and `AnnotationRegistry` |
| `input` | Input simulation with `EventQueue` |
| `adapter` | `DualBackend` for simultaneous real + capture rendering |

## Minimum Supported Rust Version

The minimum supported Rust version is **1.81**.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
