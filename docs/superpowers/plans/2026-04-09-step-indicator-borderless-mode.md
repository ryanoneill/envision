# StepIndicator Borderless Mode Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an opt-in borderless rendering mode to `StepIndicator` so it can be used as an inline breadcrumb (single row, no box around the steps).

**Architecture:** Add a `show_border: bool` field to `StepIndicatorState` (default `true` for backwards compatibility) with matching builder (`with_show_border`), getter (`show_border`), and mutator (`set_show_border`) — mirroring the existing `StyledTextState::show_border` convention. The `view()` function branches on the field: when `true`, it constructs a `Block` with borders and optional title as before and renders into the block's inner area; when `false`, it skips the block entirely and renders steps directly into the full widget area. When borderless, the state's title is silently suppressed (it only lives in the block), matching the same behavior in `StyledTextState`.

**Tech Stack:** Rust (edition 2024), ratatui (terminal rendering), insta (snapshot tests), cargo-nextest (test runner).

**Spec:** `docs/superpowers/specs/2026-04-09-step-indicator-border-toggle-design.md`

---

## Project Context

- **Working branch:** `step-indicator-borderless-mode` (already created, spec already committed).
- **Files touched:**
  - Modify: `src/component/step_indicator/mod.rs` — add field, methods, update `view()`
  - Modify: `src/component/step_indicator/tests.rs` — add state tests + snapshot tests
  - Modify: `CHANGELOG.md` — add unreleased entry
- **Test runner:** Use `cargo nextest run` for unit tests (per project conventions); use `cargo test --doc` for doc tests separately. Plain `cargo test` also works but is slower.
- **Commits:** All commits must be signed (`git commit -S`). The git config on this machine already has `commit.gpgsign=true`, so the `-S` flag is redundant but explicit-is-fine.
- **No warnings allowed** — the project CLAUDE.md rule. Run `cargo clippy -- -D warnings` before finalizing.
- **Format before committing** — run `cargo fmt` and expect no diffs.
- **Existing snapshots** under `src/component/step_indicator/snapshots/` cover the with-border path and act as regression coverage. If any of them change after the refactor in Task 2, that's a bug — the refactor must be behaviorally identical for `show_border: true`.

---

## Task 1: Add `show_border` state field, accessors, and unit tests (TDD)

**Files:**
- Modify: `src/component/step_indicator/mod.rs`
  - `StepIndicatorState` struct at lines 217-224 — add field
  - `Default for StepIndicatorState` at lines 226-237 — initialize field
  - Insert three new methods after `with_show_descriptions` at line 318 (builder methods) and after `set_orientation` at line 444 (setter — keep setters grouped)
- Modify: `src/component/step_indicator/tests.rs`
  - Insert new state tests after `test_state_with_show_descriptions` at line 93
  - Update `test_state_new` at lines 51-61 to assert the new default

---

- [ ] **Step 1.1: Write the failing unit tests**

Open `src/component/step_indicator/tests.rs`. After the existing `test_state_with_show_descriptions` test (ends at line 93), add three new tests. Also update `test_state_new` to assert the new default.

Update `test_state_new` at lines 51-61. Replace with:

```rust
#[test]
fn test_state_new() {
    let steps = vec![Step::new("A"), Step::new("B"), Step::new("C")];
    let state = StepIndicatorState::new(steps);
    assert_eq!(state.steps().len(), 3);
    assert_eq!(state.focused_index(), 0);
    assert_eq!(state.orientation(), &StepOrientation::Horizontal);
    assert_eq!(state.connector(), "───");
    assert_eq!(state.title(), None);
    assert!(!state.show_descriptions());
    assert!(state.show_border());
}
```

Add these three new tests after line 93 (after the closing `}` of `test_state_with_show_descriptions`), keeping them in the "State Creation Tests" section:

```rust
#[test]
fn test_state_default_show_border() {
    let state = StepIndicatorState::default();
    assert!(
        state.show_border(),
        "show_border must default to true for backwards compatibility",
    );
}

#[test]
fn test_state_with_show_border() {
    let state = StepIndicatorState::new(vec![Step::new("A")]).with_show_border(false);
    assert!(!state.show_border());

    // Chaining with other builders works and does not interfere.
    let state = StepIndicatorState::new(vec![Step::new("A")])
        .with_title("Pipeline")
        .with_show_border(false);
    assert!(!state.show_border());
    // Title is still stored on the state; only rendering is suppressed.
    assert_eq!(state.title(), Some("Pipeline"));
}

#[test]
fn test_state_set_show_border() {
    let mut state = StepIndicatorState::new(vec![Step::new("A")]);
    assert!(state.show_border());
    state.set_show_border(false);
    assert!(!state.show_border());
    state.set_show_border(true);
    assert!(state.show_border());
}
```

- [ ] **Step 1.2: Run the tests and verify they fail**

Run:

```bash
cargo nextest run -p envision step_indicator::tests::test_state_default_show_border step_indicator::tests::test_state_with_show_border step_indicator::tests::test_state_set_show_border step_indicator::tests::test_state_new
```

Expected: all four tests fail to compile with errors like `no method named \`show_border\` found for struct \`StepIndicatorState\`` and `no method named \`with_show_border\` found...` and `no method named \`set_show_border\` found...`. Compile error is an acceptable "red" state for TDD.

- [ ] **Step 1.3: Add the field to `StepIndicatorState`**

Open `src/component/step_indicator/mod.rs`. Modify the struct definition at lines 217-224:

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct StepIndicatorState {
    steps: Vec<Step>,
    orientation: StepOrientation,
    focused_index: usize,
    show_descriptions: bool,
    title: Option<String>,
    connector: String,
    show_border: bool,
}
```

- [ ] **Step 1.4: Update the `Default` impl**

Modify lines 226-237 to include the new field:

```rust
impl Default for StepIndicatorState {
    fn default() -> Self {
        Self {
            steps: Vec::new(),
            orientation: StepOrientation::Horizontal,
            focused_index: 0,
            show_descriptions: false,
            title: None,
            connector: "───".to_string(),
            show_border: true,
        }
    }
}
```

- [ ] **Step 1.5: Add the `with_show_border` builder method (with doc test)**

In `src/component/step_indicator/mod.rs`, insert the following method into the `impl StepIndicatorState` block, placed immediately after `with_show_descriptions` (which ends around line 318) and before the `Returns the steps.` accessor section. Doc test style matches existing methods.

```rust
    /// Sets whether the border is shown (builder pattern).
    ///
    /// Defaults to `true`. When set to `false`, the `StepIndicator` renders
    /// its steps directly into the full widget area with no surrounding
    /// box — useful for inline breadcrumbs and single-row layouts.
    ///
    /// # Title interaction
    ///
    /// When the border is hidden, the state's [`title`](Self::title) is
    /// **not rendered**. The title is drawn as part of the border block,
    /// so disabling the border silently suppresses it. If you want this
    /// to be explicit, set the title to `None`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::step_indicator::Step;
    /// use envision::component::StepIndicatorState;
    ///
    /// let state = StepIndicatorState::new(vec![Step::new("A")])
    ///     .with_show_border(false);
    /// assert!(!state.show_border());
    /// ```
    pub fn with_show_border(mut self, show: bool) -> Self {
        self.show_border = show;
        self
    }
```

- [ ] **Step 1.6: Add the `show_border` getter (with doc test)**

Insert the following method into the `impl StepIndicatorState` block near the other getters — alongside `show_descriptions()` at line 410 is a good spot.

```rust
    /// Returns whether the border is shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StepIndicatorState;
    ///
    /// let state = StepIndicatorState::default();
    /// assert!(state.show_border());
    /// ```
    pub fn show_border(&self) -> bool {
        self.show_border
    }
```

- [ ] **Step 1.7: Add the `set_show_border` mutator (with doc test)**

Insert the following method into the `impl StepIndicatorState` block after `set_orientation` (ends around line 444), keeping the setters grouped:

```rust
    /// Sets whether the border is shown.
    ///
    /// See [`with_show_border`](Self::with_show_border) for the title
    /// interaction when `show` is `false`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StepIndicatorState;
    /// use envision::component::step_indicator::Step;
    ///
    /// let mut state = StepIndicatorState::new(vec![Step::new("A")]);
    /// state.set_show_border(false);
    /// assert!(!state.show_border());
    /// ```
    pub fn set_show_border(&mut self, show: bool) {
        self.show_border = show;
    }
```

- [ ] **Step 1.8: Run the unit tests and verify they pass**

```bash
cargo nextest run -p envision step_indicator::tests::test_state_default_show_border step_indicator::tests::test_state_with_show_border step_indicator::tests::test_state_set_show_border step_indicator::tests::test_state_new
```

Expected: all four tests pass.

- [ ] **Step 1.9: Run the doc tests for the new methods**

```bash
cargo test --doc -p envision step_indicator
```

Expected: all doc tests pass, including the three new ones (`with_show_border`, `show_border`, `set_show_border`).

- [ ] **Step 1.10: Run the full `step_indicator` module test suite to ensure no existing tests broke**

```bash
cargo nextest run -p envision step_indicator
```

Expected: all existing tests (including snapshots) still pass. The rendering in `view()` has not changed yet, so snapshots must still match.

- [ ] **Step 1.11: Format and lint**

```bash
cargo fmt
cargo clippy -p envision -- -D warnings
```

Expected: no formatting diffs; clippy reports zero warnings.

- [ ] **Step 1.12: Commit**

```bash
git add src/component/step_indicator/mod.rs src/component/step_indicator/tests.rs
git commit -S -m "$(cat <<'EOF'
Add show_border field and accessors to StepIndicatorState

Adds `show_border: bool` (default `true`) with builder
`with_show_border`, getter `show_border`, and mutator
`set_show_border`, matching the `StyledTextState` convention.

This task adds only the state plumbing. The `view()` function is
updated in a follow-up commit; this commit is behavior-preserving.

Part of docs/superpowers/specs/2026-04-09-step-indicator-border-toggle-design.md

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: Update `view()` to honor `show_border` (refactor, no new tests)

**Files:**
- Modify: `src/component/step_indicator/mod.rs` — the `view()` function at lines 637-682

This task is a behavior-preserving refactor for `show_border: true` and a new code path for `show_border: false`. Correctness is verified by:
1. Existing snapshot tests (regression coverage for `show_border: true`).
2. Compilation (the new branch must type-check).

New borderless snapshots are added in Task 3.

- [ ] **Step 2.1: Replace the `view()` function body**

In `src/component/step_indicator/mod.rs`, replace the current `view` implementation at lines 637-682:

```rust
    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, ctx: &ViewContext) {
        crate::annotation::with_registry(|reg| {
            reg.register(
                area,
                crate::annotation::Annotation::new(crate::annotation::WidgetType::StepIndicator)
                    .with_id("step_indicator")
                    .with_focus(ctx.focused)
                    .with_disabled(ctx.disabled),
            );
        });

        let block = if let Some(title) = &state.title {
            Block::default()
                .title(format!(" {} ", title))
                .borders(Borders::ALL)
                .border_style(if ctx.focused {
                    theme.focused_border_style()
                } else {
                    theme.border_style()
                })
        } else {
            Block::default()
                .borders(Borders::ALL)
                .border_style(if ctx.focused {
                    theme.focused_border_style()
                } else {
                    theme.border_style()
                })
        };

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if state.steps.is_empty() {
            return;
        }

        match state.orientation {
            StepOrientation::Horizontal => {
                render_horizontal(state, frame, inner, theme, ctx.focused);
            }
            StepOrientation::Vertical => {
                render_vertical(state, frame, inner, theme, ctx.focused);
            }
        }
    }
```

with:

```rust
    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, ctx: &ViewContext) {
        crate::annotation::with_registry(|reg| {
            reg.register(
                area,
                crate::annotation::Annotation::new(crate::annotation::WidgetType::StepIndicator)
                    .with_id("step_indicator")
                    .with_focus(ctx.focused)
                    .with_disabled(ctx.disabled),
            );
        });

        let inner = if state.show_border {
            let mut block = Block::default()
                .borders(Borders::ALL)
                .border_style(if ctx.focused {
                    theme.focused_border_style()
                } else {
                    theme.border_style()
                });
            if let Some(title) = &state.title {
                block = block.title(format!(" {} ", title));
            }
            let inner = block.inner(area);
            frame.render_widget(block, area);
            inner
        } else {
            area
        };

        if state.steps.is_empty() {
            return;
        }

        match state.orientation {
            StepOrientation::Horizontal => {
                render_horizontal(state, frame, inner, theme, ctx.focused);
            }
            StepOrientation::Vertical => {
                render_vertical(state, frame, inner, theme, ctx.focused);
            }
        }
    }
```

Notes on what changed:
- The two-armed `if let Some(title)` block (which duplicated the `borders`/`border_style` construction) is collapsed into one arm. Title is applied conditionally via `block.title(...)` on the same block.
- The whole block construction is wrapped in `if state.show_border`.
- When `show_border` is `false`, `inner = area` (the full widget area) and no block is rendered.
- The annotation registration is unchanged, runs unconditionally, and still registers on the full `area`.

- [ ] **Step 2.2: Run the existing snapshot tests to verify no regression**

```bash
cargo nextest run -p envision step_indicator
```

Expected: **all existing tests pass, including all snapshot tests** (`test_view_horizontal`, `test_view_vertical`, `test_view_with_title`, `test_view_focused_step`, `test_view_vertical_descriptions`, `test_view_all_statuses`, `test_view_empty`). If any snapshot fails, the refactor is not behavior-preserving for `show_border: true` — investigate and fix. Do not `cargo insta accept` a snapshot regression at this stage; something is wrong.

- [ ] **Step 2.3: Format and lint**

```bash
cargo fmt
cargo clippy -p envision -- -D warnings
```

Expected: no formatting diffs; clippy reports zero warnings.

- [ ] **Step 2.4: Commit**

```bash
git add src/component/step_indicator/mod.rs
git commit -S -m "$(cat <<'EOF'
Branch StepIndicator::view on show_border

When show_border is false, skip block construction and render steps
into the full widget area. When true, behavior is unchanged: the
existing snapshot tests act as regression coverage.

Also collapses the duplicated two-armed Block construction in the
with-border path into a single arm that applies the title
conditionally via block.title(...).

Part of docs/superpowers/specs/2026-04-09-step-indicator-border-toggle-design.md

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: Add borderless rendering snapshot tests

**Files:**
- Modify: `src/component/step_indicator/tests.rs` — add four new snapshot tests in the "Rendering Snapshot Tests" section
- Create: four new snapshot files under `src/component/step_indicator/snapshots/` (auto-generated by insta, then reviewed and accepted by the engineer)

Insta workflow reminder: when a snapshot test runs for the first time, insta writes a `.snap.new` file next to the existing snapshots directory. Running `cargo insta review` lets you inspect each pending snapshot; `a` accepts, `r` rejects. For this task, **review each pending snapshot by eye** before accepting — do not bulk-accept — because these snapshots are the only assertion that the rendering is correct.

- [ ] **Step 3.1: Add `test_view_borderless_horizontal`**

Insert into the "Rendering Snapshot Tests" section of `src/component/step_indicator/tests.rs`, after the existing `test_view_empty` test (around line 727):

```rust
#[test]
fn test_view_borderless_horizontal() {
    let (mut terminal, theme) = setup_render(60, 3);
    let steps = vec![
        Step::new("Build").with_status(StepStatus::Completed),
        Step::new("Test").with_status(StepStatus::Active),
        Step::new("Deploy"),
    ];
    let state = StepIndicatorState::new(steps).with_show_border(false);

    terminal
        .draw(|frame| {
            StepIndicator::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    let display = terminal.backend().to_string();
    insta::assert_snapshot!("view_borderless_horizontal", display);
}
```

- [ ] **Step 3.2: Run the test, review the pending snapshot, accept it**

```bash
cargo nextest run -p envision step_indicator::tests::test_view_borderless_horizontal
```

Expected: test fails with "snapshot pending". Then:

```bash
cargo insta review
```

Review the snapshot interactively. It should show three steps rendered horizontally with icons and connectors, and **no box-drawing characters** (no `┌`, `┐`, `└`, `┘`, `─`, `│` from the border — but note the connector itself uses `───`, so `───` between steps is expected and correct; the *outer frame* lines are what should be absent). Accept with `a` if correct.

Re-run to confirm it now passes:

```bash
cargo nextest run -p envision step_indicator::tests::test_view_borderless_horizontal
```

Expected: PASS.

- [ ] **Step 3.3: Add `test_view_borderless_vertical`**

Insert after `test_view_borderless_horizontal`:

```rust
#[test]
fn test_view_borderless_vertical() {
    let (mut terminal, theme) = setup_render(20, 8);
    let steps = vec![
        Step::new("Build").with_status(StepStatus::Completed),
        Step::new("Test").with_status(StepStatus::Active),
        Step::new("Deploy"),
    ];
    let state = StepIndicatorState::new(steps)
        .with_orientation(StepOrientation::Vertical)
        .with_show_border(false);

    terminal
        .draw(|frame| {
            StepIndicator::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    let display = terminal.backend().to_string();
    insta::assert_snapshot!("view_borderless_vertical", display);
}
```

- [ ] **Step 3.4: Run, review, accept**

```bash
cargo nextest run -p envision step_indicator::tests::test_view_borderless_vertical
cargo insta review
```

Expected after review: three steps rendered vertically with the internal `│` connector between steps but **no outer frame**. Accept if correct, then re-run to confirm PASS.

- [ ] **Step 3.5: Add `test_view_borderless_one_row` (the breadcrumb use case)**

Insert after `test_view_borderless_vertical`:

```rust
#[test]
fn test_view_borderless_one_row() {
    // The canonical breadcrumb use case: a single row of steps
    // inline in a larger layout, with no surrounding box.
    // Before this feature, a 1-row area rendered nothing because
    // the border consumed all vertical space.
    let (mut terminal, theme) = setup_render(60, 1);
    let steps = vec![
        Step::new("Home"),
        Step::new("Docs").with_status(StepStatus::Active),
        Step::new("Guide"),
    ];
    let state = StepIndicatorState::new(steps).with_show_border(false);

    terminal
        .draw(|frame| {
            StepIndicator::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    let display = terminal.backend().to_string();
    insta::assert_snapshot!("view_borderless_one_row", display);
}
```

- [ ] **Step 3.6: Run, review, accept**

```bash
cargo nextest run -p envision step_indicator::tests::test_view_borderless_one_row
cargo insta review
```

Expected after review: a single line showing `○ Home ─── ● Docs ─── ○ Guide` (or similar with theme colors). **Crucially, the single line must contain actual step content** — this is the test that proves the customer's use case works. If the snapshot is empty or only whitespace, the rendering is broken in 1-row mode and the rest of Task 3 should not proceed. Accept if correct, then re-run to confirm PASS.

- [ ] **Step 3.7: Add `test_view_borderless_drops_title`**

Insert after `test_view_borderless_one_row`:

```rust
#[test]
fn test_view_borderless_drops_title() {
    // Locks in the design decision that the title is silently
    // suppressed when show_border is false. The title field still
    // exists on the state; only the rendering is suppressed.
    let (mut terminal, theme) = setup_render(60, 3);
    let steps = vec![
        Step::new("Build").with_status(StepStatus::Completed),
        Step::new("Test").with_status(StepStatus::Active),
    ];
    let state = StepIndicatorState::new(steps)
        .with_title("Pipeline")
        .with_show_border(false);

    // Sanity: the title IS still stored on the state; it's only
    // rendering that drops it.
    assert_eq!(state.title(), Some("Pipeline"));

    terminal
        .draw(|frame| {
            StepIndicator::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    let display = terminal.backend().to_string();

    // The rendered output must contain the step labels but NOT the
    // title text. We check this explicitly (in addition to the
    // snapshot) because the title-drop behavior is the whole point
    // of this test.
    assert!(display.contains("Build"), "step label 'Build' must be visible");
    assert!(display.contains("Test"), "step label 'Test' must be visible");
    assert!(
        !display.contains("Pipeline"),
        "title must not be rendered when show_border is false, but display was:\n{display}",
    );

    insta::assert_snapshot!("view_borderless_drops_title", display);
}
```

- [ ] **Step 3.8: Run, review, accept**

```bash
cargo nextest run -p envision step_indicator::tests::test_view_borderless_drops_title
cargo insta review
```

Expected after review: only step labels visible, no "Pipeline" text anywhere in the output. The inline `assert!` calls will catch a regression even if a future maintainer blindly re-accepts the snapshot. Accept if correct, then re-run to confirm PASS.

- [ ] **Step 3.9: Run the full `step_indicator` test module to confirm everything passes together**

```bash
cargo nextest run -p envision step_indicator
```

Expected: all tests pass, including the 7 existing snapshot tests and the 4 new borderless ones.

- [ ] **Step 3.10: Format and lint**

```bash
cargo fmt
cargo clippy -p envision -- -D warnings
```

Expected: no formatting diffs; clippy reports zero warnings.

- [ ] **Step 3.11: Commit**

```bash
git add src/component/step_indicator/tests.rs src/component/step_indicator/snapshots/
git commit -S -m "$(cat <<'EOF'
Add snapshot tests for StepIndicator borderless mode

Four new tests:
- test_view_borderless_horizontal: 3 steps, horizontal, borderless.
- test_view_borderless_vertical: 3 steps, vertical, borderless.
- test_view_borderless_one_row: the canonical breadcrumb use case
  (1-row area, previously impossible because the border consumed
  all vertical space).
- test_view_borderless_drops_title: locks in the title-suppression
  behavior when borderless, with explicit inline assertions on the
  rendered output in addition to the snapshot.

Part of docs/superpowers/specs/2026-04-09-step-indicator-border-toggle-design.md

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: Update CHANGELOG and run full verification

**Files:**
- Modify: `CHANGELOG.md` — add an entry under `## [Unreleased]`

- [ ] **Step 4.1: Add a changelog entry**

Open `CHANGELOG.md`. Find the `## [Unreleased]` heading (near the top of the file, around line 8). If there is no `### Added` subsection under it, add one. Append this entry:

```markdown
## [Unreleased]

### Added

- `StepIndicatorState::with_show_border(bool)`, `show_border()`, and
  `set_show_border(bool)` for opting out of the border box. When the
  border is disabled, `StepIndicator` becomes usable as an inline
  breadcrumb in a single-row area. Defaults to `true` so existing
  callers see no change. Matches the naming convention of
  `StyledTextState::with_show_border`. Note: when the border is
  hidden, the state's title is not rendered (the title is drawn as
  part of the border block).
```

If there is already an `### Added` subsection under `[Unreleased]`, append the bullet at the end of the existing list rather than creating a new subsection.

- [ ] **Step 4.2: Run the full project test suite**

```bash
cargo nextest run -p envision
cargo test --doc -p envision
```

Expected: all tests pass (unit, integration, and doc tests).

- [ ] **Step 4.3: Final format + clippy + build check**

```bash
cargo fmt
cargo clippy -p envision --all-targets -- -D warnings
cargo build -p envision
```

Expected: no formatting diffs, zero clippy warnings, clean build.

- [ ] **Step 4.4: Commit the changelog and verify branch state**

```bash
git add CHANGELOG.md
git commit -S -m "$(cat <<'EOF'
Document StepIndicator show_border in CHANGELOG

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
EOF
)"
git log --oneline main..HEAD
```

Expected commit graph (spec commit was already on the branch before this plan started):

```
<hash> Document StepIndicator show_border in CHANGELOG
<hash> Add snapshot tests for StepIndicator borderless mode
<hash> Branch StepIndicator::view on show_border
<hash> Add show_border field and accessors to StepIndicatorState
<hash> Add design spec for StepIndicator borderless mode
```

- [ ] **Step 4.5: Merge latest origin/main into the branch before pushing**

Per project conventions, feature branches must include the latest main before opening a PR.

```bash
git fetch origin
git merge origin/main --no-ff -S
```

If there are merge conflicts, resolve them manually, then re-run the test + clippy commands from Step 4.3 to ensure nothing broke. Commit the merge with a signed signature (already forced by `commit.gpgsign=true`).

If there are no changes on origin/main since the branch was created, the merge is a no-op and can be skipped.

- [ ] **Step 4.6: Push the branch and open the PR**

```bash
git push -u origin step-indicator-borderless-mode
gh pr create --title "Add show_border toggle for StepIndicator (breadcrumb mode)" --body "$(cat <<'EOF'
## Summary

- Adds `StepIndicatorState::with_show_border(bool)` + `show_border()` + `set_show_border(bool)` for opting out of the `StepIndicator` border box.
- When disabled, `StepIndicator` becomes usable as an inline breadcrumb in a single-row area (previously impossible — the border consumed both rows).
- Defaults to `true`, so every existing caller is unaffected. Matches the naming convention already established by `StyledTextState::with_show_border`.
- When the border is hidden, the state's title is silently suppressed (matching `StyledTextState`). Documented on the builder's doc comment.

Design spec: `docs/superpowers/specs/2026-04-09-step-indicator-border-toggle-design.md`

Addresses customer feedback for the next release.

## Test plan

- [ ] `cargo nextest run -p envision step_indicator` — all tests pass (11 snapshot tests: 7 existing regression + 4 new borderless)
- [ ] `cargo test --doc -p envision step_indicator` — doc tests for the three new methods pass
- [ ] Existing snapshot tests unchanged (regression coverage for the with-border path)
- [ ] `cargo clippy -p envision --all-targets -- -D warnings` — no warnings
- [ ] `cargo fmt` — no diffs

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

- [ ] **Step 4.7: Wait for CI and check results**

```bash
gh pr checks $(gh pr view --json number -q .number)
```

Do not merge until all required checks pass (Clippy, Format, Test on 3 platforms × 2 Rust versions per project conventions). If anything fails, investigate and push fixes.

---

## Definition of done

- [ ] Branch contains 5 commits (spec + 4 implementation commits).
- [ ] All 11 snapshot tests in `step_indicator` pass (7 existing + 4 new borderless).
- [ ] All doc tests for the three new methods pass.
- [ ] `cargo clippy --all-targets -- -D warnings` reports zero warnings.
- [ ] `cargo fmt` reports no diffs.
- [ ] `CHANGELOG.md` has an `[Unreleased]` entry describing the addition.
- [ ] PR opened, CI green, ready for review/merge.
- [ ] PR will be squash-merged per project conventions.
