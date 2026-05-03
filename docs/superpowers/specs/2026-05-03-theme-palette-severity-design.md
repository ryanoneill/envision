# Theme palette + severity helper — design spec

**Date:** 2026-05-03
**Status:** Approved; implementation plan to follow.
**Tracks:** leadline gaps **D6** + **D9** in `docs/customer-feedback/2026-05-01-leadline-gaps.md`
**Source brief:** `~/workspace/ryanoneill/rust-ai-explorations/notes/envision_theme_palette_severity_redesign.md`

---

## TL;DR

`Theme` exposes named *slots* (`focused`, `success`, `info`, `warning`, `error`, `muted`,
etc.) but no way to access the underlying palette colors by name. Consumers who want
non-slot colors — Lavender for brand identity, Peach for "yellow but worse," Teal for a
domain-specific label — have to reach past the abstraction and import raw `CATPPUCCIN_*`
constants. The moment they swap themes, the constants stop matching.

Worse: every consumer rebuilds the same severity-coloring helper (`severity_color_for_ratio`
+ `severity_status_style` + `severity_cell_style`), bound to Catppuccin and duplicated
across files. "Color this number by how bad it is" is the most common visual primitive in
any monitoring/profiling/status TUI; envision should own the color mapping.

Add `Theme::color(NamedColor) -> Color` for theme-aware named-color lookup. Add `Severity`
enum + `Severity::from_thresholds(value, &[(f64, Severity)]) -> Severity` bucketer +
`Theme::severity_color(Severity) -> Color` and `Theme::severity_style(Severity) -> Style`
accessors. Internally, every theme constructor populates a `Palette` struct with all 26
named-color entries — including nearest-equivalent mappings for themes (Nord, Dracula,
etc.) that don't natively have every Catppuccin palette member. Consumers always get a
color; no `Option<Color>`, no panics.

The change is purely additive. Existing `CATPPUCCIN_*` / `NORD*` / `DRACULA_*` `pub const`
exports stay in place with `#[deprecated]` for one transition window, then removed in a
follow-up cleanup PR.

---

## Goals

1. **Theme-aware named-color access.** Consumers can write `theme.color(NamedColor::Lavender)`
   and get the active theme's lavender (or its nearest equivalent), without importing any
   raw color constants.
2. **Always succeeds.** `theme.color(N)` returns `Color` for every variant of `NamedColor`
   on every shipped theme. No `Option`, no panic.
3. **Unified severity vocabulary.** `Severity` enum (`Good | Mild | Bad | Critical`) +
   `from_thresholds` bucketer + `Theme::severity_color` / `severity_style` accessors. Each
   theme inherits consistent severity coloring through its palette mapping.
4. **Custom-theme friendly.** Custom user themes can construct their own `Palette` directly;
   no central match table for envision to maintain.
5. **Theme stays trivially derive-able.** No function pointers in `Theme`; storage is plain
   data. `Clone + Debug + PartialEq` work as before.
6. **Purely additive.** No breaking changes. Existing slot accessors and existing color
   constants (with `#[deprecated]`) keep working through the migration.

## Non-goals

- **Removing the existing slot accessors.** `Theme::focused_style()`, `success_style()`,
  etc. stay. They're the right abstraction for components; the new APIs are for consumers.
- **Generic threshold types.** `Severity::from_thresholds` takes `f64`. Generalizing to
  `T: PartialOrd` is non-breaking and can land later if a real case demands it.
- **A `Severity::Neutral` variant.** "No severity" is `CellStyle::Default`. `Severity` is
  for the four-band gradient only.
- **StatusBar severity restoration.** Currently `severity_status_style` collapses two bands
  to `Warning` because StatusBar lacks a Peach slot. That's a separate G5 concern (per-item
  StatusBarItem color override). When G5 lands, StatusBar can resolve `Severity::Bad` via
  `theme.severity_color(Bad)` — no work needed in this spec.
- **Removing `CATPPUCCIN_*` / `NORD_*` / `DRACULA_*` constants.** Deprecated here, removed
  in a follow-up cleanup PR after leadline migrates.

---

## Design

### `NamedColor` enum (`src/theme/mod.rs`)

A flat enum of 26 palette names derived from Catppuccin Mocha (the most complete shipped
palette). Marked `#[non_exhaustive]` so envision can add palette names later without
breaking consumers.

```rust
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NamedColor {
    // Accent colors (warm)
    Rosewater, Flamingo, Pink, Mauve,
    Red, Maroon,
    Peach, Yellow,
    Green, Teal,
    // Accent colors (cool)
    Sky, Sapphire, Blue, Lavender,
    // Text + overlay (light → dark)
    Text, Subtext1, Subtext0,
    Overlay2, Overlay1, Overlay0,
    Surface2, Surface1, Surface0,
    Base, Mantle, Crust,
}
```

26 variants total. Future additions append; `#[non_exhaustive]` requires consumers to use
`_` arms in matches.

### `Palette` struct (`src/theme/mod.rs`)

Every theme stores a complete `Palette`. The struct has one field per `NamedColor` variant.

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Palette {
    pub rosewater: Color, pub flamingo: Color, pub pink: Color, pub mauve: Color,
    pub red: Color, pub maroon: Color,
    pub peach: Color, pub yellow: Color,
    pub green: Color, pub teal: Color,
    pub sky: Color, pub sapphire: Color, pub blue: Color, pub lavender: Color,
    pub text: Color, pub subtext1: Color, pub subtext0: Color,
    pub overlay2: Color, pub overlay1: Color, pub overlay0: Color,
    pub surface2: Color, pub surface1: Color, pub surface0: Color,
    pub base: Color, pub mantle: Color, pub crust: Color,
}
```

#### Why a struct field, not a function pointer or theme-kind enum

- `Theme` stays trivially `Clone + Debug + PartialEq` (no function pointers).
- New themes fill in their palette mapping at construction — no central match table for
  envision to keep in sync as themes are added.
- ~104 bytes per `Theme` instance — negligible storage cost.
- Custom user themes can provide their own `Palette` without modifying envision. The
  `Palette` struct's public fields support direct construction:

  ```rust
  let palette = Palette {
      rosewater: Color::Rgb(0xF5, 0xE0, 0xDC),
      flamingo: Color::Rgb(0xF2, 0xCD, 0xCD),
      // ... 24 more fields ...
  };
  ```

### `Theme::color(NamedColor) -> Color`

```rust
impl Theme {
    pub fn color(&self, named: NamedColor) -> Color {
        match named {
            NamedColor::Rosewater => self.palette.rosewater,
            NamedColor::Flamingo  => self.palette.flamingo,
            NamedColor::Pink      => self.palette.pink,
            NamedColor::Mauve     => self.palette.mauve,
            NamedColor::Red       => self.palette.red,
            NamedColor::Maroon    => self.palette.maroon,
            NamedColor::Peach     => self.palette.peach,
            NamedColor::Yellow    => self.palette.yellow,
            NamedColor::Green     => self.palette.green,
            NamedColor::Teal      => self.palette.teal,
            NamedColor::Sky       => self.palette.sky,
            NamedColor::Sapphire  => self.palette.sapphire,
            NamedColor::Blue      => self.palette.blue,
            NamedColor::Lavender  => self.palette.lavender,
            NamedColor::Text      => self.palette.text,
            NamedColor::Subtext1  => self.palette.subtext1,
            NamedColor::Subtext0  => self.palette.subtext0,
            NamedColor::Overlay2  => self.palette.overlay2,
            NamedColor::Overlay1  => self.palette.overlay1,
            NamedColor::Overlay0  => self.palette.overlay0,
            NamedColor::Surface2  => self.palette.surface2,
            NamedColor::Surface1  => self.palette.surface1,
            NamedColor::Surface0  => self.palette.surface0,
            NamedColor::Base      => self.palette.base,
            NamedColor::Mantle    => self.palette.mantle,
            NamedColor::Crust     => self.palette.crust,
        }
    }
}
```

The 26-arm match is verbose but trivial; the compiler exhaustiveness check catches missing
arms when new `NamedColor` variants are added (the `#[non_exhaustive]` attribute applies to
external consumers; within the crate the match must still be exhaustive).

### Per-theme palette mapping

Each shipped theme's constructor populates its `palette` field with every `NamedColor`
mapped to a sensible color from its own design. The mappings are documented per theme.

#### Catppuccin Mocha — 1:1 mapping

The source palette. Direct mapping; every `NamedColor` variant has an exact Catppuccin
equivalent.

```rust
pub fn catppuccin_mocha() -> Self {
    Self {
        // ... existing flat fields unchanged ...
        palette: Palette {
            rosewater: CATPPUCCIN_ROSEWATER,
            flamingo:  CATPPUCCIN_FLAMINGO,
            // ... 24 more fields, exact 1:1 mapping ...
        },
    }
}
```

#### Nord — nearest-equivalent mapping

Nord's 16-color palette doesn't have a "Peach" or "Lavender" by those names. Maps to
Nord's nearest-shade equivalents. Documented in the `Theme::nord()` docstring.

Approximate mapping (final values determined during implementation, subject to color
inspection):
- `Peach` → Nord12 (orange `#D08770`)
- `Lavender` → Nord15 (purple `#B48EAD`)
- `Mauve` → Nord15 (closest purple)
- `Pink` → Nord15 (no native pink; purple closest)
- `Rosewater` → Nord4 (light surface) or Nord13 (yellow) depending on visual fit
- `Teal` → Nord7 (frost teal `#8FBCBB`)
- `Sky` → Nord8 (light blue `#88C0D0`)
- `Sapphire` → Nord9 (mid blue `#81A1C1`)
- `Blue` → Nord10 (deep blue `#5E81AC`)
- Surface tones → Nord0–Nord3
- Text tones → Nord4–Nord6

#### Dracula — nearest-equivalent mapping

Dracula's palette has named colors (Foreground, Background, Comment, Cyan, Green, Orange,
Pink, Purple, Red, Yellow). Mapping:
- `Peach` → Orange `#FFB86C`
- `Lavender` → Purple `#BD93F9`
- `Mauve` → Purple `#BD93F9`
- `Pink` → Pink `#FF79C6`
- ... etc.

#### Solarized Dark — nearest-equivalent mapping

Solarized Dark uses base colors (base03, base02, ..., yellow, orange, red, magenta, violet,
blue, cyan, green). Mapping:
- `Peach` → orange `#CB4B16`
- `Lavender` → violet `#6C71C4`
- `Mauve` → magenta `#D33682`
- ... etc.

#### Gruvbox Dark — nearest-equivalent mapping

Gruvbox Dark uses gruvbox-named colors. Mapping:
- `Peach` → gruvbox-orange `#FE8019`
- `Lavender` → gruvbox-purple `#D3869B`
- ... etc.

#### Default — basic-Color collapse mapping

The `Default` theme uses ratatui's basic `Color` enum (Reset, Yellow, Red, Cyan, etc.) for
maximum terminal compatibility. Many `NamedColor` variants collapse to the same basic
`Color`:

- `Peach`, `Yellow`, `Maroon` → `Color::Yellow`
- `Lavender`, `Mauve`, `Pink`, `Sapphire` → `Color::Magenta`
- `Red`, `Flamingo`, `Rosewater` → `Color::Red`
- `Green`, `Teal` → `Color::Green`
- `Sky`, `Blue` → `Color::Blue`
- Surface tones → `Color::Black` / `Color::DarkGray`
- Text tones → `Color::White` / `Color::Gray`

The collapse is intentional and documented on the `Default` theme: in raw-VT100
environments the 16-color basic palette is what's available. Consumers wanting full
palette fidelity should use Catppuccin Mocha or another full-palette theme. The
`Theme::default()` docstring will note this explicitly.

### `Severity` enum + `from_thresholds`

```rust
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Severity {
    Good,
    Mild,
    Bad,
    Critical,
}

impl Severity {
    /// Pick a `Severity` by linear thresholds.
    ///
    /// Thresholds are sorted ascending; the first threshold the value is
    /// *less than* wins. Values at or above all thresholds get `Critical`.
    ///
    /// # Behavior on unsorted input
    ///
    /// First-match-wins: the function iterates thresholds in slice order
    /// and returns at the first cutoff `value < cutoff` succeeds. Callers
    /// passing unsorted slices get well-defined but possibly counter-
    /// intuitive results. Sort ascending for predictable bucketing.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use envision::theme::Severity;
    ///
    /// let thresholds = [
    ///     (1.0, Severity::Good),
    ///     (3.0, Severity::Mild),
    ///     (10.0, Severity::Bad),
    /// ];
    ///
    /// assert_eq!(Severity::from_thresholds(0.5, &thresholds), Severity::Good);
    /// assert_eq!(Severity::from_thresholds(2.0, &thresholds), Severity::Mild);
    /// assert_eq!(Severity::from_thresholds(5.0, &thresholds), Severity::Bad);
    /// assert_eq!(Severity::from_thresholds(20.0, &thresholds), Severity::Critical);
    /// ```
    pub fn from_thresholds(value: f64, thresholds: &[(f64, Severity)]) -> Severity {
        for (cutoff, sev) in thresholds {
            if value < *cutoff {
                return *sev;
            }
        }
        Severity::Critical
    }
}
```

### `Theme::severity_color` / `severity_style`

```rust
impl Theme {
    /// Color matching the severity band — for use in cells, banners, custom inline runs.
    /// Maps to the theme's palette via `NamedColor`.
    pub fn severity_color(&self, sev: Severity) -> Color {
        match sev {
            Severity::Good     => self.color(NamedColor::Green),
            Severity::Mild     => self.color(NamedColor::Yellow),
            Severity::Bad      => self.color(NamedColor::Peach),
            Severity::Critical => self.color(NamedColor::Red),
        }
    }

    /// Full `Style` (color + reasonable defaults like bold for Critical).
    /// Drop-in for `Cell::with_style(CellStyle::Custom(...))` and
    /// `StyledInline::Styled { style, ... }` sites.
    pub fn severity_style(&self, sev: Severity) -> Style {
        let style = Style::default().fg(self.severity_color(sev));
        if sev == Severity::Critical {
            style.add_modifier(Modifier::BOLD)
        } else {
            style
        }
    }
}
```

`severity_color` reuses the palette infrastructure — themes don't store a separate severity
table. Catppuccin gets the exact band gradient (Green → Yellow → Peach → Red); other themes
get their nearest equivalents naturally via their palette mapping.

### `severity_style` modifier rationale

`Critical` adds `BOLD` to its style. `Good` / `Mild` / `Bad` use color only. The visual
hierarchy is intentional: Critical events should stand out beyond color alone (handles
color-blind users, low-contrast terminals, partial color rendering). The two-method API
(`severity_color` for color-only, `severity_style` for color+modifier) lets consumers pick.

### Deprecation of raw constants

Existing `pub const` color constants stay in place with `#[deprecated]`:

```rust
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Peach) instead — palette-aware and theme-swap-safe"
)]
pub const CATPPUCCIN_PEACH: Color = Color::Rgb(250, 179, 135);
```

Same treatment for `CATPPUCCIN_*`, `NORD*`, `DRACULA_*`, `SOLARIZED_*`, `GRUVBOX_*`. The
constants stay accessible during the transition window (one minor version), then removed
in a follow-up cleanup PR after leadline migrates.

---

## Migration

### Consumer-side: replacing raw constants

```rust
// Before
use envision::theme::{CATPPUCCIN_GREEN, CATPPUCCIN_PEACH, CATPPUCCIN_RED, CATPPUCCIN_YELLOW};
// ... later in code ...
let c = match ratio {
    r if r < 1.0 => CATPPUCCIN_GREEN,
    r if r < 3.0 => CATPPUCCIN_YELLOW,
    r if r < 10.0 => CATPPUCCIN_PEACH,
    _ => CATPPUCCIN_RED,
};

// After — uses Severity + from_thresholds + theme.severity_color
use envision::theme::Severity;
let sev = Severity::from_thresholds(ratio, &[
    (1.0,  Severity::Good),
    (3.0,  Severity::Mild),
    (10.0, Severity::Bad),
]);
let c = theme.severity_color(sev);
```

### Consumer-side: replacing severity helpers

```rust
// Before — leadline/src/app.rs:467–487, baseline.rs:69–77, per_op.rs:64–72
fn severity_color_for_ratio(ratio: f64) -> Color { /* ... */ }
fn severity_status_style(ratio: f64) -> StatusBarStyle { /* ... */ }
fn severity_cell_style(ratio: f64) -> CellStyle { /* ... */ }

// After — three helpers DELETED, replaced inline with:
let sev = Severity::from_thresholds(ratio, &[
    (1.0,  Severity::Good),
    (3.0,  Severity::Mild),
    (10.0, Severity::Bad),
]);
let cell_style = CellStyle::Custom(theme.severity_style(sev));
```

The threshold *table* stays in consumer code because the bands are domain-specific (a ratio
of 3.0 is "Mild" for GPU profiling, "Bad" for network latency). The *color mapping* moves
into envision.

### Migration table

| Old | New |
|---|---|
| `use envision::theme::CATPPUCCIN_PEACH;` + `Color::Rgb(...)` constant | `theme.color(NamedColor::Peach)` |
| Hand-rolled `severity_color_for_ratio(value) -> Color` | `Severity::from_thresholds(value, &[...])` + `theme.severity_color(sev)` |
| Hand-rolled `severity_cell_style(value) -> CellStyle` | `CellStyle::Custom(theme.severity_style(sev))` |
| `pub const CATPPUCCIN_*` import | `#[deprecated]`; consumers see warning, migrate to `theme.color(...)` |

### Internal envision migration

- envision's own components don't currently use raw `CATPPUCCIN_*` / `NORD_*` constants in
  rendering paths (they use slot accessors like `theme.success_style()`). No breaking
  changes to existing components.
- Examples that demonstrate theme palette directly may opt into the new API as a teaching
  vehicle, but no example *requires* migration.

---

## Files to touch

| File | Change |
|---|---|
| `src/theme/mod.rs` | Add `NamedColor` enum, `Palette` struct, `Severity` enum + `from_thresholds`. Add `palette: Palette` field to `Theme`. Add `Theme::color`, `severity_color`, `severity_style` methods. Each theme constructor (`default`, `nord`, `dracula`, `solarized_dark`, `gruvbox_dark`, `catppuccin_mocha`) populates its `palette` field with all 26 entries per the documented per-theme mapping. |
| `src/theme/catppuccin.rs` | Mark all `pub const CATPPUCCIN_*` with `#[deprecated(since = "0.17.0", note = "use theme.color(NamedColor::X) instead")]`. Don't delete. |
| `src/theme/<other>.rs` (Nord/Dracula/Solarized/Gruvbox color-constant modules) | Same deprecation treatment if the modules exist — verify during plan-writing. |
| `src/theme/tests.rs` | Add 9 new tests (palette completeness per theme, NamedColor lookup, Severity::from_thresholds with band boundaries / empty / unsorted, severity_color per theme, severity_style critical-is-bold, severity_style color-matches-severity_color, doc test). |
| `src/lib.rs` | Re-export `NamedColor`, `Palette`, `Severity` from the crate root if `Theme` is already re-exported. |
| `src/lib.rs` (prelude) | Add `NamedColor`, `Severity` to the prelude alongside `Theme`. |
| `CHANGELOG.md` | Additive entry: `NamedColor`, `Palette`, `Severity` APIs added. `CATPPUCCIN_*` (and any other theme-specific) constants deprecated. |

---

## Tests

Nine new tests in `src/theme/tests.rs`:

1. **`test_palette_completeness_per_theme`** — for each of the 6 shipped themes, every
   `NamedColor` variant returns a non-`Color::Reset` color (or specifically the documented
   fallback, e.g. `Color::Reset` for `Default`'s background-tone slots). Pin per-theme
   palette table values; intentional palette changes show up as test diffs.

2. **`test_namedcolor_lookup_returns_palette_entry`** — `theme.color(NamedColor::Lavender)`
   matches `theme.palette.lavender`. Same for a sample of other variants. Round-trip the
   accessor.

3. **`test_namedcolor_distinct_per_theme`** — different `NamedColor` variants return
   different colors on Catppuccin (e.g., `Peach != Yellow != Red`). Pins palette
   distinctness; would fail if a theme accidentally collapsed two slots.

4. **`test_severity_from_thresholds_band_boundaries`** — value just below threshold gets
   the corresponding band; value at or above the threshold falls through to the next band
   (or `Critical` if past all). Specifically tests the four canonical points around each
   threshold.

5. **`test_severity_from_thresholds_empty`** — empty threshold slice → always `Critical`.

6. **`test_severity_from_thresholds_unsorted_first_match_wins`** — pins the documented
   first-match-wins behavior. Passing unsorted thresholds returns whatever the iteration
   order produces.

7. **`test_severity_color_per_theme`** — Catppuccin: `Good=Green, Mild=Yellow, Bad=Peach,
   Critical=Red` (their palette colors). Other themes: their palette equivalents. Pin the
   mapping per theme.

8. **`test_severity_style_critical_is_bold`** — `severity_style(Critical)` includes
   `Modifier::BOLD`; other variants don't.

9. **`test_severity_style_fg_matches_severity_color`** — `severity_style(sev).fg ==
   Some(severity_color(sev))` for all four severities.

Plus the doc test on `Severity::from_thresholds` showing the threshold-table → severity →
style flow.

---

## Risks & open questions

### Risks

- **Per-theme palette mapping subjectivity.** Mapping Nord's 16-color palette to all 26
  Catppuccin names involves judgment calls (which Nord blue is closer to "Sapphire" vs
  "Blue"?). Mitigation: ship leadline-validated mappings; consumers who disagree can
  construct a custom theme. The test suite pins current values; deliberate changes are
  reviewed via test-diff.
- **Default theme palette collapse.** Multiple `NamedColor` variants map to the same basic
  `Color` on the `Default` theme. Consumer code that depends on palette distinctness will
  produce visually-identical output for distinct variants under `Default`. Notably this
  affects `severity_color`: on `Default`, `Mild` and `Bad` both render as `Color::Yellow`
  because `Peach` and `Yellow` collapse to the same basic color. The four-band gradient
  effectively becomes three-band (Green / Yellow / Yellow / Red) — color-only severity is
  ambiguous. `severity_style(Critical)` still adds `BOLD`, so Critical stays distinguishable
  even on `Default`. Mitigation: documented on the `Default` theme's `palette` field with
  an explicit note; consumers wanting full fidelity choose Catppuccin or another
  full-palette theme.
- **`Palette` struct add as breaking-shape change?** Adding a non-public field to `Theme`
  is technically a breaking change if any consumer constructs `Theme` via struct-literal
  (rather than `Theme::nord()` etc.). Mitigation: `Theme` already has many fields and
  envision is pre-1.0; the struct literal is documented as an internal pattern. Custom
  themes using struct literals will need to add the `palette: Palette { ... }` field.
  Worth a CHANGELOG note. (This applies only if the existing `Theme` struct's other fields
  are pub — verify during implementation.)

### Decisions resolved during brainstorming

| Question | Resolution |
|---|---|
| Palette completeness vs minimalism | Ship full Catppuccin palette upfront (26 NamedColor variants) |
| Themes that lack a palette name (Q2) | Per-theme `palette: Palette` field populated at construction with nearest-equivalent mappings; `theme.color(N)` always returns a sensible color |
| Severity with non-numeric values | `f64` only; `T: PartialOrd` deferred |
| `Severity::Neutral` variant | Skip — `CellStyle::Default` is the no-severity case |
| Naming | `Good / Mild / Bad / Critical` |
| StatusBar `Bad → Peach` restoration | Out of scope (depends on G5 — separate brief) |
| Internal storage shape | `Palette` struct field (not function pointer, not theme-kind enum) |
| `severity_style` modifiers | `BOLD` for Critical only; color-only for other variants. Two-method API: `severity_color` for color, `severity_style` for color+modifier. |
| Deprecation of raw constants | `#[deprecated]` for one transition window; remove in follow-up cleanup PR |

### Plan-time docstring additions (per leadline review)

These are noted here so they don't get lost; they are docstring-only, no design impact:

1. **Default theme collapse note.** Docstring on `Theme::default()` and on the
   `Default`-flavored `Palette` mapping notes "many `NamedColor` variants collapse to the
   same basic `Color` in this theme; for full palette fidelity use Catppuccin Mocha or
   another full-palette theme."
2. **Palette correctness vs completeness.** Docstring on `test_palette_completeness_per_theme`
   explicitly notes the test pins current values, so any per-theme palette change shows up
   in test diffs (intentional or accidental).
3. **Custom-theme construction example.** Docstring on `Palette` shows the direct-
   construction pattern (`Palette { rosewater: Color::Rgb(...), ... }`) so consumers who
   want a custom theme have a worked example.

---

## Cadence

Same 4-PR cadence as G7 / D1 / chrome:

1. **PR α** — this design spec (`docs/superpowers/specs/2026-05-03-theme-palette-severity-design.md`).
2. **PR β** — implementation plan (`docs/superpowers/plans/2026-05-03-theme-palette-severity.md`).
3. **PR γ** — implementation. **Purely additive** (no breaking changes to existing
   consumers; raw constants get `#[deprecated]` only). Single coherent PR.
4. **Tracking-doc PR** — mark D6 + D9 ✅ resolved in
   `docs/customer-feedback/2026-05-01-leadline-gaps.md`.

---

## Related context

- leadline's customer-feedback inventory:
  `docs/customer-feedback/2026-05-01-leadline-gaps.md` (D6, D9)
- leadline-side gaps tracking:
  `~/workspace/ryanoneill/rust-ai-explorations/notes/envision_gaps.md`
- Prior atomic-migration playbooks:
  - G1 + G3 + G7 spec/plan/impl (PRs #459/#460/#461)
  - D1 spec/plan/impl (PRs #463/#464/#465)
  - G2 + D2 + D11 chrome ownership (PRs #467/#468/#469)
- leadline call sites this redesign simplifies:
  - `leadline/src/app.rs:33–34` (raw imports), `467–487` (severity fns)
  - `leadline/src/baseline.rs:19, 48, 69–77`
  - `leadline/src/per_op.rs:15, 45, 64–72`

This is the third coherent redesign drawing from leadline's May 2026 brief suite. After
this, D5 (styled-line primitive), D7 (snapshot testing docs), G4 (PaneLayout per-pane title
style), G5 (per-item StatusBar color override), G6 (StyledInline composable styles) plus
the small-rough-edges punch list remain.
