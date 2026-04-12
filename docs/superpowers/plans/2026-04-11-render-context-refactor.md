# RenderContext Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor `Component::view()` from 5 parameters to 2 by introducing `RenderContext` (a wrapper for frame/area/theme/focus state) and renaming `ViewContext` → `EventContext`.

**Architecture:** Pure refactor with no behavioral changes. Add two new types in `src/component/mod.rs`. Change the `Component` trait signature. Cascade updates through all 73 components, all examples, all tests, and all integration code in a single atomic PR. Existing snapshot tests must produce byte-identical output.

**Tech Stack:** Rust (edition 2024), ratatui (Frame/Rect), cargo-nextest, insta snapshots.

**Spec:** `docs/superpowers/specs/2026-04-11-render-context-design.md`

**Target version:** 0.14.0 (breaking)

---

## Project Context

- **Working branch:** `render-context-refactor` (already created from main, spec already committed at `c9003d6`).
- **Atomic strategy:** This is one giant PR. The trait signature change cannot be done incrementally because all 73 components implement `view()` and the signature is part of the trait. Phase 1 adds types non-atomically; Phase 2 is the atomic switch.
- **Snapshot tests are the canary:** all 179 `.snap` files under `src/component/*/snapshots/` must produce byte-identical output after the migration. Any snapshot diff is a regression bug, NOT something to accept with `cargo insta accept`.
- **Test runner:** `cargo nextest run -p envision` (faster than `cargo test`).
- **Doc tests:** run via `cargo test --doc -p envision` (separate from nextest).
- **Signed commits required.** Git config has `commit.gpgsign=true`.
- **No warnings allowed:** `cargo clippy --all-targets -- -D warnings` must be clean.

---

## File Structure

This is a refactor across many files, but the file structure itself doesn't change. Files affected:

**Modified (no new files in src/):**
- `src/component/mod.rs` — type definitions, trait signature
- `src/component/test_utils.rs` — test helper signatures
- 73 component modules under `src/component/*/mod.rs` and their test files
- `src/app/mod.rs` and runtime code that calls `Component::view()`
- `src/harness/app_harness/mod.rs` — AppHarness component view calls
- 88 example files under `examples/*.rs`
- `tests/*.rs` — integration tests calling Component::view directly
- `benches/component_view.rs`, `benches/component_events.rs`, `benches/memory.rs`
- `CHANGELOG.md`
- `MIGRATION.md`

**New files:**
- `src/component/tests/render_context.rs` — unit tests for the new types (or inline in existing tests file)

---

## Task 1: Add `EventContext` and `RenderContext` types (TDD)

**Goal:** Introduce both types without yet changing the `Component` trait. Compiles cleanly. The new types coexist with the old `ViewContext` temporarily.

**Files:**
- Modify: `src/component/mod.rs` — add types
- Modify: `src/component/tests.rs` — add unit tests (or wherever component-level tests live)

---

- [ ] **Step 1.1: Verify the current state of `src/component/mod.rs`**

```bash
grep -n "ViewContext" src/component/mod.rs | head -10
```

Confirm `ViewContext` is defined at lines ~523-548 and the `Component::view` signature uses it at line ~611.

- [ ] **Step 1.2: Write failing unit tests for `RenderContext` and `EventContext`**

Create `src/component/tests/render_context.rs` (new file). If `src/component/tests/` doesn't exist as a directory yet, create it and ensure `src/component/mod.rs` declares `#[cfg(test)] mod tests;` somewhere.

Actually, check first whether `src/component/tests.rs` exists or `src/component/tests/` directory exists. If neither, write tests inline in `src/component/mod.rs` under `#[cfg(test)] mod render_context_tests { ... }`.

```bash
ls src/component/tests* 2>&1
```

If you see a file or directory, add tests there. Otherwise, add an inline test module at the end of `src/component/mod.rs` (just before any existing `#[cfg(test)]` block).

Add these tests:

```rust
#[cfg(test)]
mod render_context_tests {
    use super::*;
    use crate::component::test_utils::setup_render;

    #[test]
    fn test_event_context_default() {
        let ctx = EventContext::default();
        assert!(!ctx.focused);
        assert!(!ctx.disabled);
    }

    #[test]
    fn test_event_context_builder() {
        let ctx = EventContext::new().focused(true);
        assert!(ctx.focused);
        assert!(!ctx.disabled);

        let ctx = EventContext::new().focused(true).disabled(true);
        assert!(ctx.focused);
        assert!(ctx.disabled);
    }

    #[test]
    fn test_render_context_construction() {
        let (mut terminal, theme) = setup_render(60, 5);
        terminal
            .draw(|frame| {
                let area = frame.area();
                let ctx = RenderContext::new(frame, area, &theme);
                assert!(!ctx.focused);
                assert!(!ctx.disabled);
                assert_eq!(ctx.area, area);
            })
            .unwrap();
    }

    #[test]
    fn test_render_context_builder() {
        let (mut terminal, theme) = setup_render(60, 5);
        terminal
            .draw(|frame| {
                let area = frame.area();
                let ctx = RenderContext::new(frame, area, &theme)
                    .focused(true)
                    .disabled(true);
                assert!(ctx.focused);
                assert!(ctx.disabled);
            })
            .unwrap();
    }

    #[test]
    fn test_render_context_with_area() {
        let (mut terminal, theme) = setup_render(60, 10);
        terminal
            .draw(|frame| {
                let parent_area = frame.area();
                let mut ctx = RenderContext::new(frame, parent_area, &theme).focused(true);
                let child_area = ratatui::layout::Rect::new(5, 2, 20, 3);
                {
                    let child_ctx = ctx.with_area(child_area);
                    assert_eq!(child_ctx.area, child_area);
                    assert!(child_ctx.focused);
                    assert_eq!(child_ctx.theme as *const _, ctx.theme as *const _);
                }
                // Parent context is valid again here
                assert_eq!(ctx.area, parent_area);
                assert!(ctx.focused);
            })
            .unwrap();
    }

    #[test]
    fn test_render_context_render_widget() {
        use ratatui::widgets::Paragraph;
        let (mut terminal, theme) = setup_render(60, 5);
        terminal
            .draw(|frame| {
                let area = frame.area();
                let mut ctx = RenderContext::new(frame, area, &theme);
                ctx.render_widget(Paragraph::new("hello"));
            })
            .unwrap();

        let display = terminal.backend().to_string();
        assert!(display.contains("hello"));
    }

    #[test]
    fn test_event_context_from_render_context() {
        let (mut terminal, theme) = setup_render(60, 5);
        terminal
            .draw(|frame| {
                let area = frame.area();
                let ctx = RenderContext::new(frame, area, &theme)
                    .focused(true)
                    .disabled(false);
                let event_ctx: EventContext = (&ctx).into();
                assert!(event_ctx.focused);
                assert!(!event_ctx.disabled);
            })
            .unwrap();
    }

    #[test]
    fn test_render_context_event_context_method() {
        let (mut terminal, theme) = setup_render(60, 5);
        terminal
            .draw(|frame| {
                let area = frame.area();
                let ctx = RenderContext::new(frame, area, &theme)
                    .focused(true)
                    .disabled(true);
                let event_ctx = ctx.event_context();
                assert!(event_ctx.focused);
                assert!(event_ctx.disabled);
            })
            .unwrap();
    }
}
```

- [ ] **Step 1.3: Run the tests and verify they fail to compile**

```bash
cargo nextest run -p envision render_context_tests 2>&1 | tail -20
```

Expected: compilation errors about `EventContext` and `RenderContext` not existing.

- [ ] **Step 1.4: Add `EventContext` to `src/component/mod.rs`**

In `src/component/mod.rs`, just before the existing `pub struct ViewContext` at line ~523, add the new `EventContext`:

```rust
/// Context passed to [`Component::handle_event`].
///
/// Carries focus and disabled state from the parent so the component
/// can decide whether and how to handle events. Use [`RenderContext`]
/// for `view()`.
///
/// # Example
///
/// ```rust
/// use envision::component::EventContext;
///
/// let ctx = EventContext::default();
/// assert!(!ctx.focused);
///
/// let ctx = EventContext::new().focused(true).disabled(false);
/// assert!(ctx.focused);
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EventContext {
    /// Whether this component currently has keyboard focus.
    pub focused: bool,
    /// Whether this component is currently disabled.
    pub disabled: bool,
}

impl EventContext {
    /// Creates a new default EventContext (unfocused, enabled).
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the focused state (builder pattern).
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Sets the disabled state (builder pattern).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}
```

Leave the existing `ViewContext` definition untouched for now.

- [ ] **Step 1.5: Add `RenderContext` to `src/component/mod.rs`**

After the `EventContext` definition, add:

```rust
/// Context passed to [`Component::view`].
///
/// Bundles the frame, area, theme, and focus/disabled state into a
/// single value so component view signatures stay short and adding
/// new render-time fields is non-breaking.
///
/// # Example
///
/// ```rust,no_run
/// use envision::component::{Component, RenderContext};
/// use envision::theme::Theme;
/// use envision::backend::CaptureBackend;
/// use ratatui::Terminal;
///
/// let backend = CaptureBackend::new(80, 24);
/// let mut terminal = Terminal::new(backend).unwrap();
/// let theme = Theme::default();
/// terminal.draw(|frame| {
///     let area = frame.area();
///     let mut ctx = RenderContext::new(frame, area, &theme).focused(true);
///     // Pass `&mut ctx` to a component's `view` method.
/// }).unwrap();
/// ```
pub struct RenderContext<'a> {
    /// The ratatui frame to render into.
    pub frame: &'a mut Frame<'a>,
    /// The area within the frame to render to.
    pub area: Rect,
    /// The theme to use for styling.
    pub theme: &'a Theme,
    /// Whether the component currently has keyboard focus.
    pub focused: bool,
    /// Whether the component is currently disabled.
    pub disabled: bool,
}

impl<'a> RenderContext<'a> {
    /// Constructs a new RenderContext with `focused` and `disabled` both `false`.
    pub fn new(frame: &'a mut Frame<'a>, area: Rect, theme: &'a Theme) -> Self {
        Self {
            frame,
            area,
            theme,
            focused: false,
            disabled: false,
        }
    }

    /// Sets the focused state (builder pattern).
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Sets the disabled state (builder pattern).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Returns a context with the same frame, theme, and focus state but
    /// a different area.
    ///
    /// The returned context borrows the frame for a shorter lifetime, so
    /// the parent context becomes valid again after the child context
    /// goes out of scope.
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
    pub fn render_widget<W: ratatui::widgets::Widget>(&mut self, widget: W) {
        self.frame.render_widget(widget, self.area);
    }

    /// Returns the [`EventContext`] slice of this RenderContext.
    pub fn event_context(&self) -> EventContext {
        EventContext {
            focused: self.focused,
            disabled: self.disabled,
        }
    }
}

impl From<&RenderContext<'_>> for EventContext {
    fn from(ctx: &RenderContext<'_>) -> Self {
        EventContext {
            focused: ctx.focused,
            disabled: ctx.disabled,
        }
    }
}
```

- [ ] **Step 1.6: Re-export the new types from lib.rs**

In `src/lib.rs`, find the line that re-exports `ViewContext`:

```bash
grep -n "ViewContext" src/lib.rs
```

There should be a line like `pub use component::{Component, FocusManager, Toggleable, ViewContext};`. Update it to also export the new types:

```rust
pub use component::{Component, EventContext, FocusManager, RenderContext, Toggleable, ViewContext};
```

(Keep `ViewContext` in the export for now — Task 2 will remove it.)

- [ ] **Step 1.7: Verify the new types compile and tests pass**

```bash
cargo check -p envision 2>&1 | tail -10
```

Expected: clean compile. If there are lifetime errors on `RenderContext`, the most likely culprit is the `&'a mut Frame<'a>` self-referential lifetime. If `cargo check` fails, escalate — DO NOT try to bypass the lifetime issue. The trait change in Task 2 won't work without this.

Then run the new tests:

```bash
cargo nextest run -p envision render_context_tests 2>&1 | tail -10
```

Expected: all 8 tests pass.

Then run the doc test on the new `RenderContext`:

```bash
cargo test --doc -p envision RenderContext 2>&1 | tail -10
```

Expected: doc test passes.

- [ ] **Step 1.8: Run the full test suite to confirm no regressions**

```bash
cargo nextest run -p envision 2>&1 | tail -5
```

Expected: all existing tests pass (16,136+ tests) plus the 8 new ones. If any existing test fails, the type additions broke something — investigate.

- [ ] **Step 1.9: Format and lint**

```bash
cargo fmt
cargo clippy -p envision -- -D warnings 2>&1 | tail -3
```

Expected: no formatting diffs, no clippy warnings.

- [ ] **Step 1.10: Commit (signed)**

```bash
git add src/component/mod.rs src/lib.rs
git commit -S -m "$(cat <<'EOF'
Add RenderContext and EventContext types (foundation for 0.14.0)

Introduces RenderContext bundling frame/area/theme/focus state, and
EventContext as the renamed-soon successor to ViewContext for use in
handle_event. The Component trait still uses the old (frame, area,
theme, ctx) signature; the next commit makes the breaking switch.

Adds 8 unit tests covering construction, builder, with_area reborrow,
render_widget shortcut, and From conversion.

Part of docs/superpowers/specs/2026-04-11-render-context-design.md

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: Switch the `Component` trait + cascade through all callers (atomic)

**Goal:** Atomic change that updates the `Component` trait, removes `ViewContext`, and migrates every component, every test, every example, and every internal call site to use `RenderContext` and `EventContext`.

**Files:** Many. The expected change is roughly ~5000 lines across ~150 files. The transformation is mechanical pattern-substitution.

**This is the largest single task in the project's history. Work methodically.**

---

- [ ] **Step 2.1: Update the `Component` trait in `src/component/mod.rs`**

Change the trait method signatures:

```rust
// FROM
fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, ctx: &ViewContext);

// TO
fn view(state: &Self::State, ctx: &mut RenderContext<'_>);
```

```rust
// FROM
fn handle_event(state: &Self::State, event: &Event, ctx: &ViewContext) -> Option<Self::Message> {
    let _ = (state, event, ctx);
    None
}

// TO
fn handle_event(state: &Self::State, event: &Event, ctx: &EventContext) -> Option<Self::Message> {
    let _ = (state, event, ctx);
    None
}
```

```rust
// FROM
fn traced_view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, ctx: &ViewContext) {
    #[cfg(feature = "tracing")]
    let _span = tracing::trace_span!(
        "component_view",
        component = std::any::type_name::<Self>(),
        area.x = area.x,
        area.y = area.y,
        area.width = area.width,
        area.height = area.height,
    )
    .entered();
    Self::view(state, frame, area, theme, ctx);
}

// TO
fn traced_view(state: &Self::State, ctx: &mut RenderContext<'_>) {
    #[cfg(feature = "tracing")]
    let _span = tracing::trace_span!(
        "component_view",
        component = std::any::type_name::<Self>(),
        area.x = ctx.area.x,
        area.y = ctx.area.y,
        area.width = ctx.area.width,
        area.height = ctx.area.height,
    )
    .entered();
    Self::view(state, ctx);
}
```

Also update the `dispatch_event` function (search for it in `src/component/mod.rs` — it currently takes a `ViewContext`):

```bash
grep -n "dispatch_event\|fn dispatch_event" src/component/mod.rs
```

Update its signature: any `&ViewContext` parameter becomes `&EventContext`.

- [ ] **Step 2.2: Remove `ViewContext` definition and re-export**

In `src/component/mod.rs`, delete the existing `pub struct ViewContext` definition (was around line 523-548 before Task 1's additions). It's been replaced by `EventContext`.

In `src/lib.rs`, remove `ViewContext` from the re-export:

```rust
// Before
pub use component::{Component, EventContext, FocusManager, RenderContext, Toggleable, ViewContext};

// After
pub use component::{Component, EventContext, FocusManager, RenderContext, Toggleable};
```

Also check `src/lib.rs` for any prelude module that re-exports `ViewContext`:

```bash
grep -n "ViewContext" src/lib.rs
```

Replace all occurrences with `EventContext` and `RenderContext` (or just `EventContext` if the prelude doesn't include rendering primitives).

- [ ] **Step 2.3: Run cargo check to discover the cascade**

```bash
cargo check -p envision 2>&1 | head -100
```

Expected: many compile errors, all of them in component implementations and call sites that use the old `view()` signature or `ViewContext`.

Save the error list to a file for tracking:

```bash
cargo check -p envision 2>&1 > /tmp/render-context-errors.txt
```

Count the errors:

```bash
grep -c "^error" /tmp/render-context-errors.txt
```

This gives a rough sense of how many call sites need updating. Expect 200-500 errors initially.

- [ ] **Step 2.4: Update `src/component/test_utils.rs`**

```bash
grep -n "ViewContext\|fn view" src/component/test_utils.rs
```

Update any helper functions that:
- Take a `ViewContext` parameter → take an `EventContext` parameter
- Construct a `ViewContext` → construct a `RenderContext` if for view, `EventContext` if for event handling

If there's a `setup_render` helper or similar, it likely doesn't need changes (it returns `(terminal, theme)` and the caller does the rest).

- [ ] **Step 2.5: Update `src/component/mod.rs` `dispatch_event`**

The `dispatch_event` function takes `&ViewContext` and needs to take `&EventContext`. Update it.

- [ ] **Step 2.6: Update each component's `view()` and `handle_event()` impls**

This is the bulk of the work. For each of the 73 components under `src/component/*/`, update:

1. The `Component` trait impl's `view()` signature and body
2. The `Component` trait impl's `handle_event()` signature
3. Any internal helper functions that take `frame`/`area`/`theme`/`ViewContext` parameters
4. Any tests in that component's test files that call `view()` or `handle_event()` directly

The transformation pattern (apply mechanically):

```rust
// view() — BEFORE
fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, ctx: &ViewContext) {
    // body uses: frame, area, theme, ctx.focused, ctx.disabled
}

// view() — AFTER
fn view(state: &Self::State, ctx: &mut RenderContext<'_>) {
    // body uses: ctx.frame, ctx.area, ctx.theme, ctx.focused, ctx.disabled
}
```

```rust
// handle_event() — BEFORE
fn handle_event(state: &Self::State, event: &Event, ctx: &ViewContext) -> Option<Self::Message> {
    // ...
}

// handle_event() — AFTER (only the type name changes)
fn handle_event(state: &Self::State, event: &Event, ctx: &EventContext) -> Option<Self::Message> {
    // ...
}
```

For internal render helper functions that take separate frame/area/theme:

```rust
// BEFORE
fn render_header(frame: &mut Frame, area: Rect, theme: &Theme, focused: bool) { ... }

// Caller
render_header(frame, header_area, theme, ctx.focused);
```

You have two choices:
- **A.** Keep the helper signature as-is, just update the caller to pass `ctx.frame, header_area, ctx.theme, ctx.focused`. This is the path of least resistance and preserves internal API.
- **B.** Update the helper to take `&mut RenderContext` and use `ctx.with_area(header_area)` at the call site.

**Use approach A** for this PR. The goal is the trait surface change, not internal refactoring. We can revisit internal helpers in a follow-up.

For sub-area rendering when calling another Component's `view()`:

```rust
// BEFORE
SomeComponent::view(&state.child, frame, child_area, theme, &child_ctx);

// AFTER
let mut child_render_ctx = ctx.with_area(child_area);
child_render_ctx.focused = child_ctx.focused;
child_render_ctx.disabled = child_ctx.disabled;
SomeComponent::view(&state.child, &mut child_render_ctx);
```

Or more concisely if focused/disabled match the parent:

```rust
let mut child_render_ctx = ctx.with_area(child_area);
SomeComponent::view(&state.child, &mut child_render_ctx);
```

**Strategy:** work alphabetically. After updating each component, run:

```bash
cargo check -p envision -- --bin envision 2>&1 | grep "error\[" | wc -l
```

The error count should go down. When it hits zero, the migration is structurally complete.

**Components to update (alphabetical, all 73):**

accordion, alert_panel, big_text, box_plot, breadcrumb, button, calendar,
canvas, chart, checkbox, code_block, collapsible, command_palette,
confirm_dialog, conversation_view, data_grid, dependency_graph, dialog,
diff_viewer, divider, dropdown, event_stream, file_browser, flame_graph,
focus_manager, form, gauge, heatmap, help_panel, histogram, input_field,
key_hints, line_input, loading_list, log_correlation, log_viewer,
markdown_renderer, menu, metrics_dashboard, multi_progress, number_input,
paginator, pane_layout, progress_bar, radio_group, router, scroll_view,
scrollable_text, searchable_list, select, selectable_list, slider,
span_tree, sparkline, spinner, split_panel, status_bar, status_log,
step_indicator, styled_text, switch, tab_bar, table, tabs, terminal_output,
text_area, timeline, title_card, toast, tooltip, tree, treemap,
usage_display

Track progress in a checklist if helpful:

```bash
echo "=== Component migration progress ==="
for comp in accordion alert_panel big_text ...; do
    if grep -q "ctx: &mut RenderContext" src/component/$comp/mod.rs 2>/dev/null; then
        echo "✓ $comp"
    else
        echo "✗ $comp"
    fi
done
```

- [ ] **Step 2.7: Update `src/app/` runtime call sites**

The runtime is what eventually calls `Component::view()` for the App's view tree. Find call sites:

```bash
grep -rn "::view(" src/app/ | head -20
```

Most relevant: `App::view()` is implemented by users, but the runtime calls it. Check `src/app/runtime/mod.rs` and related files for any internal `view()` calls.

The `App` trait itself has a `view()` method too — check its signature:

```bash
grep -n "fn view" src/app/model/mod.rs
```

If `App::view()` takes the same 5-arg signature, update it to match the new pattern:

```rust
// BEFORE (if applicable)
fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme);

// AFTER
fn view(state: &Self::State, ctx: &mut RenderContext<'_>);
```

The `App::view()` doesn't currently have a `ViewContext` because the App is at the root (always focused). Decide whether the App should also use RenderContext for consistency. **Recommended: yes.** Apps benefit from the same `render_widget` shortcut and the future-proof signature.

Update the runtime to construct a `RenderContext` and call `App::view()`:

```rust
// In the runtime's draw closure
terminal.draw(|frame| {
    let area = frame.area();
    let mut ctx = RenderContext::new(frame, area, &theme);
    A::view(&state, &mut ctx);
})?;
```

- [ ] **Step 2.8: Update `src/harness/app_harness/mod.rs`**

```bash
grep -n "::view(\|ViewContext" src/harness/app_harness/mod.rs
```

Update any call sites to use the new API. AppHarness likely has internal `view()` calls for testing.

- [ ] **Step 2.9: Run cargo check until clean**

```bash
cargo check -p envision 2>&1 | tail -10
```

Expected: clean compile. If errors remain, address them one at a time. Common remaining issues:
- A test file you forgot to update
- A doc test in a comment that uses the old API
- An internal helper function that still takes `&ViewContext`

For doc tests inside source files (not separate test files), search for them:

```bash
grep -rn "ViewContext::default\(\)\|frame: &mut Frame" src/ | grep -v test
```

- [ ] **Step 2.10: Update all examples**

```bash
ls examples/*.rs | wc -l  # should be 89
```

For each example file, update:
- `App::view()` impl (if it implements App and has the old signature)
- Any `Component::view()` direct calls
- Any `ViewContext` references → `EventContext` for handle_event, `RenderContext` for view

```bash
for ex in examples/*.rs; do
    echo "=== $ex ==="
    grep -l "ViewContext\|fn view" "$ex" 2>/dev/null
done
```

Verify all examples compile:

```bash
cargo build --examples --all-features 2>&1 | tail -10
```

Expected: clean build of all 89 examples.

- [ ] **Step 2.11: Update integration tests**

```bash
grep -rn "ViewContext\|::view(" tests/ | head -20
```

For each integration test file in `tests/`, update:
- `ViewContext` references → `EventContext`
- `Component::view(state, frame, area, theme, &ctx)` → construct a `RenderContext` and pass it

```bash
cargo nextest run -p envision --tests 2>&1 | tail -10
```

Expected: all integration tests pass.

- [ ] **Step 2.12: Update benchmarks**

```bash
grep -rn "ViewContext\|::view(" benches/ | head -20
```

Three benchmark files likely need updates:
- `benches/component_view.rs`
- `benches/component_events.rs`
- `benches/memory.rs`

Update each to construct a `RenderContext` instead of passing 5 separate args.

Verify benchmarks compile (without running them):

```bash
cargo bench --no-run 2>&1 | tail -10
```

- [ ] **Step 2.13: Update doc tests**

Doc tests embedded in `///` comments throughout `src/` need updating. They're the easiest to forget.

```bash
cargo test --doc -p envision 2>&1 | tail -20
```

Expected: all doc tests pass. If any fail with "no method named X" or "expected ViewContext, found EventContext" errors, find the failing doc test and update it.

The most common pattern in doc tests:

```rust
/// ```rust
/// use envision::component::{Component, Button, ButtonState, ViewContext};
/// // ... old call ...
/// ```
```

Becomes:

```rust
/// ```rust
/// use envision::component::{Component, Button, ButtonState, RenderContext, EventContext};
/// // ... new call ...
/// ```
```

Many doc tests don't actually call `view()` and only need the import update.

- [ ] **Step 2.14: Run the full test suite — snapshot equality is critical**

```bash
cargo nextest run -p envision 2>&1 | tail -10
```

Expected: ALL tests pass. Specifically, the 179 snapshot tests must produce byte-identical output.

If any snapshot test fails, do **NOT** run `cargo insta accept`. A snapshot diff means the rendering output changed, which is a regression bug introduced during the migration. Investigate the failing component.

Check for unexpected snapshot changes:

```bash
git diff --stat src/component/*/snapshots/ | tail -5
```

Expected: zero changes to snapshot files.

- [ ] **Step 2.15: Format and lint**

```bash
cargo fmt
cargo clippy -p envision --all-targets -- -D warnings 2>&1 | tail -3
```

Expected: no formatting diffs, no clippy warnings.

- [ ] **Step 2.16: Final cargo check across all targets**

```bash
cargo check -p envision --all-targets --all-features 2>&1 | tail -3
```

Expected: clean.

- [ ] **Step 2.17: Update CHANGELOG.md**

Under `## [Unreleased]`, add a `### Breaking` section (or append to an existing one):

```markdown
### Breaking

- **`Component::view` signature changed** from
  `(state, frame, area, theme, ctx)` to `(state, ctx)`. The new
  `ctx: &mut RenderContext<'_>` bundles `frame`, `area`, `theme`,
  `focused`, and `disabled` into a single value. See MIGRATION.md
  for the upgrade path.

- **`ViewContext` renamed to `EventContext`.** It is now used only
  by `Component::handle_event`, not by `Component::view`. The fields
  and builder methods are unchanged.

- **`Component::handle_event` signature changed** from
  `(state, event, ctx: &ViewContext)` to
  `(state, event, ctx: &EventContext)`. Pure type rename.

- **`Component::traced_view` signature changed** to match the new
  `view` signature.
```

- [ ] **Step 2.18: Update MIGRATION.md**

Add a new section for 0.14.0:

```markdown
## Migrating from 0.13.x to 0.14.0

### `Component::view` signature change

The `view()` method on the `Component` trait now takes a single
`RenderContext` parameter instead of five separate parameters.

**Before (0.13.x):**

\```rust
fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, ctx: &ViewContext) {
    let style = if ctx.focused { theme.focused_style() } else { theme.normal_style() };
    frame.render_widget(Paragraph::new("hello").style(style), area);
}
\```

**After (0.14.0):**

\```rust
fn view(state: &Self::State, ctx: &mut RenderContext<'_>) {
    let style = if ctx.focused { ctx.theme.focused_style() } else { ctx.theme.normal_style() };
    ctx.render_widget(Paragraph::new("hello").style(style));
}
\```

**Substitution rules:**

1. Replace the parameter list `frame: &mut Frame, area: Rect, theme: &Theme, ctx: &ViewContext` with `ctx: &mut RenderContext<'_>`.
2. Replace `theme.X` with `ctx.theme.X`.
3. Replace `frame.render_widget(widget, area)` with `ctx.render_widget(widget)` (or equivalent `ctx.frame.render_widget(widget, ctx.area)`).
4. `ctx.focused` and `ctx.disabled` are still accessed directly — they're now fields on `RenderContext`.

### `ViewContext` renamed to `EventContext`

The type previously used by both `view()` and `handle_event()` is now
called `EventContext` and is used only by `handle_event()`. The fields
(`focused`, `disabled`) and builder methods are unchanged.

**Before:**

\```rust
fn handle_event(state: &Self::State, event: &Event, ctx: &ViewContext) -> Option<Self::Message> { ... }
\```

**After:**

\```rust
fn handle_event(state: &Self::State, event: &Event, ctx: &EventContext) -> Option<Self::Message> { ... }
\```

### Sub-area rendering for nested components

If your component renders a child into a sub-area, use `ctx.with_area`:

\```rust
// Before
SomeChild::view(&state.child, frame, child_area, theme, &ctx);

// After
let mut child_ctx = ctx.with_area(child_area);
SomeChild::view(&state.child, &mut child_ctx);
\```

The returned `child_ctx` borrows the frame for a shorter lifetime so
the parent context becomes valid again after the child render returns.

### Test code calling `Component::view` directly

**Before:**

\```rust
let (mut terminal, theme) = setup_render(60, 5);
terminal.draw(|frame| {
    Button::view(&state, frame, frame.area(), &theme, &ViewContext::default());
}).unwrap();
\```

**After:**

\```rust
let (mut terminal, theme) = setup_render(60, 5);
terminal.draw(|frame| {
    let mut ctx = RenderContext::new(frame, frame.area(), &theme);
    Button::view(&state, &mut ctx);
}).unwrap();
\```
```

- [ ] **Step 2.19: Final full verification**

```bash
cargo nextest run -p envision 2>&1 | tail -3
cargo test --doc -p envision 2>&1 | tail -3
cargo build --examples --all-features 2>&1 | tail -3
cargo clippy -p envision --all-targets -- -D warnings 2>&1 | tail -3
cargo fmt --check
```

All five must succeed.

- [ ] **Step 2.20: Commit (signed) — the atomic migration**

```bash
git add -A
git commit -S -m "$(cat <<'EOF'
Migrate Component::view to RenderContext, rename ViewContext

BREAKING CHANGE for 0.14.0.

Changes Component::view from:
    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, ctx: &ViewContext)
to:
    fn view(state: &Self::State, ctx: &mut RenderContext<'_>)

Renames ViewContext -> EventContext (used only by handle_event now,
not by view). Updates Component::handle_event to take &EventContext.
Updates Component::traced_view to match the new view signature.

Cascade through all 73 components, all 89 examples, all integration
tests, all benchmarks, all doc tests, the runtime, and the AppHarness.

Snapshot tests verify byte-identical rendering output — no behavioral
changes, only signature changes.

CHANGELOG and MIGRATION.md updated with upgrade path.

Part of docs/superpowers/specs/2026-04-11-render-context-design.md

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: Push and open PR

- [ ] **Step 3.1: Push the branch**

```bash
git push -u origin render-context-refactor 2>&1 | tail -5
```

- [ ] **Step 3.2: Open the PR**

```bash
gh pr create --title "Refactor Component::view to use RenderContext (BREAKING, 0.14.0)" --body "$(cat <<'EOF'
## Summary

**Breaking change for 0.14.0.** Refactors `Component::view` from 5 parameters to 2 by introducing `RenderContext`, which bundles `frame`, `area`, `theme`, `focused`, and `disabled` into a single value.

**Before:**
\`\`\`rust
fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, ctx: &ViewContext);
\`\`\`

**After:**
\`\`\`rust
fn view(state: &Self::State, ctx: &mut RenderContext<'_>);
\`\`\`

Also renames \`ViewContext\` -> \`EventContext\` since it is now used only by \`handle_event\`, not by \`view\`.

## What changed

- New types: \`RenderContext<'a>\` and \`EventContext\` in \`src/component/mod.rs\`
- \`Component::view\` and \`Component::traced_view\` use \`&mut RenderContext<'_>\`
- \`Component::handle_event\` uses \`&EventContext\` (renamed from \`ViewContext\`)
- All 73 components migrated to the new signatures
- All 89 examples updated
- All integration tests, doc tests, benchmarks, and the AppHarness updated
- CHANGELOG.md and MIGRATION.md updated with upgrade path

## Why

The 5-parameter \`view()\` signature was the audit's #1 trust-eroding finding for the library: users implementing custom components had to understand and propagate ratatui's \`Frame\` and \`Rect\` types plus the framework's \`Theme\` and \`ViewContext\`. The new \`RenderContext\` simplifies the signature, provides a place for future render-time fields without breaking changes, and matches how mature TUI/UI frameworks bundle render state.

Design spec: \`docs/superpowers/specs/2026-04-11-render-context-design.md\`

## Test plan

- [x] All 16,000+ existing tests pass without modification beyond signature updates
- [x] All 179 snapshot tests produce byte-identical output (no \`cargo insta accept\` allowed)
- [x] 8 new unit tests for \`RenderContext\` and \`EventContext\`
- [x] All 89 examples compile
- [x] \`cargo doc --no-deps --all-features\` succeeds
- [x] \`cargo clippy --all-targets -- -D warnings\` clean
- [x] \`cargo fmt\` clean

## Migration

See MIGRATION.md for the 0.14.0 upgrade path with before/after code samples.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

- [ ] **Step 3.3: Check CI**

```bash
gh pr checks $(gh pr view --json number -q .number)
```

Wait for all required checks to pass before merging.

---

## Definition of done

- [ ] `RenderContext` and `EventContext` types exist in `src/component/mod.rs`
- [ ] `Component::view`, `Component::handle_event`, and `Component::traced_view` use the new types
- [ ] `ViewContext` is removed from the codebase (no occurrences in `grep -r "ViewContext" src/`)
- [ ] All 73 components compile and pass tests
- [ ] All 89 examples compile
- [ ] All snapshot tests produce byte-identical output (zero `.snap` file changes in the diff)
- [ ] All doc tests pass
- [ ] CHANGELOG.md has the breaking change entry under `[Unreleased]`
- [ ] MIGRATION.md has a 0.14.0 section with upgrade examples
- [ ] PR opened, CI green
- [ ] PR will be squash-merged
