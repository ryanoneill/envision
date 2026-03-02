# Migration Guide

## Migrating from 0.4.x to 0.5.0

Version 0.5.0 unifies the runtime, streamlines the component API, and
introduces an overlay system. This guide covers every breaking change
with before/after code examples.

### Unified Runtime

The sync `Runtime` and `AsyncRuntime` have been merged into a single
async `Runtime`. All apps now use the same type.

**Before (0.4.x):**

```rust
// Synchronous usage
let runtime = Runtime::new(MyApp)?;
runtime.run()?;

// Async usage
let runtime = AsyncRuntime::new(MyApp)?;
runtime.run().await?;
```

**After (0.5.0):**

```rust
// Interactive terminal (async)
Runtime::<MyApp>::new_terminal()?.run_terminal().await?;

// Headless / testing (sync-style with tick)
let mut vt = Runtime::<MyApp, _>::virtual_terminal(80, 24)?;
vt.tick()?;
```

### Runtime Method Renames

| 0.4.x | 0.5.0 | Notes |
|-------|-------|-------|
| `Runtime::terminal()` | `Runtime::new_terminal()` | Creates interactive terminal |
| `Runtime::inner_terminal()` | `Runtime::terminal()` | Access underlying terminal |
| `AsyncTestHarness` | `AppHarness` | Test harness for full app testing |
| `step()` | `tick()` | Advance one frame |

**Before (0.4.x):**

```rust
let harness = AsyncTestHarness::new(MyApp, 80, 24)?;
harness.step()?;
let terminal = runtime.inner_terminal();
```

**After (0.5.0):**

```rust
let harness = AppHarness::new(MyApp, 80, 24)?;
harness.tick()?;
let terminal = runtime.terminal();
```

### Message Requires Send + 'static

`App::Message` (and by extension `Component::Message`) must now be
`Send + 'static` to support async command execution across tokio tasks.

**Before (0.4.x):**

```rust
// This worked even with non-Send types
enum Msg {
    Data(Rc<String>),  // Rc is not Send
}
```

**After (0.5.0):**

```rust
// Messages must be Send + 'static
enum Msg {
    Data(Arc<String>),  // Arc is Send
}
```

If your message type contains non-Send fields, wrap them in `Arc` or
restructure to store data in state rather than passing it through
messages.

### State: Clone No Longer Required

The `Clone` bound has been removed from `App::State` and
`Component::State`. You can still derive `Clone` if needed, but the
framework no longer requires it.

**Before (0.4.x):**

```rust
// Clone was required
#[derive(Clone)]
struct AppState {
    // Must clone entire state each tick
    large_data: Vec<Record>,
}
```

**After (0.5.0):**

```rust
// Clone is optional
struct AppState {
    large_data: Vec<Record>,
}
```

### SimulatedEvent Renamed to Event

The `SimulatedEvent` type has been renamed to `Event` for consistency.

**Before (0.4.x):**

```rust
use envision::input::SimulatedEvent;

let event = SimulatedEvent::key(KeyCode::Enter);
```

**After (0.5.0):**

```rust
use envision::input::Event;

let event = Event::key(KeyCode::Enter);
```

### New Event Handling Methods

Components now support `handle_event` and `dispatch_event` for
streamlined event routing. The `dispatch_event` method combines event
mapping and state update in a single call.

**Before (0.4.x):**

```rust
// Manual two-step event handling
if let Some(msg) = my_event_handler(&state, &event) {
    MyComponent::update(&mut state, msg);
}
```

**After (0.5.0):**

```rust
// One-step dispatch
MyComponent::dispatch_event(&mut state, &event);

// Or using instance methods (eliminates turbofish for generics)
state.dispatch_event(&event);
```

### Slimmed Prelude

The prelude no longer re-exports `ratatui::prelude::*`. Layout and
style types are now re-exported through envision's own modules.

**Before (0.4.x):**

```rust
use envision::prelude::*;
// ratatui types were available directly
```

**After (0.5.0):**

```rust
use envision::prelude::*;
// Layout types available: Rect, Constraint, Direction, Layout, etc.
// Style types available: Color, Style, Modifier, Stylize
// If you need additional ratatui types, import them explicitly:
// use ratatui::widgets::Paragraph;
```

### Removed APIs

The following APIs have been removed without replacement:

- **`step()`** — Use `tick()` instead.
- **`Command::clone()`** — `Command` contains `Box<dyn FnOnce>` and
  `Pin<Box<dyn Future>>` which cannot be cloned. If you need to send
  the same command multiple times, create it in a function and call
  the function each time.
- **All deprecated methods** from 0.4.x have been removed.

### Overlay System (New)

Version 0.5.0 introduces a runtime-owned `OverlayStack` for layered
UI elements like modals and dialogs. This is additive — no migration
needed unless you were managing dialog visibility manually.

```rust
use envision::prelude::*;

// Overlays are managed by the runtime
// Implement the Overlay trait for custom overlays
// Use OverlayAction to push/pop overlays
```

### Quick Migration Checklist

- [ ] Replace `AsyncRuntime` / sync `Runtime` with unified `Runtime`
- [ ] Rename `terminal()` → `new_terminal()`
- [ ] Rename `inner_terminal()` → `terminal()`
- [ ] Rename `AsyncTestHarness` → `AppHarness`
- [ ] Replace `step()` with `tick()`
- [ ] Replace `SimulatedEvent` with `Event`
- [ ] Ensure `Message` types are `Send + 'static` (use `Arc` not `Rc`)
- [ ] Remove `Clone` derive from `State` if it was only there for the framework
- [ ] Remove any `Command::clone()` calls
- [ ] Update imports if relying on ratatui re-exports from the prelude
- [ ] Consider adopting `dispatch_event` for simpler event routing
