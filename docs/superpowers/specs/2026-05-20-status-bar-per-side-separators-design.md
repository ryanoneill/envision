# StatusBar per-side separator overrides (D12) — design spec

**Date:** 2026-05-20
**Status:** Approved; implementation plan to follow.
**Tracks:** leadline gap **D12** in `docs/customer-feedback/2026-05-01-leadline-gaps.md`
**Source brief:** `~/workspace/ryanoneill/rust-ai-explorations/notes/envision_small_rough_edges.md` (originally D10+D12+D13; scope shrunk to D12 alone after leadline re-verification confirmed D10 resolved-via-docs and D13 silently shipped as `App::on_exit`)

---

## TL;DR

`StatusBarState` has a single `separator: String` field applied globally across
all three sections (left, center, right). Consumers wanting a different
separator per side — leadline's right-section slowdown segments
(`+810.75 ms`, `28.57x slower`) want `" "` instead of the global `" · "` —
have to suppress separators per item via `StatusBarItem::with_separator(false)`,
which scales linearly with item count.

Add three new optional fields with matching builder methods:
- `left_separator: Option<String>` + `with_left_separator(impl Into<String>)`
- `center_separator: Option<String>` + `with_center_separator(impl Into<String>)`
- `right_separator: Option<String>` + `with_right_separator(impl Into<String>)`

Render-time precedence: per-side override wins when `Some`; falls back to the
existing global `separator` when `None`. Default state has all three Options as
`None` — behavior is unchanged for consumers who don't opt in.

Purely additive. All new fields get `#[serde(default)]` for serialization
forward-compat. No breaking changes; no migration. Same shape as G4
(`title_style: Option<Style>`) and G5 (`color: Option<Color>`,
`style_override: Option<Style>`) — optional override fields with render-time
precedence over a more general default.

This is the ninth coherent redesign from leadline's May 2026 brief suite and
the smallest by far — three builder methods, three getters, three render-path
fallback expressions. The "punch list, not a full design brief" framing
holds; the compressed brainstorm settled the design space in a single round
of pre-answers + two narrow envision-side picks.

---

## Goals

1. **Per-side separator override.** Consumers can set a distinct separator for
   the left, center, or right section without affecting the others. leadline's
   right-section slowdown segments use `" "` while the rest of the bar uses
   `" · "` — single call, no per-item suppression.
2. **Backward-compatible default.** Existing consumers who only call
   `with_separator(...)` (or use the default `" | "`) see no behavior change.
   The three new fields default to `None`; render falls back to the global
   `separator` for any section without an explicit override.
3. **Complete coverage of all three sections.** `with_left_separator`,
   `with_center_separator`, `with_right_separator` form a complete set —
   one per section. Skipping `center` would create an arbitrary asymmetry
   (the struct has three sections; the per-side surface should cover all
   three) and a future contributor would close the gap anyway.
4. **Builder ergonomics consistent with existing `with_disabled`.** The new
   methods take `mut self` and chain cleanly with both the
   `StatusBarState::with_separator(...)` constructor and the
   `StatusBarState::new()` constructor. Existing `with_separator(...)`
   constructor stays as-is (it's a static constructor today — refactoring it
   to a builder is out of D12's scope).
5. **Field placement clusters with `separator`.** New fields land immediately
   after the existing global `separator: String` field. Matches the
   field-grouping convention from G4 (`title_style` next to `title`) and G5
   (`color`/`style_override` next to `style`).

## Non-goals

- **Refactoring `with_separator(...)` from constructor to builder.** Currently
  `StatusBarState::with_separator(...)` is a static constructor (creates a
  fresh state with the given separator from `Self::default()`). The new
  per-side methods are true builders (take `mut self`). The inconsistency
  between `with_separator` (constructor) and `with_disabled` (builder) is
  pre-existing; D12 doesn't fix it. A future cleanup PR could harmonize.
- **Per-item separator override** (already exists as
  `StatusBarItem::with_separator(bool)` — boolean suppression only, not custom
  text). D12 is the per-section knob; the per-item knob stays as it is.
- **`with_separator_for(StatusBarSection, ...)` enum-keyed accessor.** Three
  named methods are more discoverable than one enum-keyed method. Mirrors the
  G4/G5 precedent (named methods, not enum-keyed accessor).
- **Removing the global `separator` field.** It's the default for any section
  without an explicit override. Removing it would force every consumer to
  set per-side separators explicitly — net negative ergonomics for the
  "uniform separator" case that's still common.
- **Per-side separator styling** (e.g., a different theme color for the right
  separator). Out of scope. Separator style today is hard-coded to
  `theme.disabled_style()` at `mod.rs:669`; consumers who want styled
  separators can layer via `StatusBarItem`-level styling. A future PR could
  add `with_separator_style(...)` if a consumer needs it.
- **`#[non_exhaustive]` on `StatusBarState`.** The struct is already public
  with public fields elsewhere (well — they're private actually); adding
  `#[non_exhaustive]` now would be a separate breaking-change decision out of
  D12's scope. New fields are private (matches existing field visibility).

---

## Design

### Field additions to `StatusBarState`

In `src/component/status_bar/mod.rs:182-195`:

```rust
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct StatusBarState {
    left: Vec<StatusBarItem>,
    center: Vec<StatusBarItem>,
    right: Vec<StatusBarItem>,
    /// Global separator used between items in any section that does
    /// not have a per-side override.
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
    background: Color,
    disabled: bool,
}
```

Field placement: the three new `*_separator: Option<String>` fields cluster
immediately after the global `separator: String` field. Matches G4
(`title_style` next to `title`) and G5 (`color`/`style_override` next to
`style`) precedent.

`#[serde(default)]` on each new field means existing serialized blobs without
these fields deserialize cleanly with `None`. Forward-compat preserved.

### `Default` impl + existing constructors update

The `Default` impl at `mod.rs:212-221` initializes all fields. Add
`left_separator: None, center_separator: None, right_separator: None,` to the
`Self { ... }` body:

```rust
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
```

The `with_separator(...)` static constructor at `mod.rs:251-256` uses
`..Self::default()` spread so it picks up the new field defaults automatically.
No change needed beyond verifying the spread is intact.

### Three new builder methods + three getters

Add to `impl StatusBarState` (location: near the existing `with_disabled`
builder method at line 515, to keep style-related builders clustered):

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
/// // leadline's use case: global " · " but " " on the right side.
/// let state = StatusBarState::with_separator(" · ")
///     .with_right_separator(" ");
/// assert_eq!(state.right_separator(), Some(" "));
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
/// let state = StatusBarState::new().with_center_separator(" :: ");
/// assert_eq!(state.center_separator(), Some(" :: "));
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
/// let state = StatusBarState::new().with_right_separator(" ");
/// assert_eq!(state.right_separator(), Some(" "));
/// ```
pub fn right_separator(&self) -> Option<&str> {
    self.right_separator.as_deref()
}
```

Six new public methods total: three builders + three getters. Each carries a
`# Example` doc test (lesson from G4+G5 and G6 audit-coverage regressions —
every new `pub fn` needs a doc test to preserve 100% audit coverage).

### Render-path precedence resolution

At `mod.rs:850-852` the current code passes the same `&state.separator` to all
three sections:

```rust
let left_spans = Self::render_section(&state.left, &state.separator, ctx.theme);
let center_spans = Self::render_section(&state.center, &state.separator, ctx.theme);
let right_spans = Self::render_section(&state.right, &state.separator, ctx.theme);
```

Replace with per-side fallback:

```rust
let left_sep = state.left_separator.as_deref().unwrap_or(&state.separator);
let center_sep = state.center_separator.as_deref().unwrap_or(&state.separator);
let right_sep = state.right_separator.as_deref().unwrap_or(&state.separator);
let left_spans = Self::render_section(&state.left, left_sep, ctx.theme);
let center_spans = Self::render_section(&state.center, center_sep, ctx.theme);
let right_spans = Self::render_section(&state.right, right_sep, ctx.theme);
```

Three single-line precedence resolutions inserted; render-section call shape
unchanged. The fallback uses `as_deref().unwrap_or(...)` which returns
`&str` (no allocation; references the existing `String` storage).

### Layered semantics: render-time, not setter-time

Same model as G5 (per-component style overrides): each per-side field is
independent of the global `separator`. Setting `with_separator(...)` then
`with_right_separator(...)` results in both fields populated:
`separator = " · "` and `right_separator = Some(" ")`. Render-time precedence
applies — right section uses `" "`, left and center use `" · "`.

The `with_separator(...)` constructor creating fresh state means chaining
`StatusBarState::new().with_left_separator(...)` and
`StatusBarState::with_separator(...).with_left_separator(...)` both work
naturally — the constructor returns `Self`, the builder takes `mut self`.

---

## Migration

### Consumer-side: leadline's `app.rs:278` collapse

Current shape:

```rust
StatusBarState::with_separator(" · ")
    .push_right(...)
    .push_right(...)  // these segments get " · " between them, want " " or none
```

Post-D12:

```rust
StatusBarState::with_separator(" · ")
    .with_right_separator(" ")  // right section uses " " now
    .push_right(...)
    .push_right(...)  // these segments get " " between them
```

One additional builder method on the chain. Left and center sections
continue to use the global `" · "` separator unchanged.

### Migration table

| Old | New |
|---|---|
| `StatusBarState::with_separator(" · ")` (all sections use " · ") | unchanged — still works; global default |
| Per-item suppression via `StatusBarItem::with_separator(false)` on every right-section item | `StatusBarState::with_separator(" · ").with_right_separator(" ")` — one-call replacement |
| (Future) center section with its own separator | `with_center_separator(...)` |

### Internal envision migration

envision's own components don't use `StatusBarState::with_separator(...)`
beyond test fixtures. No internal migration needed. Existing tests stay
unchanged (default `None` per-side means existing behavior preserved).

---

## Files to touch

| File | Change |
|---|---|
| `src/component/status_bar/mod.rs` | Add 3 fields with `#[serde(default)]` after global `separator`; update `Default` impl; add 3 builder methods + 3 getters near `with_disabled`; update render-path at lines 850-852 with per-side fallback |
| `src/component/status_bar/tests/state.rs` (or existing state test file) | Add 6 unit tests: 3 builder roundtrips, 3 default-None getter assertions, plus 2 render-path tests (snapshot showing distinct per-side separators + ANSI assertion of separator-string presence per section) |
| `CHANGELOG.md` | Additive entry under `[Unreleased]` |

File-size check: `status_bar/mod.rs` is at 919 lines today. Adding ~70 lines
for 3 fields + 6 methods + render-path tweak → ~990 lines. Just under the
1000-line cap; close enough to monitor but within bounds. If the entry crosses
during implementation, the methods can move to a sibling file (same pattern as
`pane_layout/title_style.rs` in G4).

---

## Tests

Eight new tests across two files. Each pins a distinct invariant.

### Unit tests in `src/component/status_bar/tests/state.rs`

1. **`test_default_per_side_separators_none`** — `StatusBarState::default()` has
   `left_separator() == None`, `center_separator() == None`,
   `right_separator() == None`. Pins the defaults.

2. **`test_with_left_separator_sets_field`** — `StatusBarState::new().with_left_separator(" | ")`
   produces a state with `left_separator() == Some(" | ")`. Center and right
   stay `None`. Round-trip.

3. **`test_with_center_separator_sets_field`** — symmetric to test 2 but for
   center.

4. **`test_with_right_separator_sets_field`** — symmetric to test 2 but for
   right.

5. **`test_with_separator_then_per_side_override`** — chain
   `StatusBarState::with_separator(" · ").with_right_separator(" ")`.
   Assert `separator() == " · "` (global unchanged) AND
   `right_separator() == Some(" ")` (override set). Pins the layered model:
   per-side override doesn't clear the global.

6. **`test_per_side_separator_independent_setters`** — setting all three
   per-side overrides leaves the global unchanged; each per-side field is
   independent. Pins that the three builders don't interfere with each other
   or with the global.

### Render-path tests in `src/component/status_bar/snapshot_tests.rs`

7. **`snapshot_status_bar_per_side_separators`** — render a state with
   global `" · "`, right-section override `" "`, and items in both left and
   right sections. Snapshot the rendered plain text. ANSI-assert that the
   left section contains the `" · "` separator string between its items;
   right section does NOT contain `" · "` between its items. Pins the
   render-path precedence resolution.

8. **`snapshot_status_bar_per_side_separators_byte_identical_when_unset`** —
   render a state with only the global `with_separator(" · ")` set (no
   per-side overrides). Snapshot byte-identical to the equivalent pre-D12
   snapshot. Pins zero-regression for consumers who don't opt in.

---

## Risks & open questions

### Risks

- **File-size cap proximity.** `status_bar/mod.rs` projected at ~990 lines
  post-D12. Just under cap. If insta snapshot tests in the file grow beyond
  expectations, mitigation: move the 3 builder methods + 3 getters into a
  sibling file `src/component/status_bar/per_side_separators.rs` (same pattern
  as `pane_layout/title_style.rs` from G4). Decision deferred to plan-writing
  based on actual line count.
- **Forward-compat for serialized blobs.** Three new `Option<String>` fields
  with `#[serde(default)]`. Pre-D12 serialized blobs deserialize cleanly with
  all three as `None` — behavior identical to pre-D12. No risk.
- **External pattern-matching on `StatusBarState`.** Unlikely (the struct
  fields are private); no risk to external `match` arms. CHANGELOG flags the
  additive shape.

### Decisions resolved during brainstorming

| Question | Resolution |
|---|---|
| JC1: Include `with_center_separator` for symmetry vs leave at left+right only | Include. Same precedent as G6 strikethrough helper and G5 complete-helper-set — per-axis surface earns its keep when complete across all axes (3 sections here). Trivial cost; future contributor closes the gap anyway. |
| JC2: Field placement — cluster next to global `separator` vs append at struct end | Cluster. Matches G4 (`title_style` next to `title`) and G5 (`color`/`style_override` next to `style`) field-grouping convention. Pure cosmetic but consistency with prior precedent. |
| Builder shape — true `mut self` builder vs static constructor like existing `with_separator` | True `mut self` builder. Consistent with the existing `with_disabled` builder. Chaining works cleanly with both `StatusBarState::new()` and `StatusBarState::with_separator(...)` because both return `Self`. |
| Storage — `Option<String>` per side | Yes. Clean separation between "not set" (use global) vs "explicit override". Natural Rust idiom. |
| `#[serde(default)]` on new fields | Yes. Preserves forward-compat for pre-D12 serialized data. |
| Refactor `with_separator(...)` from constructor to builder | No. Out of scope. Existing pattern stays as-is; D12 doesn't fix the pre-existing constructor/builder inconsistency between `with_separator` (constructor) and `with_disabled` (builder). |
| Enum-keyed accessor `with_separator_for(StatusBarSection, ...)` instead of three named methods | No. Three named methods more discoverable. Mirrors G4/G5 (named methods, not enum-keyed). |
| Per-side separator styling (e.g., different color for right separator) | Skip. Separator style is hard-coded to `theme.disabled_style()` today; a future PR can add `with_separator_style(...)` if needed. |
| `#[non_exhaustive]` on `StatusBarState` | Skip. Separate breaking-change decision; new fields are private (matches existing field visibility) so no struct-literal break externally. |

---

## Cadence

Same 4-PR cadence as the prior eight landings:

1. **PR α** — this design spec (`docs/superpowers/specs/2026-05-20-status-bar-per-side-separators-design.md`).
2. **PR β** — implementation plan (`docs/superpowers/plans/2026-05-20-status-bar-per-side-separators.md`).
3. **PR γ** — implementation. Purely additive. Single coherent PR adding 3
   fields + 6 methods + 3-line render-path tweak. 8 new tests.
4. **Tracking-doc PR** — mark D12 ✅ resolved in
   `docs/customer-feedback/2026-05-01-leadline-gaps.md`. **Bundled with**
   the closure markers for D10 (resolved-via-docs — `handle_event` vs
   `handle_event_with_state` ambiguity gone with current crystal-clear
   docstrings at `app/model/mod.rs:238-255`) and D13 (silently shipped as
   `App::on_exit(state: &Self::State)` default-no-op trait method at
   `src/app/model/mod.rs:257-260`, wired into both terminal and virtual
   runtimes). One coherent "punch-list closure" tracking PR per leadline's
   confirmed bundling preference.

Flag leadline at spec-PR open for review.

---

## Related context

- leadline's customer-feedback inventory: `docs/customer-feedback/2026-05-01-leadline-gaps.md` (D12 — also flags D10 + D13 as remaining; tracking PR closes all three)
- leadline-side gaps tracking: `~/workspace/ryanoneill/rust-ai-explorations/notes/envision_gaps.md` — D10 + D13 already marked RESOLVED in commit `2661d6e` on leadline's side
- Source brief: `~/workspace/ryanoneill/rust-ai-explorations/notes/envision_small_rough_edges.md` (originally D10+D12+D13; D12 alone after re-verification)
- Prior atomic-migration playbooks (8 shipped):
  - G1 + G3 + G7 (PRs #459 / #460 / #461 / #458)
  - D1 (PRs #463 / #464 / #465 / #466)
  - G2 + D2 + D11 (PRs #467 / #468 / #469 / #470)
  - D6 + D9 (PRs #471 / #472 / #473 / #474)
  - D15 (PRs #476 / #477 / #478 / #479)
  - D5 + D14 (PRs #480 / #481 / #482 / #483)
  - G4 + G5 (PRs #485 / #486 / #487 / #488)
  - G6 (PRs #489 / #490 / #491 / #492)
- leadline call sites this redesign simplifies:
  - `leadline/src/app.rs:278` — `StatusBarState::with_separator(" · ")` gains a `.with_right_separator(" ")` for the slowdown segments
- Related envision specs:
  - `docs/superpowers/specs/2026-05-19-per-component-style-overrides-design.md` — G4 + G5 layered per-component overrides with render-time precedence. Same model as D12's per-side fallback to global.

This is the ninth coherent redesign drawing from leadline's May 2026 brief
suite — and the smallest. After this, D7 (snapshot testing docs) + D3 + D8
(documentation suite: column docs + multi-view drill-down example) remain
as the last load-bearing items. The "punch list" framing (D10 + D12 + D13)
closes with this PR; the docs suite is a separate workstream.
