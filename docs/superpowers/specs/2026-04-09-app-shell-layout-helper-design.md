# AppShell layout helper — design

**Status:** approved
**Date:** 2026-04-09
**Source:** customer feedback — overlays and views independently hardcode
the same `(header, content, footer)` layout split and drift out of sync
(e.g., HelpOverlay used `Length(3)` while the main view used `Length(4)`
after adding a breadcrumb row).
**Scope:** single PR

## Problem

Apps built on envision typically use a canonical three-region layout:
header, content, footer. Today every consumer (main view, overlays,
dialogs) independently calls `Layout::vertical` with its own copy of the
constraints. When the layout changes (e.g., a breadcrumb row is added to
the header, bumping `Length(3)` to `Length(4)`), each copy must be updated
separately. Missed copies produce off-by-one rendering bugs that are
silent — no compile error, no test failure, just a visually wrong overlay.

## Goal

Provide a reusable `AppShell` struct that encapsulates the layout
definition once, so all consumers call `.split(area)` on the same
instance and get consistent rects.

Non-goals:
- Nested layouts (sidebar + content). `AppShell` is strictly the
  top-level vertical (header, content, footer) split. Consumers can
  further subdivide the returned `content` rect themselves.
- Rendering or widget behavior. `AppShell` is a layout utility, not a
  component. It has no state, messages, or event handling.
- Replacing the existing `vertical` / `horizontal` helpers. `AppShell`
  is a higher-level convenience built on top of them.

## Design

### Types

Two new public types in `src/layout/mod.rs`:

```rust
/// A reusable app-level layout definition for the canonical
/// (header, content, footer) split.
///
/// Construct once at app init with your header/footer constraints,
/// then call [`split`](Self::split) from views and overlays to get
/// consistent rects without duplicating layout constants.
///
/// # Example
///
/// ```rust
/// use envision::layout::{AppShell, Constraint, Rect};
///
/// let shell = AppShell::new()
///     .header(Constraint::Length(4))
///     .footer(Constraint::Length(1));
///
/// let area = Rect::new(0, 0, 80, 24);
/// let regions = shell.split(area);
///
/// assert_eq!(regions.header.height, 4);
/// assert_eq!(regions.footer.height, 1);
/// assert_eq!(regions.content.height, 19); // 24 - 4 - 1
/// ```
#[derive(Clone, Copy, Debug)]
pub struct AppShell {
    header: Option<Constraint>,
    footer: Option<Constraint>,
}

/// The regions produced by [`AppShell::split`].
///
/// All three fields are always present. When no header or footer is
/// configured on the `AppShell`, the corresponding rect has zero
/// height.
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

### API on `AppShell`

- `pub fn new() -> Self` — no header, no footer.
- `pub fn header(mut self, constraint: Constraint) -> Self` — sets the
  header constraint (builder pattern).
- `pub fn footer(mut self, constraint: Constraint) -> Self` — sets the
  footer constraint (builder pattern).
- `pub fn split(self, area: Rect) -> AppRegions` — computes the three
  rects. Internally calls `vertical(area, [h, Min(0), f])` where `h`
  and `f` are the configured constraints or `Length(0)` if not set.

### Behavior

| Configuration | `header` rect | `content` rect | `footer` rect |
|---|---|---|---|
| `.header(Length(4)).footer(Length(1))` | 4 rows at top | remaining rows | 1 row at bottom |
| `.header(Length(4))` only | 4 rows at top | remaining rows | 0 rows at bottom |
| `.footer(Length(1))` only | 0 rows at top | remaining rows | 1 row at bottom |
| neither | 0 rows at top | full area | 0 rows at bottom |

Content always gets `Min(0)` — it expands to fill whatever the header
and footer don't consume.

### Derives and traits

`AppShell`: `Clone`, `Copy`, `Debug`. No `PartialEq` needed (it's a
builder, not a value type you'd compare).

`AppRegions`: `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`. Equality
is useful for testing.

### Location

Both types are added to `src/layout/mod.rs` (currently 157 lines).
They're re-exported via `pub use` in `src/lib.rs` alongside the
existing `vertical`, `horizontal`, `centered` helpers — so consumers
write `use envision::layout::{AppShell, AppRegions, Constraint, Rect}`.

## Testing

Unit tests in `src/layout/tests.rs` (currently 273 lines):

- `test_app_shell_header_and_footer` — both configured, verify all
  three rects have the expected position/height/width.
- `test_app_shell_header_only` — footer is zero-height at bottom.
- `test_app_shell_footer_only` — header is zero-height at top.
- `test_app_shell_neither` — content is the full area, header/footer
  are zero-height.
- `test_app_shell_area_too_small` — area smaller than header + footer.
  Verify no panic; ratatui's `Layout` handles this by truncating.
- `test_app_shell_zero_area` — `Rect::default()`. Verify no panic.

Doc tests on `AppShell::new`, `header`, `footer`, `split`, and
`AppRegions`.

## Risk and rollback

Risk is very low. This is a pure addition — two new types and one
function, no existing code modified. The types are standalone (they
don't change any existing component or trait). Rollback: revert the PR.
