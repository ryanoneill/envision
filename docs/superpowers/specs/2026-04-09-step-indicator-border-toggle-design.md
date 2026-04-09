# StepIndicator borderless mode — design

**Status:** approved
**Date:** 2026-04-09
**Source:** customer feedback for next release
**Scope:** single PR / feature branch

## Problem

`StepIndicator` currently renders an unconditional `Block` with `Borders::ALL`
around its steps (`src/component/step_indicator/mod.rs:648-668`). There is no
way to disable the border. Customer feedback asks for a borderless variant so
`StepIndicator` can be used as a breadcrumb — a single row of steps embedded
inline in a larger layout, with no surrounding box. In the current
implementation, `StepIndicator` cannot be rendered in a 1-row area at all
(the border consumes both rows, leaving nothing for the content).

## Goal

Add an opt-in borderless rendering mode to `StepIndicator` via a `show_border`
field on `StepIndicatorState`, defaulting to `true` so existing callers see
zero visual change.

Non-goals:
- Alternative border styles (rounded, partial, dashed, etc.). YAGNI.
- A shared "optional block" helper factored out across components. Worth
  considering after feedback item #3 (shared app-slot primitive) lands — the
  primitive layer may absorb this concern.
- Migration of `StyledTextState` or other components. Out of scope for this PR.

## Design

### API surface

Add a single `bool` field to `StepIndicatorState`, mirroring the convention
already established by `StyledTextState` (`src/component/styled_text/mod.rs:107`):

```rust
pub struct StepIndicatorState {
    steps: Vec<Step>,
    orientation: StepOrientation,
    focused_index: usize,
    show_descriptions: bool,
    title: Option<String>,
    connector: String,
    show_border: bool,  // NEW
}
```

The `Default` impl initializes `show_border: true`. Every existing caller
continues to render identically.

Three new methods on `StepIndicatorState`, matching the `StyledTextState`
surface exactly for consistency across components:

- `pub fn with_show_border(mut self, show: bool) -> Self` — builder, chains
  with the existing `with_orientation`/`with_title`/`with_connector`/
  `with_show_descriptions` methods.
- `pub fn show_border(&self) -> bool` — getter.
- `pub fn set_show_border(&mut self, show: bool)` — mutator, matching the
  existing `set_title`/`set_orientation`/`set_show_descriptions` pattern.

Each method gets a doc comment with an executable doc test, matching the
style of the existing methods on `StepIndicatorState`.

The doc comment on `with_show_border` includes a note about the title
interaction (see below):

> When `false`, the state's title is not rendered. The title is only drawn as
> part of the border block, so disabling the border silently suppresses it.
> Set the title to `None` if you want to make that explicit.

### Title behavior when borderless

**The title is silently dropped when `show_border == false`.** This matches
the existing behavior of `StyledTextState` at
`src/component/styled_text/mod.rs:468-482`: when the block isn't constructed,
the title has nowhere to live and is not rendered anywhere else.

The `title` field on `StepIndicatorState` still exists and `title()` still
returns its value — only the rendering is suppressed. Consumers can read the
title for their own purposes (e.g., annotation, logging) regardless of border
state.

This is a known footgun (caller sets a title, disables the border, and
wonders where the title went), but it is the existing convention in this
codebase. The doc comment on `with_show_border` documents the interaction.
If we ever decide to change this, it should be a cross-cutting decision that
updates both `StepIndicatorState` and `StyledTextState` in a single PR.

### Rendering

The existing `view()` at `src/component/step_indicator/mod.rs:637-682`
unconditionally constructs a block, computes `inner`, and renders the block.
The new shape wraps that in a branch on `state.show_border`:

```rust
fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, ctx: &ViewContext) {
    crate::annotation::with_registry(|reg| {
        reg.register(
            area,
            crate::annotation::Annotation::new(crate::annotation::WidgetType::StepIndicator)
                .with_id("step_indicator")
                .with_focus(ctx.focused)
                .with_disabled(ctx.disabled),
        );
    });

    let inner = if state.show_border {
        let mut block = Block::default()
            .borders(Borders::ALL)
            .border_style(if ctx.focused {
                theme.focused_border_style()
            } else {
                theme.border_style()
            });
        if let Some(title) = &state.title {
            block = block.title(format!(" {} ", title));
        }
        let inner = block.inner(area);
        frame.render_widget(block, area);
        inner
    } else {
        area
    };

    if state.steps.is_empty() {
        return;
    }

    match state.orientation {
        StepOrientation::Horizontal => render_horizontal(state, frame, inner, theme, ctx.focused),
        StepOrientation::Vertical => render_vertical(state, frame, inner, theme, ctx.focused),
    }
}
```

Notes:

- **Annotation registration is unchanged** — it runs unconditionally on the
  full `area`, as before.
- **The two branches of the current `if let Some(title)` block are collapsed
  into one.** The existing code duplicates the border construction between
  the title-set and title-unset arms. This refactor is incidental to the
  feature work and makes the new code cleaner; it is not scope creep.
- **`render_horizontal` and `render_vertical` are unchanged.** They already
  take a `Rect` and render correctly whether that rect is a block inner or
  the full widget area.
- **Focus indication when borderless:** there are two focus affordances in
  this component. The border styling is one. The other is `step_style()`
  at `src/component/step_indicator/mod.rs:685-698`, which adds
  `BOLD | UNDERLINED` to the focused step via the `is_focused_step`
  parameter. That still fires in borderless mode, so focused breadcrumbs
  retain a visible focus indicator without additional work.
- **1-row rendering:** with `show_border: false`, `StepIndicator` becomes
  usable in a 1-row area. Previously `area.height == 1` produced
  `inner.height == 0` and rendered nothing. This is the breadcrumb use case.

### Serialization

The `show_border` field is included in the existing
`#[cfg_attr(feature = "serialization", derive(serde::Serialize, serde::Deserialize))]`
derive on `StepIndicatorState`. No `#[serde(default)]` helper is added —
the library is pre-1.0 and serialized-state backwards compatibility is not
a goal. Old snapshots missing the field will fail to deserialize, which is
acceptable.

## Testing

Existing snapshot tests already cover the default (with-border) path:
`test_view_horizontal`, `test_view_vertical`, `test_view_with_title`,
`test_view_focused_step`, `test_view_vertical_descriptions`,
`test_view_all_statuses`, `test_view_empty`
(`src/component/step_indicator/tests.rs:584+`). These snapshots act as
regression coverage for the with-border path — if the refactored
rendering code changes any of those outputs, the snapshots will fail.

New tests added to `src/component/step_indicator/tests.rs`:

**State / builder unit tests:**

- `test_state_default_show_border` — `StepIndicatorState::default().show_border()` is `true`.
- `test_state_with_show_border` — `with_show_border(false)` sets the field; chaining with `with_title("...")` leaves the title stored (not dropped at the state level, only at render time).
- `test_state_set_show_border` — `set_show_border` toggles the field in place.

**Doc tests:** each of the three new methods (`with_show_border`,
`show_border`, `set_show_border`) carries an executable doc test following
the existing pattern.

**Rendering snapshot tests** (using `test_utils::setup_render` and `insta`):

- `test_view_borderless_horizontal` — 3 steps, horizontal, borderless, in a small area. Snapshot shows steps + connectors with no box-drawing characters.
- `test_view_borderless_vertical` — same, vertical orientation.
- `test_view_borderless_one_row` — the breadcrumb case: 1-row area, horizontal, borderless. Snapshot shows one line of steps. This test proves the customer's stated use case works and guards against regressions that would consume rows for a border that isn't there.
- `test_view_borderless_drops_title` — set a title *and* `show_border(false)`; snapshot must contain the step labels and must not contain the title text. Locks in the title-suppression behavior described in the "Title behavior when borderless" section above so it cannot silently regress.

**Not tested (deliberately):**

- Theme-specific border rendering — already covered by the existing theme tests.
- Keyboard focus navigation — unchanged by this feature.
- Serialization round-trip of `show_border` — pre-1.0, not a stability target.

## Risk and rollback

Risk is low. The change is a single additive field with a default-on migration
and a conditional wrapper around a single rendering block. No existing public
API is removed or renamed. Existing tests act as regression coverage.

Rollback: revert the single feature PR.
