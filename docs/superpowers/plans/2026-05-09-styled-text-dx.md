# StyledText DX (D5 + D14) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `envision::render::styled_line(frame, area, &[StyledInline], theme)` free-function primitive and rename the misleading `StyledContent::paragraph(...)` / `StyledBlock::Paragraph(...)` to `line` / `Line`. Frees the `paragraph` name for future block-level semantics and gives consumers a one-call line-rendering path.

**Architecture:** Three coupled changes in a single PR: (1) rename `StyledBlock::Paragraph` enum variant + `StyledContent::paragraph()` builder method + private `render_paragraph` helper to `Line` / `line` / `render_line` (atomic — match exhaustiveness requires both sides), (2) new top-level `src/render.rs` module hosting `styled_line` primitive that internally builds a one-block `StyledContent` and renders via the existing `render_lines` path (no new rendering logic), (3) mechanical `.paragraph(` → `.line(` migration across 10 example call sites. Rendered output (insta snapshots) is byte-identical pre/post rename — the rename is name-only.

**Tech Stack:** Rust 1.85+, ratatui 0.29 (`Frame`, `Rect`, `Paragraph` widget, `Line`, `Text`), envision `styled_text` module (`StyledContent`, `StyledBlock`, `StyledInline`), `Theme`.

**Spec:** `docs/superpowers/specs/2026-05-09-styled-text-dx-design.md` (PR #480, merged)

---

## Pre-execution gotchas

- **Signed commits required.** Project rule. If `git commit -S` fails, ask the user; never bypass with `--no-gpg-sign`.
- **Rename + primitive ordering: rename first, then primitive.** Rename is purely mechanical (no behavior change). Primitive uses the renamed `StyledContent::line(...)` internally — cleaner to have the API stable before adding callers. Task 1 lands the rename atomically; Tasks 2-3 finish the migration; Tasks 4-5 add the primitive.
- **Rename atomicity.** `StyledBlock::Paragraph` + `StyledContent::paragraph()` + internal `render_paragraph` + the `StyledBlock::Paragraph =>` match arm must all rename in the same commit. Splitting them leaves either a non-existent variant (compile error) or a non-exhaustive match (compile error). Task 1 bundles all four into one commit.
- **Match exhaustiveness.** `match block { StyledBlock::Heading {..} => ..., StyledBlock::Paragraph(..) => ..., ... }` at `content.rs:390-454` is the render path. After rename: `StyledBlock::Line(..) =>`. There's no `#[non_exhaustive]` on `StyledBlock` today; adding it is out of scope for this PR (would be a separate decision matching D6/D9/D15 precedent).
- **Insta snapshot stability.** Rendered output is byte-identical pre/post rename. Verify by running `cargo nextest run --all-features styled_text::` after Task 1 — snapshots should NOT require `cargo insta accept`. If they do, investigate before committing; the variant rename should not change rendered text.
- **Test function rename.** `tests.rs:47` defines `fn test_content_paragraph_with_inlines()`. Rename to `fn test_content_line_with_inlines()` for naming coherence (the test exercises `.line(...)` after rename).
- **Search-and-replace patterns.** Use precise patterns to avoid false positives:
  - `\.paragraph(` for method calls (the trailing paren disambiguates from `paragraph` as a word in doc comments)
  - `StyledBlock::Paragraph` for variant references
  - `render_paragraph` for the private helper
  - **Do NOT** blanket-rename `paragraph` as a word — it appears in docstrings and example text like `.text("Introduction paragraph.")` where it's just English prose. Those stay.
- **`ratatui::widgets::Paragraph` is unrelated.** `src/component/styled_text/mod.rs:40,498,502` imports and uses `ratatui::widgets::Paragraph` — that's the upstream widget type, not our `StyledBlock::Paragraph` variant. Leave it alone.
- **Verified call-site counts (re-grep'd 2026-05-09):**
  - `.paragraph(` method calls: 13 sites — 10 in `examples/styling_showcase.rs`, 2 in `src/component/styled_text/content.rs` (method definition + `text()` delegate), 1 in `src/component/styled_text/tests.rs`
  - `StyledBlock::Paragraph` references: 4 sites — 3 in `content.rs` (variant def line 22, push body line 161, match arm line 401), 1 in `tests.rs` (pattern match line 42)
  - Combined: 17 sites across 3 files
- **cargo nextest** for unit tests (not `cargo test`). Doc tests run separately via `cargo test --all-features --doc`.
- **Audit baseline.** `./tools/audit/target/release/envision-audit scorecard` is 8/9 on main. No regression expected.

---

## File Structure

| File | Responsibility | Lines (before → after) |
|---|---|---|
| `src/render.rs` (NEW) | `pub fn styled_line(...)` + inline `#[cfg(test)] mod tests` with 5 tests | 0 → ~180 |
| `src/lib.rs` | Add `pub mod render;` + `pub use render::styled_line;` re-export | +2 lines |
| `src/component/styled_text/content.rs` | `StyledBlock::Paragraph` → `Line` variant rename; `StyledContent::paragraph()` → `line()` method rename; private `render_paragraph` → `render_line` rename; `text()` delegate update | unchanged size (in-place renames) |
| `src/component/styled_text/tests.rs` | Update 1 pattern match, 1 test function name, 1 method call | unchanged size |
| `examples/styling_showcase.rs` | 10 mechanical `.paragraph(` → `.line(` replacements | unchanged size |
| `CHANGELOG.md` | Additive entry under `[Unreleased]` | adds ~25 lines |

All files stay well under the 1000-line cap.

---

## Build + test setup verification

Run once before Task 1 to confirm the environment is healthy:

```bash
cargo build --all-features 2>&1 | tail -5
cargo nextest run --all-features styled_text:: 2>&1 | tail -10
```

Expected: build succeeds; existing styled_text tests all pass.

---

## Task 1: Rename `StyledBlock::Paragraph` + `StyledContent::paragraph()` + `render_paragraph` (atomic)

**Files:**
- Modify: `src/component/styled_text/content.rs`
- Modify: `src/component/styled_text/tests.rs`

This task is a single atomic commit covering the variant rename, the method rename, the internal helper rename, and the test fixtures that reference them. The render path's match arm flips from `StyledBlock::Paragraph(inlines) =>` to `StyledBlock::Line(inlines) =>` in the same commit.

- [ ] **Step 1: Rename the variant in `content.rs:22`**

In `src/component/styled_text/content.rs`, find the `StyledBlock` enum (line 13 onwards). The variant at line 22:

```rust
pub enum StyledBlock {
    Heading { level: u8, text: String },
    Paragraph(Vec<StyledInline>),
    BulletList(Vec<Vec<StyledInline>>),
    // ... etc
}
```

Change `Paragraph(Vec<StyledInline>),` to `Line(Vec<StyledInline>),`:

```rust
pub enum StyledBlock {
    Heading { level: u8, text: String },
    /// One line of styled inline elements (renamed from `Paragraph` —
    /// the variant produces a single line, not a wrapped block).
    Line(Vec<StyledInline>),
    BulletList(Vec<Vec<StyledInline>>),
    // ... etc
}
```

- [ ] **Step 2: Rename the builder method in `content.rs:160`**

Find the `pub fn paragraph` method definition (around line 160):

```rust
    pub fn paragraph(mut self, inlines: Vec<StyledInline>) -> Self {
        self.blocks.push(StyledBlock::Paragraph(inlines));
        self
    }
```

Replace the entire method block (preserving its docstring) with:

```rust
    /// Append a single styled line composed of inline elements.
    ///
    /// (Renamed from `paragraph(...)` — but the method produces one line,
    /// not a block-level paragraph. The `paragraph` name is reserved for
    /// future real block-level wrapped text.)
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::styled_text::{StyledContent, StyledInline};
    ///
    /// let content = StyledContent::new()
    ///     .line(vec![
    ///         StyledInline::Plain("Hello, ".to_string()),
    ///         StyledInline::Bold("world".to_string()),
    ///     ]);
    /// assert_eq!(content.len(), 1);
    /// ```
    pub fn line(mut self, inlines: Vec<StyledInline>) -> Self {
        self.blocks.push(StyledBlock::Line(inlines));
        self
    }
```

The old docstring on `paragraph(...)` referenced the `Paragraph` variant; the new docstring matches the renamed variant. The old `paragraph` method is fully replaced — no `#[deprecated]` shim.

- [ ] **Step 3: Update the `text()` delegate at `content.rs:177`**

Find the `text()` method (around line 176-178):

```rust
    pub fn text(self, text: impl Into<String>) -> Self {
        self.paragraph(vec![StyledInline::Plain(text.into())])
    }
```

Replace the body with:

```rust
    pub fn text(self, text: impl Into<String>) -> Self {
        self.line(vec![StyledInline::Plain(text.into())])
    }
```

(Public name `text()` unchanged — only the internal delegate updates.)

- [ ] **Step 4: Update the render-path match arm at `content.rs:401`**

Find the `match block { ... }` block in `fn render_block` (around line 390). The arm at line 401-403:

```rust
        StyledBlock::Paragraph(inlines) => {
            render_paragraph(inlines, theme, base_style, lines);
        }
```

Replace with:

```rust
        StyledBlock::Line(inlines) => {
            render_line(inlines, theme, base_style, lines);
        }
```

- [ ] **Step 5: Rename the private helper `render_paragraph` → `render_line` at `content.rs:457`**

Find the function definition (around line 457):

```rust
fn render_paragraph(
    inlines: &[StyledInline],
    theme: &Theme,
    base_style: Style,
    lines: &mut Vec<RatLine<'static>>,
) {
    let spans: Vec<RatSpan<'static>> = inlines
        .iter()
        .map(|i| render_inline(i, theme, base_style))
        .collect();
    lines.push(RatLine::from(spans));
}
```

Rename the function to `render_line`. Body unchanged:

```rust
fn render_line(
    inlines: &[StyledInline],
    theme: &Theme,
    base_style: Style,
    lines: &mut Vec<RatLine<'static>>,
) {
    let spans: Vec<RatSpan<'static>> = inlines
        .iter()
        .map(|i| render_inline(i, theme, base_style))
        .collect();
    lines.push(RatLine::from(spans));
}
```

- [ ] **Step 6: Update test fixtures in `tests.rs`**

In `src/component/styled_text/tests.rs`, find line 42:

```rust
        StyledBlock::Paragraph(inlines) if inlines.len() == 1
```

Replace with:

```rust
        StyledBlock::Line(inlines) if inlines.len() == 1
```

Find line 47 (the test function name):

```rust
fn test_content_paragraph_with_inlines() {
```

Replace with:

```rust
fn test_content_line_with_inlines() {
```

Find line 52 (the method call inside that test):

```rust
    let content = StyledContent::new().paragraph(inlines);
```

Replace with:

```rust
    let content = StyledContent::new().line(inlines);
```

Verify no other references to `StyledBlock::Paragraph`, `.paragraph(`, or `render_paragraph` remain in `tests.rs`:

```bash
grep -n "paragraph\|Paragraph" src/component/styled_text/tests.rs
```

Expected matches are ONLY in user-facing text content (e.g. `.text("Introduction paragraph.")`, `.text("This is a paragraph.")`) — those are English prose inside string literals and stay. No method/variant references should remain.

- [ ] **Step 7: Verify build is clean**

Run: `cargo build --all-features 2>&1 | tail -5`
Expected: clean build, no warnings, no errors.

- [ ] **Step 8: Verify tests pass + snapshots unchanged**

Run: `cargo nextest run --all-features styled_text:: 2>&1 | tail -10`
Expected: all styled_text tests pass — including any rendering snapshot tests. **Critical:** insta should NOT report any pending snapshot diffs. If `cargo nextest` output mentions pending snapshots or insta review prompts, investigate before committing. The rename is internal-only; rendered output must be byte-identical.

Also verify globally:

```bash
git status src/component/styled_text/snapshots/
```

Expected: no `.snap.new` files. If any appear, the rename inadvertently changed rendered output — debug before committing.

- [ ] **Step 9: Verify no stale references in src/**

Run:

```bash
grep -rn "StyledBlock::Paragraph\|render_paragraph" src/ 2>&1
```

Expected: zero matches.

Run:

```bash
grep -rn '\.paragraph(' src/ 2>&1
```

Expected: zero matches (the only remaining `.paragraph(` calls live in `examples/styling_showcase.rs`, addressed in Task 2).

- [ ] **Step 10: Commit**

```bash
git add src/component/styled_text/content.rs src/component/styled_text/tests.rs
git commit -S -m "Rename StyledBlock::Paragraph -> Line and StyledContent::paragraph -> line

Single atomic rename covering:
- StyledBlock::Paragraph(Vec<StyledInline>) -> StyledBlock::Line(Vec<StyledInline>)
- StyledContent::paragraph(inlines) -> StyledContent::line(inlines) (old deleted)
- private fn render_paragraph -> fn render_line
- match arm + internal text() delegate updated
- test fixture pattern + test function name updated

The method produced one line, not a wrapped block-level paragraph — the
old name was a misnomer. Source-level coherence: variant and method
share the new Line name. The paragraph name is reserved for future
real block-level wrapped-text semantics.

No rendered-output changes; insta snapshots byte-identical pre/post.
Breaking change for external consumers matching StyledBlock::Paragraph
or calling .paragraph() (envision is pre-1.0)."
```

---

## Task 2: Migrate examples/styling_showcase.rs

**Files:**
- Modify: `examples/styling_showcase.rs`

10 mechanical `.paragraph(` → `.line(` replacements.

- [ ] **Step 1: Verify the current call-site count**

Run: `grep -c '\.paragraph(' examples/styling_showcase.rs`
Expected: `10`

- [ ] **Step 2: Apply the mechanical replacement**

Replace all 10 occurrences of `.paragraph(` with `.line(` in `examples/styling_showcase.rs`. The neighboring code (the `vec![StyledInline::...]` argument expressions) stays identical — only the method name changes.

Use a per-site Edit tool call or a single `sed`-style approach: each occurrence is `.paragraph(` (with the parenthesis included to disambiguate from any prose containing "paragraph" as a word).

After the replacement, verify:

```bash
grep -c '\.paragraph(' examples/styling_showcase.rs
```

Expected: `0`.

```bash
grep -c '\.line(' examples/styling_showcase.rs
```

Expected: `10` (or more, if the file had existing `.line(` calls from other contexts — verify via diff that the count went UP by exactly 10).

- [ ] **Step 3: Verify the example compiles**

Run: `cargo build --all-features --examples 2>&1 | tail -5`
Expected: clean build of all examples.

- [ ] **Step 4: Commit**

```bash
git add examples/styling_showcase.rs
git commit -S -m "Migrate styling_showcase example to StyledContent::line

Mechanical .paragraph( -> .line( across 10 call sites. No behavioral
changes; the example renders identically. Pairs with the variant +
method rename in the previous commit."
```

---

## Task 3: Verify global rename completion + insta snapshot stability

**Files:**
- (none — verification only)

This task confirms the rename is complete across src/ + examples/ and that insta snapshots remained byte-identical through the rename.

- [ ] **Step 1: Verify zero stale references in src/ and examples/**

Run:

```bash
grep -rn "StyledBlock::Paragraph\|render_paragraph" src/ examples/ 2>&1
```

Expected: zero matches.

Run:

```bash
grep -rn '\.paragraph(' src/ examples/ 2>&1
```

Expected: zero matches.

- [ ] **Step 2: Verify renamed surface exists**

Run:

```bash
grep -c "StyledBlock::Line\b" src/component/styled_text/content.rs
```

Expected: at least `3` (variant definition, push body in line(), match arm).

Run:

```bash
grep -c "pub fn line\b" src/component/styled_text/content.rs
```

Expected: at least `1` (the new method).

Run:

```bash
grep -c "fn render_line\b" src/component/styled_text/content.rs
```

Expected: `1`.

- [ ] **Step 3: Run full test suite + verify no snapshot diffs**

Run:

```bash
cargo nextest run --all-features 2>&1 | tail -5
```

Expected: all tests pass.

Run:

```bash
git status src/component/styled_text/snapshots/
```

Expected: clean (no `.snap.new` files; no modified `.snap` files).

If any snapshot file appears in `git status`, the rename inadvertently changed rendered output — `git diff` the snapshot and investigate.

- [ ] **Step 4: No commit**

Task 3 is verification only. If any check fails, fix in-place and amend the appropriate prior commit (Task 1 for src/, Task 2 for examples/) — but NEVER amend if any earlier commit has already been pushed. If commits are pushed, add a new corrective commit.

---

## Task 4: Add new `src/render.rs` module with `styled_line` primitive

**Files:**
- Create: `src/render.rs`
- Modify: `src/lib.rs`

This task adds the new primitive in a new top-level module. The primitive internally constructs a one-block `StyledContent` and renders via the existing path — zero new rendering logic.

- [ ] **Step 1: Create `src/render.rs` with module scaffold + `styled_line` function**

Create file `src/render.rs` with the following content:

```rust
//! Standalone render primitives that don't belong to a single component.
//!
//! These functions take a `Frame`, a `Rect`, the data to render, and a `Theme`,
//! and produce immediate rendered output. They carry no state, draw no chrome,
//! and own no border — primitives consume an externally-owned area and produce
//! styled content directly into it.
//!
//! Compose freely with the chrome-ownership protocol: a primitive called inside
//! a chrome_owned context owns no chrome of its own, so there's nothing to
//! suppress.
//!
//! # Example
//!
//! ```rust,no_run
//! use envision::component::styled_text::StyledInline;
//! use envision::render::styled_line;
//! use envision::theme::Theme;
//! use ratatui::layout::Rect;
//!
//! fn render(frame: &mut ratatui::Frame, area: Rect, theme: &Theme) {
//!     let inlines = vec![
//!         StyledInline::Plain("Hello, ".to_string()),
//!         StyledInline::Bold("world".to_string()),
//!     ];
//!     styled_line(frame, area, &inlines, theme);
//! }
//! ```

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::text::Text;

use crate::component::styled_text::{StyledContent, StyledInline};
use crate::theme::Theme;

/// Render a sequence of styled inline elements as a single line into `area`.
///
/// Equivalent to a borderless `StyledText` with one line of content, but with
/// no state plumbing — pass inlines + frame + area + theme, get rendered output.
///
/// Empty inlines render an empty buffer (no error, no panic). Inlines that
/// exceed the area width are truncated by the underlying `ratatui::Paragraph`
/// widget; no wrapping is applied (single-line semantics).
///
/// # Example
///
/// ```rust,no_run
/// use envision::component::styled_text::StyledInline;
/// use envision::render::styled_line;
/// use envision::theme::Theme;
/// use ratatui::layout::Rect;
///
/// fn render(frame: &mut ratatui::Frame, area: Rect, theme: &Theme) {
///     let inlines = vec![
///         StyledInline::Plain("status: ".to_string()),
///         StyledInline::Bold("ready".to_string()),
///     ];
///     styled_line(frame, area, &inlines, theme);
/// }
/// ```
pub fn styled_line(
    frame: &mut Frame,
    area: Rect,
    inlines: &[StyledInline],
    theme: &Theme,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    // Build a one-block StyledContent and use the existing render path.
    // Reuses render_block -> render_line -> render_inline; no new rendering logic.
    let content = StyledContent::new().line(inlines.to_vec());
    let lines = content.render_lines(area.width, theme);
    let text = Text::from(lines);
    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, area);
}
```

- [ ] **Step 2: Wire the new module into `src/lib.rs`**

In `src/lib.rs`, find the existing `pub mod scroll;` line (around line 155). Add `pub mod render;` immediately after it (alphabetical-adjacent location), then add a crate-root re-export.

Find the section that re-exports module symbols at crate root (around lines 211-220 where `pub use scroll::*;` and similar live). Add a `pub use render::styled_line;` re-export so consumers can write either `envision::render::styled_line(...)` or `envision::styled_line(...)`.

Concretely: add these two lines in their respective sections of `src/lib.rs`. The exact line numbers depend on the file shape — use `grep -n "pub mod scroll\|pub use scroll" src/lib.rs` to locate the right insertion points:

```rust
pub mod render;
```

(adjacent to other `pub mod` declarations)

```rust
pub use render::styled_line;
```

(adjacent to other `pub use module::*` re-exports — match the pattern of the file)

- [ ] **Step 3: Verify build**

Run: `cargo build --all-features 2>&1 | tail -5`
Expected: clean build, no warnings.

Run: `cargo build --all-features --examples 2>&1 | tail -5`
Expected: clean.

- [ ] **Step 4: Verify doc test compiles**

Run: `cargo test --all-features --doc render::styled_line 2>&1 | tail -10`
Expected: doc test PASS.

- [ ] **Step 5: Commit**

```bash
git add src/render.rs src/lib.rs
git commit -S -m "Add envision::render::styled_line primitive

New top-level src/render.rs module hosts standalone render primitives —
mirrors the existing envision::scroll::render_scrollbar convention.
First inhabitant is styled_line(frame, area, &[StyledInline], theme).

Internal implementation builds a one-block StyledContent and renders
via the existing render_lines path. Zero new rendering logic; lifts
the existing borderless-StyledText path into a one-call primitive.

Re-exported at the crate root: envision::styled_line works alongside
the explicit envision::render::styled_line. Tests land in the next
commit."
```

---

## Task 5: Add 5 tests for `styled_line`

**Files:**
- Modify: `src/render.rs` (append `#[cfg(test)] mod tests`)

Five named tests pinning the primitive's contract: renders inlines, applies theme color, truncates to area width, handles empty input, draws no chrome.

- [ ] **Step 1: Write the failing tests**

Append to `src/render.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::styled_text::StyledInline;
    use crate::component::test_utils::setup_render;
    use ratatui::layout::Rect;
    use ratatui::style::Color;

    #[test]
    fn test_styled_line_renders_inlines() {
        // Plain + Bold + Colored — all three appear in the rendered text in order.
        let inlines = vec![
            StyledInline::Plain("hello ".to_string()),
            StyledInline::Bold("bold".to_string()),
            StyledInline::Plain(" world".to_string()),
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

    #[test]
    fn test_styled_line_applies_theme_color() {
        // StyledInline::Colored carries an explicit fg color. ANSI red = \x1b[31m.
        let inlines = vec![StyledInline::Colored {
            text: "err".to_string(),
            fg: Some(Color::Red),
            bg: None,
        }];

        let (mut terminal, theme) = setup_render(20, 1);
        terminal
            .draw(|frame| {
                styled_line(frame, frame.area(), &inlines, &theme);
            })
            .unwrap();

        let ansi = terminal.backend().to_ansi();
        assert!(
            ansi.contains("\x1b[31m"),
            "expected red (31m) ANSI fg for Colored(Red) inline, got:\n{ansi}",
        );
    }

    #[test]
    fn test_styled_line_truncates_to_area_width() {
        // 60 chars of plain text rendered into a 20-wide area — truncates
        // per ratatui's Paragraph default. Snapshot pins truncation behavior.
        let inlines = vec![StyledInline::Plain(
            "abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWX".to_string(),
        )];

        let (mut terminal, theme) = setup_render(20, 1);
        terminal
            .draw(|frame| {
                styled_line(frame, frame.area(), &inlines, &theme);
            })
            .unwrap();

        let plain = terminal.backend().to_string();
        insta::assert_snapshot!(plain);
    }

    #[test]
    fn test_styled_line_empty_inlines_renders_nothing() {
        // Empty slice — primitive returns without panicking; buffer stays empty.
        let inlines: &[StyledInline] = &[];

        let (mut terminal, theme) = setup_render(40, 1);
        terminal
            .draw(|frame| {
                styled_line(frame, frame.area(), inlines, &theme);
            })
            .unwrap();

        let plain = terminal.backend().to_string();
        insta::assert_snapshot!(plain);
    }

    #[test]
    fn test_styled_line_no_chrome_drawn() {
        // Render into a 40x3 area but only the first row should have content;
        // rows 2 and 3 stay blank. Pins that the primitive draws no chrome /
        // no border / no fill in unused rows.
        let inlines = vec![StyledInline::Plain("only first row".to_string())];

        let (mut terminal, theme) = setup_render(40, 3);
        terminal
            .draw(|frame| {
                // Use the full frame area (40x3). styled_line renders into
                // row 0 only; rows 1 and 2 must stay blank.
                styled_line(frame, frame.area(), &inlines, &theme);
            })
            .unwrap();

        let plain = terminal.backend().to_string();
        // Snapshot captures the three-row layout; visual confirmation that
        // rows 1 and 2 are blank.
        insta::assert_snapshot!(plain);

        // Also assert rows 1 and 2 are entirely spaces (no chrome glyphs).
        let rows: Vec<&str> = plain.lines().collect();
        assert!(rows.len() >= 3, "expected 3 rows, got {}: {plain}", rows.len());
        let row1_blank = rows[1].chars().all(|c| c == ' ');
        let row2_blank = rows[2].chars().all(|c| c == ' ');
        assert!(row1_blank, "row 1 should be blank (no chrome), got: {:?}", rows[1]);
        assert!(row2_blank, "row 2 should be blank (no chrome), got: {:?}", rows[2]);
    }
}
```

- [ ] **Step 2: Run tests to verify they pass**

Run: `cargo nextest run --all-features render::tests 2>&1 | tail -15`
Expected: 5 PASS. Snapshots are created on first run for the four `insta::assert_snapshot!` calls.

If insta prompts to accept new snapshots, accept them: `cargo insta accept`. Verify by inspecting each `.snap` file under `src/render/snapshots/` (or wherever insta stores them based on test location):

- `test_styled_line_renders_inlines.snap` — should show "hello bold world" in a 40-wide row
- `test_styled_line_truncates_to_area_width.snap` — should show the first 20 chars only
- `test_styled_line_empty_inlines_renders_nothing.snap` — should be 40 spaces
- `test_styled_line_no_chrome_drawn.snap` — should show "only first row" plus padding in row 0, two blank rows below

- [ ] **Step 3: Run full test suite for no regressions**

Run: `cargo nextest run --all-features 2>&1 | tail -5`
Expected: all tests pass — 5 new + all existing.

- [ ] **Step 4: Commit**

```bash
git add src/render.rs src/render/snapshots/
git commit -S -m "Add 5 tests for envision::render::styled_line

Pins the primitive's contract:
- test_styled_line_renders_inlines: Plain + Bold + Colored render in order
- test_styled_line_applies_theme_color: Colored(Red) -> ANSI 31m
- test_styled_line_truncates_to_area_width: 60 chars -> 20-wide buffer
- test_styled_line_empty_inlines_renders_nothing: &[] -> empty buffer
- test_styled_line_no_chrome_drawn: 40x3 area -> only row 0 populated

Insta snapshots pin the rendered output for each case; direct
assertions catch ANSI color escape codes and verify unused rows
contain only spaces (no chrome glyphs)."
```

---

## Task 6: Verify clippy + fmt + doc + audit

**Files:**
- (none — verification only)

- [ ] **Step 1: Run clippy with all features**

Run: `cargo clippy --all-features --all-targets -- -D warnings 2>&1 | tail -10`
Expected: clean — no warnings.

If failures appear, fix in-place and `git commit -S -m "Address clippy lint"`.

- [ ] **Step 2: Run rustdoc with deny-warnings**

Run: `RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps 2>&1 | tail -10`
Expected: clean.

If broken intra-doc links appear (e.g., a link to `StyledBlock::Paragraph` that no longer exists), fix them.

- [ ] **Step 3: Run cargo fmt check**

Run: `cargo fmt --all -- --check 2>&1 | wc -l`
Expected: `0` (zero lines of output = clean).

If formatting drift, run `cargo fmt --all` and commit:

```bash
git add -A
git commit -S -m "Apply cargo fmt"
```

- [ ] **Step 4: Run full test suite**

Run: `cargo nextest run --all-features 2>&1 | tail -5`
Expected: all tests pass.

Run: `cargo test --all-features --doc 2>&1 | tail -5`
Expected: all doc tests pass.

- [ ] **Step 5: Run audit scorecard**

Run: `./tools/audit/target/release/envision-audit scorecard 2>&1 | grep "Result:"`
Expected: `Result: 8/9 checks passing` — same baseline as main. No regression.

The 1 pre-existing failure (`resource_gauge::set_values has no matching getter`) is unchanged from main and unrelated to this work.

- [ ] **Step 6: Commit if any fixes were needed**

If any verification step required a fix, commit it. Otherwise no commit — Task 6 is verification-only.

---

## Task 7: CHANGELOG entry

**Files:**
- Modify: `CHANGELOG.md`

- [ ] **Step 1: Add the entry under `## [Unreleased]`**

Open `CHANGELOG.md`. Find the `## [Unreleased]` section. Below the most recent existing sub-section (likely "CellStyle::Severity(Severity) (D15)" from PR #478), add this new sub-section:

```markdown
### StyledText DX: line primitive + `paragraph` rename (D5 + D14)

Two coupled changes in one PR, both targeting `StyledText` / `StyledContent`
ergonomics:

**New `envision::render` module + primitive:**

- `envision::render::styled_line(frame, area, &[StyledInline], theme)` —
  free-function primitive that renders one styled line into the given area.
  Replaces the six-types-three-methods construction pattern (`StyledTextState::new()
  .with_content(StyledContent::new().line(...)).with_show_border(false)` +
  `StyledText::view(...)`) with one call. Also re-exported at the crate root
  as `envision::styled_line`.

**Method + variant rename:**

- `StyledContent::paragraph(inlines)` → `StyledContent::line(inlines)`. The
  method always produced one line, not a wrapped block-level paragraph; the
  old name was a misnomer.
- `StyledBlock::Paragraph(Vec<StyledInline>)` → `StyledBlock::Line(Vec<StyledInline>)`.
  Internal coherence with the method rename; source-spelunkers no longer hit
  the misnomer from the variant side.
- Private helper `render_paragraph` → `render_line` (no API impact;
  source-level coherence).

Both renames delete the old names outright — no `#[deprecated]` shim. envision
is pre-1.0; one mechanical migration in the same PR.

**Breaking change:** any external code that matches `StyledBlock::Paragraph` or
calls `.paragraph(...)` on `StyledContent` must rename to `StyledBlock::Line`
and `.line(...)` respectively. Renamed APIs are functionally identical; no
behavior changes.

**Reserved for future:** the `paragraph` name is now free for real block-level
wrapped-text semantics. Lands as a separate PR when a consumer needs it.

**Migration count:** 17 call sites updated in this PR (10 in
`examples/styling_showcase.rs`, 7 internal across `src/component/styled_text/`).
```

- [ ] **Step 2: Commit**

```bash
git add CHANGELOG.md
git commit -S -m "CHANGELOG: StyledText DX (D5 + D14)

Document the new envision::render::styled_line primitive + the
StyledContent::paragraph -> line method rename + the
StyledBlock::Paragraph -> Line variant rename. Flag the breaking
change for external consumers and note the reserved 'paragraph'
name for future block-level wrapped-text semantics."
```

---

## Task 8: Final verification + push + open PR

**Files:**
- (none — verification + git only)

- [ ] **Step 1: Verify all commits are signed**

Run: `git log --show-signature main..HEAD 2>&1 | grep -c 'Good signature'`
Expected: count matches the number of commits added on this branch (5 or 6 depending on whether Task 6 needed a fix commit).

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

- [ ] **Step 3: Verify zero stale references one final time**

```bash
grep -rn "StyledBlock::Paragraph\|render_paragraph" src/ examples/ 2>&1 | wc -l
```

Expected: `0`.

```bash
grep -rn '\.paragraph(' src/ examples/ 2>&1 | wc -l
```

Expected: `0`.

- [ ] **Step 4: Push the branch**

Run: `git push -u origin styled-text-dx-impl`

(Branch name is `styled-text-dx-impl` — the implementation branch, set by the controller before plan execution begins. The current plan branch is `styled-text-dx-plan`.)

Expected: pushes cleanly.

- [ ] **Step 5: Open the implementation PR**

Run:

```bash
gh pr create --title "StyledText DX: line primitive + paragraph rename (D5 + D14)" --body "$(cat <<'EOF'
## Summary

Implementation of leadline gaps **D5** (no \"render styled Line into Rect\" primitive — six types and three method calls to draw one styled line) and **D14** (\`StyledContent::paragraph(...)\` misleadingly produces a single line, not a wrapped block-level paragraph).

Spec: PR #480 (\`docs/superpowers/specs/2026-05-09-styled-text-dx-design.md\`)
Plan: PR β (\`docs/superpowers/plans/2026-05-09-styled-text-dx.md\`)

## What changed

**New primitive:**
- \`envision::render::styled_line(frame, area, &[StyledInline], theme)\` — free-function primitive in new \`src/render.rs\` module. Re-exported at crate root.

**Method + variant rename (atomic, single PR):**
- \`StyledContent::paragraph(...)\` → \`StyledContent::line(...)\` (method)
- \`StyledBlock::Paragraph(Vec<StyledInline>)\` → \`StyledBlock::Line(Vec<StyledInline>)\` (variant)
- Private \`fn render_paragraph(...)\` → \`fn render_line(...)\` (helper)
- Old names deleted outright (pre-1.0)

**Migration:**
- 17 call sites updated (10 in \`examples/styling_showcase.rs\`, 7 internal across \`src/component/styled_text/\`)
- All insta snapshots byte-identical pre/post — rename is name-only, no rendering changes

**Breaking change:**
- External code matching \`StyledBlock::Paragraph\` or calling \`.paragraph(...)\` must rename. Functionally identical after the rename.

## Test plan

- [ ] CI green on all platforms
- [ ] Audit scorecard 8/9 (same baseline as main)
- [ ] leadline migrates \`build_summary_banner_state\` (\`app.rs:392\`) and empty-state path (\`app.rs:372\`) to direct \`envision::render::styled_line\` calls; updates any \`.paragraph(...)\` call sites to \`.line(...)\`

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
| `envision::render` module — new top-level utility namespace | Task 4 |
| `envision::render::styled_line` function signature + implementation | Task 4 |
| `StyledContent::line(inlines)` method rename | Task 1 |
| `StyledBlock::Line(Vec<StyledInline>)` variant rename | Task 1 |
| Private `render_paragraph` → `render_line` rename | Task 1 |
| Delete `paragraph` method outright (no deprecation shim) | Task 1 |
| 17-call-site mechanical migration (10 example + 7 internal) | Tasks 1 + 2 + verified in Task 3 |
| 5 named tests for `styled_line` | Task 5 |
| Insta snapshot stability (byte-identical pre/post rename) | Task 1 Step 8 + Task 3 |
| Re-export at crate root (`envision::styled_line`) | Task 4 |
| CHANGELOG entry under `[Unreleased]` | Task 7 |
| Breaking-change footprint documented | Task 7 (CHANGELOG) + Task 1 (commit message) |

All spec requirements have a corresponding task.

### 2. Placeholder scan

No "TBD", "TODO", "implement later", "fill in details". Every step has either complete code, exact commands, or explicit verification criteria.

### 3. Type consistency

- `StyledBlock::Line(Vec<StyledInline>)` shape consistent across all references. ✅
- `StyledContent::line(self, inlines: Vec<StyledInline>) -> Self` matches the old `paragraph` signature exactly. ✅
- `fn styled_line(frame: &mut Frame, area: Rect, inlines: &[StyledInline], theme: &Theme)` signature matches spec. ✅
- `fn render_line(inlines: &[StyledInline], theme: &Theme, base_style: Style, lines: &mut Vec<RatLine<'static>>)` private helper signature matches the old `render_paragraph` exactly. ✅

---

## Plan complete

The plan covers 8 tasks producing approximately 5-6 signed commits (Task 3 and Task 6 are verification-only; Task 8 is push + PR). Estimated implementation time: 2-3 hours of focused work.

After plan PR β merges, controller creates `styled-text-dx-impl` branch from main and dispatches subagent-driven execution. After implementation PR γ merges, controller opens tracking-doc PR δ marking D5 + D14 ✅ resolved in `docs/customer-feedback/2026-05-01-leadline-gaps.md`.
