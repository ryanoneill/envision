# Cadence D — selection-accessor completion (v0.18.0) — design

## Purpose

Finish the selection-accessor unification that Cadence A began. Cadence A canonicalized `selected_item()` / `selected_index()` / `set_selected_index()` across six components (`dropdown`, `select`, `heatmap`, `tab_bar`, `data_grid`, `table`) but deliberately scoped out the remaining components carrying a `selected()` accessor. That deferral is documented in `CHANGELOG.md:188-192` and in the Cadence A closure record as "Cadence D (v0.18+)".

**Terminal state:** every component whose selection accessor is *spelled* `selected` now spells it `selected_index`. Concretely — after this cadence, `grep -rn 'pub fn selected(&self)' src/component/` returns exactly one hit: `heatmap`, whose accessor returns coordinates rather than an index and is a genuinely different concept.

That is a narrower and truer claim than "every component exposing a selection index calls it `selected_index()`", which is false — see [Surveyed and deliberately excluded](#surveyed-and-deliberately-excluded).

- **Prior cadence:** Cadence A (spec PR #506, plan PR #507, impl PR #508, tracking PR #509), closure record at [`docs/audits/2026-07-05-post-consistency-cleanup.md`](../../audits/2026-07-05-post-consistency-cleanup.md).
- **Release target:** v0.18.0. `Cargo.toml` is at `0.17.0`; this cadence is breaking; pre-1.0 semver → minor bump.

## Why `selected_index` is the right destination name

The spec's first draft justified the destination purely by "consistency." That is the weaker case and invites the obvious counter: `selected()` is shorter, `_index` is redundant when the return type already says `Option<usize>`, and Rust convention favors terseness (`Vec::len`, not `Vec::element_count`).

The real argument is that **`selected` is ambiguous about return type**, and this codebase proves it. Four different signatures share the spelling today:

| Signature | Where |
|---|---|
| `selected(&self) -> Option<usize>` | 15 components (this cadence) |
| `selected(&self) -> Option<(usize, usize)>` | `heatmap/mod.rs:449` — coordinates |
| `selected(&self) -> usize` | `box_plot/state.rs:249` |
| `selected(mut self, selected: bool) -> Self` | `annotation/widget.rs:75` — a *builder* |

`selected_index` / `selected_item` / `value_at_selection` is a system where **the name predicts the type**. Across a 74-component library consumed by downstream apps, that predictability is worth more than four characters.

## Scope survey (verified against the tree at `ec023ee`)

The inherited backlog description (`CHANGELOG.md:192`) calls all ~15 sites "literal aliases for `selected_index()`". That is flatly wrong for four of them, which have no `selected_index()` sibling at all. The sites split into **three in-scope categories plus one exclusion**.

### Category A — literal aliases (12 components)

Body is exactly `self.selected_index()`. Delete outright.

| Component | `selected()` site | `set_selected` site |
|---|---|---|
| `accordion` | `accordion/mod.rs:362` | — |
| `dropdown` | `dropdown/mod.rs:234` | `mod.rs:267` |
| `file_browser` | `file_browser/mod.rs:446` | — |
| `loading_list` | `loading_list/mod.rs:357` | `mod.rs:417` |
| `menu` | `menu/mod.rs:322` | `mod.rs:345` |
| `metrics_dashboard` | `metrics_dashboard/mod.rs:359` | `mod.rs:380` |
| `radio_group` | `radio_group/mod.rs:199` | `mod.rs:233` |
| `searchable_list` | `searchable_list/mod.rs:322` | `mod.rs:361` |
| `select` | `select/mod.rs:222` | `mod.rs:255` |
| `selectable_list` | `selectable_list/mod.rs:242` | `mod.rs:305` |
| `tabs` | `tabs/mod.rs:164` | `mod.rs:198` |
| `tree` | `tree/mod.rs:528` | `mod.rs:550` |

**Caveat on "pure redundancy":** for `accordion` this is a layer shallower than it looks. `accordion/mod.rs:320-323` documents `selected_index()` as a convenience alias for `focused_index()` (`mod.rs:302`, returns bare `usize`), Option-normalizing the empty case at `mod.rs:339-343`. So accordion has a chain `selected()` → `selected_index()` → `focused_index()`. Deleting the outer layer is still correct — `selected_index()` does real type-normalization work, so it is not itself a byte-identical alias — but this cadence does **not** touch `focused_index()`, and the spec should not oversell the deletion as removing all indirection.

### Category B — primary accessors (3 components)

Body is `self.selected` (a direct field read). **No** `selected_index()` sibling, so there is no redundancy to remove — renaming is a pure naming-consistency change.

| Component | `selected()` site | `set_selected` site |
|---|---|---|
| `alert_panel` | `alert_panel/mod.rs:347` | — |
| `diagram` | `diagram/state.rs:263` | — |
| `multi_progress` | `multi_progress/mod.rs:434` | `mod.rs:466` |

**Decision: rename to `selected_index()`.** Leaving them would end Cadence D with 3 components on `selected()` and 12 on `selected_index()` — the exact inconsistency this cadence exists to eliminate.

### Category C — `box_plot` (1 component)

`box_plot/state.rs:249` — `pub fn selected(&self) -> usize`, with `set_selected(&mut self, index: usize)` at `state.rs:267`.

**Decision: in scope.** The first draft excluded it, arguing that normalizing the bare-`usize` return would force a semantic decision about representing "no selection". That was a false dilemma, for two reasons:

1. **A bare-`usize` `selected_index()` is already shipped and canonical.** `flame_graph/mod.rs:369` is `pub fn selected_index(&self) -> usize`. So `box_plot::selected() -> selected_index()` is a *pure rename requiring zero semantic decision*. The `Option`-shape question is genuinely separate and stays deferred — under the new name.
2. **`CHANGELOG.md:192` names `box_plot` by name** in the published Cadence D backlog. Deferring the one component the public CHANGELOG promised would repeat the Cadence A M3 failure — a "sweep" that leaves the identical divergence in place — and would leave `box_plot` as the *only* component in the crate still pairing `selected()` with `set_selected()`: a strictly worse outlier after the sweep than before it.

Rename `selected` → `selected_index` and `set_selected` → `set_selected_index`, **both keeping `usize`**.

### Surveyed and deliberately excluded

**`heatmap::selected(&self) -> Option<(usize, usize)>`** (`heatmap/mod.rs:449`) — returns cursor *coordinates*, not an index. Already excluded during Cadence A, whose companion rename (`selected_value` → `value_at_selection`, `CHANGELOG.md:65`) exists precisely to disentangle heatmap's accessors from the selection-index pattern. Unchanged, and its 35 call sites must be left alone.

**Selection-index accessors under different names — surveyed, out of scope, no action:**

| Site | Signature |
|---|---|
| `chart/state.rs:111` | `active_series(&self) -> usize` |
| `log_correlation/mod.rs:504` | `active_stream(&self) -> usize` |
| `diff_viewer/mod.rs:442` | `current_hunk(&self) -> usize` |
| `step_indicator/state.rs:262` | `active_step_index(&self) -> Option<usize>` |
| `paginator/mod.rs:270` | `current_page(&self) -> usize` |
| `breadcrumb/mod.rs:321` | `focused_index(&self) -> usize` |

These are genuinely index-shaped and arguably belong in a naming-consistency sweep — Cadence A established the precedent by renaming `tab_bar::active_tab()` → `selected_item()` as a "semantic outlier" (`CHANGELOG.md:67`). They are excluded here because each carries **domain meaning the generic name would erase**: `current_page` is pagination state, `active_step_index` is workflow position, `current_hunk` is diff navigation. Collapsing them to `selected_index` would be a semantic flattening, not a consistency win — a different judgment call than the mechanical rename this cadence performs.

Recorded in the CHANGELOG's deferred block as an open question for a future cadence, so the next audit does not read this cadence as claiming they were missed.

### Verification that Cadence A holds

`grep -rn 'pub fn selected(' src/component/{table,data_grid,tab_bar}/` → zero hits at `ec023ee`. Cadence A's work is intact and is not re-touched.

## Setter symmetry: pre-empted as a design constraint

The audit tool's accessor-symmetry check (`tools/audit/src/code_analysis.rs:238-266`) requires every `set_X()` to have a getter named `X`, `is_X`, or `X_value`. Deleting or renaming `selected()` orphans `set_selected()`.

Cadence A hit exactly this mid-implementation — its Task 4 gauntlet caught the 9/9 → 8/9 regression after Task 1's deletions, forcing an unplanned fix commit and a mid-flight decision. The Cadence A closure record predicted the recurrence by name (`docs/audits/2026-07-05-post-consistency-cleanup.md:89`).

**Cadence D pre-empts it:** every affected setter renames in the *same commit* as its getter change, and the audit tool runs at the end of **each unit** rather than only in a final gauntlet task.

**12 setters** need renaming — 10 from Category A, plus `multi_progress` (B) and `box_plot` (C). Categories A and B use the identical signature `(&mut self, index: Option<usize>)`; `box_plot` uses `(&mut self, index: usize)`.

`accordion`, `file_browser`, `alert_panel`, and `diagram` have no setter — getter change only.

### Audit-tool blind spot (relevant to what the scorecard can prove)

`tools/audit/src/scorecard.rs:275-295` (`read_non_test_sources`) reads a component's `mod.rs` plus only those sibling files named by a `pub mod X;` or `pub use X::…` line. Files behind a **private** `mod X;` are invisible to every scorecard check.

Verified: `diagram/mod.rs:61`, `box_plot/mod.rs:31`, and `table/mod.rs:59` are all `mod state;` — private. So `diagram/state.rs` and `box_plot/state.rs` contribute nothing to the current 9/9 scorecard or to the `100.0% (1777/1777)` doctest-coverage metric.

Two consequences the implementer must hold:

1. **"Scorecard stays 9/9" is not evidence that `box_plot` is fine.** Its `selected`/`set_selected` pair was never measured. We bring it in scope for real consistency, not to satisfy the tool.
2. **The symmetry-regression prediction remains correct**, because all 12 setters that matter to the tool live in `mod.rs`. Verified individually.

## Cadence structure

Standard 4-PR pattern, matching Cadence A:

1. Spec PR (this document)
2. Plan PR
3. Impl PR — three signed commits (Unit 1 / Unit 2 / Unit 3)
4. Tracking-doc PR — closure record at `docs/audits/2026-07-26-post-cadence-d.md`

## Unit 1 — Category A: delete 12 aliases, rename 10 setters

For each of the 12 Category A components:

- **Delete** `pub fn selected(&self) -> Option<usize> { self.selected_index() }` and its docstring.
- **Keep** `selected_index()` unchanged.
- **Rename** `set_selected` → `set_selected_index` where present (10 of 12), updating the docstring.

### Docstring convention

Matching the *actual* Cadence A precedent at `table/state.rs:422-423` — plain backticks, no intra-doc link:

```rust
/// Renamed from `set_selected()` in v0.18.0 for symmetry with
/// `selected_index()`. See MIGRATION.md.
```

An intra-doc link form (`[`selected_index()`](Self::selected_index)`) would be nicer in rustdoc, but adopting it here would create a *new* inconsistency with the three components Cadence A already shipped unless those were retrofitted too. Out of scope; keep the shipped form.

## Unit 2 — Categories B and C: 4 getter renames, 2 setter renames

- **`alert_panel`** (`mod.rs:347`): `selected()` → `selected_index()`. Body unchanged. No setter.
- **`diagram`** (`state.rs:263` — note: *not* `mod.rs`): `selected()` → `selected_index()`. Body unchanged. No setter.
- **`multi_progress`** (`mod.rs:434`): `selected()` → `selected_index()`. Body unchanged. Also `set_selected` → `set_selected_index` (`mod.rs:466`).
- **`box_plot`** (`state.rs:249`, `state.rs:267` — note: *not* `mod.rs`): `selected() -> usize` → `selected_index() -> usize`; `set_selected(usize)` → `set_selected_index(usize)`. **Return types unchanged** — bare `usize` matches the shipped `flame_graph::selected_index()` precedent.

### Private fields are NOT renamed

Category B and C bodies read a private field named `selected`. Leave it. This differs from Cadence A's `data_grid::selected_row` field rename, which was necessary there specifically to break a **grep collision** with a deleted method of the same spelling (commit `09608d6` states this reason). No such collision exists here: `self.selected` is a field read, invisible to the callsite-form (`\.selected()`) gates this cadence uses. `self.selected` behind `pub fn selected_index()` is ordinary Rust.

### Docstring convention

```rust
/// Renamed from `selected()` in v0.18.0 for consistency with the
/// `selected_index()` accessor used across every other component.
/// See MIGRATION.md.
```

## Unit 3 — prelude carry-over + CHANGELOG + MIGRATION

### 3a. Prelude harness re-exports

Sanctioned by the Cadence A closure record, which parks this item in Cadence D (`docs/audits/2026-07-05-post-consistency-cleanup.md:100`) — it is bundled by prior agreement, not smuggled in.

Cadence A's final review raised only `MessageSendError` + `TrySendError`. Fixing 2 of 4 would recreate the inconsistency one level down: the crate root (`src/lib.rs:408-410`) exports **seven** harness types while the prelude (`src/lib.rs:478`) re-exports **three**.

| Crate root | In prelude today |
|---|---|
| `AppHarness` | ✅ |
| `MessageSender` | ✅ |
| `TestHarness` | ✅ |
| `Assertion` | ❌ |
| `MessageSendError` | ❌ |
| `Snapshot` | ❌ |
| `TrySendError` | ❌ |

**Add all four.** Non-breaking, additive:

```rust
pub use crate::harness::{
    AppHarness, Assertion, MessageSendError, MessageSender, Snapshot, TestHarness, TrySendError,
};
```

### 3b. CHANGELOG

There is currently **no** `[Unreleased]` block — it was renamed to `[0.17.0]` during the v0.17.0 release. Unit 3 creates a fresh one above `## [0.17.0] - 2026-07-26`.

**The `[0.17.0]` section is released and tagged; leave it byte-identical.** Keep a Changelog treats released sections as immutable, and mutating it would rewrite history a consumer may already have read. The existing Known Deferred Findings block at `CHANGELOG.md:188` stays exactly as shipped.

The new `[Unreleased]` block contains:

- `### Breaking Changes` → `#### Selection accessors completed on selected_index()` — narrative covering all three units, cross-referencing the MIGRATION table.
- `### Added` → `#### Harness types available from the prelude` — the four additions.
- `### Known Deferred Findings` — a **new** block that opens with an explicit supersedes pointer:

  > Supersedes the Known Deferred Findings block under `[0.17.0]`. The Cadence D item listed there (selection-accessor aliases, including `box_plot`) is **closed** by this release.

  Remaining items:
  - **Selection-index accessors under domain-specific names** — `chart::active_series`, `log_correlation::active_stream`, `diff_viewer::current_hunk`, `step_indicator::active_step_index`, `paginator::current_page`, `breadcrumb::focused_index`. Surveyed during Cadence D and deliberately excluded: each name carries domain meaning a generic rename would erase. Open question for a future cadence.
  - **`box_plot::selected_index() -> usize` returns a bare `usize`**, so "no selection" is unrepresentable. Whether it should become `Option<usize>` is a semantic question, deferred. (The *naming* half closed in this release.)
  - **`accordion::selected_index()` remains a convenience alias for `focused_index()`.** Type-normalizing, not redundant, but the indirection stands.
  - **`compact_str` adoption is sporadic** — 2 non-test source files (`src/component/cell.rs`, `src/backend/cell/mod.rs`). Needs a commit-or-drop decision.
  - **N3 (partially closed)** — the `tab_bar` setter-naming outlier is resolved by this cadence; the residue is `is_checked` and `label_text`. Plus N4–N7: `restore_terminal` → `restore`, `AppShell` README placement, five files near the 1000-line cap, snapshot-coverage concentration.

  N2 (the README `version = "0.17"` pin) is **not** listed — it is a release-checklist item handled during `/release`, not a code deferral. It becomes `version = "0.18"` when v0.18.0 ships.

### 3c. MIGRATION.md

New `## v0.17.x to v0.18.0` section inserted above `## v0.16.x to v0.17.0`, containing:

- `### Selection accessors completed on selected_index()` — a **28-row** table (16 getter changes + 12 setter renames), alphabetized by component.
- A grep hint, extending the v0.17 section's style:

  ```
  Search for `.selected()` and `.set_selected(` on the 16 components below.

  `heatmap::selected()` returns `Option<(usize, usize)>` coordinates and is
  UNCHANGED — do not migrate it.

  Because these are renames, `cargo build` identifies every site and the
  compiler error names the receiver type. Note that method references
  (`SelectState::selected`, or `.map(SelectState::selected)`) do not match a
  `.selected()` grep but will still fail to compile.
  ```

- `### Harness types available from the prelude` — a short note that the explicit `use envision::{Assertion, MessageSendError, Snapshot, TrySendError};` import is no longer required.

**Archiving policy note:** PR #510 set a last-3-versions boundary for MIGRATION.md. Adding a fourth section takes the file to ~254 lines — nowhere near the 1000-line cap — so the policy is **waived this cycle**. Archive `v0.14.x → v0.15.0` at v0.19.0.

## Rejected alternative: `#[deprecated]` shims

The pre-v0.17.0 audit recommends the opposite of what this spec does (`docs/audits/2026-07-25-pre-v0.17.0-release.md:106`):

> Add `#[deprecated(note = "use selected_index()/selected_item()")]` to the 17 `selected()` aliases (Cadence D) — turns the backlog into a compiler-guided consumer migration.

**This spec rejects that recommendation**, and records the rejection rather than diverging silently:

- Deprecation costs consumers **two upgrade hops** for a pure rename.
- The fix is mechanically obvious from the compiler error — rustc emits `no method named 'selected' … did you mean 'selected_index'?` for exactly this shape.
- Deprecation shims earn their keep for *semantic* changes, where the old call still compiles but means something subtly different. A rename is not that.
- Delete-outright is the established pre-1.0 precedent across D5, D14, G7, D12, D3, D8, `resource_gauge::new`, `FileSortDirection`, and Cadence A's own ten renames.

## Testing strategy

- Every test referencing a renamed accessor migrates mechanically.
- Tests that exist *purely* to assert alias-equivalence are **deleted**, not renamed — post-deletion they become `assert_eq!(x, x)`. This is the Cadence A A2 lesson. Two exist: `accordion/tests.rs:798` and `file_browser/helper_tests.rs:260`. Three more live in doctests (`dropdown/mod.rs:232`, `metrics_dashboard/mod.rs:357`, `searchable_list/mod.rs:320`) and die with their methods. **Note the non-standard filename `helper_tests.rs`** — scoping a search to `tests.rs` would miss it.

### Migration surface (in-scope vs out-of-scope)

The first draft cited "209 call sites" without separating out-of-scope hits. Corrected:

| Area | Total `.selected()` | In scope | Out of scope |
|---|---|---|---|
| `src/component/` (16 in-scope dirs) | 83 | **83** | — |
| `src/component/heatmap/` | 29 | — | 29 |
| `tests/integration_stress.rs` | 5 | **5** (`DiagramState`) | — |
| `tests/integration_new_components.rs` | 6 | — | 6 (`HeatmapState`) |
| **`.selected()` subtotal** | 123 | **88** | 35 |
| `.set_selected(` (all in `src/component/`) | 86 | **86** | — |
| **Total** | **209** | **174** | **35** |

The per-component directory greps used as gates **cannot** scope `tests/`, since integration tests are not organized by component. A tree-wide `sed` over `tests/` would break the six heatmap assertions. Hence the explicit `tests/` gate below.

### Grep gates (callsite form, per Cadence A's M1/M2 lesson)

Token-boundary greps collide with the private `selected` fields that Category B and C bodies read. All gates use callsite form.

```bash
# No component exposes the index-shaped accessor any more.
grep -rn 'pub fn selected(&self)' src/component/
# expect: exactly 1 — heatmap/mod.rs:449 (Option<(usize, usize)>)

# Every setter renamed — box_plot is in scope, so this is now tree-wide and honest.
grep -rn '\.set_selected(' src/ tests/ examples/ benches/
# expect: zero hits

# In-scope components carry no `.selected()` call sites.
grep -rn '\.selected()' src/component/{accordion,alert_panel,box_plot,diagram,dropdown,file_browser,loading_list,menu,metrics_dashboard,multi_progress,radio_group,searchable_list,select,selectable_list,tabs,tree}/
# expect: zero hits

# Out-of-scope heatmap is UNCHANGED — these must NOT go to zero.
grep -rnc '\.selected()' src/component/heatmap/     # expect: 29
grep -rn '\.selected()' tests/                      # expect: 6, all HeatmapState in integration_new_components.rs
grep -c 'pub fn selected(&self) -> Option<(usize, usize)>' src/component/heatmap/mod.rs   # expect: 1
```

### Full gauntlet

- `cargo fmt --check`
- `cargo clippy --all-features -- -D warnings`
- `cargo nextest run --all-features`
- `cargo test --all-features --doc`
- `cargo build --no-default-features`
- `cargo test --no-default-features --no-run` (D8 lesson)
- `cargo build --examples --all-features`
- `cargo doc --no-deps --all-features` — zero intra-doc-link warnings
- `./tools/audit/target/release/envision-audit all` — **scorecard 9/9**, "Accessor Symmetry: All setters have matching getters", **doctest coverage still 100%**

## Success criteria

1. `grep -rn 'pub fn selected(&self)' src/component/` returns exactly one hit (heatmap).
2. `grep -rn '\.set_selected(' src/ tests/ examples/ benches/` returns zero hits.
3. `heatmap` unchanged: 29 `src/component/` call sites, 6 `tests/` call sites, signature intact.
4. `box_plot` renamed with return types unchanged (`usize`, not `Option<usize>`).
5. Audit scorecard 9/9 **with doctest coverage still at 100%** — verified *within* each unit's commit, not deferred to a final gauntlet.
6. `envision::prelude::*` exposes all seven harness types.
7. CHANGELOG has a fresh `[Unreleased]` block; the `[0.17.0]` block is byte-identical to what shipped.
8. MIGRATION.md has a `## v0.17.x to v0.18.0` section with the 28-row table and grep hint.
9. Full verification gauntlet clean.

## Risk register

- **Setter-symmetry regression mid-cadence.** The known Cadence A failure mode. Mitigated by renaming each setter in the same commit as its getter, and running the audit tool at the end of each unit.
- **Doctest-coverage regression — a second, independent way to drop the scorecard.** The scorecard gates doctest coverage at **100% (1777/1777)**, not a threshold, and counts a `pub fn` as covered only when a ` ``` ` fence appears in the `///` block immediately above it (`scorecard.rs`, `has_doc_test_above`). Inserting the "Renamed from…" line in a way that separates the doc block from its example fence — or dropping a fence while editing — silently drops coverage below 100% and fails the scorecard for a reason unrelated to symmetry. **Add the note adjacent to existing prose, never between the prose and the fence.** Each unit's audit run catches it.
- **Over-migrating heatmap.** 35 out-of-scope call sites, 6 of them in `tests/` where per-component directory scoping is structurally impossible. Mitigated by the explicit "expect 29 / expect 6" gates, which fail loudly on over-migration rather than silently passing.
- **`diagram` and `box_plot` accessors live in `state.rs`, not `mod.rs`** — and those files are invisible to the audit tool (private `mod state;`). The tool will not catch mistakes there. File paths are enumerated per-component here and repeated in the plan.
- **Large mechanical diff (~174 in-scope call sites).** These are breaking renames, so the compiler catches misses; grep gates back it up.
- **Tautology tests in a non-standard file.** `file_browser/helper_tests.rs:260` — a search scoped to `tests.rs` would miss it.

## Open questions

None. Decisions resolved during the 2026-07-26 brainstorm and the subsequent adversarial spec review:

- **Category B treatment:** rename to `selected_index()` (brainstorm).
- **`box_plot`:** brought in scope after review — the `flame_graph::selected_index() -> usize` precedent makes it a pure rename, and `CHANGELOG.md:192` publicly promised it.
- **Differently-named accessors** (`active_series`, `current_page`, …): surveyed, excluded with per-item reasoning, recorded as an open question.
- **Prelude scope:** all four missing harness types, not the two the Cadence A review named.
- **`#[deprecated]`:** rejected, with the audit recommendation cited and the reasoning recorded.
- **CHANGELOG mechanics:** `[0.17.0]` left immutable; new block supersedes with an explicit pointer.
- **Ceremony:** full 4-PR cadence.

## Reference

- **Cadence A** — spec PR #506 (`cfb7cec`), plan PR #507 (`70c44bf`), impl PR #508 (`09608d6`), tracking PR #509 (`4f3bd06`). Closure record: [`docs/audits/2026-07-05-post-consistency-cleanup.md`](../../audits/2026-07-05-post-consistency-cleanup.md).
- **Final pre-v0.17.0 audit** — `A` (3.95 GPA), [`docs/audits/2026-07-25-pre-v0.17.0-release.md`](../../audits/2026-07-25-pre-v0.17.0-release.md).
- **Delete-outright precedent (pre-1.0):** D5 `paragraph`→`line`, D14, G7, D12, D3, D8, `resource_gauge::new`, `FileSortDirection`, Cadence A's ten renames.
- **Cadence A lessons carried forward:** callsite-form grep gates that can actually pass (M1/M2); no out-of-scope exclusion that makes the sweep dishonest (M3); tautology tests deleted not renamed (A2); setter symmetry fixed in-commit (the Task 4 fix, promoted to a design constraint).
