# Envision audit archive

This directory contains the periodic library-audit reports produced by the `superpowers:audit` skill. Each report is a 25-category graded evaluation from the perspective of a demanding consumer deciding whether to bet a production app on the library.

## Purpose

Audits historically lived only in session transcripts, which meant:

- Grades and findings couldn't be diffed across releases.
- New maintainers had to rediscover the current known-issues surface from scratch.
- A "known deferred finding" claim in the CHANGELOG couldn't be cross-referenced to its originating audit.

Checking in each audit report at `docs/audits/YYYY-MM-DD-<label>.md` gives the audit history durability — every published grade, every finding, every deferral has a git-blame lineage.

## Naming

`YYYY-MM-DD-<label>.md` where `<label>` describes the audit's context:

- `pre-release-hygiene` — audit that precipitated a release-readiness cadence
- `post-release-hygiene` — verification audit after such a cadence
- `pre-v0.NN.0` — audit run to gate a release
- `interim-<topic>` — audit run to check on a specific concern

## Reading a report

Every report follows the format the `superpowers:audit` skill produces:

1. Overall grade + weighted GPA (target ≥ `A` / 3.85)
2. Group-level GPAs across 6 weighted groups (First Impressions 15% / API Design 25% / Engineering Quality 20% / Testing 20% / Architecture 10% / Missing Pieces 10%)
3. 25 category grades with evidence
4. Trust-eroding findings (ranked by severity)
5. Top 5 improvements (highest-ROI recommendations)
6. Detailed findings per category

## Archive

- [2026-07-04 pre-release-hygiene](2026-07-04-pre-release-hygiene.md) — `A-` (3.62 GPA). Surfaced 5 blocking findings that gated v0.17.0. Precipitated the release-readiness cadence (spec PR #502, plan PR #503, impl PR #504).
- [2026-07-05 post-release-hygiene](2026-07-05-post-release-hygiene.md) — Fable verification audit after the release-readiness cadence merged. `A` (3.91 GPA), 9/9 scorecard, all 5 findings closed.
- [2026-07-05 post-consistency-cleanup](2026-07-05-post-consistency-cleanup.md) — closure record for the consistency-cleanup cadence (spec PR #506, plan PR #507, impl PR #508). Marks audit findings #6 (`selected_value`/`selected_item`/`active_tab` divergence) and #8 (`tokio::sync::mpsc::Sender` dep leakage on AppHarness) as CLOSED. Scorecard 9/9 preserved. No Fable re-audit — plan opted out; the changes were mechanical and the verification gauntlet was declared sufficient at spec-writing time.
- [2026-07-05 post-doc-hygiene](2026-07-05-post-doc-hygiene.md) — closure record for the doc-hygiene split cadence. Marks audit finding #3 (CHANGELOG.md + MIGRATION.md over 1000-line human-facing cap) as CLOSED. CHANGELOG: 1469 → 356 lines; MIGRATION: 1265 → 224 lines. Older entries archived to `CHANGELOG-legacy.md` + `MIGRATION-legacy.md`. Compressed single-PR execution (mechanical file split, no design decisions to spec).
- [2026-07-05 post-example-imports](2026-07-05-post-example-imports.md) — closure record for the `example-imports-cleanup` branch. Marks in-session-audit finding #5 (22 of 91 examples with redundant top-of-file `use ratatui::...` imports duplicating `envision::prelude::*`) as CLOSED. Docs/examples-only fix, no API surface change.
- [2026-07-05 post-cleanup](2026-07-05-post-cleanup-audit.md) — Fable snapshot audit after the consistency-cleanup + pending doc-hygiene/example-imports cadences. `A` (3.90 GPA); effectively flat vs 3.91 baseline. Confirmed findings #6 and #8 closed. Surfaced one moderate new finding: README duplicate `### Display Components` tables with conflicting descriptions (fixed in the same PR as this audit report check-in). Remaining findings deferred to Cadence D or v0.18+.
- [2026-07-25 pre-v0.17.0-release](2026-07-25-pre-v0.17.0-release.md) — **final pre-release gate.** `A` (3.95 GPA), up from 3.90. Verified the three cleanup PRs (#510 doc-hygiene, #511 example imports, #512 README dedup) landed as intended: Getting Started A- → A, Examples A- → A, Code Organization B+ → A-. Zero new findings. Scorecard 9/9, 17,363 tests green, 0 clippy warnings, 0 unsafe. **Recommendation: SHIP v0.17.0** — with one release-checklist item (verify `README.md:47` `version = "0.17"` resolves post-publish).
