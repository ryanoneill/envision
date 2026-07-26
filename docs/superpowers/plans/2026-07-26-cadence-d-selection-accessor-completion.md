# Cadence D — selection-accessor completion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Finish the selection-accessor unification Cadence A began — after this cadence, `grep -rn 'pub fn selected(&self)' src/component/` returns exactly one hit (`heatmap`, whose accessor returns coordinates and is a different concept).

**Architecture:** Three units, each a signed commit. Unit 1 deletes 12 literal `selected()` aliases and renames 10 setters. Unit 2 renames 4 primary accessors (`alert_panel`, `diagram`, `multi_progress`, `box_plot`) and 2 more setters. Unit 3 adds four missing harness types to the prelude and writes CHANGELOG + MIGRATION. Setter renames land in the *same commit* as their getter change — this is a design constraint, not a convenience, because deleting a getter orphans its setter and drops the audit scorecard.

**Tech Stack:** Rust (edition 2024, MSRV 1.85), ratatui 0.29, cargo-nextest, insta snapshots.

## Global Constraints

Every task's requirements implicitly include this section.

- Signed commits required. If `git commit -S` fails, STOP and ask the user — never bypass with `--no-gpg-sign`.
- Files must stay under 1000 lines.
- `cargo clippy --all-features -- -D warnings` must be clean.
- `cargo doc --no-deps --all-features` must produce zero intra-doc-link warnings.
- `cargo build --no-default-features` AND `cargo test --no-default-features --no-run` must both pass (D8 lesson).
- **Audit scorecard 9/9 AND doctest coverage at 100%** — verified at the end of **each unit**, not deferred to a final task. Two independent regression vectors exist; see the doctest hazard below.
- **`heatmap` is OUT OF SCOPE and must not be touched.** Its 29 `src/component/` call sites and 6 `tests/` call sites must survive unchanged.
- **`box_plot` return types stay bare `usize`** — do NOT convert to `Option<usize>`. Naming only.
- Docstring notes use plain backticks, matching the shipped Cadence A form at `table/state.rs:422-423`. Do **not** introduce intra-doc links.
- Tautology alias-equivalence tests are **deleted**, not renamed.

### The doctest-coverage hazard (read before editing any docstring)

The scorecard gates doctest coverage at **100% (1777/1777)** — a hard equality, not a threshold — and counts a `pub fn` as covered only when a ` ``` ` fence appears in the `///` block **immediately above** it (`tools/audit/src/scorecard.rs`, `has_doc_test_above`).

When adding a "Renamed from…" note to a renamed method, **place it adjacent to existing prose, never between the prose and the closing fence.** Splitting the doc block drops coverage below 100% and fails the scorecard for a reason unrelated to symmetry.

Correct:

```rust
/// Returns the selected index.
///
/// Renamed from `set_selected()` in v0.18.0 for symmetry with
/// `selected_index()`. See MIGRATION.md.
///
/// # Example
/// ```rust
/// # …
/// ```
pub fn set_selected_index(&mut self, index: Option<usize>) {
```

Wrong — note inserted after the fence, orphaning it:

```rust
/// # Example
/// ```rust
/// # …
/// ```
///
/// Renamed from `set_selected()` in v0.18.0.
pub fn set_selected_index(&mut self, index: Option<usize>) {
```

---

## Pre-execution gotchas (read once before Task 1)

- **Two components keep their accessors in `state.rs`, not `mod.rs`:** `diagram/state.rs` and `box_plot/state.rs`. Both are behind a private `mod state;` (`diagram/mod.rs:61`, `box_plot/mod.rs:31`), which means the audit tool **cannot see them at all** (`scorecard.rs:275-295` reads `mod.rs` plus only files named by `pub mod X;` / `pub use X::`). The scorecard will not catch mistakes in those two files. The compiler will.
- **`file_browser/helper_tests.rs` is a non-standard test filename.** A search scoped to `tests.rs` misses it. It holds one of the two tautology tests.
- **`loading_list` has 30 `.set_selected(` call sites** — by far the largest single-component migration. `multi_progress` has 14, `menu` 9, `box_plot` 8.
- **`alert_panel` (20) and `box_plot` (17) have the most `.selected()` call sites**, despite being one-line signature changes.
- All Category A and B setters share the signature `(&mut self, index: Option<usize>)`. **`box_plot`'s is `(&mut self, index: usize)`** — different, and it stays that way.
- These are breaking renames, so the compiler finds every missed site. Grep gates are a backstop, not the primary mechanism.

## File structure

Files modified in Unit 1 (12 components):
`accordion`, `dropdown`, `file_browser`, `loading_list`, `menu`, `metrics_dashboard`, `radio_group`, `searchable_list`, `select`, `selectable_list`, `tabs`, `tree` — each `mod.rs` plus its test files.

Files modified in Unit 2 (4 components):
`alert_panel/mod.rs`, `diagram/state.rs`, `multi_progress/mod.rs`, `box_plot/state.rs` — plus their test files and `tests/integration_stress.rs` (5 `DiagramState` call sites).

Files modified in Unit 3:
`src/lib.rs`, `CHANGELOG.md`, `MIGRATION.md`.

---

## Task 1: Unit 1 — Category A (12 alias deletions + 10 setter renames)

**Files:**
- Modify: `src/component/{accordion,dropdown,file_browser,loading_list,menu,metrics_dashboard,radio_group,searchable_list,select,selectable_list,tabs,tree}/mod.rs`
- Modify: those components' `tests.rs` / `tests/` files
- Modify: `src/component/file_browser/helper_tests.rs`
- Modify: `src/component/accordion/tests.rs`

**Interfaces:**
- Consumes: nothing (first task).
- Produces: 12 components exposing only `selected_index()`; 10 renamed to `set_selected_index()`. Unit 3 renders these into the MIGRATION table.

### Step 1: Delete the 12 `selected()` aliases

- [ ] For each component below, delete the `pub fn selected(&self) -> Option<usize>` method **and its full docstring**. Each body is exactly `self.selected_index()` — verified. Leave `selected_index()` untouched.

| Component | Delete at |
|---|---|
| `accordion` | `accordion/mod.rs:362` |
| `dropdown` | `dropdown/mod.rs:234` |
| `file_browser` | `file_browser/mod.rs:446` |
| `loading_list` | `loading_list/mod.rs:357` |
| `menu` | `menu/mod.rs:322` |
| `metrics_dashboard` | `metrics_dashboard/mod.rs:359` |
| `radio_group` | `radio_group/mod.rs:199` |
| `searchable_list` | `searchable_list/mod.rs:322` |
| `select` | `select/mod.rs:222` |
| `selectable_list` | `selectable_list/mod.rs:242` |
| `tabs` | `tabs/mod.rs:164` |
| `tree` | `tree/mod.rs:528` |

Line numbers shift as you delete — work bottom-up within a file, or re-grep between edits.

Three of these docstrings contain a doctest that asserts alias-equivalence (`dropdown/mod.rs:232`, `metrics_dashboard/mod.rs:357`, `searchable_list/mod.rs:320`). Those die with the method — no separate action.

### Step 2: Rename the 10 Category A setters

- [ ] Rename `set_selected` → `set_selected_index` at each site. Signature is unchanged: `(&mut self, index: Option<usize>)`.

| Component | Rename at |
|---|---|
| `dropdown` | `dropdown/mod.rs:267` |
| `loading_list` | `loading_list/mod.rs:417` |
| `menu` | `menu/mod.rs:345` |
| `metrics_dashboard` | `metrics_dashboard/mod.rs:380` |
| `radio_group` | `radio_group/mod.rs:233` |
| `searchable_list` | `searchable_list/mod.rs:361` |
| `select` | `select/mod.rs:255` |
| `selectable_list` | `selectable_list/mod.rs:305` |
| `tabs` | `tabs/mod.rs:198` |
| `tree` | `tree/mod.rs:550` |

`accordion` and `file_browser` have **no** setter — getter deletion only.

- [ ] Add the rename note to each renamed setter's docstring, **adjacent to existing prose, above any `# Example` fence**:

```rust
/// Renamed from `set_selected()` in v0.18.0 for symmetry with
/// `selected_index()`. See MIGRATION.md.
```

### Step 3: Delete the two tautology tests

- [ ] `src/component/accordion/tests.rs:795-799` — delete the entire test function. Its body is nothing but the tautology:

```rust
#[test]
fn test_selected_alias_matches_selected_index() {
    let panels = vec![AccordionPanel::new("A", "1"), AccordionPanel::new("B", "2")];
    let state = AccordionState::new(panels);
    assert_eq!(state.selected(), state.selected_index());
}
```

- [ ] `src/component/file_browser/helper_tests.rs:257-262` — this one is **not** pure tautology. Current body:

```rust
#[test]
fn test_selected_alias() {
    let state = FileBrowserState::new("/", sample_entries());
    assert_eq!(state.selected(), state.selected_index());
    assert_eq!(state.selected(), Some(0));
}
```

The first assertion is the tautology; the second is real coverage (default selection is index 0). Delete the tautology line, migrate the second, and rename the function since "alias" no longer describes it:

```rust
#[test]
fn test_selected_index_defaults_to_first_entry() {
    let state = FileBrowserState::new("/", sample_entries());
    assert_eq!(state.selected_index(), Some(0));
}
```

- [ ] Verify no other alias-equivalence assertions remain:

```bash
grep -rn 'selected(), *state.selected_index()\|selected_index(), *state.selected()' src/component/
```

Expected: zero hits.

### Step 4: Migrate remaining call sites in the 12 components

- [ ] Compile to find them:

```bash
cargo check --all-features 2>&1 | tail -30
```

Every remaining `.selected()` on these 12 types and every `.set_selected(` is now a compile error. Migrate each: `.selected()` → `.selected_index()`, `.set_selected(` → `.set_selected_index(`.

Per-component call-site counts to expect (getter / setter):

| Component | `.selected()` | `.set_selected(` |
|---|---|---|
| `accordion` | 5 | 0 |
| `dropdown` | 1 | 4 |
| `file_browser` | 5 | 0 |
| `loading_list` | 1 | 30 |
| `menu` | 1 | 9 |
| `metrics_dashboard` | 1 | 2 |
| `radio_group` | 1 | 6 |
| `searchable_list` | 2 | 1 |
| `select` | 1 | 4 |
| `selectable_list` | 6 | 2 |
| `tabs` | 1 | 4 |
| `tree` | 1 | 2 |

- [ ] Repeat `cargo check --all-features` until clean.

### Step 5: Unit 1 verification

- [ ] Scoped grep gates:

```bash
grep -rn 'pub fn selected(&self)' src/component/{accordion,dropdown,file_browser,loading_list,menu,metrics_dashboard,radio_group,searchable_list,select,selectable_list,tabs,tree}/
# expect: zero hits

grep -rn '\.selected()\|\.set_selected(' src/component/{accordion,dropdown,file_browser,loading_list,menu,metrics_dashboard,radio_group,searchable_list,select,selectable_list,tabs,tree}/
# expect: zero hits
```

- [ ] **heatmap must be untouched:**

```bash
grep -rn '\.selected()' src/component/heatmap/ | wc -l    # expect: 29
grep -c 'pub fn selected(&self) -> Option<(usize, usize)>' src/component/heatmap/mod.rs   # expect: 1
```

- [ ] Test + lint:

```bash
cargo fmt --check
cargo clippy --all-features -- -D warnings
cargo nextest run --all-features
cargo test --all-features --doc
```

Expected: all clean. Test count drops by 1 (the deleted accordion tautology test) plus 3 doctests.

- [ ] **Audit gate — do not defer this to a later task:**

```bash
./tools/audit/target/release/envision-audit all 2>&1 | grep -A3 "Accessor symmetry"
./tools/audit/target/release/envision-audit all 2>&1 | grep -i "doc test coverage"
```

Expected: "All setters have matching getters" and coverage at **100.0%**. If symmetry regressed, a setter rename was missed. If coverage dropped, a docstring note split a doc block from its fence.

### Step 6: Commit Unit 1

- [ ] Stage and commit:

```bash
git add src/component/{accordion,dropdown,file_browser,loading_list,menu,metrics_dashboard,radio_group,searchable_list,select,selectable_list,tabs,tree}/
git commit -S -m "$(cat <<'EOF'
cadence-d/unit-1: delete 12 selected() aliases, rename 10 setters

Category A of the selection-accessor completion. Each deleted method
had a body of exactly `self.selected_index()` — pure redundancy.

Deleted `selected()` from: accordion, dropdown, file_browser,
loading_list, menu, metrics_dashboard, radio_group, searchable_list,
select, selectable_list, tabs, tree.

Renamed `set_selected` -> `set_selected_index` on the 10 of those that
have a setter (accordion and file_browser have none). The rename lands
in this same commit by design: deleting `selected()` orphans
`set_selected()` under the audit tool's symmetry rule
(tools/audit/src/code_analysis.rs:238-266), which is exactly the 9/9 ->
8/9 regression Cadence A discovered mid-implementation and had to fix
in an unplanned follow-up.

Deleted two alias-equivalence tests rather than renaming them — post
deletion they assert `x == x`. accordion's test body was nothing else,
so the whole fn went; file_browser's also carried a real
default-selection assertion, which is preserved under a name that
describes what it actually checks. Three more alias doctests died with
the methods whose docstrings held them.

heatmap is untouched (29 call sites intact) — its `selected()` returns
`Option<(usize, usize)>` coordinates, a different concept, excluded
since Cadence A.

Audit verified in-commit: 9/9 scorecard, symmetry clean, doctest
coverage still 100%.
EOF
)"
```

- [ ] Verify the signature: `git log --show-signature -1 HEAD | head -5`

---

## Task 2: Unit 2 — Categories B and C (4 getter renames + 2 setter renames)

**Files:**
- Modify: `src/component/alert_panel/mod.rs`
- Modify: `src/component/diagram/state.rs`
- Modify: `src/component/multi_progress/mod.rs`
- Modify: `src/component/box_plot/state.rs`
- Modify: those components' test files
- Modify: `tests/integration_stress.rs` (5 `DiagramState` call sites)

**Interfaces:**
- Consumes: nothing from Unit 1 (independent components).
- Produces: 4 more components on `selected_index()`; 2 more on `set_selected_index()`. With Unit 1, this completes the rename set that Unit 3 documents.

### Step 1: Rename the 4 getters

- [ ] Rename `selected` → `selected_index` at each site. **Bodies are unchanged** — each reads a private field, which is *not* renamed.

| Component | Site | Signature after rename |
|---|---|---|
| `alert_panel` | `alert_panel/mod.rs:347` | `pub fn selected_index(&self) -> Option<usize>` |
| `diagram` | `diagram/state.rs:263` | `pub fn selected_index(&self) -> Option<usize>` |
| `multi_progress` | `multi_progress/mod.rs:434` | `pub fn selected_index(&self) -> Option<usize>` |
| `box_plot` | `box_plot/state.rs:249` | `pub fn selected_index(&self) -> usize` |

**`box_plot` keeps its bare `usize` return.** Do not convert to `Option<usize>` — that is a separate semantic question, deferred. The bare-`usize` shape already has precedent at `flame_graph/mod.rs:369`.

The private field `selected` on each state struct stays as-is. `self.selected` behind `pub fn selected_index()` is ordinary Rust, and unlike Cadence A's `data_grid::selected_row` there is no grep collision to break (that rename existed because a *deleted method* shared the field's spelling).

- [ ] Add the rename note to each, adjacent to existing prose and above any fence:

```rust
/// Renamed from `selected()` in v0.18.0 for consistency with the
/// `selected_index()` accessor used across every other component.
/// See MIGRATION.md.
```

### Step 2: Rename the 2 setters

- [ ] `multi_progress/mod.rs:466` — `set_selected` → `set_selected_index`, signature `(&mut self, index: Option<usize>)` unchanged.
- [ ] `box_plot/state.rs:267` — `set_selected` → `set_selected_index`, signature `(&mut self, index: usize)` unchanged.

- [ ] Add the same style of rename note to both.

`alert_panel` and `diagram` have no setter.

### Step 3: Migrate call sites

- [ ] Compile to find them:

```bash
cargo check --all-features 2>&1 | tail -30
```

Per-component counts to expect (getter / setter):

| Component | `.selected()` | `.set_selected(` |
|---|---|---|
| `alert_panel` | 20 | 0 |
| `box_plot` | 17 | 8 |
| `diagram` | 5 | 0 |
| `multi_progress` | 15 | 14 |

- [ ] **`tests/integration_stress.rs` has 5 `DiagramState` call sites** at lines 427, 441, 450, 470, 488. Migrate those.

- [ ] **Do NOT touch `tests/integration_new_components.rs`** — its 6 `.selected()` calls are `HeatmapState` and must stay. Verify:

```bash
grep -rn '\.selected()' tests/
# expect: exactly 6, all in integration_new_components.rs, all HeatmapState
```

- [ ] Repeat `cargo check --all-features` until clean.

### Step 4: Unit 2 verification

- [ ] Whole-tree gate — this is the payoff:

```bash
grep -rn 'pub fn selected(&self)' src/component/
# expect: exactly 1 — heatmap/mod.rs (Option<(usize, usize)>)

grep -rn '\.set_selected(' src/ tests/ examples/ benches/
# expect: zero hits
```

The setter gate is tree-wide and honest only because `box_plot` is in scope. If it returns hits, a setter was missed.

- [ ] heatmap still untouched:

```bash
grep -rn '\.selected()' src/component/heatmap/ | wc -l   # expect: 29
grep -rn '\.selected()' tests/ | wc -l                    # expect: 6
```

- [ ] Full gauntlet:

```bash
cargo fmt --check
cargo clippy --all-features -- -D warnings
cargo nextest run --all-features
cargo test --all-features --doc
cargo build --no-default-features
cargo test --no-default-features --no-run
cargo build --examples --all-features
cargo doc --no-deps --all-features 2>&1 | grep -iE "warning|error" | head -5
```

- [ ] **Audit gate again:**

```bash
./tools/audit/target/release/envision-audit all 2>&1 | grep -iE "scorecard|Accessor symmetry|doc test coverage" -A2 | head -20
```

Expected: 9/9, symmetry clean, doctest coverage 100%.

Note that `diagram/state.rs` and `box_plot/state.rs` are invisible to the audit tool (private `mod state;`), so a clean scorecard is **not** evidence those two files are correct. The compiler and the grep gates are the real checks there.

### Step 5: Commit Unit 2

```bash
git add src/component/{alert_panel,box_plot,diagram,multi_progress}/ tests/integration_stress.rs
git commit -S -m "$(cat <<'EOF'
cadence-d/unit-2: rename 4 primary selected() accessors + 2 setters

Categories B and C. Unlike Unit 1's aliases, these four components have
no `selected_index()` sibling — `selected()` IS the accessor, reading a
private field. Renaming is pure naming consistency, not
redundancy removal.

Renamed `selected` -> `selected_index` on alert_panel (mod.rs:347),
diagram (state.rs:263), multi_progress (mod.rs:434), and box_plot
(state.rs:249). Renamed `set_selected` -> `set_selected_index` on
multi_progress (mod.rs:466) and box_plot (state.rs:267).

box_plot keeps its bare `usize` return rather than gaining `Option`.
The bare shape already has precedent at flame_graph/mod.rs:369, so this
is a pure rename needing no semantic decision; whether "no selection"
should be representable is a separate question, deferred under the new
name.

box_plot was excluded in the spec's first draft on the grounds that
normalizing it forced that decision. Adversarial review showed the
dilemma was false, and that CHANGELOG.md:192 names box_plot BY NAME in
the published Cadence D backlog — deferring it would have left the crate
with exactly one component still pairing selected() with set_selected(),
a worse outlier after the sweep than before it.

Private `selected` fields are NOT renamed. `self.selected` behind
`pub fn selected_index()` is ordinary Rust; Cadence A's analogous field
rename existed only to break a grep collision with a deleted method of
the same spelling, which does not apply here.

Whole-tree gates now pass: exactly one `pub fn selected(&self)` remains
in src/component/ (heatmap, returning coordinates), and
`grep -rn '\.set_selected(' src/ tests/ examples/ benches/` returns
zero.

heatmap untouched: 29 call sites in src/component/, 6 in
tests/integration_new_components.rs.

Audit verified in-commit: 9/9, symmetry clean, doctest coverage 100%.
Note that diagram/state.rs and box_plot/state.rs sit behind a private
`mod state;` and are invisible to the audit tool — the compiler and grep
gates are the real verification for those two.
EOF
)"
```

- [ ] Verify the signature.

---

## Task 3: Unit 3 — prelude + CHANGELOG + MIGRATION

**Files:**
- Modify: `src/lib.rs:478`
- Modify: `CHANGELOG.md`
- Modify: `MIGRATION.md`

**Interfaces:**
- Consumes: the complete rename set from Units 1 and 2 (28 rows).
- Produces: consumer-facing documentation.

### Step 1: Extend the prelude with all four missing harness types

- [ ] `src/lib.rs:478` currently reads:

```rust
    pub use crate::harness::{AppHarness, MessageSender, TestHarness};
```

Replace with:

```rust
    pub use crate::harness::{
        AppHarness, Assertion, MessageSendError, MessageSender, Snapshot, TestHarness,
        TrySendError,
    };
```

This mirrors the crate-root export at `src/lib.rs:408-410` exactly. Cadence A's review flagged only `MessageSendError` + `TrySendError`; fixing 2 of 4 would recreate the inconsistency one level down, so `Assertion` and `Snapshot` come along.

- [ ] Verify:

```bash
cargo check --all-features 2>&1 | tail -3
```

### Step 2: CHANGELOG — new `[Unreleased]` block

- [ ] **Leave `## [0.17.0] - 2026-07-26` byte-identical.** It is released and tagged; Keep a Changelog treats released sections as immutable. Its Known Deferred Findings block stays exactly as shipped.

- [ ] Insert a fresh `## [Unreleased]` block directly above `## [0.17.0] - 2026-07-26` (i.e. after the format preamble):

```markdown
## [Unreleased]

### Breaking Changes

#### Selection accessors completed on `selected_index()`

Cadence A unified selection accessors across six components. This release finishes the job: every component whose selection accessor was spelled `selected` now spells it `selected_index`, and every corresponding mutator is `set_selected_index`.

The motivation is that `selected` was ambiguous about return type. Four different signatures shared the spelling: `Option<usize>` (15 components), `Option<(usize, usize)>` (`heatmap`), bare `usize` (`box_plot`), and a `bool` builder (`annotation::widget`). The `selected_index` / `selected_item` / `value_at_selection` system makes the name predict the type.

**Aliases deleted** (body was exactly `self.selected_index()`) — `accordion`, `dropdown`, `file_browser`, `loading_list`, `menu`, `metrics_dashboard`, `radio_group`, `searchable_list`, `select`, `selectable_list`, `tabs`, `tree`.

**Primary accessors renamed** (no `selected_index()` sibling existed) — `alert_panel`, `diagram`, `multi_progress`, `box_plot`.

**Setters renamed** `set_selected` → `set_selected_index` on all twelve components that had one.

`box_plot::selected_index()` keeps its bare `usize` return, matching the existing `flame_graph::selected_index()`. Whether it should become `Option<usize>` is a separate question, deferred.

`heatmap::selected()` is **unchanged** — it returns `Option<(usize, usize)>` coordinates, not an index, and was disentangled from this pattern during Cadence A.

See `MIGRATION.md` § *v0.17.x to v0.18.0* for the full before/after table.

### Added

#### Harness types available from the prelude

`envision::prelude::*` now re-exports all seven harness types, matching the crate root. Previously only `AppHarness`, `MessageSender`, and `TestHarness` were included, so consumers matching on `TrySendError::{Full, Closed}`, destructuring `MessageSendError`, or using `Assertion` / `Snapshot` needed a second explicit import.

Added: `Assertion`, `MessageSendError`, `Snapshot`, `TrySendError`.

### Known Deferred Findings

> Supersedes the Known Deferred Findings block under `[0.17.0]`. The Cadence D item listed there — selection-accessor aliases, including `box_plot` — is **closed** by this release.

- **Selection-index accessors under domain-specific names.** `chart::active_series`, `log_correlation::active_stream`, `diff_viewer::current_hunk`, `step_indicator::active_step_index`, `paginator::current_page`, `breadcrumb::focused_index`. Surveyed during Cadence D and deliberately excluded: each name carries domain meaning that a generic rename would erase. Open question for a future cadence.
- **`box_plot::selected_index()` returns a bare `usize`**, so "no selection" is unrepresentable. Whether it should become `Option<usize>` is a semantic question, deferred — the naming half closed in this release.
- **`accordion::selected_index()` remains a convenience alias for `focused_index()`.** It performs real work (Option-normalizing the empty case) so it is not redundant, but the indirection stands.
- **`compact_str` adoption is sporadic** — 2 non-test source files (`src/component/cell.rs`, `src/backend/cell/mod.rs`). Needs a commit-or-drop decision.
- **Naming outliers** — `is_checked`, `label_text` (the `tab_bar` setter outlier previously grouped here is closed by this release). Plus `restore_terminal` → `restore`, `AppShell` placement in the README component table, five files near the 1000-line cap, and snapshot-coverage concentration in ~20 of 74 components.
```

### Step 3: MIGRATION.md — new `## v0.17.x to v0.18.0` section

- [ ] Insert directly above `## v0.16.x to v0.17.0` (currently at line 3):

```markdown
## v0.17.x to v0.18.0

### Selection accessors completed on `selected_index()`

Every component whose selection accessor was spelled `selected` now spells it `selected_index`; every corresponding mutator is `set_selected_index`.

| Component | Old | New |
|---|---|---|
| `accordion` | `state.selected()` | `state.selected_index()` |
| `alert_panel` | `state.selected()` | `state.selected_index()` |
| `box_plot` | `state.selected()` | `state.selected_index()` |
| `box_plot` | `state.set_selected(i)` | `state.set_selected_index(i)` |
| `diagram` | `state.selected()` | `state.selected_index()` |
| `dropdown` | `state.selected()` | `state.selected_index()` |
| `dropdown` | `state.set_selected(i)` | `state.set_selected_index(i)` |
| `file_browser` | `state.selected()` | `state.selected_index()` |
| `loading_list` | `state.selected()` | `state.selected_index()` |
| `loading_list` | `state.set_selected(i)` | `state.set_selected_index(i)` |
| `menu` | `state.selected()` | `state.selected_index()` |
| `menu` | `state.set_selected(i)` | `state.set_selected_index(i)` |
| `metrics_dashboard` | `state.selected()` | `state.selected_index()` |
| `metrics_dashboard` | `state.set_selected(i)` | `state.set_selected_index(i)` |
| `multi_progress` | `state.selected()` | `state.selected_index()` |
| `multi_progress` | `state.set_selected(i)` | `state.set_selected_index(i)` |
| `radio_group` | `state.selected()` | `state.selected_index()` |
| `radio_group` | `state.set_selected(i)` | `state.set_selected_index(i)` |
| `searchable_list` | `state.selected()` | `state.selected_index()` |
| `searchable_list` | `state.set_selected(i)` | `state.set_selected_index(i)` |
| `select` | `state.selected()` | `state.selected_index()` |
| `select` | `state.set_selected(i)` | `state.set_selected_index(i)` |
| `selectable_list` | `state.selected()` | `state.selected_index()` |
| `selectable_list` | `state.set_selected(i)` | `state.set_selected_index(i)` |
| `tabs` | `state.selected()` | `state.selected_index()` |
| `tabs` | `state.set_selected(i)` | `state.set_selected_index(i)` |
| `tree` | `state.selected()` | `state.selected_index()` |
| `tree` | `state.set_selected(i)` | `state.set_selected_index(i)` |

Return types are unchanged in every case, including `box_plot`, whose accessor stays a bare `usize`.

**Finding your call sites.** Search for `.selected()` and `.set_selected(` on the sixteen components above.

`heatmap::selected()` returns `Option<(usize, usize)>` coordinates and is **unchanged** — do not migrate it.

Because these are renames, `cargo build` identifies every site and the compiler error names the receiver type, which disambiguates `heatmap` from the rest. Note that method references (`SelectState::selected`, or `.map(SelectState::selected)`) will not match a `.selected()` grep but will still fail to compile.

### Harness types available from the prelude

`envision::prelude::*` now re-exports `Assertion`, `MessageSendError`, `Snapshot`, and `TrySendError` alongside the three it already provided. The explicit `use envision::{Assertion, MessageSendError, Snapshot, TrySendError};` import is no longer required.
```

- [ ] **Archiving policy note:** PR #510 set a last-3-versions boundary for `MIGRATION.md`. This addition makes four sections and takes the file to roughly 254 lines — far below the 1000-line cap — so the policy is waived this cycle. Archive `v0.14.x → v0.15.0` at v0.19.0. No action now; this is recorded so the next maintainer does not rediscover the rule.

### Step 4: Unit 3 verification

```bash
grep -c '^## \[Unreleased\]' CHANGELOG.md    # expect: 1
grep -c '^## \[0.17.0\]' CHANGELOG.md        # expect: 1
grep -c '^## v0.17.x to v0.18.0' MIGRATION.md # expect: 1
cargo test --all-features --doc 2>&1 | tail -3
cargo doc --no-deps --all-features 2>&1 | grep -iE "warning|error" | head -5
```

- [ ] Confirm `[0.17.0]` is unmodified:

```bash
git diff HEAD -- CHANGELOG.md | grep '^-' | grep -v '^---' | head -5
```

Expected: no deletions inside the `[0.17.0]` section — only additions above it.

### Step 5: Commit Unit 3

```bash
git add src/lib.rs CHANGELOG.md MIGRATION.md
git commit -S -m "$(cat <<'EOF'
cadence-d/unit-3: prelude harness types + CHANGELOG + MIGRATION

Extends the prelude to re-export all seven harness types, matching the
crate root at src/lib.rs:408-410. Cadence A's final review flagged only
MessageSendError and TrySendError; adding just those two would have
recreated the same inconsistency one level down, so Assertion and
Snapshot come along. Additive, non-breaking. Bundling this with Cadence D
was sanctioned by the Cadence A closure record
(docs/audits/2026-07-05-post-consistency-cleanup.md:100).

CHANGELOG gains a fresh [Unreleased] block. The [0.17.0] section is left
byte-identical — it is released and tagged, and Keep a Changelog treats
released sections as immutable. The new Known Deferred Findings block
opens with an explicit pointer that it supersedes the one under
[0.17.0], and drops the now-closed Cadence D entry.

What remains deferred is stated precisely rather than by reference:
selection-index accessors under domain-specific names (active_series,
current_page, focused_index, and three others — surveyed during this
cadence and excluded because each name carries domain meaning a generic
rename would erase); box_plot's bare-usize return; accordion's
focused_index indirection; compact_str's 2-file adoption; and the
residual naming/size/coverage items.

MIGRATION gains § v0.17.x to v0.18.0 with a 28-row before/after table
and a grep hint that explicitly warns off heatmap and notes that method
references won't match a `.selected()` grep but will still fail to
compile.

The last-3-versions archiving policy from PR #510 is waived this cycle —
four sections lands the file near 254 lines, nowhere near the 1000-line
cap. Archive v0.14.x -> v0.15.0 at v0.19.0.
EOF
)"
```

- [ ] Verify the signature.

---

## Task 4: Full verification gauntlet

**Files:** none — verification only, no commit.

### Step 1: Complete gauntlet

```bash
cargo fmt --check
cargo clippy --all-features -- -D warnings
cargo nextest run --all-features
cargo test --all-features --doc
cargo build --no-default-features
cargo test --no-default-features --no-run
cargo build --examples --all-features
cargo doc --no-deps --all-features 2>&1 | grep -iE "warning|error" | head -10
./tools/audit/target/release/envision-audit all 2>&1 | grep -iE "scorecard|PASS|FAIL" | head -20
```

Expected: everything clean; scorecard 9/9; doctest coverage 100%.

### Step 2: Final success-criteria gates

```bash
# 1. Exactly one selected() left in components — heatmap's coordinate accessor
grep -rn 'pub fn selected(&self)' src/component/

# 2. No set_selected anywhere
grep -rn '\.set_selected(' src/ tests/ examples/ benches/

# 3. heatmap intact
grep -rn '\.selected()' src/component/heatmap/ | wc -l   # 29
grep -rn '\.selected()' tests/ | wc -l                    # 6

# 4. box_plot kept bare usize
grep -n 'pub fn selected_index(&self) -> usize' src/component/box_plot/state.rs
grep -n 'pub fn set_selected_index(&mut self, index: usize)' src/component/box_plot/state.rs

# 5. prelude has all seven harness types
grep -A3 'pub use crate::harness::' src/lib.rs

# 6. CHANGELOG structure
grep -c '^## \[Unreleased\]' CHANGELOG.md   # 1
```

If any gate fails, backtrack to the offending unit and fix before proceeding.

### Step 3: No commit

Task 4 is verification-only. Units 1–3 carry the commits.

---

## Task 5: Push + open impl PR

### Step 1: Confirm branch state

```bash
git log --oneline -4
```

Expected, most recent first: Unit 3, Unit 2, Unit 1, then the branch parent.

### Step 2: Merge latest main

```bash
git fetch origin main
git merge origin/main --no-ff -S -m "Merge origin/main into cadence-d-impl"
```

`CHANGELOG.md` is the likely conflict candidate if anything landed on main meanwhile. If signing fails, STOP and ask the user.

### Step 3: Push

```bash
git push -u origin cadence-d-impl
```

### Step 4: Open the PR

```bash
gh pr create --title "Impl: Cadence D — selection-accessor completion (v0.18.0)" --body "$(cat <<'EOF'
## Summary

Finishes the selection-accessor unification Cadence A began. After this PR, `grep -rn 'pub fn selected(&self)' src/component/` returns **exactly one** hit — `heatmap`, whose accessor returns coordinates and is a genuinely different concept.

- **Unit 1** — deleted 12 literal `selected()` aliases (body was exactly `self.selected_index()`), renamed 10 setters to `set_selected_index`.
- **Unit 2** — renamed 4 primary accessors (`alert_panel`, `diagram`, `multi_progress`, `box_plot`) plus 2 more setters. These had no `selected_index()` sibling, so this is naming consistency rather than redundancy removal.
- **Unit 3** — prelude gains all four missing harness types; CHANGELOG `[Unreleased]`; MIGRATION § v0.17.x to v0.18.0 with a 28-row table.

## Why `selected_index`

`selected` was ambiguous about return type, and the codebase proved it — four signatures shared the spelling: `Option<usize>` (15 components), `Option<(usize, usize)>` (`heatmap`), bare `usize` (`box_plot`), and a `bool` builder (`annotation::widget:75`). The `selected_index` / `selected_item` / `value_at_selection` system makes the name predict the type.

## Setter symmetry, pre-empted

Deleting a `selected()` getter orphans its `set_selected()` under the audit tool's symmetry rule — the 9/9 → 8/9 regression Cadence A hit mid-implementation and fixed in an unplanned follow-up commit. Here every setter renames in the **same commit** as its getter, and the audit runs at the end of **each unit**.

## box_plot

The spec's first draft excluded it. Adversarial review showed that was wrong on two counts: `flame_graph::selected_index() -> usize` already ships the bare-`usize` shape, so the rename needs no semantic decision; and `CHANGELOG.md:192` names `box_plot` by name in the published Cadence D backlog. Excluding it would have left the crate with exactly one component still pairing `selected()` with `set_selected()`.

Its return types stay bare `usize`. The `Option` question is deferred under the new name.

## Verification

- [x] `grep -rn 'pub fn selected(&self)' src/component/` → 1 (heatmap)
- [x] `grep -rn '\.set_selected(' src/ tests/ examples/ benches/` → 0
- [x] heatmap untouched — 29 `src/component/` sites, 6 `tests/` sites
- [x] `cargo fmt --check` / `clippy --all-features -D warnings` clean
- [x] `cargo nextest run --all-features` passing
- [x] `cargo test --all-features --doc` passing
- [x] `cargo build --no-default-features` + `cargo test --no-default-features --no-run` clean
- [x] `cargo build --examples --all-features` clean
- [x] `cargo doc --no-deps --all-features` zero warnings
- [x] Audit scorecard **9/9**, doctest coverage **100%**

## Spec / plan

- Spec: `docs/superpowers/specs/2026-07-26-cadence-d-selection-accessor-completion-design.md` (PR #514)
- Plan: `docs/superpowers/plans/2026-07-26-cadence-d-selection-accessor-completion.md`

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

### Step 5: CI watch

- [ ] `gh pr checks <PR>` until required checks complete. The Coverage job has a known tarpaulin timeout flake — if it fails *after* the test summary shows all tests passed, retrigger with `gh run rerun <RUN_ID> --failed`. Coverage is not a required check.

---

## Out of scope for this plan

- Tracking-doc PR (separate branch after impl merges) — closure record at `docs/audits/2026-07-26-post-cadence-d.md`.
- Renaming the domain-specific accessors (`active_series`, `current_page`, `focused_index`, …) — surveyed and deferred.
- Converting `box_plot` to `Option<usize>` — semantic question, deferred.
- `accordion::selected_index()` → `focused_index()` indirection.
- The v0.18.0 release itself.

## Recovery patterns

- **`git commit -S` fails** → STOP, ask the user. Never `--no-gpg-sign`.
- **Audit symmetry regresses mid-unit** → a setter rename was missed. `./tools/audit/target/release/envision-audit all | grep -A5 "Accessor symmetry"` names the component.
- **Doctest coverage drops below 100%** → a "Renamed from…" note split a doc block from its fence. Move the note adjacent to existing prose, above the fence.
- **heatmap gate returns a count other than 29 / 6** → over-migration. Revert the heatmap changes; its accessor is out of scope.
- **`cargo fmt --check` drifts** → run `cargo fmt`, stage, add a small follow-up signed commit. Don't amend.
- **Merge conflict on CHANGELOG.md in Task 5** → keep both, ordering `[Unreleased]` above `[0.17.0]`.

## Reference

- Spec: `docs/superpowers/specs/2026-07-26-cadence-d-selection-accessor-completion-design.md` (PR #514, commits `d80988a` + `bbe5ba7`).
- Cadence A: spec #506, plan #507, impl #508, tracking #509. Closure record at `docs/audits/2026-07-05-post-consistency-cleanup.md`.
- Audit tool symmetry rule: `tools/audit/src/code_analysis.rs:238-266`. Source-selection rule (the `state.rs` blind spot): `tools/audit/src/scorecard.rs:275-295`.
- CLAUDE.md: PRs required; signed commits; squash-merge; merge `origin/main` before push; files under 1000 lines; no clippy warnings.
