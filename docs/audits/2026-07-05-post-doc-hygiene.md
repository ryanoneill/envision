# Envision post-doc-hygiene closure record — 2026-07-05

**Target commit:** doc-hygiene split PR (this cadence)
**Preceding audits:**
- [`2026-07-04-pre-release-hygiene.md`](2026-07-04-pre-release-hygiene.md) — original A- (3.62 GPA); named finding #3 as one of 5 blocking findings that gated v0.17.0
- [`2026-07-05-post-release-hygiene.md`](2026-07-05-post-release-hygiene.md) — A (3.91 GPA) after the release-readiness cadence closed 5/5 findings BUT noted #3 as pre-existing/persistent (CHANGELOG.md + MIGRATION.md > 1000-line cap not addressed by that cadence; deferred as follow-up)
- [`2026-07-05-post-consistency-cleanup.md`](2026-07-05-post-consistency-cleanup.md) — consistency-cleanup cadence closed findings #6 + #8; noted #3 as still-outstanding follow-up

## Purpose

Close audit finding #3 (CHANGELOG.md + MIGRATION.md over the project's 1000-line human-facing cap). The release-readiness cadence noted this as pre-existing; the release-readiness cadence's Task 1 net-shrunk CHANGELOG by 278 lines through consolidation but grew MIGRATION.md by 108 lines through the v0.16→v0.17 backfill. Consistency-cleanup added ~40 more lines to both. Neither prior cadence had scope to actually address the line-count issue itself.

This cadence splits both files at a last-3-versions boundary, archiving older entries to `-legacy.md` companions. Purely mechanical file split; no content dropped.

## Verification

### CHANGELOG.md
- **Before:** 1469 lines, 24 version blocks (`[Unreleased]` + `[0.16.0]` through `[0.1.0]`)
- **After:** 356 lines, 4 version blocks (`[Unreleased]` + `[0.16.0]` + `[0.15.1]` + `[0.15.0]`)
- **Archive:** `CHANGELOG-legacy.md` — 1124 lines, 20 version blocks (`[0.14.1]` through `[0.1.0]`)
- **Reduction:** 1469 → 356 lines (76% reduction on the primary file)
- **Under 1000-line cap:** ✅ (356 < 1000, 644-line headroom)

### MIGRATION.md
- **Before:** 1265 lines, 7 upgrade paths (`v0.16 → v0.17` through `v0.4 → v0.5`)
- **After:** 224 lines, 3 upgrade paths (`v0.16 → v0.17` + `v0.15 → v0.16` + `v0.14 → v0.15`)
- **Archive:** `MIGRATION-legacy.md` — 1048 lines, 4 upgrade paths (`v0.13 → v0.14` through `v0.4 → v0.5`)
- **Reduction:** 1265 → 224 lines (82% reduction on the primary file)
- **Under 1000-line cap:** ✅ (224 < 1000, 776-line headroom)

### Content integrity
- Total line count preserved (2734 → 2752 lines; +18 lines from the legacy-archive banner headers)
- Zero content dropped
- Both current files link to their legacy companions via a "Historical entries: see [`...-legacy.md`]" footer

### Cross-references verified
- Current `CHANGELOG.md` still contains `[0.16.0]` version block (grep count: 1)
- `CHANGELOG-legacy.md` contains `[0.14.1]` at the top (grep count: 1)
- Current `MIGRATION.md` still contains `## v0.16.x to v0.17.0` header (grep count: 1)
- `MIGRATION-legacy.md` contains `## v0.13.x to v0.14.0` at the top (grep count: 1)

## Design decision

**Split point:** last 3 released versions on each file.

**Rationale:**
- CHANGELOG entries older than 3 minor versions rarely get consulted for release notes; consumers on 0.14.x upgrading to 0.17.x follow MIGRATION.md, not CHANGELOG.
- MIGRATION keeps 3 upgrade paths (v0.14 → v0.15 → v0.16 → v0.17) so any consumer on 0.14.x or newer sees their full path in the main file without an archive lookup.
- Both files land well under the 1000-line cap with comfortable margin for the next 2-3 releases before another archive rotation is needed.

**Alternatives considered and rejected:**
- Keep MORE versions (e.g., 6 releases) — bloats the current files without proportional discoverability benefit.
- Keep FEWER versions (e.g., only current + one prior) — pushes too much history to the archive; forces frequent lookups.

## Compressed cadence execution

Unlike the prior 4-PR cadences (brainstorm → spec → plan → impl → tracking), this cadence used a compressed single-PR execution:
- Design proposal + user approval in one brainstorm turn (via `AskUserQuestion`).
- No formal spec or plan document — the design is mechanical (line-boundary decision + banner text), and there are no downstream API implications to review.
- Single PR with 2 commits: (1) the file split; (2) this closure record + audit archive README update.
- No formal reviewer/adversarial-review dispatch — the diff is a pure move-and-rename operation; verification is by grep and line-count comparison.

Justified because:
- Zero code changes; only files touched are `CHANGELOG.md`, `CHANGELOG-legacy.md`, `MIGRATION.md`, `MIGRATION-legacy.md`, `docs/audits/2026-07-05-post-doc-hygiene.md`, `docs/audits/README.md`.
- Zero API surface implications for downstream consumers.
- Full 4-PR cadence would be over-ceremony for what's essentially a `cp && edit && rm` operation.

## Verdict

- **Finding #3: CLOSED.**
- **CHANGELOG.md: 1469 → 356 lines** (under 1000-line cap by 644-line margin).
- **MIGRATION.md: 1265 → 224 lines** (under 1000-line cap by 776-line margin).
- **All content preserved** in legacy archives with clear cross-references.

## Next audit outlook

The remaining audit finding backlog before v0.17.0 release:
- **Finding #4** (`compact_str` sporadic adoption — 3 files) — still deferred; commitment-vs-drop decision needed; not blocking.
- **Finding #5** (22 examples still `use ratatui::...` directly) — cosmetic; deferred.
- **Cadence D backlog** (finish `selected()` alias removal across ~15 remaining components + expand to include `set_selected → set_selected_index` renames on dropdown/select) — pre-1.0 API polish; scheduled for v0.18+.

None of the outstanding items block v0.17.0. All three original release-hygiene findings (#1 README, #2 CHANGELOG stacked-blocks, #3 file-size cap) are now closed, plus findings #6 (selection accessor divergence) and #8 (dep leakage on AppHarness surface) from the Cadence A cleanup. Ready to proceed with `/release minor` when the crates.io key is sorted.
