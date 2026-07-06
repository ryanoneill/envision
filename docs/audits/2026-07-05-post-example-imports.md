# Envision post-example-imports closure record — 2026-07-05

**Target commit:** `example-imports-cleanup` branch — migration commit + this closure record
**Preceding audit:** In-session `/audit` at 2026-07-05 morning, finding #5 — "22 of 91 examples still `use ratatui::...` directly at the top (not just via `envision::prelude::*`)." Documented as explicitly out-of-scope/deferred by the consistency-cleanup cadence spec: [`docs/superpowers/specs/2026-07-05-consistency-cleanup-cadence-design.md:25`](../superpowers/specs/2026-07-05-consistency-cleanup-cadence-design.md) ("22 examples that `use ratatui::...` directly (finding #5). Cosmetic; skip until it matters."). Cross-referenced in the same-day [`2026-07-05-post-release-hygiene.md`](2026-07-05-post-release-hygiene.md) audit archive entry.

## Purpose

Close audit finding #5 by dropping, from each of the 22 flagged examples, the `use ratatui::...` imports that duplicate what `envision::prelude::*` already re-exports (`src/lib.rs:481-490`): layout types (`Alignment`, `Constraint`, `Direction`, `Frame`, `Layout`, `Margin`, `Position`, `Rect`, `Size`, `Terminal`), style types (`Color`, `Modifier`, `Style`, `Stylize`), text types (`Line`, `Span`, `Text`), and the widget traits (`Widget`, `StatefulWidget`). Concrete ratatui widgets (`Paragraph`, `Block`, `Borders`, `BorderType`, `Padding`, `List`, `ListItem`) are a deliberate "smaller surface area" omission from the prelude (`src/lib.rs:479-482`) and were left untouched.

## Before/after

- **Before:** 22 of 91 examples had a top-of-file `use ratatui::...` import that duplicated a prelude-covered type.
- **After migration:** all 22 examples still have a `use ratatui::widgets::{...}` line (`grep -l "^use ratatui::" examples/*.rs | wc -l` → still 22) — but every remaining line now contains **only** concrete-widget names never re-exported by the prelude. Zero prelude-covered names (`Alignment`, `Constraint`, `Direction`, `Layout`, `Terminal`, `Color`, `Modifier`, `Style`, `Line`, `Span`, `Widget`, `StatefulWidget`, etc.) remain in any example's `ratatui::` import.
- **8 of the 22 files required an actual edit** (multi-line prelude-covered imports removed): `annotations.rs`, `async_counter.rs`, `beautiful_dashboard.rs`, `capture_backend.rs`, `component_showcase.rs`, `counter_app.rs`, `test_harness.rs`, `themed_app.rs`.
- **14 of the 22 files needed no change** — they already only imported concrete widgets (`big_text.rs`, `code_block.rs`, `dashboard_demo.rs`, `diff_viewer.rs`, `focus_manager.rs`, `help_panel.rs`, `line_input.rs`, `markdown_renderer.rs`, `production_app.rs`, `scroll_view.rs`, `scrollable_text.rs`, `styling_showcase.rs`, `terminal_output.rs`, `title_card.rs`).
- Net diff: 26 redundant `use ratatui::...` lines removed, 2 lines added (`use envision::prelude::*;` in the two examples — `capture_backend.rs` and `annotations.rs` — that weren't using the prelude at all before this change; both also had single-item `envision::...` imports for types the prelude already covers, e.g. `envision::backend::CaptureBackend`, `envision::harness::TestHarness`, which were replaced by the prelude import).

The "significantly fewer than 22" framing in the finding's original triage assumed some examples imported *only* prelude-covered ratatui types. In practice every one of the 22 also imports at least one concrete widget, so the file-count metric is unchanged — the substantive signal is the per-file import-name audit above (zero prelude-covered names remain).

## Examples that still legitimately import ratatui widgets (post-migration)

All 22 examples retain a `use ratatui::widgets::{...}` (or single-name) import for concrete widgets not re-exported by the prelude:

| Example | Retained ratatui import |
|---|---|
| `annotations.rs` | `Block, Borders, List, ListItem, Paragraph` |
| `async_counter.rs` | `Block, Borders, Paragraph` |
| `beautiful_dashboard.rs` | `Block, BorderType, Borders, Padding` |
| `big_text.rs` | `Block, Borders, Paragraph` |
| `capture_backend.rs` | `Block, Borders, Paragraph` |
| `code_block.rs` | `Paragraph` |
| `component_showcase.rs` | `Block, Borders, Paragraph` |
| `counter_app.rs` | `Block, Borders, Paragraph` |
| `dashboard_demo.rs` | `Block, Borders, Paragraph` |
| `diff_viewer.rs` | `Paragraph` |
| `focus_manager.rs` | `Block, Borders, Paragraph` |
| `help_panel.rs` | `Paragraph` |
| `line_input.rs` | `Block, Borders, Paragraph` |
| `markdown_renderer.rs` | `Paragraph` |
| `production_app.rs` | `Block, BorderType, Borders, Padding, Paragraph` |
| `scroll_view.rs` | `Paragraph` |
| `scrollable_text.rs` | `Paragraph` |
| `styling_showcase.rs` | `Block, Borders, Paragraph` |
| `terminal_output.rs` | `Paragraph` |
| `test_harness.rs` | `Block, Borders, Paragraph` |
| `themed_app.rs` | `Block, Borders, Paragraph` |
| `title_card.rs` | `Block, Borders, Paragraph` |

These are all concrete ratatui widget types with no envision equivalent — the prelude deliberately doesn't re-export them (`src/lib.rs:479-482`), so zero further migration is possible or desired here.

## Verification gauntlet

- `cargo build --examples --all-features` — clean
- `cargo build --example <name> --all-features` — clean, spot-checked for every edited file
- `cargo clippy --examples --all-features -- -D warnings` — zero warnings
- `cargo clippy --all-features -- -D warnings` — zero warnings
- `cargo clippy -- -D warnings` (default features) — zero warnings
- `cargo test --no-default-features --no-run` — clean (D8 lesson: 1.85 CI job runs `--no-default-features`, no markdown feature)
- `cargo fmt --check` — clean
- `grep -l "^use ratatui::" examples/*.rs | wc -l` — 22 before, 22 after (see "Before/after" note on why the file-count metric doesn't drop even though the redundant imports are gone)

## Cadence artifacts

- **This PR:** `example-imports-cleanup` branch — one signed commit dropping the redundant imports, one signed commit adding this closure record.
- **Originating audit:** in-session `/audit` at 2026-07-05 morning (finding #5), explicitly deferred by the consistency-cleanup cadence spec ([`docs/superpowers/specs/2026-07-05-consistency-cleanup-cadence-design.md`](../superpowers/specs/2026-07-05-consistency-cleanup-cadence-design.md), PR #506) as "cosmetic; skip until it matters."
- **Same-day archived audit report:** [`2026-07-05-post-release-hygiene.md`](2026-07-05-post-release-hygiene.md).

## Verdict

**Finding #5: CLOSED.** No API surface change; docs/examples-only fix. The prelude was already sufficient for every prelude-covered type used across the 22 examples — the redundant `use ratatui::...` lines just wasted line space and obscured which imports were actually load-bearing.
