# Contributing to Envision

Thank you for your interest in contributing to Envision! This guide will help
you get started.

## Getting Started

### Prerequisites

- Rust 1.81 or later (check `rust-version` in `Cargo.toml` for the current MSRV)
- Git with GPG signing configured

### Setup

```bash
git clone https://github.com/ryanoneill/envision.git
cd envision
cargo build
cargo test
```

## Development Workflow

### Branch Strategy

All changes must be made via Pull Requests. Do not commit directly to `main`.

```bash
git checkout -b feature/your-feature-name
# ... make changes ...
git push -u origin feature/your-feature-name
# Open a PR on GitHub
```

### Code Quality

Before submitting a PR, ensure:

```bash
cargo test                          # All tests pass
cargo clippy -- -D warnings         # No clippy warnings
cargo fmt --check                   # Code is formatted
cargo doc --no-deps --all-features  # Docs build cleanly
cargo test --no-default-features    # Compiles without optional features
```

### Commit Guidelines

- All commits must be signed (`git commit -S`)
- Write clear, descriptive commit messages
- Make focused commits (one logical change per commit)

## Architecture

Envision follows The Elm Architecture (TEA):

- **State**: Data model for a component
- **Message**: Events that trigger state changes
- **Update**: Pure function that produces new state from old state + message
- **View**: Pure function that renders state to the UI

### Component Structure

Each component lives in `src/component/{name}/`:

- `mod.rs` - Component implementation (State, Message, Output, Component impl)
- `tests.rs` - Unit tests and snapshot tests

### Key Traits

- `Component` - Core trait: `init()`, `update()`, `view()`, `handle_event()`, `dispatch_event()`
- `Focusable` - Focus management: `is_focused()`, `set_focused()`
- `Toggleable` - Visibility: `is_visible()`, `set_visible()`

### Patterns to Follow

Every interactive component should have:

1. **State struct** with `disabled: bool` field
2. **Accessors**: `is_disabled()`, `set_disabled()`, `with_disabled()`
3. **Instance methods**: `handle_event()`, `dispatch_event()`, `update()` on the State type
4. **`selected_item()`** accessor if the component has selection
5. **Event guards**: `handle_event` checks `focused && !disabled`
6. **Update guards**: `update` checks `!disabled`
7. **View disabled style**: Uses `theme.disabled_style()` when disabled

## Testing

### Test-Driven Development

We use TDD. Write tests before or alongside implementation.

### Test Utilities

```rust
use crate::test_utils::setup_render;

// Create a test rendering context
let (mut terminal, area) = setup_render(40, 5);
```

### Snapshot Testing

Use `insta` for view tests:

```rust
#[test]
fn test_view_focused() {
    let (mut terminal, area) = setup_render(20, 3);
    let mut state = MyState::new();
    state.set_focused(true);
    let theme = Theme::default();

    terminal.draw(|frame| {
        MyComponent::view(&state, frame, area, &theme);
    }).unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}
```

### Event Testing

```rust
use crate::input::{Event, KeyCode};

#[test]
fn test_handle_event_focused() {
    let mut state = MyState::new();
    state.set_focused(true);
    let event = Event::key(KeyCode::Enter);
    let msg = MyComponent::handle_event(&state, &event);
    assert_eq!(msg, Some(MyMessage::Confirm));
}
```

## File Size Limit

No source file should exceed 1000 lines. If a file is approaching this limit,
refactor by extracting into submodules.

## Feature Flags

The `serialization` feature (enabled by default) gates serde support. When
adding serde derives, use conditional compilation:

```rust
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serialization", derive(serde::Serialize, serde::Deserialize))]
pub struct MyType { ... }
```

## Questions?

Open an issue on GitHub if you have questions or need guidance.
