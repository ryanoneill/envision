# `StyledInline` composable styles (G6) — design spec

**Date:** 2026-05-20
**Status:** Approved; implementation plan to follow.
**Tracks:** leadline gap **G6** in `docs/customer-feedback/2026-05-01-leadline-gaps.md`
**Source brief:** `~/workspace/ryanoneill/rust-ai-explorations/notes/envision_styled_inline_compose_redesign.md` (542 lines, re-verified 2026-05-20, commit add30b3)
**Builds on:** D5 + D14 (PR #482) `envision::render::styled_line`, G4 + G5 (PR #487) per-component style overrides, D6 + D9 (PR #473) theme palette + severity, D15 (PR #478) `CellStyle::Severity`

---

## TL;DR

`StyledInline` is a 7-variant enum where each leaf variant captures one styling
dimension at a time: `Plain | Bold | Italic | Underline | Strikethrough | Colored | Code`.
There's no way to combine dimensions — "bold red text" requires either two adjacent
inlines (`Bold + Colored`) or a new combinatorial variant like `BoldColored`. Six
dimensions yield 2^6 = 64 combinatorial variants for full coverage. Wrong shape.

Replace with a composable variant carrying a struct of optional dimensions:

```rust
#[non_exhaustive]
pub enum StyledInline {
    Plain(String),
    Code(String),
    Styled { text: String, style: InlineStyle },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct InlineStyle {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underlined: bool,
    pub strikethrough: bool,
}
```

Two-layer constructor surface: `InlineStyle::new().bold().fg(c)` builder for
style composition + `StyledInline::styled(text, style)` one-call wrapper + five
leaf-helper constructors (`bold`, `italic`, `underlined`, `strikethrough`,
`colored`) covering single-dimension cases for call-site terseness.

Top-line payoff (the brief's load-bearing motivation): the per-op summary banner
at `leadline/src/app.rs:412-455` (`build_summary_inlines`) renders five
iconnx/ort/ratio/delta/iters segments. Each `value` segment should read
**bold + severity-colored** so magnitude jumps at the user via weight contrast
in addition to severity color. Today only color survives — `Bold(text)` has no
color field, `Colored { ... }` has no bold field. Post-G6, `push_label_value`
emits `StyledInline::styled(value, InlineStyle::new().fg(value_color).bold())`
and the bold lands alongside the color.

Mechanical migration across **102 call sites in 6 files**. Five leaf variants
delete; `Plain` and `Code` stay. Old API deleted outright (pre-1.0; same pattern
as D14 `paragraph→line`, D5+D14 `StyledTextState` collapse, G5 `severity_status_style`
deletion).

---

## Goals

1. **Composable dimensions in a single inline.** `StyledInline::styled(text,
   InlineStyle::new().fg(c).bold())` renders one styled span carrying both
   color and bold. No more two-inline workaround for the bold-colored case.
2. **Constructor ergonomics unchanged for single-dimension cases.** The 80% of
   call sites that need just one dimension (`Bold(t)` / `Italic(t)` / `Colored {...}`)
   collapse to leaf-helper calls (`bold(t)` / `italic(t)` / `colored(t, c)`) —
   same call-site length, same readability.
3. **Future-proof via `#[non_exhaustive]`.** Three placements: enum, no individual
   variant, struct. New variants and new style dimensions land additively.
4. **Bold-on-banner-values payoff visible.** The per-op summary banner reads with
   weight contrast on value segments. Snapshot test ANSI-asserts both `\x1b[31m`
   (color) AND `\x1b[1m` (BOLD) appear on the same span.
5. **One coherent breaking change.** Five leaf variants delete in one PR; old
   variants don't linger as `#[deprecated]` shims. Matches every prior envision
   migration (D14, D6+D9, D5+D14, G5).

## Non-goals

- **Keeping leaf variants as `#[deprecated]` shims.** Explicitly rejected per Q-β
  + the consistent envision precedent. Pre-1.0 ruthlessness; one mechanical
  migration in the same PR.
- **Bitflags or `Vec<Modifier>` for the modifier set.** A flat struct with bool
  fields is simpler, copies cheaply (`InlineStyle` is `Copy` after `Color` is
  `Copy`), pattern-matches well, no dependency cost, and reads better at call
  sites (`InlineStyle::new().bold()` vs `InlineStyle { modifiers: BOLD | ITALIC, ... }`).
- **Newtype wrapper around `ratatui::Style`.** Leaks renderer into the envision
  public API; envision currently abstracts ratatui and reserves the ability to
  swap renderers. Rejected.
- **Separate `StyledRun` type with `StyledInline` as legacy.** Splitting types
  creates two parallel API surfaces. Composability lives at the same layer as
  the leaf variants — same type, redesigned shape is cleaner.
- **Enum-chained builder** (e.g., `StyledInline::Plain(t).bold()`). Would silently
  convert `Plain → Styled` on `.bold()`, surprising. Two-layer (struct builder
  for style, enum constructor for the pair) keeps each type's role distinct.
- **Folding `Code` into `InlineStyle`** (e.g., adding `code: bool`). `Code` is
  theme-coupled (uses `theme.info_style().add_modifier(Modifier::BOLD)` —
  consumer doesn't specify color). Conceptually distinct from style-coupled
  composition. Stays as its own variant.
- **`StyledInline::colored_bg(text, bg)` or `colored(text, fg, bg)` 3-arg form.**
  bg-only inline runs are rare; the full fg+bg case has 3 args which doesn't fit
  a clean two-arg helper signature. Both route through `styled(t, InlineStyle::new()...)`.
  Keeping the helper set at 5 (one per single dimension) is the principled size.
- **`with_style_override` or "override the underlying base_style" semantics.**
  G6 is about composing dimensions in a single inline. The render path applies
  `InlineStyle` fields on top of the `base_style` from the surrounding context
  (same way `Colored { fg, bg }` does today). No layering rules need change.

---

## Design

### `StyledInline` enum (post-G6 shape)

In `src/component/styled_text/content.rs:43-67`:

```rust
/// An inline styling element within a paragraph or list item.
///
/// `#[non_exhaustive]` so envision can add inline variants later without
/// breaking downstream `match` arms in consumer crates.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum StyledInline {
    /// Plain text, no styling.
    Plain(String),
    /// Inline code (renders with theme-coupled styling — bold info color).
    Code(String),
    /// Styled run combining color, modifiers, and optional background.
    Styled {
        /// The text content.
        text: String,
        /// Style dimensions applied on top of the surrounding base style.
        style: InlineStyle,
    },
}
```

Three variants total (down from seven). `Plain` and `Code` keep their existing
shape; the five leaf variants (`Bold`, `Italic`, `Underline`, `Strikethrough`,
`Colored`) delete and are replaced by `Styled` + helpers.

### `InlineStyle` struct (new)

```rust
/// Style dimensions for a styled inline run.
///
/// All dimensions are optional and compose freely. Use [`InlineStyle::new`]
/// + builder methods (`fg`, `bg`, `bold`, `italic`, `underlined`, `strikethrough`)
/// to construct; struct-literal construction is intentionally not supported
/// (`#[non_exhaustive]`) so future modifier additions land additively without
/// breaking consumers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct InlineStyle {
    /// Foreground color override.
    pub fg: Option<Color>,
    /// Background color override.
    pub bg: Option<Color>,
    /// Render text in bold.
    pub bold: bool,
    /// Render text in italic.
    pub italic: bool,
    /// Render text underlined (past tense — matches `ratatui::Modifier::UNDERLINED`).
    pub underlined: bool,
    /// Render text with strikethrough.
    pub strikethrough: bool,
}

impl InlineStyle {
    /// Creates an empty style (no modifiers, no colors).
    pub const fn new() -> Self {
        Self {
            fg: None,
            bg: None,
            bold: false,
            italic: false,
            underlined: false,
            strikethrough: false,
        }
    }

    /// Builder: set foreground color.
    pub const fn fg(mut self, c: Color) -> Self {
        self.fg = Some(c);
        self
    }

    /// Builder: set background color.
    pub const fn bg(mut self, c: Color) -> Self {
        self.bg = Some(c);
        self
    }

    /// Builder: enable bold.
    pub const fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    /// Builder: enable italic.
    pub const fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    /// Builder: enable underlined (past tense — matches ratatui's `Modifier::UNDERLINED`).
    pub const fn underlined(mut self) -> Self {
        self.underlined = true;
        self
    }

    /// Builder: enable strikethrough.
    pub const fn strikethrough(mut self) -> Self {
        self.strikethrough = true;
        self
    }
}
```

All builder methods are `const fn` — `InlineStyle::new().bold().fg(c)` can be
used in `const` contexts (e.g., module-level static styles).

`#[derive(Copy)]` because every field is `Copy` (`Option<Color>` is `Copy`;
`Color` is `Copy`; `bool` is `Copy`). Render path can pass `InlineStyle` by
value without `.clone()`.

`#[non_exhaustive]` on the struct forces builder use: external consumers can't
struct-literal-construct `InlineStyle { fg: None, ... }`. Internal envision code
also goes through the builder (consistency). Future modifier additions (e.g.,
`reversed`, `dim`) land additively without breaking external code.

### `StyledInline` constructors (two-layer surface)

```rust
impl StyledInline {
    /// Wrap text with an explicit `InlineStyle`. The general-purpose
    /// constructor for any combination of dimensions.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::styled_text::{InlineStyle, StyledInline};
    /// use ratatui::style::Color;
    ///
    /// let inline = StyledInline::styled(
    ///     "840.16 ms",
    ///     InlineStyle::new().fg(Color::Red).bold(),
    /// );
    /// // Renders as red AND bold.
    /// ```
    pub fn styled(text: impl Into<String>, style: InlineStyle) -> Self {
        Self::Styled {
            text: text.into(),
            style,
        }
    }

    /// Single-dimension helper: bold text.
    ///
    /// Equivalent to `StyledInline::styled(text, InlineStyle::new().bold())`.
    pub fn bold(text: impl Into<String>) -> Self {
        Self::styled(text, InlineStyle::new().bold())
    }

    /// Single-dimension helper: italic text.
    pub fn italic(text: impl Into<String>) -> Self {
        Self::styled(text, InlineStyle::new().italic())
    }

    /// Single-dimension helper: underlined text.
    pub fn underlined(text: impl Into<String>) -> Self {
        Self::styled(text, InlineStyle::new().underlined())
    }

    /// Single-dimension helper: strikethrough text.
    pub fn strikethrough(text: impl Into<String>) -> Self {
        Self::styled(text, InlineStyle::new().strikethrough())
    }

    /// Single-dimension helper: text with foreground color.
    ///
    /// "Colored" idiomatically means foreground in TUI contexts (matches
    /// `Span::styled(text, Style::default().fg(...))` ergonomics). For
    /// bg-only or fg+bg cases, use [`StyledInline::styled`] with
    /// `InlineStyle::new().bg(...)` or `.fg(...).bg(...)`.
    pub fn colored(text: impl Into<String>, fg: Color) -> Self {
        Self::styled(text, InlineStyle::new().fg(fg))
    }
}
```

Five leaf helpers — one per single dimension (`bold`, `italic`, `underlined`,
`strikethrough`, `colored`). Symmetric across all boolean dimensions; `colored`
covers the fg case. Two-arg helpers don't fit ergonomically; multi-dimension
cases route through `styled(...)`.

### Render path simplification

`render_inline_styled` at `src/component/styled_text/content.rs:497-525` collapses
from 7 match arms to 3 (Plain / Code / Styled):

```rust
fn render_inline_styled(inline: &StyledInline, base_style: Style) -> RatSpan<'static> {
    match inline {
        StyledInline::Plain(text) => RatSpan::styled(text.clone(), base_style),
        StyledInline::Code(text) => {
            // Code stays bold + base style (theme override happens in render_inline)
            RatSpan::styled(text.clone(), base_style.add_modifier(Modifier::BOLD))
        }
        StyledInline::Styled { text, style } => {
            let mut s = base_style;
            if let Some(fg) = style.fg {
                s = s.fg(fg);
            }
            if let Some(bg) = style.bg {
                s = s.bg(bg);
            }
            if style.bold {
                s = s.add_modifier(Modifier::BOLD);
            }
            if style.italic {
                s = s.add_modifier(Modifier::ITALIC);
            }
            if style.underlined {
                s = s.add_modifier(Modifier::UNDERLINED);
            }
            if style.strikethrough {
                s = s.add_modifier(Modifier::CROSSED_OUT);
            }
            RatSpan::styled(text.clone(), s)
        }
    }
}
```

`render_inline` at `content.rs:475-494` (the theme-aware wrapper) keeps the
theme-coupled `Code` arm:

```rust
fn render_inline(inline: &StyledInline, theme: &Theme, base_style: Style) -> RatSpan<'static> {
    match inline {
        StyledInline::Code(text) => RatSpan::styled(
            text.clone(),
            theme.info_style().add_modifier(Modifier::BOLD),
        ),
        other => render_inline_styled(other, base_style),
    }
}
```

`Code` keeps its theme-coupled fast path. `Styled` (and `Plain`) fall through
to `render_inline_styled` for the dimension-merge logic.

### `Underline` → `underlined` consistency (per brief flag)

The old variant was `StyledInline::Underline` (verb form). The new struct field
and helper use `underlined` (past tense), matching ratatui's `Modifier::UNDERLINED`
and the boolean field naming convention (state = past tense). Old variant deletes;
no symbolic renaming layer.

---

## Migration

### Call-site counts (verified 2026-05-20)

```bash
grep -rn "StyledInline::Plain\|StyledInline::Bold\|StyledInline::Italic\|StyledInline::Underline\|StyledInline::Strikethrough\|StyledInline::Colored\|StyledInline::Code" src/ examples/
```

102 references across 6 files:

| File | Count |
|---|---|
| `examples/styling_showcase.rs` | 34 |
| `src/component/styled_text/tests.rs` | 28 |
| `src/component/styled_text/content.rs` | 17 (mostly doc-test examples) |
| `src/render.rs` | 11 (test fixtures) |
| `examples/styled_text.rs` | 10 |
| `src/component/styled_text/mod.rs` | 2 |

### Mechanical replacement patterns

| Old | New |
|---|---|
| `StyledInline::Bold(t)` | `StyledInline::bold(t)` |
| `StyledInline::Italic(t)` | `StyledInline::italic(t)` |
| `StyledInline::Underline(t)` | `StyledInline::underlined(t)` |
| `StyledInline::Strikethrough(t)` | `StyledInline::strikethrough(t)` |
| `StyledInline::Colored { text: t, fg: Some(c), bg: None }` | `StyledInline::colored(t, c)` |
| `StyledInline::Colored { text: t, fg: Some(c), bg: Some(b) }` | `StyledInline::styled(t, InlineStyle::new().fg(c).bg(b))` |
| `StyledInline::Colored { text: t, fg: None, bg: Some(b) }` | `StyledInline::styled(t, InlineStyle::new().bg(b))` |
| `StyledInline::Plain(t)` | unchanged (variant kept) |
| `StyledInline::Code(t)` | unchanged (variant kept) |

### Consumer-side: leadline's `push_label_value` collapse

`leadline/src/app.rs:473-490` today:

```rust
fn push_label_value(
    out: &mut Vec<StyledInline>,
    label: &str,
    value: &str,
    value_color: Color,
    label_color: Color,
) {
    out.push(StyledInline::Colored {
        text: label.to_string(),
        fg: Some(label_color),
        bg: None,
    });
    out.push(StyledInline::Colored {        // <-- want: bold + colored, get: just colored
        text: value.to_string(),
        fg: Some(value_color),
        bg: None,
    });
}
```

Post-G6:

```rust
fn push_label_value(
    out: &mut Vec<StyledInline>,
    label: &str,
    value: &str,
    value_color: Color,
    label_color: Color,
) {
    out.push(StyledInline::colored(label, label_color));
    out.push(StyledInline::styled(
        value,
        InlineStyle::new().fg(value_color).bold(),  // bold + color, both
    ));
}
```

The two single-dimension `Colored` calls become helper calls; the multi-dimension
case (`bold + colored`) uses `styled(...)` with explicit `InlineStyle`.

### Pattern-matching consumers

Any external code that exhaustively matches `StyledInline` (today: 7 arms)
must rewrite for the new shape (3 arms). The 5 deleted variants don't have a
1:1 match arm in the new form — they collapse into `Styled { style: InlineStyle, .. }`
with the modifier fields encoding which dimensions are active. Match-based
consumers gain a `_` arm (per `#[non_exhaustive]`) and need to inspect
`InlineStyle` fields rather than enum variants.

No external consumers in envision's own repo pattern-match `StyledInline` —
all the references are construction sites. CHANGELOG flags the break for any
external consumer doing custom rendering atop `StyledInline`.

---

## Files to touch

| File | Change |
|---|---|
| `src/component/styled_text/content.rs` | Delete 5 leaf variants; add `Styled` variant; add `#[non_exhaustive]` on `enum StyledInline`. Add `InlineStyle` struct + `#[non_exhaustive]` + builder impl. Add 5 leaf-helper constructors on `StyledInline` (`bold`, `italic`, `underlined`, `strikethrough`, `colored`) + `styled` general constructor. Update `render_inline_styled` match (7 arms → 3). Migrate 17 internal doc-test references. |
| `src/component/styled_text/tests.rs` | Migrate 28 references; add 8 new tests for `InlineStyle` builder + `Styled` round-trip + composed-modifier render pin (bold+colored) + full-dimension combo |
| `src/render.rs` | Migrate 11 test-fixture references |
| `examples/styling_showcase.rs` | Migrate 34 references |
| `examples/styled_text.rs` | Migrate 10 references |
| `src/component/styled_text/mod.rs` | Migrate 2 doc references |
| `CHANGELOG.md` | Additive entry under `[Unreleased]` |

102 mechanical replacements total. File-size check: `src/component/styled_text/content.rs` is currently 521 lines (the file the `Styled` variant + `InlineStyle` struct + helpers live in); estimated +120 lines for the new struct + helpers, +5 lines on the enum, –40 lines from the simplified render match → final ~600 lines. Well under cap.

---

## Tests

Eight new tests in `src/component/styled_text/tests.rs`. Each pins a distinct
invariant.

### Unit tests

1. **`test_inline_style_new_is_default`** — `InlineStyle::new()` equals
   `InlineStyle::default()`. Pins the constructor contract.

2. **`test_inline_style_builder_chain`** — `InlineStyle::new().bold().fg(c).underlined()`
   sets exactly `bold=true`, `fg=Some(c)`, `underlined=true`; other fields stay
   default. Round-trip through field accessors.

3. **`test_styled_inline_styled_pairs_text_and_style`** — `StyledInline::styled(t, s)`
   produces `Styled { text: t, style: s }`. Direct destructure assert.

4. **`test_styled_inline_leaf_helpers_match_builder`** — for each of the 5 helpers,
   `StyledInline::bold(t)` equals `StyledInline::styled(t, InlineStyle::new().bold())`.
   Same for italic / underlined / strikethrough / colored. Pins the leaf-helper
   contract against drift.

### Render-path tests (insta snapshots + ANSI assertions)

5. **`snapshot_styled_inline_bold_and_colored_combined`** — render a styled
   inline with `InlineStyle::new().fg(Color::Red).bold()` into a row.
   ANSI-assert BOTH `\x1b[31m` (red) AND `\x1b[1m` (BOLD) appear. **The
   top-line payoff pin** — proves the bold-on-banner-values combo lands.

6. **`snapshot_styled_inline_full_dimension_combo`** — render styled inline
   with bold + italic + underlined + strikethrough + fg + bg. ANSI-assert all
   six effects appear in the output. Pins the full composability surface.

7. **`test_inline_style_default_no_modifiers_applied`** — render
   `StyledInline::styled(t, InlineStyle::default())` and `StyledInline::Plain(t)`.
   Assert the rendered output is identical (default `InlineStyle` applies no
   modifiers; equivalent to `Plain` for rendering purposes).

8. **`snapshot_styled_inline_plain_and_code_unchanged_postmigration`** — render
   `Plain(t)` and `Code(t)` separately. Snapshots should be byte-identical to
   the pre-G6 baseline — `Plain` and `Code` are unchanged by this PR.

Plus migration of the existing 28 tests in `tests.rs` from leaf variants to
helpers. Existing insta snapshots for those tests should be byte-identical
pre/post migration (the rename is value-level; rendered output stays the same
for single-dimension cases).

---

## Risks & open questions

### Risks

- **102-site mechanical migration scope.** Risk: missed site → compile error
  (since 5 variants get deleted, any stale reference fails to compile). The
  compiler is the safety net here — no sites can hide.
- **Insta snapshot regressions.** For existing tests that exercise single-
  dimension styling (`Bold(t)`, `Italic(t)`, etc.), the rendered output should
  be byte-identical when migrated to `StyledInline::bold(t)` etc. — both forms
  produce the same `Span::styled(t, base.add_modifier(...))`. If any snapshot
  diffs, investigate before accepting. The render-arm change should produce
  identical output for all single-dimension cases; the new surface is the
  ability to combine dimensions.
- **External consumers pattern-matching `StyledInline`.** Anyone with custom
  rendering logic that exhaustively matches `StyledInline` (the 7-arm match
  pattern from current code) must migrate to the 3-arm shape with `InlineStyle`
  field inspection. envision is pre-1.0; consumers expect breaking changes.
  CHANGELOG flags this explicitly.
- **`#[non_exhaustive]` on `InlineStyle` blocks struct-literal construction
  from external code.** Intentional — forces builder use, future-proofs
  modifier additions. Consumers who somehow constructed `InlineStyle { ... }`
  directly today (none in envision repo; none expected) would break. CHANGELOG
  notes the constraint.

### Decisions resolved during brainstorming

| Question | Resolution |
|---|---|
| Q-α: `#[non_exhaustive]` placement (3-layer) | Enum YES (future variants); new `Styled` variant NO (shape stable); `InlineStyle` struct YES (future modifier additions). Matches Severity/NamedColor/CellStyle precedent. |
| Q-β: Deprecate vs delete leaf variants | Delete in one shot. Matches every prior envision migration (D14 paragraph, D6+D9 helpers, D5+D14 StyledTextState, G5 StatusBarStyle severity-map). Helpers preserve single-dimension call-site terseness. |
| Q-γ: Constructor surface | Two-layer: `InlineStyle::new().bold().fg(c)` builder + `StyledInline::styled(t, s)` one-call wrapper + 5 leaf helpers. Rejects enum-chained builder (would silently convert `Plain → Styled`). |
| Q-δ: Migration shape | Mechanical leaf→composable with helpers preserved. ~80% of styled inline use is single-dimension; helpers stay terse. `styled()` reserved for actual composition. |
| `StyledInline::Underline` → `underlined` (past tense) | Yes — matches ratatui's `Modifier::UNDERLINED` and the boolean field naming convention (state = past tense). |
| `Code` variant — keep separate or fold into `InlineStyle`? | Keep separate. Theme-coupled (uses `theme.info_style().add_modifier(Modifier::BOLD)`); consumer doesn't specify color. Conceptually distinct from style-coupled composition. |
| `StyledInline::strikethrough(text)` helper — add or skip? (JC1) | Add. Brief listing 4 helpers was an oversight; the "one helper per single dimension" pattern only earns its keep when complete. Three trivial lines; future contributor would close the gap anyway. |
| `StyledInline::colored(text, fg)` signature (JC2) | Keep fg-only per brief. "Colored" idiomatically means foreground in TUI contexts. bg-only is rare; full fg+bg routes through `styled(t, InlineStyle::new().fg(f).bg(b))`. Helper set stays principled (one per single dimension). |
| `Default` derive on `InlineStyle` | Yes. `InlineStyle::default()` produces all-None/all-false, equivalent to `InlineStyle::new()`. Documented contract. |
| `Copy` derive on `InlineStyle` | Yes. Every field is `Copy` (`Option<Color>`, `bool`). Render path passes by value without `.clone()`. |
| Builder methods `const fn` | Yes. `InlineStyle::new().bold().fg(c)` usable in `const` contexts (module-level static styles). |

---

## Cadence

Same 4-PR cadence as G7 / D1 / chrome-ownership / D6+D9 / D15 / D5+D14 / G4+G5:

1. **PR α** — this design spec (`docs/superpowers/specs/2026-05-20-styled-inline-compose-design.md`).
2. **PR β** — implementation plan (`docs/superpowers/plans/2026-05-20-styled-inline-compose.md`).
3. **PR γ** — implementation. Single coherent breaking-change PR: enum shape
   change + 5 leaf-variant deletions + `InlineStyle` struct addition +
   constructor helpers + render-arm simplification + 102 mechanical migrations.
4. **Tracking-doc PR** — mark G6 ✅ resolved in
   `docs/customer-feedback/2026-05-01-leadline-gaps.md`.

Flag leadline at spec-PR open for review.

---

## Related context

- leadline's customer-feedback inventory: `docs/customer-feedback/2026-05-01-leadline-gaps.md` (G6)
- leadline-side gaps tracking: `~/workspace/ryanoneill/rust-ai-explorations/notes/envision_gaps.md`
- Source brief: `~/workspace/ryanoneill/rust-ai-explorations/notes/envision_styled_inline_compose_redesign.md` (542 lines, re-verified 2026-05-20, commit add30b3)
- Prior atomic-migration playbooks (7 shipped):
  - G1 + G3 + G7 (PRs #459 / #460 / #461 / #458)
  - D1 (PRs #463 / #464 / #465 / #466)
  - G2 + D2 + D11 (PRs #467 / #468 / #469 / #470)
  - D6 + D9 (PRs #471 / #472 / #473 / #474)
  - D15 (PRs #476 / #477 / #478 / #479)
  - D5 + D14 (PRs #480 / #481 / #482 / #483)
  - G4 + G5 (PRs #485 / #486 / #487 / #488)
- leadline call sites this redesign simplifies:
  - `leadline/src/app.rs:473-490` — `push_label_value` helper (bold-colored value segments)
  - `leadline/src/app.rs:492-498` — `push_separator`
  - `leadline/src/app.rs:385-389` — empty-state inlines
  - `leadline/src/app.rs:418-422` — None fallback inlines
- Related envision specs:
  - `docs/superpowers/specs/2026-05-03-theme-palette-severity-design.md` — `Severity` + `Theme::severity_color` that drives the consumer-side severity input to `with_color`/`InlineStyle::new().fg(theme.severity_color(sev))`
  - `docs/superpowers/specs/2026-05-09-styled-text-dx-design.md` — `envision::render::styled_line` that renders `StyledInline` sequences
  - `docs/superpowers/specs/2026-05-19-per-component-style-overrides-design.md` — G4 + G5 layered style precedence at the component level (G6 layers within a single inline run)

This is the eighth coherent redesign drawing from leadline's May 2026 brief
suite. After this, D7 (snapshot testing docs), D3+D8 (docs suite), D10+D12+D13
(small rough edges) remain.
