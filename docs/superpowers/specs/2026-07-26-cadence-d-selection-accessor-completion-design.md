# Cadence D — selection-accessor completion (v0.18.0) — design

## Purpose

Finish the selection-accessor unification that Cadence A began. Cadence A canonicalized `selected_item()` / `selected_index()` / `set_selected_index()` across six components (`dropdown`, `select`, `heatmap`, `tab_bar`, `data_grid`, `table`) but deliberately scoped out the remaining components carrying a `selected()` accessor. That deferral was documented in the CHANGELOG's Known Deferred Findings block and in the Cadence A closure record as "Cadence D (v0.18+)".

This cadence closes it. After Cadence D, **every** envision component that exposes a selection index calls it `selected_index()`, and every corresponding mutator is `set_selected_index()`.

- **Prior cadence:** Cadence A (spec PR #506, plan PR #507, impl PR #508, tracking PR #509), closure record at [`docs/audits/2026-07-05-post-consistency-cleanup.md`](../../audits/2026-07-05-post-consistency-cleanup.md).
- **Release target:** v0.18.0. v0.17.0 shipped 2026-07-26 at commit `ec023ee`.
- **Terminal state:** zero `pub fn selected(&self) -> Option<usize>` in `src/component/`; audit scorecard stays 9/9; the Known Deferred Findings block loses its Cadence D entry.

## Scope survey (verified against the tree at `ec023ee`)

The prior audit described this backlog as "~15 components with a `selected()` alias". That framing is imprecise: the 15 sites split into **three materially different categories**, and only 12 are actually aliases.

### Category A — literal aliases (12 components)

Body is exactly `self.selected_index()`. Pure redundancy; delete outright.

| Component | `selected()` site | Has `set_selected`? |
|---|---|---|
| `loading_list` | `loading_list/mod.rs:357` | yes — `mod.rs:417` |
| `select` | `select/mod.rs:222` | yes — `mod.rs:255` |
| `radio_group` | `radio_group/mod.rs:199` | yes — `mod.rs:233` |
| `accordion` | `accordion/mod.rs:362` | **no** |
| `dropdown` | `dropdown/mod.rs:234` | yes — `mod.rs:267` |
| `selectable_list` | `selectable_list/mod.rs:242` | yes — `mod.rs:305` |
| `menu` | `menu/mod.rs:322` | yes — `mod.rs:345` |
| `metrics_dashboard` | `metrics_dashboard/mod.rs:359` | yes — `mod.rs:380` |
| `tabs` | `tabs/mod.rs:164` | yes — `mod.rs:198` |
| `tree` | `tree/mod.rs:528` | yes — `mod.rs:550` |
| `searchable_list` | `searchable_list/mod.rs:322` | yes — `mod.rs:361` |
| `file_browser` | `file_browser/mod.rs:446` | **no** |

### Category B — primary accessors (3 components)

Body is `self.selected` (a direct field read). These components have **no** `selected_index()` sibling, so there is no redundancy to remove — renaming them is a pure naming-consistency change.

| Component | `selected()` site | Has `set_selected`? |
|---|---|---|
| `diagram` | `diagram/state.rs:263` | **no** |
| `alert_panel` | `alert_panel/mod.rs:347` | **no** |
| `multi_progress` | `multi_progress/mod.rs:434` | yes — `mod.rs:466` |

**Decision: rename Category B to `selected_index()`** rather than leaving them. Leaving them would end Cadence D with 3 components exposing `selected()` while 12 expose `selected_index()` — the exact inconsistency this cadence exists to eliminate, and a guaranteed re-flag at the next audit. Approved during brainstorm.

### Setter-symmetry consequence (the Cadence A trap, pre-empted)

The audit tool's accessor-symmetry check (`tools/audit/src/code_analysis.rs:238-266`) requires every `set_X()` to have a matching getter named `X`, `is_X`, or `X_value`. Deleting or renaming `selected()` orphans `set_selected()` and regresses the scorecard from 9/9 to 8/9.

Cadence A hit exactly this mid-implementation: Task 4's verification gauntlet caught it after Task 1's alias deletions, forcing an unplanned fix commit. **Cadence D pre-empts it** by renaming the setters in the same commits as the getter changes.

**11 setters need renaming** — all with the identical signature `pub fn set_selected(&mut self, index: Option<usize>)`:

10 from Category A (`loading_list`, `select`, `radio_group`, `dropdown`, `selectable_list`, `menu`, `metrics_dashboard`, `tabs`, `tree`, `searchable_list`) plus `multi_progress` from Category B.

`accordion`, `file_browser`, `diagram`, and `alert_panel` have no setter — getter change only.

### Explicitly out of scope

Two components expose a `selected()` whose return type makes it a genuinely different concept, not an index accessor:

- **`heatmap::selected(&self) -> Option<(usize, usize)>`** at `heatmap/mod.rs:449` — returns cursor *coordinates*, not an index. Already established as out-of-scope during Cadence A, whose companion rename (`selected_value` → `value_at_selection`) exists precisely to disentangle heatmap's value accessor from the selection-accessor pattern. Unchanged.
- **`box_plot::selected(&self) -> usize`** at `box_plot/state.rs:249` — returns a bare `usize`, not `Option<usize>`. Normalizing it would require deciding whether the return type becomes `Option<usize>` (a semantic change about whether "no selection" is representable), which is a different question from naming consistency. Deferred; noted in the CHANGELOG as a known remaining item.

### Verification that Cadence A holds

`table`, `data_grid`, and `tab_bar` each have zero `pub fn selected(&self) -> Option<usize>` — confirmed at `ec023ee`. Cadence A's work is intact and is not re-touched here.

### Migration surface

209 call sites across the tree:

| Area | `.selected()` | `.set_selected(` |
|---|---|---|
| `src/component/` | 112 | 86 |
| `tests/` | 11 | 0 |
| `src/app/`, `src/harness/`, `examples/`, `benches/` | 0 | 0 |

Note that the 112 `src/component/` `.selected()` hits include calls on `heatmap` and `box_plot` (out of scope) — the implementer must migrate only in-scope call sites. Per-component grep gates in the plan handle this.

## Cadence structure

Standard 4-PR pattern, matching Cadence A:

1. Spec PR (this document)
2. Plan PR
3. Impl PR — three signed commits (Unit 1 / Unit 2 / Unit 3)
4. Tracking-doc PR — closure record at `docs/audits/2026-07-26-post-cadence-d.md`

Full ceremony is warranted: 15 breaking renames across 15 components, ~209 call sites, and a real design decision (Category B). Cadence A was the same magnitude and its adversarial spec review surfaced four must-fix design bugs — worth the gate.

## Unit 1 — Category A: delete 12 aliases, rename 10 setters

Files touched: the 12 components' `mod.rs` files, their `tests.rs`/`tests/` files, plus any in-tree caller.

### Per-component change

For each of the 12 Category A components:

- **Delete** `pub fn selected(&self) -> Option<usize> { self.selected_index() }` and its docstring.
- **Keep** `selected_index()` unchanged — it is already the canonical accessor.
- **Rename** `set_selected` → `set_selected_index` where present (10 of 12), updating the docstring to reference `selected_index()` as its getter counterpart and noting the rename with a MIGRATION.md pointer.

### Docstring convention for renamed setters

Each renamed setter gains a line matching the Cadence A precedent:

```rust
/// Renamed from `set_selected()` in v0.18.0 for symmetry with
/// [`selected_index()`](Self::selected_index). See MIGRATION.md.
```

### Migration table (rendered in Unit 3)

```markdown
| Component | Old | New |
|---|---|---|
| `accordion` | `state.selected()` | `state.selected_index()` |
| `dropdown` | `state.selected()` | `state.selected_index()` |
| `dropdown` | `state.set_selected(i)` | `state.set_selected_index(i)` |
| `file_browser` | `state.selected()` | `state.selected_index()` |
| `loading_list` | `state.selected()` | `state.selected_index()` |
| `loading_list` | `state.set_selected(i)` | `state.set_selected_index(i)` |
| `menu` | `state.selected()` | `state.selected_index()` |
| `menu` | `state.set_selected(i)` | `state.set_selected_index(i)` |
| `metrics_dashboard` | `state.selected()` | `state.selected_index()` |
| `metrics_dashboard` | `state.set_selected(i)` | `state.set_selected_index(i)` |
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
```

## Unit 2 — Category B: rename 3 primary accessors, 1 setter

Files touched: `diagram/state.rs`, `alert_panel/mod.rs`, `multi_progress/mod.rs`, plus their tests and callers.

### Per-component change

- **`diagram`** (`state.rs:263`): rename `selected()` → `selected_index()`. Body unchanged (`self.selected`). No setter.
- **`alert_panel`** (`mod.rs:347`): rename `selected()` → `selected_index()`. Body unchanged. No setter.
- **`multi_progress`** (`mod.rs:434`): rename `selected()` → `selected_index()`. Body unchanged. **Also** rename `set_selected` → `set_selected_index` (`mod.rs:466`).

The private field `selected` on each state struct is **not** renamed — it is private, carries no ambiguity with the public method name once the method is `selected_index()`, and renaming it would inflate the diff without benefit. This differs from Cadence A's `data_grid::selected_row` field rename, which was necessary there specifically to break a *grep collision* with a deleted method name of the same spelling. No such collision exists here.

### Docstring convention

Each renamed getter gains:

```rust
/// Renamed from `selected()` in v0.18.0 for consistency with the
/// `selected_index()` accessor used across every other component.
/// See MIGRATION.md.
```

### Migration table addition (rendered in Unit 3)

```markdown
| `alert_panel` | `state.selected()` | `state.selected_index()` |
| `diagram` | `state.selected()` | `state.selected_index()` |
| `multi_progress` | `state.selected()` | `state.selected_index()` |
| `multi_progress` | `state.set_selected(i)` | `state.set_selected_index(i)` |
```

## Unit 3 — prelude carry-over + CHANGELOG + MIGRATION

Files touched: `src/lib.rs`, `CHANGELOG.md`, `MIGRATION.md`.

### 3a. Prelude re-export of the MessageSender error types

Cadence A's final whole-branch review raised this as Minor #3: `MessageSendError` and `TrySendError` are re-exported at the crate root (`src/lib.rs:408`) but **not** in the prelude (`src/lib.rs:478` re-exports only `AppHarness`, `MessageSender`, `TestHarness`). A consumer doing `use envision::prelude::*` who wants to `match` on `TrySendError::{Full, Closed}` or destructure `MessageSendError(msg)` must add an explicit second import.

Extend the prelude line at `src/lib.rs:478`:

```rust
pub use crate::harness::{
    AppHarness, MessageSendError, MessageSender, TestHarness, TrySendError,
};
```

Non-breaking, additive.

### 3b. CHANGELOG

v0.17.0 shipped, so the `[Unreleased]` block was renamed to `[0.17.0]` during the release. Unit 3 creates a **fresh `[Unreleased]` block** above it, containing:

- `### Breaking Changes` → `#### Selection accessors completed on selected_index()` — narrative covering both units, cross-referencing the MIGRATION.md table.
- `### Added` → `#### MessageSendError + TrySendError in prelude`.
- `### Known Deferred Findings` — **rewritten**. The Cadence D entry is removed (this cadence closes it). What remains:
  - `box_plot::selected() -> usize` non-Option shape — needs a semantic decision about representability of "no selection" before it can be normalized.
  - `compact_str` sporadic adoption (3 files) — needs a commit-or-drop decision.
  - The N3–N7 cosmetic items from the 2026-07-05 audit (naming outliers `is_checked` / `label_text`, `restore_terminal` → `restore`, `AppShell` README placement, files near the 1000-line cap, snapshot-coverage concentration).

This also satisfies Cadence A final-review Minor #2, which asked that the deferred-findings block name the setter renames explicitly rather than only the alias removal — moot once the block's Cadence D entry is deleted, but the rewrite makes the remaining scope precise.

### 3c. MIGRATION.md

New `## v0.17.x to v0.18.0` section inserted **above** the existing `## v0.16.x to v0.17.0`, containing:

- `### Selection accessors completed on selected_index()` — the combined 26-row table from Units 1 and 2, alphabetized by component.
- A grep hint for consumers, in the style of the v0.17 section:

  ```
  Search your codebase for `.selected()` and `.set_selected(` on any of the
  15 components listed below. Note that `heatmap::selected()` (returns
  `Option<(usize, usize)>` coordinates) and `box_plot::selected()` (returns
  bare `usize`) are unchanged and should NOT be migrated.
  ```

- `### MessageSendError + TrySendError available from the prelude` — a short note that the explicit `use envision::{MessageSendError, TrySendError};` import is no longer required.

## Testing strategy

- Every test referencing a renamed accessor migrates mechanically.
- Any test that exists *purely* to assert alias-equivalence (`assert_eq!(state.selected(), state.selected_index())`) is **deleted**, not renamed — post-deletion it becomes `assert_eq!(x, x)`. This is the Cadence A A2 lesson.
- Doc-tests on the renamed methods update in place.

### Grep gates (callsite form, per Cadence A's M1/M2 lesson)

Token-boundary greps collide with private fields named `selected` and with out-of-scope components. All gates use callsite form and are scoped:

```bash
# In-scope components must have zero `.selected()` and zero `.set_selected(`
grep -rn '\.selected()' \
  src/component/{loading_list,select,radio_group,accordion,dropdown,selectable_list,menu,metrics_dashboard,tabs,tree,searchable_list,file_browser,diagram,alert_panel,multi_progress}/
# expect: zero hits

grep -rn '\.set_selected(' src/ tests/ examples/ benches/
# expect: zero hits (no component retains the old setter name)

# Out-of-scope components must be UNCHANGED
grep -c 'pub fn selected(&self) -> Option<(usize, usize)>' src/component/heatmap/mod.rs   # expect 1
grep -c 'pub fn selected(&self) -> usize' src/component/box_plot/state.rs                 # expect 1

# Whole-tree: no component exposes the index-shaped alias any more
grep -rn 'pub fn selected(&self) -> Option<usize>' src/component/
# expect: zero hits
```

### Full gauntlet (unchanged from prior cadences)

- `cargo fmt --check`
- `cargo clippy --all-features -- -D warnings`
- `cargo nextest run --all-features`
- `cargo test --all-features --doc`
- `cargo build --no-default-features`
- `cargo test --no-default-features --no-run` (D8 lesson)
- `cargo build --examples --all-features`
- `cargo doc --no-deps --all-features` — zero intra-doc-link warnings
- `./tools/audit/target/release/envision-audit all` — **scorecard 9/9**, "Accessor Symmetry: All setters have matching getters"

## Success criteria

1. `grep -rn 'pub fn selected(&self) -> Option<usize>' src/component/` returns zero hits.
2. `grep -rn '\.set_selected(' src/ tests/ examples/ benches/` returns zero hits.
3. `heatmap::selected() -> Option<(usize, usize)>` and `box_plot::selected() -> usize` unchanged.
4. Audit scorecard 9/9 with zero accessor-symmetry gaps — verified *within* each unit's commit, not deferred to a final gauntlet (the Cadence A lesson).
5. `envision::prelude::*` exposes `MessageSendError` and `TrySendError`.
6. CHANGELOG has a fresh `[Unreleased]` block whose Known Deferred Findings no longer lists Cadence D.
7. MIGRATION.md has a `## v0.17.x to v0.18.0` section with the 26-row table and the grep hint.
8. Full verification gauntlet clean.

## Risk register

- **Setter-symmetry regression mid-cadence.** This is the known Cadence A failure mode. Mitigated by renaming each setter in the *same commit* as its getter change, and by running the audit tool at the end of Unit 1 and Unit 2 rather than only in a final gauntlet task.
- **Over-migrating out-of-scope call sites.** 112 of the `.selected()` hits are in `src/component/`, and some belong to `heatmap` / `box_plot`. Mitigated by per-component scoped greps rather than a tree-wide search-and-replace, and by explicit "expect 1" gates on the two out-of-scope signatures.
- **Private field named `selected` confusing a naive grep.** Category B bodies read `self.selected`. A token-boundary grep for `selected` will hit these legitimately. Mitigated by callsite-form gates (`\.selected()`) throughout.
- **`diagram`'s accessor lives in `state.rs`, not `mod.rs`.** The audit tool parses `mod.rs` preferentially for some checks. `diagram` has no setter, so there is no symmetry risk, but the implementer must not assume every component keeps its accessors in `mod.rs`. File paths are enumerated per-component in this spec and will be repeated in the plan.
- **Large mechanical diff (~209 call sites).** Volume raises the chance of a missed site. Mitigated by the compiler (these are breaking renames — a missed site fails to build) plus the grep gates.
- **Tautology tests.** Components with both `selected()` and `selected_index()` may have tests asserting they agree. Those must be deleted, not renamed. Enumerated at implementation time via `grep -rn 'assert_eq!(.*selected(),.*selected_index()' src/component/`.

## Open questions

None. Decisions resolved during the 2026-07-26 brainstorm:

- **Category B treatment:** rename to `selected_index()` (chosen over leaving them, which would guarantee a Cadence E).
- **`box_plot` normalization:** out of scope — the non-`Option` return type raises a semantic question distinct from naming.
- **Prelude carry-over:** included (Cadence A final-review Minor #3).
- **Known Deferred Findings rewrite:** included (Cadence A final-review Minor #2).
- **Ceremony:** full 4-PR cadence, matching Cadence A's magnitude.

## Reference

- **Cadence A** — spec PR #506 (`cfb7cec`), plan PR #507 (`70c44bf`), impl PR #508 (`09608d6`), tracking PR #509 (`4f3bd06`). Closure record: [`docs/audits/2026-07-05-post-consistency-cleanup.md`](../../audits/2026-07-05-post-consistency-cleanup.md).
- **Final pre-v0.17.0 audit** — `A` (3.95 GPA), [`docs/audits/2026-07-25-pre-v0.17.0-release.md`](../../audits/2026-07-25-pre-v0.17.0-release.md). Names the Cadence D backlog as the largest remaining consistency item.
- **Delete-outright precedent (pre-1.0):** D5 `paragraph`→`line`, D14, G7, D12, D3, D8, `resource_gauge::new`, `FileSortDirection`, and Cadence A's own ten renames. No `#[deprecated]` shims.
- **Cadence A lessons carried forward:** callsite-form grep gates (M1/M2); tautology tests deleted not renamed (A2); setter symmetry fixed in the same commit as the getter change (the Task 4 fix, promoted here to a design constraint).
