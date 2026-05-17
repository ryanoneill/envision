# StyledText DX: line primitive + `paragraph` rename — design spec

**Date:** 2026-05-09
**Status:** Approved; implementation plan to follow.
**Tracks:** leadline gaps **D5** + **D14** in `docs/customer-feedback/2026-05-01-leadline-gaps.md`
**Source brief:** `~/workspace/ryanoneill/rust-ai-explorations/notes/envision_styled_text_dx_redesign.md` (327 lines)
**Builds on:** `docs/superpowers/specs/2026-05-02-chrome-ownership-design.md` (chrome_owned propagation)

---

## TL;DR

Two small DX gaps in the same `StyledText` / `StyledContent` surface area, bundled
into one design pass:

- **D5** — drawing one styled line takes six types and three method calls
  (`StyledTextState::new().with_content(StyledContent::new().line(...)).with_show_border(false)`
  + `StyledText::view(&state, &mut ctx)`). The capability is one `Paragraph::new(line).render(...)`
  internally; envision should expose it.
- **D14** — `StyledContent::paragraph(inlines)` produces a *single line*, not a wrapped
  block-level paragraph. The name is misleading; every new contributor pays a half-hour
  source-spelunking tax.

Add `envision::render::styled_line(frame, area, inlines: &[StyledInline], theme)` —
a free-function primitive that renders one styled line into the given area. Rename
`StyledContent::paragraph(...)` → `StyledContent::line(...)` AND rename the
underlying `StyledBlock::Paragraph(Vec<StyledInline>)` variant → `StyledBlock::Line(Vec<StyledInline>)`
to match. Delete the old method/variant outright (pre-1.0 ruthlessness; same-PR
mechanical migration). Free the `paragraph` name for future real block-level
paragraph behavior — that semantics lands when a consumer needs it.

The `envision::render::styled_line` primitive composes naturally with the
`chrome_owned` propagation protocol from PR #469 — it's a no-chrome render path
by definition (single line, no border), so it auto-suppresses chrome when called
from inside a `chrome_owned: true` context.

---

## Goals

1. **One-call line rendering.** `envision::render::styled_line(frame, area, &inlines, theme)`
   replaces the six-types-three-methods construction pattern. Consumer-side line count
   for a styled summary banner drops from ~6 lines to 1.
2. **Method name matches behavior.** `StyledContent::line(inlines)` does what consumers
   expect from a method named "line." The misleading `paragraph(...)` method goes away.
3. **Source-level coherence.** The internal `StyledBlock::Line(Vec<StyledInline>)` variant
   matches the method that pushes it; future maintainers and contributors don't hit the
   `StyledBlock::Paragraph` misnomer when source-spelunking.
4. **Free the `paragraph` name.** Future real block-level paragraph behavior (wrapped
   text, trailing blank line per conventional paragraph semantics) lands as a separate
   PR when a consumer needs it. The name is reserved.
5. **Composes with chrome_owned.** The new primitive participates in the chrome ownership
   protocol from PR #469 without per-component logic — single-line rendering owns no
   chrome by construction.

## Non-goals

- **Real block-level `paragraph(...)` method.** Deferred until a consumer needs wrapped
  block text. The rename frees the name; new semantics land in a follow-up PR.
- **`render::line(frame, area, &ratatui::Line, theme)` for raw `Line` input.** Optional
  per the brief; no consumer needs it today. envision avoids extending raw ratatui types
  into the public surface (the renderer might not be ratatui forever). YAGNI.
- **Dedicated `render::empty_state` primitive.** Centered placeholder text + theme
  styling for empty states. leadline's empty-state path is one styled line — the new
  `styled_line` primitive handles it. Adding a dedicated empty-state primitive is
  over-spec for a single consumer pattern.
- **Deprecation period for the rename.** envision is pre-1.0; ruthlessness preferred.
  `paragraph` method + `StyledBlock::Paragraph` variant delete outright in the same PR
  that adds `line()` + `StyledBlock::Line`. Mechanical migration across all 17 call
  sites in envision + examples.
- **Component-form `StyledLine` (Shape B from the brief).** Free function over component.
  Lower API surface; idiomatic for stateless primitives; matches existing
  `envision::scroll::render_scrollbar` pattern.
- **Renaming `StyledContent::text(text)`.** The convenience wrapper at line 176 keeps
  its name (it pushes a styled line internally; the public name `text` is fine —
  consumers reading `content.text("hello")` aren't surprised by behavior).

---

## Design

### `envision::render` module — new top-level utility namespace

Create a new top-level module `src/render.rs` (or `src/render/mod.rs` if it grows).
Houses standalone render primitives that don't fit any specific component module.
First inhabitant is `styled_line`; future primitives can land here without polluting
component modules.

Why a new top-level module: envision already has the pattern of utility namespaces
with helper functions exported at the crate level — `envision::scroll::render_scrollbar`,
`envision::scroll::render_scrollbar_inside_border`. The `render` module follows the
same convention. Discoverable: `envision::render::*` is where consumers look for
"render this thing here" primitives.

```rust
//! Standalone render primitives that don't belong to a single component.
//!
//! These functions take a `Frame`, a `Rect`, the data to render, and a `Theme`,
//! and produce immediate rendered output. No state; no chrome; no border. Compose
//! freely with the chrome-ownership protocol — primitives consume an externally-
//! owned area and don't draw chrome of their own.

use ratatui::layout::Rect;
use ratatui::Frame;

use crate::component::styled_text::StyledInline;
use crate::theme::Theme;

/// Render a sequence of styled inline elements as a single line into `area`.
///
/// Equivalent to a borderless `StyledText` with one line of content, but with
/// no state plumbing — pass inlines + frame + area + theme, get rendered output.
///
/// ...
pub fn styled_line(
    frame: &mut Frame,
    area: Rect,
    inlines: &[StyledInline],
    theme: &Theme,
) {
    // Implementation lifts the existing borderless-StyledText render path.
}
```

Re-export at the crate root: `pub use render::styled_line;` in `src/lib.rs` so
consumers can write either `envision::render::styled_line(...)` or
`envision::styled_line(...)`. (Match the existing `envision::render_scrollbar`
convention from `scroll`.)

### `envision::render::styled_line` — function signature

```rust
pub fn styled_line(
    frame: &mut Frame,
    area: Rect,
    inlines: &[StyledInline],
    theme: &Theme,
) {
    // ... renders one line of styled inlines into area, themed.
}
```

**Why `&[StyledInline]` not `Vec<StyledInline>`:** slice signature is more flexible.
Consumers can pass `&inlines` (where `inlines: Vec<StyledInline>`) or a literal
slice `&[StyledInline::Plain("hello".into())]`. `Vec` would force consumers to own
or clone — extra allocation for what is a read-only render-time pass.

**Why no `RenderContext` parameter:** the primitive is below the `Component` trait
layer. `RenderContext` carries state (focused, disabled, chrome_owned) that doesn't
apply to a single-line render: there's no chrome to own, no focus border to draw,
no disabled-state styling distinct from theme styling. The `theme` is the only
piece of `RenderContext` the primitive needs.

If a consumer wants disabled-style rendering, they pass theme-disabled styles in
the inlines themselves (or use `StyledInline::Colored { color: theme.disabled, .. }`).
This matches the existing `Cell` / `CellStyle` pattern where styling lives in the
data.

**Internal implementation:** lifts the existing borderless-StyledText render path
in `src/component/styled_text/mod.rs`. The render path already exists for the
`with_show_border(false)` case — extracting it to a standalone function is purely
ergonomic, no rendering changes.

### `StyledContent::line(inlines: Vec<StyledInline>)` — method rename

In `src/component/styled_text/content.rs`, rename `StyledContent::paragraph(...)`
to `StyledContent::line(...)`. Body unchanged (just the variant pushed updates per
the next section).

```rust
impl StyledContent {
    /// Append a single styled line composed of inline elements.
    ///
    /// (Pre-rename name was `paragraph(...)` — but the method produced one
    /// line, not a block-level paragraph. Renamed for clarity. The
    /// `paragraph(...)` name is reserved for future block-level wrapped text.)
    pub fn line(mut self, inlines: Vec<StyledInline>) -> Self {
        self.blocks.push(StyledBlock::Line(inlines));
        self
    }
}
```

Delete the old `paragraph(...)` method. No `#[deprecated]` shim — pre-1.0; one
mechanical migration pass over all call sites in the same PR.

`StyledContent::text(text: impl Into<String>)` (the convenience wrapper at content.rs:176)
keeps its public name. Internally it now delegates to `line(...)` instead of
`paragraph(...)`.

### `StyledBlock::Line(Vec<StyledInline>)` — variant rename

In `src/component/styled_text/content.rs`, rename the `StyledBlock::Paragraph(Vec<StyledInline>)`
variant to `StyledBlock::Line(Vec<StyledInline>)`. Update every match arm in the
crate (render path, tests, doc examples).

```rust
pub enum StyledBlock {
    Heading { level: u8, text: String },
    Line(Vec<StyledInline>),  // was: Paragraph(Vec<StyledInline>)
    BulletList(Vec<Vec<StyledInline>>),
    NumberedList(Vec<Vec<StyledInline>>),
    CodeBlock { language: Option<String>, source: String },
    HorizontalRule,
    BlankLine,
    Raw(Vec<String>),
}
```

The internal render arm in `src/component/styled_text/render.rs` (or wherever the
match lives — verify during plan-writing) becomes:

```rust
StyledBlock::Line(inlines) => render_line(inlines, theme, ...),
```

(Was: `StyledBlock::Paragraph(inlines) => render_line(inlines, theme, ...)` — the
function it dispatches to was already named `render_line` or similar; the variant
rename eliminates the inconsistency.)

This rename is the load-bearing internal-coherence change. Combined with the
method rename it produces a single source-level name (`Line`) for the single-line
concept, freeing `Paragraph` for future block-level semantics.

### Migration: 17 call sites

`grep -rn "\.paragraph(\|StyledBlock::Paragraph" src examples 2>&1 | wc -l` shows
17 sites. Breakdown (verify during plan-writing):

- `src/component/styled_text/content.rs` — variant definition + builder method
- `src/component/styled_text/mod.rs` — render-path matches + doc examples
- `src/component/styled_text/tests.rs` — test fixtures
- `examples/styling_showcase.rs` — heaviest concentration (~10 sites; the example
  exercises every block variant)

Mechanical migration:
- `\.paragraph(` → `\.line(` (method calls)
- `StyledBlock::Paragraph(` → `StyledBlock::Line(` (variant constructions + matches)

No semantic changes; no test fixture changes; no rendered-output changes (the
underlying render path is unchanged — only the name on the variant and the method
that pushes it).

---

## Migration

### Consumer-side: `build_summary_banner_state` collapses

leadline's `app.rs:392` (the per-op summary banner) currently:

```rust
fn build_summary_banner_state(row: Option<&RosterRow>) -> StyledTextState {
    let inlines = build_summary_inlines(row);
    StyledTextState::new()
        .with_content(StyledContent::new().paragraph(inlines))
        .with_show_border(false)
}

// And at the call site:
let summary = build_summary_banner_state(row);
StyledText::view(&summary, &mut RenderContext::new(frame, area, theme));
```

After D5+D14:

```rust
fn render_summary_banner(frame: &mut Frame, area: Rect, row: Option<&RosterRow>, theme: &Theme) {
    let inlines = build_summary_inlines(row);
    envision::render::styled_line(frame, area, &inlines, theme);
}
```

State plumbing (`StyledTextState::new()`, `StyledContent::new()`, `with_show_border`)
all gone. Six-line construction → two lines. The render call moves into the function;
the consumer no longer needs to pair "build state" with "view state."

The empty-state path (`app.rs:372`) collapses similarly — it's another single-styled-line
case where the placeholder text becomes inlines passed to `styled_line`.

### Migration table

| Old | New |
|---|---|
| `StyledContent::new().paragraph(inlines)` | `StyledContent::new().line(inlines)` |
| `StyledBlock::Paragraph(inlines)` | `StyledBlock::Line(inlines)` |
| `StyledTextState::new().with_content(StyledContent::new().line(inlines)).with_show_border(false)` + `StyledText::view(...)` | `envision::render::styled_line(frame, area, &inlines, theme)` |
| `match block { StyledBlock::Paragraph(inlines) => ... }` | `match block { StyledBlock::Line(inlines) => ... }` |

### Internal envision migration

- `src/component/styled_text/content.rs` — variant + method rename, internal `text()`
  delegate updated.
- `src/component/styled_text/mod.rs` — render-path match arm renamed; any `view()`
  examples in module docs updated.
- `src/component/styled_text/tests.rs` — test fixtures + assertions updated; insta
  snapshots stay unchanged (the rendered output is identical).
- `examples/styling_showcase.rs` — mechanical `.paragraph(` → `.line(` across ~10 sites.
- `src/lib.rs` — re-export `pub use render::styled_line;` at crate root.
- `src/render.rs` — new file housing `pub fn styled_line(...)`.

---

## Files to touch

| File | Change |
|---|---|
| `src/render.rs` (NEW) | `pub mod render;` housing `pub fn styled_line(frame, area, &[StyledInline], theme)`. ~50 lines. |
| `src/lib.rs` | Add `pub mod render;` and `pub use render::styled_line;` re-export at crate root. |
| `src/component/styled_text/content.rs` | Rename `StyledBlock::Paragraph` → `StyledBlock::Line`; rename `StyledContent::paragraph(...)` → `StyledContent::line(...)`; delete old; update `text()` delegate. |
| `src/component/styled_text/mod.rs` | Update render-path match arm; update module-level doc examples. |
| `src/component/styled_text/tests.rs` | Update test fixtures and assertions for the variant rename; verify snapshots unchanged. |
| `src/component/styled_text/tests.rs` (or new test file) | Add tests for `envision::render::styled_line` — render-into-area, theme-driven coloring, slice-input form. |
| `examples/styling_showcase.rs` | Mechanical `.paragraph(` → `.line(` (~10 sites). |
| `CHANGELOG.md` | Additive entry under `[Unreleased]` documenting the new primitive + the rename + the internal variant rename. |

---

## Tests

### Render-path tests for `envision::render::styled_line`

Add to `src/render.rs` inline `#[cfg(test)] mod tests`, or to a new
`src/render/tests.rs` if the file grows. Five named tests:

1. **`test_styled_line_renders_inlines`** — pass a vec of three inlines (Plain, Bold,
   Colored), render into a 40×1 area, snapshot the plain-text output. Pin that the
   text appears in order.

2. **`test_styled_line_applies_theme_color`** — pass `StyledInline::Colored { color: Color::Red, text: "err" }`,
   render to area, assert ANSI output contains `\x1b[31m`. Pins theme-driven color
   resolution.

3. **`test_styled_line_truncates_to_area_width`** — pass inlines totaling 60 chars,
   render into a 20-char-wide area, snapshot. Pin truncation behavior matches
   ratatui's `Paragraph` default.

4. **`test_styled_line_empty_inlines_renders_nothing`** — pass `&[]`, render to a
   40×1 area, snapshot empty buffer. Pin no-input behavior is well-defined.

5. **`test_styled_line_no_chrome_drawn`** — render to a 40×3 area; assert that rows
   above/below the rendered line stay blank. Pins that the primitive draws no
   border / chrome / fill — just one line of styled content into the first row.

### Variant-rename regression tests (existing; update)

The existing `styled_text/tests.rs` test fixtures use `StyledBlock::Paragraph(...)`
and `.paragraph(...)`. After the rename, all references update mechanically. Pin
that:

- Rendered output (insta snapshots) is byte-identical pre/post rename. The variant
  rename is internal-only; rendered text doesn't change.
- The `text()` convenience wrapper still produces a single styled line.

### Documentation examples

All `///` doc examples on `StyledContent::line(...)`, `StyledBlock::Line`, and
`envision::render::styled_line(...)` use active `rust` doc-test fences (no
`,ignore`) — the API is complete on landing, so examples compile and run.

---

## Risks & open questions

### Risks

- **17 call sites mechanical migration.** Risk: missed call site → compile error.
  Mitigation: `grep -c "paragraph\|Paragraph" src examples` before and after the
  migration confirms zero remaining references to the old names. CI catches any
  miss across all platforms.
- **Insta snapshot churn.** The variant rename changes serialized debug output of
  `StyledBlock`. If any insta snapshot serializes `StyledBlock` (rather than
  rendered text), the snapshot diffs. Mitigation: audit existing snapshots during
  implementation; either accept the diff (debug output is internal) or use
  `redactions` to mask the variant name. Most snapshots in the codebase are
  rendered-text snapshots, not Debug-formatted struct snapshots.
- **Public API breakage scope.** `StyledBlock::Paragraph` is `pub` — external
  consumers can match on it. The rename is breaking. Mitigation: this is exactly
  what `[Unreleased]` CHANGELOG entries flag for. envision is pre-1.0; consumers
  expect breaking changes between minor versions. The 4-PR cadence's tracking-doc
  PR documents the migration footprint.

### Decisions resolved during brainstorming

| Question | Resolution |
|---|---|
| Shape A vs Shape B for D5 (Q1) | Shape A — free function `envision::render::styled_line(frame, area, &inlines, theme)`. Lower API surface; idiomatic for stateless primitives; matches existing `envision::scroll::render_scrollbar` pattern. |
| Module location for the primitive (Q2) | New top-level `src/render.rs` (`envision::render`). Discoverable; consistent with existing utility namespaces (`scroll`); not polluting any single component module. Re-export at crate root: `envision::styled_line` works too. |
| Empty-state primitive (Q3) | Out of scope — `styled_line` is sufficient. A dedicated `render::empty_state` is over-spec for one consumer pattern. |
| Future block-level `paragraph()` method (Q4) | Deferred. The rename frees the name; real wrapped-block-paragraph semantics land when a consumer needs them. |
| Slice vs Vec for inlines parameter (Q5) | `&[StyledInline]` — more flexible; no allocation churn at the call boundary. |
| Deprecate vs delete `paragraph(...)` method (Q6) | Delete outright. Pre-1.0; ruthlessness preferred; same-PR mechanical migration across all 17 call sites. |
| `StyledBlock::Paragraph` variant rename | Rename to `StyledBlock::Line` in same PR. Coherence with method rename; frees `StyledBlock::Paragraph` for future real-paragraph variant. Half-fix would leave the same name-vs-meaning gap from the source-spelunking angle. |
| `StyledContent::text(text)` convenience wrapper | Keep name. Public-facing `text()` is fine; only internal delegate updates from `paragraph(...)` to `line(...)`. |
| Optional `render::line(frame, area, &ratatui::Line, theme)` for raw Line input | Skip (YAGNI). No consumer needs it; envision avoids extending raw ratatui types into the public surface. |

---

## Cadence

Same 4-PR cadence as G7 / D1 / chrome-ownership / D6+D9 / D15:

1. **PR α** — this design spec (`docs/superpowers/specs/2026-05-09-styled-text-dx-design.md`).
2. **PR β** — implementation plan (`docs/superpowers/plans/2026-05-09-styled-text-dx.md`).
3. **PR γ** — implementation. Single coherent breaking-change PR (variant rename + method
   rename + new primitive bundled). Mechanical migration across all 17 call sites.
4. **Tracking-doc PR** — mark D5 + D14 ✅ resolved in
   `docs/customer-feedback/2026-05-01-leadline-gaps.md`.

Flag leadline at spec-PR open for review.

---

## Related context

- leadline's customer-feedback inventory: `docs/customer-feedback/2026-05-01-leadline-gaps.md` (D5, D14)
- leadline-side gaps tracking: `~/workspace/ryanoneill/rust-ai-explorations/notes/envision_gaps.md`
- Source brief: `~/workspace/ryanoneill/rust-ai-explorations/notes/envision_styled_text_dx_redesign.md` (327 lines)
- Prior atomic-migration playbooks (5 shipped):
  - G1 + G3 + G7 (PRs #459 / #460 / #461 / #458)
  - D1 (PRs #463 / #464 / #465 / #466)
  - G2 + D2 + D11 (PRs #467 / #468 / #469 / #470)
  - D6 + D9 (PRs #471 / #472 / #473 / #474)
  - D15 (PRs #476 / #477 / #478 / #479)
- leadline call sites this redesign simplifies:
  - `leadline/src/app.rs:372` — empty-state styled line
  - `leadline/src/app.rs:392–442` — `build_summary_banner_state`
  - All `.paragraph(...)` call sites — mechanical rename to `.line(...)`
- Related envision specs:
  - `docs/superpowers/specs/2026-05-02-chrome-ownership-design.md` — chrome_owned propagation
    that the new primitive composes with naturally
  - Future: G6 (StyledInline composable styles) brief at
    `~/workspace/ryanoneill/rust-ai-explorations/notes/envision_styled_inline_compose_redesign.md` —
    the composable `StyledInline` form that this primitive will render. D5+D14 land first;
    G6 layers on top.

This is the sixth coherent redesign drawing from leadline's May 2026 brief suite.
After this, G4+G5 (per-component style overrides — restores user-visible severity
on the StatusBar), G6 (StyledInline composable), D7 (snapshot testing docs), the
documentation suite (D3+D8), and the small-rough-edges punch list (D10+D12+D13)
remain.
