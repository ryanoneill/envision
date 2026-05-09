# `CellStyle::Severity(Severity)` — design spec

**Date:** 2026-05-08
**Status:** Approved; implementation plan to follow.
**Tracks:** leadline gap **D15** in `docs/customer-feedback/2026-05-01-leadline-gaps.md`
**Source brief:** `~/workspace/ryanoneill/rust-ai-explorations/notes/envision_gaps.md` D15 entry
**Builds on:** `docs/superpowers/specs/2026-05-03-theme-palette-severity-design.md` (D6 + D9)

---

## TL;DR

`TableRow::cells(&self) -> Vec<Cell>` takes no `&Theme` parameter. After D6 + D9
shipped `Severity` and `theme.severity_color/style`, the natural call shape from
inside a row's `cells()` impl wants the active theme — but there's no way to
thread it through. Consumers (leadline) work around this by constructing
`Theme::catppuccin_mocha()` inline at the top of each `severity_cell_style`
helper. Workable; defeats theme-swap.

Add `CellStyle::Severity(Severity)` as a new variant to the `CellStyle` enum.
Resolve at render time in `cell_style_to_ratatui` via the renderer's already-in-scope
`&Theme`: `CellStyle::Severity(sev) => theme.severity_style(*sev)`. No change to
`TableRow` trait signature. No change to render-path arity. Pushes severity awareness
into the `Cell` type where it belongs; the renderer is already the place that owns
theme-coupled style resolution.

Bundle a forward-compatibility cleanup: mark `CellStyle` `#[non_exhaustive]` in the
same PR. Adding a variant to a non-`#[non_exhaustive]` enum is already a breaking
change for any external `match cell.style() { ... }` that's exhaustive; doing both
together yields one breakage instead of two and matches the convention set by
`Severity` and `NamedColor` in PR #473.

Add two `Cell` constructors mirroring the existing `Cell::success / warning / error /
muted` family plus the typed-cell builder pattern from G7:

- `Cell::severity(text, sev)` — semantic shorthand for the simple case.
- `Cell::with_severity(sev)` — builder, for the typed-cell chain like
  `Cell::number(value).with_text(formatted).with_severity(sev)` that preserves the
  G7 typed `SortKey`.

---

## Goals

1. **Theme-aware severity cells.** A `TableRow::cells()` impl that wants to color
   a cell by severity does so by emitting `CellStyle::Severity(sev)` and trusting
   the renderer to resolve through the active theme. No `Theme::catppuccin_mocha()`
   inline constructions; theme-swap works end to end.
2. **No `TableRow` trait churn.** The trait signature stays `fn cells(&self) ->
   Vec<Cell>`. Every existing impl keeps compiling. Migration for severity-aware
   rows is a per-cell change.
3. **Compose with G7 typed sort keys.** `Cell::number(value).with_text(formatted)
   .with_severity(sev)` chains cleanly. The typed `SortKey` survives — severity
   coloring layers on top, doesn't replace.
4. **Compose with `chrome_owned`.** Adding a `Severity(Severity)` variant doesn't
   touch the chrome ownership protocol. `cell_style_to_ratatui` already runs inside
   the table's internal render path, after chrome resolution.
5. **One breaking change.** Bundle `#[non_exhaustive]` on `CellStyle` with the
   variant addition; future variants land additively without further breakage.

## Non-goals

- **Extending `TableRow::cells()` to take `&Theme`.** Option (a) in the gap brief.
  Direct, but breaks every existing `TableRow` impl across consumers; doesn't
  generalize beyond severity.
- **Generalizing to `CellStyle::Themed(impl Fn(&Theme) -> Style)`.** Function-typed
  variants would lose `Clone + Debug + PartialEq` derives on `CellStyle` and force
  a major API churn. `Severity(Severity)` is a value-typed variant — same shape as
  `Custom(Style)`, just resolved through the theme rather than passed through.
- **Adding a "muted severity" variant.** Disabled cells already render dark-gray
  via the existing `disabled` override path. No special-casing needed.
- **Per-cell `disabled` styling.** Out of scope. The render path's `disabled` is a
  table-level signal that overrides every cell's style; per-cell disable would be
  a different feature.
- **Removing `CellStyle::Custom(Style)`.** The escape hatch stays. `Severity` is
  for the four-band gradient case; `Custom` is for arbitrary one-off styling.
- **Adding more semantic variants (Info, Critical, Highlight, etc.).** Severity is
  a value-typed gradient; adding fixed semantic variants doesn't generalize. If
  more value-typed cell coloring lands later, it gets its own variant.

---

## Design

### `CellStyle::Severity(Severity)` variant

Add the variant to the existing `CellStyle` enum in `src/component/cell.rs`. Mark
the enum `#[non_exhaustive]` in the same PR.

```rust
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq)]
pub enum CellStyle {
    /// No override — render with the theme's default cell style.
    #[default]
    Default,
    /// Maps to the theme's success color (typically green).
    Success,
    /// Maps to the theme's warning color (typically yellow).
    Warning,
    /// Maps to the theme's error color (typically red).
    Error,
    /// Maps to the theme's muted color (typically dark gray) for de-emphasized text.
    Muted,
    /// Applies a raw `ratatui::style::Style` directly, bypassing theme mapping.
    Custom(Style),
    /// Resolves to the theme's severity color + style at render time.
    /// `Critical` adds a `BOLD` modifier; other variants are color-only.
    /// See [`Theme::severity_style`].
    Severity(Severity),
}
```

The variant carries the `Severity` band by value (no extra parameters). The render
path resolves the band through the active theme — the renderer is the right place
because it already takes `&Theme`.

### Render arm in `cell_style_to_ratatui`

Add one match arm to `cell_style_to_ratatui` in `src/component/table/render.rs`:

```rust
fn cell_style_to_ratatui(style: &CellStyle, theme: &Theme, disabled: bool) -> Style {
    if disabled {
        return Style::default().fg(Color::DarkGray);
    }
    match style {
        CellStyle::Default => Style::default(),
        CellStyle::Success => theme.success_style(),
        CellStyle::Warning => theme.warning_style(),
        CellStyle::Error => theme.error_style(),
        CellStyle::Muted => Style::default().fg(Color::DarkGray),
        CellStyle::Custom(s) => *s,
        CellStyle::Severity(sev) => theme.severity_style(*sev),
    }
}
```

`theme.severity_style(sev)` is the function shipped in PR #473. It routes
`Good → Green / Mild → Yellow / Bad → Peach / Critical → Red` through the theme's
palette, and adds `Modifier::BOLD` for `Critical` only. The renderer doesn't need
any new theme accessor.

### `Cell::severity(text, sev)` constructor

Add to `src/component/cell.rs` next to the existing semantic constructors
(`Cell::success`, `Cell::warning`, `Cell::error`, `Cell::muted`):

```rust
impl Cell {
    /// Semantic severity-styled cell. Resolves color through the active theme
    /// at render time via [`Theme::severity_style`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::cell::Cell;
    /// use envision::theme::Severity;
    ///
    /// let cell = Cell::severity("CrashLoopBackOff", Severity::Critical);
    /// // Renders as the theme's red + BOLD on Catppuccin Mocha.
    /// ```
    pub fn severity(text: impl Into<CompactString>, sev: Severity) -> Self {
        Self {
            text: text.into(),
            style: CellStyle::Severity(sev),
            sort_key: None,
        }
    }
}
```

### `Cell::with_severity(sev)` builder

Add as a builder, for the typed-cell pattern that preserves G7 typed `SortKey`:

```rust
impl Cell {
    /// Builder: set the cell style to severity-styled.
    ///
    /// Composes with the typed-cell pattern from G7:
    ///
    /// ```rust
    /// use envision::component::cell::Cell;
    /// use envision::theme::Severity;
    ///
    /// let ratio = 5.2;
    /// let cell = Cell::number(ratio)
    ///     .with_text(format!("{:.2}x", ratio))
    ///     .with_severity(Severity::Bad);
    /// // Numeric SortKey preserved; severity color layered on top.
    /// ```
    ///
    /// # Precedence
    ///
    /// Last-call-wins with [`with_style`](Self::with_style). Calling
    /// `.with_style(CellStyle::Custom(...)).with_severity(Bad)` ends with
    /// `CellStyle::Severity(Bad)`; the prior `Custom` is dropped. This is
    /// natural builder-pattern semantics — each setter overwrites.
    pub fn with_severity(mut self, sev: Severity) -> Self {
        self.style = CellStyle::Severity(sev);
        self
    }
}
```

The docstring's **Precedence** section is leadline's plan-time addition: explicit
documentation that `with_severity` overwrites any prior `with_style(...)` call,
matching the universal builder convention. Consumers get clarity instead of
surprise.

### Disabled handling

No new code path. `disabled` already overrides every variant in
`cell_style_to_ratatui`:

```rust
if disabled {
    return Style::default().fg(Color::DarkGray);
}
```

`CellStyle::Severity(Critical)` on a disabled row renders as plain dark-gray (no
BOLD, no red). Same as every other variant. The disabled-cell-is-muted contract
is intact.

### `#[non_exhaustive]` on `CellStyle`

Two reasons to bundle this with the variant addition:

1. **One breaking change.** Adding a non-`#[non_exhaustive]` enum variant is
   already a breaking change for any external code that pattern-matches the enum
   exhaustively. Adding `#[non_exhaustive]` retroactively is also breaking. Doing
   both at once costs one breakage instead of two.
2. **Future variants land additively.** Once `#[non_exhaustive]`, any later
   variant addition (e.g., a hypothetical `CellStyle::Highlight` or `CellStyle::Themed`)
   doesn't break consumers' `match` arms. They already need a `_` arm because of
   `#[non_exhaustive]`; the same arm covers future additions.

The breaking-change footprint is small. The accessor returns `&CellStyle`; most
consumers only call `cell.text()` or pass `Cell` instances through. External code
that pattern-matches `CellStyle` exhaustively is rare. Migration: add a `_` arm.

This matches the convention set by `Severity` and `NamedColor` in PR #473 — both
shipped as `#[non_exhaustive]` for the same forward-compat reason.

### Builder precedence with existing `with_style`

`Cell::with_style(style)` already exists. `with_severity(sev)` is shorthand for
`with_style(CellStyle::Severity(sev))`. Both are single-field setters; calling
either overwrites whatever's currently in `self.style`. Last-call-wins.

Documented explicitly in `with_severity`'s docstring (per leadline's plan-time
note). The same precedence applies to `with_style → with_severity` and
`with_severity → with_style` chains.

---

## Migration

### Consumer-side: leadline's two `severity_cell_style` helpers collapse

**Before (current workaround — defeats theme-swap):**

```rust
// leadline/src/baseline.rs
fn severity_cell_style(ratio: f64) -> CellStyle {
    let theme = Theme::catppuccin_mocha();  // hardcoded — defeats theme-swap
    let sev = Severity::from_thresholds(ratio, RATIO_SEVERITY_THRESHOLDS);
    CellStyle::Custom(theme.severity_style(sev))
}

// per_op.rs has the same shape.
```

**After (post-D15):**

```rust
// leadline/src/baseline.rs — helper deleted; inline at call site.
let sev = Severity::from_thresholds(self.ratio, RATIO_SEVERITY_THRESHOLDS);
Cell::number(self.ratio)
    .with_text(format!("{:.2}x", self.ratio))
    .with_severity(sev)
```

The two `severity_cell_style` helpers in `leadline/src/baseline.rs` and
`leadline/src/per_op.rs` delete entirely. The inline `Theme::catppuccin_mocha()`
constructions disappear. `per_op`'s `<5% = Default` branch becomes an inline `if`
or stays a tiny helper — call site decides.

### Migration table

| Old (workaround) | New (D15 API) |
|---|---|
| `let theme = Theme::catppuccin_mocha();`<br>`let sev = Severity::from_thresholds(...);`<br>`CellStyle::Custom(theme.severity_style(sev))` | `let sev = Severity::from_thresholds(...);`<br>`CellStyle::Severity(sev)` |
| `Cell::number(x).with_text(...).with_style(severity_cell_style(x))` | `Cell::number(x).with_text(...).with_severity(sev)` |
| `severity_cell_style` helper in consumer code | Helper deleted; inline `Severity::from_thresholds + with_severity` at call site |

### Internal envision migration

- `envision/examples/beautiful_dashboard.rs` migrated to `theme.color(NamedColor::X)`
  in PR #473 already; no severity cells there. No change.
- envision's own components don't currently use severity coloring in cells. No
  internal migration needed.
- The four band-boundary parity test leadline writes during their D6+D9 migration
  doesn't need rewriting after D15: the `Severity::from_thresholds` math doesn't
  change; the consumer just uses `with_severity(sev)` instead of
  `with_style(CellStyle::Custom(theme.severity_style(sev)))`. The output style is
  identical.

---

## Files to touch

| File | Change |
|---|---|
| `src/component/cell.rs` | Add `Severity(Severity)` variant to `CellStyle` enum; add `#[non_exhaustive]` to enum; add `Cell::severity(text, sev)` constructor; add `Cell::with_severity(sev)` builder. |
| `src/component/cell.rs` (inline `#[cfg(test)] mod tests`) | Add tests: `Cell::severity` constructor, `Cell::with_severity` builder, last-call-wins precedence with `with_style`, severity preservation through typed-cell chain. |
| `src/component/table/render.rs` | Add `CellStyle::Severity(sev) => theme.severity_style(*sev)` arm to `cell_style_to_ratatui`. |
| `src/component/table/view_tests.rs` | Add: render snapshot for a row with mixed severity cells (Good + Bad + Critical) on Catppuccin Mocha; render assertion for `Severity(Critical)` includes `BOLD`; render assertion for disabled override (severity → dark-gray, no BOLD). |
| `CHANGELOG.md` | Additive entry under `[Unreleased]`. Note: `CellStyle::Severity(Severity)` added; `CellStyle` marked `#[non_exhaustive]`. |

---

## Tests

Five new tests across two files. Each pins a distinct invariant.

### Cell-type tests (inline `#[cfg(test)] mod tests` in `src/component/cell.rs`)

1. **`test_cell_severity_constructor`** — `Cell::severity("text", Severity::Bad)`
   produces a cell with `text() == "text"`, `style() == CellStyle::Severity(Bad)`,
   and `sort_key() == None`.

2. **`test_cell_with_severity_preserves_text_and_sort_key`** — start with
   `Cell::number(5.2).with_text("5.20x")`, chain `.with_severity(Severity::Bad)`,
   assert text is "5.20x", sort key is `SortKey::Number(5.2)` (preserved from G7
   typed constructor), style is `CellStyle::Severity(Bad)`.

3. **`test_cell_with_severity_overwrites_with_style`** — chain
   `Cell::new("x").with_style(CellStyle::Custom(Style::default().fg(Color::Magenta)))
   .with_severity(Severity::Critical)`. Assert style is `CellStyle::Severity(Critical)`;
   the prior `Custom` is gone (last-call-wins).

### Render-path tests (`src/component/table/view_tests.rs`)

4. **`test_severity_cell_renders_palette_color_with_critical_bold`** — render a
   single-row table with three cells: `Cell::severity("ok", Good)`,
   `Cell::severity("warn", Bad)`, `Cell::severity("crit", Critical)` on Catppuccin
   Mocha. Snapshot the rendered buffer. Assert the third cell's style includes
   `Modifier::BOLD`; the first two don't. Style fg colors match
   `theme.severity_color(sev)` for each band.

5. **`test_severity_cell_disabled_renders_dark_gray_no_bold`** — render the same
   row with `disabled: true` passed to the renderer. All three cells render with
   `fg = Color::DarkGray`, no BOLD. Disabled override wins over severity.

---

## Risks & open questions

### Risks

- **Breaking change footprint.** Adding `#[non_exhaustive]` + a new variant
  breaks any external code that pattern-matches `CellStyle` exhaustively without
  a `_` arm. Mitigation: search envision's own crate (no external matches found
  expected — `CellStyle` is consumed internally by `cell_style_to_ratatui` and
  externally by users who construct cells, not match on them). leadline migrates
  in their own PR after D15 lands. CHANGELOG flags the break under `[Unreleased]`.
- **Default theme severity collapse.** Already documented in D6+D9 design spec:
  on the `Default` theme, `Severity::Mild` and `Severity::Bad` both render as
  `Color::Yellow` because `Peach` and `Yellow` collapse there. `CellStyle::Severity`
  inherits this behavior — `Critical` stays distinguishable via `BOLD`. No new
  caveat introduced; the existing one applies.
- **Test snapshot fragility.** Snapshot test for the mixed-severity row pins the
  Catppuccin Mocha palette colors. If the Catppuccin palette ever changes upstream
  (it shouldn't — those are pinned to the official spec), the snapshot diff would
  surface. Acceptable: the existing `test_catppuccin_palette_pinned` test catches
  palette changes first.

### Decisions resolved during brainstorming

| Question | Resolution |
|---|---|
| Variant carrier shape | `Severity(Severity)` — value-typed, single argument. No extra parameters (e.g., disabled override). |
| `#[non_exhaustive]` on `CellStyle` | Add in same PR. One breaking change beats two. Matches `Severity` / `NamedColor` precedent from PR #473. |
| Constructor surface | Two: `Cell::severity(text, sev)` (mirrors `Cell::success/warning/error/muted` family) and `Cell::with_severity(sev)` (typed-cell builder, mirrors `with_text/with_sort_key/with_style`). Distinct ergonomic needs, no overlap. |
| Builder precedence | Last-call-wins with `with_style(...)`. Documented explicitly on `with_severity`'s docstring. Matches universal builder-pattern semantics. |
| Disabled handling | No new code path. Existing `disabled` override in `cell_style_to_ratatui` covers severity cells. |
| Render path location | `cell_style_to_ratatui` in `src/component/table/render.rs` — already takes `&Theme`, already runs after chrome resolution. No changes elsewhere. |
| Trait signature change | None. `TableRow::cells(&self) -> Vec<Cell>` unchanged. Severity awareness lives in the `Cell` value, not the trait. |

---

## Cadence

Same 4-PR cadence as G7 / D1 / chrome-ownership / D6+D9:

1. **PR α** — this design spec (`docs/superpowers/specs/2026-05-08-cellstyle-severity-design.md`).
2. **PR β** — implementation plan (`docs/superpowers/plans/2026-05-08-cellstyle-severity.md`).
3. **PR γ** — implementation. Single coherent breaking-change PR (variant + `#[non_exhaustive]`
   bundled). Additive on the constructor side; breaking on the enum side (small footprint).
4. **Tracking-doc PR** — mark D15 ✅ resolved in
   `docs/customer-feedback/2026-05-01-leadline-gaps.md`.

Flag leadline at spec-PR open for review.

---

## Related context

- leadline's customer-feedback inventory: `docs/customer-feedback/2026-05-01-leadline-gaps.md` (D15)
- leadline-side gaps tracking: `~/workspace/ryanoneill/rust-ai-explorations/notes/envision_gaps.md`
- Prior atomic-migration playbooks:
  - G1 + G3 + G7 spec/plan/impl (PRs #459 / #460 / #461)
  - D1 spec/plan/impl (PRs #463 / #464 / #465)
  - G2 + D2 + D11 chrome ownership (PRs #467 / #468 / #469)
  - D6 + D9 theme palette + severity helper (PRs #471 / #472 / #473 / #474)
- leadline call sites this redesign simplifies:
  - `leadline/src/baseline.rs` — `severity_cell_style` helper + inline `Theme::catppuccin_mocha()`
  - `leadline/src/per_op.rs` — same shape

This is the fourth coherent redesign drawing from leadline's May 2026 brief suite.
After this, D5+D14 (styled-text DX), G4+G5 (per-component style overrides), G6
(StyledInline composable), D7 (snapshot testing docs), and the small-rough-edges
punch list remain.
