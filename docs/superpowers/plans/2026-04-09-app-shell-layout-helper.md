# AppShell Layout Helper — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `AppShell` and `AppRegions` types to `src/layout/mod.rs` so apps can define their (header, content, footer) layout split once and share it across views and overlays.

**Architecture:** `AppShell` is a `Copy` struct with optional header/footer `Constraint`s. Its `.split(area)` method delegates to the existing `vertical` helper and returns `AppRegions { header, content, footer }`.

**Tech Stack:** Rust (edition 2024), ratatui layout types, cargo-nextest.

**Spec:** `docs/superpowers/specs/2026-04-09-app-shell-layout-helper-design.md`

---

## File Structure

- Modify: `src/layout/mod.rs` — add `AppShell`, `AppRegions`, and their methods
- Modify: `src/layout/tests.rs` — add unit tests
- Modify: `CHANGELOG.md` — add entry under `[Unreleased]`

---

## Task 1: Add `AppRegions` struct with tests (TDD)

**Files:**
- Modify: `src/layout/mod.rs`
- Modify: `src/layout/tests.rs`

---

- [ ] **Step 1.1: Write the failing test for `AppRegions`**

Add to `src/layout/tests.rs`:

```rust
#[test]
fn test_app_regions_equality() {
    use super::{AppRegions, Rect};

    let regions_a = AppRegions {
        header: Rect::new(0, 0, 80, 4),
        content: Rect::new(0, 4, 80, 19),
        footer: Rect::new(0, 23, 80, 1),
    };
    let regions_b = AppRegions {
        header: Rect::new(0, 0, 80, 4),
        content: Rect::new(0, 4, 80, 19),
        footer: Rect::new(0, 23, 80, 1),
    };
    assert_eq!(regions_a, regions_b);
}
```

- [ ] **Step 1.2: Run to verify failure (compile error — type doesn't exist)**

```bash
cargo nextest run -p envision layout::tests::test_app_regions_equality
```

Expected: compile error.

- [ ] **Step 1.3: Add `AppRegions` to `src/layout/mod.rs`**

Insert before the `#[cfg(test)]` line at the bottom of the file (before line 156):

```rust
/// The regions produced by [`AppShell::split`].
///
/// All three fields are always present. When no header or footer is
/// configured on the [`AppShell`], the corresponding rect has zero
/// height.
///
/// # Example
///
/// ```rust
/// use envision::layout::{AppShell, AppRegions, Constraint, Rect};
///
/// let shell = AppShell::new()
///     .header(Constraint::Length(3))
///     .footer(Constraint::Length(1));
///
/// let regions = shell.split(Rect::new(0, 0, 80, 24));
/// assert_eq!(regions.header.height, 3);
/// assert_eq!(regions.content.height, 20);
/// assert_eq!(regions.footer.height, 1);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppRegions {
    /// The header area. Zero-height if no header was configured.
    pub header: Rect,
    /// The main content area.
    pub content: Rect,
    /// The footer area. Zero-height if no footer was configured.
    pub footer: Rect,
}
```

- [ ] **Step 1.4: Run test, verify it passes**

```bash
cargo nextest run -p envision layout::tests::test_app_regions_equality
```

Expected: PASS.

- [ ] **Step 1.5: Format, lint, commit**

```bash
cargo fmt
cargo clippy -p envision -- -D warnings
git add src/layout/mod.rs src/layout/tests.rs
git commit -S -m "$(cat <<'EOF'
Add AppRegions struct for canonical layout regions

A simple struct with pub fields (header, content, footer) that
AppShell::split will return. Derives Clone, Copy, Debug, PartialEq, Eq.

Part of docs/superpowers/specs/2026-04-09-app-shell-layout-helper-design.md

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: Add `AppShell` struct with builder + split + tests (TDD)

**Files:**
- Modify: `src/layout/mod.rs`
- Modify: `src/layout/tests.rs`

---

- [ ] **Step 2.1: Write the failing tests**

Add to `src/layout/tests.rs`:

```rust
#[test]
fn test_app_shell_header_and_footer() {
    use super::{AppShell, Constraint, Rect};

    let shell = AppShell::new()
        .header(Constraint::Length(4))
        .footer(Constraint::Length(1));
    let regions = shell.split(Rect::new(0, 0, 80, 24));

    assert_eq!(regions.header, Rect::new(0, 0, 80, 4));
    assert_eq!(regions.content, Rect::new(0, 4, 80, 19));
    assert_eq!(regions.footer, Rect::new(0, 23, 80, 1));
}

#[test]
fn test_app_shell_header_only() {
    use super::{AppShell, Constraint, Rect};

    let shell = AppShell::new().header(Constraint::Length(4));
    let regions = shell.split(Rect::new(0, 0, 80, 24));

    assert_eq!(regions.header, Rect::new(0, 0, 80, 4));
    assert_eq!(regions.content, Rect::new(0, 4, 80, 20));
    assert_eq!(regions.footer.height, 0);
}

#[test]
fn test_app_shell_footer_only() {
    use super::{AppShell, Constraint, Rect};

    let shell = AppShell::new().footer(Constraint::Length(1));
    let regions = shell.split(Rect::new(0, 0, 80, 24));

    assert_eq!(regions.header.height, 0);
    assert_eq!(regions.content, Rect::new(0, 0, 80, 23));
    assert_eq!(regions.footer, Rect::new(0, 23, 80, 1));
}

#[test]
fn test_app_shell_neither() {
    use super::{AppShell, Rect};

    let shell = AppShell::new();
    let regions = shell.split(Rect::new(0, 0, 80, 24));

    assert_eq!(regions.header.height, 0);
    assert_eq!(regions.content, Rect::new(0, 0, 80, 24));
    assert_eq!(regions.footer.height, 0);
}

#[test]
fn test_app_shell_area_too_small() {
    use super::{AppShell, Constraint, Rect};

    // Area is 5 rows but header wants 4 + footer wants 3 = 7.
    // ratatui's Layout handles this by truncating; no panic.
    let shell = AppShell::new()
        .header(Constraint::Length(4))
        .footer(Constraint::Length(3));
    let regions = shell.split(Rect::new(0, 0, 80, 5));

    // Content gets Min(0) so it may be 0 height. The exact
    // distribution depends on ratatui's layout solver; we just
    // verify no panic and the total height doesn't exceed area.
    let total = regions.header.height + regions.content.height + regions.footer.height;
    assert!(total <= 5, "total height {} exceeds area height 5", total);
}

#[test]
fn test_app_shell_zero_area() {
    use super::{AppShell, Constraint, Rect};

    let shell = AppShell::new()
        .header(Constraint::Length(4))
        .footer(Constraint::Length(1));
    let regions = shell.split(Rect::default());

    // All rects should be zero-sized. No panic.
    assert_eq!(regions.header.area(), 0);
    assert_eq!(regions.content.area(), 0);
    assert_eq!(regions.footer.area(), 0);
}
```

- [ ] **Step 2.2: Run to verify failure (compile error)**

```bash
cargo nextest run -p envision layout::tests::test_app_shell
```

Expected: compile error — `AppShell` doesn't exist yet.

- [ ] **Step 2.3: Add `AppShell` to `src/layout/mod.rs`**

Insert after the `AppRegions` struct definition and before the `#[cfg(test)]` line:

```rust
/// A reusable app-level layout definition for the canonical
/// (header, content, footer) split.
///
/// Construct once at app init with your header and/or footer constraints,
/// then call [`split`](Self::split) from views and overlays to get
/// consistent rects without duplicating layout constants.
///
/// # Example
///
/// ```rust
/// use envision::layout::{AppShell, Constraint, Rect};
///
/// // Define the layout once at app init.
/// let shell = AppShell::new()
///     .header(Constraint::Length(4))
///     .footer(Constraint::Length(1));
///
/// // Use it from both the main view and overlays.
/// let area = Rect::new(0, 0, 80, 24);
/// let regions = shell.split(area);
///
/// assert_eq!(regions.header.height, 4);
/// assert_eq!(regions.content.height, 19);
/// assert_eq!(regions.footer.height, 1);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct AppShell {
    header: Option<Constraint>,
    footer: Option<Constraint>,
}

impl AppShell {
    /// Creates a new `AppShell` with no header or footer.
    ///
    /// Content will occupy the full area until a header or footer is
    /// configured via [`header`](Self::header) or
    /// [`footer`](Self::footer).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::layout::{AppShell, Rect};
    ///
    /// let shell = AppShell::new();
    /// let regions = shell.split(Rect::new(0, 0, 80, 24));
    /// assert_eq!(regions.content.height, 24);
    /// ```
    pub fn new() -> Self {
        Self {
            header: None,
            footer: None,
        }
    }

    /// Sets the header constraint (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::layout::{AppShell, Constraint, Rect};
    ///
    /// let shell = AppShell::new().header(Constraint::Length(3));
    /// let regions = shell.split(Rect::new(0, 0, 80, 24));
    /// assert_eq!(regions.header.height, 3);
    /// ```
    pub fn header(mut self, constraint: Constraint) -> Self {
        self.header = Some(constraint);
        self
    }

    /// Sets the footer constraint (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::layout::{AppShell, Constraint, Rect};
    ///
    /// let shell = AppShell::new().footer(Constraint::Length(1));
    /// let regions = shell.split(Rect::new(0, 0, 80, 24));
    /// assert_eq!(regions.footer.height, 1);
    /// ```
    pub fn footer(mut self, constraint: Constraint) -> Self {
        self.footer = Some(constraint);
        self
    }

    /// Splits the given area into header, content, and footer regions.
    ///
    /// Content always receives [`Constraint::Min(0)`] so it expands
    /// to fill whatever the header and footer don't consume.
    ///
    /// When no header or footer is configured, the corresponding region
    /// has zero height.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::layout::{AppShell, Constraint, Rect};
    ///
    /// let shell = AppShell::new()
    ///     .header(Constraint::Length(4))
    ///     .footer(Constraint::Length(1));
    /// let regions = shell.split(Rect::new(0, 0, 80, 24));
    ///
    /// assert_eq!(regions.header.height, 4);
    /// assert_eq!(regions.content.height, 19);
    /// assert_eq!(regions.footer.height, 1);
    /// ```
    pub fn split(self, area: Rect) -> AppRegions {
        let h = self.header.unwrap_or(Constraint::Length(0));
        let f = self.footer.unwrap_or(Constraint::Length(0));
        let [header, content, footer] = vertical(area, [h, Constraint::Min(0), f]);
        AppRegions {
            header,
            content,
            footer,
        }
    }
}

impl Default for AppShell {
    fn default() -> Self {
        Self::new()
    }
}
```

- [ ] **Step 2.4: Run all tests**

```bash
cargo nextest run -p envision layout::tests
```

Expected: all layout tests pass (existing + 7 new).

- [ ] **Step 2.5: Run doc tests**

```bash
cargo test --doc -p envision layout
```

Expected: all doc tests pass.

- [ ] **Step 2.6: Format, lint, commit**

```bash
cargo fmt
cargo clippy -p envision -- -D warnings
git add src/layout/mod.rs src/layout/tests.rs
git commit -S -m "$(cat <<'EOF'
Add AppShell layout helper for consistent header/content/footer splits

AppShell encapsulates the canonical (header, content, footer) layout
definition so apps can construct it once at init and call .split(area)
from both views and overlays — eliminating the off-by-one bugs that
arise when multiple consumers independently hardcode the same layout
constraints.

Builder pattern: AppShell::new().header(Length(4)).footer(Length(1)).
Header and footer are both optional — unconfigured regions produce
zero-height rects. Content always gets Min(0).

Part of docs/superpowers/specs/2026-04-09-app-shell-layout-helper-design.md

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: CHANGELOG + full verification + PR

**Files:**
- Modify: `CHANGELOG.md`

---

- [ ] **Step 3.1: Add changelog entry**

Under `## [Unreleased]` in `CHANGELOG.md`, append to the existing
`### Added` subsection:

```markdown
- `AppShell` layout helper for consistent `(header, content, footer)`
  splits. Construct once at app init with
  `AppShell::new().header(Constraint::Length(4)).footer(Constraint::Length(1))`,
  then call `.split(area)` from views and overlays to get the same rects
  without duplicating layout constants. Returns `AppRegions` with
  `header`, `content`, and `footer` fields. Header and footer are
  optional — unconfigured regions produce zero-height rects.
```

- [ ] **Step 3.2: Full test suite**

```bash
cargo nextest run -p envision
cargo test --doc -p envision
```

Expected: all tests pass.

- [ ] **Step 3.3: Format, clippy, build**

```bash
cargo fmt
cargo clippy -p envision --all-targets -- -D warnings
cargo build -p envision
```

- [ ] **Step 3.4: Commit**

```bash
git add CHANGELOG.md
git commit -S -m "$(cat <<'EOF'
Document AppShell layout helper in CHANGELOG

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
EOF
)"
```

- [ ] **Step 3.5: Merge origin/main + push + open PR**

```bash
git fetch origin
git merge origin/main --no-ff -S
git push -u origin app-shell-layout-helper
gh pr create --title "Add AppShell layout helper for consistent header/content/footer splits" --body "$(cat <<'EOF'
## Summary

- Adds `AppShell` and `AppRegions` types to `envision::layout` for defining the canonical `(header, content, footer)` layout split once and reusing it across views and overlays.
- Solves the off-by-one drift problem where overlays independently hardcode layout constraints that silently diverge from the main view (e.g., HelpOverlay using `Length(3)` after the app added a breadcrumb row and moved to `Length(4)`).
- Builder pattern: `AppShell::new().header(Constraint::Length(4)).footer(Constraint::Length(1))`. Both header and footer are optional.
- `.split(area)` returns `AppRegions { header, content, footer }`. Unconfigured regions produce zero-height rects.

Design spec: `docs/superpowers/specs/2026-04-09-app-shell-layout-helper-design.md`

Addresses customer feedback for the next release.

## Test plan

- [x] 7 unit tests covering all configurations (both, header-only, footer-only, neither, too-small area, zero area, equality)
- [x] Doc tests on `AppShell::new`, `header`, `footer`, `split`, and `AppRegions`
- [x] `cargo nextest run -p envision layout::tests` — all pass
- [x] `cargo clippy -p envision --all-targets -- -D warnings` — no warnings
- [x] `cargo fmt` — no diffs

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

- [ ] **Step 3.6: Check CI**

```bash
gh pr checks $(gh pr view --json number -q .number)
```

---

## Definition of done

- [ ] `AppShell` and `AppRegions` exist in `src/layout/mod.rs`.
- [ ] Builder methods `new`, `header`, `footer` work.
- [ ] `split(area)` returns correct rects for all configurations.
- [ ] 7 unit tests + doc tests pass.
- [ ] CHANGELOG updated.
- [ ] PR opened, CI green.
