# `StyledInline` composable styles (G6) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the 7-variant `StyledInline` enum with a 3-variant shape (`Plain | Code | Styled { text, style: InlineStyle }`) so consumers can combine dimensions in a single inline run, restoring bold-on-banner-values in leadline's per-op summary banner.

**Architecture:** Additive-first migration in 3 phases for clean bisect granularity. Phase 1 lands the new surface (InlineStyle struct + Styled variant + 6 constructors + 7 const fn builder methods) alongside the existing 5 leaf variants — both APIs coexist. Phase 2 migrates 102 internal envision references mechanically to the new helpers. Phase 3 deletes the 5 leaf variants + simplifies the render path. Compiler enforces migration completeness in Phase 3; if any site was missed, compilation fails. `#[non_exhaustive]` on the enum and InlineStyle struct.

**Tech Stack:** Rust 1.85+, ratatui 0.29 (`Color`, `Style`, `Modifier::{BOLD, ITALIC, UNDERLINED, CROSSED_OUT}`), insta for snapshot tests.

**Spec:** `docs/superpowers/specs/2026-05-20-styled-inline-compose-design.md` (PR #489, merged)

---

## Pre-execution gotchas

- **Signed commits required.** Project rule. If `git commit -S` fails, ask the user; never bypass with `--no-gpg-sign`.
- **`Modifier::CROSSED_OUT`, not `STRIKETHROUGH`.** ratatui names the strikethrough modifier `CROSSED_OUT`. Plan bakes this into the render-arm comment per leadline soft note #1 so future contributors don't hit the source-spelunking surprise.
- **`#[non_exhaustive]` on `InlineStyle` struct blocks external struct-literal construction.** Intentional — forces builder use. Internal envision code (this PR) uses `InlineStyle::new()...` everywhere; no struct-literal sites exist in envision today.
- **`InlineStyle` derives `Eq`.** All fields must be `Eq`: `Option<Color>` is `Eq` because ratatui `Color` is `Eq`; `bool` is `Eq`. Verified safe. The `Style` type (ratatui's full style struct) is NOT `Eq` — but we don't use `Style` inside `InlineStyle`.
- **All 7 builder methods on `InlineStyle` must be `const fn`.** Plan verifies via a const test (`const _: InlineStyle = InlineStyle::new().bold().fg(Color::Red);`) so this constraint is mechanical.
- **3-phase deletion ordering matters.** Phase 3 (delete leaf variants) MUST run after Phase 2 (migrate sites). Skipping or reordering produces compile failures. Each Phase is one commit; clean bisect.
- **Insta snapshot stability for single-dimension cases.** Existing tests that used `StyledInline::Bold(t)` migrate to `StyledInline::bold(t)`. The rendered output is byte-identical (same `Span::styled(t, base.add_modifier(BOLD))` produced). If any snapshot diffs in Phase 2, investigate before accepting — the migration should preserve all single-dimension rendering.
- **Verified call-site counts (re-grep'd 2026-05-20):**
  - `examples/styling_showcase.rs` — 34
  - `src/component/styled_text/tests.rs` — 28
  - `src/component/styled_text/content.rs` — 17 (mostly doc-test examples)
  - `src/render.rs` — 11 (test fixtures)
  - `examples/styled_text.rs` — 10
  - `src/component/styled_text/mod.rs` — 2
  - **Total: 102 references across 6 files**
- **File-size headroom.** `content.rs` is 526 lines; projected ~600 post-G6 (net +~80). `tests.rs` is 713 lines; projected ~860 with 8 new tests + 28 migrated tests. Both under cap.
- **No-default-features.** `cargo build --no-default-features` must pass (preventive check from D5+D14 lesson; `styled_text` is gated behind `display-components`).
- **Audit baseline 8/9.** Resource_gauge gap is pre-existing on main. New public functions need doc tests (lesson from G4+G5 PR #487's audit regression).

---

## File Structure

| File | Responsibility | Lines (before → after) |
|---|---|---|
| `src/component/styled_text/content.rs` | `StyledInline` enum + new `InlineStyle` struct + all builder methods + 6 constructor helpers + simplified render path | 526 → ~600 |
| `src/component/styled_text/tests.rs` | Migrate 28 leaf-variant references to helpers + add 8 new tests for new surface | 713 → ~860 |
| `src/component/styled_text/mod.rs` | Migrate 2 doc-example references | unchanged size |
| `src/render.rs` | Migrate 11 test-fixture references | minor delta |
| `examples/styling_showcase.rs` | Migrate 34 references | unchanged size |
| `examples/styled_text.rs` | Migrate 10 references | unchanged size |
| `CHANGELOG.md` | Additive entry under `[Unreleased]` | adds ~30 lines |

All files stay well under the 1000-line cap.

---

## Build + test setup verification

Run once before Task 1 to confirm the environment is healthy:

```bash
cargo build --all-features 2>&1 | tail -5
cargo build --no-default-features 2>&1 | tail -3
cargo nextest run --all-features styled_text:: 2>&1 | tail -10
```

Expected: build succeeds in both feature configurations; existing styled_text tests all pass.

---

## Task 1: Phase 1 — add new surface alongside existing leaf variants

**Files:**
- Modify: `src/component/styled_text/content.rs` (add `InlineStyle` struct + `Styled` variant + helpers; leaf variants stay)

This task lands the additive surface only. The new `InlineStyle` struct, the `Styled { text, style }` variant, and the 6 constructor helpers all land. The 5 existing leaf variants (Bold, Italic, Underline, Strikethrough, Colored) stay in place. Both APIs coexist. No migration. No deletion. Render arms for the old variants are unchanged; a new arm is added for `Styled`.

After this task: codebase compiles cleanly; old behavior preserved; new surface available but unused internally.

- [ ] **Step 1: Add `#[non_exhaustive]` to the `StyledInline` enum + new `Styled` variant**

In `src/component/styled_text/content.rs`, find the enum definition (lines 43-67). Update to:

```rust
/// An inline styling element within a paragraph or list item.
///
/// `#[non_exhaustive]` so envision can add inline variants later without
/// breaking downstream `match` arms in consumer crates.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum StyledInline {
    /// Plain unstyled text.
    Plain(String),
    /// Bold text.
    Bold(String),
    /// Italic text.
    Italic(String),
    /// Underlined text.
    Underline(String),
    /// Strikethrough text.
    Strikethrough(String),
    /// Text with explicit foreground and/or background colors.
    Colored {
        /// The text content.
        text: String,
        /// Optional foreground color.
        fg: Option<Color>,
        /// Optional background color.
        bg: Option<Color>,
    },
    /// Inline code (displayed with distinct styling).
    Code(String),
    /// Styled run combining color, modifiers, and optional background.
    ///
    /// The composable form. Use [`StyledInline::styled`] or one of the
    /// leaf-helper constructors (`bold`, `italic`, `underlined`,
    /// `strikethrough`, `colored`) to construct.
    Styled {
        /// The text content.
        text: String,
        /// Style dimensions applied on top of the surrounding base style.
        style: InlineStyle,
    },
}
```

- [ ] **Step 2: Add the `InlineStyle` struct + builder impl**

Append immediately after the `StyledInline` enum definition (around line 68):

```rust
/// Style dimensions for a styled inline run.
///
/// All dimensions are optional and compose freely. Use [`InlineStyle::new`]
/// + builder methods (`fg`, `bg`, `bold`, `italic`, `underlined`,
/// `strikethrough`) to construct; struct-literal construction is
/// intentionally not supported (`#[non_exhaustive]`) so future modifier
/// additions land additively without breaking consumers.
///
/// All builder methods are `const fn` — `InlineStyle` chains can be used
/// in `const` contexts (e.g., module-level static styles).
///
/// # Example
///
/// ```rust
/// use envision::component::styled_text::InlineStyle;
/// use ratatui::style::Color;
///
/// let style = InlineStyle::new().fg(Color::Red).bold();
/// assert_eq!(style.fg, Some(Color::Red));
/// assert!(style.bold);
/// ```
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
    /// Render text underlined (past tense — matches `ratatui::style::Modifier::UNDERLINED`).
    pub underlined: bool,
    /// Render text with strikethrough.
    ///
    /// Note: ratatui's modifier name for this is `Modifier::CROSSED_OUT`,
    /// not `STRIKETHROUGH`. The render path maps `strikethrough: true` to
    /// `add_modifier(Modifier::CROSSED_OUT)`.
    pub strikethrough: bool,
}

impl InlineStyle {
    /// Creates an empty style (no modifiers, no colors).
    ///
    /// Equivalent to [`InlineStyle::default`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::styled_text::InlineStyle;
    ///
    /// let s = InlineStyle::new();
    /// assert_eq!(s, InlineStyle::default());
    /// ```
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

    /// Builder: enable strikethrough (maps to `Modifier::CROSSED_OUT` in ratatui).
    pub const fn strikethrough(mut self) -> Self {
        self.strikethrough = true;
        self
    }
}
```

- [ ] **Step 3: Add 6 new constructor methods on `StyledInline`**

There may or may not be an existing `impl StyledInline` block in content.rs. Use `grep -n "impl StyledInline" src/component/styled_text/content.rs` to check.

If no block exists, append a new one immediately after the `impl InlineStyle` from Step 2. If one exists, append the 6 new methods to it. The methods:

```rust
impl StyledInline {
    /// Wrap text with an explicit [`InlineStyle`]. The general-purpose
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
    /// # let _ = inline;
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::styled_text::StyledInline;
    /// let inline = StyledInline::bold("emphasis");
    /// # let _ = inline;
    /// ```
    pub fn bold(text: impl Into<String>) -> Self {
        Self::styled(text, InlineStyle::new().bold())
    }

    /// Single-dimension helper: italic text.
    ///
    /// Equivalent to `StyledInline::styled(text, InlineStyle::new().italic())`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::styled_text::StyledInline;
    /// let inline = StyledInline::italic("aside");
    /// # let _ = inline;
    /// ```
    pub fn italic(text: impl Into<String>) -> Self {
        Self::styled(text, InlineStyle::new().italic())
    }

    /// Single-dimension helper: underlined text.
    ///
    /// Equivalent to `StyledInline::styled(text, InlineStyle::new().underlined())`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::styled_text::StyledInline;
    /// let inline = StyledInline::underlined("link");
    /// # let _ = inline;
    /// ```
    pub fn underlined(text: impl Into<String>) -> Self {
        Self::styled(text, InlineStyle::new().underlined())
    }

    /// Single-dimension helper: strikethrough text.
    ///
    /// Equivalent to `StyledInline::styled(text, InlineStyle::new().strikethrough())`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::styled_text::StyledInline;
    /// let inline = StyledInline::strikethrough("deleted");
    /// # let _ = inline;
    /// ```
    pub fn strikethrough(text: impl Into<String>) -> Self {
        Self::styled(text, InlineStyle::new().strikethrough())
    }

    /// Single-dimension helper: text with foreground color.
    ///
    /// "Colored" idiomatically means foreground in TUI contexts (matches
    /// `Span::styled(text, Style::default().fg(...))` ergonomics). For
    /// bg-only or fg+bg cases, use [`StyledInline::styled`] with
    /// `InlineStyle::new().bg(...)` or `.fg(...).bg(...)`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::styled_text::StyledInline;
    /// use ratatui::style::Color;
    ///
    /// let inline = StyledInline::colored("warning", Color::Yellow);
    /// # let _ = inline;
    /// ```
    pub fn colored(text: impl Into<String>, fg: Color) -> Self {
        Self::styled(text, InlineStyle::new().fg(fg))
    }
}
```

- [ ] **Step 4: Add the `Styled` arm to `render_inline_styled`**

In `src/component/styled_text/content.rs`, find the `render_inline_styled` function (around line 497). The existing 7-arm match exists; ADD an 8th arm for `Styled`. (Phase 3 will collapse the old arms; this phase keeps them.)

The current function:

```rust
fn render_inline_styled(inline: &StyledInline, base_style: Style) -> RatSpan<'static> {
    match inline {
        StyledInline::Plain(text) => RatSpan::styled(text.clone(), base_style),
        StyledInline::Bold(text) => {
            RatSpan::styled(text.clone(), base_style.add_modifier(Modifier::BOLD))
        }
        StyledInline::Italic(text) => {
            RatSpan::styled(text.clone(), base_style.add_modifier(Modifier::ITALIC))
        }
        StyledInline::Underline(text) => {
            RatSpan::styled(text.clone(), base_style.add_modifier(Modifier::UNDERLINED))
        }
        StyledInline::Strikethrough(text) => {
            RatSpan::styled(text.clone(), base_style.add_modifier(Modifier::CROSSED_OUT))
        }
        StyledInline::Colored { text, fg, bg } => {
            let mut style = base_style;
            if let Some(fg) = fg {
                style = style.fg(*fg);
            }
            if let Some(bg) = bg {
                style = style.bg(*bg);
            }
            RatSpan::styled(text.clone(), style)
        }
        StyledInline::Code(text) => {
            RatSpan::styled(text.clone(), base_style.add_modifier(Modifier::BOLD))
        }
    }
}
```

Add the `Styled` arm (insert immediately before the closing `}` of the match):

```rust
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
                // ratatui names this modifier CROSSED_OUT, not STRIKETHROUGH.
                s = s.add_modifier(Modifier::CROSSED_OUT);
            }
            RatSpan::styled(text.clone(), s)
        }
```

- [ ] **Step 5: Add `Styled` arm to `render_inline` (the theme-aware wrapper)**

`render_inline` at content.rs:475 currently has:

```rust
fn render_inline(inline: &StyledInline, theme: &Theme, base_style: Style) -> RatSpan<'static> {
    match inline {
        StyledInline::Code(text) => RatSpan::styled(
            text.clone(),
            theme.info_style().add_modifier(Modifier::BOLD),
        ),
        StyledInline::Colored { text, fg, bg } => {
            let mut style = base_style;
            if let Some(fg) = fg {
                style = style.fg(*fg);
            }
            if let Some(bg) = bg {
                style = style.bg(*bg);
            }
            RatSpan::styled(text.clone(), style)
        }
        other => render_inline_styled(other, base_style),
    }
}
```

The `other` catch-all handles all non-Code, non-Colored variants — and the new `Styled` variant routes through `other → render_inline_styled` which now has a `Styled` arm. No change needed in `render_inline`; the wrapper already delegates correctly.

- [ ] **Step 6: Add a `const` test for builder methods**

In `src/component/styled_text/content.rs`, find any existing `#[cfg(test)]` mod section (or near the bottom of the file). Add a compile-time const test verifying all 7 builder methods are usable in const context:

```rust
#[cfg(test)]
mod const_builder_test {
    use super::InlineStyle;
    use ratatui::style::Color;

    // Compile-time verification: all 7 builder methods are const fn.
    // If any method drops const, this const declaration fails to compile.
    const _STYLE: InlineStyle = InlineStyle::new()
        .fg(Color::Red)
        .bg(Color::Black)
        .bold()
        .italic()
        .underlined()
        .strikethrough();
}
```

This module compiles but doesn't add a runtime test — it's a compile-time invariant.

- [ ] **Step 7: Verify build is clean**

Run: `cargo build --all-features 2>&1 | tail -5`
Expected: clean build, no warnings, no errors. The unused `Styled` variant + `InlineStyle` struct + 6 new constructors won't warn because they're `pub` (compiler doesn't warn on unused public items).

Run: `cargo build --no-default-features 2>&1 | tail -3`
Expected: clean (styled_text gated behind `display-components`; the new types live in the same gated module).

- [ ] **Step 8: Run existing tests to verify no regressions**

Run: `cargo nextest run --all-features styled_text:: 2>&1 | tail -10`
Expected: all existing styled_text tests pass (the old leaf variants still work).

Run: `cargo test --all-features --doc styled_text 2>&1 | tail -5`
Expected: doc tests pass — both old leaf-variant doc examples AND new `InlineStyle`/`StyledInline::styled`/helper doc tests.

- [ ] **Step 9: Commit Phase 1**

```bash
git add src/component/styled_text/content.rs
git commit -S -m "Add InlineStyle struct + Styled variant + 6 constructors (G6 Phase 1)

Additive surface alongside existing leaf variants:
- #[non_exhaustive] on StyledInline enum
- New Styled { text, style: InlineStyle } variant
- New InlineStyle struct (#[non_exhaustive]) with 6 optional dimensions:
  fg, bg, bold, italic, underlined, strikethrough
- 7 const fn builder methods on InlineStyle (new, fg, bg, bold, italic,
  underlined, strikethrough) — usable in const contexts
- 6 new constructors on StyledInline: styled(t, s) general wrapper +
  5 leaf helpers (bold, italic, underlined, strikethrough, colored)
- Render arm for Styled variant: maps InlineStyle dimensions onto
  Span::styled. Bool modifiers map: bold/italic/underlined to
  Modifier::BOLD/ITALIC/UNDERLINED; strikethrough to Modifier::CROSSED_OUT
  (ratatui's name)

Old leaf variants (Bold/Italic/Underline/Strikethrough/Colored) stay
in place — Phase 2 migrates internal sites; Phase 3 deletes them.
Both APIs coexist after this commit; existing tests pass unchanged."
```

---

## Task 2: Phase 2 — migrate 102 internal references to helpers

**Files:**
- Modify: `src/component/styled_text/content.rs` (17 references — doc-test examples)
- Modify: `src/component/styled_text/tests.rs` (28 references)
- Modify: `src/component/styled_text/mod.rs` (2 doc references)
- Modify: `src/render.rs` (11 test-fixture references)
- Modify: `examples/styling_showcase.rs` (34 references)
- Modify: `examples/styled_text.rs` (10 references)

This task migrates all 102 internal envision references from leaf variants to the new constructor helpers. The old leaf variants stay defined (Phase 3 deletes them). After Phase 2: codebase compiles cleanly; no internal references to the old leaf variants remain; existing insta snapshots are byte-identical (rendering unchanged for single-dimension cases).

- [ ] **Step 1: Verify the call-site count baseline**

Run:

```bash
grep -rn "StyledInline::Bold\|StyledInline::Italic\|StyledInline::Underline\|StyledInline::Strikethrough\|StyledInline::Colored" src/ examples/
```

Expected: 102 total references. By file:

| File | Count |
|---|---|
| `examples/styling_showcase.rs` | 34 |
| `src/component/styled_text/tests.rs` | 28 |
| `src/component/styled_text/content.rs` | 17 |
| `src/render.rs` | 11 |
| `examples/styled_text.rs` | 10 |
| `src/component/styled_text/mod.rs` | 2 |

(Counts are pre-migration. `Plain` and `Code` references stay untouched, not counted here.)

- [ ] **Step 2: Apply mechanical migrations per file**

For each file, apply the mechanical replacement table:

| Pattern (regex-ish) | Replacement |
|---|---|
| `StyledInline::Bold(t)` | `StyledInline::bold(t)` |
| `StyledInline::Italic(t)` | `StyledInline::italic(t)` |
| `StyledInline::Underline(t)` | `StyledInline::underlined(t)` (past tense rename) |
| `StyledInline::Strikethrough(t)` | `StyledInline::strikethrough(t)` |
| `StyledInline::Colored { text: t, fg: Some(c), bg: None }` | `StyledInline::colored(t, c)` |
| `StyledInline::Colored { text: t, fg: Some(c), bg: Some(b) }` | `StyledInline::styled(t, InlineStyle::new().fg(c).bg(b))` |
| `StyledInline::Colored { text: t, fg: None, bg: Some(b) }` | `StyledInline::styled(t, InlineStyle::new().bg(b))` |

For the `Colored` cases, the field ordering and `Some(...)` patterns may vary across call sites. Use sequential per-site editing rather than blind sed for `Colored` — the multi-variant cases need careful inspection. The 4 boolean leaf variants (`Bold`, `Italic`, `Underline`, `Strikethrough`) are safe for sed-style replacement.

Suggested per-file approach:

1. Read the file with grep to enumerate sites: `grep -n "StyledInline::Bold\|...etc" <file>`
2. For each site, apply the appropriate replacement from the table
3. After all sites in the file are migrated, run `grep -c "StyledInline::Bold\|...etc" <file>` to verify count goes to 0
4. After all files migrated, run `cargo build --all-features 2>&1 | tail -5` to verify compilation

Note: if `Colored { text: t, fg: None, bg: None }` exists anywhere, that's semantically equivalent to `Plain(t)` — but it's also semantically equivalent to `StyledInline::styled(t, InlineStyle::default())`. Use the latter to preserve `Colored` semantics; don't collapse to `Plain` (changes the enum-variant identity which existing tests might assert on).

If any test asserts on enum variant identity (e.g., `matches!(x, StyledInline::Bold(_))`), the assertion must update to match the new shape: `matches!(x, StyledInline::Styled { style: InlineStyle { bold: true, .. }, .. })`. Watch for these in `tests.rs` during migration.

- [ ] **Step 3: Add the `Styled` arm where pattern-matching is exhaustive**

Some test code in `src/component/styled_text/tests.rs` may exhaustively match `StyledInline`. With `#[non_exhaustive]` on the enum (added in Phase 1), exhaustive matches outside the crate need a `_` arm. Inside the crate, the compiler still requires exhaustive matching.

After Phase 1 added the `Styled` variant, any internal `match styled_inline { ... }` block must include a `Styled { .. }` arm OR a `_` catch-all.

Run: `cargo build --all-features 2>&1 | tail -20`
Expected: if any internal match is non-exhaustive, the compiler reports it. Add a `_` catch-all arm to each such match (since these are pre-Phase-3 transitional and Phase 3 deletes the old arms).

- [ ] **Step 4: Verify per-file migration counts**

For each file, confirm the leaf-variant references are gone:

```bash
grep -c "StyledInline::Bold\|StyledInline::Italic\|StyledInline::Underline\|StyledInline::Strikethrough\|StyledInline::Colored" src/component/styled_text/content.rs
grep -c "StyledInline::Bold\|StyledInline::Italic\|StyledInline::Underline\|StyledInline::Strikethrough\|StyledInline::Colored" src/component/styled_text/tests.rs
grep -c "StyledInline::Bold\|StyledInline::Italic\|StyledInline::Underline\|StyledInline::Strikethrough\|StyledInline::Colored" src/component/styled_text/mod.rs
grep -c "StyledInline::Bold\|StyledInline::Italic\|StyledInline::Underline\|StyledInline::Strikethrough\|StyledInline::Colored" src/render.rs
grep -c "StyledInline::Bold\|StyledInline::Italic\|StyledInline::Underline\|StyledInline::Strikethrough\|StyledInline::Colored" examples/styling_showcase.rs
grep -c "StyledInline::Bold\|StyledInline::Italic\|StyledInline::Underline\|StyledInline::Strikethrough\|StyledInline::Colored" examples/styled_text.rs
```

Each should return `0`. The crate-wide check:

```bash
grep -rn "StyledInline::Bold\|StyledInline::Italic\|StyledInline::Underline\|StyledInline::Strikethrough\|StyledInline::Colored" src/ examples/
```

Expected: no output (zero matches).

- [ ] **Step 5: Verify build + tests pass + snapshots byte-identical**

Run:

```bash
cargo build --all-features 2>&1 | tail -5
cargo build --no-default-features 2>&1 | tail -3
cargo nextest run --all-features 2>&1 | tail -5
cargo test --all-features --doc 2>&1 | tail -3
```

Expected: all green.

**CRITICAL: Verify insta snapshots are byte-identical post-migration.** Run:

```bash
git status src/component/styled_text/snapshots/ src/snapshots/ 2>&1 | tail -10
```

Expected: clean (no `.snap.new` files; no modified `.snap` files). If any snapshot has drifted, the migration changed rendering — investigate before committing. The migration should preserve byte-identical output for all single-dimension cases because:

- `StyledInline::Bold(t)` → `RatSpan::styled(t, base.add_modifier(BOLD))`
- `StyledInline::bold(t)` = `StyledInline::Styled { text: t, style: InlineStyle::new().bold() }` → `RatSpan::styled(t, base.add_modifier(BOLD))`

Same output, different construction path.

- [ ] **Step 6: Commit Phase 2**

```bash
git add src/ examples/
git commit -S -m "Migrate 102 internal sites to StyledInline helpers (G6 Phase 2)

Mechanical migration of all internal envision references from leaf
variants to the new helper constructors:

- examples/styling_showcase.rs: 34 references
- src/component/styled_text/tests.rs: 28 references
- src/component/styled_text/content.rs: 17 doc-example references
- src/render.rs: 11 test-fixture references
- examples/styled_text.rs: 10 references
- src/component/styled_text/mod.rs: 2 doc-example references

Replacements:
- StyledInline::Bold(t) -> StyledInline::bold(t)
- StyledInline::Italic(t) -> StyledInline::italic(t)
- StyledInline::Underline(t) -> StyledInline::underlined(t) (past tense)
- StyledInline::Strikethrough(t) -> StyledInline::strikethrough(t)
- StyledInline::Colored { text, fg: Some(c), bg: None } -> StyledInline::colored(t, c)
- Multi-field Colored cases -> StyledInline::styled(t, InlineStyle::new()...)

Old leaf variants still defined (Phase 3 deletes them). Insta snapshots
byte-identical pre/post — migration preserves rendering for all
single-dimension cases. No behavior change."
```

---

## Task 3: Phase 3 — delete 5 leaf variants + simplify render path

**Files:**
- Modify: `src/component/styled_text/content.rs` (delete 5 variants + their render arms; render path collapses 7-arm match → 3-arm)

After Phase 2, no internal references to the leaf variants remain. This task deletes them at the type level. Compiler enforces completeness: if any reference was missed in Phase 2, this task's `cargo build` fails.

- [ ] **Step 1: Delete the 5 leaf variants from the enum**

In `src/component/styled_text/content.rs`, find the `StyledInline` enum (post-Phase-1 it has 8 variants: Plain, Bold, Italic, Underline, Strikethrough, Colored, Code, Styled). Delete the 5 leaf variants. The post-Phase-3 enum becomes:

```rust
/// An inline styling element within a paragraph or list item.
///
/// `#[non_exhaustive]` so envision can add inline variants later without
/// breaking downstream `match` arms in consumer crates.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum StyledInline {
    /// Plain unstyled text.
    Plain(String),
    /// Inline code (renders with theme-coupled styling — bold info color).
    Code(String),
    /// Styled run combining color, modifiers, and optional background.
    ///
    /// The composable form. Use [`StyledInline::styled`] or one of the
    /// leaf-helper constructors (`bold`, `italic`, `underlined`,
    /// `strikethrough`, `colored`) to construct.
    Styled {
        /// The text content.
        text: String,
        /// Style dimensions applied on top of the surrounding base style.
        style: InlineStyle,
    },
}
```

3 variants total.

- [ ] **Step 2: Delete the 5 obsolete render arms from `render_inline_styled`**

In `src/component/styled_text/content.rs`, find `fn render_inline_styled` (the 8-arm match after Phase 1). Delete the 5 arms for the deleted variants (Bold, Italic, Underline, Strikethrough, Colored). Post-Phase-3 the function becomes:

```rust
fn render_inline_styled(inline: &StyledInline, base_style: Style) -> RatSpan<'static> {
    match inline {
        StyledInline::Plain(text) => RatSpan::styled(text.clone(), base_style),
        StyledInline::Code(text) => {
            // Code keeps theme-coupled bold + info color in render_inline;
            // here in render_inline_styled (theme-less path), apply bold only.
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
                // ratatui names this modifier CROSSED_OUT, not STRIKETHROUGH.
                s = s.add_modifier(Modifier::CROSSED_OUT);
            }
            RatSpan::styled(text.clone(), s)
        }
    }
}
```

3 arms total.

- [ ] **Step 3: Update `render_inline` (the theme-aware wrapper)**

The current `render_inline` at content.rs:475 has a `Colored { text, fg, bg }` arm that handles theme-aware coloring. After deleting `Colored`, the theme-aware path collapses:

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

Only `Code` keeps a theme-coupled fast path; `Plain` and `Styled` fall through to `render_inline_styled` (which now handles both via the slimmer 3-arm match).

- [ ] **Step 4: Verify build catches any missed migration**

Run: `cargo build --all-features 2>&1 | tail -20`

Expected: clean build. If any reference to a deleted variant was missed in Phase 2, the compiler reports it as an "unresolved variant" error here. Fix any such site by applying the appropriate replacement from Phase 2's table, then re-run.

Run: `cargo build --no-default-features 2>&1 | tail -3`
Expected: clean.

- [ ] **Step 5: Run full test suite + verify snapshots still byte-identical**

```bash
cargo nextest run --all-features 2>&1 | tail -5
cargo test --all-features --doc 2>&1 | tail -3
git status src/component/styled_text/snapshots/ src/snapshots/ 2>&1 | tail -10
```

Expected: all tests pass. Insta snapshots remain unchanged (the variant deletion is purely at the type level; render output is byte-identical because Phase 2 already routed every previously-leaf-variant render through the new `Styled` arm with equivalent semantics).

- [ ] **Step 6: Verify file-size constraint**

Run: `wc -l src/component/styled_text/*.rs`
Expected: `content.rs` ~600 lines (was 526; net +~74 from struct + builder + helpers minus deleted variants and arms). All under 1000-line cap.

- [ ] **Step 7: Commit Phase 3**

```bash
git add src/component/styled_text/content.rs
git commit -S -m "Delete 5 leaf StyledInline variants + simplify render path (G6 Phase 3)

Mechanical at the type level — completes the G6 breaking change:
- Delete StyledInline::Bold(String) variant
- Delete StyledInline::Italic(String) variant
- Delete StyledInline::Underline(String) variant
- Delete StyledInline::Strikethrough(String) variant
- Delete StyledInline::Colored { text, fg, bg } variant

Enum collapses from 7 → 3 variants (Plain, Code, Styled).

render_inline_styled collapses from 7-arm match → 3-arm match.
render_inline (theme-aware wrapper) collapses from 3-arm match →
2-arm match (Code fast path + delegated other).

Compiler enforces completeness: any leftover internal reference to a
deleted variant would fail to compile here. Phase 2's migration was
verified by grep + this Phase's clean build.

Insta snapshots byte-identical pre/post; rendering is unchanged for
all single-dimension cases (Phase 2 routed them through Styled with
equivalent semantics)."
```

---

## Task 4: Add 8 new tests for the composable surface

**Files:**
- Modify: `src/component/styled_text/tests.rs`

This task adds 8 named tests pinning the new API surface. 4 are unit tests (builder behavior, round-trip semantics, helper equivalence); 4 are render-path tests including the load-bearing bold+colored snapshot.

- [ ] **Step 1: Append the 4 unit tests to `tests.rs`**

Add these 4 tests in a logical location (e.g., after existing builder tests or at the end of the file):

```rust
#[test]
fn test_inline_style_new_is_default() {
    use crate::component::styled_text::InlineStyle;

    // Pins the contract that ::new() and ::default() produce equivalent
    // empty styles. Consumer code can use either interchangeably.
    assert_eq!(InlineStyle::new(), InlineStyle::default());
}

#[test]
fn test_inline_style_builder_chain() {
    use crate::component::styled_text::InlineStyle;
    use ratatui::style::Color;

    // Each builder method sets exactly the field it names; other fields
    // stay at their default. Pin via field-level assertions.
    let style = InlineStyle::new().bold().fg(Color::Red).underlined();

    assert!(style.bold);
    assert_eq!(style.fg, Some(Color::Red));
    assert!(style.underlined);

    // Untouched fields remain default.
    assert!(!style.italic);
    assert!(!style.strikethrough);
    assert_eq!(style.bg, None);
}

#[test]
fn test_styled_inline_styled_pairs_text_and_style() {
    use crate::component::styled_text::{InlineStyle, StyledInline};
    use ratatui::style::Color;

    let style = InlineStyle::new().fg(Color::Magenta).bold();
    let inline = StyledInline::styled("hello", style);

    // The general-purpose constructor pairs text with style verbatim.
    match inline {
        StyledInline::Styled { text, style: s } => {
            assert_eq!(text, "hello");
            assert_eq!(s, style);
        }
        _ => panic!("expected Styled variant, got: {inline:?}"),
    }
}

#[test]
fn test_styled_inline_leaf_helpers_match_builder() {
    use crate::component::styled_text::{InlineStyle, StyledInline};
    use ratatui::style::Color;

    // Each leaf helper must produce the same StyledInline value as
    // StyledInline::styled(t, InlineStyle::new().<dim>()). Pins the
    // helper-vs-builder contract so refactors don't drift.
    assert_eq!(
        StyledInline::bold("t"),
        StyledInline::styled("t", InlineStyle::new().bold()),
    );
    assert_eq!(
        StyledInline::italic("t"),
        StyledInline::styled("t", InlineStyle::new().italic()),
    );
    assert_eq!(
        StyledInline::underlined("t"),
        StyledInline::styled("t", InlineStyle::new().underlined()),
    );
    assert_eq!(
        StyledInline::strikethrough("t"),
        StyledInline::styled("t", InlineStyle::new().strikethrough()),
    );
    assert_eq!(
        StyledInline::colored("t", Color::Red),
        StyledInline::styled("t", InlineStyle::new().fg(Color::Red)),
    );
}
```

- [ ] **Step 2: Add the 4 render-path snapshot/ANSI tests**

The exact render-path setup depends on existing test patterns in this file. The standard envision pattern (from prior PRs like #482, #487) is `crate::component::test_utils::setup_render(width, height)` returning `(terminal, theme)`, then `envision::render::styled_line(frame, area, &[StyledInline], theme)` to render a single line.

Append the 4 render-path tests:

```rust
#[test]
fn snapshot_styled_inline_bold_and_colored_combined() {
    use crate::component::styled_text::{InlineStyle, StyledInline};
    use crate::component::test_utils::setup_render;
    use crate::render::styled_line;
    use ratatui::style::Color;

    // THE LOAD-BEARING G6 PAYOFF PIN.
    //
    // The bold+colored combo specifically because it's the user-visible
    // payoff for G6: leadline's build_summary_inlines (app.rs:412-455)
    // emits 5 value segments (iconnx/ort/ratio/delta/iters) that need
    // bold + severity-color in a SINGLE inline run. Pre-G6, Bold(t)
    // had no color field and Colored {..} had no bold field, so the
    // bold half was dropped. Post-G6, this test asserts the combo
    // lands — both \x1b[31m (red) AND \x1b[1m (BOLD) appear on the
    // same Span in the rendered ANSI output.
    //
    // If either escape goes missing, build_summary_inlines reads flat
    // again — the user loses the magnitude-jump weight contrast.
    let inlines = vec![StyledInline::styled(
        "840.16 ms",
        InlineStyle::new().fg(Color::Red).bold(),
    )];

    let (mut terminal, theme) = setup_render(20, 1);
    terminal
        .draw(|frame| {
            styled_line(frame, frame.area(), &inlines, &theme);
        })
        .unwrap();

    let plain = terminal.backend().to_string();
    let ansi = terminal.backend().to_ansi();

    assert!(
        ansi.contains("\x1b[31m"),
        "expected red (31m) ANSI fg for fg(Red), got:\n{ansi}",
    );
    assert!(
        ansi.contains("\x1b[1m"),
        "expected BOLD (1m) ANSI modifier for bold(), got:\n{ansi}",
    );

    insta::assert_snapshot!(plain);
}

#[test]
fn snapshot_styled_inline_full_dimension_combo() {
    use crate::component::styled_text::{InlineStyle, StyledInline};
    use crate::component::test_utils::setup_render;
    use crate::render::styled_line;
    use ratatui::style::Color;

    // Render every dimension at once. Pins the full composability
    // surface: 6 dimensions in a single inline (bold + italic +
    // underlined + strikethrough + fg + bg).
    let inlines = vec![StyledInline::styled(
        "ALL",
        InlineStyle::new()
            .bold()
            .italic()
            .underlined()
            .strikethrough()
            .fg(Color::Red)
            .bg(Color::Black),
    )];

    let (mut terminal, theme) = setup_render(20, 1);
    terminal
        .draw(|frame| {
            styled_line(frame, frame.area(), &inlines, &theme);
        })
        .unwrap();

    let ansi = terminal.backend().to_ansi();

    // All 6 SGR codes appear:
    // - \x1b[1m  bold
    // - \x1b[3m  italic
    // - \x1b[4m  underlined
    // - \x1b[9m  strikethrough (ratatui Modifier::CROSSED_OUT)
    // - \x1b[31m red foreground
    // - \x1b[40m black background
    assert!(ansi.contains("\x1b[1m"), "expected bold (1m), got:\n{ansi}");
    assert!(ansi.contains("\x1b[3m"), "expected italic (3m), got:\n{ansi}");
    assert!(ansi.contains("\x1b[4m"), "expected underlined (4m), got:\n{ansi}");
    assert!(ansi.contains("\x1b[9m"), "expected strikethrough/crossed_out (9m), got:\n{ansi}");
    assert!(ansi.contains("\x1b[31m"), "expected red fg (31m), got:\n{ansi}");
    assert!(ansi.contains("\x1b[40m"), "expected black bg (40m), got:\n{ansi}");

    let plain = terminal.backend().to_string();
    insta::assert_snapshot!(plain);
}

#[test]
fn test_inline_style_default_no_modifiers_applied() {
    use crate::component::styled_text::{InlineStyle, StyledInline};
    use crate::component::test_utils::setup_render;
    use crate::render::styled_line;

    // Rendering StyledInline::styled(t, InlineStyle::default()) should
    // produce no ANSI modifiers — equivalent to Plain(t) at the
    // rendering level.
    let styled_default = vec![StyledInline::styled("text", InlineStyle::default())];
    let plain_variant = vec![StyledInline::Plain("text".into())];

    let (mut term_styled, theme) = setup_render(20, 1);
    term_styled
        .draw(|frame| {
            styled_line(frame, frame.area(), &styled_default, &theme);
        })
        .unwrap();
    let styled_ansi = term_styled.backend().to_ansi();

    let (mut term_plain, _) = setup_render(20, 1);
    term_plain
        .draw(|frame| {
            styled_line(frame, frame.area(), &plain_variant, &theme);
        })
        .unwrap();
    let plain_ansi = term_plain.backend().to_ansi();

    // No bold/italic/underlined/strikethrough/fg/bg escapes for either form.
    for escape in ["\x1b[1m", "\x1b[3m", "\x1b[4m", "\x1b[9m"] {
        assert!(
            !styled_ansi.contains(escape),
            "Styled(default) should not emit modifier {escape}, got:\n{styled_ansi}",
        );
    }
}

#[test]
fn snapshot_styled_inline_plain_and_code_unchanged_postmigration() {
    use crate::component::styled_text::StyledInline;
    use crate::component::test_utils::setup_render;
    use crate::render::styled_line;

    // Plain and Code are the two variants that survive G6 unchanged.
    // Their rendering must be byte-identical post-migration — pin via
    // snapshot. If either snapshot drifts, the G6 enum reshape
    // inadvertently altered the surviving variants.
    let inlines = vec![
        StyledInline::Plain("plain text".into()),
        StyledInline::Code("code text".into()),
    ];

    let (mut terminal, theme) = setup_render(40, 1);
    terminal
        .draw(|frame| {
            styled_line(frame, frame.area(), &inlines, &theme);
        })
        .unwrap();

    let plain = terminal.backend().to_string();
    insta::assert_snapshot!(plain);
}
```

- [ ] **Step 3: Run the new tests**

Run: `cargo nextest run --all-features styled_text::tests:: 2>&1 | tail -15`
Expected: 8 new tests PASS (4 unit + 4 render-path).

For the 3 snapshot tests, insta will create new `.snap` files on first run. Accept via `cargo insta accept` if prompted.

Run: `cargo test --all-features --doc styled_text 2>&1 | tail -5`
Expected: doc tests pass (new InlineStyle + StyledInline::styled + helper doc tests).

Run: `cargo nextest run --all-features 2>&1 | tail -3`
Expected: full suite passes (existing + 8 new).

- [ ] **Step 4: Commit Task 4**

```bash
git add src/component/styled_text/tests.rs src/component/styled_text/snapshots/
git commit -S -m "Add 8 tests for composable StyledInline (G6)

Pins the new surface contract from multiple angles:

Unit tests (4):
- test_inline_style_new_is_default: ::new() == ::default()
- test_inline_style_builder_chain: each builder method sets only its field
- test_styled_inline_styled_pairs_text_and_style: ::styled() round-trip
- test_styled_inline_leaf_helpers_match_builder: helpers equivalent to
  styled() + InlineStyle::new().<dim>()

Render-path tests (4):
- snapshot_styled_inline_bold_and_colored_combined: THE LOAD-BEARING
  G6 PAYOFF — ANSI-asserts both \\x1b[31m AND \\x1b[1m appear on same
  span. Body comment links to leadline's build_summary_inlines as the
  consumer use case (bold + severity-color value segments).
- snapshot_styled_inline_full_dimension_combo: all 6 SGR codes appear
  (bold, italic, underlined, strikethrough/CROSSED_OUT, fg, bg)
- test_inline_style_default_no_modifiers_applied: empty InlineStyle
  produces no SGR modifiers (equivalent to Plain rendering)
- snapshot_styled_inline_plain_and_code_unchanged_postmigration:
  Plain + Code render byte-identical post-migration (sanity for the
  two variants that survived G6)"
```

---

## Task 5: Verify clippy + fmt + doc + audit + no-default-features

**Files:** (none — verification only)

- [ ] **Step 1: Clippy**

Run: `cargo clippy --all-features --all-targets -- -D warnings 2>&1 | tail -10`
Expected: clean.

If warnings appear, fix in-place. Common ones to watch for:
- Unused imports in tests after the migration (`use super::*;` may pull in newly-unused items)
- `match` arms that could be simplified after the render-path collapse

- [ ] **Step 2: Rustdoc**

Run: `RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps 2>&1 | tail -10`
Expected: clean. New `InlineStyle` + helper doc tests compile.

- [ ] **Step 3: cargo fmt**

Run: `cargo fmt --all -- --check 2>&1 | wc -l`
Expected: `0`.

If drift, apply: `cargo fmt --all` and commit:

```bash
git add -A
git commit -S -m "Apply cargo fmt"
```

- [ ] **Step 4: Full test suite + doc tests**

```bash
cargo nextest run --all-features 2>&1 | tail -3
cargo test --all-features --doc 2>&1 | tail -3
```

Expected: all pass.

- [ ] **Step 5: No-default-features build**

Run: `cargo build --no-default-features 2>&1 | tail -3`
Expected: clean. Preventive check — `styled_text` is gated behind `display-components`; the new types live in the same gated module so this should pass naturally.

- [ ] **Step 6: Audit scorecard**

Run: `./tools/audit/target/release/envision-audit scorecard 2>&1 | grep "Result:"`
Expected: `Result: 8/9 checks passing` — same baseline as main. No regression.

If audit drops to 7/9, check the doc-test coverage category: the 6 new constructor methods on `StyledInline` + 7 builder methods on `InlineStyle` all need `# Example` doc tests. Plan provides these in Task 1 Steps 2-3; verify none were dropped.

- [ ] **Step 7: Commit if any fixes were needed**

If any step required a fix, commit it. Otherwise no commit — Task 5 is verification-only.

---

## Task 6: CHANGELOG entry

**Files:** Modify `CHANGELOG.md`

- [ ] **Step 1: Add the entry under `## [Unreleased]`**

Below the most recent sub-section (likely "Per-component style overrides (G4 + G5)" from PR #487), add:

```markdown
### `StyledInline` composable styles (G6)

Replaces the 7-variant `StyledInline` enum (`Plain | Bold | Italic | Underline |
Strikethrough | Colored | Code`) with a 3-variant composable shape. The leaf
variants forced single-dimension styling — `Bold + Colored` required two adjacent
inlines because each leaf captured one dimension. Combinatorial explosion (2^6 =
64 variants for full dimension coverage) was the wrong shape; composable struct
is right.

**New 3-variant enum:**

- `StyledInline::Plain(String)` — unchanged
- `StyledInline::Code(String)` — unchanged (theme-coupled fast path)
- `StyledInline::Styled { text, style: InlineStyle }` — new composable variant
- Enum gains `#[non_exhaustive]` for future variant additions

**New `InlineStyle` struct:**

- 6 optional dimensions: `fg`, `bg`, `bold`, `italic`, `underlined`, `strikethrough`
- 7 `const fn` builder methods (`new`, `fg`, `bg`, `bold`, `italic`, `underlined`,
  `strikethrough`) — usable in `const` contexts (module-level static styles)
- `#[non_exhaustive]` — forces builder use over struct literal; future modifier
  additions land additively
- Note: `strikethrough: bool` maps to `ratatui::style::Modifier::CROSSED_OUT`
  (ratatui's name for this modifier)

**Six new constructors on `StyledInline`:**

- `StyledInline::styled(text, style)` — general-purpose pair-with-style wrapper
- `StyledInline::bold(text)`, `italic(text)`, `underlined(text)`,
  `strikethrough(text)`, `colored(text, fg)` — leaf helpers for single-dimension
  cases (~80% of styled-inline usage)

**Top-line payoff — bold-on-banner-values:**

leadline's per-op summary banner at `app.rs:412-455` (`build_summary_inlines`)
renders 5 value segments (iconnx/ort/ratio/delta/iters) that need bold +
severity-color in a single inline run. Pre-G6, the bold half was dropped
because `Bold(t)` had no color field and `Colored {..}` had no bold field.
Post-G6, `StyledInline::styled(value, InlineStyle::new().fg(value_color).bold())`
lands the combo. The summary banner reads with weight contrast on value
segments — magnitude of slowdown "jumps" at the user via bold weight in
addition to severity color.

**Migration:**

- `StyledInline::Bold(t)` → `StyledInline::bold(t)`
- `StyledInline::Italic(t)` → `StyledInline::italic(t)`
- `StyledInline::Underline(t)` → `StyledInline::underlined(t)` (past-tense rename)
- `StyledInline::Strikethrough(t)` → `StyledInline::strikethrough(t)`
- `StyledInline::Colored { text, fg: Some(c), bg: None }` → `StyledInline::colored(t, c)`
- Multi-field `Colored` cases → `StyledInline::styled(t, InlineStyle::new().fg(c).bg(b))`

102 internal envision references migrated mechanically across 6 files. Old
leaf variants deleted outright (pre-1.0 ruthlessness; same pattern as D14
`paragraph→line`, D5+D14 `StyledTextState` collapse, G5 `severity_status_style`
deletion).

**Breaking changes:**

- 5 enum variants deleted (`Bold`, `Italic`, `Underline`, `Strikethrough`,
  `Colored`). External code pattern-matching `StyledInline` exhaustively must
  rewrite for the new 3-variant shape (with `_` arm for `#[non_exhaustive]`).
- `StyledInline::Underline` renamed to the helper `StyledInline::underlined`
  (past-tense, matches ratatui's `Modifier::UNDERLINED` and the boolean field
  naming convention).
- `InlineStyle` struct uses `#[non_exhaustive]` — external code cannot
  construct via struct literal; must use the `InlineStyle::new()...` builder.
```

- [ ] **Step 2: Commit**

```bash
git add CHANGELOG.md
git commit -S -m "CHANGELOG: StyledInline composable styles (G6)

Document the 7→3 variant collapse, new InlineStyle struct + 7 const
fn builder methods, 6 new constructors on StyledInline, the
strikethrough → Modifier::CROSSED_OUT mapping note, and the
bold-on-banner-values payoff for leadline's per-op summary. Flag the
breaking changes (5 deleted variants; Underline → underlined rename;
#[non_exhaustive] blocks struct literal on InlineStyle)."
```

---

## Task 7: Final verification + push + open PR

**Files:** (none — verification + git only)

- [ ] **Step 1: Verify all commits are signed**

Run: `git log --show-signature main..HEAD 2>&1 | grep -c 'Good signature'`
Expected: count matches the commits added on this branch (5 or 6 depending on whether Task 5 needed a fix commit).

If any commit is unsigned, **stop** and ask the user how to handle it — never bypass.

- [ ] **Step 2: Full gauntlet**

```bash
cargo build --all-features 2>&1 | tail -1
cargo build --no-default-features 2>&1 | tail -1
cargo clippy --all-features --all-targets -- -D warnings 2>&1 | tail -1
cargo fmt --all -- --check 2>&1 | wc -l
cargo nextest run --all-features 2>&1 | tail -3
cargo test --all-features --doc 2>&1 | tail -2
RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps 2>&1 | tail -1
./tools/audit/target/release/envision-audit scorecard 2>&1 | grep "Result:"
```

Expected: every command succeeds. Audit shows `Result: 8/9 checks passing` (same baseline as main).

- [ ] **Step 3: Verify final stale-reference check**

```bash
grep -rn "StyledInline::Bold\|StyledInline::Italic\|StyledInline::Underline\|StyledInline::Strikethrough\|StyledInline::Colored" src/ examples/
```

Expected: zero matches. (`StyledInline::Plain` and `StyledInline::Code` references are fine — they're the variants that survive.)

- [ ] **Step 4: Push the branch**

Run: `git push -u origin styled-inline-compose-impl`

(Branch name is `styled-inline-compose-impl` — the implementation branch, set by the controller before plan execution begins. The current plan branch is `styled-inline-compose-plan`.)

Expected: pushes cleanly.

- [ ] **Step 5: Open the implementation PR**

Run:

```bash
gh pr create --title "StyledInline composable styles (G6)" --body "$(cat <<'EOF'
## Summary

Implementation of leadline gap **G6** — \`StyledInline\` 7-variant leaf-enum collapses to a 3-variant composable shape, restoring bold-on-banner-values in leadline's per-op summary banner.

Spec: PR #489 (\`docs/superpowers/specs/2026-05-20-styled-inline-compose-design.md\`)
Plan: PR β (\`docs/superpowers/plans/2026-05-20-styled-inline-compose.md\`)

## What changed

**3-phase additive-first migration** for clean bisect granularity:

- **Phase 1 (Task 1):** Add new surface — \`InlineStyle\` struct + \`Styled\` variant + 6 constructors + 7 const fn builder methods. Old leaf variants stay; both APIs coexist.
- **Phase 2 (Task 2):** Migrate all 102 internal references across 6 files from leaf variants to new helpers. Old variants still defined; insta snapshots byte-identical pre/post.
- **Phase 3 (Task 3):** Delete 5 leaf variants + simplify render path. Compiler enforces migration completeness — any missed reference fails to compile here.

**New API:**

- 3-variant enum: \`Plain | Code | Styled { text, style: InlineStyle }\` (with \`#[non_exhaustive]\`)
- \`InlineStyle\` struct with 6 dimensions (fg, bg, bold, italic, underlined, strikethrough) + 7 \`const fn\` builder methods
- 6 new constructors on \`StyledInline\` (\`styled\` + 5 leaf helpers)

**Top-line payoff:** Snapshot test ANSI-asserts both \`\\x1b[31m\` (red) AND \`\\x1b[1m\` (BOLD) appear on the same span — vindication of the bold + severity-color combo for leadline's banner-value segments.

**Breaking changes:** 5 enum variants deleted; \`Underline\` → \`underlined\` past-tense rename; \`#[non_exhaustive]\` on \`InlineStyle\` blocks struct literal.

## Stats

- 4-6 signed commits (1 per phase + tests + CHANGELOG + optional fmt cleanup)
- 8 new tests (4 unit + 4 render-path)
- 102-site mechanical migration across 6 files
- File-size delta: \`content.rs\` 526 → ~600; \`tests.rs\` 713 → ~860. All under 1000-line cap.

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
- [ ] leadline migrates \`build_summary_inlines\` value segments to \`StyledInline::styled(value, InlineStyle::new().fg(value_color).bold())\` for the bold-on-banner-values payoff

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
| Enum reshape (7 → 3 variants) | Tasks 1 (add Styled) + 3 (delete 5 leaves) |
| `#[non_exhaustive]` on enum | Task 1 Step 1 |
| `InlineStyle` struct + `#[non_exhaustive]` + 7 const fn builder methods | Task 1 Step 2 |
| 6 new constructors on `StyledInline` | Task 1 Step 3 |
| Render path simplification (7-arm → 3-arm) | Tasks 1 (add Styled arm) + 3 (delete 5 old arms) |
| `Code` theme-coupled fast path preserved | Task 1 Step 5 (verified `render_inline` wrapper unchanged at Code) + Task 3 Step 3 (preserved on collapse) |
| 102-site mechanical migration | Task 2 |
| Underline → underlined past-tense rename | Task 2 (migration table) + helper at Task 1 Step 3 |
| `strikethrough: bool` → `Modifier::CROSSED_OUT` mapping + comment (leadline soft note #1) | Task 1 Step 2 (field docstring) + Step 4 (render-arm comment) + Task 3 Step 2 (preserved on collapse) |
| 8 named tests | Task 4 (4 unit + 4 render-path) |
| Test #5 leadline use case link (leadline soft note #2) | Task 4 Step 2 (body comment explicitly references `build_summary_inlines`) |
| CHANGELOG entry | Task 6 |
| Insta snapshot stability for single-dimension cases | Task 2 Step 5 (verification gate) |
| Compile-time `const fn` verification | Task 1 Step 6 |

All spec requirements have a corresponding task. Both leadline soft notes baked in.

### 2. Placeholder scan

No "TBD", "TODO", "implement later". Every step has either complete code, exact commands, or explicit verification criteria. The Task 2 migration step uses a structured per-file approach rather than a placeholder — each replacement pattern is explicit in the table.

### 3. Type consistency

- `InlineStyle` field names match across struct def + builder methods + render arm + tests: `fg`, `bg`, `bold`, `italic`, `underlined`, `strikethrough`. ✅
- `StyledInline::Styled { text, style }` field names consistent across enum def + constructors + render arm + tests. ✅
- Helper method names match leaf-helper convention: `bold`, `italic`, `underlined` (past tense), `strikethrough`, `colored`. ✅
- Render arms use ratatui modifier constants consistently: `Modifier::BOLD`, `Modifier::ITALIC`, `Modifier::UNDERLINED`, `Modifier::CROSSED_OUT`. ✅
- ANSI escape codes consistent: `\x1b[1m` bold, `\x1b[3m` italic, `\x1b[4m` underlined, `\x1b[9m` strikethrough, `\x1b[31m` red fg, `\x1b[40m` black bg. ✅

---

## Plan complete

The plan covers 7 tasks producing approximately 4-6 signed commits (Tasks 1, 2, 3, 4, 6 produce commits; Task 5 verification-only; Task 7 push + PR; optional fmt commit if Task 5 surfaces drift). Estimated implementation time: 3-4 hours of focused work (Task 2's 102-site migration is the longest single task).

After plan PR β merges, controller creates `styled-inline-compose-impl` branch from main and dispatches subagent-driven execution. After implementation PR γ merges, controller opens tracking-doc PR δ marking G6 ✅ resolved in `docs/customer-feedback/2026-05-01-leadline-gaps.md`.
