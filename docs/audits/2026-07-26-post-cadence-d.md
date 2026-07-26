# Envision post-Cadence-D closure record — 2026-07-26

**Target commit:** [`4c8894d`](https://github.com/ryanoneill/envision/commit/4c8894d) — "Impl: Cadence D — selection-accessor completion (v0.18.0) (#516)"
**Cadence PRs:** spec [#514](https://github.com/ryanoneill/envision/pull/514), plan [#515](https://github.com/ryanoneill/envision/pull/515), impl [#516](https://github.com/ryanoneill/envision/pull/516), this record.
**Preceding audit:** [`2026-07-05-post-cleanup-audit.md`](2026-07-05-post-cleanup-audit.md) (`A`, 3.90 GPA), which deferred the residual selection-accessor aliases to "Cadence D or v0.18+". Re-confirmed as still-open by [`2026-07-25-pre-v0.17.0-release.md`](2026-07-25-pre-v0.17.0-release.md).

## Purpose

Finish the getter/setter half of the selection-accessor unification Cadence A began. Every component whose selection accessor was spelled `selected` now spells it `selected_index`; every corresponding mutator is `set_selected_index`.

The problem was not the name in isolation — it was that `selected` did not predict the return type. Four different signatures shared the spelling:

| Signature | Where |
|---|---|
| `Option<usize>` | 15 components |
| `Option<(usize, usize)>` | `heatmap` |
| bare `usize` | `box_plot` |
| `bool` consuming builder | `annotation::widget` |

The `selected_index` / `selected_item` / `value_at_selection` system makes the name carry the type.

## What shipped

Net **−52 lines** across 42 files (331 insertions, 383 deletions) — deletions exceed insertions, which is the shape of a real cleanup rather than an addition.

- **12 aliases deleted** (body was exactly `self.selected_index()`) — `accordion`, `dropdown`, `file_browser`, `loading_list`, `menu`, `metrics_dashboard`, `radio_group`, `searchable_list`, `select`, `selectable_list`, `tabs`, `tree`.
- **4 primary accessors renamed** (no `selected_index()` sibling existed) — `alert_panel`, `diagram`, `multi_progress`, `box_plot`.
- **12 setters renamed** `set_selected` → `set_selected_index`.
- **Prelude** gained the four missing harness types (`Assertion`, `MessageSendError`, `Snapshot`, `TrySendError`), so `envision::prelude::*` now matches the crate-root re-export.
- **`CHANGELOG.md`** gained an `[Unreleased]` block; **`MIGRATION.md`** gained § *v0.17.x to v0.18.0* with a 28-row before/after table.

### Payoff gates at `4c8894d`

```
pub fn selected( in src/            → 2   (both intentional survivors)
.set_selected( tree-wide            → 0
```

The two survivors are `heatmap/mod.rs:449` (`Option<(usize, usize)>` coordinates — not an index, disentangled from this pattern during Cadence A, and MIGRATION explicitly warns users off migrating it) and `annotation/widget.rs:75` (a `bool` consuming builder, a different concept entirely).

**Note for future gate authors:** a naive `.selected()` callsite grep still returns 42 hits and looks like a failure. It is not — 35 are ratatui's `ListState::selected()` on private fields, and 6 are heatmap's tuple-returning test asserts. The honest gate is `pub fn selected(` over `src/`, not the callsite form. The reverse was true in Cadence A, where the callsite form was the honest one. Neither form is universally correct; pick per-cadence and verify the residue by hand.

## Findings closed

- **Selection-accessor aliases (Cadence D item, carried since the 2026-07-05 post-cleanup audit)** — CLOSED.
- **`tab_bar` setter naming outlier (audit finding N3)** — CLOSED, though not by touching `tab_bar`. N3 was defined as "`tab_bar` uses `set_selected_index()` where every sibling uses `set_selected()`". Converging the twelve siblings onto `tab_bar`'s spelling closes it from the other direction; `tab_bar`'s API is untouched and owes no migration row.
- **Prelude harness-type gap (raised by Cadence A's final review)** — CLOSED. Cadence A flagged only `MessageSendError` and `TrySendError`; adding just those two would have recreated the same inconsistency one level down, so `Assertion` and `Snapshot` came along.

## Findings opened

Two symmetric incompletenesses that the whole-branch review surfaced. Both are recorded in the `[Unreleased]` Known Deferred Findings block rather than silently left:

- **The `with_selected` builder leg was not renamed.** `menu`, `tabs`, `tree`, `loading_list`, `selectable_list`, `radio_group`, `table`, and `tab_bar` still expose `with_selected(i)` alongside `set_selected_index()` and `selected_index()` — one type, two spellings of one concept. `menu/mod.rs` shows `with_selected` literally delegating to `set_selected_index`. Renaming it is a second breaking change and belongs in its own cadence.
- **Three `selected_item()` one-line aliases survive** — `file_browser::selected_item()` → `selected_entry()`, `tree::selected_item()` → `selected_node()`, `accordion::selected_item()` → `focused_panel()`. These are the exact "pure redundancy" shape this cadence deleted twelve times. They were kept only because `selected_item()` is the canonical cross-component spelling Cadence A established, which means the cadence applied two incompatible rules to the same code shape. Delete-or-keep is genuinely open.

The CHANGELOG originally claimed this release "finishes the job." It does not, and the claim was corrected to "finishes the getter/setter half" before merge.

## Verification

| Gate | Result |
|---|---|
| `cargo fmt --check` | clean |
| `cargo clippy --all-features --all-targets -- -D warnings` | 0 warnings |
| `cargo nextest run --all-features` | 7464/7464 |
| `cargo test --all-features --doc` | 2579/2579 |
| `cargo build --no-default-features` | pass |
| `cargo doc --no-deps --all-features` | 0 warnings |
| Audit scorecard | **9/9**, accessor symmetry gaps 0 |
| CI on #516 | 17/17 checks, 3 platforms × 2 toolchains |

## Process notes

**Setters were renamed in the same commit as their getter.** This was a hard design constraint, not a stylistic preference. During Cadence A, deleting a getter without its setter orphaned `tab_bar::set_selected` and dropped the audit scorecard from 9/9 to 8/9 — the audit's symmetry rule (`tools/audit/src/code_analysis.rs:238-266`) requires every `set_X()` to have a matching getter. The audit was re-run at the end of every unit for the same reason.

**The audit's symmetry rule is directional.** It checks setter→getter, never getter→reverse. Components that ship a getter with no setter (`accordion`, `alert_panel`, `diagram`, `file_browser`, plus pre-existing `flame_graph`, `command_palette`, `span_tree`) are correctly *not flagged* rather than silently passing — but a future cadence should not read a green symmetry check as evidence that getter-only components were considered.

**The audit cannot see files behind a private `mod state;`.** `box_plot/state.rs`, `diagram/state.rs`, and `table/state.rs` are invisible to every scorecard check, because source selection (`tools/audit/src/scorecard.rs:275-295`) reads `mod.rs` plus only files named by `pub mod X;` / `pub use X::`. Two of the four Unit 2 renames landed in exactly those blind spots and were hand-verified instead.

**Adversarial and whole-branch review both earned their cost.** The adversarial spec review (user persona) found 4 must-fix issues, two of which reproduced Cadence A's failures — an unpassable tree-wide gate, and a `box_plot` exclusion that was a rationalization rather than a reason. Bringing `box_plot` in scope fixed both. The final whole-branch review then found 3 defects no per-commit review could see, including dead code where a mechanical rename turned an assertion into a byte-identical duplicate of the line above it, and the overclaiming CHANGELOG sentence.

**Test-name drift is a recurring cadence gap, not a one-off.** Fourteen test functions and two comments still spelled `set_selected` after the rename; two of them (`table/tests.rs`, `tab_bar/tests.rs`) dated from Cadence A. Mechanical renames update call sites because the compiler forces it, and leave identifiers the compiler does not check. Future cadences should add a test-name grep to the gauntlet.

## Not re-audited

No Fable re-audit was run. The changes are mechanical renames with a full verification gauntlet, three independent per-unit reviews, and one whole-branch review. The next full audit should gate v0.18.0's release rather than this cadence.
