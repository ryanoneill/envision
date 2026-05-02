# Phase 1 handoff — Table sort & cell unification

**Branch:** `table-sort-cell-unification`
**Final Phase 1 commit:** see `git log -1` (filled in after this commit lands)
**Spec PR:** #459 (merged) — `docs/superpowers/specs/2026-05-02-table-sort-cell-unification-design.md` lives on `main`
**Plan PR:** #460 (merged) — implementation plan with all 34 tasks
**Customer-feedback PR:** #458 (merged) — leadline feedback tracker the work derives from

## Phase 1 in one sentence

Phase 1 is purely additive: the new `Cell` / `SortKey` / `CellStyle` / `RowStatus` types ship at `src/component/cell.rs`, `Column` gains `default_sort`, `TableMessage` gains 8 new sort variants — all alongside the existing API. Nothing has been deleted yet.

## What landed (Tasks 1–12 + polish)

- **`src/component/cell.rs`** (653 lines, well under the 1000-line cap). New module with:
  - `Cell` struct: `text` / `style` / `sort_key` private fields, `new`, builders (`with_style`, `with_sort_key`, `with_text`), accessors (`text` / `style` / `sort_key`), typed constructors (`number` / `int` / `uint` / `bool` / `duration` / `datetime`), style flavours (`success` / `warning` / `error` / `muted`), and `From<&str>` / `From<String>` / `From<CompactString>` impls.
  - `SortKey` enum: 8 variants in spec order (`String`, `I64`, `U64`, `F64`, `Bool`, `Duration`, `DateTime`, `None`) with the **`DO NOT REORDER VARIANTS`** banner. `compare()` implements (1) None-policy first, (2) same-variant fast path, (3) cross-variant discriminant fallback. Uses `f64::total_cmp` (not `partial_cmp`).
  - `CellStyle` enum: `Default` (default), `Success`, `Warning`, `Error`, `Muted`, `Custom(Style)`.
  - `RowStatus` enum: `None` (default), `Healthy`, `Warning`, `Error`, `Unknown`, `Custom { symbol, color }`. `indicator()` returns `Option<(&'static str, Color)>`.
- **`src/component/table/types.rs`** modifications (additive):
  - `Column::default_sort: SortDirection` field, defaulting to `Ascending`. `with_default_sort` builder + `default_sort()` getter. `Debug` + `PartialEq` impls updated.
  - New `InitialSort { column: usize, direction: SortDirection }` struct (`Copy + Eq` + serde-gated).
  - 8 new `TableMessage` variants: `SortAsc`, `SortDesc`, `SortToggle`, `SortClear`, `RemoveSort`, `AddSortAsc`, `AddSortDesc`, `AddSortToggle`. Each tagged `#[allow(dead_code)]` and documented as "handler lands in Phase 2".
- **`src/component/table/mod.rs`**: a no-op match arm in `update()` that catches all 8 new variants (so the enum is exhaustive). **This arm gets replaced in Phase 2 Tasks 16–18**.
- **`src/lib.rs` + `src/component/mod.rs`**: re-export `InitialSort`. (See "known gotcha" below.)
- **`src/component/table/tests.rs`**: new tests for `Column::default_sort`, `with_default_sort` round-trip, and that all 8 new `TableMessage` variants exist + derive `Clone`/`Debug`/`PartialEq`.

What stays untouched in Phase 1 (deleted in Phase 2):
- `TableRow::cells() -> Vec<String>` (Phase 2 Task 14 changes the signature to `Vec<Cell>`).
- `Column::with_comparator` / `comparator()` / `SortComparator` / `numeric_comparator` / `date_comparator` (Phase 2 Task 15).
- `TableMessage::SortBy` / `AddSort` / `ClearSort` (Phase 2 Task 16).
- `src/component/resource_table/` (Phase 2 Task 21 — its `Cell` / `RowStatus` analogues are subsumed by the new `cell` module).

## Phase 2 starting point — Task 14

Change `TableRow::cells() -> Vec<String>` to `Vec<Cell>`, add `fn status(&self) -> RowStatus { RowStatus::None }` default. This is the atomic switch — every existing `impl TableRow` in the codebase, examples, doc tests, integration tests, and `TableState`'s sort path needs to flip. Expect substantial fan-out. The plan estimates Tasks 14–22 cover this; Tasks 23–28 do consumer migration, Tasks 29–34 are docs/audit/release.

## Verification gate (run at handoff time)

| Check | Phase 1 result |
|---|---|
| `cargo nextest run -p envision --all-features` | 7364/7364 PASS |
| `cargo test --doc -p envision --all-features` | 2586/0 PASS |
| `cargo clippy --all-features --all-targets -- -D warnings` | clean |
| `cargo fmt --check` | clean |
| `cargo build --examples --all-features` | clean |
| `wc -l src/component/cell.rs` | 653 |
| Audit scorecard | 8/9 — same as `main` (the 1 failing accessor-symmetry gap is pre-existing in `resource_table`, will be removed in Phase 2 Task 21) |

## Known gotchas / pickup notes for the fresh session

1. **`mod.rs` no-op arm.** `src/component/table/mod.rs` lines ~362–370 catch all 8 new variants with `{}`. Tasks 16–18 replace this with the real handlers (`apply_table_msg` updates). Don't delete the arm before then — clippy/exhaustiveness will catch it.
2. **Doc-test intra-doc-link brackets.** Some doc comments reference upcoming types (e.g. `TableState::with_initial_sorts`) that don't exist yet. Where rustdoc would have flagged broken intra-doc links, the brackets were dropped. **Restore the brackets in Phase 2 once the targets exist** — search for the affected getters (`Column::default_sort`, `InitialSort`) and re-bracket once `with_initial_sort` / `with_initial_sorts` land.
3. **`Cell` and `SortKey` are not yet re-exported at the crate root.** They live at `envision::component::cell::{Cell, SortKey}`. The doc tests import from there. Once `TableRow::cells()` returns `Vec<Cell>` (Task 14), add `Cell, SortKey` to the `pub use component::{...}` block in `src/lib.rs` and the `pub use` block in `src/component/mod.rs` so consumers can write `use envision::Cell;`. (`CellStyle` and `RowStatus` are already re-exported because `resource_table` referenced them.)
4. **Audit scorecard interpretation.** Phase 1 is at 8/9, identical to `main`. Phase 2 will temporarily *worsen* this (the unfinished migration may break doc-test coverage on `Cell` / `SortKey` accessors mid-flight). The 9/9 target applies to the **final** `pub use` shape after Task 28+. Don't panic if it dips during Phase 2.
5. **No `unsafe`, no `TODO`, no `unimplemented!`, no `panic!` introduced.** The only `#[allow(dead_code)]` markers are on the 8 new `TableMessage` variants (deliberate — handler lands in Phase 2) and one bare `#![allow(dead_code)]` at the top of `cell.rs` from Task 1 that **should be removed** in Phase 2 once `cell.rs` is wired into the sort path. Search for `Placeholder during Phase 1` in `cell.rs`.
6. **Commits are signed throughout** — keep that contract for Phase 2/3.

## Phase 1 commit history (oldest → newest)

```
f8a68dc Add cell module skeleton (phase 1 additive)
6e73f49 Fix Task 1 doc-test placeholder
7b116b3 Add SortKey enum scaffolding and same-variant unit tests
c544402 Implement SortKey::compare (same-variant, cross-variant, None policy)
89569cd Document SortKey discriminant drift risk and total_cmp NaN handling
1fe3e7f Add CellStyle enum
bc4e321 Add RowStatus enum and indicator() accessor
88fb0eb Add Cell struct, new(), builder methods, accessors
501a333 Add Cell typed-value constructors + un-ignore Task 6 doc test
d4cef02 Add Cell style-flavored constructors
0a3c8ab Add Cell From<&str>, From<String>, From<CompactString>
38e452c Add InitialSort struct
7e89703 Add Column::default_sort field, with_default_sort, default_sort()
b823196 Add new TableMessage sort variants (alongside old)
796d320 Add doc test for Column::default_sort getter (Phase 1 polish)
```

Plus this handoff doc as the final Phase 1 commit.

Good luck.
