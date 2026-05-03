# Theme palette + severity helper Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `NamedColor`/`Palette`/`Severity` APIs to the `Theme` module so consumers can access named palette colors theme-aware (`theme.color(NamedColor::Lavender)`) and bucket numeric values into a four-band severity gradient (`theme.severity_color(Severity::Bad)`), eliminating the need to import raw `CATPPUCCIN_*` / `NORD_*` constants.

**Architecture:** Purely additive. Add `NamedColor` enum (26 variants, `#[non_exhaustive]`), `Palette` struct (one `Color` field per variant), `Severity` enum (`Good | Mild | Bad | Critical`, `#[non_exhaustive]`), and three new `Theme` methods (`color`, `severity_color`, `severity_style`). Each shipped theme constructor populates its `palette` field with all 26 entries; non-Catppuccin themes use documented nearest-equivalent mappings. Existing `pub const` color constants stay in place with `#[deprecated]` for one transition window.

**Tech Stack:** Rust 1.85+, ratatui 0.29 (`Color`, `Style`, `Modifier`), envision theme module.

**Spec:** `docs/superpowers/specs/2026-05-03-theme-palette-severity-design.md` (PR #471)

---

## Pre-execution gotchas

- **Signed commits required.** Project rule. If `git commit -S` fails, ask the user; never bypass with `--no-gpg-sign`.
- **File size cap.** `src/theme/mod.rs` is currently 633 lines. Naively adding everything in-place would push it past 1200 lines. The plan therefore puts the new types (`NamedColor`, `Palette`, `Severity`) and the new `Theme` accessor methods (`color`, `severity_color`, `severity_style`) in a **new file** `src/theme/palette.rs` from the start. Rust allows multiple `impl Theme { ... }` blocks across modules within the same crate, so the new methods live cleanly in `palette.rs` alongside the types they use. Per-theme palette population blocks stay inside each theme constructor in `mod.rs` (they reference theme-specific constants). Projected final sizes: `mod.rs` ~820 lines, `palette.rs` ~440 lines — both comfortably under the 1000-line cap.
- **`#[non_exhaustive]` matches.** Within the crate, exhaustive `match named { ... }` on `NamedColor` is required (the `#[non_exhaustive]` attribute applies only to external consumers). Compiler will catch missing arms when new variants are added.
- **Severity::Critical must add BOLD; other variants must NOT.** Spec is explicit. Test pins this behavior.
- **`from_thresholds` semantics: first-match-wins**, not "find smallest cutoff that exceeds value." For sorted ascending input these are equivalent. For unsorted input the test pins the documented behavior.
- **`Color::Reset` is a valid palette value** for some `Default` theme slots (background tones). Test 1 (`palette_completeness_per_theme`) must accept `Color::Reset` for those specific slots, not blanket-reject it.
- **CHANGELOG goes in `## [Unreleased]` section.** Already has Chrome ownership entries; append a new sub-heading.
- **No `pub const` re-exports of `CATPPUCCIN_*` etc. at crate root.** Verified via `grep -n "CATPPUCCIN\|NORD\|DRACULA" src/lib.rs` (returns nothing). Deprecation lives only in `src/theme/mod.rs` and `src/theme/catppuccin.rs`.
- **Prelude check.** `src/lib.rs:449` has `pub use crate::theme::Theme;` in the prelude. Add `NamedColor` and `Severity` next to it; `Palette` only at the crate root since users construct it explicitly (not common enough for prelude).

---

## File Structure

| File | Responsibility | Lines (before → after) |
|---|---|---|
| `src/theme/palette.rs` (NEW) | `NamedColor`, `Palette`, `Severity` types; `impl Theme` block with `color`, `severity_color`, `severity_style` methods | 0 → ~440 |
| `src/theme/mod.rs` | `Theme` struct + constructors + style helpers + `pub mod palette;` re-export + per-theme `palette: Palette { ... }` population blocks | 633 → ~820 |
| `src/theme/catppuccin.rs` | Catppuccin Mocha `pub const` color constants | 86 → 86 (only `#[deprecated]` attrs added) |
| `src/theme/tests.rs` | Theme unit tests | 314 → ~480 (9 new tests + helpers) |
| `src/lib.rs` | Crate root re-exports + prelude | unchanged structure; adds 2 lines for `NamedColor, Palette, Severity` re-exports + 1 line for prelude |
| `CHANGELOG.md` | Release notes | adds ~25-line entry under `[Unreleased]` |

The split between `palette.rs` (new types + new methods) and `mod.rs` (existing `Theme` struct + per-theme constructor palette population) keeps each file focused and well under the 1000-line cap. The per-theme palette blocks must stay in `mod.rs` because they reference theme-specific constants (`NORD0`, `DRACULA_PURPLE`, etc.) that live there.

---

## Build + test setup verification

Run once before Task 1 to confirm the environment is healthy:

```bash
cargo build --all-features 2>&1 | tail -5
cargo nextest run --all-features --no-fail-fast theme:: 2>&1 | tail -10
```

Expected: build succeeds; 28 existing tests in `theme::tests` pass.

---

## Task 0: Create `src/theme/palette.rs` scaffold

**Files:**
- Create: `src/theme/palette.rs`
- Modify: `src/theme/mod.rs`

- [ ] **Step 1: Create the new module file with header**

Create `src/theme/palette.rs` with this content:

```rust
//! Named-color palette and severity helper for `Theme`.
//!
//! This module adds three new public types — [`NamedColor`], [`Palette`],
//! and [`Severity`] — plus three new methods on [`Theme`] (`color`,
//! `severity_color`, `severity_style`). Together they let consumers access
//! palette colors by name and bucket numeric values into a four-band
//! severity gradient without reaching for raw color constants.
//!
//! See the [theme module documentation](super) for an overview.

use ratatui::style::{Color, Modifier, Style};

use super::Theme;

// Subsequent tasks add NamedColor, Palette, Severity, and the impl Theme block
// for color / severity_color / severity_style.
```

- [ ] **Step 2: Wire the module into `src/theme/mod.rs`**

In `src/theme/mod.rs`, immediately after the existing `pub mod catppuccin;` / `pub use catppuccin::*;` block (around line 42-43), add:

```rust
pub mod palette;
// Re-exports added progressively as the types land:
//   Task 1 → adds Severity
//   Task 3 → adds NamedColor
//   Task 4 → adds Palette
```

(No `pub use` line yet — the new module exports nothing in this task. Subsequent tasks add the `pub use palette::Severity;` line, then extend it to `{NamedColor, Severity}`, then to `{NamedColor, Palette, Severity}`.)

- [ ] **Step 3: Verify the scaffold compiles**

Run: `cargo build --all-features 2>&1 | tail -10`
Expected: clean build (the new module is empty; rustc accepts it).

- [ ] **Step 4: Commit**

```bash
git add src/theme/palette.rs src/theme/mod.rs
git commit -S -m "Add src/theme/palette.rs module scaffold

Empty new module gated by 'pub mod palette;' in mod.rs. Subsequent commits
add NamedColor, Palette, Severity, and the impl Theme accessors. Re-export
of the new types is commented out and progressively uncommented per task."
```

---

## Task 1: Add `Severity` enum (failing test)

**Files:**
- Modify: `src/theme/tests.rs` (append at end of file)
- Modify: `src/theme/palette.rs`
- Modify: `src/theme/mod.rs` (uncomment the `Severity` import)

- [ ] **Step 1: Write the failing test**

Append to `src/theme/tests.rs`:

```rust
#[test]
fn test_severity_enum_variants() {
    // Pin the four severity variants and their ordering.
    let _good = Severity::Good;
    let _mild = Severity::Mild;
    let _bad = Severity::Bad;
    let _critical = Severity::Critical;
    // Pin Copy/Clone/Eq so consumers can destructure freely.
    let s = Severity::Bad;
    let s2 = s;
    assert_eq!(s, s2);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo nextest run --all-features theme::tests::test_severity_enum_variants 2>&1 | tail -10`
Expected: FAIL with "cannot find type `Severity` in this scope" or "cannot find value `Severity` in this scope".

- [ ] **Step 3: Add `Severity` enum to `src/theme/palette.rs`**

Append to `src/theme/palette.rs` after the existing `use` lines:

```rust
// =============================================================================
// Severity
// =============================================================================

/// A four-band severity gradient for value-based coloring (good → mild → bad → critical).
///
/// `Severity` provides a unified vocabulary for "color this number by how bad it is" —
/// the most common visual primitive in monitoring, profiling, and status dashboards.
/// Pair with [`Severity::from_thresholds`] to bucket a numeric value, then pass to
/// [`Theme::severity_color`] or [`Theme::severity_style`] for theme-aware coloring.
///
/// `#[non_exhaustive]` so envision can add severity bands later without breaking
/// downstream `match` arms.
///
/// # Example
///
/// ```rust
/// use envision::theme::{Severity, Theme};
///
/// let theme = Theme::catppuccin_mocha();
/// let ratio = 5.2;
/// let sev = Severity::from_thresholds(ratio, &[
///     (1.0,  Severity::Good),
///     (3.0,  Severity::Mild),
///     (10.0, Severity::Bad),
/// ]);
/// let style = theme.severity_style(sev);
/// // Use `style` in a Cell or StyledInline for value-coloring.
/// # let _ = style;
/// ```
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Severity {
    /// Healthy band — typically rendered green.
    Good,
    /// Slightly elevated band — typically rendered yellow.
    Mild,
    /// Concerning band — typically rendered peach/orange.
    Bad,
    /// Critical band — typically rendered red and bold.
    Critical,
}
```

- [ ] **Step 4: Uncomment the `Severity` import in `mod.rs`**

In `src/theme/mod.rs`, change the import line from:

```rust
// pub use palette::{NamedColor, Palette, Severity};
```

to:

```rust
pub use palette::Severity;
```

`NamedColor` and `Palette` are added (and uncommented) in Tasks 3 and 4.

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo nextest run --all-features theme::tests::test_severity_enum_variants 2>&1 | tail -10`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/theme/palette.rs src/theme/mod.rs src/theme/tests.rs
git commit -S -m "Add Severity enum (Good/Mild/Bad/Critical, #[non_exhaustive])

Skeleton type for the value-bucketing severity vocabulary. Lives in
src/theme/palette.rs; re-exported from src/theme/mod.rs. No methods or
theme integration yet; subsequent commits add from_thresholds and the
Theme::severity_color / severity_style accessors."
```

---

## Task 2: Add `Severity::from_thresholds` (failing tests)

**Files:**
- Modify: `src/theme/tests.rs`
- Modify: `src/theme/palette.rs`

- [ ] **Step 1: Write the failing tests**

Append to `src/theme/tests.rs`:

```rust
#[test]
fn test_severity_from_thresholds_band_boundaries() {
    let thresholds = [
        (1.0, Severity::Good),
        (3.0, Severity::Mild),
        (10.0, Severity::Bad),
    ];
    // Below first cutoff: Good.
    assert_eq!(Severity::from_thresholds(0.5, &thresholds), Severity::Good);
    assert_eq!(Severity::from_thresholds(0.999, &thresholds), Severity::Good);
    // At cutoff falls through to next band (Mild).
    assert_eq!(Severity::from_thresholds(1.0, &thresholds), Severity::Mild);
    // Inside Mild range.
    assert_eq!(Severity::from_thresholds(2.0, &thresholds), Severity::Mild);
    assert_eq!(Severity::from_thresholds(2.999, &thresholds), Severity::Mild);
    // At Mild cutoff falls through to Bad.
    assert_eq!(Severity::from_thresholds(3.0, &thresholds), Severity::Bad);
    assert_eq!(Severity::from_thresholds(5.0, &thresholds), Severity::Bad);
    // At Bad cutoff falls through to Critical (default).
    assert_eq!(Severity::from_thresholds(10.0, &thresholds), Severity::Critical);
    assert_eq!(Severity::from_thresholds(20.0, &thresholds), Severity::Critical);
}

#[test]
fn test_severity_from_thresholds_empty() {
    // Empty threshold slice: every value is Critical.
    assert_eq!(Severity::from_thresholds(0.0, &[]), Severity::Critical);
    assert_eq!(Severity::from_thresholds(-1.0, &[]), Severity::Critical);
    assert_eq!(Severity::from_thresholds(1e9, &[]), Severity::Critical);
}

#[test]
fn test_severity_from_thresholds_unsorted_first_match_wins() {
    // Documented first-match-wins: iteration is in slice order, not by sorted cutoff.
    // Unsorted thresholds give well-defined but possibly counter-intuitive results.
    let unsorted = [
        (10.0, Severity::Bad),
        (1.0,  Severity::Good),
        (3.0,  Severity::Mild),
    ];
    // value=2.0: first cutoff is 10.0; 2.0 < 10.0, so returns Bad immediately.
    assert_eq!(Severity::from_thresholds(2.0, &unsorted), Severity::Bad);
    // value=15.0: 15.0 < 10.0? no. 15.0 < 1.0? no. 15.0 < 3.0? no. Critical default.
    assert_eq!(Severity::from_thresholds(15.0, &unsorted), Severity::Critical);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo nextest run --all-features theme::tests::test_severity_from_thresholds 2>&1 | tail -10`
Expected: FAIL with "no associated function or method named `from_thresholds`".

- [ ] **Step 3: Add `from_thresholds` impl in `src/theme/palette.rs`**

Append immediately after the `pub enum Severity { ... }` block:

```rust
impl Severity {
    /// Pick a `Severity` by linear thresholds.
    ///
    /// Thresholds are evaluated in slice order: the first `(cutoff, severity)` entry where
    /// `value < cutoff` wins. Values at or above all cutoffs return `Severity::Critical`.
    ///
    /// # Sorting
    ///
    /// Pass thresholds sorted ascending by cutoff for predictable bucketing. Unsorted
    /// input is well-defined (first-match-wins) but typically counter-intuitive.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use envision::theme::Severity;
    ///
    /// let thresholds = [
    ///     (1.0,  Severity::Good),
    ///     (3.0,  Severity::Mild),
    ///     (10.0, Severity::Bad),
    /// ];
    ///
    /// assert_eq!(Severity::from_thresholds(0.5,  &thresholds), Severity::Good);
    /// assert_eq!(Severity::from_thresholds(2.0,  &thresholds), Severity::Mild);
    /// assert_eq!(Severity::from_thresholds(5.0,  &thresholds), Severity::Bad);
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

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo nextest run --all-features theme::tests::test_severity_from_thresholds 2>&1 | tail -10`
Expected: 3 tests PASS.

- [ ] **Step 5: Run doc test**

Run: `cargo test --all-features --doc theme::Severity 2>&1 | tail -10`
Expected: doc test PASSes.

- [ ] **Step 6: Commit**

```bash
git add src/theme/palette.rs src/theme/tests.rs
git commit -S -m "Add Severity::from_thresholds (first-match-wins bucketer)

Iterates thresholds in slice order; returns the first severity whose
cutoff exceeds the value; falls through to Critical if past all cutoffs.
Doc test demonstrates threshold-table -> severity flow."
```

---

## Task 3: Add `NamedColor` enum (failing test)

**Files:**
- Modify: `src/theme/tests.rs`
- Modify: `src/theme/palette.rs`
- Modify: `src/theme/mod.rs` (extend the import)

- [ ] **Step 1: Write the failing test**

Append to `src/theme/tests.rs`:

```rust
#[test]
fn test_named_color_enum_variants() {
    // Pin all 26 NamedColor variants — the spec's complete Catppuccin-derived palette.
    let variants = [
        NamedColor::Rosewater,
        NamedColor::Flamingo,
        NamedColor::Pink,
        NamedColor::Mauve,
        NamedColor::Red,
        NamedColor::Maroon,
        NamedColor::Peach,
        NamedColor::Yellow,
        NamedColor::Green,
        NamedColor::Teal,
        NamedColor::Sky,
        NamedColor::Sapphire,
        NamedColor::Blue,
        NamedColor::Lavender,
        NamedColor::Text,
        NamedColor::Subtext1,
        NamedColor::Subtext0,
        NamedColor::Overlay2,
        NamedColor::Overlay1,
        NamedColor::Overlay0,
        NamedColor::Surface2,
        NamedColor::Surface1,
        NamedColor::Surface0,
        NamedColor::Base,
        NamedColor::Mantle,
        NamedColor::Crust,
    ];
    assert_eq!(variants.len(), 26);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo nextest run --all-features theme::tests::test_named_color_enum_variants 2>&1 | tail -10`
Expected: FAIL with "cannot find type `NamedColor`".

- [ ] **Step 3: Add `NamedColor` enum in `src/theme/palette.rs`**

Append after the `Severity` impl block (after Task 2's additions):

```rust
// =============================================================================
// Named Palette Colors
// =============================================================================

/// A flat enum of 26 palette color names derived from Catppuccin Mocha — the most
/// complete shipped palette.
///
/// Use [`Theme::color`] to look up a named color in the active theme. Every theme
/// returns a sensible color for every variant via its [`Palette`] mapping; for
/// non-Catppuccin themes that lack a native equivalent, the mapping uses the
/// nearest-shade match (documented per theme).
///
/// `#[non_exhaustive]` so envision can add palette names later without breaking
/// downstream `match` arms.
///
/// # Example
///
/// ```rust
/// use envision::theme::{NamedColor, Theme};
///
/// let theme = Theme::catppuccin_mocha();
/// let lavender = theme.color(NamedColor::Lavender);
/// // Use `lavender` in a Style, Cell, or StyledInline span.
/// # let _ = lavender;
/// ```
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NamedColor {
    // Accent colors (warm)
    /// Rosewater — pastel pink with rose undertone.
    Rosewater,
    /// Flamingo — pastel pink with peach undertone.
    Flamingo,
    /// Pink — saturated pink.
    Pink,
    /// Mauve — pastel purple.
    Mauve,
    /// Red — saturated red.
    Red,
    /// Maroon — darker red with brown undertone.
    Maroon,
    /// Peach — pastel orange.
    Peach,
    /// Yellow — pastel yellow.
    Yellow,
    /// Green — pastel green.
    Green,
    /// Teal — pastel teal (green-cyan).
    Teal,

    // Accent colors (cool)
    /// Sky — light cyan-blue.
    Sky,
    /// Sapphire — saturated cyan-blue.
    Sapphire,
    /// Blue — pastel blue.
    Blue,
    /// Lavender — pastel purple-blue.
    Lavender,

    // Text + overlay (light → dark)
    /// Text — primary foreground (lightest text tone).
    Text,
    /// Subtext1 — slightly muted foreground.
    Subtext1,
    /// Subtext0 — more muted foreground.
    Subtext0,
    /// Overlay2 — lightest overlay tone.
    Overlay2,
    /// Overlay1 — medium overlay tone.
    Overlay1,
    /// Overlay0 — darkest overlay tone.
    Overlay0,
    /// Surface2 — lightest surface tone (panels, popovers).
    Surface2,
    /// Surface1 — medium surface tone.
    Surface1,
    /// Surface0 — darkest surface tone (background-ish).
    Surface0,
    /// Base — primary background.
    Base,
    /// Mantle — secondary background (slightly darker than Base).
    Mantle,
    /// Crust — darkest background tone.
    Crust,
}
```

- [ ] **Step 4: Extend the `mod.rs` import to include `NamedColor`**

In `src/theme/mod.rs`, change:

```rust
pub use palette::Severity;
```

to:

```rust
pub use palette::{NamedColor, Severity};
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo nextest run --all-features theme::tests::test_named_color_enum_variants 2>&1 | tail -10`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/theme/palette.rs src/theme/mod.rs src/theme/tests.rs
git commit -S -m "Add NamedColor enum (26 variants, #[non_exhaustive])

Flat enum of palette color names derived from Catppuccin Mocha. Every
variant has a documented semantic role (accent warm/cool, text/overlay,
surface/base). Subsequent commits add the Palette struct, Theme.color()
accessor, and per-theme palette population."
```

---

## Task 4: Add `Palette` struct (failing test)

**Files:**
- Modify: `src/theme/tests.rs`
- Modify: `src/theme/palette.rs`
- Modify: `src/theme/mod.rs` (extend the import)

- [ ] **Step 1: Write the failing test**

Append to `src/theme/tests.rs`:

```rust
#[test]
fn test_palette_struct_construction() {
    // Direct construction: a custom user theme can build a Palette with no envision changes.
    let custom = Palette {
        rosewater: Color::Rgb(255, 0, 0),
        flamingo:  Color::Rgb(255, 0, 0),
        pink:      Color::Rgb(255, 0, 0),
        mauve:     Color::Rgb(255, 0, 0),
        red:       Color::Rgb(255, 0, 0),
        maroon:    Color::Rgb(255, 0, 0),
        peach:     Color::Rgb(255, 0, 0),
        yellow:    Color::Rgb(255, 0, 0),
        green:     Color::Rgb(255, 0, 0),
        teal:      Color::Rgb(255, 0, 0),
        sky:       Color::Rgb(255, 0, 0),
        sapphire:  Color::Rgb(255, 0, 0),
        blue:      Color::Rgb(255, 0, 0),
        lavender:  Color::Rgb(255, 0, 0),
        text:      Color::Rgb(255, 0, 0),
        subtext1:  Color::Rgb(255, 0, 0),
        subtext0:  Color::Rgb(255, 0, 0),
        overlay2:  Color::Rgb(255, 0, 0),
        overlay1:  Color::Rgb(255, 0, 0),
        overlay0:  Color::Rgb(255, 0, 0),
        surface2:  Color::Rgb(255, 0, 0),
        surface1:  Color::Rgb(255, 0, 0),
        surface0:  Color::Rgb(255, 0, 0),
        base:      Color::Rgb(255, 0, 0),
        mantle:    Color::Rgb(255, 0, 0),
        crust:     Color::Rgb(255, 0, 0),
    };
    assert_eq!(custom.rosewater, Color::Rgb(255, 0, 0));
    assert_eq!(custom.crust, Color::Rgb(255, 0, 0));
    // Pin Clone + Copy + Debug + PartialEq.
    let cloned = custom;
    assert_eq!(custom, cloned);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo nextest run --all-features theme::tests::test_palette_struct_construction 2>&1 | tail -10`
Expected: FAIL with "cannot find struct `Palette`".

- [ ] **Step 3: Add `Palette` struct in `src/theme/palette.rs`**

Append immediately after the `pub enum NamedColor { ... }` block:

```rust
/// A complete 26-color palette mapping every [`NamedColor`] variant to a `Color`.
///
/// Each shipped theme stores a `Palette` populated at construction time. Custom user
/// themes can construct a `Palette` directly; no envision modification required.
///
/// # Example
///
/// Construct a custom palette for a hand-crafted theme:
///
/// ```rust
/// use envision::theme::Palette;
/// use ratatui::style::Color;
///
/// let palette = Palette {
///     rosewater: Color::Rgb(0xF5, 0xE0, 0xDC),
///     flamingo:  Color::Rgb(0xF2, 0xCD, 0xCD),
///     pink:      Color::Rgb(0xF5, 0xC2, 0xE7),
///     mauve:     Color::Rgb(0xCB, 0xA6, 0xF7),
///     red:       Color::Rgb(0xF3, 0x8B, 0xA8),
///     maroon:    Color::Rgb(0xEB, 0xA0, 0xAC),
///     peach:     Color::Rgb(0xFA, 0xB3, 0x87),
///     yellow:    Color::Rgb(0xF9, 0xE2, 0xAF),
///     green:     Color::Rgb(0xA6, 0xE3, 0xA1),
///     teal:      Color::Rgb(0x94, 0xE2, 0xD5),
///     sky:       Color::Rgb(0x89, 0xDC, 0xEB),
///     sapphire:  Color::Rgb(0x74, 0xC7, 0xEC),
///     blue:      Color::Rgb(0x89, 0xB4, 0xFA),
///     lavender:  Color::Rgb(0xB4, 0xBE, 0xFE),
///     text:      Color::Rgb(0xCD, 0xD6, 0xF4),
///     subtext1:  Color::Rgb(0xBA, 0xC2, 0xDE),
///     subtext0:  Color::Rgb(0xA6, 0xAD, 0xC8),
///     overlay2:  Color::Rgb(0x93, 0x99, 0xB2),
///     overlay1:  Color::Rgb(0x7F, 0x84, 0x9C),
///     overlay0:  Color::Rgb(0x6C, 0x70, 0x86),
///     surface2:  Color::Rgb(0x58, 0x5B, 0x70),
///     surface1:  Color::Rgb(0x45, 0x47, 0x5A),
///     surface0:  Color::Rgb(0x31, 0x32, 0x44),
///     base:      Color::Rgb(0x1E, 0x1E, 0x2E),
///     mantle:    Color::Rgb(0x18, 0x18, 0x25),
///     crust:     Color::Rgb(0x11, 0x11, 0x1B),
/// };
/// # let _ = palette;
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Palette {
    /// Color for [`NamedColor::Rosewater`].
    pub rosewater: Color,
    /// Color for [`NamedColor::Flamingo`].
    pub flamingo: Color,
    /// Color for [`NamedColor::Pink`].
    pub pink: Color,
    /// Color for [`NamedColor::Mauve`].
    pub mauve: Color,
    /// Color for [`NamedColor::Red`].
    pub red: Color,
    /// Color for [`NamedColor::Maroon`].
    pub maroon: Color,
    /// Color for [`NamedColor::Peach`].
    pub peach: Color,
    /// Color for [`NamedColor::Yellow`].
    pub yellow: Color,
    /// Color for [`NamedColor::Green`].
    pub green: Color,
    /// Color for [`NamedColor::Teal`].
    pub teal: Color,
    /// Color for [`NamedColor::Sky`].
    pub sky: Color,
    /// Color for [`NamedColor::Sapphire`].
    pub sapphire: Color,
    /// Color for [`NamedColor::Blue`].
    pub blue: Color,
    /// Color for [`NamedColor::Lavender`].
    pub lavender: Color,
    /// Color for [`NamedColor::Text`].
    pub text: Color,
    /// Color for [`NamedColor::Subtext1`].
    pub subtext1: Color,
    /// Color for [`NamedColor::Subtext0`].
    pub subtext0: Color,
    /// Color for [`NamedColor::Overlay2`].
    pub overlay2: Color,
    /// Color for [`NamedColor::Overlay1`].
    pub overlay1: Color,
    /// Color for [`NamedColor::Overlay0`].
    pub overlay0: Color,
    /// Color for [`NamedColor::Surface2`].
    pub surface2: Color,
    /// Color for [`NamedColor::Surface1`].
    pub surface1: Color,
    /// Color for [`NamedColor::Surface0`].
    pub surface0: Color,
    /// Color for [`NamedColor::Base`].
    pub base: Color,
    /// Color for [`NamedColor::Mantle`].
    pub mantle: Color,
    /// Color for [`NamedColor::Crust`].
    pub crust: Color,
}
```

- [ ] **Step 4: Extend the `mod.rs` import to include `Palette`**

In `src/theme/mod.rs`, change:

```rust
pub use palette::{NamedColor, Severity};
```

to:

```rust
pub use palette::{NamedColor, Palette, Severity};
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo nextest run --all-features theme::tests::test_palette_struct_construction 2>&1 | tail -10`
Expected: PASS.

- [ ] **Step 6: Run doc test**

Run: `cargo test --all-features --doc theme::Palette 2>&1 | tail -10`
Expected: doc test PASSes.

- [ ] **Step 7: Commit**

```bash
git add src/theme/palette.rs src/theme/mod.rs src/theme/tests.rs
git commit -S -m "Add Palette struct (26 fields, one per NamedColor variant)

Public-field struct so custom themes can build a Palette directly.
Derives Clone+Copy+Debug+PartialEq+Eq. Doc test demonstrates direct
construction with the Catppuccin Mocha hex values for reference."
```

---

## Task 5: Add `palette: Palette` field to `Theme`; populate Catppuccin

**Files:**
- Modify: `src/theme/tests.rs`
- Modify: `src/theme/mod.rs`

- [ ] **Step 1: Write the failing test**

Append to `src/theme/tests.rs`:

```rust
#[test]
fn test_catppuccin_palette_pinned() {
    // Pin every Catppuccin palette entry to its source constant.
    // Future palette tweaks show up as test diffs.
    let theme = Theme::catppuccin_mocha();
    let p = &theme.palette;
    assert_eq!(p.rosewater, CATPPUCCIN_ROSEWATER);
    assert_eq!(p.flamingo, CATPPUCCIN_FLAMINGO);
    assert_eq!(p.pink, CATPPUCCIN_PINK);
    assert_eq!(p.mauve, CATPPUCCIN_MAUVE);
    assert_eq!(p.red, CATPPUCCIN_RED);
    assert_eq!(p.maroon, CATPPUCCIN_MAROON);
    assert_eq!(p.peach, CATPPUCCIN_PEACH);
    assert_eq!(p.yellow, CATPPUCCIN_YELLOW);
    assert_eq!(p.green, CATPPUCCIN_GREEN);
    assert_eq!(p.teal, CATPPUCCIN_TEAL);
    assert_eq!(p.sky, CATPPUCCIN_SKY);
    assert_eq!(p.sapphire, CATPPUCCIN_SAPPHIRE);
    assert_eq!(p.blue, CATPPUCCIN_BLUE);
    assert_eq!(p.lavender, CATPPUCCIN_LAVENDER);
    assert_eq!(p.text, CATPPUCCIN_TEXT);
    assert_eq!(p.subtext1, CATPPUCCIN_SUBTEXT1);
    assert_eq!(p.subtext0, CATPPUCCIN_SUBTEXT0);
    assert_eq!(p.overlay2, CATPPUCCIN_OVERLAY2);
    assert_eq!(p.overlay1, CATPPUCCIN_OVERLAY1);
    assert_eq!(p.overlay0, CATPPUCCIN_OVERLAY0);
    assert_eq!(p.surface2, CATPPUCCIN_SURFACE2);
    assert_eq!(p.surface1, CATPPUCCIN_SURFACE1);
    assert_eq!(p.surface0, CATPPUCCIN_SURFACE0);
    assert_eq!(p.base, CATPPUCCIN_BASE);
    assert_eq!(p.mantle, CATPPUCCIN_MANTLE);
    assert_eq!(p.crust, CATPPUCCIN_CRUST);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo nextest run --all-features theme::tests::test_catppuccin_palette_pinned 2>&1 | tail -10`
Expected: FAIL with "no field `palette` on type `&Theme`" — every existing constructor will need to be updated.

- [ ] **Step 3: Add `palette: Palette` field to `Theme`**

In `src/theme/mod.rs`, modify the `Theme` struct definition (around line 195-231). Add the field at the end of the existing fields (after `progress_empty`):

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Theme {
    // Base colors
    /// Background color for UI elements.
    pub background: Color,
    /// Foreground (text) color.
    pub foreground: Color,
    /// Border color for boxes and frames.
    pub border: Color,

    // Interactive states
    /// Color for focused elements (borders, text).
    pub focused: Color,
    /// Color for selected items in lists/tables.
    pub selected: Color,
    /// Color for disabled elements.
    pub disabled: Color,
    /// Color for placeholder text.
    pub placeholder: Color,

    // Semantic colors
    /// Primary accent color.
    pub primary: Color,
    /// Success state color (green).
    pub success: Color,
    /// Warning state color (yellow/orange).
    pub warning: Color,
    /// Error state color (red).
    pub error: Color,
    /// Informational state color (blue/cyan).
    pub info: Color,

    // Progress bar specific
    /// Filled portion of progress bars.
    pub progress_filled: Color,
    /// Empty portion of progress bars.
    pub progress_empty: Color,

    // Named-color palette (26 entries; populated per-theme)
    /// Theme-specific palette of named colors. Use [`Theme::color`] for theme-aware
    /// lookup; this field is exposed primarily for users constructing custom themes.
    pub palette: Palette,
}
```

- [ ] **Step 4: Update `Theme::catppuccin_mocha()` to populate the palette**

In `src/theme/mod.rs`, find `pub fn catppuccin_mocha() -> Self { ... }` (around line 465). Replace the body with:

```rust
pub fn catppuccin_mocha() -> Self {
    Self {
        background: CATPPUCCIN_BASE,
        foreground: CATPPUCCIN_TEXT,
        border: CATPPUCCIN_SURFACE2,

        focused: CATPPUCCIN_LAVENDER,
        selected: CATPPUCCIN_MAUVE,
        disabled: CATPPUCCIN_SURFACE2,
        placeholder: CATPPUCCIN_OVERLAY0,

        primary: CATPPUCCIN_BLUE,
        success: CATPPUCCIN_GREEN,
        warning: CATPPUCCIN_YELLOW,
        error: CATPPUCCIN_RED,
        info: CATPPUCCIN_SAPPHIRE,

        progress_filled: CATPPUCCIN_LAVENDER,
        progress_empty: CATPPUCCIN_SURFACE0,

        palette: Palette {
            rosewater: CATPPUCCIN_ROSEWATER,
            flamingo:  CATPPUCCIN_FLAMINGO,
            pink:      CATPPUCCIN_PINK,
            mauve:     CATPPUCCIN_MAUVE,
            red:       CATPPUCCIN_RED,
            maroon:    CATPPUCCIN_MAROON,
            peach:     CATPPUCCIN_PEACH,
            yellow:    CATPPUCCIN_YELLOW,
            green:     CATPPUCCIN_GREEN,
            teal:      CATPPUCCIN_TEAL,
            sky:       CATPPUCCIN_SKY,
            sapphire:  CATPPUCCIN_SAPPHIRE,
            blue:      CATPPUCCIN_BLUE,
            lavender:  CATPPUCCIN_LAVENDER,
            text:      CATPPUCCIN_TEXT,
            subtext1:  CATPPUCCIN_SUBTEXT1,
            subtext0:  CATPPUCCIN_SUBTEXT0,
            overlay2:  CATPPUCCIN_OVERLAY2,
            overlay1:  CATPPUCCIN_OVERLAY1,
            overlay0:  CATPPUCCIN_OVERLAY0,
            surface2:  CATPPUCCIN_SURFACE2,
            surface1:  CATPPUCCIN_SURFACE1,
            surface0:  CATPPUCCIN_SURFACE0,
            base:      CATPPUCCIN_BASE,
            mantle:    CATPPUCCIN_MANTLE,
            crust:     CATPPUCCIN_CRUST,
        },
    }
}
```

- [ ] **Step 5: Update other constructors with placeholder palette to make compile pass**

The `Theme` struct now has a required `palette` field, so every other constructor (`default`, `nord`, `dracula`, `solarized_dark`, `gruvbox_dark`) needs to populate it. Insert a temporary identical-Catppuccin palette into each — Tasks 6-10 will replace with the real per-theme mappings. This task only ensures the code compiles.

In `src/theme/mod.rs`, in `impl Default for Theme { fn default() -> Self { Self { ... } } }` (around line 241), append `palette: Theme::catppuccin_mocha().palette,` as the last field. Same for `nord()`, `dracula()`, `solarized_dark()`, `gruvbox_dark()`.

For example, `Theme::nord()` becomes:

```rust
pub fn nord() -> Self {
    Self {
        background: NORD0,
        foreground: NORD6,
        border: NORD3,

        focused: NORD8,
        selected: NORD9,
        disabled: NORD3,
        placeholder: NORD3,

        primary: NORD10,
        success: NORD14,
        warning: NORD13,
        error: NORD11,
        info: NORD8,

        progress_filled: NORD8,
        progress_empty: NORD1,

        // Placeholder palette — replaced with Nord-specific mappings in Task 6.
        palette: Theme::catppuccin_mocha().palette,
    }
}
```

Apply the same `palette: Theme::catppuccin_mocha().palette,` line to `default()`, `dracula()`, `solarized_dark()`, `gruvbox_dark()`.

- [ ] **Step 6: Run test to verify it passes**

Run: `cargo nextest run --all-features theme::tests::test_catppuccin_palette_pinned 2>&1 | tail -10`
Expected: PASS.

- [ ] **Step 7: Run all theme tests to verify no regressions**

Run: `cargo nextest run --all-features theme:: 2>&1 | tail -10`
Expected: all theme tests pass (including the existing 28).

- [ ] **Step 8: Commit**

```bash
git add src/theme/mod.rs src/theme/tests.rs
git commit -S -m "Add Palette field to Theme; populate Catppuccin Mocha palette

Theme now stores a 26-entry Palette field populated per constructor.
Catppuccin Mocha gets the canonical 1:1 mapping. Non-Catppuccin
constructors temporarily reuse the Catppuccin palette to keep the build
green; Tasks 6-10 replace those with theme-specific nearest-equivalent
mappings."
```

---

## Task 6: Populate Nord palette (nearest-equivalent mapping)

**Files:**
- Modify: `src/theme/tests.rs`
- Modify: `src/theme/mod.rs`

- [ ] **Step 1: Write the failing test**

Append to `src/theme/tests.rs`:

```rust
#[test]
fn test_nord_palette_pinned() {
    // Pin Nord's nearest-equivalent palette mapping. Documented in Theme::nord().
    let theme = Theme::nord();
    let p = &theme.palette;

    // Nord Aurora warm accents map to closest hue.
    assert_eq!(p.red, NORD11);              // Nord red
    assert_eq!(p.maroon, NORD11);           // No native maroon; reuse red
    assert_eq!(p.peach, NORD12);            // Nord orange
    assert_eq!(p.yellow, NORD13);           // Nord yellow
    assert_eq!(p.green, NORD14);            // Nord green
    assert_eq!(p.teal, NORD7);              // Nord frost teal

    // Nord pinks/rosewaters: no native; map to Snow Storm light tones.
    assert_eq!(p.rosewater, NORD4);
    assert_eq!(p.flamingo, NORD4);

    // Nord purples (only Nord15 exists): pink/mauve/lavender all to Nord15.
    assert_eq!(p.pink, NORD15);
    assert_eq!(p.mauve, NORD15);
    assert_eq!(p.lavender, NORD15);

    // Nord Frost cool blues.
    assert_eq!(p.sky, NORD8);               // light blue
    assert_eq!(p.sapphire, NORD9);          // mid blue
    assert_eq!(p.blue, NORD10);             // deep blue

    // Text/overlay tones from Snow Storm (light → less light).
    assert_eq!(p.text, NORD6);
    assert_eq!(p.subtext1, NORD5);
    assert_eq!(p.subtext0, NORD4);
    assert_eq!(p.overlay2, NORD3);
    assert_eq!(p.overlay1, NORD3);
    assert_eq!(p.overlay0, NORD3);

    // Surface/background tones from Polar Night (light → dark).
    assert_eq!(p.surface2, NORD2);
    assert_eq!(p.surface1, NORD1);
    assert_eq!(p.surface0, NORD1);
    assert_eq!(p.base, NORD0);
    assert_eq!(p.mantle, NORD0);
    assert_eq!(p.crust, NORD0);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo nextest run --all-features theme::tests::test_nord_palette_pinned 2>&1 | tail -10`
Expected: FAIL — Nord constructor still uses Catppuccin's palette.

- [ ] **Step 3: Replace Nord palette with the documented mapping**

In `src/theme/mod.rs`, find the `Theme::nord()` constructor. Replace the placeholder `palette: Theme::catppuccin_mocha().palette,` line with the explicit per-theme mapping. Also update the docstring on `Theme::nord` to document the palette mapping.

```rust
/// Creates a new Nord-themed color scheme.
///
/// The Nord theme uses the popular Nord color palette with its
/// characteristic frost blues and aurora accent colors.
///
/// # Colors
///
/// - Focused: Nord8 (light blue #88C0D0)
/// - Selected: Nord9 (blue #81A1C1)
/// - Disabled: Nord3 (muted gray #4C566A)
/// - Success: Nord14 (green #A3BE8C)
/// - Warning: Nord13 (yellow #EBCB8B)
/// - Error: Nord11 (red #BF616A)
///
/// # Palette mapping
///
/// Nord's 16-color palette doesn't include every Catppuccin name. The palette is
/// populated with nearest-equivalent mappings:
///
/// - `Rosewater` / `Flamingo` → Nord4 (Snow Storm light)
/// - `Pink` / `Mauve` / `Lavender` → Nord15 (Aurora purple — closest hue)
/// - `Red` / `Maroon` → Nord11 (Aurora red)
/// - `Peach` → Nord12 (Aurora orange)
/// - `Yellow` → Nord13 (Aurora yellow)
/// - `Green` → Nord14 (Aurora green)
/// - `Teal` → Nord7 (Frost teal)
/// - `Sky` / `Sapphire` / `Blue` → Nord8 / Nord9 / Nord10 (Frost blues, light → deep)
/// - Text tones → Nord4–Nord6 (Snow Storm)
/// - Overlay tones → Nord3 (Polar Night borders)
/// - Surface / base tones → Nord0–Nord2 (Polar Night)
///
/// # Example
///
/// ```rust
/// use envision::theme::Theme;
///
/// let theme = Theme::nord();
/// // Use with components:
/// // let mut ctx = RenderContext::new(frame, area, &theme);
/// // Button::view(&state, &mut ctx);
/// ```
pub fn nord() -> Self {
    Self {
        background: NORD0,
        foreground: NORD6,
        border: NORD3,

        focused: NORD8,
        selected: NORD9,
        disabled: NORD3,
        placeholder: NORD3,

        primary: NORD10,
        success: NORD14,
        warning: NORD13,
        error: NORD11,
        info: NORD8,

        progress_filled: NORD8,
        progress_empty: NORD1,

        palette: Palette {
            rosewater: NORD4,
            flamingo:  NORD4,
            pink:      NORD15,
            mauve:     NORD15,
            red:       NORD11,
            maroon:    NORD11,
            peach:     NORD12,
            yellow:    NORD13,
            green:     NORD14,
            teal:      NORD7,
            sky:       NORD8,
            sapphire:  NORD9,
            blue:      NORD10,
            lavender:  NORD15,
            text:      NORD6,
            subtext1:  NORD5,
            subtext0:  NORD4,
            overlay2:  NORD3,
            overlay1:  NORD3,
            overlay0:  NORD3,
            surface2:  NORD2,
            surface1:  NORD1,
            surface0:  NORD1,
            base:      NORD0,
            mantle:    NORD0,
            crust:     NORD0,
        },
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo nextest run --all-features theme::tests::test_nord_palette_pinned 2>&1 | tail -10`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/theme/mod.rs src/theme/tests.rs
git commit -S -m "Populate Nord palette with nearest-equivalent mappings

Nord lacks native equivalents for several Catppuccin names (Rosewater,
Pink, Mauve, etc.). Maps each missing name to the nearest Nord shade by
hue: pinks/mauves/lavender to Nord15 purple; rosewaters to Nord4 Snow
Storm light. Text and surface tones use Nord's Snow Storm and Polar
Night ranges. Documented inline in the Theme::nord() docstring."
```

---

## Task 7: Populate Dracula palette (nearest-equivalent mapping)

**Files:**
- Modify: `src/theme/tests.rs`
- Modify: `src/theme/mod.rs`

- [ ] **Step 1: Write the failing test**

Append to `src/theme/tests.rs`:

```rust
#[test]
fn test_dracula_palette_pinned() {
    let theme = Theme::dracula();
    let p = &theme.palette;

    // Dracula has named accents: Cyan, Green, Orange, Pink, Purple, Red, Yellow.
    assert_eq!(p.red, DRACULA_RED);
    assert_eq!(p.maroon, DRACULA_RED);          // No maroon; reuse red
    assert_eq!(p.peach, DRACULA_ORANGE);
    assert_eq!(p.yellow, DRACULA_YELLOW);
    assert_eq!(p.green, DRACULA_GREEN);
    assert_eq!(p.teal, DRACULA_CYAN);           // Closest cool teal-ish

    // Pinks/mauves/lavender: native pink and purple available.
    assert_eq!(p.pink, DRACULA_PINK);
    assert_eq!(p.mauve, DRACULA_PURPLE);
    assert_eq!(p.lavender, DRACULA_PURPLE);
    assert_eq!(p.rosewater, DRACULA_PINK);      // Closest pastel pink
    assert_eq!(p.flamingo, DRACULA_PINK);

    // Cool blues: Dracula has only Cyan; map all blues to Cyan.
    assert_eq!(p.sky, DRACULA_CYAN);
    assert_eq!(p.sapphire, DRACULA_CYAN);
    assert_eq!(p.blue, DRACULA_CYAN);

    // Text + overlay tones.
    assert_eq!(p.text, DRACULA_FG);
    assert_eq!(p.subtext1, DRACULA_FG);
    assert_eq!(p.subtext0, DRACULA_COMMENT);
    assert_eq!(p.overlay2, DRACULA_COMMENT);
    assert_eq!(p.overlay1, DRACULA_COMMENT);
    assert_eq!(p.overlay0, DRACULA_COMMENT);

    // Surface / base tones.
    assert_eq!(p.surface2, DRACULA_CURRENT);
    assert_eq!(p.surface1, DRACULA_CURRENT);
    assert_eq!(p.surface0, DRACULA_CURRENT);
    assert_eq!(p.base, DRACULA_BG);
    assert_eq!(p.mantle, DRACULA_BG);
    assert_eq!(p.crust, DRACULA_BG);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo nextest run --all-features theme::tests::test_dracula_palette_pinned 2>&1 | tail -10`
Expected: FAIL.

- [ ] **Step 3: Replace Dracula palette with the documented mapping**

In `src/theme/mod.rs`, find `Theme::dracula()`. Replace the placeholder palette and update the docstring:

```rust
/// Creates a new Dracula-themed color scheme.
///
/// The Dracula theme uses the popular Dracula color palette with its
/// characteristic purples, pinks, and vibrant accent colors.
///
/// # Colors
///
/// - Focused: Purple (#BD93F9)
/// - Selected: Pink (#FF79C6)
/// - Disabled: Comment (#6272A4)
/// - Success: Green (#50FA7B)
/// - Warning: Yellow (#F1FA8C)
/// - Error: Red (#FF5555)
///
/// # Palette mapping
///
/// Dracula's 9-color accent palette (Cyan/Green/Orange/Pink/Purple/Red/Yellow plus
/// FG/BG/Comment/CurrentLine) maps as follows:
///
/// - `Pink` / `Rosewater` / `Flamingo` → Dracula Pink (the only native pink)
/// - `Mauve` / `Lavender` → Dracula Purple
/// - `Red` / `Maroon` → Dracula Red
/// - `Peach` → Dracula Orange
/// - `Sky` / `Sapphire` / `Blue` / `Teal` → Dracula Cyan (the only native cool color)
/// - Text/Overlay → FG / Comment
/// - Surface / Base → CurrentLine / BG
///
/// # Example
///
/// ```rust
/// use envision::theme::Theme;
///
/// let theme = Theme::dracula();
/// assert_eq!(theme.focused, envision::theme::DRACULA_PURPLE);
/// ```
pub fn dracula() -> Self {
    Self {
        background: DRACULA_BG,
        foreground: DRACULA_FG,
        border: DRACULA_COMMENT,

        focused: DRACULA_PURPLE,
        selected: DRACULA_PINK,
        disabled: DRACULA_COMMENT,
        placeholder: DRACULA_COMMENT,

        primary: DRACULA_CYAN,
        success: DRACULA_GREEN,
        warning: DRACULA_YELLOW,
        error: DRACULA_RED,
        info: DRACULA_CYAN,

        progress_filled: DRACULA_PURPLE,
        progress_empty: DRACULA_CURRENT,

        palette: Palette {
            rosewater: DRACULA_PINK,
            flamingo:  DRACULA_PINK,
            pink:      DRACULA_PINK,
            mauve:     DRACULA_PURPLE,
            red:       DRACULA_RED,
            maroon:    DRACULA_RED,
            peach:     DRACULA_ORANGE,
            yellow:    DRACULA_YELLOW,
            green:     DRACULA_GREEN,
            teal:      DRACULA_CYAN,
            sky:       DRACULA_CYAN,
            sapphire:  DRACULA_CYAN,
            blue:      DRACULA_CYAN,
            lavender:  DRACULA_PURPLE,
            text:      DRACULA_FG,
            subtext1:  DRACULA_FG,
            subtext0:  DRACULA_COMMENT,
            overlay2:  DRACULA_COMMENT,
            overlay1:  DRACULA_COMMENT,
            overlay0:  DRACULA_COMMENT,
            surface2:  DRACULA_CURRENT,
            surface1:  DRACULA_CURRENT,
            surface0:  DRACULA_CURRENT,
            base:      DRACULA_BG,
            mantle:    DRACULA_BG,
            crust:     DRACULA_BG,
        },
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo nextest run --all-features theme::tests::test_dracula_palette_pinned 2>&1 | tail -10`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/theme/mod.rs src/theme/tests.rs
git commit -S -m "Populate Dracula palette with nearest-equivalent mappings

Dracula's 9-color accent palette covers most Catppuccin names; pinks
collapse to native pink, mauves/lavender to purple, all blues to cyan
(Dracula has no distinct sky/sapphire/blue). Documented in docstring."
```

---

## Task 8: Populate Solarized Dark palette (nearest-equivalent mapping)

**Files:**
- Modify: `src/theme/tests.rs`
- Modify: `src/theme/mod.rs`

- [ ] **Step 1: Write the failing test**

Append to `src/theme/tests.rs`:

```rust
#[test]
fn test_solarized_dark_palette_pinned() {
    let theme = Theme::solarized_dark();
    let p = &theme.palette;

    // Solarized has yellow/orange/red/magenta/violet/blue/cyan/green plus base shades.
    assert_eq!(p.red, SOLARIZED_RED);
    assert_eq!(p.maroon, SOLARIZED_RED);
    assert_eq!(p.peach, SOLARIZED_ORANGE);
    assert_eq!(p.yellow, SOLARIZED_YELLOW);
    assert_eq!(p.green, SOLARIZED_GREEN);
    assert_eq!(p.teal, SOLARIZED_CYAN);

    // Pinks: no native pink; magenta is closest pink-ish hue.
    assert_eq!(p.pink, SOLARIZED_MAGENTA);
    assert_eq!(p.rosewater, SOLARIZED_MAGENTA);
    assert_eq!(p.flamingo, SOLARIZED_MAGENTA);
    assert_eq!(p.mauve, SOLARIZED_MAGENTA);
    assert_eq!(p.lavender, SOLARIZED_MAGENTA);  // No violet const exported; magenta closest

    // Cool blues.
    assert_eq!(p.sky, SOLARIZED_CYAN);
    assert_eq!(p.sapphire, SOLARIZED_BLUE);
    assert_eq!(p.blue, SOLARIZED_BLUE);

    // Text/overlay/surface tones.
    assert_eq!(p.text, SOLARIZED_BASE1);
    assert_eq!(p.subtext1, SOLARIZED_BASE0);
    assert_eq!(p.subtext0, SOLARIZED_BASE01);
    assert_eq!(p.overlay2, SOLARIZED_BASE01);
    assert_eq!(p.overlay1, SOLARIZED_BASE01);
    assert_eq!(p.overlay0, SOLARIZED_BASE01);
    assert_eq!(p.surface2, SOLARIZED_BASE02);
    assert_eq!(p.surface1, SOLARIZED_BASE02);
    assert_eq!(p.surface0, SOLARIZED_BASE02);
    assert_eq!(p.base, SOLARIZED_BASE03);
    assert_eq!(p.mantle, SOLARIZED_BASE03);
    assert_eq!(p.crust, SOLARIZED_BASE03);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo nextest run --all-features theme::tests::test_solarized_dark_palette_pinned 2>&1 | tail -10`
Expected: FAIL.

- [ ] **Step 3: Replace Solarized Dark palette and update docstring**

In `src/theme/mod.rs`, find `Theme::solarized_dark()`. Update docstring + replace palette:

```rust
/// Creates a new Solarized Dark-themed color scheme.
///
/// The Solarized Dark theme uses Ethan Schoonover's carefully designed
/// color palette optimized for readability and reduced eye strain.
///
/// # Colors
///
/// - Focused: Blue (#268BD2)
/// - Selected: Cyan (#2AA198)
/// - Disabled: Base01 (#586E75)
/// - Success: Green (#859900)
/// - Warning: Yellow (#B58900)
/// - Error: Red (#DC322F)
///
/// # Palette mapping
///
/// Solarized's accent palette (yellow/orange/red/magenta/blue/cyan/green) maps
/// to Catppuccin names as follows. Solarized has no native pink — magenta is
/// the closest pinkish hue.
///
/// - `Pink` / `Rosewater` / `Flamingo` / `Mauve` / `Lavender` → Magenta
/// - `Red` / `Maroon` → Red
/// - `Peach` → Orange
/// - `Sky` / `Teal` → Cyan
/// - `Sapphire` / `Blue` → Blue
/// - Text/Overlay → Base1 / Base0 / Base01
/// - Surface / Base → Base02 / Base03
///
/// # Example
///
/// ```rust
/// use envision::theme::Theme;
///
/// let theme = Theme::solarized_dark();
/// assert_eq!(theme.focused, envision::theme::SOLARIZED_BLUE);
/// ```
pub fn solarized_dark() -> Self {
    Self {
        background: SOLARIZED_BASE03,
        foreground: SOLARIZED_BASE0,
        border: SOLARIZED_BASE01,

        focused: SOLARIZED_BLUE,
        selected: SOLARIZED_CYAN,
        disabled: SOLARIZED_BASE01,
        placeholder: SOLARIZED_BASE01,

        primary: SOLARIZED_BLUE,
        success: SOLARIZED_GREEN,
        warning: SOLARIZED_YELLOW,
        error: SOLARIZED_RED,
        info: SOLARIZED_CYAN,

        progress_filled: SOLARIZED_BLUE,
        progress_empty: SOLARIZED_BASE02,

        palette: Palette {
            rosewater: SOLARIZED_MAGENTA,
            flamingo:  SOLARIZED_MAGENTA,
            pink:      SOLARIZED_MAGENTA,
            mauve:     SOLARIZED_MAGENTA,
            red:       SOLARIZED_RED,
            maroon:    SOLARIZED_RED,
            peach:     SOLARIZED_ORANGE,
            yellow:    SOLARIZED_YELLOW,
            green:     SOLARIZED_GREEN,
            teal:      SOLARIZED_CYAN,
            sky:       SOLARIZED_CYAN,
            sapphire:  SOLARIZED_BLUE,
            blue:      SOLARIZED_BLUE,
            lavender:  SOLARIZED_MAGENTA,
            text:      SOLARIZED_BASE1,
            subtext1:  SOLARIZED_BASE0,
            subtext0:  SOLARIZED_BASE01,
            overlay2:  SOLARIZED_BASE01,
            overlay1:  SOLARIZED_BASE01,
            overlay0:  SOLARIZED_BASE01,
            surface2:  SOLARIZED_BASE02,
            surface1:  SOLARIZED_BASE02,
            surface0:  SOLARIZED_BASE02,
            base:      SOLARIZED_BASE03,
            mantle:    SOLARIZED_BASE03,
            crust:     SOLARIZED_BASE03,
        },
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo nextest run --all-features theme::tests::test_solarized_dark_palette_pinned 2>&1 | tail -10`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/theme/mod.rs src/theme/tests.rs
git commit -S -m "Populate Solarized Dark palette with nearest-equivalent mappings

Pinks/mauves/lavender collapse to magenta (no native pink in Solarized).
Sapphire and blue both use Solarized blue. Surface tones use base02/03.
Documented in docstring."
```

---

## Task 9: Populate Gruvbox Dark palette (nearest-equivalent mapping)

**Files:**
- Modify: `src/theme/tests.rs`
- Modify: `src/theme/mod.rs`

- [ ] **Step 1: Write the failing test**

Append to `src/theme/tests.rs`:

```rust
#[test]
fn test_gruvbox_dark_palette_pinned() {
    let theme = Theme::gruvbox_dark();
    let p = &theme.palette;

    // Gruvbox accents: red/green/yellow/blue/purple/aqua/orange.
    assert_eq!(p.red, GRUVBOX_RED);
    assert_eq!(p.maroon, GRUVBOX_RED);
    assert_eq!(p.peach, GRUVBOX_ORANGE);
    assert_eq!(p.yellow, GRUVBOX_YELLOW);
    assert_eq!(p.green, GRUVBOX_GREEN);
    assert_eq!(p.teal, GRUVBOX_AQUA);

    // Pinks/mauves/lavender: gruvbox purple is the only purple.
    assert_eq!(p.pink, GRUVBOX_PURPLE);
    assert_eq!(p.rosewater, GRUVBOX_PURPLE);
    assert_eq!(p.flamingo, GRUVBOX_PURPLE);
    assert_eq!(p.mauve, GRUVBOX_PURPLE);
    assert_eq!(p.lavender, GRUVBOX_PURPLE);

    // Cool blues: only one blue + aqua.
    assert_eq!(p.sky, GRUVBOX_AQUA);
    assert_eq!(p.sapphire, GRUVBOX_BLUE);
    assert_eq!(p.blue, GRUVBOX_BLUE);

    // Text/overlay tones.
    assert_eq!(p.text, GRUVBOX_FG);
    assert_eq!(p.subtext1, GRUVBOX_FG);
    assert_eq!(p.subtext0, GRUVBOX_GRAY);
    assert_eq!(p.overlay2, GRUVBOX_GRAY);
    assert_eq!(p.overlay1, GRUVBOX_GRAY);
    assert_eq!(p.overlay0, GRUVBOX_GRAY);

    // Surface/base tones.
    assert_eq!(p.surface2, GRUVBOX_BG1);
    assert_eq!(p.surface1, GRUVBOX_BG1);
    assert_eq!(p.surface0, GRUVBOX_BG1);
    assert_eq!(p.base, GRUVBOX_BG);
    assert_eq!(p.mantle, GRUVBOX_BG);
    assert_eq!(p.crust, GRUVBOX_BG);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo nextest run --all-features theme::tests::test_gruvbox_dark_palette_pinned 2>&1 | tail -10`
Expected: FAIL.

- [ ] **Step 3: Replace Gruvbox Dark palette and update docstring**

In `src/theme/mod.rs`, find `Theme::gruvbox_dark()`. Update docstring + palette:

```rust
/// Creates a new Gruvbox Dark-themed color scheme.
///
/// The Gruvbox Dark theme uses the retro-groove Gruvbox color palette
/// with its warm, earthy tones and high contrast.
///
/// # Colors
///
/// - Focused: Yellow (#FABD2F)
/// - Selected: Blue (#83A598)
/// - Disabled: Gray (#928374)
/// - Success: Green (#B8BB26)
/// - Warning: Orange (#FE8019)
/// - Error: Red (#FB4934)
///
/// # Palette mapping
///
/// Gruvbox's 7-color accent palette (red/green/yellow/blue/purple/aqua/orange)
/// maps as follows:
///
/// - `Pink` / `Rosewater` / `Flamingo` / `Mauve` / `Lavender` → Purple
/// - `Red` / `Maroon` → Red
/// - `Peach` → Orange
/// - `Teal` / `Sky` → Aqua
/// - `Sapphire` / `Blue` → Blue
/// - Text/Overlay → FG / Gray
/// - Surface/Base → BG1 / BG
///
/// # Example
///
/// ```rust
/// use envision::theme::Theme;
///
/// let theme = Theme::gruvbox_dark();
/// assert_eq!(theme.focused, envision::theme::GRUVBOX_YELLOW);
/// ```
pub fn gruvbox_dark() -> Self {
    Self {
        background: GRUVBOX_BG,
        foreground: GRUVBOX_FG,
        border: GRUVBOX_GRAY,

        focused: GRUVBOX_YELLOW,
        selected: GRUVBOX_BLUE,
        disabled: GRUVBOX_GRAY,
        placeholder: GRUVBOX_GRAY,

        primary: GRUVBOX_AQUA,
        success: GRUVBOX_GREEN,
        warning: GRUVBOX_ORANGE,
        error: GRUVBOX_RED,
        info: GRUVBOX_BLUE,

        progress_filled: GRUVBOX_YELLOW,
        progress_empty: GRUVBOX_BG1,

        palette: Palette {
            rosewater: GRUVBOX_PURPLE,
            flamingo:  GRUVBOX_PURPLE,
            pink:      GRUVBOX_PURPLE,
            mauve:     GRUVBOX_PURPLE,
            red:       GRUVBOX_RED,
            maroon:    GRUVBOX_RED,
            peach:     GRUVBOX_ORANGE,
            yellow:    GRUVBOX_YELLOW,
            green:     GRUVBOX_GREEN,
            teal:      GRUVBOX_AQUA,
            sky:       GRUVBOX_AQUA,
            sapphire:  GRUVBOX_BLUE,
            blue:      GRUVBOX_BLUE,
            lavender:  GRUVBOX_PURPLE,
            text:      GRUVBOX_FG,
            subtext1:  GRUVBOX_FG,
            subtext0:  GRUVBOX_GRAY,
            overlay2:  GRUVBOX_GRAY,
            overlay1:  GRUVBOX_GRAY,
            overlay0:  GRUVBOX_GRAY,
            surface2:  GRUVBOX_BG1,
            surface1:  GRUVBOX_BG1,
            surface0:  GRUVBOX_BG1,
            base:      GRUVBOX_BG,
            mantle:    GRUVBOX_BG,
            crust:     GRUVBOX_BG,
        },
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo nextest run --all-features theme::tests::test_gruvbox_dark_palette_pinned 2>&1 | tail -10`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/theme/mod.rs src/theme/tests.rs
git commit -S -m "Populate Gruvbox Dark palette with nearest-equivalent mappings

Gruvbox's 7-color accent palette covers Red/Green/Yellow/Blue/Purple/
Aqua/Orange; pinks and mauves all collapse to purple. Documented in
docstring."
```

---

## Task 10: Populate Default theme palette (basic-Color collapse)

**Files:**
- Modify: `src/theme/tests.rs`
- Modify: `src/theme/mod.rs`

- [ ] **Step 1: Write the failing test**

Append to `src/theme/tests.rs`:

```rust
#[test]
fn test_default_palette_pinned() {
    let theme = Theme::default();
    let p = &theme.palette;

    // The Default theme uses ratatui's basic Color enum for max terminal compat.
    // Many NamedColor variants intentionally collapse to the same basic Color.
    assert_eq!(p.red, Color::Red);
    assert_eq!(p.maroon, Color::Red);
    assert_eq!(p.flamingo, Color::Red);
    assert_eq!(p.rosewater, Color::Red);

    assert_eq!(p.peach, Color::Yellow);
    assert_eq!(p.yellow, Color::Yellow);

    assert_eq!(p.green, Color::Green);
    assert_eq!(p.teal, Color::Green);

    assert_eq!(p.sky, Color::Cyan);
    assert_eq!(p.sapphire, Color::Cyan);
    assert_eq!(p.blue, Color::Blue);

    assert_eq!(p.pink, Color::Magenta);
    assert_eq!(p.mauve, Color::Magenta);
    assert_eq!(p.lavender, Color::Magenta);

    // Text + overlay tones.
    assert_eq!(p.text, Color::White);
    assert_eq!(p.subtext1, Color::Gray);
    assert_eq!(p.subtext0, Color::Gray);
    assert_eq!(p.overlay2, Color::DarkGray);
    assert_eq!(p.overlay1, Color::DarkGray);
    assert_eq!(p.overlay0, Color::DarkGray);

    // Surface/base tones.
    assert_eq!(p.surface2, Color::DarkGray);
    assert_eq!(p.surface1, Color::Black);
    assert_eq!(p.surface0, Color::Black);
    assert_eq!(p.base, Color::Reset);
    assert_eq!(p.mantle, Color::Reset);
    assert_eq!(p.crust, Color::Black);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo nextest run --all-features theme::tests::test_default_palette_pinned 2>&1 | tail -10`
Expected: FAIL.

- [ ] **Step 3: Replace Default palette and update docstring**

In `src/theme/mod.rs`, find `impl Default for Theme { fn default() -> Self { ... } }`. Replace placeholder + update docstring:

```rust
impl Default for Theme {
    /// Returns the default theme matching ratatui's standard colors.
    ///
    /// This theme uses:
    /// - Yellow for focused elements
    /// - DarkGray for disabled/placeholder elements
    /// - Cyan for primary/info
    /// - Standard Green/Yellow/Red for success/warning/error
    ///
    /// # Palette collapse note
    ///
    /// The `Default` theme uses ratatui's basic `Color` enum (Reset, Yellow, Red,
    /// Cyan, etc.) for maximum terminal compatibility. Many [`NamedColor`] variants
    /// collapse to the same basic `Color`:
    ///
    /// - `Peach` / `Yellow` → `Color::Yellow`
    /// - `Pink` / `Mauve` / `Lavender` → `Color::Magenta`
    /// - `Red` / `Maroon` / `Flamingo` / `Rosewater` → `Color::Red`
    /// - `Green` / `Teal` → `Color::Green`
    /// - `Sky` / `Sapphire` → `Color::Cyan`; `Blue` → `Color::Blue`
    /// - Text tones → `Color::White` / `Color::Gray`; overlay/surface → `Color::DarkGray` / `Color::Black`
    ///
    /// Notably this affects [`Theme::severity_color`]: on `Default`, `Mild`
    /// (`Yellow`) and `Bad` (`Peach`) both render as `Color::Yellow`. The
    /// four-band gradient effectively becomes three-band on the `Default` theme.
    /// [`Theme::severity_style`] still adds `BOLD` for `Critical`, so the
    /// strongest band stays distinguishable. Consumers wanting full palette
    /// fidelity should use [`Theme::catppuccin_mocha`] or another full-palette
    /// theme.
    fn default() -> Self {
        Self {
            background: Color::Reset,
            foreground: Color::Reset,
            border: Color::Reset,

            focused: Color::Yellow,
            selected: Color::Reset,
            disabled: Color::DarkGray,
            placeholder: Color::DarkGray,

            primary: Color::Cyan,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            info: Color::Cyan,

            progress_filled: Color::Cyan,
            progress_empty: Color::Black,

            palette: Palette {
                rosewater: Color::Red,
                flamingo:  Color::Red,
                pink:      Color::Magenta,
                mauve:     Color::Magenta,
                red:       Color::Red,
                maroon:    Color::Red,
                peach:     Color::Yellow,
                yellow:    Color::Yellow,
                green:     Color::Green,
                teal:      Color::Green,
                sky:       Color::Cyan,
                sapphire:  Color::Cyan,
                blue:      Color::Blue,
                lavender:  Color::Magenta,
                text:      Color::White,
                subtext1:  Color::Gray,
                subtext0:  Color::Gray,
                overlay2:  Color::DarkGray,
                overlay1:  Color::DarkGray,
                overlay0:  Color::DarkGray,
                surface2:  Color::DarkGray,
                surface1:  Color::Black,
                surface0:  Color::Black,
                base:      Color::Reset,
                mantle:    Color::Reset,
                crust:     Color::Black,
            },
        }
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo nextest run --all-features theme::tests::test_default_palette_pinned 2>&1 | tail -10`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/theme/mod.rs src/theme/tests.rs
git commit -S -m "Populate Default theme palette with basic-Color collapse

Default theme intentionally collapses many NamedColor variants onto a
small set of basic Color values (e.g., Peach and Yellow both → Yellow).
This is documented on Theme::default() with an explicit caveat about
severity_color collapsing the four-band gradient to three on Default.
Critical band stays distinguishable via BOLD modifier."
```

---

## Task 11: Add `Theme::color(NamedColor)` accessor

**Files:**
- Modify: `src/theme/tests.rs`
- Modify: `src/theme/palette.rs`

- [ ] **Step 1: Write the failing tests**

Append to `src/theme/tests.rs`:

```rust
#[test]
fn test_namedcolor_lookup_returns_palette_entry() {
    // theme.color(N) must return the same value as theme.palette.<field for N>.
    let theme = Theme::catppuccin_mocha();
    assert_eq!(theme.color(NamedColor::Lavender), theme.palette.lavender);
    assert_eq!(theme.color(NamedColor::Peach),    theme.palette.peach);
    assert_eq!(theme.color(NamedColor::Crust),    theme.palette.crust);
    assert_eq!(theme.color(NamedColor::Rosewater), theme.palette.rosewater);
    assert_eq!(theme.color(NamedColor::Teal),     theme.palette.teal);

    // Same round-trip for a non-Catppuccin theme.
    let nord = Theme::nord();
    assert_eq!(nord.color(NamedColor::Lavender), nord.palette.lavender);
    assert_eq!(nord.color(NamedColor::Peach),    nord.palette.peach);
}

#[test]
fn test_namedcolor_distinct_per_theme() {
    // On Catppuccin Mocha, palette accents must be distinct.
    let theme = Theme::catppuccin_mocha();
    assert_ne!(theme.color(NamedColor::Peach), theme.color(NamedColor::Yellow));
    assert_ne!(theme.color(NamedColor::Red),   theme.color(NamedColor::Maroon));
    assert_ne!(theme.color(NamedColor::Blue),  theme.color(NamedColor::Sapphire));
    assert_ne!(theme.color(NamedColor::Pink),  theme.color(NamedColor::Mauve));
    assert_ne!(theme.color(NamedColor::Green), theme.color(NamedColor::Teal));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo nextest run --all-features theme::tests::test_namedcolor 2>&1 | tail -10`
Expected: FAIL with "no method named `color`".

- [ ] **Step 3: Add `Theme::color` method in `src/theme/palette.rs`**

Append a new `impl Theme { ... }` block at the end of `src/theme/palette.rs`. Rust allows multiple impl blocks across modules in the same crate, so the new methods compose cleanly with the existing impl in `mod.rs`.

```rust
// =============================================================================
// Theme accessors (color, severity_color, severity_style)
// =============================================================================

impl Theme {
    /// Returns the theme's color for a [`NamedColor`] palette name.
    ///
    /// This is the recommended way to access named palette colors that aren't
    /// covered by a semantic slot (`focused`, `success`, etc.). Always returns
    /// a sensible color for every variant; non-Catppuccin themes use
    /// nearest-equivalent mappings documented per theme.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::theme::{NamedColor, Theme};
    ///
    /// let theme = Theme::catppuccin_mocha();
    /// let lavender = theme.color(NamedColor::Lavender);
    /// // Use in a Cell, Span, or Style.
    /// # let _ = lavender;
    /// ```
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

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo nextest run --all-features theme::tests::test_namedcolor 2>&1 | tail -10`
Expected: 2 tests PASS.

- [ ] **Step 5: Run doc test**

Run: `cargo test --all-features --doc theme::Theme::color 2>&1 | tail -10`
Expected: doc test PASSes.

- [ ] **Step 6: Commit**

```bash
git add src/theme/palette.rs src/theme/tests.rs
git commit -S -m "Add Theme::color(NamedColor) accessor

26-arm exhaustive match routing each NamedColor variant to its Palette
field. Always returns a Color (no Option, no panic). Lives in palette.rs
in a second impl Theme block (mod.rs holds the existing impl). Tests
pin both round-trip equality with palette fields and accent distinctness
on Catppuccin Mocha."
```

---

## Task 12: Add `Theme::severity_color` and `Theme::severity_style`

**Files:**
- Modify: `src/theme/tests.rs`
- Modify: `src/theme/palette.rs`

- [ ] **Step 1: Write the failing tests**

Append to `src/theme/tests.rs`:

```rust
#[test]
fn test_severity_color_per_theme() {
    // Catppuccin: Good → Green, Mild → Yellow, Bad → Peach, Critical → Red.
    let cat = Theme::catppuccin_mocha();
    assert_eq!(cat.severity_color(Severity::Good),     CATPPUCCIN_GREEN);
    assert_eq!(cat.severity_color(Severity::Mild),     CATPPUCCIN_YELLOW);
    assert_eq!(cat.severity_color(Severity::Bad),      CATPPUCCIN_PEACH);
    assert_eq!(cat.severity_color(Severity::Critical), CATPPUCCIN_RED);

    // Nord: routes through palette.
    let nord = Theme::nord();
    assert_eq!(nord.severity_color(Severity::Good),     NORD14);
    assert_eq!(nord.severity_color(Severity::Mild),     NORD13);
    assert_eq!(nord.severity_color(Severity::Bad),      NORD12);
    assert_eq!(nord.severity_color(Severity::Critical), NORD11);

    // Default theme: Mild and Bad collapse to Color::Yellow per documented behavior.
    let def = Theme::default();
    assert_eq!(def.severity_color(Severity::Good),     Color::Green);
    assert_eq!(def.severity_color(Severity::Mild),     Color::Yellow);
    assert_eq!(def.severity_color(Severity::Bad),      Color::Yellow); // collapse
    assert_eq!(def.severity_color(Severity::Critical), Color::Red);
}

#[test]
fn test_severity_style_critical_is_bold() {
    let theme = Theme::catppuccin_mocha();
    let critical = theme.severity_style(Severity::Critical);
    let good = theme.severity_style(Severity::Good);
    let mild = theme.severity_style(Severity::Mild);
    let bad = theme.severity_style(Severity::Bad);

    assert!(critical.add_modifier.contains(Modifier::BOLD));
    assert!(!good.add_modifier.contains(Modifier::BOLD));
    assert!(!mild.add_modifier.contains(Modifier::BOLD));
    assert!(!bad.add_modifier.contains(Modifier::BOLD));
}

#[test]
fn test_severity_style_fg_matches_severity_color() {
    let theme = Theme::catppuccin_mocha();
    for sev in [Severity::Good, Severity::Mild, Severity::Bad, Severity::Critical] {
        let style = theme.severity_style(sev);
        assert_eq!(
            style.fg,
            Some(theme.severity_color(sev)),
            "severity_style({:?}).fg must equal severity_color({:?})",
            sev,
            sev
        );
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo nextest run --all-features theme::tests::test_severity_color theme::tests::test_severity_style 2>&1 | tail -10`
Expected: FAIL with "no method named `severity_color`".

- [ ] **Step 3: Add `severity_color` and `severity_style` methods**

In `src/theme/palette.rs`, inside the `impl Theme { ... }` block created in Task 11, immediately after the `color` method:

```rust
    /// Returns the theme's color for a [`Severity`] band.
    ///
    /// Maps `Good` → green, `Mild` → yellow, `Bad` → peach, `Critical` → red,
    /// routed through the theme's palette. Non-Catppuccin themes use their
    /// nearest-equivalent palette mappings.
    ///
    /// On the [`Default`](Theme::default) theme, `Mild` and `Bad` both collapse
    /// to `Color::Yellow` (basic-Color palette has no peach). Use
    /// [`severity_style`](Theme::severity_style) for distinguishability via the
    /// `BOLD` modifier on `Critical`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::theme::{Severity, Theme};
    ///
    /// let theme = Theme::catppuccin_mocha();
    /// let color = theme.severity_color(Severity::Bad);
    /// // Use in a Cell, Span, or Style.
    /// # let _ = color;
    /// ```
    pub fn severity_color(&self, sev: Severity) -> Color {
        match sev {
            Severity::Good     => self.color(NamedColor::Green),
            Severity::Mild     => self.color(NamedColor::Yellow),
            Severity::Bad      => self.color(NamedColor::Peach),
            Severity::Critical => self.color(NamedColor::Red),
        }
    }

    /// Returns a `Style` for a [`Severity`] band — color plus reasonable defaults.
    ///
    /// Equivalent to `Style::default().fg(theme.severity_color(sev))` plus a
    /// `BOLD` modifier when `sev == Severity::Critical`. The `BOLD` on Critical
    /// is intentional: critical events should stand out beyond color alone (for
    /// color-blind users, low-contrast terminals, partial color rendering).
    ///
    /// Drop-in for `Cell::with_style(CellStyle::Custom(...))` and
    /// `StyledInline::Styled { style, ... }` sites.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::theme::{Severity, Theme};
    ///
    /// let theme = Theme::catppuccin_mocha();
    /// let style = theme.severity_style(Severity::Critical);
    /// // style.fg is Catppuccin red and style includes BOLD.
    /// # let _ = style;
    /// ```
    pub fn severity_style(&self, sev: Severity) -> Style {
        let style = Style::default().fg(self.severity_color(sev));
        if sev == Severity::Critical {
            style.add_modifier(Modifier::BOLD)
        } else {
            style
        }
    }
```

(Note: these methods go *inside* the `impl Theme { ... }` block from Task 11, before the closing `}` of that block. Don't introduce a separate impl block for them.)

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo nextest run --all-features theme::tests::test_severity_color theme::tests::test_severity_style 2>&1 | tail -10`
Expected: 3 tests PASS.

- [ ] **Step 5: Run doc tests**

Run: `cargo test --all-features --doc theme::Theme::severity 2>&1 | tail -10`
Expected: 2 doc tests PASS.

- [ ] **Step 6: Commit**

```bash
git add src/theme/palette.rs src/theme/tests.rs
git commit -S -m "Add Theme::severity_color and Theme::severity_style

severity_color routes through palette via NamedColor (Good→Green,
Mild→Yellow, Bad→Peach, Critical→Red). severity_style adds BOLD for
Critical only (intentional: distinguishability beyond color alone).
Tests pin per-theme severity colors, BOLD-on-critical, and fg-matches-
severity_color invariant."
```

---

## Task 13: Add the palette-completeness test

**Files:**
- Modify: `src/theme/tests.rs`

- [ ] **Step 1: Write the test**

Append to `src/theme/tests.rs`:

```rust
#[test]
fn test_palette_completeness_per_theme() {
    // For each shipped theme, every NamedColor variant returns *some* color.
    // The Default theme legitimately uses Color::Reset for some background tones,
    // so this test does not blanket-reject Reset; it only verifies that
    // theme.color(N) executes the match without panicking and the field has been
    // assigned (no inadvertent Color::default() leftovers from incomplete
    // construction).
    //
    // This test pins current values transitively via the per-theme
    // `*_palette_pinned` tests; this test specifically guards against future
    // additions to NamedColor that lack per-theme handling. When a new variant
    // is added, the exhaustive match in Theme::color forces an arm; this test
    // ensures the per-theme constructors got updated.
    let themes = [
        ("default",          Theme::default()),
        ("nord",             Theme::nord()),
        ("dracula",          Theme::dracula()),
        ("solarized_dark",   Theme::solarized_dark()),
        ("gruvbox_dark",     Theme::gruvbox_dark()),
        ("catppuccin_mocha", Theme::catppuccin_mocha()),
    ];
    let all_named = [
        NamedColor::Rosewater, NamedColor::Flamingo, NamedColor::Pink,
        NamedColor::Mauve, NamedColor::Red, NamedColor::Maroon,
        NamedColor::Peach, NamedColor::Yellow, NamedColor::Green,
        NamedColor::Teal, NamedColor::Sky, NamedColor::Sapphire,
        NamedColor::Blue, NamedColor::Lavender, NamedColor::Text,
        NamedColor::Subtext1, NamedColor::Subtext0, NamedColor::Overlay2,
        NamedColor::Overlay1, NamedColor::Overlay0, NamedColor::Surface2,
        NamedColor::Surface1, NamedColor::Surface0, NamedColor::Base,
        NamedColor::Mantle, NamedColor::Crust,
    ];
    for (name, theme) in &themes {
        for n in &all_named {
            // Just call it; assert each call returns a value (compile guarantee).
            let _ = theme.color(*n);
        }
        // Sanity: at minimum, accent colors should not all be the same value.
        // (Catches a fully-defaulted Palette { ..Default::default() } accident.)
        assert_ne!(
            theme.color(NamedColor::Red),
            theme.color(NamedColor::Green),
            "{}: Red and Green collapsed — palette likely uninitialized",
            name,
        );
    }
}
```

- [ ] **Step 2: Run test to verify it passes**

Run: `cargo nextest run --all-features theme::tests::test_palette_completeness_per_theme 2>&1 | tail -10`
Expected: PASS (all themes have palettes populated by Tasks 5-10).

- [ ] **Step 3: Commit**

```bash
git add src/theme/tests.rs
git commit -S -m "Add palette completeness test across all 6 shipped themes

Iterates every (theme, NamedColor) pair and confirms theme.color(N)
returns a value. Pins Red != Green per theme as a sanity check against
fully-default Palette construction. Pin per-theme values live in the
existing *_palette_pinned tests — this test is the global guard."
```

---

## Task 14: Re-export `NamedColor`, `Palette`, `Severity` from crate root + prelude

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 1: Read the current lib.rs re-export and prelude blocks**

Run: `grep -n "pub use crate::theme\|pub use theme::Theme" src/lib.rs`
Expected output: line 413 `pub use theme::Theme;` and line 449 `pub use crate::theme::Theme;`

- [ ] **Step 2: Update the crate-root re-export**

In `src/lib.rs`, find line 413 (`pub use theme::Theme;`). Replace with:

```rust
pub use theme::{NamedColor, Palette, Severity, Theme};
```

- [ ] **Step 3: Update the prelude**

In `src/lib.rs`, find the prelude section near line 449. Replace `pub use crate::theme::Theme;` with:

```rust
    // Theme
    pub use crate::theme::{NamedColor, Severity, Theme};
```

(Note: `Palette` is intentionally omitted from the prelude. Custom-theme construction is rare; users opt into it explicitly via `envision::theme::Palette`.)

- [ ] **Step 4: Add a smoke test for the re-exports**

Append to `src/theme/tests.rs`:

```rust
#[test]
fn test_crate_root_reexports() {
    // Verify the new types are reachable from `envision::*` (compiled-out check).
    // The actual import path is exercised by usage; this test just pins the
    // smoke check.
    use crate::{NamedColor, Palette, Severity};
    let _ = NamedColor::Lavender;
    let _ = Severity::Good;
    // Palette construction is verified in test_palette_struct_construction.
    let _ = std::mem::size_of::<Palette>();
}
```

- [ ] **Step 5: Run tests + ensure compile**

Run: `cargo nextest run --all-features theme::tests::test_crate_root_reexports 2>&1 | tail -10`
Expected: PASS.

Run: `cargo build --all-features 2>&1 | tail -5`
Expected: clean build.

- [ ] **Step 6: Commit**

```bash
git add src/lib.rs src/theme/tests.rs
git commit -S -m "Re-export NamedColor, Palette, Severity from crate root + prelude

Adds NamedColor + Severity to the prelude alongside Theme. Palette stays
crate-root-only since direct construction is rare; users access it via
the explicit envision::theme::Palette path."
```

---

## Task 15: Deprecate raw color constants — Catppuccin

**Files:**
- Modify: `src/theme/catppuccin.rs`

- [ ] **Step 1: Add `#[deprecated]` attributes to every Catppuccin constant**

In `src/theme/catppuccin.rs`, decorate every `pub const CATPPUCCIN_* : Color = ...;` with the deprecation attribute. The deprecation note points to the new `theme.color(NamedColor::X)` path.

Replace each `pub const CATPPUCCIN_<NAME>: Color = ...;` line with:

```rust
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::<NAME>) for theme-aware lookup"
)]
pub const CATPPUCCIN_<NAME>: Color = ...;
```

For example, the first three (Crust, Mantle, Base) become:

```rust
/// Catppuccin Mocha - Crust (darkest background, #11111B)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Crust) for theme-aware lookup"
)]
pub const CATPPUCCIN_CRUST: Color = Color::Rgb(17, 17, 27);

/// Catppuccin Mocha - Mantle (darker background, #181825)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Mantle) for theme-aware lookup"
)]
pub const CATPPUCCIN_MANTLE: Color = Color::Rgb(24, 24, 37);

/// Catppuccin Mocha - Base (primary background, #1E1E2E)
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Base) for theme-aware lookup"
)]
pub const CATPPUCCIN_BASE: Color = Color::Rgb(30, 30, 46);
```

Apply the matching `NamedColor::<NAME>` for every constant: SURFACE0/1/2 → Surface0/1/2, OVERLAY0/1/2 → Overlay0/1/2, SUBTEXT0 → Subtext0, SUBTEXT1 → Subtext1, TEXT → Text, plus the 14 accent constants (ROSEWATER, FLAMINGO, PINK, MAUVE, RED, MAROON, PEACH, YELLOW, GREEN, TEAL, SKY, SAPPHIRE, BLUE, LAVENDER).

- [ ] **Step 2: Suppress deprecation warnings inside envision (we still use them in mod.rs)**

In `src/theme/mod.rs`, the existing constructor `Theme::catppuccin_mocha()` and tests reference `CATPPUCCIN_*` constants directly. To avoid a sea of deprecation warnings inside our own crate, add a blanket `#[allow(deprecated)]` on the `pub use catppuccin::*;` re-export.

In `src/theme/mod.rs`, line 42-43, replace:

```rust
pub mod catppuccin;
pub use catppuccin::*;
```

with:

```rust
pub mod catppuccin;
#[allow(deprecated)]
pub use catppuccin::*;
```

This re-exports the constants for external consumers (who get the deprecation warning at their use site) while letting envision's own internal references compile cleanly.

- [ ] **Step 3: Verify the build is clean**

Run: `cargo build --all-features 2>&1 | tail -10`
Expected: no deprecation warnings inside envision.

Run: `cargo nextest run --all-features theme:: 2>&1 | tail -10`
Expected: all tests pass.

- [ ] **Step 4: Verify external deprecation actually triggers**

Create a temporary file `/tmp/envision_deprecation_check.rs` with:

```rust
#![allow(unused_imports)]
fn main() {
    use envision::theme::CATPPUCCIN_PEACH;
    let _ = CATPPUCCIN_PEACH;
}
```

Run: `rustc --edition 2024 --extern envision=$(cargo build --all-features --message-format=json 2>/dev/null | grep -o '"filenames":\["[^"]*libenvision[^"]*\.rlib"' | head -1 | sed 's/.*"\([^"]*\)".*/\1/') -L target/debug/deps /tmp/envision_deprecation_check.rs -o /tmp/envision_deprecation_check 2>&1 | grep -i deprecat`

Expected: at least one `warning: use of deprecated constant` line referencing `CATPPUCCIN_PEACH`.

If the rustc one-liner is awkward, the simpler alternative: visually confirm by running `cargo doc --all-features --open 2>&1 | grep -i deprecat | head -5` — the rustdoc surface will show the deprecation banner on each constant.

- [ ] **Step 5: Commit**

```bash
git add src/theme/catppuccin.rs src/theme/mod.rs
git commit -S -m "Deprecate Catppuccin color constants in favor of theme.color(NamedColor)

Each CATPPUCCIN_* gets #[deprecated(since=\"0.17.0\")] pointing to the
theme-aware path. Internal uses (mod.rs constructors and tests) ride on
an #[allow(deprecated)] on the re-export to keep envision's own build
warning-free. External consumers see the deprecation at use site."
```

---

## Task 16: Deprecate raw color constants — Nord, Dracula, Solarized, Gruvbox

**Files:**
- Modify: `src/theme/mod.rs`

- [ ] **Step 1: Apply `#[deprecated]` attributes to all four palette blocks**

In `src/theme/mod.rs`, find the four `pub const NORD*: Color = ...;` constants (lines 52-85), the `pub const DRACULA_*: Color = ...;` constants (lines 92-112), `pub const SOLARIZED_*: Color = ...;` constants (lines 119-141), and `pub const GRUVBOX_*: Color = ...;` constants (lines 148-168).

Decorate each `pub const` with a `#[deprecated]` attribute pointing to the appropriate `NamedColor` mapping (per the per-theme docstrings added in Tasks 6-9).

For Nord, the mapping table to use in `#[deprecated]` notes:

| Constant | Note |
|---|---|
| `NORD0` / `NORD1` | "use theme.color(NamedColor::Base) or NamedColor::Surface*" |
| `NORD2` | "use theme.color(NamedColor::Surface2)" |
| `NORD3` | "use theme.color(NamedColor::Overlay*)" |
| `NORD4` | "use theme.color(NamedColor::Subtext0)" |
| `NORD5` | "use theme.color(NamedColor::Subtext1)" |
| `NORD6` | "use theme.color(NamedColor::Text)" |
| `NORD7` | "use theme.color(NamedColor::Teal)" |
| `NORD8` | "use theme.color(NamedColor::Sky)" |
| `NORD9` | "use theme.color(NamedColor::Sapphire)" |
| `NORD10` | "use theme.color(NamedColor::Blue)" |
| `NORD11` | "use theme.color(NamedColor::Red)" |
| `NORD12` | "use theme.color(NamedColor::Peach)" |
| `NORD13` | "use theme.color(NamedColor::Yellow)" |
| `NORD14` | "use theme.color(NamedColor::Green)" |
| `NORD15` | "use theme.color(NamedColor::Lavender) or NamedColor::Mauve" |

For Dracula, the mapping:

| Constant | Note |
|---|---|
| `DRACULA_BG` | "use theme.color(NamedColor::Base)" |
| `DRACULA_CURRENT` | "use theme.color(NamedColor::Surface2)" |
| `DRACULA_FG` | "use theme.color(NamedColor::Text)" |
| `DRACULA_COMMENT` | "use theme.color(NamedColor::Subtext0) or NamedColor::Overlay*" |
| `DRACULA_CYAN` | "use theme.color(NamedColor::Sky) or NamedColor::Sapphire" |
| `DRACULA_GREEN` | "use theme.color(NamedColor::Green)" |
| `DRACULA_ORANGE` | "use theme.color(NamedColor::Peach)" |
| `DRACULA_PINK` | "use theme.color(NamedColor::Pink)" |
| `DRACULA_PURPLE` | "use theme.color(NamedColor::Mauve) or NamedColor::Lavender" |
| `DRACULA_RED` | "use theme.color(NamedColor::Red)" |
| `DRACULA_YELLOW` | "use theme.color(NamedColor::Yellow)" |

For Solarized:

| Constant | Note |
|---|---|
| `SOLARIZED_BASE03` | "use theme.color(NamedColor::Base)" |
| `SOLARIZED_BASE02` | "use theme.color(NamedColor::Surface2)" |
| `SOLARIZED_BASE01` | "use theme.color(NamedColor::Subtext0) or NamedColor::Overlay*" |
| `SOLARIZED_BASE0` | "use theme.color(NamedColor::Subtext1)" |
| `SOLARIZED_BASE1` | "use theme.color(NamedColor::Text)" |
| `SOLARIZED_BLUE` | "use theme.color(NamedColor::Blue) or NamedColor::Sapphire" |
| `SOLARIZED_CYAN` | "use theme.color(NamedColor::Sky) or NamedColor::Teal" |
| `SOLARIZED_GREEN` | "use theme.color(NamedColor::Green)" |
| `SOLARIZED_YELLOW` | "use theme.color(NamedColor::Yellow)" |
| `SOLARIZED_ORANGE` | "use theme.color(NamedColor::Peach)" |
| `SOLARIZED_RED` | "use theme.color(NamedColor::Red)" |
| `SOLARIZED_MAGENTA` | "use theme.color(NamedColor::Pink) or NamedColor::Mauve" |

For Gruvbox:

| Constant | Note |
|---|---|
| `GRUVBOX_BG` | "use theme.color(NamedColor::Base)" |
| `GRUVBOX_BG1` | "use theme.color(NamedColor::Surface2)" |
| `GRUVBOX_FG` | "use theme.color(NamedColor::Text)" |
| `GRUVBOX_GRAY` | "use theme.color(NamedColor::Subtext0) or NamedColor::Overlay*" |
| `GRUVBOX_RED` | "use theme.color(NamedColor::Red)" |
| `GRUVBOX_GREEN` | "use theme.color(NamedColor::Green)" |
| `GRUVBOX_YELLOW` | "use theme.color(NamedColor::Yellow)" |
| `GRUVBOX_BLUE` | "use theme.color(NamedColor::Blue) or NamedColor::Sapphire" |
| `GRUVBOX_PURPLE` | "use theme.color(NamedColor::Mauve) or NamedColor::Lavender" |
| `GRUVBOX_AQUA` | "use theme.color(NamedColor::Teal) or NamedColor::Sky" |
| `GRUVBOX_ORANGE` | "use theme.color(NamedColor::Peach)" |

For each `pub const X: Color = Color::Rgb(...);` line, prepend its existing doc comment with the deprecation. Example for `NORD0`:

```rust
/// Nord Polar Night - darkest background
#[deprecated(
    since = "0.17.0",
    note = "use theme.color(NamedColor::Base) or NamedColor::Surface* for theme-aware lookup"
)]
pub const NORD0: Color = Color::Rgb(46, 52, 64);
```

Apply this pattern to all 15 NORD, 11 DRACULA, 12 SOLARIZED, 11 GRUVBOX constants — total 49 deprecations.

- [ ] **Step 2: Suppress internal deprecation warnings**

Inside `src/theme/mod.rs`, the constructors (`Theme::nord`, `Theme::dracula`, `Theme::solarized_dark`, `Theme::gruvbox_dark`) reference these constants. Annotate each constructor (and the relevant tests) with `#[allow(deprecated)]` so envision's own build stays warning-free.

For each constructor, add `#[allow(deprecated)]` directly above the `pub fn` line:

```rust
#[allow(deprecated)]
pub fn nord() -> Self {
    Self {
        background: NORD0,
        // ...
    }
}
```

Apply to `Theme::nord`, `Theme::dracula`, `Theme::solarized_dark`, `Theme::gruvbox_dark`.

Also, in `src/theme/tests.rs`, add `#[allow(deprecated)]` above each existing test that asserts equality with a NORD/DRACULA/SOLARIZED/GRUVBOX constant. Specifically:

- `test_nord_theme`, `test_nord_colors`, `test_normal_style_nord`, `test_focused_border_style_differs_from_focused_style`, `test_primary_style_nord`, `test_border_style`, `test_selected_highlight_style_focused`, `test_selected_highlight_style_unfocused`, `test_dracula_theme`, `test_dracula_colors`, `test_solarized_dark_theme`, `test_solarized_dark_colors`, `test_gruvbox_dark_theme`, `test_gruvbox_dark_colors`, `test_catppuccin_mocha_theme`, `test_catppuccin_mocha_colors`
- The new per-theme palette pinned tests (Tasks 6-10): `test_nord_palette_pinned`, `test_dracula_palette_pinned`, `test_solarized_dark_palette_pinned`, `test_gruvbox_dark_palette_pinned`, `test_catppuccin_palette_pinned`
- The Catppuccin `test_severity_color_per_theme` (uses `CATPPUCCIN_GREEN` etc.)

For each, prepend `#[allow(deprecated)]` between `#[test]` and the `fn name()` line:

```rust
#[test]
#[allow(deprecated)]
fn test_nord_theme() {
    // ...existing body...
}
```

- [ ] **Step 3: Verify clean build**

Run: `cargo build --all-features 2>&1 | tail -10`
Expected: no deprecation warnings.

Run: `cargo nextest run --all-features theme:: 2>&1 | tail -10`
Expected: all tests pass.

- [ ] **Step 4: Verify external deprecation triggers**

Create `/tmp/envision_deprecation_check2.rs`:

```rust
#![allow(unused_imports)]
fn main() {
    use envision::theme::{NORD0, DRACULA_PURPLE, SOLARIZED_BLUE, GRUVBOX_YELLOW};
    let _ = (NORD0, DRACULA_PURPLE, SOLARIZED_BLUE, GRUVBOX_YELLOW);
}
```

Run a quick sanity build using a doctest in the spec or just run `cargo doc --all-features --no-deps 2>&1 | grep -i 'deprecated' | wc -l`. Expected: > 50 (49 individual constants × 1+ surface).

Alternative: visually confirm via `cargo doc --all-features --open` and inspect the theme module — every deprecated const has a `Deprecated since 0.17.0` banner.

- [ ] **Step 5: Commit**

```bash
git add src/theme/mod.rs src/theme/tests.rs
git commit -S -m "Deprecate Nord/Dracula/Solarized/Gruvbox color constants

49 pub const items get #[deprecated(since=\"0.17.0\")] with notes
pointing at the right NamedColor variant(s). Internal uses (constructors
and existing tests) ride on per-function #[allow(deprecated)] to keep
envision's build warning-free. External consumers see the deprecation
banner at use site."
```

---

## Task 17: Verify clippy + doc + format clean

**Files:**
- (none — verification only)

- [ ] **Step 1: Run clippy with all features**

Run: `cargo clippy --all-features --all-targets -- -D warnings 2>&1 | tail -20`
Expected: clean — no warnings.

If failures appear, fix in-place and `git commit -S -m "Address clippy lint"`.

- [ ] **Step 2: Run rustdoc with deny-warnings**

Run: `RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps 2>&1 | tail -20`
Expected: clean.

If broken intra-doc links appear (e.g., `[NamedColor]` resolving wrong), use the fully-qualified form `[\`crate::theme::NamedColor\`]` or `[\`Theme::color\`](crate::theme::Theme::color)`.

- [ ] **Step 3: Run cargo fmt check**

Run: `cargo fmt --all -- --check 2>&1 | tail -10`
Expected: clean.

If formatting drift, run `cargo fmt --all` and commit:

```bash
git add -A
git commit -S -m "Apply cargo fmt"
```

- [ ] **Step 4: Run full test suite**

Run: `cargo nextest run --all-features 2>&1 | tail -20`
Expected: all tests pass — existing ~15K tests + the new ones from Tasks 1-13.

Run: `cargo test --all-features --doc theme:: 2>&1 | tail -20`
Expected: all theme doc tests pass.

- [ ] **Step 5: Run audit scorecard**

Run: `./tools/audit/target/release/envision-audit scorecard 2>&1 | tail -30`
Expected: 9/9 passing (no regression from current baseline).

If audit shows a regression, investigate. The new public types (NamedColor, Palette, Severity) push the public-item count up; this is expected and the scorecard treats it as a positive signal.

- [ ] **Step 6: Commit (if any fixes were needed)**

If any of the verification steps required a fix, commit it. Otherwise, no commit needed for this task.

---

## Task 18: CHANGELOG entry

**Files:**
- Modify: `CHANGELOG.md`

- [ ] **Step 1: Add the entry under `## [Unreleased]`**

Open `CHANGELOG.md`. Below the existing "Chrome ownership protocol" section, add a new sub-section:

```markdown
### Theme palette + severity helper (D6 + D9)

Three new types extend `Theme` with theme-aware named-color access and a unified
severity vocabulary, eliminating the need to import raw `CATPPUCCIN_*` /
`NORD_*` / `DRACULA_*` / `SOLARIZED_*` / `GRUVBOX_*` constants.

**New public types:**

- `NamedColor` enum (`#[non_exhaustive]`) — 26 variants derived from Catppuccin
  Mocha (Rosewater, Flamingo, Pink, Mauve, Red, Maroon, Peach, Yellow, Green,
  Teal, Sky, Sapphire, Blue, Lavender, Text, Subtext1, Subtext0, Overlay2,
  Overlay1, Overlay0, Surface2, Surface1, Surface0, Base, Mantle, Crust).
- `Palette` struct — one public `Color` field per `NamedColor` variant. Custom
  themes can construct a `Palette` directly without modifying envision.
- `Severity` enum (`#[non_exhaustive]`) — `Good | Mild | Bad | Critical`. Use
  `Severity::from_thresholds(value, &[(cutoff, severity), ...])` to bucket a
  numeric value via first-match-wins.

**New `Theme` methods:**

- `theme.color(NamedColor) -> Color` — theme-aware named-color lookup. Always
  succeeds; non-Catppuccin themes use documented nearest-equivalent palette
  mappings.
- `theme.severity_color(Severity) -> Color` — palette-routed severity color
  (Good→Green, Mild→Yellow, Bad→Peach, Critical→Red).
- `theme.severity_style(Severity) -> Style` — color + `BOLD` modifier on
  `Critical` only.

**New `Theme` field:**

- `theme.palette: Palette` — the theme's full 26-color palette. Already
  populated by every shipped theme constructor.

**Deprecations (no removals):**

- `CATPPUCCIN_*`, `NORD0`–`NORD15`, `DRACULA_*`, `SOLARIZED_*`, `GRUVBOX_*`
  `pub const` items are now `#[deprecated(since = "0.17.0")]` in favor of
  `theme.color(NamedColor::X)`. Constants stay accessible during the
  transition window; a follow-up PR will remove them.

**Default theme palette collapse:** the `Default` theme uses ratatui's basic
`Color` enum, so multiple `NamedColor` variants collapse to the same basic
color (e.g., `Peach` and `Yellow` both map to `Color::Yellow`). This affects
`severity_color`: on `Default`, `Mild` and `Bad` render identically. The
`severity_style(Critical)` `BOLD` modifier keeps the strongest band visually
distinguishable. Consumers wanting full palette fidelity should use
`Theme::catppuccin_mocha()` or another full-palette theme. Documented on
`Theme::default()`.
```

- [ ] **Step 2: Commit**

```bash
git add CHANGELOG.md
git commit -S -m "CHANGELOG: theme palette + severity helper (D6 + D9)

Document the additive Theme API: NamedColor enum, Palette struct,
Severity enum + from_thresholds, Theme::color/severity_color/
severity_style methods. Document deprecation of raw color constants
(no removals)."
```

---

## Task 19: Final verification + push

**Files:**
- (none — verification only)

- [ ] **Step 1: Verify all commits are signed**

Run: `git log --show-signature main..HEAD 2>&1 | grep -c 'Good signature'`
Expected: count matches the number of commits added on this branch (should be ~18).

If any commit is unsigned, **stop** and ask the user how to handle it — never bypass.

- [ ] **Step 2: Run the full verification gauntlet**

Run all four in parallel:

```bash
cargo build --all-features
cargo clippy --all-features --all-targets -- -D warnings
cargo fmt --all -- --check
cargo nextest run --all-features
cargo test --all-features --doc
RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps
./tools/audit/target/release/envision-audit scorecard
```

Expected: every command succeeds with no warnings. Audit shows 9/9.

- [ ] **Step 3: Push the branch**

Run: `git push -u origin theme-palette-severity-impl`
(Branch name is the implementation branch — set by the controller before plan execution begins.)

Expected: pushes cleanly.

- [ ] **Step 4: Open the implementation PR**

Run:

```bash
gh pr create --title "Theme palette + severity helper (D6 + D9)" --body "$(cat <<'EOF'
## Summary

Implementation of leadline gaps **D6** (severity helper in `Theme`) and **D9** (theme color access uneven; raw constants leak past abstraction).

Spec: PR #471 (\`docs/superpowers/specs/2026-05-03-theme-palette-severity-design.md\`)
Plan: PR β (\`docs/superpowers/plans/2026-05-03-theme-palette-severity.md\`)

## What changed

- New \`NamedColor\` enum (26 variants, \`#[non_exhaustive]\`)
- New \`Palette\` struct (one public \`Color\` field per variant)
- New \`Severity\` enum (\`Good | Mild | Bad | Critical\`) + \`from_thresholds\` bucketer
- New \`Theme::color(NamedColor) -> Color\` method
- New \`Theme::severity_color(Severity) -> Color\` and \`Theme::severity_style(Severity) -> Style\` methods
- New \`Theme.palette: Palette\` field, populated per-theme with documented nearest-equivalent mappings
- \`CATPPUCCIN_*\` / \`NORD_*\` / \`DRACULA_*\` / \`SOLARIZED_*\` / \`GRUVBOX_*\` constants \`#[deprecated(since="0.17.0")]\` (no removals)

## Test plan

- [ ] CI green on all platforms
- [ ] Audit scorecard 9/9
- [ ] leadline migrates per-op/baseline severity helpers; deletes \`severity_color_for_ratio\` / \`severity_status_style\` / \`severity_cell_style\`

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

Expected: PR URL printed; record it.

---

## Self-review (run by the controller before dispatch)

The plan author runs this checklist after writing all tasks; the implementer runs verification at Task 17 and Task 19.

**1. Spec coverage:**

| Spec section | Tasks |
|---|---|
| New `src/theme/palette.rs` module scaffold | Task 0 |
| `NamedColor` enum (26 variants, `#[non_exhaustive]`) | Task 3 |
| `Palette` struct (26 public fields, derive Clone/Copy/Debug/PartialEq/Eq) | Task 4 |
| `Theme::color(NamedColor) -> Color` (26-arm exhaustive match) | Task 11 |
| Per-theme palette mapping: Catppuccin 1:1 | Task 5 |
| Per-theme palette mapping: Nord nearest-equivalent | Task 6 |
| Per-theme palette mapping: Dracula | Task 7 |
| Per-theme palette mapping: Solarized Dark | Task 8 |
| Per-theme palette mapping: Gruvbox Dark | Task 9 |
| Per-theme palette mapping: Default basic-Color collapse | Task 10 |
| `Severity` enum (Good/Mild/Bad/Critical, `#[non_exhaustive]`) | Task 1 |
| `Severity::from_thresholds(f64, &[(f64, Severity)]) -> Severity` (first-match-wins) | Task 2 |
| `Theme::severity_color(Severity) -> Color` | Task 12 |
| `Theme::severity_style(Severity) -> Style` (BOLD on Critical) | Task 12 |
| Deprecation of `CATPPUCCIN_*` constants | Task 15 |
| Deprecation of `NORD_*`, `DRACULA_*`, `SOLARIZED_*`, `GRUVBOX_*` constants | Task 16 |
| `lib.rs` re-exports (`NamedColor`, `Palette`, `Severity`) + prelude | Task 14 |
| 9 named tests (palette completeness, namedcolor lookup, namedcolor distinct, from_thresholds boundaries, from_thresholds empty, from_thresholds unsorted, severity_color per theme, severity_style critical bold, severity_style fg matches severity_color) | Tasks 1, 2, 11, 12, 13 (test_palette_completeness in Task 13; the rest distributed across Tasks 1-12) |
| Doc test on `Severity::from_thresholds` | Task 2 |
| Default theme collapse note (docstring addition #1) | Task 10 |
| Palette test pins current values (docstring addition #2) | Task 13 |
| Custom-theme construction example on `Palette` (docstring addition #3) | Task 4 |
| CHANGELOG additive entry | Task 18 |

All spec requirements have a corresponding task.

**2. Placeholder scan:**

No `TBD`, `TODO`, `implement later`, `fill in details` in any task body. Each step has either complete code, an exact command, or an explicit verification criterion. ✅

**3. Type consistency:**

- `Severity` enum variants used consistently: `Good`, `Mild`, `Bad`, `Critical` (Tasks 1, 2, 12). ✅
- `NamedColor` variants used consistently across Tasks 3, 4, 5-10, 11, 12, 13, 15, 16. ✅
- Method signatures consistent: `theme.color(N)`, `theme.severity_color(S)`, `theme.severity_style(S)`, `Severity::from_thresholds(v, t)`. ✅
- `Palette` field names match `NamedColor` lower-case (rosewater, flamingo, etc.) — verified in Tasks 4 and 5. ✅

---

## Plan complete

The plan covers 20 tasks (Task 0 through Task 19) producing approximately 19 signed commits across the implementation branch. Estimated implementation time: 4-6 hours of focused work (each task is 5-15 minutes with TDD discipline).

After plan PR β merges, controller creates `theme-palette-severity-impl` branch from main and dispatches subagent-driven execution. After implementation PR γ merges, controller opens tracking-doc PR δ marking D6 + D9 ✅ resolved in `docs/customer-feedback/2026-05-01-leadline-gaps.md`.
