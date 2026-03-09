# Migration Guide

## v0.5.0 to v0.6.0

### Breaking Changes

#### 1. `io::Result` Replaced with `envision::Result`

All public API methods that previously returned `std::io::Result<T>` now return
`envision::Result<T>` (an alias for `Result<T, EnvisionError>`). This provides
structured error variants (`Io`, `Render`, `Config`, `Subscription`) instead of
a flat `io::Error`.

Since `EnvisionError` implements `From<std::io::Error>`, existing `?` usage
continues to work. The main change is in return types and error matching.

**Affected types:**
- `Runtime` — all constructors and `tick()`, `run()`, `run_ticks()`, `render()`, `run_terminal()`, `run_terminal_blocking()`
- `AppHarness` — all constructors and `tick()`, `run_ticks()`, `render()`
- `TestHarness` — `render()`
- `Snapshot` — `write_to_file()`, `load_from_file()`
- `SnapshotTest` — `assert()`
- `DualBackend` — `with_auto_capture()`
- `DualBackendBuilder` — `build()`

```rust
// Before (v0.5.0)
fn main() -> std::io::Result<()> {
    let mut rt = Runtime::<MyApp>::new_terminal()?;
    // ...
    Ok(())
}

// After (v0.6.0)
fn main() -> envision::Result<()> {
    let mut rt = Runtime::<MyApp>::new_terminal()?;
    // ...
    Ok(())
}
```

**Error matching:**

```rust
// Before (v0.5.0)
match result {
    Err(e) if e.kind() == io::ErrorKind::NotFound => { /* ... */ }
    _ => {}
}

// After (v0.6.0)
match result {
    Err(EnvisionError::Io(e)) if e.kind() == io::ErrorKind::NotFound => { /* ... */ }
    Err(EnvisionError::Render { component, detail }) => { /* ... */ }
    Err(EnvisionError::Config { field, reason }) => { /* ... */ }
    _ => {}
}
```

### Migration Steps

1. Replace `std::io::Result<()>` with `envision::Result<()>` in your `main()` and any functions that call envision APIs
2. If you were matching on `io::Error` variants, wrap the match with `EnvisionError::Io(_)`
3. `EnvisionError` is re-exported from `envision::EnvisionError` and `envision::prelude::*` — no additional imports needed

---

## v0.4.x to v0.5.0

This guide covers all breaking changes, new required patterns, and migration
steps for upgrading from envision v0.4.x to v0.5.0. Each breaking change
includes concrete before/after code examples so you can follow the migration
mechanically.

---

### Breaking Changes

#### 1. Unified Runtime (AsyncRuntime Removed)

The separate sync `Runtime` and `AsyncRuntime` have been merged into a single
async `Runtime`. All runtime usage is now async.

```rust
// Before (v0.4.x) -- sync runtime
let mut runtime = Runtime::<MyApp>::new(terminal)?;
runtime.run()?;

// Before (v0.4.x) -- async runtime
let mut runtime = AsyncRuntime::<MyApp>::new(terminal)?;
runtime.run().await?;

// After (v0.5.0) -- single async runtime
let mut runtime = Runtime::<MyApp>::new_terminal()?;
runtime.run().await?;
```

For testing, `virtual_terminal()` continues to work the same way:

```rust
// Both v0.4.x and v0.5.0
let mut runtime = Runtime::<MyApp>::virtual_terminal(80, 24)?;
```

#### 2. Runtime Method Renames

| v0.4.x | v0.5.0 |
|--------|--------|
| `Runtime::terminal()` | `Runtime::new_terminal()` |
| `Runtime::inner_terminal()` | `Runtime::terminal()` |
| `AsyncTestHarness` | `AppHarness` |
| `step()` | `tick()` |

```rust
// Before (v0.4.x)
let runtime = Runtime::<MyApp>::terminal()?;
let terminal = runtime.inner_terminal();
harness.step()?;

// After (v0.5.0)
let runtime = Runtime::<MyApp>::new_terminal()?;
let terminal = runtime.terminal();
harness.tick().await?;
```

#### 3. `App::Message` Requires `Send + 'static`

All `App::Message` types must now be `Send + 'static` for async compatibility.

```rust
// Before (v0.4.x) -- this worked even without Send
#[derive(Clone)]
enum Msg {
    Data(Rc<String>),        // Rc is not Send
    Callback(Box<dyn Fn()>), // dyn Fn() is not Send
}

// After (v0.5.0) -- must be Send + 'static
#[derive(Clone)]
enum Msg {
    Data(Arc<String>),               // Arc is Send
    Callback(Box<dyn Fn() + Send>),  // explicitly Send
}
```

#### 4. `State: Clone` No Longer Required

The `Clone` bound has been removed from `App::State` and `Component::State`.
This is not breaking for most users, but if you had generic code relying on
the implicit `Clone` bound from the trait, you will need to add it explicitly.

```rust
// Before (v0.4.x) -- Clone was enforced by the trait
impl App for MyApp {
    type State = State;  // State: Clone was required
}

// After (v0.5.0) -- Clone is optional
impl App for MyApp {
    type State = State;  // Clone no longer required
}
```

You can now use non-cloneable types in your state (file handles, channels, etc.).

#### 5. `SimulatedEvent` Renamed to `Event`

This was deprecated in v0.4.0 and is now fully removed.

```rust
// Before (v0.4.x)
use envision::input::SimulatedEvent;
let event = SimulatedEvent::key(KeyCode::Enter);

// After (v0.5.0)
use envision::input::Event;
let event = Event::key(KeyCode::Enter);
```

#### 6. `step()` Method and All Deprecated APIs Removed

```rust
// Before (v0.4.x)
runtime.step()?;

// After (v0.5.0)
runtime.tick().await?;
```

#### 7. `Command` Is No Longer `Clone`

`Command` contains `Box<dyn FnOnce>` and `Pin<Box<dyn Future>>` which cannot
be cloned. The broken `Clone` implementation has been removed.

```rust
// Before (v0.4.x) -- compiled but was unsound
let cmd = Command::none();
let cmd2 = cmd.clone();

// After (v0.5.0) -- will not compile
let cmd = Command::none();
// let cmd2 = cmd.clone();  // Error: Command does not implement Clone

// Fix: create commands via a function
fn make_cmd() -> Command<Msg> {
    Command::none()
}
```

#### 8. Selection API Standardized

The selection accessor methods have been standardized across all components.

**`selected()` removed -- use `selected_item()`**

```rust
// Before (v0.4.x)
let item = state.selected();

// After (v0.5.0)
let item = state.selected_item();
```

Affected components: RadioGroup, Tabs, Table, Dropdown, Select, Tree, DataGrid.

**`set_selected_index()` renamed to `set_selected()`**

```rust
// Before (v0.4.x)
state.set_selected_index(2);

// After (v0.5.0)
state.set_selected(2);
```

Affected components: SelectableList, SearchableList, DataGrid, Tree,
MetricsDashboard.

**`selected_row_index()` renamed to `selected_index()`**

```rust
// Before (v0.4.x)
let idx: usize = state.selected_row_index();

// After (v0.5.0)
let idx: Option<usize> = state.selected_index();
```

`selected_index()` now consistently returns `Option<usize>` across all
components. Code that assumed a bare `usize` return will need to handle
the `Option`.

```rust
// Before (v0.4.x)
let index: usize = state.selected_index();

// After (v0.5.0)
if let Some(index) = state.selected_index() {
    // handle selection
}
```

#### 9. Cursor API Standardized

**`set_cursor()` renamed to `set_cursor_position()`**

```rust
// Before (v0.4.x)
state.set_cursor(5);

// After (v0.5.0)
state.set_cursor_position(5);
```

Affected components: InputField, TextArea.

#### 10. Message and Output Naming Standardized

All component message/output types now follow `{Component}Message` /
`{Component}Output` naming.

```rust
// Before (v0.4.x) -- varied naming
use envision::component::SelectableListMsg;

// After (v0.5.0) -- consistent naming
use envision::component::SelectableListMessage;
```

Navigation message variants are standardized to `Up`/`Down`/`Left`/`Right`:

```rust
// Before (v0.4.x)
SelectableListMsg::MoveUp
TabsMsg::SelectNext

// After (v0.5.0)
SelectableListMessage::Up
TabsMessage::Right
```

#### 11. Display-Only Components Use `()` for Output

Components that are display-only (ProgressBar, Spinner, etc.) now use `()`
as their `Output` type instead of custom empty enums. This only affects you
if you were pattern-matching on their output type.

#### 12. SearchableListState Matcher Uses `Arc`

```rust
// Before (v0.4.x)
let matcher: Box<dyn Fn(&T, &str) -> bool> = Box::new(|item, query| { ... });

// After (v0.5.0)
let matcher: Arc<dyn Fn(&T, &str) -> bool> = Arc::new(|item, query| { ... });
```

This enables proper `Clone` support for `SearchableListState`.

#### 13. Slimmed Prelude

The prelude no longer re-exports `ratatui::prelude::*` directly. Layout and
style types are re-exported through envision's own modules. If you need
additional ratatui types, import them explicitly.

The prelude still includes: `App`, `Command`, `Runtime`, `RuntimeConfig`,
`Subscription`, `Event`, `KeyCode`, `Theme`, all component types,
`CaptureBackend`, `TestHarness`, `AppHarness`, layout types (`Rect`,
`Constraint`, `Layout`, etc.), and style types (`Color`, `Style`, etc.).

```rust
// If you need ratatui types not in the prelude:
use ratatui::widgets::Paragraph;
```

---

### Migration Steps

Follow these steps in order to migrate your project.

#### Step 1: Update Cargo.toml

```toml
[dependencies]
envision = "0.5"
```

All features are enabled by default. To customize, see the Feature Flags
section below.

#### Step 2: Fix Runtime Usage

Replace sync/async runtime constructors with the unified runtime:

```rust
// Replace AsyncRuntime with Runtime
// Replace Runtime::terminal() with Runtime::new_terminal()
// Replace runtime.inner_terminal() with runtime.terminal()
```

#### Step 3: Fix Event Type References

```rust
// Search and replace:
// SimulatedEvent -> Event
```

#### Step 4: Fix `step()` to `tick()`

```rust
// Replace all step() calls with tick().await
```

#### Step 5: Fix Selection and Cursor API Calls

| Old Method | New Method |
|---|---|
| `.selected()` (on component state) | `.selected_item()` |
| `.set_selected_index(n)` | `.set_selected(n)` |
| `.selected_row_index()` | `.selected_index()` |
| `.set_cursor(n)` | `.set_cursor_position(n)` |

#### Step 6: Fix Message Type Names

Replace old message/output type names with the standardized versions. The
pattern is always `{Component}Message` and `{Component}Output`.

#### Step 7: Ensure Message Types Are `Send + 'static`

Audit your `App::Message` enum for non-`Send` types (`Rc`, `Cell`,
`RefCell`, raw pointers). Replace with `Arc`, `Mutex`, or restructure
to store data in state instead of messages.

#### Step 8: Remove `Command::clone()` Calls

If you were cloning commands, create them via a helper function instead.

---

### New Required Patterns

#### handle_event and dispatch_event

Every component now supports `handle_event` (read-only event-to-message
mapping) and `dispatch_event` (combines handle_event + update in one call).
Both are available as static trait methods and as instance methods on all
state types.

**Static trait method (works, but verbose with generics):**

```rust
let msg = SelectableList::<String>::handle_event(&state, &event);
if let Some(msg) = msg {
    let output = SelectableList::<String>::update(&mut state, msg);
}
```

**Instance method (preferred, eliminates turbofish):**

```rust
if let Some(output) = state.dispatch_event(&event) {
    match output {
        SelectableListOutput::Selected(item) => { /* ... */ }
        SelectableListOutput::SelectionChanged(idx) => { /* ... */ }
        _ => {}
    }
}
```

The instance methods are available on every state type: `ButtonState`,
`InputFieldState`, `SelectableListState<T>`, `TabsState<T>`,
`TableState<T>`, etc.

#### Typical Event Routing Pattern

```rust
fn update(state: &mut AppState, msg: AppMsg) -> Command<AppMsg> {
    match msg {
        AppMsg::TerminalEvent(event) => {
            // Route event to the focused component
            if let Some(output) = state.list.dispatch_event(&event) {
                match output {
                    SelectableListOutput::Selected(item) => {
                        // handle item selection
                    }
                    SelectableListOutput::SelectionChanged(idx) => {
                        // handle navigation
                    }
                    _ => {}
                }
            }
        }
        // ...
    }
    Command::none()
}
```

---

### Feature Flags

Components are now organized behind feature flags for smaller binaries
and faster compile times. All are enabled by default via the `full` feature.

| Feature | Components |
|---------|-----------|
| `input-components` | Button, Checkbox, Dropdown, InputField, LineInput, RadioGroup, Select, TextArea |
| `data-components` | LoadingList, SelectableList, Table, Tree |
| `display-components` | KeyHints, MultiProgress, ProgressBar, ScrollableText, Spinner, StatusBar, StatusLog, TitleCard, Toast |
| `navigation-components` | Accordion, Breadcrumb, Menu, Router, Tabs |
| `overlay-components` | Dialog, Tooltip |
| `compound-components` | Chart, ChatView, DataGrid, Form, LogViewer, MetricsDashboard, SearchableList, SplitPanel (implies input + data + display) |
| `serialization` (default) | serde support for all state types |
| `clipboard` | System clipboard integration for InputField and TextArea |
| `tracing` | Tracing instrumentation for component rendering and dispatch |

**Include everything (default):**

```toml
[dependencies]
envision = "0.5"
```

**Opt out of serialization:**

```toml
[dependencies]
envision = { version = "0.5", default-features = false, features = ["full"] }
```

**Only specific component groups:**

```toml
[dependencies]
envision = { version = "0.5", default-features = false, features = [
    "input-components",
    "data-components",
] }
```

---

### Deprecated Patterns

The following patterns still work in v0.5.0 but should be updated.

#### Static Trait Methods for Event Handling

Static trait methods work but are verbose, especially with generic components:

```rust
// Works but verbose (especially with generics requiring turbofish)
let msg = SelectableList::<String>::handle_event(&state, &event);
if let Some(msg) = msg {
    SelectableList::<String>::update(&mut state, msg);
}

// Preferred: use instance methods
state.dispatch_event(&event);
```

#### Static Trait Methods for Focus

```rust
// Works but verbose
Button::set_focused(&mut state, true);
assert!(Button::is_focused(&state));

// Preferred: use instance methods
state.set_focused(true);
assert!(state.is_focused());
```

#### Static Trait Methods for Update

```rust
// Works but verbose
Button::update(&mut state, ButtonMessage::Press);

// Preferred: use instance methods
state.update(ButtonMessage::Press);
```

---

### New Capabilities

#### Disabled State on All Focusable Components

Every focusable component now supports `is_disabled()`, `set_disabled()`,
and `with_disabled()`. Disabled components ignore all input events and
render with `theme.disabled_style()`.

```rust
// Builder pattern
let state = ButtonState::new("Submit").with_disabled(true);

// Mutation
state.set_disabled(true);
assert!(state.is_disabled());

// Disabled components ignore events
let output = state.dispatch_event(&Event::key(KeyCode::Enter));
assert!(output.is_none());
```

#### Overlay/Modal System

The new `OverlayStack` provides runtime-managed layered UI for modals,
dialogs, and other overlays with priority-based rendering and dismiss support.

```rust
use envision::overlay::{Overlay, OverlayAction, OverlayStack};

struct HelpOverlay;

impl Overlay for HelpOverlay {
    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        // render overlay content
    }

    fn handle_event(&mut self, event: &Event) -> OverlayAction {
        if let Some(key) = event.as_key() {
            if key.code == KeyCode::Esc {
                return OverlayAction::Dismiss;
            }
        }
        OverlayAction::Consumed
    }
}
```

#### `Command::future()` Alias

A more ergonomic way to create async commands:

```rust
Command::future(async { fetch_data().await }, Msg::DataLoaded)
```

#### `cell_at()` Convenience Method

Available on `Runtime`, `TestHarness`, and `AppHarness` for easier cell
inspection in tests:

```rust
let cell = runtime.cell_at(5, 2);
assert_eq!(cell.symbol(), "X");
```

#### `AppHarness::snapshot()`

Capture test snapshots directly from the harness:

```rust
let harness = AppHarness::<MyApp>::new(80, 24)?;
harness.dispatch(Msg::LoadData).await;
let snapshot = harness.snapshot();
insta::assert_snapshot!(snapshot);
```

#### Placeholder Support on Dropdown and Select

```rust
let state = DropdownState::new(options).with_placeholder("Choose an option...");
let state = SelectState::new(options).with_placeholder("Select...");
```

---

### Quick Migration Checklist

- [ ] Update `Cargo.toml` to `envision = "0.5"`
- [ ] Replace `AsyncRuntime` / sync `Runtime` with unified `Runtime`
- [ ] Rename `Runtime::terminal()` to `Runtime::new_terminal()`
- [ ] Rename `runtime.inner_terminal()` to `runtime.terminal()`
- [ ] Rename `AsyncTestHarness` to `AppHarness`
- [ ] Replace `step()` with `tick()`
- [ ] Replace `SimulatedEvent` with `Event`
- [ ] Ensure `App::Message` types are `Send + 'static` (use `Arc` not `Rc`)
- [ ] Remove any `Command::clone()` calls
- [ ] Rename `selected()` to `selected_item()`
- [ ] Rename `set_selected_index()` to `set_selected()`
- [ ] Rename `selected_row_index()` to `selected_index()`
- [ ] Handle `selected_index()` returning `Option<usize>` instead of `usize`
- [ ] Rename `set_cursor()` to `set_cursor_position()`
- [ ] Update component message/output type names to `{Component}Message` / `{Component}Output`
- [ ] Update ratatui imports if relying on types no longer in the prelude
- [ ] Consider adopting `dispatch_event` and instance methods for cleaner code
- [ ] Add feature flags to `Cargo.toml` if you want to reduce compile times
