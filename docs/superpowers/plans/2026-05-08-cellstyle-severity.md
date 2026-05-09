# `CellStyle::Severity(Severity)` Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `CellStyle::Severity(Severity)` variant to the `Cell` type so severity-aware cells reach the active theme at render time, eliminating leadline's `Theme::catppuccin_mocha()` workaround in `severity_cell_style` helpers.

**Architecture:** One new variant, one render-path arm, one `#[non_exhaustive]` attribute, two `Cell` constructors (`Cell::severity` semantic shorthand + `Cell::with_severity` typed-cell builder). The renderer's already-in-scope `&Theme` resolves the variant via `theme.severity_style(*sev)` — the function shipped in PR #473. No `TableRow` trait churn; severity awareness lives in the `Cell` value.

**Tech Stack:** Rust 1.85+, ratatui 0.29 (`Color`, `Style`, `Modifier`), envision `theme` module (`Severity`, `Theme::severity_style`).

**Spec:** `docs/superpowers/specs/2026-05-08-cellstyle-severity-design.md` (PR #476, merged)

---

## Pre-execution gotchas

- **Signed commits required.** Project rule. If `git commit -S` fails, ask the user; never bypass with `--no-gpg-sign`.
- **Breaking change footprint.** Adding `#[non_exhaustive]` to `CellStyle` AND a new `Severity(Severity)` variant is a breaking change for any external code that pattern-matches `CellStyle` exhaustively without a `_` arm. Project rule (verified during brainstorming): no internal exhaustive matches outside `cell_style_to_ratatui`. CHANGELOG entry under `[Unreleased]` flags the break.
- **Atomic variant + render arm.** Adding `CellStyle::Severity(Severity)` without the render arm makes `cell_style_to_ratatui`'s `match` non-exhaustive (compile error). Task 2 lands the variant AND the render arm together in a single commit. Do not split — the build would be broken between commits.
- **`#[non_exhaustive]` semantics.** The attribute applies only to *external* consumers. Within the crate, `match` on `CellStyle` is still required to be exhaustive — the compiler enforces this. So adding `#[non_exhaustive]` (Task 1) is non-breaking internally; subsequent variant addition in Task 2 still requires updating the internal match.
- **Default theme severity collapse.** Documented in D6+D9 spec. On `Theme::default()`, `Severity::Mild` and `Severity::Bad` both render as `Color::Yellow` (`Peach` and `Yellow` collapse). Render-path tests run against the default theme (via `setup_render`) — assertions must accept the collapse, not flag it.
- **ANSI escape codes used in render-path tests.**
  - `\x1b[32m` — green foreground (Severity::Good)
  - `\x1b[33m` — yellow foreground (Severity::Mild AND Severity::Bad on default theme)
  - `\x1b[31m` — red foreground (Severity::Critical)
  - `\x1b[1m` — BOLD modifier (Severity::Critical only)
  - `\x1b[90m` — dark gray (disabled override)
  - `\x1b[0m` — reset
- **`cargo nextest` for unit tests.** Project uses `cargo-nextest` rather than `cargo test`. Doc tests run separately via `cargo test --all-features --doc`.
- **Audit baseline.** `./tools/audit/target/release/envision-audit scorecard` is currently 8/9 on main (the `resource_gauge::set_values` accessor symmetry gap is pre-existing, independent of this branch). No regression expected from this work.

---

## File Structure

| File | Responsibility | Lines (before → after) |
|---|---|---|
| `src/component/cell.rs` | `Cell` + `CellStyle` types, constructors, builders, inline tests | 651 → ~720 |
| `src/component/table/render.rs` | `cell_style_to_ratatui` + `render_table` | 188 → ~190 |
| `src/component/table/view_tests.rs` | Render-path snapshot + ANSI tests | 563 → ~640 |
| `CHANGELOG.md` | Release notes | adds ~25 lines under `[Unreleased]` |

All files stay well under the 1000-line cap.

---

## Build + test setup verification

Run once before Task 1 to confirm the environment is healthy:

```bash
cargo build --all-features 2>&1 | tail -5
cargo nextest run --all-features cell:: 2>&1 | tail -5
cargo nextest run --all-features table:: 2>&1 | tail -5
```

Expected: build succeeds; existing cell + table tests all pass.

---

## Task 1: Add `#[non_exhaustive]` to `CellStyle`

**Files:**
- Modify: `src/component/cell.rs:113`

This task lands `#[non_exhaustive]` first as an attribute-only change. No new variants yet. Internal `match` arms still need to be exhaustive (the attribute applies to external consumers only), so the build stays clean. Ships separately so the breaking-change footprint is visible per-commit.

- [ ] **Step 1: Add the `#[non_exhaustive]` attribute**

In `src/component/cell.rs`, modify the `CellStyle` enum declaration (around line 112-113) from:

```rust
/// Semantic cell styling.
///
/// `Default` renders with no override (theme-driven). `Success`,
/// `Warning`, `Error`, `Muted` map to the theme's semantic colors.
/// `Custom(Style)` applies a raw `ratatui::style::Style` directly.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum CellStyle {
```

to:

```rust
/// Semantic cell styling.
///
/// `Default` renders with no override (theme-driven). `Success`,
/// `Warning`, `Error`, `Muted` map to the theme's semantic colors.
/// `Custom(Style)` applies a raw `ratatui::style::Style` directly.
/// `Severity(Severity)` (added in PR γ) resolves the four-band severity
/// gradient through the active theme at render time.
///
/// `#[non_exhaustive]` so envision can add cell-style variants later without
/// breaking downstream `match` arms in consumer crates.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq)]
pub enum CellStyle {
```

(The `Severity(Severity)` mention in the docstring lands here so the docstring stays correct after Task 2 adds the variant.)

- [ ] **Step 2: Verify build is clean**

Run: `cargo build --all-features 2>&1 | tail -5`
Expected: clean build, no warnings.

Run: `cargo nextest run --all-features cell:: 2>&1 | tail -5`
Expected: all existing cell tests pass.

- [ ] **Step 3: Commit**

```bash
git add src/component/cell.rs
git commit -S -m "Mark CellStyle as #[non_exhaustive]

Forward-compat attribute lands separately from the Severity variant
(Task 2) so the breaking-change footprint is visible per-commit. The
attribute applies only to external consumers — internal match arms
still need to be exhaustive, so the build stays clean.

Matches the convention set by Severity and NamedColor in PR #473."
```

---

## Task 2: Add `CellStyle::Severity(Severity)` variant + render arm + render test

**Files:**
- Modify: `src/component/cell.rs` (add variant)
- Modify: `src/component/table/render.rs:19-31` (add match arm)
- Modify: `src/component/table/view_tests.rs` (add failing test first)

**Critical:** Variant + render arm must land together. Adding the variant alone breaks `cell_style_to_ratatui`'s exhaustiveness (compile error). This task bundles both into a single commit.

- [ ] **Step 1: Write the failing render test**

Append to `src/component/table/view_tests.rs`:

```rust
#[test]
fn snapshot_table_cells_with_severity_style() {
    use crate::component::cell::Cell;
    use crate::theme::Severity;

    // Render four severity bands in one row. Default theme: Good=Green,
    // Mild=Yellow, Bad=Yellow (collapses with Mild on Default — documented
    // behavior from D6+D9), Critical=Red+BOLD.
    let columns = vec![
        Column::new("G", Constraint::Length(8)),
        Column::new("M", Constraint::Length(8)),
        Column::new("B", Constraint::Length(8)),
        Column::new("C", Constraint::Length(8)),
    ];
    let cells = vec![
        Cell::severity("good", Severity::Good),
        Cell::severity("mild", Severity::Mild),
        Cell::severity("bad", Severity::Bad),
        Cell::severity("crit", Severity::Critical),
    ];
    let mut state = TableState::new(vec![StyledRow { cells }], columns);
    state.set_selected(None);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Table::<StyledRow>::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    let plain = terminal.backend().to_string();
    let ansi = terminal.backend().to_ansi();

    // ANSI assertions pin per-band coloring. Default theme: Mild and Bad
    // both render as Color::Yellow (\x1b[33m) — the documented collapse
    // from D6+D9 means severity bands degrade from four to three on
    // Default. Critical stays distinguishable via BOLD.
    assert!(
        ansi.contains("\x1b[32m"),
        "expected green (32m) for Severity::Good, got:\n{ansi}",
    );
    assert!(
        ansi.contains("\x1b[33m"),
        "expected yellow (33m) for Severity::Mild and Severity::Bad, got:\n{ansi}",
    );
    assert!(
        ansi.contains("\x1b[31m"),
        "expected red (31m) for Severity::Critical, got:\n{ansi}",
    );
    assert!(
        ansi.contains("\x1b[1m"),
        "expected BOLD (1m) for Severity::Critical, got:\n{ansi}",
    );

    insta::assert_snapshot!(plain);
}
```

- [ ] **Step 2: Run test to verify it fails (compile error expected)**

Run: `cargo nextest run --all-features table::view_tests::snapshot_table_cells_with_severity_style 2>&1 | tail -15`
Expected: COMPILE FAIL — `Cell::severity` doesn't exist (Task 4) and `CellStyle::Severity` doesn't exist yet.

(The test references `Cell::severity` which is added in Task 4. The build fails earlier than the test run. That's fine — the failing-mode is "test code doesn't compile because production code is missing" which is valid TDD. The test passes once Tasks 2 and 4 both land.)

**To advance Task 2 without Task 4 yet:** temporarily replace `Cell::severity("text", sev)` calls in the test body with the equivalent explicit form using `with_style`:

```rust
// Temporary form (used until Task 4 lands the Cell::severity constructor):
Cell::new("good").with_style(CellStyle::Severity(Severity::Good)),
```

This lets Task 2 stand on its own. Task 4 will swap the call sites back to `Cell::severity(...)`. The render-path test exercises the same code path either way.

Use `Cell::new(...).with_style(CellStyle::Severity(...))` form for now. Re-run:

Run: `cargo nextest run --all-features table::view_tests::snapshot_table_cells_with_severity_style 2>&1 | tail -15`
Expected: COMPILE FAIL — `CellStyle::Severity` doesn't exist.

- [ ] **Step 3: Add `Severity(Severity)` variant to `CellStyle` in `src/component/cell.rs`**

In `src/component/cell.rs`, find the `CellStyle` enum (around line 113-127). Append `Severity(Severity)` as the last variant. Add `use` import for `Severity` at the top of the file if not already present.

First, check for existing import: `grep -n "use crate::theme::\|use super::Severity\|theme::Severity" src/component/cell.rs`. If `Severity` is not already imported, add to the existing `use` block at the top of the file:

```rust
use crate::theme::Severity;
```

Then modify the enum body (after `Custom(Style),`):

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
    /// See [`crate::theme::Theme::severity_style`].
    Severity(Severity),
}
```

- [ ] **Step 4: Add render arm in `src/component/table/render.rs`**

In `src/component/table/render.rs` (line 19-31), modify `cell_style_to_ratatui` to add the new arm:

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

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo nextest run --all-features table::view_tests::snapshot_table_cells_with_severity_style 2>&1 | tail -15`
Expected: PASS. The new snapshot file is created on first run; verify the snapshot content shows the four cells in plain text.

If insta requests review of the new snapshot, accept it: `cargo insta accept`.

- [ ] **Step 6: Run full test suite for no regressions**

```bash
cargo nextest run --all-features 2>&1 | tail -5
cargo test --all-features --doc 2>&1 | tail -5
```

Expected: all pass.

- [ ] **Step 7: Commit (variant + render arm together)**

```bash
git add src/component/cell.rs src/component/table/render.rs src/component/table/view_tests.rs src/component/table/snapshots/
git commit -S -m "Add CellStyle::Severity(Severity) variant + render arm

Variant resolves at render time via theme.severity_style(*sev), routing
through the theme's palette. Color-only for Good/Mild/Bad; adds
Modifier::BOLD for Critical. Render-path test pins per-band ANSI
output on the default theme — including the documented Mild/Bad collapse
to Color::Yellow (D6+D9 caveat).

Variant + render arm land together to keep cell_style_to_ratatui's
match exhaustive across the commit boundary."
```

---

## Task 3: Add disabled-override render test

**Files:**
- Modify: `src/component/table/view_tests.rs`

This task adds a render-path test asserting that the existing `disabled` override in `cell_style_to_ratatui` still wins for `CellStyle::Severity(...)`. No production code change — just pinning the existing behavior so the contract is captured.

- [ ] **Step 1: Write the test**

Append to `src/component/table/view_tests.rs`:

```rust
#[test]
fn snapshot_table_cells_severity_disabled_renders_dark_gray_no_bold() {
    use crate::component::cell::{Cell, CellStyle};
    use crate::theme::Severity;

    // Same row as snapshot_table_cells_with_severity_style, but the
    // RenderContext is marked disabled. Disabled override wins: every
    // cell renders dark-gray (\x1b[90m), no BOLD.
    let columns = vec![
        Column::new("G", Constraint::Length(8)),
        Column::new("M", Constraint::Length(8)),
        Column::new("B", Constraint::Length(8)),
        Column::new("C", Constraint::Length(8)),
    ];
    let cells = vec![
        Cell::new("good").with_style(CellStyle::Severity(Severity::Good)),
        Cell::new("mild").with_style(CellStyle::Severity(Severity::Mild)),
        Cell::new("bad").with_style(CellStyle::Severity(Severity::Bad)),
        Cell::new("crit").with_style(CellStyle::Severity(Severity::Critical)),
    ];
    let mut state = TableState::new(vec![StyledRow { cells }], columns);
    state.set_selected(None);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Table::<StyledRow>::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).disabled(true),
            );
        })
        .unwrap();

    let plain = terminal.backend().to_string();
    let ansi = terminal.backend().to_ansi();

    // Disabled override: all cells render as dark-gray. No severity-band
    // colors should appear.
    assert!(
        ansi.contains("\x1b[90m"),
        "expected dark-gray (90m) for all cells under disabled, got:\n{ansi}",
    );
    assert!(
        !ansi.contains("\x1b[32m"),
        "did not expect green (32m) under disabled — Severity::Good should collapse to dark-gray, got:\n{ansi}",
    );
    assert!(
        !ansi.contains("\x1b[31m"),
        "did not expect red (31m) under disabled — Severity::Critical should collapse to dark-gray, got:\n{ansi}",
    );
    assert!(
        !ansi.contains("\x1b[1m"),
        "did not expect BOLD (1m) under disabled — Severity::Critical's BOLD must drop, got:\n{ansi}",
    );

    insta::assert_snapshot!(plain);
}
```

- [ ] **Step 2: Run test to verify it passes**

Run: `cargo nextest run --all-features table::view_tests::snapshot_table_cells_severity_disabled 2>&1 | tail -15`
Expected: PASS. Snapshot created on first run; accept with `cargo insta accept`.

- [ ] **Step 3: Commit**

```bash
git add src/component/table/view_tests.rs src/component/table/snapshots/
git commit -S -m "Test: disabled override wins for CellStyle::Severity

No production code change — pins the existing cell_style_to_ratatui
disabled-first short-circuit for the new Severity variant. All four
severity bands collapse to dark-gray under disabled; Critical's BOLD
modifier drops. Inverse-assertion form catches accidental regressions
where disabled stops winning."
```

---

## Task 4: Add `Cell::severity(text, sev)` constructor

**Files:**
- Modify: `src/component/cell.rs:280-298` (constructor block)
- Modify: `src/component/cell.rs:600-625` (`cell_style_constructor_tests` module)
- Modify: `src/component/table/view_tests.rs` (swap back to `Cell::severity`)

- [ ] **Step 1: Write the failing test**

In `src/component/cell.rs`, find the `cell_style_constructor_tests` module (line ~600). Append a new test:

```rust
    #[test]
    fn severity_sets_style() {
        use crate::theme::Severity;
        let c = Cell::severity("crash", Severity::Critical);
        assert_eq!(c.text(), "crash");
        assert_eq!(*c.style(), CellStyle::Severity(Severity::Critical));
        assert_eq!(c.sort_key(), None);
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo nextest run --all-features cell::cell_style_constructor_tests::severity_sets_style 2>&1 | tail -10`
Expected: COMPILE FAIL — `Cell::severity` not found.

- [ ] **Step 3: Add `Cell::severity` constructor**

In `src/component/cell.rs`, find the existing semantic constructors (line ~280-298). Append `Cell::severity` after `Cell::muted`:

```rust
    /// Muted-styled cell (theme dark gray).
    pub fn muted(text: impl Into<CompactString>) -> Self {
        Self::new(text).with_style(CellStyle::Muted)
    }

    /// Severity-styled cell. Resolves color through the active theme at
    /// render time via [`crate::theme::Theme::severity_style`].
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
        Self::new(text).with_style(CellStyle::Severity(sev))
    }
}
```

(Replaces the closing `}` of `impl Cell` with the `severity` method appended before it.)

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo nextest run --all-features cell::cell_style_constructor_tests::severity_sets_style 2>&1 | tail -10`
Expected: PASS.

Run: `cargo test --all-features --doc cell::Cell::severity 2>&1 | tail -10`
Expected: doc test PASS.

- [ ] **Step 5: Swap render-test back to `Cell::severity` form**

In `src/component/table/view_tests.rs`, find the `snapshot_table_cells_with_severity_style` test (added in Task 2). Change the four `Cell::new("...").with_style(CellStyle::Severity(...))` calls back to the `Cell::severity(...)` shorthand the test was originally written with:

```rust
    let cells = vec![
        Cell::severity("good", Severity::Good),
        Cell::severity("mild", Severity::Mild),
        Cell::severity("bad", Severity::Bad),
        Cell::severity("crit", Severity::Critical),
    ];
```

(The disabled-override test in Task 3 stays on the explicit `Cell::new("...").with_style(...)` form deliberately — it tests the more verbose path so both shapes have render coverage.)

Re-run the test: `cargo nextest run --all-features table::view_tests::snapshot_table_cells_with_severity_style 2>&1 | tail -10`
Expected: PASS. Snapshot unchanged (the constructor is just sugar for the explicit form).

- [ ] **Step 6: Commit**

```bash
git add src/component/cell.rs src/component/table/view_tests.rs
git commit -S -m "Add Cell::severity(text, sev) constructor

Semantic shorthand mirroring Cell::success/warning/error/muted family.
Equivalent to Cell::new(text).with_style(CellStyle::Severity(sev)).
Renderer resolves through the active theme at render time. Doc test
demonstrates the simple-case usage."
```

---

## Task 5: Add `Cell::with_severity(sev)` builder + 2 tests

**Files:**
- Modify: `src/component/cell.rs:189-200` (builder block) and `:600-625` (test module)

- [ ] **Step 1: Write the two failing tests**

In `src/component/cell.rs`, find the `cell_struct_tests` module (line ~496). Append:

```rust
    #[test]
    fn with_severity_preserves_text_and_sort_key() {
        use crate::theme::Severity;

        let c = Cell::number(5.2)
            .with_text(format!("{:.2}x", 5.2))
            .with_severity(Severity::Bad);
        assert_eq!(c.text(), "5.20x");
        assert_eq!(c.sort_key(), Some(&SortKey::F64(5.2)));
        assert_eq!(*c.style(), CellStyle::Severity(Severity::Bad));
    }

    #[test]
    fn with_severity_overwrites_with_style() {
        use crate::theme::Severity;
        use ratatui::style::{Color, Style};

        // Last-call-wins: with_severity drops the prior with_style(Custom(...)).
        let c = Cell::new("x")
            .with_style(CellStyle::Custom(Style::default().fg(Color::Magenta)))
            .with_severity(Severity::Critical);
        assert_eq!(*c.style(), CellStyle::Severity(Severity::Critical));
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo nextest run --all-features cell::cell_struct_tests::with_severity 2>&1 | tail -10`
Expected: 2 COMPILE FAILs — `with_severity` not found.

- [ ] **Step 3: Add `Cell::with_severity` builder**

In `src/component/cell.rs`, find the existing builder methods (line ~189-212: `with_style`, `with_sort_key`, `with_text`). Add `with_severity` next to them. Suggested location: immediately after `with_style` so the related setters cluster:

```rust
    /// Builder: set the cell style.
    pub fn with_style(mut self, style: CellStyle) -> Self {
        self.style = style;
        self
    }

    /// Builder: set the cell style to severity-styled.
    ///
    /// Composes with the typed-cell pattern from G7 (preserves a typed
    /// [`SortKey`]):
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
    /// `CellStyle::Severity(Bad)`; the prior `Custom` is dropped. Natural
    /// builder-pattern semantics — each setter overwrites.
    pub fn with_severity(mut self, sev: Severity) -> Self {
        self.style = CellStyle::Severity(sev);
        self
    }

    /// Builder: set the typed sort key.
    pub fn with_sort_key(mut self, key: SortKey) -> Self {
        self.sort_key = Some(key);
        self
    }
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo nextest run --all-features cell::cell_struct_tests::with_severity 2>&1 | tail -10`
Expected: 2 PASS.

Run: `cargo test --all-features --doc cell::Cell::with_severity 2>&1 | tail -10`
Expected: doc test PASS.

- [ ] **Step 5: Run full test suite**

```bash
cargo nextest run --all-features 2>&1 | tail -5
cargo test --all-features --doc 2>&1 | tail -5
```

Expected: all pass — both new tests + all existing.

- [ ] **Step 6: Commit**

```bash
git add src/component/cell.rs
git commit -S -m "Add Cell::with_severity(sev) builder

Typed-cell builder for the G7-style chain
Cell::number(x).with_text(formatted).with_severity(sev) — preserves
typed SortKey while layering severity color on top. Two tests pin
the chain (sort_key + text round-trip) and the last-call-wins
precedence with prior with_style(Custom(...)). Docstring documents
the precedence explicitly per leadline's plan-time review."
```

---

## Task 6: Verify clippy + fmt + doc clean

**Files:**
- (none — verification only)

- [ ] **Step 1: Run clippy**

Run: `cargo clippy --all-features --all-targets -- -D warnings 2>&1 | tail -10`
Expected: clean.

If warnings appear, fix in-place. The most likely warning would be on the `Severity` import in `cell.rs` if it's only used in `match` — verify it's actually used.

- [ ] **Step 2: Run rustdoc with deny-warnings**

Run: `RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps 2>&1 | tail -10`
Expected: clean. The new doc tests on `Cell::severity` and `Cell::with_severity` should compile + run.

- [ ] **Step 3: Run cargo fmt check**

Run: `cargo fmt --all -- --check 2>&1 | wc -l`
Expected: `0` (zero lines of output = clean).

If formatting drift, run `cargo fmt --all` and commit:

```bash
git add -A
git commit -S -m "Apply cargo fmt"
```

- [ ] **Step 4: Run audit scorecard**

Run: `./tools/audit/target/release/envision-audit scorecard 2>&1 | tail -15`
Expected: 8/9 passing — same baseline as main. No regression. The pre-existing `resource_gauge::set_values has no matching getter` gap remains the one failure.

If the scorecard shows a NEW failure (anything beyond the resource_gauge gap), investigate before proceeding.

- [ ] **Step 5: Commit (only if any fixes were needed)**

If a fix was committed in this task, the commit lands here. Otherwise no commit — Task 6 is verification-only.

---

## Task 7: CHANGELOG entry

**Files:**
- Modify: `CHANGELOG.md`

- [ ] **Step 1: Add the entry under `## [Unreleased]`**

Open `CHANGELOG.md`. Find the `## [Unreleased]` section. Below the most recent existing sub-section (likely "Theme palette + severity helper (D6 + D9)" from PR #473), add:

```markdown
### CellStyle::Severity(Severity) (D15)

`Cell` gains a new severity-styled variant resolved at render time, closing
the loop on the D6+D9 theme palette + severity helper. Eliminates the need
for consumers to construct `Theme::catppuccin_mocha()` inline at row-build
time to access `theme.severity_style(sev)`.

**New variants:**

- `CellStyle::Severity(Severity)` — resolves to `theme.severity_style(*sev)`
  at render time. Color routes through the theme's palette (Good→Green,
  Mild→Yellow, Bad→Peach, Critical→Red); `Critical` adds `BOLD`.

**New `Cell` constructors:**

- `Cell::severity(text, sev)` — semantic shorthand mirroring
  `Cell::success/warning/error/muted`.
- `Cell::with_severity(sev)` — typed-cell builder for the G7 chain
  `Cell::number(x).with_text(formatted).with_severity(sev)`. Preserves the
  typed `SortKey` while layering severity color. Last-call-wins precedence
  with `with_style(...)`.

**Breaking change:**

- `CellStyle` is now `#[non_exhaustive]`. External code that pattern-matches
  `CellStyle` exhaustively must add a `_` arm. Matches the convention set
  by `Severity` and `NamedColor` in the prior release. Internal `match` arms
  inside the crate are still exhaustive — the attribute applies only to
  external consumers.

**Migration for severity-aware cells:** drop any `severity_cell_style`-style
helper that constructs a hardcoded theme. Replace with
`Severity::from_thresholds(...)` + `CellStyle::Severity(sev)` (or
`Cell::severity(text, sev)` / `.with_severity(sev)` shortcuts). Theme-swap
now works correctly.
```

- [ ] **Step 2: Commit**

```bash
git add CHANGELOG.md
git commit -S -m "CHANGELOG: CellStyle::Severity(Severity) (D15)

Document the additive Cell API extensions: CellStyle::Severity variant
(resolves at render time), Cell::severity / Cell::with_severity
constructors, and the #[non_exhaustive] breaking change on CellStyle."
```

---

## Task 8: Final verification + push + open PR

**Files:**
- (none — verification + git only)

- [ ] **Step 1: Verify all commits are signed**

Run: `git log --show-signature main..HEAD 2>&1 | grep -c 'Good signature'`
Expected: count matches the number of commits added on this branch (6 or 7 depending on whether Task 6 committed).

If any commit is unsigned, **stop** and ask the user how to handle it — never bypass.

- [ ] **Step 2: Run the full verification gauntlet in parallel**

```bash
cargo build --all-features
cargo clippy --all-features --all-targets -- -D warnings
cargo fmt --all -- --check
cargo nextest run --all-features
cargo test --all-features --doc
RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps
./tools/audit/target/release/envision-audit scorecard
```

Expected: every command succeeds. Audit shows 8/9 (same as main baseline).

- [ ] **Step 3: Push the branch**

Run: `git push -u origin cellstyle-severity-impl`

(Branch name is `cellstyle-severity-impl` — the implementation branch, set by the controller before plan execution begins. The current plan branch is `cellstyle-severity-plan`.)

Expected: pushes cleanly.

- [ ] **Step 4: Open the implementation PR**

Run:

```bash
gh pr create --title "CellStyle::Severity(Severity) (D15)" --body "$(cat <<'EOF'
## Summary

Implementation of leadline gap **D15** — \`TableRow::cells(&self)\` takes no \`&Theme\`; severity-aware cell construction can't reach the active theme at row-build time.

Spec: PR #476 (\`docs/superpowers/specs/2026-05-08-cellstyle-severity-design.md\`)
Plan: PR β (\`docs/superpowers/plans/2026-05-08-cellstyle-severity.md\`)

## What changed

**New variant:**
- \`CellStyle::Severity(Severity)\` — resolves at render time via \`theme.severity_style(*sev)\`. Routes through the theme's palette; Critical adds BOLD.

**New \`Cell\` constructors:**
- \`Cell::severity(text, sev)\` — semantic shorthand
- \`Cell::with_severity(sev)\` — typed-cell builder; preserves G7 \`SortKey\`; last-call-wins precedence with \`with_style\`

**Breaking change:**
- \`CellStyle\` marked \`#[non_exhaustive]\` — matches \`Severity\` / \`NamedColor\` precedent from PR #473. Bundled in same PR for one-breakage-not-two reasons.

**Render arm:**
- \`cell_style_to_ratatui\` gets one new arm for the variant. No new render-path plumbing.

## Test plan

- [ ] CI green on all platforms
- [ ] Audit scorecard 8/9 (baseline; the resource_gauge::set_values gap is pre-existing)
- [ ] leadline migrates the two \`severity_cell_style\` helpers; deletes inline \`Theme::catppuccin_mocha()\` constructions; replaces with \`Severity::from_thresholds(...)\` + \`with_severity(sev)\` chains

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

Expected: PR URL printed.

---

## Self-review (controller runs before dispatch)

Run this checklist after writing all tasks; the implementer runs verification at Tasks 6 and 8.

### 1. Spec coverage

| Spec section | Tasks |
|---|---|
| `CellStyle::Severity(Severity)` variant | Task 2 |
| `#[non_exhaustive]` on `CellStyle` | Task 1 |
| `cell_style_to_ratatui` render arm | Task 2 |
| `Cell::severity(text, sev)` constructor | Task 4 |
| `Cell::with_severity(sev)` builder | Task 5 |
| Last-call-wins precedence with `with_style` (docstring + test) | Task 5 |
| Disabled override preserved | Task 3 |
| 5 named tests from spec | Tasks 2 (test #4 — combined snapshot+ANSI per existing convention), 3 (test #5), 4 (test #1), 5 (tests #2 + #3) |
| CHANGELOG entry | Task 7 |
| Breaking-change footprint documented | Task 7 (CHANGELOG) + Task 1 (commit message) |

All spec requirements have a corresponding task.

### 2. Placeholder scan

No "TBD", "TODO", "fill in", "implement later". Every step has either complete code, exact commands, or explicit verification criteria.

### 3. Type consistency

- `Severity` enum used consistently: `Severity::Good`, `Severity::Mild`, `Severity::Bad`, `Severity::Critical` (Tasks 2, 3, 4, 5). ✅
- `CellStyle::Severity(Severity)` shape consistent across all references. ✅
- Method signatures match the spec: `Cell::severity(text: impl Into<CompactString>, sev: Severity) -> Self`, `Cell::with_severity(self, sev: Severity) -> Self`. ✅
- ANSI escape codes used consistently: `\x1b[32m`, `\x1b[33m`, `\x1b[31m`, `\x1b[1m`, `\x1b[90m`. ✅

---

## Plan complete

The plan covers 8 tasks producing approximately 7 signed commits (Task 6 is verification-only and may not commit). Estimated implementation time: 2-3 hours of focused work.

After plan PR β merges, controller creates `cellstyle-severity-impl` branch from main and dispatches subagent-driven execution. After implementation PR γ merges, controller opens tracking-doc PR δ marking D15 ✅ resolved in `docs/customer-feedback/2026-05-01-leadline-gaps.md`.
