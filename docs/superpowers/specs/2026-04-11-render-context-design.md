# RenderContext refactor — design

**Status:** approved
**Date:** 2026-04-11
**Target version:** 0.14.0 (breaking)
**Source:** library audit finding #1 (Complexity Hiding B+ → A-)

## Problem

The `Component::view()` trait method takes 5 parameters:

```rust
fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, ctx: &ViewContext);
```

Every Component impl signature carries this verbosity. Adding a new
render-time concern (e.g., a new render-time configuration value)
requires touching every component's signature — a breaking change.

The audit's #1 trust-eroding finding for the library: users
implementing custom components must understand and propagate ratatui's
`Frame` and `Rect` types, plus the framework's `Theme` and
`ViewContext`. A wrapper that bundles these into one value would:

1. Shorten every `view()` signature to two parameters
2. Make adding new render-time fields non-breaking
3. Provide a single place to put convenience methods like `render_widget`
4. Match how mature TUI/UI frameworks (e.g., GPUI, Iced) wrap render context

## Goal

Introduce `RenderContext` bundling the rendering machinery, simplify
`Component::view()` to two parameters, and rename `ViewContext` →
`EventContext` since it is no longer the context passed to `view()`.

Non-goals:
- Changing component rendering behavior. This is a pure refactor.
- Adding new render-time fields. The struct gives us room to grow,
  but the initial fields exactly mirror today's parameters.
- Touching `Component::handle_event` semantics. Only the parameter
  type name changes (`ViewContext` → `EventContext`).

## Design

### New types in `src/component/mod.rs`

```rust
/// Context passed to [`Component::handle_event`].
///
/// Carries focus and disabled state from the parent so the component
/// can decide whether and how to handle events.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EventContext {
    /// Whether this component currently has keyboard focus.
    pub focused: bool,
    /// Whether this component is currently disabled.
    pub disabled: bool,
}

impl EventContext {
    pub fn new() -> Self { Self::default() }
    pub fn focused(mut self, focused: bool) -> Self { self.focused = focused; self }
    pub fn disabled(mut self, disabled: bool) -> Self { self.disabled = disabled; self }
}

/// Context passed to [`Component::view`].
///
/// Bundles the frame, area, theme, and focus/disabled state into a
/// single value so component view signatures stay short and adding
/// new render-time fields is non-breaking.
pub struct RenderContext<'a> {
    pub frame: &'a mut Frame<'a>,
    pub area: Rect,
    pub theme: &'a Theme,
    pub focused: bool,
    pub disabled: bool,
}

impl<'a> RenderContext<'a> {
    /// Constructs a new RenderContext with focused/disabled both `false`.
    pub fn new(frame: &'a mut Frame<'a>, area: Rect, theme: &'a Theme) -> Self {
        Self { frame, area, theme, focused: false, disabled: false }
    }

    pub fn focused(mut self, focused: bool) -> Self { self.focused = focused; self }
    pub fn disabled(mut self, disabled: bool) -> Self { self.disabled = disabled; self }

    /// Returns a context with the same frame/theme/focus state but a
    /// different area. The returned context borrows the frame for a
    /// shorter lifetime, so the parent context becomes valid again
    /// after the child context is dropped.
    pub fn with_area(&mut self, area: Rect) -> RenderContext<'_> {
        RenderContext {
            frame: self.frame,
            area,
            theme: self.theme,
            focused: self.focused,
            disabled: self.disabled,
        }
    }

    /// Convenience: render a widget into this context's area.
    ///
    /// Equivalent to `self.frame.render_widget(widget, self.area)`.
    pub fn render_widget<W: Widget>(&mut self, widget: W) {
        self.frame.render_widget(widget, self.area);
    }

    /// Returns the [`EventContext`] slice of this RenderContext.
    pub fn event_context(&self) -> EventContext {
        EventContext { focused: self.focused, disabled: self.disabled }
    }
}

impl From<&RenderContext<'_>> for EventContext {
    fn from(ctx: &RenderContext<'_>) -> Self {
        EventContext { focused: ctx.focused, disabled: ctx.disabled }
    }
}
```

### Updated `Component` trait

```rust
pub trait Component: Sized {
    type State;
    type Message: Clone;
    type Output: Clone;

    fn init() -> Self::State;
    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output>;

    /// Render the component using the given context.
    fn view(state: &Self::State, ctx: &mut RenderContext<'_>);

    /// Map an input event to a component message.
    fn handle_event(
        state: &Self::State,
        event: &Event,
        ctx: &EventContext,
    ) -> Option<Self::Message> {
        let _ = (state, event, ctx);
        None
    }

    /// Renders the component with optional tracing instrumentation.
    fn traced_view(state: &Self::State, ctx: &mut RenderContext<'_>) {
        #[cfg(feature = "tracing")]
        let _span = tracing::trace_span!(
            "component_view",
            component = std::any::type_name::<Self>(),
            area.x = ctx.area.x,
            area.y = ctx.area.y,
            area.width = ctx.area.width,
            area.height = ctx.area.height,
        ).entered();
        Self::view(state, ctx);
    }
}
```

### Migration pattern

Every component's `view()` impl follows this mechanical transformation:

```rust
// Before
fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, ctx: &ViewContext) {
    let style = if ctx.focused { theme.focused_style() } else { theme.normal_style() };
    let widget = Paragraph::new(state.label.as_str()).style(style);
    frame.render_widget(widget, area);
}

// After
fn view(state: &Self::State, ctx: &mut RenderContext<'_>) {
    let style = if ctx.focused { ctx.theme.focused_style() } else { ctx.theme.normal_style() };
    let widget = Paragraph::new(state.label.as_str()).style(style);
    ctx.render_widget(widget);
}
```

Substitution rules:
1. `frame: &mut Frame, area: Rect, theme: &Theme, ctx: &ViewContext` → `ctx: &mut RenderContext<'_>`
2. `theme.X` → `ctx.theme.X`
3. `frame.render_widget(w, area)` → `ctx.render_widget(w)` (preferred) or `ctx.frame.render_widget(w, ctx.area)`
4. `frame.render_stateful_widget(w, area, &mut state)` → `ctx.frame.render_stateful_widget(w, ctx.area, &mut state)` (no shortcut)
5. Sub-area rendering: `frame.render_widget(w, sub_area)` → `let mut sub = ctx.with_area(sub_area); sub.render_widget(w);` OR keep explicit form
6. `ViewContext` references in `handle_event` signatures → `EventContext`

Components that subdivide their area for child components use `ctx.with_area(child_area)` to construct a child context that borrows the same frame for a shorter lifetime, then pass it to the child component's view.

### Files affected

- `src/component/mod.rs` — trait definition, type definitions, ViewContext rename
- 73 component modules under `src/component/*/` — every `view()` impl, every `handle_event` impl
- `src/component/test_utils.rs` — test helper signatures
- `src/app/` — `App::view()` impl signatures and runtime call sites that invoke `Component::view`
- `src/harness/` — AppHarness component view calls
- 88 example files under `examples/` — most call `Component::view` or implement `App::view`
- `tests/` integration tests — any test that calls `Component::view` directly
- `benches/component_view.rs` and `benches/component_events.rs` — benchmark call sites
- `benches/memory.rs` — same
- `CHANGELOG.md` — breaking change entry under `[0.14.0]`
- `MIGRATION.md` — new section explaining the upgrade path

### PR strategy

**One atomic PR.** Partial updates won't compile because the trait
signature change is atomic. Splitting would require deprecation
shims that don't work for trait method signatures.

The PR will be large (estimated ~5000 lines changed across ~150 files)
but **mechanically reviewable**: most changes are pattern-substitution.
Reviewers can spot-check a few representative components and trust
the rest.

## Testing

### Regression coverage

The existing 16,136 tests must all pass after the migration. Component
behavior is unchanged — only signatures change. Specifically:

- **179 snapshot tests** must produce byte-identical output. Any
  snapshot diff is a bug.
- **1978 doc tests** must all pass. Doc test code blocks need updating
  to use the new API.
- **All unit tests** must pass without modification beyond signature
  updates at the call sites.

### New tests for `RenderContext` and `EventContext`

In `src/component/tests/render_context.rs` (new file):

1. `test_render_context_construction` — `RenderContext::new(frame, area, theme)` produces a context with focused=false, disabled=false.
2. `test_render_context_builder` — `.focused(true).disabled(true)` chain produces the expected values.
3. `test_render_context_with_area` — `with_area(child_area)` returns a context with the new area, same frame/theme/focus, and the parent context is valid again after the child context goes out of scope.
4. `test_render_context_render_widget` — calling `ctx.render_widget(widget)` produces the same rendered output as `frame.render_widget(widget, area)`.
5. `test_event_context_from_render_context` — the `From<&RenderContext<'_>>` impl produces matching focused/disabled fields.
6. `test_render_context_event_context_method` — `ctx.event_context()` matches the `From` impl.
7. `test_event_context_default` — `EventContext::default()` is `{ focused: false, disabled: false }`.
8. `test_event_context_builder` — `.focused(true)` and `.disabled(true)` work.

### Verification gates (in order)

1. `cargo check` — compiles
2. `cargo nextest run -p envision` — all 16,136 existing tests pass
3. `cargo test --doc -p envision` — all doc tests pass
4. **Snapshot byte-identical check**: `git diff -- src/**/snapshots/` after running tests should be empty. If any `.snap` file changed, that's a behavioral regression.
5. `cargo build --examples --all-features` — all 88 examples compile
6. `cargo clippy -p envision --all-targets -- -D warnings` — no warnings
7. `cargo fmt --check` — formatted
8. New `RenderContext` unit tests pass

## Risk and rollback

**Risk: medium-high.** This is the largest atomic refactor in the
project's history. Risks:

1. **Compile errors during migration** — partial updates won't compile.
   Mitigation: do all the work on a feature branch, only merge when
   the entire build is green.

2. **Lifetime issues with `Frame<'a>`** — `&'a mut Frame<'a>` is a
   self-referential lifetime that can be tricky. Mitigation: prototype
   the trait change with one or two components first to confirm the
   lifetime works, then cascade.

3. **`with_area` lifetime gymnastics** — the reborrow pattern needs
   careful annotation. Mitigation: write the unit test first to verify
   the borrow checker accepts the pattern.

4. **Snapshot test regressions** — any byte-level rendering difference
   indicates a bug introduced during migration. Mitigation: the
   snapshot equality check is the canary. No `cargo insta accept`
   allowed during this PR.

5. **Reviewer fatigue** — 150+ files in one PR. Mitigation: make the
   changes mechanical and pattern-based so spot-checking is enough.

**Rollback:** revert the single PR. Because the change is atomic, the
revert is also atomic.

## CHANGELOG / MIGRATION entries

### CHANGELOG (under `[0.14.0]` → `### Breaking`)

```markdown
- **`Component::view` signature changed** from
  `(state, frame, area, theme, ctx)` to `(state, ctx)`. The new
  `ctx: &mut RenderContext<'_>` bundles `frame`, `area`, `theme`,
  `focused`, and `disabled` into a single value. See MIGRATION.md
  for the upgrade path.

- **`ViewContext` renamed to `EventContext`.** It is now used only by
  `Component::handle_event`, not by `Component::view`. The fields
  and builder methods are unchanged.

- **`Component::handle_event` signature changed** from
  `(state, event, ctx: &ViewContext)` to
  `(state, event, ctx: &EventContext)`. Pure type rename.

- **`Component::traced_view` signature changed** to match the new
  `view` signature.
```

### MIGRATION.md

A new section explaining the upgrade path with before/after code
samples for the most common patterns:

1. Simple component view (Button-style)
2. Component view that uses theme heavily
3. Component view with sub-area rendering
4. App::view implementation
5. Test code that calls Component::view directly
6. Custom Component implementations
