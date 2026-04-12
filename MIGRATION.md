# Migration Guide

## v0.13.x to v0.14.0

### Envision-owned input types

0.14.0 replaces crossterm event re-exports with envision-owned types. The
key changes:

| Before (v0.13.x) | After (v0.14.0) |
|---|---|
| `KeyCode` | `Key` |
| `KeyModifiers` | `Modifiers` |
| `key.code` (crossterm) | `key.code` (envision) |
| `KeyCode::Char('a')` | `Key::Char('a')` |
| `KeyCode::BackTab` | `Key::Tab` with `key.modifiers.shift()` |
| `key.modifiers.contains(KeyModifiers::SHIFT)` | `key.modifiers.shift()` |
| `key.modifiers.contains(KeyModifiers::CONTROL)` | `key.modifiers.ctrl()` |
| `Event::key(KeyCode::Enter)` | `Event::key(Key::Enter)` |
| `use envision::input::KeyCode` | `use envision::input::Key` |
| `use envision::input::KeyModifiers` | `use envision::input::Modifiers` |

#### Imports

```rust
// Before
use envision::input::{Event, KeyCode, KeyModifiers};

// After
use envision::input::{Event, Key, Modifiers};
```

#### Keybindings (handle_event match arms)

```rust
// Before
match key.code {
    KeyCode::Enter => Some(Msg::Submit),
    KeyCode::Char('q') => Some(Msg::Quit),
    KeyCode::BackTab => Some(Msg::FocusPrev),
    _ => None,
}

// After
match key.code {
    Key::Enter => Some(Msg::Submit),
    Key::Char('q') => Some(Msg::Quit),
    Key::Tab if key.modifiers.shift() => Some(Msg::FocusPrev),
    _ => None,
}
```

#### Modifier checking

```rust
// Before
let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
let shift = key.modifiers.contains(KeyModifiers::SHIFT);

// After
let ctrl = key.modifiers.ctrl();
let shift = key.modifiers.shift();
```

#### Text input (Insert(c) patterns)

ASCII letter keys are normalized to lowercase in `key.code`. For text
input, use `key.raw_char` which preserves the original character:

```rust
// Before
KeyCode::Char(c) if !ctrl => Some(Msg::Insert(c)),

// After
Key::Char(_) if !ctrl => key.raw_char.map(Msg::Insert),
```

#### Shift+G (scroll-to-bottom) pattern

With normalization, `Key::Char('G')` becomes `Key::Char('g')` with
`modifiers.shift()`. Place the shift-guarded arm before the plain arm:

```rust
// Before
KeyCode::Home | KeyCode::Char('g') => Some(Msg::Top),
KeyCode::End | KeyCode::Char('G') if shift || key.code == KeyCode::End => Some(Msg::Bottom),

// After
Key::Char('g') if key.modifiers.shift() => Some(Msg::Bottom),
Key::Home | Key::Char('g') => Some(Msg::Top),
Key::End => Some(Msg::Bottom),
```

#### TerminalEventSubscription

The handler now receives `Event` (envision's) instead of
`crossterm::event::Event`:

```rust
// Before
use crossterm::event::{Event, KeyCode, KeyEvent};
let sub = terminal_events(|event| {
    if let Event::Key(KeyEvent { code: KeyCode::Char('q'), .. }) = event {
        Some(Msg::Quit)
    } else { None }
});

// After
use envision::input::{Event, Key};
let sub = terminal_events(|event| {
    if let Some(key) = event.as_key() {
        if key.code == Key::Char('q') { return Some(Msg::Quit); }
    }
    None
});
```

### RenderContext refactor

0.14.0 introduces a large, cascading breakage around component rendering.
`Component::view` now takes a single `&mut RenderContext<'_, '_>` argument
instead of five separate arguments, and `ViewContext` has been renamed to
`EventContext` and is now used only for `handle_event`. The upside is that
component signatures are stable: adding a new render-time field (e.g.
scaling factor, accessibility hint) is a non-breaking change because callers
never name those fields explicitly.

### 1. Component `view` implementations

```rust
// Before (v0.13.x)
use envision::component::{Component, ViewContext};
use envision::theme::Theme;
use ratatui::prelude::*;

impl Component for MyButton {
    // ...
    fn view(
        state: &Self::State,
        frame: &mut Frame,
        area: Rect,
        theme: &Theme,
        ctx: &ViewContext,
    ) {
        let style = if ctx.focused {
            theme.focused_style()
        } else {
            theme.normal_style()
        };
        frame.render_widget(Paragraph::new("Hello").style(style), area);
    }
}
```

```rust
// After (v0.14.0)
use envision::component::{Component, RenderContext};
use ratatui::prelude::*;

impl Component for MyButton {
    // ...
    fn view(state: &Self::State, ctx: &mut RenderContext<'_, '_>) {
        let style = if ctx.focused {
            ctx.theme.focused_style()
        } else {
            ctx.theme.normal_style()
        };
        ctx.frame.render_widget(Paragraph::new("Hello").style(style), ctx.area);
        // Or equivalently:
        // ctx.render_widget(Paragraph::new("Hello").style(style));
    }
}
```

Mechanical steps:
- Replace the parameter list with `ctx: &mut RenderContext<'_, '_>`.
- Replace `theme` with `ctx.theme`.
- Replace `frame` with `ctx.frame` (or use `ctx.render_widget(...)` when
  the widget fills the entire `area`).
- Replace `area` with `ctx.area`.
- `ctx.focused` and `ctx.disabled` continue to work unchanged.

### 2. Component `handle_event` implementations

```rust
// Before
fn handle_event(
    state: &Self::State,
    event: &Event,
    ctx: &ViewContext,
) -> Option<Self::Message> { /* ... */ }
```

```rust
// After
fn handle_event(
    state: &Self::State,
    event: &Event,
    ctx: &EventContext,
) -> Option<Self::Message> { /* ... */ }
```

This is a pure type rename. Field access (`ctx.focused`, `ctx.disabled`) is
unchanged. The same applies to `Component::dispatch_event`.

### 3. Sub-area rendering: `RenderContext::with_area`

When a parent component lays out sub-areas and delegates to child
components:

```rust
// Before
fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, ctx: &ViewContext) {
    let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(area);
    Header::view(&state.header, frame, chunks[0], theme, ctx);
    Body::view(&state.body, frame, chunks[1], theme, ctx);
}
```

```rust
// After
fn view(state: &Self::State, ctx: &mut RenderContext<'_, '_>) {
    let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(ctx.area);
    Header::view(&state.header, &mut ctx.with_area(chunks[0]));
    Body::view(&state.body, &mut ctx.with_area(chunks[1]));
}
```

`ctx.with_area(new_area)` returns a `RenderContext` that borrows `frame`
for a shorter lifetime, reborrows the theme, and preserves `focused` and
`disabled`. When the child context goes out of scope, the parent `ctx` is
usable again.

If a child needs different focus/disabled state, you can chain the
builder methods directly on the returned context:

```rust
// Chaining pattern (preferred for simple cases)
Header::view(
    &state.header,
    &mut ctx.with_area(chunks[0]).focused(state.current_focus == FocusTarget::Header),
);
```

Or mutate the fields explicitly for more complex logic:

```rust
let mut child_ctx = ctx.with_area(chunks[0]);
child_ctx.focused = state.current_focus == FocusTarget::Header;
child_ctx.disabled = !state.header_enabled;
Header::view(&state.header, &mut child_ctx);
```

### 4. Test code calling `Component::view` directly

```rust
// Before
terminal.draw(|frame| {
    Button::view(&state, frame, frame.area(), &theme, &ViewContext::default());
}).unwrap();
```

```rust
// After
terminal.draw(|frame| {
    Button::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
}).unwrap();
```

For focused or disabled rendering:

```rust
// Before
Button::view(&state, frame, frame.area(), &theme, &ViewContext::new().focused(true));

// After
Button::view(
    &state,
    &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
);
```

### 5. Test code calling `Component::handle_event` / `dispatch_event`

Pure type rename:

```rust
// Before
Button::handle_event(&state, &event, &ViewContext::new().focused(true));
Button::dispatch_event(&mut state, &event, &ViewContext::default());
```

```rust
// After
Button::handle_event(&state, &event, &EventContext::new().focused(true));
Button::dispatch_event(&mut state, &event, &EventContext::default());
```

### 6. `App::view` is unchanged

`App::view` still has signature `(state, frame)` — the `App` trait is not
affected by the `Component::view` change. Only `Component::view` (and its
helpers like `traced_view`) moved to `RenderContext`.

### 7. `ChartGrid::render` and other inherent methods

A small number of inherent methods (as opposed to trait methods) also
migrated to `&mut RenderContext<'_, '_>`:

- `ChartGrid::render(&self, frame, area, theme, ctx)` →
  `ChartGrid::render(&self, ctx: &mut RenderContext<'_, '_>)`
- `ConversationView::view_from(source, state, frame, area, theme, ctx)` →
  `ConversationView::view_from(source, state, ctx: &mut RenderContext<'_, '_>)`
- `Router::view(state, frame, area, theme, ctx)` →
  `Router::view(state, ctx: &mut RenderContext<'_, '_>)`

Call sites need the same `&mut RenderContext::new(...)` treatment as
`Component::view` call sites.

### 8. Worker module: `ProgressSender<P>` and `Command::subscribe`

`ProgressSender` is now generic over your progress type:

```rust
// Before (0.13.x)
progress_sender.send_percentage(0.5).await?;
progress_sender.send_status(0.3, "downloading").await?;

// After (0.14.0)
// Use any type — an enum, struct, or the built-in WorkerProgress
progress_sender.send(WorkerProgress::new(0.5, None)).await?;
progress_sender.send(MyProgress::ChapterCount(12)).await?;

// Non-blocking for high-frequency updates
progress_sender.try_send(MyProgress::Tick(0.5)).ok();
```

Workers can now be spawned on-demand from `update()` using
`Command::subscribe` to register the progress subscription:

```rust
fn update(state: &mut State, msg: Msg) -> Command<Msg> {
    match msg {
        Msg::StartWork => {
            let (cmd, subscription, handle) = WorkerBuilder::new("task")
                .spawn(
                    |sender, cancel| async move { /* ... */ },
                    Msg::Progress,
                    |result| Msg::Done(result),
                );
            state.handle = Some(handle);
            Command::combine(vec![cmd, Command::subscribe(subscription)])
        }
        // ...
    }
}
```

## v0.6.0 to v0.7.0

### Breaking Changes

#### 1. Lifecycle Hook Error Type Widened

`TerminalHook` now returns `envision::Result<()>` instead of `io::Result<()>`.
This affects the `on_setup`, `on_teardown`, `on_setup_once`, and
`on_teardown_once` methods on `RuntimeConfig`.

```rust
// Before (v0.6.0)
use std::io;

let config = RuntimeConfig::default()
    .on_setup_once(|| -> io::Result<()> {
        // setup logic
        Ok(())
    });

// After (v0.7.0)
let config = RuntimeConfig::default()
    .on_setup_once(|| -> envision::Result<()> {
        // setup logic
        Ok(())
    });
```

Since `EnvisionError` implements `From<io::Error>`, existing hooks that only
produce `io::Error` continue to compile with the `?` operator. The change
primarily affects explicit return type annotations and error matching.

If your hooks use explicit `io::Result<()>` return types, update them to
`envision::Result<()>`. If they use `Ok(())` with implicit return types, no
change is needed.

#### 2. SearchableList Matcher Requires `Send + Sync`

The `MatcherFn` type used by `SearchableList::with_matcher()` now requires
`Send + Sync` bounds. This ensures matcher closures are safe to use across
async boundaries.

```rust
// Before (v0.6.0) -- non-Send closures worked
let local_data = Rc::new(vec!["data"]);
state.set_matcher(move |query, item| {
    // closure capturing Rc (not Send)
    Some(0)
});

// After (v0.7.0) -- closures must be Send + Sync
let shared_data = Arc::new(vec!["data"]);
state.set_matcher(move |query, item| {
    // closure capturing Arc (Send + Sync)
    Some(0)
});
```

Most closures that capture only owned types (`String`, `Vec`, `Arc`, etc.)
already satisfy `Send + Sync`. Only closures capturing `Rc`, `Cell`,
`RefCell`, or other non-thread-safe types need adjustment.

### New Features (Non-Breaking)

- **ChatView markdown rendering**: Enable with `with_markdown(true)` when the
  `markdown` feature is active. Supports headings, bold, italic, code blocks,
  lists, and more.

- **`Command::spawn()`**: Fire-and-forget async tasks that don't produce
  messages.

- **Command inspection**: `is_none()`, `is_quit()`, `is_batch()`, `is_async()`
  for testing command types without executing them.

- **`App::init()` default**: Applications using `with_state` constructors no
  longer need to implement `init()`.

- **`EnvisionError::Other`**: Catch-all error variant for arbitrary errors.

- **`LineInputState::visual_rows_at_width()`**: Calculate visual row count for
  dynamic layout sizing.

---

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
envision = "0.6"
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

Every component supports `handle_event` (read-only event-to-message
mapping) and `dispatch_event` (combines handle_event + update in one call).
These are static trait methods that take a `ViewContext` parameter for
focused and disabled state.

```rust
let ctx = ViewContext::new().focused(true);
let msg = SelectableList::<String>::handle_event(&state, &event, &ctx);
if let Some(msg) = msg {
    let output = SelectableList::<String>::update(&mut state, msg);
}
```

**dispatch_event (preferred, combines handle_event + update):**

```rust
let ctx = ViewContext::new().focused(true);
if let Some(output) = SelectableList::<String>::dispatch_event(&mut state, &event, &ctx) {
    match output {
        SelectableListOutput::Selected(item) => { /* ... */ }
        SelectableListOutput::SelectionChanged(idx) => { /* ... */ }
        _ => {}
    }
}
```

#### Typical Event Routing Pattern

```rust
fn handle_event_with_state(state: &AppState, event: &Event) -> Option<AppMsg> {
    // Route event to the focused component
    SelectableList::<String>::handle_event(
        &state.list, event, &ViewContext::new().focused(true)
    ).map(AppMsg::List)
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
envision = "0.6"
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

#### ViewContext for Focus and Disabled State

Focus and disabled state are passed via `ViewContext` to `handle_event`,
`dispatch_event`, and `view`:

```rust
// Focused component receives events
let ctx = ViewContext::new().focused(true);
let msg = Button::handle_event(&state, &event, &ctx);

// Disabled component ignores events
let ctx = ViewContext::new().focused(true).disabled(true);
let msg = Button::handle_event(&state, &event, &ctx);
assert!(msg.is_none());
```

#### Static Trait Methods for Update

```rust
// Works but verbose
Button::update(&mut state, ButtonMessage::Press);

// Preferred: use instance methods
state.update(ButtonMessage::Press);
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

- [ ] Update `Cargo.toml` to `envision = "0.6"`
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
- [ ] Consider adopting `dispatch_event` with `ViewContext` for cleaner event routing
- [ ] Add feature flags to `Cargo.toml` if you want to reduce compile times
