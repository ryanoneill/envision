# Chrome ownership redesign — design spec

**Date:** 2026-05-02
**Status:** Approved; implementation plan to follow.
**Tracks:** leadline gaps **G2** + **D2** + **D11** in `docs/customer-feedback/2026-05-01-leadline-gaps.md`
**Source brief:** `~/workspace/ryanoneill/rust-ai-explorations/notes/envision_chrome_ownership_redesign.md`

---

## TL;DR

Embedding components inside a `PaneLayout` today is a three-step dance with hardcoded magic
numbers, and the embedded components don't know they're embedded — so they double-render
their own borders inside their host's chrome. Three independent gaps share one root cause:
**envision has no protocol for "I'm rendered inside a parent that owns the chrome."**

Add `RenderContext::chrome_owned: bool` so children can consult-and-skip. Add
`PaneLayout::view_with(state, ctx, render_child)` so the host computes inner rects and
invokes the consumer's per-pane render closure with `chrome_owned = true` already set on the
child context. Delete `PaneLayout::view` — the degenerate "draw chrome but no children" case
is `view_with(state, ctx, |_, _| {})`.

The result: consumers stop typing `Margin { vertical: 1, horizontal: 1 }`, stop calling
`with_show_border(false)` to suppress double-rendered chrome, and stop holding envision's
internal margin sizes in their heads. Future chrome enhancements (thicker borders, focus
outlines, padded headers) become envision-internal — the inner-rect computation lives in
exactly one place.

---

## Goals

1. **One protocol for chrome ownership** — `RenderContext::chrome_owned` is the single
   signal passed from parent to child. No per-component opt-out flags, no per-component
   `with_borders(...)` builders.
2. **Closure-based pane rendering** — `view_with` lets envision own inner-rect computation
   and chrome-context construction; consumers express only "what goes in each pane."
3. **Compile-time path** — consumers cannot accidentally double-render after migrating;
   the framework wires `chrome_owned = true` automatically when invoking the closure.
4. **Self-symmetric `RenderContext`** — `chrome_owned` matches the `focused` / `disabled`
   convention exactly: public field, public builder method, propagated by `with_area`,
   default `false` in `RenderContext::new`.
5. **Audit + fix all chrome-drawing components** — `Table`, `StyledText`, plus any other
   in-tree component that draws its own border/title/focus ring should consult
   `chrome_owned`. Doing the audit now is cheaper than landing the same fix piecemeal as
   future consumers hit it.
6. **Single atomic breaking-change PR** — pre-1.0; no shim, no deprecation.

## Non-goals

- **Granular chrome ownership** (e.g. `enum ChromeOwnership { None, BorderOnly, BorderAndTitle, Full }`).
  Today every host that owns chrome owns *all* of it. If a future host wants finer
  granularity, the field can evolve to an enum without churning the protocol shape. YAGNI.
- **Inner-rect computation for non-`PaneLayout` hosts.** This spec covers the `PaneLayout`
  closure flow because that's where leadline's pain originates. Other compound components
  (e.g. `SplitPanel`) may grow their own `view_with`-style methods later if a real consumer
  surfaces the same dance; that's separate work.
- **Standalone-no-border opt-out via `chrome_owned`.** `chrome_owned = true` means *a parent
  is drawing my chrome*. The "I want no chrome at all in standalone mode" case is what
  existing builders like `StyledText::with_show_border(false)` are for. Both coexist;
  composition is `let show = !ctx.chrome_owned && state.show_border();`.
- **`Router` interaction.** `Router` switches *which* `App::view` runs — chrome ownership
  is a within-frame concern. Out of scope.

---

## Design

### `RenderContext` change (`src/component/context.rs`)

Add `chrome_owned: bool` as a public field, defaulting to `false` in `RenderContext::new`,
with a public builder method and propagation through `with_area`:

```rust
pub struct RenderContext<'frame, 'buf> {
    pub frame: &'frame mut Frame<'buf>,
    pub area: Rect,
    pub theme: &'frame Theme,
    pub focused: bool,
    pub disabled: bool,
    /// True when this context's parent has already drawn the chrome
    /// (border, title, focus ring) for `area`. Children should suppress
    /// their own chrome and render only content.
    pub chrome_owned: bool,
}

impl<'frame, 'buf> RenderContext<'frame, 'buf> {
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

    /// Builder: marks chrome as parent-owned. Children consult this flag
    /// at render time and suppress their own borders/titles/focus rings.
    #[must_use]
    pub fn chrome_owned(mut self, owned: bool) -> Self {
        self.chrome_owned = owned;
        self
    }

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
}
```

`EventContext::From<&RenderContext>` is unchanged. Chrome ownership is render-time only;
it has no effect on event routing.

#### Why public field, not private + `pub(crate)` setter

Three reasons:

1. **Convention symmetry.** `focused` and `disabled` are public fields with public
   builders. Adding `chrome_owned` as a privately-set field invents a new visibility
   convention for one field; readers have to remember which fields are which.
2. **Downstream test ergonomics.** External crates (leadline, future consumers, future
   examples) can construct a `chrome_owned = true` context to assert their components
   handle the embedded mode correctly. A `pub(crate)` setter would gate that test pattern
   behind a test-only escape hatch.
3. **Misuse mode is self-diagnosing.** A consumer who flips `chrome_owned = true` without
   a parent actually drawing chrome will see a missing border in their captured output —
   obvious wrong, not subtly wrong. The risk is real but visible.

### `PaneLayout::view_with` (`src/component/pane_layout/mod.rs`)

Replace the three-step consumer-side dance with a single call that hands the consumer each
pane's inner rect via a closure:

```rust
impl PaneLayout {
    /// Render pane chrome and invoke `render_child` once per pane with the
    /// chrome-inset `RenderContext`.
    ///
    /// `render_child` receives the pane's id (string) and a child
    /// `RenderContext` whose:
    /// - `area` is the pane's inner rect (chrome already accounted for)
    /// - `chrome_owned` is `true` (children suppress their own chrome)
    /// - `focused` is `true` only for the focused pane's child context
    /// - other fields propagated from the parent context
    pub fn view_with<F>(
        state: &PaneLayoutState,
        ctx: &mut RenderContext<'_, '_>,
        mut render_child: F,
    )
    where
        F: FnMut(&PaneId, &mut RenderContext<'_, '_>),
    {
        // 1. Compute outer rects via state.layout(ctx.area) — existing logic
        // 2. For each pane: draw chrome (border with focused/disabled style + title) — existing logic
        // 3. For each pane: compute inner rect (inset by chrome thickness; today Margin{1,1})
        // 4. For each pane: construct child_ctx via ctx.with_area(inner) and set
        //    chrome_owned = true; set focused = (pane is focused) && ctx.focused
        // 5. Invoke render_child(&pane.id, &mut child_ctx)
    }
}
```

`PaneId` is the existing string identifier already used by `PaneConfig::new(id)`. The exact
type (likely `String` or `CompactString`) is whatever envision's current source uses; the
spec adopts it as-is.

#### Why closure parameter shape `FnMut(&PaneId, &mut RenderContext<'_, '_>)`

Matches envision's existing `RenderContext::with_area` reborrow pattern. The closure body
gets ergonomic access to `frame`, `theme`, and propagated state (`focused`, `disabled`,
`chrome_owned`) without re-constructing the context. If a higher-ranked trait bound
(`for<'frame, 'buf> FnMut(&PaneId, &mut RenderContext<'frame, 'buf>)`) is required by
borrowck for the implementation, that's an acceptable equivalent — implementer's choice
based on what compiles cleanly. The plain `<'_, '_>` form is preferred for its lighter
syntactic load on the closure caller.

#### Pane identity

Closures receive `&PaneId` (string-typed), not `usize`. String identifiers survive pane
reordering and are self-documenting in the `match` body — both are readability wins over
the current `rects[0]`/`rects[1]` index coupling consumers do today.

### `PaneLayout::view` deletion

`PaneLayout::view` is **deleted** in the same atomic commit as `view_with` lands. envision
is pre-1.0; coexisting `view` and `view_with` would be permanent confusion. The "draw
chrome but no children" degenerate case is `view_with(state, ctx, |_, _| {})`.

`PaneLayout::Component::view` (the trait method, not the inherent method) needs to be
addressed: today the `Component for PaneLayout` impl provides `fn view(state, ctx)` that
draws only chrome. The atomic switch either:

- (a) keeps the trait `view` impl drawing chrome only (degenerate case), with `view_with`
  as a sibling inherent method on `PaneLayout`; consumers always call `view_with` directly.
- (b) removes the `Component` impl for `PaneLayout` entirely if the only sensible render
  path is `view_with`; consumers can't accidentally fall back to chrome-only.

Spec adopts **(a)**: keep the trait impl as the chrome-only path. Existing `Component`-trait
machinery (focus management, event routing) doesn't depend on `view_with`; pulling
`PaneLayout` out of the trait would cascade through `FocusManager`, harness wiring, and
component-registry code for marginal gain. The trait `view` body's behavior is "draw chrome,
don't render children." `view_with` is the inherent method consumers reach for.

This means consumer migration is **single-method-name**: every `PaneLayout::view(state, ctx)`
call becomes `PaneLayout::view_with(state, ctx, |_, _| {})` (degenerate) or
`PaneLayout::view_with(state, ctx, |id, child_ctx| match id.as_str() { ... })` (the typical
shape).

### Component-side changes

Each component that draws chrome consults `ctx.chrome_owned` and skips its inner Block /
border / title when true:

```rust
// src/component/table/render.rs
pub fn view(state: &TableState<R>, ctx: &mut RenderContext<'_, '_>) {
    if !ctx.chrome_owned {
        // Existing block-drawing path: borders, title, focus ring
    }
    // Data rendering proceeds against ctx.area regardless
}

// src/component/styled_text/mod.rs
pub fn view(state: &StyledTextState, ctx: &mut RenderContext<'_, '_>) {
    let show_border = !ctx.chrome_owned && state.show_border();
    // ... existing path with computed `show_border` ...
}
```

`StyledText::with_show_border(false)` **stays** as the explicit standalone-no-border opt-out.
Both compose: `chrome_owned = true` is the implicit embedded path; `show_border() = false`
is the explicit standalone-no-border path. The render check is the AND of both.

#### Audit scope (in-scope, not deferred)

Chrome-drawing components in envision's tree must be audited and patched in the same PR.
Initial inventory (to be confirmed during implementation):

- **Table** (`src/component/table/`) — confirmed double-render in leadline embeds
- **StyledText** (`src/component/styled_text/`) — confirmed; `with_show_border(false)` workaround
- **StatusBar** (`src/component/status_bar/`) — likely; uncommonly embedded
- **KeyHints** (`src/component/key_hints/`) — possible; hint panes sometimes embed
- **LogViewer** (`src/component/log_viewer/`) — possible
- **MarkdownRenderer** (`src/component/markdown_renderer/`) — possible
- **DiffViewer** / **CodeBlock** / others — audit during plan-writing

The audit pattern: any component whose `view` calls `Block::default().borders(...)` or
similar "draw a frame" widget call is a candidate. The plan will enumerate the final list
and apply the consult-and-skip uniformly. Components that don't draw their own chrome
(`Spinner`, `Sparkline`, `Gauge`, etc.) are unchanged.

leadline does not currently embed StatusBar/KeyHints/etc. inside PaneLayout, so behavior
isn't verified against a real call site. The audit is preventive: a future consumer
embedding any chrome-drawing component should not re-hit this gap.

---

## Migration

### Consumer migration (the leadline pattern)

```rust
// Before — three steps + Margin{1,1} + double-rendered borders
fn render_roster(frame: &mut Frame, area: Rect, state: &State, theme: &Theme) {
    let panes = vec![PaneConfig::new("roster").with_title("Roster").with_proportion(1.0)];
    let layout_state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
    let rects = layout_state.layout(area);
    PaneLayout::view(&layout_state, &mut RenderContext::new(frame, area, theme));
    let inset = Margin { vertical: 1, horizontal: 1 };
    let inner = rects[0].inner(inset);
    Table::<RosterRow>::view(&state.roster, &mut RenderContext::new(frame, inner, theme));
}

// After — one closure, no Margin{1,1}, no double-render
fn render_roster(frame: &mut Frame, area: Rect, state: &State, theme: &Theme) {
    let panes = vec![PaneConfig::new("roster").with_title("Roster").with_proportion(1.0)];
    let layout_state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
    PaneLayout::view_with(
        &layout_state,
        &mut RenderContext::new(frame, area, theme),
        |pane_id, child_ctx| match pane_id.as_str() {
            "roster" => Table::<RosterRow>::view(&state.roster, child_ctx),
            _ => {}
        },
    );
}
```

### Migration table

| Old | New |
|---|---|
| `let rects = layout_state.layout(area); PaneLayout::view(&layout_state, ctx); let inner = rects[i].inner(Margin{1,1}); Child::view(&state, &mut RenderContext::new(frame, inner, theme));` | `PaneLayout::view_with(&layout_state, ctx, \|id, child_ctx\| match id.as_str() { "child" => Child::view(&state, child_ctx), _ => {} });` |
| `Table::view(&state, &mut RenderContext::new(frame, inner, theme))` (always inner border) | `Table::view(&state, child_ctx)` — `chrome_owned = true` propagated; inner border skipped |
| `StyledText::view(&state.with_show_border(false), ctx)` (consumer-side suppression) | `StyledText::view(&state, child_ctx)` — `chrome_owned = true` propagated; border skipped. `with_show_border(false)` stays for standalone-no-border case |
| `PaneLayout::view(state, ctx)` (chrome-only call, no embedded children) | `PaneLayout::view_with(state, ctx, \|_, _\| {})` |
| Hardcoded `Margin { vertical: 1, horizontal: 1 }` knowledge | Deleted; envision computes inner rects in one place |

Per project rule: pre-1.0, ruthless API ripping. The `Component::view` trait impl on
`PaneLayout` keeps its chrome-only semantics; the inherent `view` method goes away.

### Internal envision migration

- All in-tree `PaneLayout::view(...)` call sites in `examples/`, `tests/`, `src/`,
  doctests — migrate to `view_with`.
- `examples/pane_layout.rs` is the canonical demonstration; rewrite to use the closure
  shape end-to-end.
- Doctests in `src/component/pane_layout/mod.rs` and any cross-references — update.

---

## Files to touch

| File | Change |
|---|---|
| `src/component/context.rs` | Add `chrome_owned: bool` field, default in `new`, builder, `with_area` propagation. Update tests to assert default `false` and propagation. |
| `src/component/pane_layout/mod.rs` | Add inherent `pub fn view_with<F>(...)` on `PaneLayout`. Keep the `Component::view` trait impl as the chrome-only path. Remove any inherent `pub fn view` that shadows the trait method. |
| `src/component/pane_layout/tests.rs` | New tests: per-pane invocation, inner rect inset, `chrome_owned = true` on child_ctx, focus propagation to focused pane only, degenerate-closure regression. |
| `src/component/pane_layout/snapshots/` | Add render-snapshot fixtures for the embedded-Table case and the degenerate-closure case. |
| `src/component/table/render.rs` (or wherever the inner Block is drawn) | Skip block when `ctx.chrome_owned`. |
| `src/component/table/snapshots/` (or sibling test file) | Add render snapshots for `chrome_owned=false` (existing behavior, regression) and `chrome_owned=true` (no inner border). |
| `src/component/styled_text/mod.rs` | Compute `show_border = !ctx.chrome_owned && state.show_border()`. |
| `src/component/styled_text/snapshots/` | Add snapshot for `chrome_owned=true` skipping the border even when `show_border()` is true. |
| Other chrome-drawing components | Audit per-spec list; apply the same consult-and-skip pattern. Snapshot tests for each. |
| `examples/pane_layout.rs` | Rewrite as canonical `view_with` demonstration. |
| Other examples that use `PaneLayout::view` | Migrate to `view_with`. |
| Doctests across `src/component/` | Migrate any `PaneLayout::view` references. |
| `CHANGELOG.md` | Breaking-change entry: `PaneLayout::view` (inherent) removed, `view_with` added, `RenderContext::chrome_owned` added; migration table. |

---

## Tests

The plan will land all of the following. Test names are illustrative; the plan-writer can
rename for clarity.

### `RenderContext` (in `src/component/context.rs::tests`)

1. **`test_chrome_owned_defaults_false`** — `RenderContext::new(...)` produces
   `chrome_owned == false`. Pin the convention.
2. **`test_chrome_owned_builder_sets_field`** — `RenderContext::new(...).chrome_owned(true)`
   produces `chrome_owned == true`. Round-trip the builder.
3. **`test_with_area_propagates_chrome_owned`** — calling `ctx.with_area(rect)` on a
   context with `chrome_owned = true` produces a child context with `chrome_owned = true`.

### `PaneLayout::view_with` (in `src/component/pane_layout/tests.rs`)

4. **`test_view_with_invokes_render_child_per_pane`** — counter-based test; closure runs
   exactly N times for N panes, in declaration order.
5. **`test_view_with_passes_inner_rect`** — assert `child_ctx.area` is inset from the
   pane's outer rect by the chrome thickness.
6. **`test_view_with_sets_chrome_owned_true`** — closure receives a child_ctx with
   `chrome_owned == true`. Pin the contract.
7. **`test_view_with_focus_propagates_to_focused_pane_only`** — when parent ctx is focused,
   only the focused pane's child_ctx has `focused = true`; others are `false`.
8. **`test_view_with_disabled_propagates`** — when parent ctx is disabled, every child_ctx
   inherits `disabled = true`.
9. **`test_view_with_degenerate_closure_matches_chrome_only`** — `view_with(state, ctx, |_, _| {})`
   produces the same captured buffer as the old `view(state, ctx)` (regression test for
   the chrome-only case).

### Component snapshots

10. **`test_table_renders_no_inner_border_when_chrome_owned`** — render snapshot.
11. **`test_table_still_renders_inner_border_when_not_chrome_owned`** — render snapshot
    (regression; existing behavior preserved in standalone case).
12. **`test_styled_text_skips_border_when_chrome_owned`** — render snapshot, even when
    `state.show_border() == true`.
13. **`test_styled_text_still_renders_border_when_not_chrome_owned`** — render snapshot
    (regression).
14. **Per audited component**: a similar pair of snapshots — `chrome_owned=false`
    preserves existing render, `chrome_owned=true` suppresses chrome.

### Integration

15. **`test_pane_layout_with_embedded_table_no_double_render`** — top-level snapshot:
    construct a `PaneLayout` with a single pane wrapping a `Table`, render via
    `view_with`, assert the captured buffer has only one set of borders (the
    `PaneLayout`'s rounded outer block) — no `┌──┐` inner box.

---

## Risks & open questions

### Risks

- **Audit completeness.** The set of "chrome-drawing components" is enumerated
  best-effort in this spec; the plan-writer must complete the audit. A missed component
  surfaces as a re-opened gap from a future consumer. Mitigation: grep for `Block::default()`
  / `borders(Borders::ALL)` / `border_style(...)` patterns across `src/component/`.
- **Snapshot churn.** Components that gain a `chrome_owned`-aware path may need refreshed
  snapshots even for the `chrome_owned=false` case, if the refactor incidentally changes
  rendering order. Mitigation: hold the existing snapshots green by changing only the
  branch boundary (`if !ctx.chrome_owned { ... }` wrapping the existing code unchanged).
- **`Component::view` trait impl on `PaneLayout` keeps its chrome-only semantics.** Any
  framework code (FocusManager, harness, registry) that calls
  `<PaneLayout as Component>::view(state, ctx)` will get chrome only — children won't
  render. Audit for in-tree call sites.
- **Closure HRTB ergonomics.** The simpler `FnMut(&PaneId, &mut RenderContext<'_, '_>)`
  bound is preferred. If borrowck rejects it (e.g. because `with_area`'s reborrow forces
  HRTB), fall back to `for<'frame, 'buf> FnMut(...)`. Implementer's choice.

### Decisions resolved during brainstorming

| Question | Resolution |
|---|---|
| Closure signature shape | `FnMut(&PaneId, &mut RenderContext<'_, '_>)` if borrowck accepts; HRTB fallback acceptable |
| `chrome_owned` granularity | `bool` (YAGNI; evolve to enum if a real case forces it) |
| Backward compat for `PaneLayout::view` (inherent method) | Delete in same PR |
| Pane identity in closure | `&PaneId` (string) — reordering robustness wins |
| `Component::view` trait impl on `PaneLayout` | Keep as the chrome-only path |
| Visibility of `chrome_owned` field | Public field + public builder, matching `focused` / `disabled` convention |
| Default of `chrome_owned` in `RenderContext::new` | `false`, explicit in `new` body |
| `with_area` propagation of `chrome_owned` | Yes (matches `focused` / `disabled` propagation) |
| Audit other chrome-drawing components in same PR | In-scope, not deferred |
| `Router` interaction | Out of scope (Router is between-frame, chrome is within-frame) |
| `with_borders(Borders)` standalone disable | Skip — no leadline use case; any future need is a separate spec |
| `with_show_border(false)` deprecation | Keep as standalone-no-border opt-out; composes with `chrome_owned` |

---

## Cadence

Same 4-PR cadence as G7 / D1:

1. **PR α** — this design spec (`docs/superpowers/specs/2026-05-02-chrome-ownership-design.md`).
2. **PR β** — implementation plan (`docs/superpowers/plans/2026-05-02-chrome-ownership.md`).
3. **PR γ** — implementation. Single atomic breaking-change PR. `chrome_owned` field
   addition is additive, but `view` deletion + chrome-component consult-and-skip migration
   is a single coherent commit (potentially with additive precursors for the field, per the
   plan).
4. **Tracking-doc PR** — mark G2 + D2 + D11 ✅ resolved in
   `docs/customer-feedback/2026-05-01-leadline-gaps.md`.

---

## Related context

- leadline's customer-feedback inventory: `docs/customer-feedback/2026-05-01-leadline-gaps.md`
  (G2, D2, D11)
- leadline-side gaps tracking:
  `~/workspace/ryanoneill/rust-ai-explorations/notes/envision_gaps.md`
- Prior atomic-migration playbooks:
  - G1 + G3 + G7 spec (`docs/superpowers/specs/2026-05-02-table-sort-cell-unification-design.md`),
    plan, implementation PR #461 (`235bcae`)
  - D1 spec (`docs/superpowers/specs/2026-05-02-app-init-args-design.md`), plan,
    implementation PR #465 (`82a9a41`)
- leadline render call sites this redesign simplifies:
  `leadline/src/app.rs:316–332` (`render_roster`), `leadline/src/app.rs:336–384`
  (`render_per_op`)

This is the second of the four high-leverage open D-gaps (D1 already shipped). After this,
D5 (styled-line primitive) and D7 (snapshot testing docs) remain.
