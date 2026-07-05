# Envision post-consistency-cleanup closure record — 2026-07-05

**Reviewer:** in-session verification gauntlet + final whole-branch opus review (no Fable re-audit — plan explicitly opted out; the changes are mechanical and the plan's verification gauntlet was declared sufficient at spec-writing time)
**Target commit:** `09608d6` — consistency-cleanup impl PR #508 merged 2026-07-05T19:32:07Z
**Preceding audit:** [`2026-07-05-post-release-hygiene.md`](2026-07-05-post-release-hygiene.md) — A (3.91 GPA), 9/9 scorecard, findings #6 + #8 documented as deferred

## Purpose

Verify closure of the two deferred audit findings the consistency-cleanup cadence targeted:

- **Finding #6** — `selected_value` / `selected_item` / `active_tab` accessor divergence across `dropdown`, `select`, `heatmap`, `tab_bar`, and `data_grid` (audit tool's original list; the cadence expanded scope to include `table` per adversarial review M3).
- **Finding #8** — Dependency leakage in the AppHarness surface (specifically the `tokio::sync::mpsc::Sender` return of `message_sender()`; the other sites the audit named were verified as legitimate escape hatches or already routed through envision's own re-exports).

## Closure verification

### Finding #6 — Selection accessor divergence — CLOSED

Six components now expose the canonical `selected_item() -> Option<&T>` accessor pattern (or `Option<&str>` for string-keyed variants), with literal aliases deleted and semantic outliers renamed. `set_selected_index()` mutator names match the `selected_index()` getters. Grep-verified callsite forms across `src/`, `tests/`, `examples/`:

- `grep -rn '\.selected_value(' src/ tests/ examples/` → zero hits
- `grep -rn '\.active_tab(' src/ tests/ examples/` → zero hits
- `grep -rn '\.active_tab_mut(' src/ tests/ examples/` → zero hits
- `grep -rn '\.selected_row(' src/ tests/ examples/` → zero hits
- `grep -rn '\.set_selected(' src/component/{tab_bar,data_grid,table}/ tests/ examples/` → zero hits on the three components (other components' `set_selected` deferred to Cadence D)

Audit tool at commit `09608d6`:

```
Accessor Symmetry (set_X without matching getter):
  All setters have matching getters.
```

Scorecard: **9/9 PASS** (accessor symmetry gap count 0).

Per-component evidence:

| Component | Old surface | New surface |
|---|---|---|
| `dropdown` | `selected_value() -> Option<&str>` + `selected_item() -> Option<&str>` | `selected_item() -> Option<&str>` (only) |
| `select` | Same as dropdown | `selected_item() -> Option<&str>` (only) |
| `heatmap` | `selected_value() -> Option<f64>` (returns data value at cursor coords — not a selection accessor) | `value_at_selection() -> Option<f64>` (renamed — `value_` prefix sorts distinctly from `selected_*` in IDE autocomplete) |
| `tab_bar` | `active_tab() + active_tab_mut() + selected() + selected_index() + set_selected()` | `selected_item() + selected_item_mut() + selected_index() + set_selected_index()` |
| `data_grid` | `selected() + selected_index() + selected_row() + selected_item() + set_selected()` | `selected_index() + selected_item() + set_selected_index()` |
| `table` | Same as data_grid | Same as data_grid |

### Finding #8 — Dependency leakage on AppHarness surface — CLOSED

`AppHarness::message_sender()` at `src/harness/app_harness/mod.rs:264` now returns `MessageSender<A::Message>` — a first-party newtype in `src/harness/message_sender.rs`. Downstream consumers no longer need `tokio` as a direct dependency to use the accessor.

Grep-verified public API surface:

- `grep -rn 'pub fn.*tokio::sync::mpsc::Sender' src/` → zero hits
- `grep -rn 'pub fn.*ratatui::layout::Position' src/` → zero hits (Position cosmetic at `virtual_terminal.rs:147` also landed)

`MessageSender<M>` design deliberately parameterized on the message type `M: Send + 'static`, NOT on `A: App`, so downstream helper functions can be written as `fn spawn_watcher<M: Send + 'static>(sender: MessageSender<M>) { ... }` without depending on envision's `App` trait.

Full passthrough API preserved (per adversarial review M4): `send`, `try_send`, `is_closed`, `capacity`, `max_capacity`, plus `into_inner()` explicit escape hatch for consumers needing tokio-specific functionality (`reserve`, `send_timeout`, `same_channel`, `downgrade`, `closed()` future). First-party error types `MessageSendError<T>` and `TrySendError<T>::{Full, Closed}` preserve tokio's semantic distinction.

The other sites the 2026-07-04 Fable audit named under "8 dep-leakage signatures" were verified during spec-writing as legitimate escape hatches or already using envision's own re-exports:

- `Color` / `Style` in `StatusBarItem` — no envision-native equivalent; ratatui owns color/style
- `Widget` / `StatefulWidget` re-exports in prelude — legitimate custom-component escape hatch
- `ratatui::buffer::Cell` in `from_ratatui_cell` — converter function that MUST reference the actual ratatui type
- ~10 `Rect` uses in layout helpers — already routed via `envision::layout::Rect` re-export
- Documented explicitly in the spec's out-of-scope section

## Cadence artifacts

- **Spec:** [`docs/superpowers/specs/2026-07-05-consistency-cleanup-cadence-design.md`](../superpowers/specs/2026-07-05-consistency-cleanup-cadence-design.md) (PR #506, commit `cfb7cec`; two rounds: initial + adversarial user-persona review folded in with 4 must-fix + 5 should-consider + 5 additional items)
- **Plan:** [`docs/superpowers/plans/2026-07-05-consistency-cleanup-cadence.md`](../superpowers/plans/2026-07-05-consistency-cleanup-cadence.md) (PR #507, commit `70c44bf`; 1803 lines)
- **Impl:** PR #508 merged as `09608d6` — 4 signed commits: consistency-sweep (`00dab7c`), message-sender (`e10f572`), CHANGELOG+MIGRATION (`3b9c150`), set_selected → set_selected_index fix (`f7d4e9d`)

## Verification gauntlet at merge

- `cargo fmt --check` — clean
- `cargo clippy --all-features -- -D warnings` — zero warnings
- `cargo nextest run --all-features` — 7466 pass
- `cargo test --all-features --doc` — 2591 pass
- `cargo build --no-default-features` — clean
- `cargo test --no-default-features --no-run` — clean (D8 lesson)
- `cargo build --examples --all-features` — clean
- `cargo doc --no-deps --all-features` — zero intra-doc-link warnings
- `./tools/audit/target/release/envision-audit all` — 9/9 scorecard PASS

## Final whole-branch review (opus) findings

**Verdict:** Approved with follow-up. 0 Critical, 0 Important, 3 Minor items — all become Cadence D backlog:

1. `dropdown::set_selected` + `select::set_selected` retain the same accessor-asymmetry Task 4 corrected on tab_bar/data_grid/table. Both have `selected_index()` getter but `set_selected()` mutator. The audit tool doesn't flag it today only because `dropdown::selected()` and `select::selected()` still exist (deferred to Cadence D — they're two of the ~15 remaining `selected()` alias sites). Once Cadence D removes those aliases, the same regression will trigger. Cadence D scope should be expanded to also rename these two mutators.
2. CHANGELOG Known Deferred Findings block currently mentions only the `selected()` alias removal for Cadence D — doesn't mention the future `set_selected → set_selected_index` renames on dropdown/select. Minor documentation gap.
3. `MessageSendError` + `TrySendError` are re-exported at crate root but NOT in prelude. Consumers doing `use envision::prelude::*` who want to match on `TrySendError::Full/Closed` still need an explicit `use envision::{MessageSendError, TrySendError};`. Minor discoverability improvement.

None block v0.17.0 release.

## Deferred to Cadence D (v0.18+)

- `selected()` alias removal on ~15 remaining components (accordion, tabs, radio_group, searchable_list, selectable_list, tree, menu, box_plot, loading_list, alert_panel, and others).
- `set_selected → set_selected_index` on dropdown + select (per final review Minor #1 above — bundle with the alias removal to avoid the mid-cadence scorecard regression Task 4 caught here).
- Update Known Deferred Findings block to explicitly name the setter renames (per final review Minor #2).
- Add `MessageSendError` + `TrySendError` to `envision::prelude` (per final review Minor #3 — one-line follow-up; could ship in Cadence B doc-hygiene or Cadence D).

## Verdict

- **Findings #6 and #8: CLOSED.**
- **Scorecard: 9/9 preserved.**
- **Cadence: complete.** Ready to proceed with Cadence B (doc-hygiene split of CHANGELOG.md + MIGRATION.md) or straight to `/release minor` for v0.17.0 per the user's preference.
