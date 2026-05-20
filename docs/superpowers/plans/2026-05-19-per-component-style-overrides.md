# Per-component style overrides (G4 + G5) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add per-component style override hooks: `PaneConfig::with_title_style(Style)` for pane titles independent of border styling (G4) and `StatusBarItem::with_color(Color)` + `with_style_override(Style)` for arbitrary per-item StatusBar coloring with layered precedence (G5).

**Architecture:** G4 adds a new `title_style: Option<Style>` field to `PaneConfig`; the new builder + getter live in a new sibling file `src/component/pane_layout/title_style.rs` to keep `mod.rs` under the 1000-line cap. Render arm in `view_with.rs` consults the field unconditionally (focus-invariant by design). G5 adds two layered `Option` fields (`color`, `style_override`) to `StatusBarItem` with four new methods all in `item.rs`; render arm in `status_bar/mod.rs` uses a three-branch precedence ladder (`style_override > color > style.style(theme)`). All field additions get `#[serde(default)]` for serialization forward-compat. Setter semantics are LAYERED — each setter writes its own field; render-time precedence picks. No last-call-wins.

**Tech Stack:** Rust 1.85+, ratatui 0.29 (`Style`, `Color`, `Span`, `Modifier`), envision `pane_layout` + `status_bar` components, `theme` module (`Theme`, `Severity`, `NamedColor`).

**Spec:** `docs/superpowers/specs/2026-05-19-per-component-style-overrides-design.md` (PR #485, merged)

---

## Pre-execution gotchas

- **Signed commits required.** Project rule. If `git commit -S` fails, ask the user; never bypass with `--no-gpg-sign`.
- **File-size cap (1000 lines) constraint.** `src/component/pane_layout/mod.rs` is at 975 lines on main. The naive approach of adding the new G4 field + builder + getter + docstrings in-place would push it over (~1007 lines). Mitigation: new sibling file `src/component/pane_layout/title_style.rs` houses the new `impl PaneConfig` block. Rust supports multi-module impl blocks for the same struct in the same crate — same pattern as `view_with.rs` already uses for `impl PaneLayout`. Field declaration + `pub mod title_style;` stay in `mod.rs` (only ~3 lines added there).
- **`status_bar/mod.rs` is 913 lines** — the 3-branch render arm change is +4 net lines, fine.
- **`status_bar/item.rs` is 548 lines** — plenty of room for the 2 fields + 4 methods.
- **`pane_layout/tests.rs` is 980 lines** — close to cap. New G4 tests (4 of them) go in a new sibling file `src/component/pane_layout/title_style_tests.rs` (or as inline `#[cfg(test)] mod tests` in `title_style.rs`). Picking inline-in-title_style.rs to keep the new G4 surface fully self-contained.
- **`ratatui::style::Style` is `Copy + PartialEq` but NOT `Eq`.** Adding `Option<Style>` fields preserves the existing `#[derive(Clone, Debug, PartialEq)]` on both `PaneConfig` and `StatusBarItem`. Neither derives `Eq` today; verified safe.
- **`ratatui::style::Color` is `Copy + Eq + PartialEq`.** Getter `color(&self) -> Option<Color>` returns by value, no `.clone()` needed.
- **Layered, NOT last-call-wins.** Spec is explicit — each setter writes its own field idempotently. Tests pin this; docstrings document it explicitly. A reader familiar with D15's `Cell::with_severity` (last-call-wins) might assume the same pattern here; the docstrings prevent surprise.
- **Focus invariance for `with_title_style`.** Render arm at `view_with.rs:53-55` consults `pane.title_style` unconditionally (doesn't check focus state). leadline soft-note #2: docstring + a focus-invariant snapshot test harden the contract.
- **Breaking-shape risk is limited to serialization forward-compat.** `PaneConfig` fields (lines 81-85) are all PRIVATE; `StatusBarItem` fields (`content`, `style`, `separator`) are `pub(super)`. External consumers cannot struct-literal-construct either; the only risk is pre-G5 serialized blobs lacking the new fields. `#[serde(default)]` on every new field handles this — `Option` defaults to `None`, preserving current rendering behavior for deserialized blobs.
- **cargo nextest** for unit tests; doc tests via `cargo test --all-features --doc`.
- **Audit baseline.** `./tools/audit/target/release/envision-audit scorecard` is at 8/9 on main; the `resource_gauge::set_values` accessor symmetry gap is pre-existing. No regression expected.

---

## File Structure

| File | Responsibility | Lines (before → after) |
|---|---|---|
| `src/component/pane_layout/mod.rs` | Existing `PaneConfig` struct + impl; adds new `title_style: Option<Style>` field with `#[serde(default)]` + `pub mod title_style;` declaration | 975 → ~980 |
| `src/component/pane_layout/title_style.rs` (NEW) | `impl PaneConfig { fn with_title_style, fn title_style }` + inline `#[cfg(test)] mod tests` with 4 G4 tests | 0 → ~200 |
| `src/component/pane_layout/view_with.rs:53-55` | Render arm consults `pane.title_style` (2-branch: `Some` → `Span::styled`, `None` → `Span::raw`) | 132 → ~140 |
| `src/component/status_bar/item.rs` | `StatusBarItem` gains `color: Option<Color>` + `style_override: Option<Style>` fields with `#[serde(default)]`; adds 4 new methods (2 builders + 2 getters) | 548 → ~620 |
| `src/component/status_bar/mod.rs:664` | Render arm: 3-branch precedence ladder (override > color > style.style(theme)) | 913 → ~917 |
| `src/component/status_bar/tests/style_item.rs` | Add 4 unit tests (with_color, with_style_override, layered preserves color, branched preserves color) | unchanged file; +~80 lines |
| `src/component/status_bar/snapshot_tests.rs` | Add 1 snapshot test for four-stop severity ramp | 128 → ~180 |
| `CHANGELOG.md` | Additive entry under `[Unreleased]` | adds ~30 lines |

All files stay well under the 1000-line cap.

**Why new sibling file for G4:** `pane_layout/mod.rs` would otherwise cross the 1000-line cap. The split mirrors the existing `view_with.rs` pattern (separate file for a focused `impl` block on the same type). Tests for the new G4 surface live inline in `title_style.rs` for self-containment. New file is fully focused on one concern.

**Why in-place for G5:** `status_bar/item.rs` has plenty of headroom (548 → ~620). The new fields and methods live alongside the existing `StatusBarItem` impl. Tests live in the existing test directory layout (`tests/style_item.rs` + `snapshot_tests.rs`).

---

## Build + test setup verification

Run once before Task 1 to confirm the environment is healthy:

```bash
cargo build --all-features 2>&1 | tail -5
cargo nextest run --all-features pane_layout:: 2>&1 | tail -10
cargo nextest run --all-features status_bar:: 2>&1 | tail -10
```

Expected: build succeeds; existing pane_layout + status_bar tests all pass.

---

## Task 1: G4 — `PaneConfig::with_title_style` field + new `title_style` module + render arm + tests

**Files:**
- Modify: `src/component/pane_layout/mod.rs:80-86` (struct field add + `pub mod title_style;`)
- Create: `src/component/pane_layout/title_style.rs` (new impl block + inline tests)
- Modify: `src/component/pane_layout/view_with.rs:53-55` (render arm 2-branch)

This task is a single atomic commit covering the G4 surface: field, module wiring, builder + getter in the new file, render arm update, and 4 tests.

- [ ] **Step 1: Add the `title_style` field to `PaneConfig` in `mod.rs`**

Find the `PaneConfig` struct definition (around line 75-86):

```rust
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct PaneConfig {
    id: String,
    title: Option<String>,
    proportion: f32,
    min_size: u16,
    max_size: u16,
}
```

Replace with (insert `title_style` immediately after `title`, with `#[serde(default)]` for forward-compat with pre-G5 serialized blobs):

```rust
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct PaneConfig {
    id: String,
    title: Option<String>,
    #[cfg_attr(feature = "serialization", serde(default))]
    title_style: Option<ratatui::style::Style>,
    proportion: f32,
    min_size: u16,
    max_size: u16,
}
```

The `ratatui::style::Style` qualifier is fully spelled here because the existing `use ratatui::prelude::*;` at line 39 brings `Style` into scope, but using the fully-qualified path on the struct definition keeps the change reviewable in isolation. If preferred, switch to the unqualified `Style` after verifying the import is in scope.

Find the `Self { ... }` constructor body in `PaneConfig::new` (around line 100-108):

```rust
pub fn new(id: impl Into<String>) -> Self {
    Self {
        id: id.into(),
        title: None,
        proportion: 1.0,
        min_size: 1,
        max_size: 0,
    }
}
```

Add `title_style: None,` between `title` and `proportion`:

```rust
pub fn new(id: impl Into<String>) -> Self {
    Self {
        id: id.into(),
        title: None,
        title_style: None,
        proportion: 1.0,
        min_size: 1,
        max_size: 0,
    }
}
```

- [ ] **Step 2: Wire the new `title_style` module into `mod.rs`**

Find any existing `pub mod ...` declarations in `src/component/pane_layout/mod.rs` (likely near the top of the file, around line 35-45). Add a new declaration:

```bash
grep -n "^pub mod\|^mod " src/component/pane_layout/mod.rs
```

Expected: shows `pub mod view_with;` somewhere near the top. Add `pub mod title_style;` adjacent to it (alphabetical-adjacent if the file follows that convention; otherwise just after `view_with`).

- [ ] **Step 3: Create `src/component/pane_layout/title_style.rs` with the impl block + tests**

Create new file `src/component/pane_layout/title_style.rs` with this content:

```rust
//! `PaneConfig::with_title_style` builder + getter (G4).
//!
//! Houses the title-style accessors in a separate file to keep
//! `mod.rs` under the 1000-line cap. The field declaration itself
//! lives on the `PaneConfig` struct in `mod.rs`; this module only
//! contains the inherent `impl` for accessor methods.
//!
//! See [`PaneConfig::with_title_style`] for the full contract.

use ratatui::style::Style;

use super::PaneConfig;

impl PaneConfig {
    /// Sets the title style (builder pattern).
    ///
    /// When `Some(style)`, the pane title renders with the given style instead
    /// of inheriting the border style. When `None` (default), title styling
    /// follows the border (current behavior).
    ///
    /// # Focus invariance
    ///
    /// `title_style` is focus-invariant: when set, it applies whether the pane
    /// is focused, unfocused, or disabled. The render arm consults `title_style`
    /// unconditionally — consumer-set styles aren't silently overridden by focus
    /// state. If a future use case needs focused-vs-unfocused title styling, that
    /// would be a separate builder (`with_focused_title_style`, etc.), not a
    /// surprise in the existing one.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::pane_layout::PaneConfig;
    /// use ratatui::style::{Color, Modifier, Style};
    ///
    /// let pane = PaneConfig::new("brand")
    ///     .with_title("leadline")
    ///     .with_title_style(
    ///         Style::default()
    ///             .fg(Color::Magenta)
    ///             .add_modifier(Modifier::BOLD),
    ///     );
    /// assert!(pane.title_style().is_some());
    /// ```
    pub fn with_title_style(mut self, style: Style) -> Self {
        self.title_style = Some(style);
        self
    }

    /// Returns the title style, if explicitly set.
    ///
    /// `None` means the title inherits the border style (default behavior).
    pub fn title_style(&self) -> Option<Style> {
        self.title_style
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    #[test]
    fn with_title_style_sets_field() {
        let style = Style::default().fg(Color::Magenta);
        let pane = PaneConfig::new("p").with_title("t").with_title_style(style);
        assert_eq!(pane.title_style(), Some(style));
    }

    #[test]
    fn title_style_default_none() {
        let pane = PaneConfig::new("p");
        assert_eq!(pane.title_style(), None);
    }

    #[test]
    fn snapshot_pane_with_branded_title_style() {
        use crate::component::pane_layout::{PaneDirection, PaneLayout, PaneLayoutState};
        use crate::component::test_utils::setup_render;
        use crate::component::RenderContext;
        use ratatui::style::Modifier;

        // 2-pane horizontal: left pane has a magenta+bold title; right is plain.
        let panes = vec![
            PaneConfig::new("left")
                .with_title("Brand")
                .with_title_style(
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                )
                .with_proportion(0.5),
            PaneConfig::new("right").with_title("Plain").with_proportion(0.5),
        ];
        let state = PaneLayoutState::new(PaneDirection::Horizontal, panes);

        let (mut terminal, theme) = setup_render(40, 5);
        terminal
            .draw(|frame| {
                PaneLayout::view_with(
                    &state,
                    &mut RenderContext::new(frame, frame.area(), &theme),
                    |_, _| {},
                );
            })
            .unwrap();

        let plain = terminal.backend().to_string();
        let ansi = terminal.backend().to_ansi();

        // Magenta fg = \x1b[35m; BOLD = \x1b[1m. Branded title carries both.
        assert!(
            ansi.contains("\x1b[35m"),
            "expected magenta (35m) for branded title, got:\n{ansi}",
        );
        assert!(
            ansi.contains("\x1b[1m"),
            "expected BOLD (1m) for branded title, got:\n{ansi}",
        );

        insta::assert_snapshot!(plain);
    }

    #[test]
    fn snapshot_pane_title_style_focus_invariant() {
        use crate::component::pane_layout::{PaneDirection, PaneLayout, PaneLayoutState};
        use crate::component::test_utils::setup_render;
        use crate::component::RenderContext;

        // 2-pane horizontal: BOTH panes get the same with_title_style(magenta).
        // One pane is focused, the other isn't. The title rendering must be
        // identical regardless of focus — title_style wins over focus-driven
        // border style adjustments.
        let style = Style::default().fg(Color::Magenta);
        let panes = vec![
            PaneConfig::new("focused")
                .with_title("F")
                .with_title_style(style)
                .with_proportion(0.5),
            PaneConfig::new("unfocused")
                .with_title("U")
                .with_title_style(style)
                .with_proportion(0.5),
        ];
        let mut state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
        state.focused_pane = 0; // focus the left pane

        let (mut terminal, theme) = setup_render(40, 5);
        terminal
            .draw(|frame| {
                let mut ctx = RenderContext::new(frame, frame.area(), &theme);
                ctx.focused = true;
                PaneLayout::view_with(&state, &mut ctx, |_, _| {});
            })
            .unwrap();

        let ansi = terminal.backend().to_ansi();
        let plain = terminal.backend().to_string();

        // The magenta escape \x1b[35m must appear at least twice in the ANSI
        // output — once per title. If focus-driven border-style adjustments
        // were inadvertently overriding title_style, the two titles would
        // render differently and only one would carry magenta.
        let magenta_count = ansi.matches("\x1b[35m").count();
        assert!(
            magenta_count >= 2,
            "expected magenta (35m) at least twice (once per title regardless of focus), got {magenta_count} occurrences:\n{ansi}",
        );

        insta::assert_snapshot!(plain);
    }
}
```

The `state.focused_pane = 0` access in the focus-invariant test assumes that field is accessible from outside the module. If it's private, use a public setter (e.g. `state.set_focused_pane(0)` if one exists, or `state.focused_pane(0)`) — adjust to whatever the public API exposes. Verify by `grep -n "pub fn focused_pane\|pub focused_pane" src/component/pane_layout/mod.rs`; if no public setter, use whatever public mutation is available, or default the layout to focus-on-first.

- [ ] **Step 4: Update the render arm in `view_with.rs`**

In `src/component/pane_layout/view_with.rs`, find the title-rendering block (around line 53-55):

```rust
if let Some(title) = &pane.title {
    block = block.title(format!(" {} ", title));
}
```

Replace with:

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

The `Span::raw` branch preserves current behavior (title inherits border style via ratatui's default styling on `Block::title`). The `Span::styled` branch applies the consumer-provided style explicitly.

- [ ] **Step 5: Verify build is clean**

Run: `cargo build --all-features 2>&1 | tail -5`
Expected: clean build, no warnings.

- [ ] **Step 6: Run G4 tests + snapshots**

Run: `cargo nextest run --all-features pane_layout::title_style:: 2>&1 | tail -15`
Expected: 4 PASS (with_title_style_sets_field, title_style_default_none, snapshot_pane_with_branded_title_style, snapshot_pane_title_style_focus_invariant).

If insta prompts for new snapshots, accept: `cargo insta accept`.

Run: `cargo nextest run --all-features pane_layout:: 2>&1 | tail -5`
Expected: all pane_layout tests pass (existing + 4 new).

- [ ] **Step 7: Run doc test**

Run: `cargo test --all-features --doc pane_layout::title_style 2>&1 | tail -10`
Expected: doc test on `with_title_style` PASSes.

- [ ] **Step 8: Verify file-size constraint**

Run: `wc -l src/component/pane_layout/*.rs`
Expected: `mod.rs` under 1000 lines (target: ~980); new `title_style.rs` < 250 lines.

- [ ] **Step 9: Commit (G4 atomic)**

```bash
git add src/component/pane_layout/
git commit -S -m "Add PaneConfig::with_title_style (G4)

Per-pane title styling independent of border. Adds title_style: Option<Style>
field to PaneConfig (with #[serde(default)] for serialization forward-compat).
New builder + getter live in src/component/pane_layout/title_style.rs to keep
mod.rs under the 1000-line cap — same multi-file impl pattern as view_with.rs.

Render arm in view_with.rs consults pane.title_style unconditionally:
Some(style) -> Span::styled, None -> Span::raw (preserves current border-style
inheritance behavior).

title_style is focus-invariant by design — consumer-set styles aren't silently
overridden by focus state. Docstring + snapshot_pane_title_style_focus_invariant
test pin the contract for future contributors.

Tests (4 in inline #[cfg(test)] mod tests):
- with_title_style_sets_field
- title_style_default_none
- snapshot_pane_with_branded_title_style (magenta+BOLD branded title)
- snapshot_pane_title_style_focus_invariant (same style across focus states)"
```

---

## Task 2: G5 — `StatusBarItem::with_color` + `with_style_override` fields + render arm + tests

**Files:**
- Modify: `src/component/status_bar/item.rs:205` (struct + impl block)
- Modify: `src/component/status_bar/mod.rs:664` (render arm)
- Modify: `src/component/status_bar/tests/style_item.rs` (4 unit tests)
- Modify: `src/component/status_bar/snapshot_tests.rs` (1 snapshot test)

This task is a single atomic commit covering the G5 surface: two new fields with `#[serde(default)]`, four new methods (2 builders + 2 getters), the three-branch render arm update, and 5 tests.

- [ ] **Step 1: Add the two new fields to `StatusBarItem` in `item.rs`**

Find the `StatusBarItem` struct definition (around line 199-212):

```rust
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct StatusBarItem {
    pub(super) content: StatusBarItemContent,
    pub(super) style: StatusBarStyle,
    separator: bool,
}
```

Replace with (insert `color` and `style_override` between `style` and `separator`):

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
    #[cfg_attr(feature = "serialization", serde(default))]
    pub(super) color: Option<ratatui::style::Color>,
    /// Layer 3: full `Style` override. Wins over `color` and `style.style(theme)`.
    /// `None` defers to `color`.
    #[cfg_attr(feature = "serialization", serde(default))]
    pub(super) style_override: Option<ratatui::style::Style>,
    separator: bool,
}
```

Both new fields are `pub(super)` (same visibility as existing fields). `#[serde(default)]` on each preserves forward-compat with pre-G5 serialized blobs.

- [ ] **Step 2: Update all existing `Self { ... }` constructors in `item.rs`**

`StatusBarItem` has multiple constructor methods (e.g., `new`, plus the elapsed-time and timestamp constructors). Each one constructs `Self { ... }`. Add `color: None, style_override: None,` to every constructor body.

Run:
```bash
grep -n "Self {" src/component/status_bar/item.rs
```

Expected: shows ~3-5 sites (constructor bodies). For each, add the two new field initializers. Example for the `new` constructor at line 225:

```rust
pub fn new(text: impl Into<String>) -> Self {
    Self {
        content: StatusBarItemContent::Static(text.into()),
        style: StatusBarStyle::Default,
        color: None,
        style_override: None,
        separator: true,
    }
}
```

Apply the same `color: None, style_override: None,` pair to each of the other constructor `Self { ... }` blocks. Run `cargo build --all-features 2>&1 | tail -10` after the edits; if any constructor is missed, the compiler will report it (missing field error).

- [ ] **Step 3: Add the two new builders + two new getters in the `impl StatusBarItem` block**

Find the existing `pub fn with_style(...)` method in `impl StatusBarItem` (around line 372-375):

```rust
pub fn with_style(mut self, style: StatusBarStyle) -> Self {
    self.style = style;
    self
}
```

Append the four new methods immediately after `with_style`. The location keeps the style-related setters and getters clustered together:

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
///
/// let theme = Theme::catppuccin_mocha();
/// let item = StatusBarItem::new("slowdown")
///     .with_color(theme.severity_color(Severity::Bad));
/// // Renders in the theme's Peach (full four-stop severity ramp).
/// assert!(item.color().is_some());
/// ```
pub fn with_color(mut self, color: ratatui::style::Color) -> Self {
    self.color = Some(color);
    self
}

/// Returns the color override, if explicitly set.
pub fn color(&self) -> Option<ratatui::style::Color> {
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
///             .add_modifier(Modifier::BOLD),
///     );
/// assert!(item.style_override().is_some());
/// ```
pub fn with_style_override(mut self, style: ratatui::style::Style) -> Self {
    self.style_override = Some(style);
    self
}

/// Returns the style override, if explicitly set.
pub fn style_override(&self) -> Option<ratatui::style::Style> {
    self.style_override
}
```

The `ratatui::style::Color` and `ratatui::style::Style` fully-qualified paths are used for the same reason as in G4 — the `use ratatui::prelude::*;` at line 7 brings them into scope, but the fully-qualified path keeps the signature self-documenting and survives any future import refactor.

- [ ] **Step 4: Update the render arm in `status_bar/mod.rs:664`**

Find the line:

```rust
let style = item.style.style(theme);
```

(verify location via `grep -n "item.style.style(theme)" src/component/status_bar/mod.rs`)

Replace with the three-branch precedence ladder:

```rust
let style = if let Some(s) = item.style_override {
    s
} else if let Some(c) = item.color {
    ratatui::style::Style::default().fg(c)
} else {
    item.style.style(theme)
};
```

The `ratatui::style::Style::default()` is fully qualified for the same reason; if `Style` is already in scope (likely via the `use ratatui::prelude::*;` near the top of mod.rs), the unqualified `Style::default()` works equivalently.

- [ ] **Step 5: Add 4 unit tests to `tests/style_item.rs`**

Append to `src/component/status_bar/tests/style_item.rs`:

```rust
#[test]
fn with_color_sets_color_only() {
    use ratatui::style::Color;

    let item = crate::component::StatusBarItem::new("x").with_color(Color::Red);
    assert_eq!(item.color(), Some(Color::Red));
    assert_eq!(item.style_override(), None);
    // Semantic baseline unchanged.
    assert_eq!(
        item.style(),
        crate::component::StatusBarStyle::Default,
    );
}

#[test]
fn with_style_override_sets_override_only() {
    use ratatui::style::{Color, Style};

    let s = Style::default().fg(Color::Magenta).bg(Color::Black);
    let item = crate::component::StatusBarItem::new("x").with_style_override(s);
    assert_eq!(item.style_override(), Some(s));
    assert_eq!(item.color(), None);
}

#[test]
fn with_color_then_with_style_override_preserves_color() {
    // Layered semantics, NOT last-call-wins: setting style_override does
    // not clear a prior color. Each setter writes its own field; render-
    // time precedence picks. This test pins the G5 contract.
    use ratatui::style::{Color, Modifier, Style};

    let override_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
    let item = crate::component::StatusBarItem::new("x")
        .with_style(crate::component::StatusBarStyle::Info)
        .with_color(Color::Red)
        .with_style_override(override_style);

    // All three fields populated; render picks per precedence at draw time.
    assert_eq!(
        item.style(),
        crate::component::StatusBarStyle::Info,
    );
    assert_eq!(item.color(), Some(Color::Red));
    assert_eq!(item.style_override(), Some(override_style));
}

#[test]
fn branched_construction_preserves_color() {
    // The practical case that decides layered vs last-call-wins: build an
    // item with a brand color, then CONDITIONALLY apply an override. After
    // the conditional, color is still set even though style_override is
    // also set. Render-time precedence gives the override priority while
    // the brand color stays rebuildable.
    use ratatui::style::{Color, Style};

    let brand = Color::Rgb(0xB4, 0xBE, 0xFE); // Lavender
    let mut item = crate::component::StatusBarItem::new("leadline").with_color(brand);

    let user_wants_emphasis = true;
    if user_wants_emphasis {
        item = item.with_style_override(Style::default().fg(Color::Red));
    }

    assert_eq!(
        item.color(),
        Some(brand),
        "branded color must persist through subsequent with_style_override",
    );
    assert!(item.style_override().is_some());
}
```

- [ ] **Step 6: Add 1 snapshot test for the four-stop severity ramp to `snapshot_tests.rs`**

Append to `src/component/status_bar/snapshot_tests.rs`:

```rust
#[test]
fn snapshot_status_bar_four_stop_severity_ramp() {
    use crate::component::status_bar::{StatusBar, StatusBarItem, StatusBarState};
    use crate::component::test_utils::setup_render;
    use crate::component::RenderContext;
    use crate::theme::{Severity, Theme};

    // The Q-gamma payoff: four distinct ANSI fg colors when each Severity
    // band gets its own theme.severity_color. Pre-G5, the consumer-side
    // severity_status_style helper collapsed Bad+Mild -> Warning because
    // StatusBarStyle had no Peach variant. Post-G5, with_color + the
    // theme palette restores the full four-stop ramp.
    let theme = Theme::catppuccin_mocha();
    let items = vec![
        StatusBarItem::new("good").with_color(theme.severity_color(Severity::Good)),
        StatusBarItem::new("mild").with_color(theme.severity_color(Severity::Mild)),
        StatusBarItem::new("bad").with_color(theme.severity_color(Severity::Bad)),
        StatusBarItem::new("crit").with_color(theme.severity_color(Severity::Critical)),
    ];
    let state = StatusBarState::new().with_left(items);

    // Render with the actual Catppuccin theme so the four colors are
    // distinct RGB values, not basic-Color collapses.
    let (mut terminal, _) = setup_render(40, 1);
    terminal
        .draw(|frame| {
            StatusBar::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme),
            );
        })
        .unwrap();

    let plain = terminal.backend().to_string();
    let ansi = terminal.backend().to_ansi();

    // ratatui emits RGB foregrounds as \x1b[38;2;R;G;Bm. Each of the four
    // bands should produce a distinct escape. Catppuccin: Good=Green (166,
    // 227, 161), Mild=Yellow (249, 226, 175), Bad=Peach (250, 179, 135),
    // Critical=Red (243, 139, 168).
    assert!(
        ansi.contains("\x1b[38;2;166;227;161m"),
        "expected Catppuccin Green for Severity::Good, got:\n{ansi}",
    );
    assert!(
        ansi.contains("\x1b[38;2;249;226;175m"),
        "expected Catppuccin Yellow for Severity::Mild, got:\n{ansi}",
    );
    assert!(
        ansi.contains("\x1b[38;2;250;179;135m"),
        "expected Catppuccin Peach for Severity::Bad (the restored band), got:\n{ansi}",
    );
    assert!(
        ansi.contains("\x1b[38;2;243;139;168m"),
        "expected Catppuccin Red for Severity::Critical, got:\n{ansi}",
    );

    insta::assert_snapshot!(plain);
}
```

The `StatusBarState::new().with_left(items)` API assumes a `with_left` builder exists. Verify: `grep -n "pub fn with_left\|pub fn new" src/component/status_bar/mod.rs | head -5`. If the actual API differs (e.g., `StatusBarState::with_items(items)` or similar), adjust to the actual public surface. The semantic intent is "construct a state with these items in the left section."

- [ ] **Step 7: Verify build is clean**

Run: `cargo build --all-features 2>&1 | tail -5`
Expected: clean build, no warnings.

If the compiler reports missing fields in any `Self { ... }` constructor in `item.rs`, fix per Step 2 (add `color: None, style_override: None,` to that constructor's body).

- [ ] **Step 8: Run G5 tests**

Run: `cargo nextest run --all-features status_bar::tests::style_item 2>&1 | tail -15`
Expected: 4 new tests PASS plus existing tests in style_item.rs.

Run: `cargo nextest run --all-features status_bar::snapshot_tests::snapshot_status_bar_four_stop_severity_ramp 2>&1 | tail -10`
Expected: PASS. New snapshot accepted via `cargo insta accept` on first run.

Run: `cargo nextest run --all-features status_bar:: 2>&1 | tail -5`
Expected: all status_bar tests pass (existing + 5 new).

- [ ] **Step 9: Run doc tests**

Run: `cargo test --all-features --doc status_bar 2>&1 | tail -10`
Expected: doc tests on `with_color` and `with_style_override` PASS.

- [ ] **Step 10: Run full test suite for no regressions**

Run: `cargo nextest run --all-features 2>&1 | tail -5`
Expected: all tests pass.

- [ ] **Step 11: Commit (G5 atomic)**

```bash
git add src/component/status_bar/
git commit -S -m "Add StatusBarItem::with_color + with_style_override (G5)

Per-item StatusBar coloring with layered precedence. Adds color:
Option<Color> and style_override: Option<Style> fields to StatusBarItem
(both with #[serde(default)] for serialization forward-compat). Plus four
new methods: with_color, with_style_override, color(), style_override().

Render-arm at status_bar/mod.rs:664 becomes a three-branch precedence
ladder: style_override > color > style.style(theme).

LAYERED setter semantics (not last-call-wins): each setter writes its own
field idempotently; render-time precedence picks. Setting with_style_override
does NOT clear a prior with_color, and vice versa. Docstrings document the
precedence + the layered model explicitly. Tests pin the contract:

- with_color_sets_color_only: with_color sets color, leaves override None
- with_style_override_sets_override_only: with_style_override sets override,
  leaves color None
- with_color_then_with_style_override_preserves_color: layered, not
  last-call-wins
- branched_construction_preserves_color: the practical case (conditional
  override application) where last-call-wins would lose the brand color

Plus snapshot_status_bar_four_stop_severity_ramp: renders the Q-gamma
payoff — four distinct Catppuccin RGB foregrounds for Severity::Good /
Mild / Bad / Critical via with_color(theme.severity_color(sev)). The
four-band gradient deferred by D6+D9 is now restored on full-palette
themes; the severity_status_style consumer helper that collapsed Bad+Mild
to Warning deletes after this PR."
```

---

## Task 3: Verify clippy + fmt + doc + audit

**Files:** (none — verification only)

- [ ] **Step 1: Run clippy**

Run: `cargo clippy --all-features --all-targets -- -D warnings 2>&1 | tail -10`
Expected: clean.

If warnings appear, fix in-place. Common ones to watch for: unused imports if the fully-qualified `ratatui::style::Style` and `ratatui::style::Color` paths render the prelude imports redundant in the specific method bodies.

- [ ] **Step 2: Run rustdoc with deny-warnings**

Run: `RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps 2>&1 | tail -10`
Expected: clean.

- [ ] **Step 3: Run cargo fmt check**

Run: `cargo fmt --all -- --check 2>&1 | wc -l`
Expected: `0`.

If formatting drift, run `cargo fmt --all` and commit:

```bash
git add -A
git commit -S -m "Apply cargo fmt"
```

- [ ] **Step 4: Run full test suite + doc tests**

```bash
cargo nextest run --all-features 2>&1 | tail -5
cargo test --all-features --doc 2>&1 | tail -3
```

Expected: all pass.

- [ ] **Step 5: Run audit scorecard**

Run: `./tools/audit/target/release/envision-audit scorecard 2>&1 | grep "Result:"`
Expected: `Result: 8/9 checks passing` — same baseline as main. No regression. The pre-existing `resource_gauge::set_values has no matching getter` gap is unchanged.

- [ ] **Step 6: Verify file-size constraint across all touched files**

```bash
wc -l src/component/pane_layout/*.rs src/component/status_bar/*.rs
```

Expected:
- `pane_layout/mod.rs` < 1000 (target ~980)
- `pane_layout/title_style.rs` < 250
- `pane_layout/view_with.rs` < 200
- `status_bar/item.rs` < 700
- `status_bar/mod.rs` < 1000
- All test files unchanged or under cap

- [ ] **Step 7: Commit if any fixes were needed**

If any verification step required a fix, commit it. Otherwise no commit — Task 3 is verification-only.

---

## Task 4: CHANGELOG entry

**Files:** Modify `CHANGELOG.md`

- [ ] **Step 1: Add the entry under `## [Unreleased]`**

Open `CHANGELOG.md`. Find the `## [Unreleased]` section. Below the most recent sub-section (likely "StyledText DX: line primitive + `paragraph` rename (D5 + D14)" or similar from PR #482), add this new sub-section:

```markdown
### Per-component style overrides (G4 + G5)

Two coupled parent-side style hooks land together. Both restore consumer
flexibility that was previously bottlenecked by closed-enum or border-inheritance
constraints.

**G4 — `PaneConfig::with_title_style(Style)`:**

- New builder + getter: `with_title_style(self, style: Style)` and
  `title_style(&self) -> Option<Style>`. When set, the pane title renders
  with the given style; when `None` (default), the title inherits the
  border style (current behavior).
- Focus-invariant by design: consumer-set title styles aren't silently
  overridden by focus state. A future focused-vs-unfocused title style
  would be a separate builder, not a surprise in this one.
- New file `src/component/pane_layout/title_style.rs` houses the impl +
  inline tests (keeps `mod.rs` under the 1000-line cap; mirrors the
  existing `view_with.rs` split pattern).

**G5 — `StatusBarItem::with_color(Color)` + `with_style_override(Style)`:**

- Two new layered builders + getters. Render-time precedence:
  `style_override > color > style.style(theme)`.
- **Layered semantics, not last-call-wins:** each setter writes its own
  field idempotently. Calling `with_style_override(s)` does NOT clear a
  prior `with_color(c)`; the override just wins until cleared. Branched
  construction (`if user_wants_emphasis { item.with_style_override(s) }
  else { item }`) keeps the brand color rebuildable.
- `with_color(c)` produces `Style::default().fg(c)` — clean separation;
  background and modifiers are not inherited from the semantic baseline.
  Consumers wanting layered semantics reach for `with_style_override`
  explicitly.

**Q-γ payoff — four-stop severity ramp restored:** The D6+D9 design
deferred the StatusBar four-stop severity ramp because `StatusBarStyle`
had no Peach variant — leadline's `severity_status_style` consumer
helper collapsed `Severity::Bad` and `Severity::Mild` both to
`StatusBarStyle::Warning`. Post-G5, the helper deletes; the call site
uses `StatusBarItem::new(t).with_color(theme.severity_color(sev))`
directly. Three convergence views (table cells via D15
`CellStyle::Severity`, summary banner via D5 `styled_line` +
`theme.severity_color`, StatusBar slowdown segments via G5
`with_color`) reach the same four-stop gradient.

**Field-add safety:** `PaneConfig` fields were already private and
`StatusBarItem` fields were `pub(super)`, so external consumers can't
struct-literal-construct either struct. The only forward-compat concern
is serialization: pre-G5 serialized blobs lack the new `title_style`
/ `color` / `style_override` fields, and `#[serde(default)]` on each
new field handles round-tripping cleanly. No struct-literal break for
external code.
```

- [ ] **Step 2: Commit**

```bash
git add CHANGELOG.md
git commit -S -m "CHANGELOG: Per-component style overrides (G4 + G5)

Document the new PaneConfig::with_title_style and StatusBarItem::
with_color / with_style_override builders. Flag the layered (not
last-call-wins) setter semantics, the focus-invariant contract on
title style, and the Q-gamma four-stop severity ramp restoration.
Note that field additions are safe for external consumers (fields
were already private/pub(super)) and that #[serde(default)] handles
serialization forward-compat."
```

---

## Task 5: Final verification + push + open PR

**Files:** (none — verification + git only)

- [ ] **Step 1: Verify all commits are signed**

Run: `git log --show-signature main..HEAD 2>&1 | grep -c 'Good signature'`
Expected: count matches the number of commits added on this branch (3 or 4 depending on whether Task 3 needed a fix commit).

If any commit is unsigned, **stop** and ask the user how to handle it — never bypass.

- [ ] **Step 2: Run the full verification gauntlet**

```bash
cargo build --all-features 2>&1 | tail -1
cargo clippy --all-features --all-targets -- -D warnings 2>&1 | tail -1
cargo fmt --all -- --check 2>&1 | wc -l
cargo nextest run --all-features 2>&1 | tail -3
cargo test --all-features --doc 2>&1 | tail -2
RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps 2>&1 | tail -1
./tools/audit/target/release/envision-audit scorecard 2>&1 | grep "Result:"
```

Expected: every command succeeds. Audit shows `Result: 8/9 checks passing` (same baseline as main).

- [ ] **Step 3: Verify no-default-features build**

Run: `cargo build --no-default-features 2>&1 | tail -3`
Expected: clean. This catches feature-gate issues like the one surfaced during D5+D14 (PR #482).

- [ ] **Step 4: Push the branch**

Run: `git push -u origin per-component-style-overrides-impl`

(Branch name is `per-component-style-overrides-impl` — the implementation branch, set by the controller before plan execution begins. The current plan branch is `per-component-style-overrides-plan`.)

Expected: pushes cleanly.

- [ ] **Step 5: Open the implementation PR**

Run:

```bash
gh pr create --title "Per-component style overrides (G4 + G5)" --body "$(cat <<'EOF'
## Summary

Implementation of leadline gaps **G4** (\`PaneConfig::with_title_style\`) and **G5** (\`StatusBarItem::with_color\` + \`with_style_override\`).

Spec: PR #485 (\`docs/superpowers/specs/2026-05-19-per-component-style-overrides-design.md\`)
Plan: PR β (\`docs/superpowers/plans/2026-05-19-per-component-style-overrides.md\`)

## What changed

**G4 — Pane title styling:**
- New \`title_style: Option<Style>\` field on \`PaneConfig\` (with \`#[serde(default)]\`).
- New builder + getter in \`src/component/pane_layout/title_style.rs\` (new sibling file — mirrors \`view_with.rs\` split to keep \`mod.rs\` under the 1000-line cap).
- Render arm in \`view_with.rs\` consults the field unconditionally — focus-invariant by design.

**G5 — StatusBar layered coloring:**
- Two new layered \`Option\` fields on \`StatusBarItem\` (\`color\`, \`style_override\`) plus four new methods.
- Three-branch render precedence at \`status_bar/mod.rs:664\`: \`style_override > color > style.style(theme)\`.
- **Layered setter semantics, not last-call-wins** — branched construction patterns stay predictable.
- \`with_color(c)\` produces \`Style::default().fg(c)\` (no theme-bg layering).

**Q-γ payoff:** The four-stop severity ramp deferred by D6+D9 is now restored. \`StatusBarItem::new(t).with_color(theme.severity_color(sev))\` distinguishes all four \`Severity\` bands on full-palette themes. Three convergence views (D15 cells, D5 banner, G5 status) reach the same gradient.

## Stats

- 5 signed commits (Tasks 1, 2, 4 produce commits; Task 3 verification-only; Task 5 push+PR; one possible fmt-cleanup commit)
- 9 new tests: 4 in pane_layout/title_style.rs (inline), 4 in status_bar/tests/style_item.rs, 1 snapshot in status_bar/snapshot_tests.rs
- New file: \`src/component/pane_layout/title_style.rs\` (~200 lines)
- File-size delta: \`pane_layout/mod.rs\` 975 → ~980; \`status_bar/item.rs\` 548 → ~620; \`status_bar/mod.rs\` 913 → ~917. All under cap.

## Verification

- \`cargo build --all-features\`: clean
- \`cargo build --no-default-features\`: clean
- \`cargo clippy --all-features --all-targets -- -D warnings\`: clean
- \`cargo fmt --all -- --check\`: clean
- \`cargo nextest run --all-features\`: all passing
- \`cargo test --all-features --doc\`: clean
- \`RUSTDOCFLAGS=\"-D warnings\" cargo doc --all-features --no-deps\`: clean
- \`./tools/audit/target/release/envision-audit scorecard\`: 8/9 (same baseline as main)

## Test plan

- [ ] CI green on all platforms
- [ ] leadline migrates \`severity_status_style\` consumer helper — deletes entirely; replaces call sites with \`StatusBarItem::new(t).with_color(theme.severity_color(Severity::from_thresholds(ratio, ...)))\`. Adds branded pane title via \`with_title_style\` if desired.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

Expected: PR URL printed.

---

## Self-review (controller runs before dispatch)

### 1. Spec coverage

| Spec section | Tasks |
|---|---|
| G4: `PaneConfig::title_style: Option<Style>` field with `#[serde(default)]` | Task 1 (Step 1) |
| G4: `with_title_style` builder + `title_style()` getter | Task 1 (Step 3) |
| G4: Render arm 2-branch (`Some` → `Span::styled`, `None` → `Span::raw`) | Task 1 (Step 4) |
| G4: 4 tests (with_title_style, default None, branded snapshot, focus-invariant snapshot) | Task 1 (Step 3, inline tests) |
| G5: `color: Option<Color>` + `style_override: Option<Style>` fields with `#[serde(default)]` | Task 2 (Step 1) |
| G5: 4 new methods (2 builders + 2 getters) | Task 2 (Step 3) |
| G5: 3-branch render precedence | Task 2 (Step 4) |
| G5: Layered semantics (not last-call-wins) documented + tested | Task 2 (Step 3 docstrings + Step 5 tests) |
| G5: 5 tests (with_color, with_style_override, layered preserves color, branched preserves color, four-stop snapshot) | Task 2 (Steps 5 + 6) |
| Focus-invariance contract for `with_title_style` (leadline soft note #2) | Task 1 (Step 3 docstring + 4th test) |
| Field-add safety language tightened (leadline soft note #1) | Task 4 (CHANGELOG: "Field-add safety" section) |
| CHANGELOG entry under `[Unreleased]` | Task 4 |
| Q-γ four-stop ramp ANSI distinctness | Task 2 (Step 6 snapshot test) |

All spec requirements have a corresponding task.

### 2. Placeholder scan

No "TBD", "TODO", "implement later", "fill in details". Every step has either complete code, exact commands, or explicit verification criteria. The two "verify the API" notes (Step 3's `state.focused_pane = 0` access; Step 6's `StatusBarState::new().with_left(items)` API) are explicit verification steps with grep commands and fallback guidance — not placeholders.

### 3. Type consistency

- `ratatui::style::Style` fully-qualified in struct field + signatures + match arms; all uses match. ✅
- `ratatui::style::Color` fully-qualified in struct field + signatures; all uses match. ✅
- `Option<Style>` and `Option<Color>` shapes consistent across PaneConfig (1 field) and StatusBarItem (2 fields). ✅
- Method signatures match the spec: `with_title_style(self, style: Style) -> Self`, `title_style(&self) -> Option<Style>`, `with_color(self, color: Color) -> Self`, `color(&self) -> Option<Color>`, `with_style_override(self, style: Style) -> Self`, `style_override(&self) -> Option<Style>`. ✅
- ANSI escape codes consistent: `\x1b[35m` magenta, `\x1b[1m` BOLD; Catppuccin RGB triples for the four-stop snapshot are sourced from `src/theme/catppuccin.rs` (Green = 166,227,161; Yellow = 249,226,175; Peach = 250,179,135; Red = 243,139,168). ✅

---

## Plan complete

The plan covers 5 tasks producing approximately 3-4 signed commits (Tasks 1, 2, 4 commit; Task 3 verification-only; Task 5 push + PR; optional fmt-cleanup commit if Task 3 surfaces drift). Estimated implementation time: 2-3 hours of focused work.

After plan PR β merges, controller creates `per-component-style-overrides-impl` branch from main and dispatches subagent-driven execution. After implementation PR γ merges, controller opens tracking-doc PR δ marking G4 + G5 ✅ resolved in `docs/customer-feedback/2026-05-01-leadline-gaps.md`.
