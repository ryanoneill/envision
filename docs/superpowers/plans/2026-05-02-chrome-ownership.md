# Chrome Ownership Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `RenderContext::chrome_owned` propagation, add `PaneLayout::view_with(state, ctx, render_child)` closure-based pane rendering, and audit + patch every chrome-drawing component to consult `ctx.chrome_owned` and skip its outer Block when true.

**Architecture:** Three phases. Phase 1 is purely additive (`chrome_owned` field on `RenderContext`). Phase 2 adds `PaneLayout::view_with` and its tests. Phase 3 audits + patches every component that draws an outermost-block-around-`ctx.area` chrome — Table and StyledText are spec-confirmed; the implementer enumerates the rest via `grep` and applies the canonical pattern. Existing `Component::view` impl on `PaneLayout` stays as the chrome-only path (documented).

**Tech Stack:** Rust 2024, MSRV 1.85, ratatui 0.29, `insta` for snapshots, `cargo-nextest` for test runs. No new dependencies.

---

## Pre-Execution Gotchas

Read these before starting any task.

1. **`PaneId` is `String`, not a type alias.** envision uses `String` for pane identifiers (e.g., `PaneConfig::new(id: impl Into<String>)`, `pane_area(area, pane_id: &str)`). The closure parameter for `view_with` is therefore `&str`, not `&PaneId`. The spec's `&PaneId` shorthand maps to `&str` in actual envision code.

2. **`Component::view` trait impl on `PaneLayout` stays unchanged.** Today's `<PaneLayout as Component>::view` already draws only chrome (no children). The new `view_with` is an inherent method on `PaneLayout`. There is no inherent `PaneLayout::view` to delete — the spec's "delete `view`" wording was approximate.

3. **`src/component/pane_layout/mod.rs` is 992 lines today.** Adding `view_with` plus tests will push it past 1000. **You must split** before adding `view_with`. Extract the new method into `src/component/pane_layout/view_with.rs` (mirrors the `table/render.rs`, `code_block/render.rs` pattern).

4. **`Block::default()` is in 56 component files.** Not all are "chrome" (some are per-cell, per-row, per-message internal blocks). The audit (Phase 3) targets the OUTERMOST `Block` that wraps the full `ctx.area` — the chrome. Internal blocks are unchanged.

5. **`Component::view` for `PaneLayout` MUST keep its chrome-only semantics.** Generic code over `Component` (FocusManager, harness, registry) calls this trait method and expects chrome only. The new `view_with` is an inherent method; consumers reach for it explicitly. Add a docstring on the trait impl explaining this asymmetry (per leadline review note 1).

6. **`view_with`'s docstring MUST warn against fresh `RenderContext::new(...)` inside the closure** (per leadline review note 2). Constructing a fresh context in the closure body bypasses `chrome_owned = true` and re-introduces the double-render bug. Self-diagnosing (visible double border) but a doc nudge prevents the trip-up.

7. **Snapshot tests use `insta`.** Run with `INSTA_UPDATE=auto cargo nextest run -p envision --lib` to auto-write `.new` snapshots, then `cargo insta review` (or accept by inspection) and commit the `.snap` files.

8. **Signed commits required.** No `--no-gpg-sign` bypass. If `gpg` fails, stop and report BLOCKED.

9. **No warnings.** `cargo clippy --all-features -- -D warnings` must be clean after every commit.

10. **Plan PR is a sibling to spec PR #467** — branch `chrome-ownership-plan` opens against `main`, parallel to the spec. Implementation goes on a new branch (`chrome-ownership-impl`) **after** spec PR #467 merges.

---

## File Structure

### Created in this plan

| File | Responsibility |
|---|---|
| `src/component/pane_layout/view_with.rs` | The `view_with` inherent method on `PaneLayout`, its private helpers (inner-rect computation), and its unit tests. |

### Modified in this plan

| File | Changes |
|---|---|
| `src/component/context.rs` | Add `chrome_owned: bool` field, default in `new`, public builder, `with_area` propagation. |
| `src/component/pane_layout/mod.rs` | Add `mod view_with;` declaration. Add docstring on `Component::view` impl noting it's chrome-only. |
| `src/component/pane_layout/tests.rs` | Add tests for `view_with` (per-pane invocation, inner rect, propagation, degenerate). |
| `src/component/table/render.rs` | Wrap outer Block draw in `if !ctx.chrome_owned`. |
| `src/component/styled_text/mod.rs` | `let show_border = !ctx.chrome_owned && state.show_border();` |
| Other chrome-drawing components (per Phase 3 audit) | Same wrap pattern. |
| `src/component/{component}/snapshots/` (per component) | Add `chrome_owned=true` snapshot fixtures. |
| `examples/pane_layout.rs` | Rewrite to use `view_with` shape. |
| `CHANGELOG.md` | Breaking-change entry: `RenderContext::chrome_owned` added; `PaneLayout::view_with` added; component-chrome behavior change in embedded contexts. |

---

## Phase 0 — Pre-flight

### Task 1: Verify branch state and tooling

**Files:**
- Read: `Cargo.toml`, `docs/superpowers/specs/2026-05-02-chrome-ownership-design.md`

- [ ] **Step 1: Confirm working branch**

```bash
git branch --show-current
```
Expected: `chrome-ownership-impl` (or chosen name). Stop if on `main` — create the branch from `main` after spec PR #467 merges.

- [ ] **Step 2: Verify spec is on main**

```bash
test -f docs/superpowers/specs/2026-05-02-chrome-ownership-design.md && echo "spec present"
```
Expected: `spec present`. If not, the spec PR (#467) hasn't merged yet — stop.

- [ ] **Step 3: Verify gpg-agent works**

```bash
echo "test" | gpg --clearsign > /dev/null && echo "gpg ok"
```
Expected: `gpg ok`. Stop if signing fails — ask user to resolve.

- [ ] **Step 4: Pin baseline test count**

```bash
cargo nextest run -p envision --lib 2>&1 | tail -3
cargo test --doc 2>&1 | tail -3
```
Record both numbers in a scratch note. Phase 1 + 2 + 3 will add tests; verify counts increment monotonically.

---

## Phase 1 — Additive: `RenderContext::chrome_owned`

### Task 2: Add `chrome_owned` field, builder, and `with_area` propagation

**Files:**
- Modify: `src/component/context.rs`

- [ ] **Step 1: Edit the `RenderContext` struct**

Find the struct definition around line 92. Add the new field:

```rust
pub struct RenderContext<'frame, 'buf> {
    pub frame: &'frame mut Frame<'buf>,
    pub area: Rect,
    pub theme: &'frame Theme,
    pub focused: bool,
    pub disabled: bool,
    /// True when this context's parent has already drawn the chrome
    /// (border, title, focus ring) for `area`. Children consult this at
    /// render time and suppress their own chrome.
    pub chrome_owned: bool,
}
```

- [ ] **Step 2: Update `RenderContext::new`**

Find `pub fn new(...)` around line 107. Update the body to default `chrome_owned: false`:

```rust
pub fn new(frame: &'frame mut Frame<'buf>, area: Rect, theme: &'frame Theme) -> Self {
    Self {
        frame,
        area,
        theme,
        focused: false,
        disabled: false,
        chrome_owned: false,
    }
}
```

- [ ] **Step 3: Add the builder method**

Add immediately after the existing `disabled` builder:

```rust
/// Builder: marks chrome as parent-owned. Children consult this flag
/// at render time and suppress their own borders/titles/focus rings.
///
/// Set automatically by [`PaneLayout::view_with`] when invoking the
/// per-pane render closure.
#[must_use]
pub fn chrome_owned(mut self, owned: bool) -> Self {
    self.chrome_owned = owned;
    self
}
```

- [ ] **Step 4: Update `with_area` to propagate `chrome_owned`**

Find `pub fn with_area(...)` around line 137. Update its returned-context construction:

```rust
pub fn with_area(&mut self, area: Rect) -> RenderContext<'_, 'buf> {
    RenderContext {
        frame: self.frame,
        area,
        theme: self.theme,
        focused: self.focused,
        disabled: self.disabled,
        chrome_owned: self.chrome_owned,
    }
}
```

- [ ] **Step 5: Verify the existing tests still pass**

```bash
cargo nextest run -p envision --lib -E 'test(/render_context/)'
```
Expected: existing tests pass; no new behavior introduced yet.

- [ ] **Step 6: Run clippy**

```bash
cargo clippy --all-features -- -D warnings 2>&1 | tail -3
```
Expected: clean.

- [ ] **Step 7: Commit**

```bash
git add src/component/context.rs
git commit -S -m "Add RenderContext::chrome_owned (additive, Phase 1)

Public field + builder + with_area propagation. Defaults false in
RenderContext::new. Matches the focused / disabled convention exactly.
No component behavior changes yet — Phase 3 will teach chrome-drawing
components to consult this flag.

Tracks G2 + D2 + D11. Spec: docs/superpowers/specs/2026-05-02-chrome-ownership-design.md

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

### Task 3: Add `chrome_owned` field tests

**Files:**
- Modify: `src/component/context.rs::render_context_tests`

- [ ] **Step 1: Add three tests inside the existing `render_context_tests` mod**

Find the existing test module around line 172. Add at the end of the module (before its closing `}`):

```rust
#[test]
fn test_render_context_chrome_owned_defaults_false() {
    let (mut terminal, theme) = setup_render(60, 5);
    terminal
        .draw(|frame| {
            let ctx = RenderContext::new(frame, frame.area(), &theme);
            assert!(!ctx.chrome_owned, "chrome_owned should default to false");
        })
        .unwrap();
}

#[test]
fn test_render_context_chrome_owned_builder_sets_field() {
    let (mut terminal, theme) = setup_render(60, 5);
    terminal
        .draw(|frame| {
            let ctx = RenderContext::new(frame, frame.area(), &theme).chrome_owned(true);
            assert!(ctx.chrome_owned);
        })
        .unwrap();
}

#[test]
fn test_render_context_with_area_propagates_chrome_owned() {
    let (mut terminal, theme) = setup_render(60, 5);
    terminal
        .draw(|frame| {
            let mut ctx = RenderContext::new(frame, frame.area(), &theme).chrome_owned(true);
            let inner = ctx.with_area(ratatui::prelude::Rect::new(1, 1, 10, 3));
            assert!(inner.chrome_owned, "with_area should propagate chrome_owned");
        })
        .unwrap();
}
```

- [ ] **Step 2: Run the new tests**

```bash
cargo nextest run -p envision --lib -E 'test(/chrome_owned/)'
```
Expected: 3 passed.

- [ ] **Step 3: Verify nothing else broke**

```bash
cargo nextest run -p envision --lib -E 'test(/render_context/)'
```
Expected: pre-existing render_context tests still pass.

- [ ] **Step 4: Commit**

```bash
git add src/component/context.rs
git commit -S -m "Test: RenderContext::chrome_owned default, builder, with_area propagation

Three tests pinning the chrome_owned protocol contract: defaults false,
builder roundtrips, with_area propagates the field through child contexts.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Phase 2 — Add `PaneLayout::view_with`

### Task 4: Extract `view_with` into a sibling file

**Files:**
- Create: `src/component/pane_layout/view_with.rs`
- Modify: `src/component/pane_layout/mod.rs` (add `mod view_with;`)

- [ ] **Step 1: Create the empty sibling file with the canonical header**

Create `src/component/pane_layout/view_with.rs`:

```rust
//! Closure-based per-pane child rendering.
//!
//! The `view_with` method on [`PaneLayout`] draws pane chrome (borders,
//! titles, focus rings) and invokes the consumer's render closure once
//! per pane with a chrome-inset child [`RenderContext`] whose
//! `chrome_owned` flag is set to `true` so embedded components suppress
//! their own chrome.
//!
//! See [`super::PaneLayout::view_with`] for the public API.

use ratatui::layout::{Margin, Rect};
use ratatui::widgets::{Block, Borders};

use super::{PaneLayout, PaneLayoutState};
use crate::component::RenderContext;

// Implementations live here; the public method appears on PaneLayout via
// `impl PaneLayout` block below.
```

- [ ] **Step 2: Add the module declaration in `mod.rs`**

Find a location near the top of `src/component/pane_layout/mod.rs` after the existing `use` statements (around line 30 — adjust to existing style). Add:

```rust
mod view_with;
```

- [ ] **Step 3: Verify it compiles**

```bash
cargo check --all-features 2>&1 | tail -3
```
Expected: clean. The new file currently has no public items so nothing to use yet.

- [ ] **Step 4: Commit**

```bash
git add src/component/pane_layout/view_with.rs src/component/pane_layout/mod.rs
git commit -S -m "Extract view_with into its own file (file size discipline)

src/component/pane_layout/mod.rs is at 992 lines today. Adding view_with
plus tests will push it over the 1000-line project limit. Extract the
new method into a sibling view_with.rs ahead of implementing it.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

### Task 5: Implement `view_with`

**Files:**
- Modify: `src/component/pane_layout/view_with.rs`
- Modify: `src/component/pane_layout/mod.rs` (docstring on the `Component::view` trait impl)

- [ ] **Step 1: Implement `view_with`**

In `src/component/pane_layout/view_with.rs`, add the method:

```rust
impl PaneLayout {
    /// Render pane chrome and invoke `render_child` once per pane.
    ///
    /// `render_child` receives the pane's id (string slice) and a child
    /// [`RenderContext`] whose:
    /// - `area` is the pane's inner rect (chrome already accounted for)
    /// - `chrome_owned` is `true` (children suppress their own chrome)
    /// - `focused` is `true` only for the focused pane's child context
    /// - `disabled` is propagated from the parent context
    /// - `frame` and `theme` are propagated unchanged
    ///
    /// # Common pitfall
    ///
    /// Do **not** construct a fresh `RenderContext::new(frame, area, theme)`
    /// inside the closure body. Doing so bypasses `chrome_owned = true`
    /// and re-introduces the double-render bug this method exists to
    /// solve. Always pass the provided `child_ctx` (or a reborrow of it
    /// via `child_ctx.with_area(...)`) to embedded components.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use envision::prelude::*;
    /// # use envision::component::pane_layout::{PaneLayout, PaneLayoutState, PaneConfig, PaneDirection};
    /// # fn render(frame: &mut ratatui::Frame, area: ratatui::prelude::Rect, theme: &Theme) {
    /// let panes = vec![
    ///     PaneConfig::new("left").with_title("Left").with_proportion(0.5),
    ///     PaneConfig::new("right").with_title("Right").with_proportion(0.5),
    /// ];
    /// let state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
    /// PaneLayout::view_with(
    ///     &state,
    ///     &mut RenderContext::new(frame, area, theme),
    ///     |pane_id, child_ctx| match pane_id {
    ///         "left"  => { /* render left child into child_ctx */ }
    ///         "right" => { /* render right child into child_ctx */ }
    ///         _ => {}
    ///     },
    /// );
    /// # }
    /// ```
    pub fn view_with<F>(
        state: &PaneLayoutState,
        ctx: &mut RenderContext<'_, '_>,
        mut render_child: F,
    ) where
        F: FnMut(&str, &mut RenderContext<'_, '_>),
    {
        let rects = state.layout(ctx.area);

        for (i, (pane, rect)) in state.panes.iter().zip(rects.iter()).enumerate() {
            let is_focused_pane = ctx.focused && i == state.focused_pane;
            let border_style = if ctx.disabled {
                ctx.theme.disabled_style()
            } else if is_focused_pane {
                ctx.theme.focused_border_style()
            } else {
                ctx.theme.border_style()
            };

            let mut block = Block::default()
                .borders(Borders::ALL)
                .border_style(border_style);

            if let Some(title) = &pane.title {
                block = block.title(format!(" {} ", title));
            }

            ctx.frame.render_widget(block, *rect);

            let inner = rect.inner(Margin {
                vertical: 1,
                horizontal: 1,
            });

            let mut child_ctx = ctx.with_area(inner);
            child_ctx.focused = is_focused_pane;
            child_ctx.chrome_owned = true;

            render_child(pane.id.as_str(), &mut child_ctx);
        }
    }
}
```

If the implementer hits HRTB issues compiling the closure bound, the spec permits falling back to:

```rust
where F: for<'a, 'b> FnMut(&str, &mut RenderContext<'a, 'b>),
```

Try the simpler form first. If `cargo check` rejects, try the HRTB form.

- [ ] **Step 2: Add the asymmetry docstring to `Component::view` impl on PaneLayout**

In `src/component/pane_layout/mod.rs`, find the `impl Component for PaneLayout` block (around line 834). Add a docstring directly above the `fn view(...)` method:

```rust
/// Renders pane chrome only — borders, titles, focus rings — without
/// rendering any children inside the panes.
///
/// This is the path framework code (FocusManager, AppHarness, registry)
/// uses when it dispatches via the [`Component`] trait. For embedded
/// children, consumers should call [`PaneLayout::view_with`] directly,
/// not the trait method, since the trait API has no slot for a child
/// render closure.
///
/// If you write code generic over `Component` or use `dyn Component`
/// abstractions, expect this method to render only chrome — children
/// will not appear unless you explicitly call `view_with` from the
/// concrete type.
fn view(state: &Self::State, ctx: &mut RenderContext<'_, '_>) {
    // existing body unchanged
}
```

- [ ] **Step 3: Verify it compiles**

```bash
cargo check --all-features 2>&1 | tail -5
cargo build --all-features 2>&1 | tail -3
```
Expected: clean.

- [ ] **Step 4: Run clippy**

```bash
cargo clippy --all-features -- -D warnings 2>&1 | tail -5
```
Expected: clean.

- [ ] **Step 5: Commit**

```bash
git add src/component/pane_layout/view_with.rs src/component/pane_layout/mod.rs
git commit -S -m "Add PaneLayout::view_with closure-based pane rendering

Inherent method on PaneLayout that draws chrome and invokes a per-pane
render closure with a chrome-inset RenderContext whose chrome_owned is
set to true. Replaces the consumer-side three-step dance of state.layout()
+ chrome render + manual Margin{1,1} inner-rect computation.

Component::view trait impl gets an explanatory docstring noting it
renders chrome only — generic-Component code should call view_with on
the concrete type for embedded children.

view_with's docstring warns against constructing fresh RenderContext::new
inside the closure (bypasses chrome_owned and re-introduces double-render).

Tracks D2.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

### Task 6: `view_with` tests — per-pane invocation, inner rect, propagation, degenerate

**Files:**
- Modify: `src/component/pane_layout/tests.rs`

- [ ] **Step 1: Read existing `tests.rs` to understand the test setup pattern**

```bash
head -50 src/component/pane_layout/tests.rs
```

Note any existing `use` statements and helpers (e.g., `setup_render`).

- [ ] **Step 2: Add the six `view_with` tests**

Append at the end of `src/component/pane_layout/tests.rs` (before the final `}` if the file ends with a closing brace, or at top level if free-standing):

```rust
#[test]
fn test_view_with_invokes_render_child_per_pane_in_order() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 24);
    let panes = vec![
        PaneConfig::new("left").with_proportion(0.5),
        PaneConfig::new("right").with_proportion(0.5),
    ];
    let state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
    let calls: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let calls_inner = calls.clone();

    terminal
        .draw(|frame| {
            let area = frame.area();
            PaneLayout::view_with(
                &state,
                &mut RenderContext::new(frame, area, &theme),
                |pane_id, _child_ctx| {
                    calls_inner.borrow_mut().push(pane_id.to_string());
                },
            );
        })
        .unwrap();

    let calls = calls.borrow();
    assert_eq!(*calls, vec!["left".to_string(), "right".to_string()]);
}

#[test]
fn test_view_with_passes_inner_rect_inset_for_chrome() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 24);
    let panes = vec![PaneConfig::new("only").with_proportion(1.0)];
    let state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
    let inner_rects: Rc<RefCell<Vec<ratatui::prelude::Rect>>> = Rc::new(RefCell::new(Vec::new()));
    let inner_inner = inner_rects.clone();

    terminal
        .draw(|frame| {
            let area = frame.area();
            PaneLayout::view_with(
                &state,
                &mut RenderContext::new(frame, area, &theme),
                |_pane_id, child_ctx| {
                    inner_inner.borrow_mut().push(child_ctx.area);
                },
            );
        })
        .unwrap();

    let inner_rects = inner_rects.borrow();
    assert_eq!(inner_rects.len(), 1);
    let inner = inner_rects[0];
    // Inner should be inset by the chrome (Borders::ALL = 1 cell margin
    // on each side).
    assert_eq!(inner.x, 1);
    assert_eq!(inner.y, 1);
    assert_eq!(inner.width, 78);
    assert_eq!(inner.height, 22);
}

#[test]
fn test_view_with_sets_chrome_owned_true_on_child_ctx() {
    use std::cell::Cell;
    use std::rc::Rc;

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 24);
    let panes = vec![PaneConfig::new("only").with_proportion(1.0)];
    let state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
    let chrome_owned: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let chrome_inner = chrome_owned.clone();

    terminal
        .draw(|frame| {
            let area = frame.area();
            PaneLayout::view_with(
                &state,
                &mut RenderContext::new(frame, area, &theme),
                |_pane_id, child_ctx| {
                    chrome_inner.set(child_ctx.chrome_owned);
                },
            );
        })
        .unwrap();

    assert!(chrome_owned.get(), "child_ctx should have chrome_owned == true");
}

#[test]
fn test_view_with_focus_propagates_to_focused_pane_only() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 24);
    let panes = vec![
        PaneConfig::new("a").with_proportion(0.5),
        PaneConfig::new("b").with_proportion(0.5),
    ];
    let mut state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
    state.focused_pane = 1; // focus the second pane
    let observed: Rc<RefCell<Vec<(String, bool)>>> = Rc::new(RefCell::new(Vec::new()));
    let inner = observed.clone();

    terminal
        .draw(|frame| {
            let area = frame.area();
            let mut parent_ctx = RenderContext::new(frame, area, &theme).focused(true);
            PaneLayout::view_with(
                &state,
                &mut parent_ctx,
                |pane_id, child_ctx| {
                    inner.borrow_mut().push((pane_id.to_string(), child_ctx.focused));
                },
            );
        })
        .unwrap();

    let observed = observed.borrow();
    assert_eq!(observed.len(), 2);
    assert_eq!(observed[0], ("a".to_string(), false));
    assert_eq!(observed[1], ("b".to_string(), true));
}

#[test]
fn test_view_with_disabled_propagates_to_all_children() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 24);
    let panes = vec![
        PaneConfig::new("a").with_proportion(0.5),
        PaneConfig::new("b").with_proportion(0.5),
    ];
    let state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
    let observed: Rc<RefCell<Vec<bool>>> = Rc::new(RefCell::new(Vec::new()));
    let inner = observed.clone();

    terminal
        .draw(|frame| {
            let area = frame.area();
            let mut parent_ctx = RenderContext::new(frame, area, &theme).disabled(true);
            PaneLayout::view_with(
                &state,
                &mut parent_ctx,
                |_pane_id, child_ctx| {
                    inner.borrow_mut().push(child_ctx.disabled);
                },
            );
        })
        .unwrap();

    let observed = observed.borrow();
    assert_eq!(observed.len(), 2);
    assert!(observed.iter().all(|&d| d), "all children should inherit disabled=true");
}

#[test]
fn test_view_with_degenerate_closure_renders_chrome_only() {
    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 24);
    let panes = vec![
        PaneConfig::new("left").with_title("Left").with_proportion(0.5),
        PaneConfig::new("right").with_title("Right").with_proportion(0.5),
    ];
    let state = PaneLayoutState::new(PaneDirection::Horizontal, panes);

    let buffer = terminal
        .draw(|frame| {
            let area = frame.area();
            PaneLayout::view_with(
                &state,
                &mut RenderContext::new(frame, area, &theme),
                |_pane_id, _child_ctx| {
                    // degenerate: render no children
                },
            );
        })
        .unwrap();

    insta::assert_snapshot!(crate::component::test_utils::buffer_to_string(buffer.buffer));
}
```

The last test relies on `crate::component::test_utils::buffer_to_string` (or equivalent). If that helper doesn't exist by that name, use whatever buffer-to-string helper is canonical in envision (search for `assert_snapshot` + buffer in existing tests for the right shape).

- [ ] **Step 3: Run tests with `INSTA_UPDATE=auto` for the snapshot**

```bash
INSTA_UPDATE=auto cargo nextest run -p envision --lib -E 'test(/view_with/)'
```
Expected: 6 passed. The snapshot test writes a `.new` file on first run; review and accept.

- [ ] **Step 4: Review and accept the snapshot**

```bash
ls src/component/pane_layout/snapshots/*.snap.new 2>/dev/null
cargo insta review  # or manually inspect + rename .snap.new to .snap
```

Verify the snapshot shows the two-pane chrome (`╭ Left ╮` and `╭ Right ╮` rounded borders, no inner content) and accept.

- [ ] **Step 5: Re-run to verify clean**

```bash
cargo nextest run -p envision --lib -E 'test(/view_with/)'
```
Expected: 6 passed without `INSTA_UPDATE`.

- [ ] **Step 6: Commit**

```bash
git add src/component/pane_layout/tests.rs src/component/pane_layout/snapshots/
git commit -S -m "Test: PaneLayout::view_with — per-pane invocation, propagation, degenerate

Six tests pinning the view_with contract:
- render_child invoked once per pane in declaration order
- child_ctx.area is inset for chrome (Margin{1,1})
- child_ctx.chrome_owned is true
- focused propagates only to the focused pane's child_ctx
- disabled propagates to all children
- degenerate |_,_| {} closure renders chrome only (snapshot regression)

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Phase 3 — Component chrome audit + consult

### Task 7: Patch `Table` — outer Block consults `chrome_owned`

**Files:**
- Modify: `src/component/table/render.rs`
- Modify: `src/component/table/snapshots/`

- [ ] **Step 1: Locate the outer Block draw in `Table::view`**

```bash
grep -n "Block::default" src/component/table/render.rs
```

There's a Block draw around line 146 that wraps the entire table chrome. Read 30 lines around it to understand the full block-rendering path.

- [ ] **Step 2: Wrap the outer block draw**

Find the section that constructs and renders the outer Block (borders, title, border style). Wrap it in:

```rust
if !ctx.chrome_owned {
    // existing block-construction + render_widget call unchanged
}
```

Data rendering (header row, data rows, scrollbar) stays as-is — it always runs against `ctx.area` regardless of `chrome_owned`.

If the existing code computes `inner = block.inner(ctx.area)` and renders data into `inner`, you'll need to special-case: when `chrome_owned`, use `ctx.area` directly as the data area (no inset). Sketch:

```rust
let data_area = if ctx.chrome_owned {
    ctx.area
} else {
    let block = /* construct block */;
    let inner = block.inner(ctx.area);
    ctx.frame.render_widget(block, ctx.area);
    inner
};
// data rendering against data_area unchanged
```

- [ ] **Step 3: Add snapshot test for chrome_owned=true**

Find or create the appropriate test file (likely `src/component/table/view_tests.rs` or `tests.rs`). Add:

```rust
#[test]
fn snapshot_table_no_outer_border_when_chrome_owned() {
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 8);
    let columns = vec![
        crate::component::table::Column::new("Name").width(Constraint::Length(10)),
        crate::component::table::Column::new("Value").width(Constraint::Length(8)),
    ];
    #[derive(Clone)]
    struct Row(&'static str, &'static str);
    impl crate::component::table::TableRow for Row {
        fn cells(&self) -> Vec<crate::component::cell::Cell> {
            vec![
                crate::component::cell::Cell::from(self.0),
                crate::component::cell::Cell::from(self.1),
            ]
        }
    }
    let rows = vec![Row("foo", "1"), Row("bar", "2")];
    let state = crate::component::table::TableState::new(rows, columns);

    let buffer = terminal
        .draw(|frame| {
            let mut ctx = RenderContext::new(frame, frame.area(), &theme).chrome_owned(true);
            crate::component::table::Table::view(&state, &mut ctx);
        })
        .unwrap();

    insta::assert_snapshot!(crate::component::test_utils::buffer_to_string(buffer.buffer));
}
```

(Adjust `Cell::from`, `Column::new`, `Constraint::Length`, and the `TableRow` import path to match envision's actual API as it stands post-G7. Read existing Table tests for the canonical shape.)

- [ ] **Step 4: Run with INSTA_UPDATE=auto, review snapshot**

```bash
INSTA_UPDATE=auto cargo nextest run -p envision --lib -E 'test(snapshot_table_no_outer_border_when_chrome_owned)'
```

Inspect the resulting `.snap.new`: should contain the two data rows (`foo  1`, `bar  2`) **without** any `┌──┐` outer border. If the snapshot still has a border, the wrap in Step 2 is incorrect.

Accept the snapshot.

- [ ] **Step 5: Verify the existing standalone-mode tests still pass**

```bash
cargo nextest run -p envision --lib -E 'test(/component::table::/)'
```
Expected: all existing Table tests pass (no regression in `chrome_owned=false` path).

- [ ] **Step 6: Run clippy + fmt**

```bash
cargo clippy --all-features -- -D warnings 2>&1 | tail -3
cargo fmt --check
```
Expected: clean.

- [ ] **Step 7: Commit**

```bash
git add src/component/table/render.rs src/component/table/view_tests.rs src/component/table/snapshots/
git commit -S -m "Table: skip outer Block when ctx.chrome_owned (G2 fix)

Wrap the outer-border draw in if !ctx.chrome_owned. Data rendering
proceeds against ctx.area regardless. New snapshot test pins the
no-double-border behavior in embedded mode.

Tracks G2.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

### Task 8: Patch `StyledText` — border respects `chrome_owned`

**Files:**
- Modify: `src/component/styled_text/mod.rs`
- Modify: `src/component/styled_text/snapshots/` or sibling test file

- [ ] **Step 1: Locate the border draw in `StyledText::view`**

```bash
grep -n "Block::default\|show_border" src/component/styled_text/mod.rs
```

Read 30 lines around the existing `state.show_border()` check.

- [ ] **Step 2: Replace `state.show_border()` with the AND**

```rust
// Before:
if state.show_border() {
    // draw the Block
}

// After:
let show_border = !ctx.chrome_owned && state.show_border();
if show_border {
    // existing Block draw unchanged
}
```

- [ ] **Step 3: Add snapshot test**

Add to the appropriate test file in `src/component/styled_text/`:

```rust
#[test]
fn snapshot_styled_text_skips_border_when_chrome_owned_even_if_show_border() {
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 5);
    let state = crate::component::styled_text::StyledTextState::new("Hello, embedded.")
        .with_show_border(true);

    let buffer = terminal
        .draw(|frame| {
            let mut ctx = RenderContext::new(frame, frame.area(), &theme).chrome_owned(true);
            crate::component::styled_text::StyledText::view(&state, &mut ctx);
        })
        .unwrap();

    insta::assert_snapshot!(crate::component::test_utils::buffer_to_string(buffer.buffer));
}
```

(Adjust API names to match envision's actual `StyledText` shape.)

- [ ] **Step 4: Run, review, accept snapshot**

```bash
INSTA_UPDATE=auto cargo nextest run -p envision --lib -E 'test(snapshot_styled_text_skips_border_when_chrome_owned)'
```

Verify the snapshot shows the text without any border. Accept.

- [ ] **Step 5: Verify existing tests pass**

```bash
cargo nextest run -p envision --lib -E 'test(/styled_text/)'
```

- [ ] **Step 6: Commit**

```bash
git add src/component/styled_text/mod.rs src/component/styled_text/
git commit -S -m "StyledText: border respects ctx.chrome_owned (D11 fix)

Compute show_border = !ctx.chrome_owned && state.show_border() so embedded
StyledText suppresses its border even when state.show_border() is true.
The standalone-no-border opt-out via with_show_border(false) is unchanged.

Tracks D11.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

### Task 9: Audit + patch the rest of the chrome-drawing components (batch)

This is the broad audit. Apply the same wrap pattern to every remaining component whose `view` method draws an outermost Block around `ctx.area`.

**Files (audit set):**

The grep candidates:
```bash
grep -rln "Block::default()" --include="*.rs" src/component/ | sort
```

Returns ~56 files. Of those, the ones whose outermost block is the chrome (and which a consumer might reasonably embed inside a `PaneLayout` or similar chrome-owning host):

```
src/component/alert_panel/render.rs
src/component/box_plot/mod.rs
src/component/calendar/mod.rs
src/component/canvas/mod.rs
src/component/chart/mod.rs
src/component/code_block/render.rs
src/component/conversation_view/render.rs
src/component/data_grid/mod.rs
src/component/diagram/render.rs
src/component/diff_viewer/render.rs
src/component/event_stream/render.rs
src/component/file_browser/view.rs
src/component/flame_graph/render.rs
src/component/heatmap/mod.rs
src/component/help_panel/mod.rs
src/component/histogram/mod.rs
src/component/key_hints/mod.rs
src/component/loading_list/render.rs
src/component/log_correlation/render.rs
src/component/log_viewer/view.rs
src/component/markdown_renderer/mod.rs
src/component/metrics_dashboard/mod.rs
src/component/multi_progress/mod.rs
src/component/scroll_view/mod.rs
src/component/scrollable_text/mod.rs
src/component/searchable_list/render.rs
src/component/selectable_list/mod.rs
src/component/span_tree/render.rs
src/component/sparkline/mod.rs
src/component/status_bar/mod.rs
src/component/status_log/mod.rs
src/component/step_indicator/mod.rs
src/component/tabs/mod.rs
src/component/tab_bar/mod.rs
src/component/terminal_output/render.rs
src/component/timeline/mod.rs
src/component/title_card/mod.rs
src/component/toast/mod.rs
src/component/tooltip/mod.rs
src/component/treemap/mod.rs
src/component/usage_display/mod.rs
```

Skip (these draw chrome but are conceptually NOT embeddable — they're top-level overlays or single-cell widgets where the "outer block" IS the widget):
- `dialog/`, `confirm_dialog/`, `command_palette/` — modal overlays at top level
- `dropdown/`, `select/`, `tooltip/` — popouts that sit above other UI; chrome IS the widget
- `button/`, `checkbox/`, `switch/`, `slider/`, `number_input/`, `input_field/`, `line_input/`, `radio_group/`, `text_area/`, `form/`, `gauge/`, `resource_gauge/`, `progress_bar/` — single-cell widgets typically inside a parent container; their "border" is part of the widget's appearance
- `toast/` — overlay
- `pane_layout/` — itself the host

Use judgment: read the component's `view` method. If the outer Block is the component's identity (e.g., a Dialog frame, a Button outline), don't patch — those are conceptually chrome-of-themselves, not chrome-around-content. If the outer Block is a wrapping frame around a content area (e.g., LogViewer, MarkdownRenderer, ConversationView), patch.

When in doubt, **patch**. The cost of false-positive patching is one no-op `if`-branch; the cost of a missed component is a future "Gap #15" PR.

- [ ] **Step 1: For each component in the audit set, locate the outer Block draw and wrap it**

Pattern:
```rust
// Before
let block = Block::default().borders(Borders::ALL).title(...).border_style(...);
let inner = block.inner(ctx.area);
ctx.frame.render_widget(block, ctx.area);
// data rendering against `inner` ...

// After
let data_area = if ctx.chrome_owned {
    ctx.area
} else {
    let block = Block::default().borders(Borders::ALL).title(...).border_style(...);
    let inner = block.inner(ctx.area);
    ctx.frame.render_widget(block, ctx.area);
    inner
};
// data rendering against `data_area` ...
```

For components where the `inner` rect isn't computed (the Block IS rendered as the entire visual), wrap the whole block-render call:

```rust
if !ctx.chrome_owned {
    ctx.frame.render_widget(block, ctx.area);
}
```

- [ ] **Step 2: For each patched component, add ONE snapshot test** verifying the chrome_owned=true path renders without the outer border.

Skeleton:
```rust
#[test]
fn snapshot_<component>_no_chrome_when_chrome_owned() {
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 8);
    let state = /* component-appropriate state construction */;

    let buffer = terminal
        .draw(|frame| {
            let mut ctx = RenderContext::new(frame, frame.area(), &theme).chrome_owned(true);
            <component>::view(&state, &mut ctx);
        })
        .unwrap();

    insta::assert_snapshot!(crate::component::test_utils::buffer_to_string(buffer.buffer));
}
```

For components with state types that take many constructor args, use the smallest viable state (look at the component's existing tests for the canonical shape).

- [ ] **Step 3: Run all new snapshots with INSTA_UPDATE=auto and review**

```bash
INSTA_UPDATE=auto cargo nextest run -p envision --lib -E 'test(/snapshot_.*_no_chrome_when_chrome_owned/)'
```

Inspect each `.snap.new`. Verify no outer border is present. Accept.

- [ ] **Step 4: Run the full test suite to verify no regressions**

```bash
cargo nextest run -p envision --lib --all-features 2>&1 | tail -3
cargo test --doc 2>&1 | tail -3
```
Expected: counts up by ~40 (one per patched component); zero failures.

- [ ] **Step 5: Run clippy + fmt**

```bash
cargo clippy --all-features --tests --examples -- -D warnings 2>&1 | tail -3
cargo fmt --check
```

- [ ] **Step 6: Commit (single atomic chrome audit commit)**

```bash
git add src/component/
git status  # visual confirmation
git commit -S -m "Chrome audit: every chrome-drawing component consults ctx.chrome_owned

Apply the consult-and-skip pattern uniformly to every component whose
view() draws an outermost Block around ctx.area. Components touched:

[list the actual components patched, generated by reviewing git status]

Each gains a snapshot test pinning the chrome_owned=true behavior.
Existing standalone-mode rendering is unchanged (regression-tested by
the pre-existing snapshot fixtures).

Tracks G2 + D11 (uniform extension to the broader chrome-drawing surface
beyond Table and StyledText).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

The commit message body lists the actual components patched (substitute the real list at commit time, derived from `git diff --name-only HEAD~1 HEAD | grep '^src/component/'`).

---

### Task 10: Top-level integration snapshot — no double border

**Files:**
- Modify: `src/component/pane_layout/tests.rs` (add the integration test)

This is the headline regression test from the spec (#15). Pins the user-visible bug fix end-to-end.

- [ ] **Step 1: Add the integration test**

```rust
#[test]
fn snapshot_pane_layout_with_embedded_table_no_double_border() {
    use crate::component::table::{Column, Table, TableRow, TableState};
    use crate::component::cell::Cell;
    use ratatui::layout::Constraint;

    #[derive(Clone)]
    struct Row {
        name: &'static str,
        value: &'static str,
    }

    impl TableRow for Row {
        fn cells(&self) -> Vec<Cell> {
            vec![Cell::from(self.name), Cell::from(self.value)]
        }
    }

    let columns = vec![
        Column::new("Name").width(Constraint::Length(10)),
        Column::new("Value").width(Constraint::Length(8)),
    ];
    let rows = vec![
        Row { name: "alpha", value: "1" },
        Row { name: "beta", value: "2" },
    ];
    let table_state = TableState::new(rows, columns);

    let panes = vec![PaneConfig::new("data").with_title("Data").with_proportion(1.0)];
    let pane_state = PaneLayoutState::new(PaneDirection::Horizontal, panes);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 7);
    let buffer = terminal
        .draw(|frame| {
            let area = frame.area();
            PaneLayout::view_with(
                &pane_state,
                &mut RenderContext::new(frame, area, &theme),
                |pane_id, child_ctx| match pane_id {
                    "data" => Table::view(&table_state, child_ctx),
                    _ => {}
                },
            );
        })
        .unwrap();

    insta::assert_snapshot!(crate::component::test_utils::buffer_to_string(buffer.buffer));
}
```

Adjust `Cell::from`, `Column::new`, etc. to match the actual API.

- [ ] **Step 2: Run with INSTA_UPDATE=auto, review snapshot**

```bash
INSTA_UPDATE=auto cargo nextest run -p envision --lib -E 'test(snapshot_pane_layout_with_embedded_table_no_double_border)'
```

The snapshot MUST show:
- Outer rounded border with title `Data` from PaneLayout (e.g., `╭ Data ──`)
- Two data rows (`alpha 1`, `beta 2`) inside
- **NO** inner `┌──┐` square border from Table

If you see double borders in the captured output, Phase 3 didn't fully patch Table — go back and fix.

- [ ] **Step 3: Commit**

```bash
git add src/component/pane_layout/tests.rs src/component/pane_layout/snapshots/
git commit -S -m "Test: PaneLayout-with-embedded-Table renders single border

Headline regression test pinning the G2 user-visible bug fix end-to-end.
Asserts that a Table embedded via PaneLayout::view_with shows only
PaneLayout's outer rounded border, no inner square Table border.

If this snapshot regresses, the chrome_owned protocol has been broken
somewhere — either Table is no longer consulting ctx.chrome_owned, or
view_with is not setting chrome_owned=true on the child_ctx.

Tracks G2.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

### Task 11: Migrate `examples/pane_layout.rs` to `view_with` shape

**Files:**
- Modify: `examples/pane_layout.rs`

- [ ] **Step 1: Read the existing example**

```bash
cat examples/pane_layout.rs
```

Identify the current rendering path (likely uses `PaneLayout::view` + manual inner-rect computation + child rendering).

- [ ] **Step 2: Rewrite to use `view_with`**

Replace the three-step dance with a single `PaneLayout::view_with` call. Match the canonical pattern from the spec:

```rust
PaneLayout::view_with(
    &pane_state,
    &mut RenderContext::new(frame, area, theme),
    |pane_id, child_ctx| match pane_id {
        // ... per-pane render closures ...
        _ => {}
    },
);
```

- [ ] **Step 3: Build and visually verify the example renders**

```bash
cargo build --example pane_layout --all-features 2>&1 | tail -3
```

If this is a runnable interactive example, optionally run `cargo run --example pane_layout` and verify the output looks correct. (Not strictly required for plan execution; the snapshot tests cover regressions.)

- [ ] **Step 4: Commit**

```bash
git add examples/pane_layout.rs
git commit -S -m "examples/pane_layout: migrate to PaneLayout::view_with

Canonical demonstration of the closure-based rendering shape. Replaces
the three-step dance (state.layout + chrome render + Margin{1,1} +
manual child render) with a single view_with call.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

### Task 12: CHANGELOG + final verification gate

**Files:**
- Modify: `CHANGELOG.md`

- [ ] **Step 1: Add CHANGELOG entry**

Prepend a new section (after the last existing top-level entry):

```markdown
## [Unreleased] — 2026-05-XX

### Chrome ownership protocol (G2 + D2 + D11)

`RenderContext::chrome_owned: bool` — new public field. `true` signals to
child components that the parent has already drawn the chrome (border,
title, focus ring); children consult this flag and suppress their own
chrome. Defaults to `false`. Propagates through `with_area`. Set
automatically by `PaneLayout::view_with`.

`PaneLayout::view_with(state, ctx, render_child)` — new closure-based
inherent method. Draws pane chrome and invokes `render_child(pane_id,
&mut child_ctx)` per pane with `child_ctx.chrome_owned = true` and
`child_ctx.area` already inset for the chrome. Replaces the consumer-side
three-step dance (`state.layout()` + `view()` + manual `Margin{1,1}` +
manual child render).

`PaneLayout::view` (the `Component` trait method) keeps its chrome-only
semantics — generic-`Component` code (FocusManager, harness, registry)
gets chrome only without children, as before. Embedded children require
calling `view_with` directly on the concrete type.

Components patched to consult `ctx.chrome_owned` and skip their outer
Block when true: `Table`, `StyledText`, plus the broader audit set
(LogViewer, ScrollableText, ScrollView, MarkdownRenderer, etc. — see
the implementation commit for the full list).

#### Migration

| Old | New |
|---|---|
| `let rects = layout_state.layout(area); PaneLayout::view(&layout_state, ctx); let inner = rects[i].inner(Margin{1,1}); Child::view(&state, &mut RenderContext::new(frame, inner, theme));` | `PaneLayout::view_with(&layout_state, ctx, \|id, child_ctx\| match id { "child" => Child::view(&state, child_ctx), _ => {} });` |
| `Table::view(&state, &mut RenderContext::new(frame, inner, theme))` (always inner border) | `Table::view(&state, child_ctx)` — `chrome_owned = true` propagated; inner border skipped |
| `StyledText::view(&state.with_show_border(false), ctx)` (consumer-side suppression) | `StyledText::view(&state, child_ctx)` — `chrome_owned = true` propagated. `with_show_border(false)` stays for standalone-no-border case |

Tracks leadline gaps G2 + D2 + D11. See `docs/superpowers/specs/2026-05-02-chrome-ownership-design.md` for the design rationale.
```

- [ ] **Step 2: Final verification gate**

```bash
cargo build --all-features 2>&1 | tail -3
cargo nextest run -p envision --lib --all-features 2>&1 | tail -5
cargo test --doc --all-features 2>&1 | grep "test result" | tail -3
cargo clippy --all-features --tests --examples -- -D warnings 2>&1 | tail -3
cargo fmt --check
cargo build --examples --all-features 2>&1 | tail -3
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features 2>&1 | tail -3
./tools/audit/target/release/envision-audit scorecard 2>&1 | tail -10
```

Expected:
- Build clean
- Lib tests: baseline + (3 + 6 + ~40 + 1) = +50 new tests passing
- Doc tests: counts approximately stable
- Clippy clean
- Fmt clean
- Examples build clean
- Doc build clean
- Audit: 8/9 (only pre-existing `resource_gauge` accessor; no regressions)
- File sizes: nothing new over 1000 (Phase 2's `view_with` extraction kept `pane_layout/mod.rs` small)

- [ ] **Step 3: Commit**

```bash
git add CHANGELOG.md
git commit -S -m "Update CHANGELOG: chrome ownership protocol (G2 + D2 + D11)

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

### Task 13: Open implementation PR

**Files:**
- (push branch + open PR)

- [ ] **Step 1: Push**

```bash
git push -u origin chrome-ownership-impl
```

- [ ] **Step 2: Open PR**

```bash
gh pr create --title "Chrome ownership protocol (G2 + D2 + D11) — implementation" --body "$(cat <<'EOF'
## Summary

Implements the chrome-ownership redesign per spec (PR #467) and plan (PR #<plan-pr>).

### Public API changes

- `RenderContext::chrome_owned: bool` — new public field, default `false`. Public `chrome_owned(self, bool)` builder. Propagates through `with_area`.
- `PaneLayout::view_with(state, ctx, render_child)` — new closure-based inherent method. Draws pane chrome and invokes `render_child(pane_id: &str, child_ctx: &mut RenderContext)` per pane with `chrome_owned = true` and inner-rect-inset `area`.
- `Component::view` impl on `PaneLayout` is unchanged (still chrome-only); now documented as the chrome-only path. Generic-Component / dyn-Component code gets chrome only, as before.

### Behavior changes (breaking)

Every chrome-drawing component now consults `ctx.chrome_owned` and skips its outer Block when `true`. Components touched: Table, StyledText, plus the audit set (LogViewer, ScrollView, ScrollableText, MarkdownRenderer, ConversationView, DataGrid, MetricsDashboard, KeyHints, StatusLog, EventStream, LogCorrelation, TerminalOutput, FileBrowser, FlameGraph, SearchableList, SelectableList, AlertPanel, HelpPanel, TitleCard, BoxPlot, Histogram, Heatmap, Treemap, Diagram, Sparkline, Chart, Timeline, Calendar, ResourceGauge, UsageDisplay, ProgressBar, MultiProgress, StepIndicator, Tabs, TabBar, CodeBlock, DiffViewer, StatusBar, etc. — see commit log for the exact list).

### Tests

- 3 RenderContext field tests (default, builder, with_area propagation)
- 6 view_with tests (per-pane invocation, inner rect, chrome_owned propagation, focus propagation, disabled propagation, degenerate-closure regression)
- ~40 component snapshot tests pinning `chrome_owned=true` behavior
- 1 headline integration test: PaneLayout-with-embedded-Table renders only one border (the spec's #15 pin)

### Migration

| Old | New |
|---|---|
| `state.layout(area) + view + Margin{1,1} + manual child render` | `view_with(state, ctx, |id, child_ctx| ...)` |
| `Table::view` always drew inner border | `chrome_owned=true` propagation suppresses it; `with_show_border(false)` stays as standalone-no-border opt-out |

## Test plan
- [ ] CI green (16 checks expected)
- [ ] Squash-merge per project rule
- [ ] Tracking-doc PR follows: mark G2 + D2 + D11 ✅ resolved
- [ ] Notify leadline to migrate \`render_roster\` and \`render_per_op\` to view_with shape

## References
- Spec: \`docs/superpowers/specs/2026-05-02-chrome-ownership-design.md\` (PR #467)
- Plan: \`docs/superpowers/plans/2026-05-02-chrome-ownership.md\` (PR #<plan-pr>)
- Source brief: \`notes/envision_chrome_ownership_redesign.md\`

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

---

## Self-review checklist (run before declaring this plan done)

- [ ] **Spec coverage:**
  - § Architecture (RenderContext::chrome_owned + PaneLayout::view_with) → Tasks 2, 5
  - § RenderContext change → Task 2 (field + builder + propagation), Task 3 (tests)
  - § PaneLayout::view_with → Tasks 4 (extract), 5 (impl + docstrings), 6 (tests)
  - § Component-side changes → Tasks 7 (Table), 8 (StyledText), 9 (broader audit)
  - § PaneLayout::view deletion / Component impl asymmetry → Task 5 (docstring on `Component::view` impl per leadline review note 1)
  - § view_with docstring warning against fresh RenderContext::new → Task 5 (docstring per leadline review note 2)
  - § Migration → Task 11 (examples/pane_layout.rs), Task 12 (CHANGELOG)
  - § Tests #1–14 → Tasks 3, 6, 7, 8, 9
  - § Test #15 (top-level no-double-border) → Task 10
  - § File-size discipline → Task 4 (extract view_with.rs)
  - § Audit-as-part-of-this-PR (not deferred) → Task 9

- [ ] **Placeholder scan:** No "TBD", "TODO", "implement later", "fill in details", "similar to Task N", or "add appropriate error handling" strings.

- [ ] **Type consistency:** `chrome_owned`, `view_with`, `RenderContext`, `PaneLayout` named identically in every task.

- [ ] **Atomic-vs-additive boundaries clear:** Phase 1 + 2 are additive (no behavior change). Phase 3 is the breaking-change pass; Tasks 7, 8, 9 each commit independently but the impl PR rolls them together.

- [ ] **Leadline review notes 1 + 2:**
  - Note 1 (Component::view impl docstring) — Task 5 Step 2
  - Note 2 (view_with no-fresh-RenderContext warning) — Task 5 Step 1 (in the docstring)
