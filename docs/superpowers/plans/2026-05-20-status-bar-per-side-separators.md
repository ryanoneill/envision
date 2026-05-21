# StatusBar per-side separator overrides (D12) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add three optional per-side separator overrides to `StatusBarState` (`left_separator`, `center_separator`, `right_separator`) with matching builders. Per-side override takes precedence over the existing global `separator` at render time. Defaults to current behavior when not opted into.

**Architecture:** Purely additive. Three `Option<String>` fields cluster next to the existing global `separator` field on `StatusBarState`. Three new builder methods (`with_left_separator`, `with_center_separator`, `with_right_separator`) take `mut self` (consistent with existing `with_disabled`). Three matching getters return `Option<&str>`. Render-path at `mod.rs:850-852` gains three single-line `as_deref().unwrap_or(&state.separator)` fallback expressions; the `render_section(items, separator: &str, theme)` signature at line 656 takes `&str` already — no helper signature change needed.

**Tech Stack:** Rust 1.85+, ratatui 0.29 (`Color`, `Span`), envision `status_bar` component, serde (gated behind `serialization` feature).

**Spec:** `docs/superpowers/specs/2026-05-20-status-bar-per-side-separators-design.md` (PR #493, merged)

---

## Pre-execution gotchas

- **Signed commits required.** Project rule. If `git commit -S` fails, ask the user; never bypass with `--no-gpg-sign`.
- **File-size cap (1000 lines).** `src/component/status_bar/mod.rs` is at 919 lines today. Adding 3 fields (+~6 lines) + 6 methods with `# Example` doc tests (+~60 lines) + render-path tweak (+3 net lines) projects to ~990 lines — close to but under cap. **After Task 1 completes, run `wc -l src/component/status_bar/mod.rs`**. If the count exceeds 995, mitigation: move the 6 new methods into a sibling file `src/component/status_bar/per_side_separators.rs` (same multi-file impl pattern as `pane_layout/title_style.rs` from G4). Decide based on actual count; don't pre-commit.
- **`with_separator(...)` static constructor uses `..Self::default()` spread.** Line 251-256 — the existing constructor picks up the new field defaults automatically via the spread. No edit needed beyond verifying the spread is intact in the existing code after Task 1's `Default` impl update.
- **`fn render_section` signature is `&str`, not `&String`.** Verified at mod.rs:656 — `fn render_section(items: &[StatusBarItem], separator: &str, theme: &Theme)`. The current call site `Self::render_section(&state.left, &state.separator, ...)` passes `&String` which works via deref coercion. The new form `Self::render_section(&state.left, left_sep, ...)` where `left_sep: &str = state.left_separator.as_deref().unwrap_or(&state.separator)` works directly. No helper signature change needed.
- **`StatusBarState` derives `PartialEq` not `Eq`.** Adding `Option<String>` fields preserves existing derives (`String` is `PartialEq` but not `Eq`).
- **Audit forward-note (lesson from G4+G5 PR #487 + G6 PR #491).** Envision's audit requires 100% doc-test coverage on public functions. Every one of the 6 new public methods (3 builders + 3 getters) MUST ship with a `# Example` doc test. Plan provides these explicitly in Task 1.
- **`#[serde(default)]` on all 3 new fields.** Preserves forward-compat for pre-D12 serialized `StatusBarState` blobs — `Option<String>` defaults to `None`, behavior identical to pre-D12.
- **cargo-nextest** for unit tests; doc tests via `cargo test --all-features --doc`.
- **cargo build --no-default-features** must pass (preventive check from D5+D14 lesson).
- **Audit baseline 8/9.** Resource_gauge gap is pre-existing on main.

---

## File Structure

| File | Responsibility | Lines (before → after) |
|---|---|---|
| `src/component/status_bar/mod.rs` | `StatusBarState` struct + impl; adds 3 fields + 6 methods + 3-line render-path tweak | 919 → ~990 (under cap; sibling-file fallback if >995) |
| `src/component/status_bar/tests/state.rs` | Existing unit-test file; adds 6 new unit tests | 547 → ~640 |
| `src/component/status_bar/snapshot_tests.rs` | Existing snapshot tests; adds 2 new render-path tests | 185 → ~250 |
| `CHANGELOG.md` | Additive entry under `[Unreleased]` | adds ~25 lines |

All files projected under 1000-line cap.

---

## Build + test setup verification

Run once before Task 1 to confirm the environment is healthy:

```bash
cargo build --all-features 2>&1 | tail -5
cargo build --no-default-features 2>&1 | tail -3
cargo nextest run --all-features status_bar:: 2>&1 | tail -10
```

Expected: build succeeds in both feature configurations; existing status_bar tests all pass.

---

## Task 1: Add 3 fields + 3 builders + 3 getters + render-path update

**Files:**
- Modify: `src/component/status_bar/mod.rs` (fields + Default impl + builders + getters + render-path)

This task is one atomic commit covering the production-code side of D12. Eight new tests land in Task 2 separately for review granularity.

- [ ] **Step 1: Add three new fields to `StatusBarState`**

In `src/component/status_bar/mod.rs`, find the struct definition (around lines 182-195):

```rust
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct StatusBarState {
    /// Items aligned to the left.
    left: Vec<StatusBarItem>,
    /// Items aligned to the center.
    center: Vec<StatusBarItem>,
    /// Items aligned to the right.
    right: Vec<StatusBarItem>,
    /// The separator character to use between items.
    separator: String,
    /// Background style for the entire bar.
    background: Color,
    /// Whether the component is disabled.
    disabled: bool,
}
```

Insert three new fields immediately after `separator: String,`:

```rust
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct StatusBarState {
    /// Items aligned to the left.
    left: Vec<StatusBarItem>,
    /// Items aligned to the center.
    center: Vec<StatusBarItem>,
    /// Items aligned to the right.
    right: Vec<StatusBarItem>,
    /// The separator character to use between items (global default).
    separator: String,
    /// Per-side override for the left section. When `Some`, takes
    /// precedence over `separator` for left-section rendering.
    #[cfg_attr(feature = "serialization", serde(default))]
    left_separator: Option<String>,
    /// Per-side override for the center section. When `Some`, takes
    /// precedence over `separator` for center-section rendering.
    #[cfg_attr(feature = "serialization", serde(default))]
    center_separator: Option<String>,
    /// Per-side override for the right section. When `Some`, takes
    /// precedence over `separator` for right-section rendering.
    #[cfg_attr(feature = "serialization", serde(default))]
    right_separator: Option<String>,
    /// Background style for the entire bar.
    background: Color,
    /// Whether the component is disabled.
    disabled: bool,
}
```

The three new fields cluster next to the existing global `separator` field (per JC2 from the spec). `#[serde(default)]` preserves forward-compat for pre-D12 serialized blobs.

- [ ] **Step 2: Update the `Default` impl**

Find `impl Default for StatusBarState` (around line 197-222). The current `default()` body initializes 6 fields. Add the three new fields (between `separator:` and `background:`):

```rust
impl Default for StatusBarState {
    /// Creates a default empty status bar with `" | "` separator and
    /// `DarkGray` background.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusBarState;
    ///
    /// let state = StatusBarState::default();
    /// assert!(state.left().is_empty());
    /// assert!(state.center().is_empty());
    /// assert!(state.right().is_empty());
    /// assert_eq!(state.separator(), " | ");
    /// ```
    fn default() -> Self {
        Self {
            left: Vec::new(),
            center: Vec::new(),
            right: Vec::new(),
            separator: " | ".to_string(),
            left_separator: None,
            center_separator: None,
            right_separator: None,
            background: Color::DarkGray,
            disabled: false,
        }
    }
}
```

The existing `StatusBarState::with_separator(...)` constructor at line 251-256 uses `..Self::default()` spread, so it picks up the new field defaults automatically without edit. Verify after this step that the spread is intact:

```bash
sed -n '251,256p' src/component/status_bar/mod.rs
```

Expected: shows `pub fn with_separator(...)` returning `Self { separator: separator.into(), ..Self::default() }` (or similar with the spread).

- [ ] **Step 3: Add three new builder methods + three matching getters**

Find the existing `pub fn with_disabled` method (around line 515). Insert the six new methods clustered together immediately AFTER `with_disabled` (or BEFORE — either side preserves the "style-related-builders-cluster-together" intent). Suggested location: immediately after `with_disabled`:

```rust
    /// Sets the separator for the left section (builder pattern).
    ///
    /// When set, takes precedence over the global `separator` for
    /// left-section rendering. When `None` (default), the global
    /// `separator` applies.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusBarState;
    ///
    /// let state = StatusBarState::with_separator(" · ")
    ///     .with_left_separator(" | ");
    /// assert_eq!(state.left_separator(), Some(" | "));
    /// assert_eq!(state.separator(), " · "); // global unchanged
    /// ```
    pub fn with_left_separator(mut self, separator: impl Into<String>) -> Self {
        self.left_separator = Some(separator.into());
        self
    }

    /// Sets the separator for the center section (builder pattern).
    ///
    /// When set, takes precedence over the global `separator` for
    /// center-section rendering. When `None` (default), the global
    /// `separator` applies.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusBarState;
    ///
    /// let state = StatusBarState::new().with_center_separator(" :: ");
    /// assert_eq!(state.center_separator(), Some(" :: "));
    /// ```
    pub fn with_center_separator(mut self, separator: impl Into<String>) -> Self {
        self.center_separator = Some(separator.into());
        self
    }

    /// Sets the separator for the right section (builder pattern).
    ///
    /// When set, takes precedence over the global `separator` for
    /// right-section rendering. When `None` (default), the global
    /// `separator` applies.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusBarState;
    ///
    /// // Global " · " separator, but " " between right-section items.
    /// let state = StatusBarState::with_separator(" · ")
    ///     .with_right_separator(" ");
    /// assert_eq!(state.right_separator(), Some(" "));
    /// assert_eq!(state.separator(), " · "); // global unchanged
    /// ```
    pub fn with_right_separator(mut self, separator: impl Into<String>) -> Self {
        self.right_separator = Some(separator.into());
        self
    }

    /// Returns the left-section separator override, if set.
    ///
    /// `None` means the left section uses the global `separator()`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusBarState;
    ///
    /// let plain = StatusBarState::new();
    /// assert_eq!(plain.left_separator(), None);
    ///
    /// let overridden = plain.with_left_separator(" | ");
    /// assert_eq!(overridden.left_separator(), Some(" | "));
    /// ```
    pub fn left_separator(&self) -> Option<&str> {
        self.left_separator.as_deref()
    }

    /// Returns the center-section separator override, if set.
    ///
    /// `None` means the center section uses the global `separator()`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusBarState;
    ///
    /// let plain = StatusBarState::new();
    /// assert_eq!(plain.center_separator(), None);
    ///
    /// let overridden = plain.with_center_separator(" :: ");
    /// assert_eq!(overridden.center_separator(), Some(" :: "));
    /// ```
    pub fn center_separator(&self) -> Option<&str> {
        self.center_separator.as_deref()
    }

    /// Returns the right-section separator override, if set.
    ///
    /// `None` means the right section uses the global `separator()`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusBarState;
    ///
    /// let plain = StatusBarState::new();
    /// assert_eq!(plain.right_separator(), None);
    ///
    /// let overridden = plain.with_right_separator(" ");
    /// assert_eq!(overridden.right_separator(), Some(" "));
    /// ```
    pub fn right_separator(&self) -> Option<&str> {
        self.right_separator.as_deref()
    }
```

Each method has a `# Example` doc test (mandatory per envision's 100% doc-test-coverage audit requirement — lesson from G4+G5 PR #487 and G6 PR #491 regressions).

- [ ] **Step 4: Update the render path with per-side fallback**

Find the render block in `Component::view`'s impl (around line 850-852). The current three-line block:

```rust
        let left_spans = Self::render_section(&state.left, &state.separator, ctx.theme);
        let center_spans = Self::render_section(&state.center, &state.separator, ctx.theme);
        let right_spans = Self::render_section(&state.right, &state.separator, ctx.theme);
```

Replace with the per-side fallback (precompute each side's separator, then call `render_section`):

```rust
        let left_sep = state.left_separator.as_deref().unwrap_or(&state.separator);
        let center_sep = state.center_separator.as_deref().unwrap_or(&state.separator);
        let right_sep = state.right_separator.as_deref().unwrap_or(&state.separator);
        let left_spans = Self::render_section(&state.left, left_sep, ctx.theme);
        let center_spans = Self::render_section(&state.center, center_sep, ctx.theme);
        let right_spans = Self::render_section(&state.right, right_sep, ctx.theme);
```

`render_section` takes `separator: &str` (verified at mod.rs:656); no signature change needed. `left_sep`, `center_sep`, `right_sep` are all `&str` from `as_deref().unwrap_or(&str)`.

- [ ] **Step 5: Verify build is clean**

Run: `cargo build --all-features 2>&1 | tail -5`
Expected: clean build, no warnings, no errors.

Run: `cargo build --no-default-features 2>&1 | tail -3`
Expected: clean (status_bar lives behind `display-components` per the existing module structure; the new fields don't change feature-gating).

- [ ] **Step 6: Run existing tests + doc tests**

Run: `cargo nextest run --all-features status_bar:: 2>&1 | tail -5`
Expected: all existing status_bar tests pass (the new fields don't break any existing behavior; render path produces identical output for defaults).

Run: `cargo test --all-features --doc status_bar 2>&1 | tail -5`
Expected: all 6 new `# Example` doc tests pass (3 builders + 3 getters).

- [ ] **Step 7: Verify file-size constraint**

Run: `wc -l src/component/status_bar/mod.rs`
Expected: under 1000 (target ~990).

**If the count exceeds 995:** STOP and split per the file-size mitigation. Move the 6 new methods (`with_left_separator`, `with_center_separator`, `with_right_separator`, `left_separator`, `center_separator`, `right_separator`) into a new sibling file `src/component/status_bar/per_side_separators.rs` housing them in a second `impl StatusBarState { ... }` block. Add `pub mod per_side_separators;` to `mod.rs`. The field declarations + Default impl + render-path stay in mod.rs. Same multi-module impl pattern as `pane_layout/title_style.rs` from G4 (PR #487).

If the split is needed, re-run Steps 5-6 after the split to confirm everything still compiles and tests pass.

- [ ] **Step 8: Commit Task 1**

```bash
git add src/component/status_bar/mod.rs
# If sibling-file split was needed:
# git add src/component/status_bar/per_side_separators.rs

git commit -S -m "Add StatusBarState per-side separator overrides (D12)

Three new Option<String> fields on StatusBarState (left_separator,
center_separator, right_separator) with #[serde(default)] clustered
next to the existing global separator field. Three new builder methods
+ three matching getters consistent with existing with_disabled shape.

Render path at mod.rs:850-852 gains per-side fallback resolution:
state.<side>_separator.as_deref().unwrap_or(&state.separator). No
render_section signature change — it already takes &str.

Purely additive; default state has all three Options as None, preserving
identical behavior for consumers who don't opt in. Each builder + getter
ships with a # Example doc test (lesson from G4+G5 + G6 audit-coverage
regressions).

Layered semantics, not last-call-wins: per-side override is independent
of global separator. Setting with_separator(\" · \").with_right_separator(\" \")
results in both fields populated; render-time precedence resolves which
applies per section."
```

---

## Task 2: Add 8 named tests (6 unit + 2 snapshot)

**Files:**
- Modify: `src/component/status_bar/tests/state.rs` (append 6 unit tests)
- Modify: `src/component/status_bar/snapshot_tests.rs` (append 2 render-path tests)

- [ ] **Step 1: Append 6 unit tests to `tests/state.rs`**

In `src/component/status_bar/tests/state.rs`, append these 6 tests at the end of the file:

```rust
#[test]
fn test_default_per_side_separators_none() {
    use crate::component::StatusBarState;

    // Defaults: all three per-side overrides are None.
    let state = StatusBarState::default();
    assert_eq!(state.left_separator(), None);
    assert_eq!(state.center_separator(), None);
    assert_eq!(state.right_separator(), None);
    // Global separator unchanged from existing default.
    assert_eq!(state.separator(), " | ");
}

#[test]
fn test_with_left_separator_sets_field() {
    use crate::component::StatusBarState;

    let state = StatusBarState::new().with_left_separator(" | ");
    assert_eq!(state.left_separator(), Some(" | "));
    // Other per-side overrides remain None.
    assert_eq!(state.center_separator(), None);
    assert_eq!(state.right_separator(), None);
    // Global separator unchanged.
    assert_eq!(state.separator(), " | ");
}

#[test]
fn test_with_center_separator_sets_field() {
    use crate::component::StatusBarState;

    let state = StatusBarState::new().with_center_separator(" :: ");
    assert_eq!(state.center_separator(), Some(" :: "));
    // Other per-side overrides remain None.
    assert_eq!(state.left_separator(), None);
    assert_eq!(state.right_separator(), None);
}

#[test]
fn test_with_right_separator_sets_field() {
    use crate::component::StatusBarState;

    let state = StatusBarState::new().with_right_separator(" ");
    assert_eq!(state.right_separator(), Some(" "));
    // Other per-side overrides remain None.
    assert_eq!(state.left_separator(), None);
    assert_eq!(state.center_separator(), None);
}

#[test]
fn test_with_separator_then_per_side_override() {
    use crate::component::StatusBarState;

    // LAYERED SEMANTICS PIN: per-side override doesn't clear the global.
    // Chain with_separator(...) then with_right_separator(...) — both
    // fields populated; render-time precedence resolves per section.
    let state = StatusBarState::with_separator(" · ")
        .with_right_separator(" ");

    // Global separator unchanged by the per-side override.
    assert_eq!(state.separator(), " · ");
    // Right-side override is set.
    assert_eq!(state.right_separator(), Some(" "));
    // Other per-side overrides remain None.
    assert_eq!(state.left_separator(), None);
    assert_eq!(state.center_separator(), None);
}

#[test]
fn test_per_side_separator_independent_setters() {
    use crate::component::StatusBarState;

    // Setting all three per-side overrides leaves the global unchanged;
    // each per-side field is independent of the others.
    let state = StatusBarState::with_separator(" · ")
        .with_left_separator(" | ")
        .with_center_separator(" :: ")
        .with_right_separator(" ");

    assert_eq!(state.left_separator(), Some(" | "));
    assert_eq!(state.center_separator(), Some(" :: "));
    assert_eq!(state.right_separator(), Some(" "));
    // Global separator unchanged.
    assert_eq!(state.separator(), " · ");
}
```

- [ ] **Step 2: Append 2 render-path snapshot tests to `snapshot_tests.rs`**

In `src/component/status_bar/snapshot_tests.rs`, append:

```rust
#[test]
fn snapshot_status_bar_per_side_separators() {
    use crate::component::status_bar::{StatusBar, StatusBarItem, StatusBarState};
    use crate::component::test_utils::setup_render;
    use crate::component::{Component, RenderContext};

    // RENDER-PATH PIN: distinct per-side separators visible in the rendered
    // ANSI output. Pins that the per-side fallback resolution at
    // mod.rs:850-852 correctly routes each section's separator through
    // render_section.
    //
    // leadline's actual transformation (the load-bearing motivation):
    // global " · " separator on left items, but " " (space) between
    // right-section slowdown segments. Pre-D12, the right section
    // inherited the global " · " — visually weird. Post-D12, the
    // right-section override lands.
    let state = StatusBarState::with_separator(" · ")
        .with_right_separator(" ")
        .push_left(StatusBarItem::new("L1"))
        .push_left(StatusBarItem::new("L2"))
        .push_right(StatusBarItem::new("R1"))
        .push_right(StatusBarItem::new("R2"));

    let (mut terminal, theme) = setup_render(40, 1);
    terminal
        .draw(|frame| {
            StatusBar::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme),
            );
        })
        .unwrap();

    let plain = terminal.backend().to_string();
    let _ansi = terminal.backend().to_ansi();

    // Left section uses global " · ": "L1 · L2" appears.
    assert!(
        plain.contains("L1 · L2"),
        "expected left section to use global ' · ' separator, got:\n{plain}",
    );

    // Right section uses override " ": "R1 R2" appears (single space
    // between, not " · ").
    assert!(
        plain.contains("R1 R2"),
        "expected right section to use override ' ' separator, got:\n{plain}",
    );

    // Right section MUST NOT contain " · " between its items.
    // (The global " · " is still allowed in left section content; we
    // search specifically for the right-side pattern.)
    assert!(
        !plain.contains("R1 · R2"),
        "right section should NOT use global ' · ' (override should win), got:\n{plain}",
    );

    insta::assert_snapshot!(plain);
}

#[test]
fn snapshot_status_bar_per_side_separators_byte_identical_when_unset() {
    use crate::component::status_bar::{StatusBar, StatusBarItem, StatusBarState};
    use crate::component::test_utils::setup_render;
    use crate::component::{Component, RenderContext};

    // REGRESSION PIN: when no per-side overrides are set, rendering is
    // byte-identical to pre-D12 behavior. Consumers who don't opt in to
    // per-side overrides see zero behavior change.
    //
    // Build a state using ONLY the global with_separator(...) (no per-side
    // calls). Render and snapshot. The snapshot should look identical to
    // the equivalent pre-D12 rendering — same separator between every item
    // in every section.
    let state = StatusBarState::with_separator(" · ")
        .push_left(StatusBarItem::new("L1"))
        .push_left(StatusBarItem::new("L2"))
        .push_right(StatusBarItem::new("R1"))
        .push_right(StatusBarItem::new("R2"));

    let (mut terminal, theme) = setup_render(40, 1);
    terminal
        .draw(|frame| {
            StatusBar::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme),
            );
        })
        .unwrap();

    let plain = terminal.backend().to_string();

    // Both sections use the global " · " separator (no per-side override
    // set, so fallback to global applies everywhere).
    assert!(
        plain.contains("L1 · L2"),
        "expected left section to use global ' · ' separator, got:\n{plain}",
    );
    assert!(
        plain.contains("R1 · R2"),
        "expected right section to also use global ' · ' separator (no override set), got:\n{plain}",
    );

    insta::assert_snapshot!(plain);
}
```

The `setup_render` + `Component::view` + `RenderContext::new` import pattern matches the existing snapshot test setup (verify via `grep -n "setup_render\|StatusBar::view" src/component/status_bar/snapshot_tests.rs | head -10`).

The `StatusBarState::with_separator(...).push_left(...)` chain assumes:
- `with_separator(...)` is a static constructor returning `Self` (verified at mod.rs:251)
- `push_left(StatusBarItem)` / `push_right(StatusBarItem)` are builder methods returning `&mut Self` or `Self` — need to verify the actual return type via `grep -n "pub fn push_left\|pub fn push_right" src/component/status_bar/mod.rs`. If they take `&mut self` and don't return self, adjust the construction to use let-mut-then-multiple-statements form.

If `push_left/push_right` are `&mut self` setters (don't chain), use this form instead:

```rust
let mut state = StatusBarState::with_separator(" · ").with_right_separator(" ");
state.push_left(StatusBarItem::new("L1"));
state.push_left(StatusBarItem::new("L2"));
state.push_right(StatusBarItem::new("R1"));
state.push_right(StatusBarItem::new("R2"));
```

Adjust per the actual API.

- [ ] **Step 3: Run the 8 new tests**

Run: `cargo nextest run --all-features status_bar:: 2>&1 | tail -10`
Expected: 6 new unit tests + 2 new snapshot tests PASS. Insta creates new `.snap` files on first run for the 2 snapshot tests.

If insta prompts to accept new snapshots: `cargo insta accept`.

Verify snapshot content briefly:
- `snapshot_status_bar_per_side_separators.snap`: shows `"L1 · L2"` and `"R1 R2"` in the rendered output
- `snapshot_status_bar_per_side_separators_byte_identical_when_unset.snap`: shows `"L1 · L2"` and `"R1 · R2"` (global separator everywhere)

- [ ] **Step 4: Run full test suite for no regressions**

Run: `cargo nextest run --all-features 2>&1 | tail -5`
Expected: full suite passes — 8 new tests + all existing.

- [ ] **Step 5: Commit Task 2**

```bash
git add src/component/status_bar/tests/state.rs src/component/status_bar/snapshot_tests.rs src/component/status_bar/snapshots/
git commit -S -m "Add 8 tests for StatusBar per-side separator overrides (D12)

Six unit tests in tests/state.rs:
- test_default_per_side_separators_none: defaults are all None
- test_with_left_separator_sets_field: left builder roundtrip
- test_with_center_separator_sets_field: center builder roundtrip
- test_with_right_separator_sets_field: right builder roundtrip
- test_with_separator_then_per_side_override: LAYERED SEMANTICS PIN —
  per-side override doesn't clear global; both fields populated
- test_per_side_separator_independent_setters: three per-side overrides
  don't interfere with each other or with the global

Two render-path tests in snapshot_tests.rs:
- snapshot_status_bar_per_side_separators: RENDER-PATH PIN — distinct
  per-side separators visible (left uses global ' · ', right uses
  override ' '). Pins leadline's actual transformation.
- snapshot_status_bar_per_side_separators_byte_identical_when_unset:
  REGRESSION PIN — consumers who don't opt in see zero behavior change."
```

---

## Task 3: Verify clippy + fmt + doc + audit + no-default-features

**Files:** (none — verification only)

- [ ] **Step 1: Run clippy**

Run: `cargo clippy --all-features --all-targets -- -D warnings 2>&1 | tail -10`
Expected: clean.

If warnings appear, fix in-place. Common ones to watch for: any unused imports if test code pulls in newly-unused items, or doc-list-without-indentation lints from `+`-prefixed continuation lines in docstrings (lesson from G6 PR #491).

- [ ] **Step 2: Run rustdoc with deny-warnings**

Run: `RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps 2>&1 | tail -10`
Expected: clean. All 6 new `# Example` doc tests compile and run.

- [ ] **Step 3: cargo fmt check**

Run: `cargo fmt --all -- --check 2>&1 | wc -l`
Expected: `0`.

If drift, run `cargo fmt --all` and commit:

```bash
git add -A
git commit -S -m "Apply cargo fmt"
```

- [ ] **Step 4: Full test suite**

```bash
cargo nextest run --all-features 2>&1 | tail -3
cargo test --all-features --doc 2>&1 | tail -3
```

Expected: all pass.

- [ ] **Step 5: cargo build --no-default-features**

Run: `cargo build --no-default-features 2>&1 | tail -3`
Expected: clean. Preventive check from D5+D14 lesson.

- [ ] **Step 6: Audit scorecard**

Run: `./tools/audit/target/release/envision-audit scorecard 2>&1 | grep "Result:"`
Expected: `Result: 8/9 checks passing` — same baseline as main.

**If audit drops to 7/9:** check doc-test coverage. The 6 new public methods (3 builders + 3 getters) must each have `# Example` doc tests per the audit requirement. Plan provides these explicitly in Task 1; verify none were dropped during implementation. Lesson from G4+G5 PR #487 + G6 PR #491.

- [ ] **Step 7: Commit only if fixes were needed**

If any verification step required a fix, commit it. Otherwise no commit — Task 3 is verification-only.

---

## Task 4: CHANGELOG entry

**Files:** Modify `CHANGELOG.md`

- [ ] **Step 1: Add the entry under `## [Unreleased]`**

Below the most recent sub-section (likely "`StyledInline` composable styles (G6)" from PR #491), add:

```markdown
### StatusBar per-side separator overrides (D12)

Adds three optional per-section separator overrides on `StatusBarState`.
Per-side override takes precedence over the existing global `separator` at
render time. Purely additive — default state has all three Options as
`None`, preserving identical behavior for consumers who don't opt in.

**New fields on `StatusBarState`:**

- `left_separator: Option<String>` (with `#[serde(default)]`)
- `center_separator: Option<String>` (with `#[serde(default)]`)
- `right_separator: Option<String>` (with `#[serde(default)]`)

Clustered immediately after the existing global `separator: String` field —
matches the G4 (`title_style` next to `title`) and G5 (`color`/`style_override`
next to `style`) field-grouping convention.

**New builder methods + getters:**

- `with_left_separator(impl Into<String>)` / `left_separator() -> Option<&str>`
- `with_center_separator(impl Into<String>)` / `center_separator() -> Option<&str>`
- `with_right_separator(impl Into<String>)` / `right_separator() -> Option<&str>`

Builder methods take `mut self` (consistent with existing `with_disabled`).
Chain cleanly with both `StatusBarState::new()` and `StatusBarState::with_separator(...)`:

```rust
// leadline's load-bearing use case: global " · " everywhere except
// the right-section slowdown segments, which use " " instead.
StatusBarState::with_separator(" · ")
    .with_right_separator(" ")
    .push_left(/* ... */)
    .push_right(/* ... */)
```

**Layered semantics, not last-call-wins:** per-side override is independent
of global `separator`. Setting `with_separator(" · ").with_right_separator(" ")`
results in BOTH fields populated; render-time precedence resolves which
applies per section (`state.right_separator.as_deref().unwrap_or(&state.separator)`).
Same model as G4/G5 per-component style overrides.

**Render-path:** three single-line fallback expressions inserted at
`status_bar/mod.rs:850-852`. `render_section` signature unchanged (it already
takes `&str`).

**Forward-compat:** `#[serde(default)]` on all three new fields means
pre-D12 serialized `StatusBarState` blobs deserialize cleanly with all three
as `None` — behavior identical to pre-D12.

**No struct-literal break** for external consumers — fields are private
(matches existing field visibility on the struct).
```

- [ ] **Step 2: Commit**

```bash
git add CHANGELOG.md
git commit -S -m "CHANGELOG: StatusBar per-side separator overrides (D12)

Document the three new with_left_separator / with_center_separator /
with_right_separator builders + matching getters, the render-time
precedence model (per-side > global), the #[serde(default)] forward-compat
treatment, and the zero-behavior-change-for-non-opt-in regression
guarantee."
```

---

## Task 5: Final verification + push + open PR

**Files:** (none — verification + git only)

- [ ] **Step 1: Verify all commits are signed**

Run: `git log --show-signature main..HEAD 2>&1 | grep -c 'Good signature'`
Expected: count matches the commits added on this branch (2 or 3 depending on whether Task 3 needed a fix commit; ~3-4 commits if a sibling-file split was needed in Task 1).

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

Expected: every command succeeds. Audit shows `Result: 8/9 checks passing`.

- [ ] **Step 3: Verify file sizes**

```bash
wc -l src/component/status_bar/*.rs
```

Expected: `mod.rs` under 1000 (target ~990). `tests/state.rs` and `snapshot_tests.rs` grown by the new test counts.

- [ ] **Step 4: Push the branch**

Run: `git push -u origin status-bar-per-side-separators-impl`

(Branch name is `status-bar-per-side-separators-impl` — the implementation branch, set by the controller before plan execution begins. The current plan branch is `status-bar-per-side-separators-plan`.)

Expected: pushes cleanly.

- [ ] **Step 5: Open the implementation PR**

Run:

```bash
gh pr create --title "StatusBar per-side separator overrides (D12)" --body "$(cat <<'EOF'
## Summary

Implementation of leadline gap **D12** — adds three per-section separator overrides on \`StatusBarState\`. Per-side override takes precedence over the existing global \`separator\` at render time. Purely additive; default behavior unchanged.

Spec: PR #493 (\`docs/superpowers/specs/2026-05-20-status-bar-per-side-separators-design.md\`)
Plan: PR β (\`docs/superpowers/plans/2026-05-20-status-bar-per-side-separators.md\`)

## What changed

**New fields on \`StatusBarState\`:**
- \`left_separator: Option<String>\`
- \`center_separator: Option<String>\`
- \`right_separator: Option<String>\`

All with \`#[serde(default)]\` for serialization forward-compat. Clustered next to the existing global \`separator\` field.

**Six new public methods:**
- \`with_left_separator(impl Into<String>)\`, \`with_center_separator(...)\`, \`with_right_separator(...)\` — true builders taking \`mut self\`, consistent with existing \`with_disabled\`
- \`left_separator(&self) -> Option<&str>\`, \`center_separator(...)\`, \`right_separator(...)\` — getters

**Render path:**
- 3-line per-side fallback inserted at \`status_bar/mod.rs:850-852\`
- \`render_section\` signature unchanged (already takes \`&str\`)

**Layered semantics, not last-call-wins:** per-side override independent of global \`separator\`. Same model as G4/G5.

## leadline use case

\`leadline/src/app.rs:278\` gains a new builder call on the chain:

\`\`\`rust
StatusBarState::with_separator(" · ")
    .with_right_separator(" ")  // <-- new
    .push_right(/* ... */)
\`\`\`

Right-section slowdown segments now use \`" "\` while the rest of the bar continues to use the global \`" · "\`.

## Stats

- 2-3 signed commits (1 atomic production + 1 tests + optional fmt cleanup)
- 8 new tests (6 unit + 2 render-path)
- 6 new public methods, each with \`# Example\` doc test
- File-size delta: \`mod.rs\` 919 → ~990 (under cap)

## Verification

- \`cargo build --all-features\`: clean
- \`cargo build --no-default-features\`: clean
- \`cargo clippy --all-features --all-targets -- -D warnings\`: clean
- \`cargo fmt --all -- --check\`: clean
- \`cargo nextest run --all-features\`: all passing
- \`cargo test --all-features --doc\`: clean
- \`RUSTDOCFLAGS=\"-D warnings\" cargo doc --all-features --no-deps\`: clean
- \`./tools/audit/target/release/envision-audit scorecard\`: 8/9 (baseline preserved)

## Test plan

- [ ] CI green on all platforms
- [ ] leadline migrates \`leadline/src/app.rs:278\` to add the \`.with_right_separator(\" \")\` call on the StatusBar construction chain
- [ ] Tracking-doc PR (next, parallel branch) marks D10 + D12 + D13 ✅ resolved in one coherent punch-list closure

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
| 3 new `Option<String>` fields with `#[serde(default)]` | Task 1 Step 1 |
| `Default` impl update | Task 1 Step 2 |
| 3 new builder methods (`with_left/center/right_separator`) | Task 1 Step 3 |
| 3 new getters (`left/center/right_separator() -> Option<&str>`) | Task 1 Step 3 |
| Render-path precedence resolution | Task 1 Step 4 |
| Field placement next to global `separator` (JC2) | Task 1 Step 1 |
| `with_center_separator` for symmetry (JC1) | Task 1 Step 3 |
| 6 builder/getter doc tests for audit-coverage | Task 1 Step 3 (each method has `# Example`) |
| 6 unit tests + 2 render-path tests | Task 2 |
| CHANGELOG entry | Task 4 |
| File-size mitigation (sibling-file fallback) | Task 1 Step 7 (conditional) |
| Tracking-doc PR bundles D10 + D12 + D13 closures | Separate tracking-doc PR after this plan ships (controller's responsibility per spec cadence) |

All spec requirements have a corresponding task. The tracking-doc PR (PR δ in the 4-PR cadence) is NOT part of this implementation plan — it's the parallel branch after PR γ merges.

### 2. Placeholder scan

No "TBD", "TODO", "implement later", "fill in details". Every step has either complete code, exact commands, or explicit verification criteria. The `push_left`/`push_right` API verification at Task 2 Step 2 is an explicit grep-and-adjust step, not a placeholder.

### 3. Type consistency

- `Option<String>` field shape consistent across struct + Default + builders + getters. ✅
- Builder method signatures consistent: `(mut self, separator: impl Into<String>) -> Self` for all three. ✅
- Getter signatures consistent: `(&self) -> Option<&str>` for all three. ✅
- Render-path expressions use the same `as_deref().unwrap_or(&state.separator)` pattern for all three sides. ✅
- Test field references use the verified field/method names (not typos like `left_sep` vs `left_separator`). ✅

---

## Plan complete

The plan covers 5 tasks producing approximately 2-3 signed commits (Task 1 + Task 2 + Task 4 produce commits; Task 3 verification-only; Task 5 push + PR; optional fmt or sibling-file-split commits). Estimated implementation time: 45-90 minutes of focused work (smallest cadence to date).

After plan PR β merges, controller creates `status-bar-per-side-separators-impl` branch from main and dispatches subagent-driven execution. After implementation PR γ merges, controller opens tracking-doc PR δ marking D10 + D12 + D13 ✅ resolved in `docs/customer-feedback/2026-05-01-leadline-gaps.md` per the spec's bundling preference.
