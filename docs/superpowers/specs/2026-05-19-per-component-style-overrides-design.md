# Per-component style overrides (G4 + G5) — design spec

**Date:** 2026-05-19
**Status:** Approved; implementation plan to follow.
**Tracks:** leadline gaps **G4** + **G5** in `docs/customer-feedback/2026-05-01-leadline-gaps.md`
**Source brief:** `~/workspace/ryanoneill/rust-ai-explorations/notes/envision_per_component_style_overrides.md` (539 lines, re-verified 2026-05-19, commit bc6088b)
**Builds on:** D6 + D9 theme palette + severity helper (PRs #471–#474), D15 `CellStyle::Severity` (PRs #476–#479), D5 + D14 `envision::render::styled_line` (PRs #480–#483)

---

## TL;DR

Two parent-side style override hooks land together because they share the same
DX problem and the same downstream payoff:

- **G4** — `PaneConfig::with_title_style(Style)`. Pane titles currently inherit
  the pane's border style implicitly. Consumers who want a branded title (e.g.,
  Lavender "leadline" on the focused pane while the border is its own
  focus/border color) have no surface. Add `title_style: Option<Style>` field
  to `PaneConfig` + matching `with_title_style` builder; `None` preserves
  current behavior (title styled by border-style inheritance), `Some(style)`
  applies the explicit style at render time.
- **G5** — `StatusBarItem::with_color(Color)` + `with_style_override(Style)`.
  The current `StatusBarStyle` enum closes the per-item style space at 6
  variants (Default/Info/Success/Warning/Error/Muted). Consumers who want
  arbitrary colors (Lavender brand on "leadline" segment) or full custom
  styles (severity-routed slowdown segments) reach a dead end. Add two
  layered `Option` fields + matching builders. Render-time precedence:
  `style_override > color > style.style(theme)`.

**Top-line payoff (Q-γ):** G5 unblocks the four-stop severity ramp deferred
during D6 + D9 design. Today, leadline's `severity_status_style` consumer
helper collapses `Severity::Bad` and `Severity::Mild` both to
`StatusBarStyle::Warning` because no Peach variant exists. After G5, the
helper deletes entirely; the StatusBar slowdown segments call
`with_color(theme.severity_color(sev))` directly and pick up the full
four-stop ramp. Three views (table cells from D15 `CellStyle::Severity`,
summary banner from D5 `styled_line` + `theme.severity_color`, StatusBar
slowdown segments) converge on the same `Severity::Good | Mild | Bad |
Critical` gradient the moment G5 lands. This convergence is the spec's
top-line success criterion — design choices that preserve it weigh heaviest.

---

## Goals

1. **Branded titles per pane.** `PaneConfig::with_title_style(style)` lets a
   parent set per-pane title coloring/modifier independently of the border
   style. The chrome ownership protocol from G2 + D2 + D11 (PR #469) makes
   titles parent-owned — `with_title_style` is purely a parent-side knob;
   no propagation work.
2. **Arbitrary per-item StatusBar coloring.** `StatusBarItem::with_color(color)`
   for the 80% one-axis case (just the foreground color); `with_style_override(style)`
   for the rare full-style escape hatch. Both layered on top of the existing
   `with_style(StatusBarStyle)` semantic baseline.
3. **Layered, not last-call-wins.** Each setter (`with_style`, `with_color`,
   `with_style_override`) configures its own independent field. Render-time
   resolution applies the precedence `style_override > color > style.style(theme)`.
   Branched construction patterns stay predictable: setting `style_override`
   doesn't silently clear `color`.
4. **Four-stop severity ramp restored.** Post-G5, consumer-side
   `severity_status_style` helpers that collapse Bad+Mild → Warning delete.
   StatusBar slowdown segments use `with_color(theme.severity_color(sev))`
   directly. Three convergence views (table / banner / status bar) reach
   the same palette.
5. **Composes with chrome_owned.** No interaction with the chrome propagation
   protocol. G4's title style applies to text drawn inside parent-owned
   chrome (no child component sees it). G5's StatusBar render path doesn't
   participate in chrome ownership (it's a leaf renderer).

## Non-goals

- **`with_title_spans(Vec<Span>)` for multi-style titles** (Q-α). Deferred to
  a follow-up PR when a consumer needs multi-segment title styling. Single
  `Style` per pane covers leadline's use case today; designing the multi-span
  API without a grounding consumer risks YAGNI bloat.
- **`with_title_modifier(Modifier)` convenience.** Skip. Consumers wanting
  just `BOLD` on a title write `with_title_style(Style::default().add_modifier(Modifier::BOLD))`.
  Tiny extra typing for a use case that may never materialize.
- **`with_color` background variant** (`with_bg_color(Color)`). Skip. Same
  YAGNI reasoning — no consumer needs bg-only override today. Available via
  `with_style_override(Style::default().bg(color))` if someone does.
- **Last-call-wins setter semantics for G5.** Explicitly rejected per Q-β +
  the branched-construction analysis below. Each field persists independently;
  precedence is render-time, not setter-time.
- **Theme-inherited `with_color` semantics** (i.e., `item.style.style(theme).fg(color)`).
  Explicitly rejected per the just-fg analysis below. `with_color(c)` produces
  `Style::default().fg(c)` — clean, predictable, matches the consumer-naming-a-specific-color
  intent.
- **`StatusBarStyle` enum expansion** (e.g., adding `StatusBarStyle::Severity(Severity)`).
  Skip. The existing 6 semantic variants stay closed; `with_color` is the
  escape hatch for everything outside them. Mirrors the D15 decision where
  `CellStyle` gained one variant (`Severity`) but `Cell::severity()` /
  `Cell::with_severity()` were the load-bearing ergonomic surface.

---

## Design

### G4 — `PaneConfig::with_title_style(Style)`

Add a new optional field to the existing struct in `src/component/pane_layout/mod.rs:80`:

```rust
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct PaneConfig {
    id: String,
    title: Option<String>,
    title_style: Option<Style>,  // NEW
    proportion: f32,
    min_size: u16,
    max_size: u16,
}
```

`title_style` defaults to `None` (constructor preserved); existing consumers
get current behavior unchanged.

Add the builder + getter to the existing `impl PaneConfig`:

```rust
/// Sets the title style (builder pattern).
///
/// When `Some(style)`, the pane title renders with the given style instead
/// of inheriting the border style. When `None` (default), title styling
/// follows the border (current behavior).
///
/// # Example
///
/// ```rust
/// use envision::component::pane_layout::PaneConfig;
/// use ratatui::style::{Color, Style, Modifier};
///
/// let pane = PaneConfig::new("brand")
///     .with_title("leadline")
///     .with_title_style(Style::default().fg(Color::Rgb(180, 190, 254)).add_modifier(Modifier::BOLD));
/// ```
pub fn with_title_style(mut self, style: Style) -> Self {
    self.title_style = Some(style);
    self
}

/// Returns the title style, if explicitly set.
pub fn title_style(&self) -> Option<Style> {
    self.title_style
}
```

Render arm at `src/component/pane_layout/view_with.rs:53-55` becomes:

```rust
if let Some(title) = &pane.title {
    let span = if let Some(style) = pane.title_style {
        ratatui::text::Span::styled(format!(" {} ", title), style)
    } else {
        ratatui::text::Span::raw(format!(" {} ", title))
    };
    block = block.title(span);
}
```

`Span::raw` keeps the current behavior (title inherits border style via
ratatui's `Block::title` default styling). `Span::styled` applies the
consumer-provided style explicitly.

#### chrome_owned composability

Pane titles are parent-owned chrome — `PaneLayout` draws them as part of
the same `Block::default()` that owns the border. The `chrome_owned`
propagation protocol from G2 + D2 + D11 governs whether *child* components
suppress their own chrome; titles are above the chrome boundary, not below.
`with_title_style` is purely a parent-side knob. No propagation work needed.

#### Field placement (serialization compat)

`title_style: Option<Style>` is inserted between `title: Option<String>`
and `proportion: f32` so the new field clusters with the title-related
fields. For consumers using struct-literal construction without spread
(`PaneConfig { id, title, ... }` instead of `PaneConfig::new(id)
.with_title(title)...`), this is a breaking change. envision is pre-1.0;
all envision tests + examples use the builder pattern, so internal impact
is zero. CHANGELOG flags the break for external consumers.

(Serialization caveat: when `feature = "serialization"` is enabled, the
serde derives now serialize the new field. Existing serialized
`PaneConfig` blobs without `title_style` will fail to deserialize unless
the field gets `#[serde(default)]`. Add `#[serde(default)]` to the new
field to preserve forward-compat for existing serialized data.)

### G5 — `StatusBarItem` layered fields

Add two new `Option` fields to `StatusBarItem` in `src/component/status_bar/item.rs:205`:

```rust
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct StatusBarItem {
    pub(super) content: StatusBarItemContent,
    pub(super) style: StatusBarStyle,
    /// Layer 2: foreground color override. Wins over `style.style(theme)` but
    /// loses to `style_override`. `None` defers to `style`.
    pub(super) color: Option<Color>,  // NEW
    /// Layer 3: full `Style` override. Wins over `color` and `style.style(theme)`.
    /// `None` defers to `color`.
    pub(super) style_override: Option<Style>,  // NEW
    separator: bool,
}
```

Same serialization caveat as G4 — new fields get `#[serde(default)]` to
preserve forward-compat with pre-G5 serialized data.

Two new builders + getters in `impl StatusBarItem`:

```rust
/// Sets a foreground color override (builder pattern).
///
/// Wins at render time over the semantic `StatusBarStyle` baseline but
/// loses to `with_style_override`. Produces `Style::default().fg(color)` —
/// background and modifiers are NOT inherited from the semantic baseline.
/// For tinted-background or modifier scenarios, use `with_style_override`.
///
/// # Precedence
///
/// At render time: `style_override > color > style.style(theme)`. Setting
/// `with_color` does NOT clear a prior `with_style_override`; the override
/// continues to win until cleared explicitly.
///
/// # Example
///
/// ```rust
/// use envision::component::StatusBarItem;
/// use envision::theme::{Severity, Theme};
/// use ratatui::style::Color;
///
/// let theme = Theme::catppuccin_mocha();
/// let item = StatusBarItem::new("slowdown")
///     .with_color(theme.severity_color(Severity::Bad));
/// // Renders in the theme's Peach (full four-stop severity ramp).
/// ```
pub fn with_color(mut self, color: Color) -> Self {
    self.color = Some(color);
    self
}

/// Returns the color override, if explicitly set.
pub fn color(&self) -> Option<Color> {
    self.color
}

/// Sets a full `Style` override (builder pattern).
///
/// Highest-precedence layer — wins over both `with_color` and the semantic
/// `StatusBarStyle` baseline. Use when arbitrary modifiers (BOLD, ITALIC,
/// UNDERLINED), background coloring, or full custom styling is needed.
///
/// # Precedence
///
/// At render time: `style_override > color > style.style(theme)`. Setting
/// `with_style_override` does NOT clear a prior `with_color`; the override
/// just wins for as long as it's set.
///
/// # Example
///
/// ```rust
/// use envision::component::StatusBarItem;
/// use ratatui::style::{Color, Modifier, Style};
///
/// let item = StatusBarItem::new("EMERGENCY")
///     .with_style_override(
///         Style::default()
///             .fg(Color::White)
///             .bg(Color::Red)
///             .add_modifier(Modifier::BOLD)
///     );
/// ```
pub fn with_style_override(mut self, style: Style) -> Self {
    self.style_override = Some(style);
    self
}

/// Returns the style override, if explicitly set.
pub fn style_override(&self) -> Option<Style> {
    self.style_override
}
```

#### Render-path resolution

Update `src/component/status_bar/mod.rs:664` from:

```rust
let style = item.style.style(theme);
```

to:

```rust
let style = if let Some(s) = item.style_override {
    s
} else if let Some(c) = item.color {
    Style::default().fg(c)
} else {
    item.style.style(theme)
};
```

Three branches, exhaustively cover the precedence ladder.

#### Layered setter semantics

Each setter independently configures its layer:
- `with_style(s)` writes `self.style`
- `with_color(c)` writes `self.color`
- `with_style_override(o)` writes `self.style_override`

Setters don't clear each other. A consumer chain like
`with_style(Info).with_color(Red).with_style_override(s)` ends with all
three fields populated; render picks per precedence.

This is **deliberately different from D15's `Cell::with_severity`** (where
last-call-wins). D15 had one `style: CellStyle` field with variant
alternatives (`CellStyle::Severity(sev)` and `CellStyle::Custom(s)` are
mutually exclusive). G5 has three independent fields; the natural setter
contract is per-field idempotent writes + render-time precedence among
layers.

#### The branched-construction case

The practical case that decides the layered model:

```rust
let item = StatusBarItem::new("latency")
    .with_style(StatusBarStyle::Info)
    .with_color(theme.severity_color(sev));

let item = if user_wants_emphasis {
    item.with_style_override(Style::default().fg(red).add_modifier(Modifier::BOLD))
} else {
    item
};
```

Under layered semantics: when the emphasis branch is skipped, `color`
drives. When taken, `style_override` wins but `color` is preserved
(rebuildable). Predictable.

Under last-call-wins: calling `with_style_override` silently clears
`color`, so the unbranched path holds the brand/severity color but the
branched path doesn't. Hidden coupling, surprising.

Layered semantics scale cleanly to branched/conditional construction
patterns; last-call-wins doesn't.

#### `with_color` semantics

`with_color(c)` produces `Style::default().fg(c)`. The semantic baseline
(`item.style.style(theme)`) is NOT layered underneath.

Theme-inheriting `item.style.style(theme).fg(color)` was rejected: it
mixes concerns. The resulting style depends on which `StatusBarStyle`
was chosen as the baseline, which is exactly what `with_color` is opting
out of. If a future `StatusBarStyle` variant grows a tinted bg or BOLD
modifier and a consumer calls `with_color(Green)`, they almost certainly
want plain green text, not "green text on Error bg."

Consumers who DO want layering have `with_style_override(item.style.style(theme).fg(color).add_modifier(Modifier::BOLD))`
available — explicit, no ambiguity, no surprise.

leadline's actual use cases all want just-fg semantics:
- Lavender brand on "leadline" pane header → `with_color(Lavender)`
- Four-stop severity on slowdown segments → `with_color(theme.severity_color(sev))`

No consumer surface needs theme-inheritance today.

---

## Migration

### Consumer-side: `severity_status_style` helper deletes

Today leadline's `severity_status_style` consumer helper:

```rust
fn severity_status_style(ratio: f64) -> StatusBarStyle {
    // Collapses Bad + Mild → Warning because no Peach variant exists.
    match Severity::from_thresholds(ratio, RATIO_THRESHOLDS) {
        Severity::Good => StatusBarStyle::Success,
        Severity::Mild => StatusBarStyle::Warning,
        Severity::Bad => StatusBarStyle::Warning,     // collapse
        Severity::Critical => StatusBarStyle::Error,
    }
}

// Call site
StatusBarItem::new(format!("slowdown: {:.2}x", ratio))
    .with_style(severity_status_style(ratio))
```

Post-G5, the helper deletes entirely; the call site uses
`theme.severity_color`:

```rust
let sev = Severity::from_thresholds(ratio, RATIO_THRESHOLDS);
StatusBarItem::new(format!("slowdown: {:.2}x", ratio))
    .with_color(theme.severity_color(sev))
// Bad now renders as Peach (full four-stop ramp).
```

The four-band-to-three-band collapse documented in D6 + D9 (Default theme
caveat) is preserved — that collapse is intrinsic to the basic-Color
palette. On Catppuccin Mocha (and other full-palette themes), the four
bands stay distinct.

### Consumer-side: branded pane titles

Today leadline's per-pane title uses no styling — the title inherits the
border style. With G4:

```rust
// Before — title color = border color (always)
PaneConfig::new("brand").with_title("leadline")

// After — title color independent of border
PaneConfig::new("brand")
    .with_title("leadline")
    .with_title_style(
        Style::default()
            .fg(theme.color(NamedColor::Lavender))
            .add_modifier(Modifier::BOLD),
    )
```

### Migration table

| Old | New |
|---|---|
| `StatusBarItem::new(t).with_style(severity_status_style(ratio))` | `StatusBarItem::new(t).with_color(theme.severity_color(Severity::from_thresholds(ratio, ...)))` |
| `severity_status_style(ratio: f64) -> StatusBarStyle` helper | Delete entirely; inline at call site |
| `PaneConfig::new(id).with_title(t)` (title inherits border) | `PaneConfig::new(id).with_title(t).with_title_style(style)` for branded title |
| StatusBar `EMERGENCY` segment in plain `StatusBarStyle::Error` | `StatusBarItem::new("EMERGENCY").with_style_override(Style::default().fg(White).bg(Red).add_modifier(BOLD))` |

### Internal envision migration

- envision's own components don't construct `PaneConfig` with title style
  or `StatusBarItem` with color overrides — the existing test fixtures
  and examples use the default surface only. Zero internal migration
  required.
- Examples that demonstrate per-pane title styling or arbitrary StatusBar
  coloring can be added as doc-test examples on the new builders, not as
  full example files (YAGNI; the builder docstrings cover the canonical
  use case).

---

## Files to touch

| File | Change |
|---|---|
| `src/component/pane_layout/mod.rs` | `PaneConfig::title_style: Option<Style>` field, `with_title_style`, `title_style()` getter, `#[serde(default)]` on the new field |
| `src/component/pane_layout/view_with.rs:53-55` | Render arm consults `pane.title_style` |
| `src/component/pane_layout/tests.rs` | Tests for title style getter/setter + render snapshot demonstrating branded title independent of border |
| `src/component/status_bar/item.rs` | `color: Option<Color>` field, `style_override: Option<Style>` field, two `with_*` builders, two getters, `#[serde(default)]` on both new fields |
| `src/component/status_bar/mod.rs:664` | Layered render arm (override > color > style.style(theme)) |
| `src/component/status_bar/tests/style_item.rs` | Tests for both new builders + precedence interactions (layered, not last-call-wins) |
| `src/component/status_bar/snapshot_tests.rs` | Snapshot test showing four-stop severity ramp distinct from current Warning collapse |
| `CHANGELOG.md` | Additive entry under `[Unreleased]` |

---

## Tests

Eight new tests across three files. Each pins a distinct invariant.

### `src/component/pane_layout/tests.rs`

1. **`test_pane_config_with_title_style`** — `PaneConfig::new("p").with_title("t").with_title_style(s)`
   produces a config with `title()` = `Some("t")` and `title_style()` = `Some(s)`.

2. **`test_pane_config_title_style_default_none`** — `PaneConfig::new("p")` has
   `title_style()` returning `None`. Constructor unchanged.

3. **`snapshot_pane_with_branded_title_style`** — render a 2-pane horizontal
   layout where the left pane has `with_title_style(Style::default().fg(Color::Magenta))`
   and the right pane has no title style. Snapshot the rendered output;
   ANSI-assert the left title contains `\x1b[35m` (magenta) and the right
   title does not.

### `src/component/status_bar/tests/style_item.rs`

4. **`test_status_bar_item_with_color`** — `StatusBarItem::new("x").with_color(Color::Red)`
   has `color()` = `Some(Color::Red)`, `style_override()` = `None`, `style()`
   = `StatusBarStyle::Default`.

5. **`test_status_bar_item_with_style_override`** — `StatusBarItem::new("x").with_style_override(s)`
   has `style_override()` = `Some(s)`, `color()` = `None`.

6. **`test_status_bar_item_layered_setters_preserve_color`** — chain
   `with_style(Info).with_color(Red).with_style_override(custom)`. All
   three fields populated; `color()` returns `Some(Red)` (NOT cleared by
   the subsequent `with_style_override`). Pins layered semantics, not
   last-call-wins.

7. **`test_status_bar_item_branched_construction_preserves_color`** — build
   an item with `with_color(brand_color)`, then conditionally apply
   `with_style_override`. After the conditional, `color()` is still
   `Some(brand_color)`. Pins the branched-construction safety net from §Design.

### `src/component/status_bar/snapshot_tests.rs`

8. **`snapshot_status_bar_four_stop_severity_ramp`** — render four status
   bar items, one per Severity band (Good/Mild/Bad/Critical), each using
   `with_color(theme.severity_color(sev))` on Catppuccin Mocha. Snapshot
   the rendered ANSI; assert four distinct foreground color codes appear.
   Specifically that Bad and Mild render to different ANSI codes (proves
   the four-stop ramp restored).

---

## Risks & open questions

### Risks

- **Public field-add as breaking-shape change.** Adding `title_style` to
  `PaneConfig` and `color` + `style_override` to `StatusBarItem` is a
  breaking change for any external code that constructs the structs via
  struct literal without `..PaneConfig::new(...)` spread. envision is
  pre-1.0; all envision tests + examples use builders. Risk is
  external-consumer only; CHANGELOG flags it.
- **Serialization forward-compat.** `feature = "serialization"` derives
  serde on both structs. Existing serialized blobs lack the new fields.
  Mitigation: `#[serde(default)]` on every new field. `Option` defaults
  to `None` naturally; struct round-trips preserve current behavior for
  pre-G5 data.
- **Layered setter semantics surprise.** Consumers familiar with
  `Cell::with_severity` (last-call-wins from D15) might expect
  `with_style_override` to clear `color`. Mitigation: docstring on every
  new builder explicitly documents the layered model + the precedence
  ladder. CHANGELOG entry highlights the difference.
- **`with_color` doesn't inherit theme bg.** A consumer who wanted
  `with_color(Green)` to render "green text on the theme's normal bg"
  gets `Style::default().fg(Green)` instead (transparent bg). Mitigation:
  docstring example is explicit. `with_style_override` is the escape
  hatch for layered cases.

### Decisions resolved during brainstorming

| Question | Resolution |
|---|---|
| Q-α: Single `Style` per pane vs `Vec<Span>` (leadline pre-answer) | Single `Style` this brief; defer `with_title_spans(Vec<Span>)` to follow-up. leadline only needs single-style today. |
| Q-β: `with_color` AND `with_style_override`, or just one? (leadline pre-answer) | Both. `with_color` is the 80% case ergonomic; `with_style_override` is the escape hatch. Tiny API surface cost for a big DX win. Precedence: `style_override > color > style.style(theme)`. |
| Q-γ: Does G5 unblock the four-stop ramp deferred by D6 + D9? (leadline pre-answer + spec confirmation) | Yes, fully. Post-G5, `severity_status_style` helper deletes; StatusBar slowdown segments call `with_color(theme.severity_color(sev))` directly. Three views (D15 cells, D5 banner, G5 status) converge on the same four-stop gradient. **Top-line success criterion.** |
| JC1: Layered vs last-call-wins setter semantics (envision-side judgment) | Layered. Three independent fields by design; setters write their own field idempotently; render-time precedence picks. Branched-construction patterns stay predictable. |
| JC2: `with_color` semantics — `Style::default().fg()` vs theme-inherited (envision-side judgment) | `Style::default().fg(color)`. Clean separation; consumer named a specific color, gets that specific color. Theme-inheriting paths route through `with_style_override(item.style.style(theme).fg(color))` explicitly. |
| Field placement in struct definitions | New fields cluster with their semantic neighbors (`title_style` next to `title`; `color` and `style_override` next to `style`). Aesthetic; no behavior impact. |
| `#[serde(default)]` on new fields | Yes. Preserves forward-compat for pre-G5 serialized `PaneConfig` / `StatusBarItem` blobs. |
| `StatusBarStyle` enum expansion (e.g., new `Severity(Severity)` variant) | No. `with_color` is the escape hatch outside the closed 6-variant set. Mirrors D15's decision pattern. |

---

## Cadence

Same 4-PR cadence as G7 / D1 / chrome-ownership / D6+D9 / D15 / D5+D14:

1. **PR α** — this design spec (`docs/superpowers/specs/2026-05-19-per-component-style-overrides-design.md`).
2. **PR β** — implementation plan (`docs/superpowers/plans/2026-05-19-per-component-style-overrides.md`).
3. **PR γ** — implementation. Mostly additive (two new builders per component
   + render-path arm updates). Field-add breaking-shape change is small;
   internal migration is zero (envision uses builders throughout).
4. **Tracking-doc PR** — mark G4 + G5 ✅ resolved in
   `docs/customer-feedback/2026-05-01-leadline-gaps.md`.

Flag leadline at spec-PR open for review.

---

## Related context

- leadline's customer-feedback inventory: `docs/customer-feedback/2026-05-01-leadline-gaps.md` (G4, G5)
- leadline-side gaps tracking: `~/workspace/ryanoneill/rust-ai-explorations/notes/envision_gaps.md`
- Source brief: `~/workspace/ryanoneill/rust-ai-explorations/notes/envision_per_component_style_overrides.md` (539 lines, re-verified 2026-05-19, commit bc6088b)
- Prior atomic-migration playbooks (6 shipped):
  - G1 + G3 + G7 (PRs #459 / #460 / #461 / #458)
  - D1 (PRs #463 / #464 / #465 / #466)
  - G2 + D2 + D11 (PRs #467 / #468 / #469 / #470)
  - D6 + D9 (PRs #471 / #472 / #473 / #474)
  - D15 (PRs #476 / #477 / #478 / #479)
  - D5 + D14 (PRs #480 / #481 / #482 / #483)
- leadline call sites this redesign simplifies:
  - `leadline/src/app.rs` — `severity_status_style` helper (deletes after G5)
  - StatusBar slowdown segments — direct `with_color(theme.severity_color(sev))` after G5
  - Per-pane title styling — branded "leadline" header after G4
- Related envision specs:
  - `docs/superpowers/specs/2026-05-02-chrome-ownership-design.md` — chrome_owned
    propagation (titles are above the chrome boundary; G4 doesn't interact)
  - `docs/superpowers/specs/2026-05-03-theme-palette-severity-design.md` —
    `Severity` enum + `theme.severity_color` that G5 unblocks for StatusBar
  - `docs/superpowers/specs/2026-05-08-cellstyle-severity-design.md` —
    `CellStyle::Severity(Severity)` that the same four-stop ramp drives
  - Future: G6 (StyledInline composable styles) brief at
    `~/workspace/ryanoneill/rust-ai-explorations/notes/envision_styled_inline_compose_redesign.md` —
    restores bold-on-banner-values, the next swing after G4+G5.

This is the seventh coherent redesign drawing from leadline's May 2026 brief
suite. After this, G6 (StyledInline composable), D7 (snapshot testing docs),
the documentation suite (D3+D8), and the small-rough-edges punch list
(D10+D12+D13) remain.
