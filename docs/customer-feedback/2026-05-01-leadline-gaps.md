# leadline customer-feedback — 2026-05-01

Surfaced while building the leadline TUI on top of envision.

## Sources

- `~/workspace/ryanoneill/rust-ai-explorations/notes/envision_gaps.md` — gaps #1–#7 with workaround / proposal / removal-trigger fields
- `~/workspace/ryanoneill/rust-ai-explorations/notes/envision_table_sort_api_redesign.md` — detailed brief for gap #7 (sort message redesign)
- Direct conversation 2026-05-01 — 14 additional pain-point items (numbered D1–D14 below)

## Items

### Resolved 2026-05-02 — Table/Sort/Cell unification

Spec PR #459, plan PR #460, implementation PR #461 (`235bcae`). Leadline migration verified in their commit `e429a6a`.

- **G1** ✅ Typed `TableRow::sort_key` — shipped via `SortKey` enum on `Cell` with same-variant + cross-variant fallback compare
- **G3 / D4** ✅ Per-cell styling — shipped as unified `Cell { text, style, sort_key }` type with style-flavored constructors and theme-mapped render
- **G7** ✅ `SortBy` 3-cycle redesign — shipped as `SortAsc/Desc/Toggle/Clear/RemoveSort` vocabulary + `Column::with_default_sort` + `TableState::with_initial_sort(s)`. Leadline confirmed: net -33 lines after migration; three workaround helpers (`apply_table_msg`, `apply_sort_persistent`, `strip_suffix_numeric_comparator`) deleted.

### Resolved 2026-05-02 — App::init args redesign

Spec PR #463, plan PR #464, implementation PR #465 (`82a9a41`).

- **D1** ✅ `App::init` takes args — shipped as `App::Args` associated type (no default; explicit `type Args = ();` per stable-Rust constraint), `App::init(args: Self::Args)`, sealed `OptionalArgs` marker gating the `()` shortcut, and a `RuntimeBuilder` → `ConfiguredRuntimeBuilder` typestate-lite split via `with_args` promotion. `RuntimeBuilder::state(...)` deleted (subsumed). `panic!` default on `init` removed. Test-ergonomics unlock pinned: multi-Runtime parallelism + trybuild compile-fail. Migrated 157 sites across 113 files.

### High-leverage follow-ups (leadline's stated priorities)

- **D7** View-snapshot testing is undocumented — `AppHarness`/`TestHarness` exist but no docs explaining when to reach for them. Want documented "render at W×H, dispatch event sequence, snapshot cell buffer" pattern.

### Resolved 2026-05-02 — Chrome ownership protocol

Spec PR #467, plan PR #468, implementation PR #469 (`aaaefa1`).

- **G2** ✅ Table inner border inside chrome host — shipped via `RenderContext::chrome_owned` propagation; Table consults the flag and skips its outer Block when embedded.
- **D2** ✅ `PaneLayout::view_with(state, ctx, |pane_id, child_ctx| ...)` — closure-based renderer; envision owns inner-rect computation; `Margin{1,1}` knowledge deleted from consumer code.
- **D11** ✅ `StyledText::with_show_border(false)` workaround unnecessary in embedded mode — chrome_owned propagation suppresses the border automatically. The standalone-no-border opt-out via `with_show_border(false)` stays.
- **Bonus uniform audit**: 35 chrome-drawing components (LogViewer, ScrollView, ScrollableText, MarkdownRenderer, ConversationView, DataGrid, MetricsDashboard absent — only per-cell, KeyHints absent — Paragraph only, etc.) all consult `chrome_owned`. Future consumers embedding any of them get correct behavior without further envision changes.

### Resolved 2026-05-08 — Theme palette + severity helper

Spec PR #471, plan PR #472, implementation PR #473 (`4d5b05e`).

- **D6** ✅ Severity helper in `Theme` — shipped as `Severity` enum (`Good | Mild | Bad | Critical`, `#[non_exhaustive]`) + `Severity::from_thresholds(value: f64, &[(f64, Severity)]) -> Severity` first-match-wins bucketer + `Theme::severity_color(Severity) -> Color` and `Theme::severity_style(Severity) -> Style` accessors. `severity_style` adds `BOLD` modifier on `Critical` only. Default theme collapse caveat documented (Mild/Bad both render as `Color::Yellow`; `BOLD`-on-Critical mitigates).
- **D9** ✅ Theme color access — shipped as `NamedColor` enum (26 variants, `#[non_exhaustive]`, derived from Catppuccin Mocha) + `Palette` struct (one public `Color` field per variant) + `Theme::color(NamedColor) -> Color` accessor. Each shipped theme constructor populates its `palette: Palette` field with documented nearest-equivalent mappings (Catppuccin 1:1; Nord/Dracula/Solarized/Gruvbox per-theme; Default basic-`Color` collapse). 75 raw `pub const` color constants (`CATPPUCCIN_*`, `NORD0`–`NORD15`, `DRACULA_*`, `SOLARIZED_*`, `GRUVBOX_*`) marked `#[deprecated(since = "0.17.0")]`; constants stay accessible during transition window.
- **Module structure**: per-palette extraction (`nord.rs`, `dracula.rs`, `solarized.rs`, `gruvbox.rs`) mirroring the existing `catppuccin.rs` pattern, after `cargo fmt` expanded multi-line `#[deprecated]` attributes pushed `mod.rs` toward the 1000-line cap.

### Resolved 2026-05-08 — `CellStyle::Severity(Severity)`

Spec PR #476, plan PR #477, implementation PR #478 (`932b205`). Closes the loop on D6 + D9 — surfaced during the leadline-side migration as D15.

- **D15** ✅ Severity-aware cells reach the active theme at render time — shipped as `CellStyle::Severity(Severity)` variant resolved via `theme.severity_style(*sev)` in `cell_style_to_ratatui` (the renderer's already-in-scope `&Theme`). No `TableRow` trait churn; severity awareness lives in the `Cell` value, not the trait. New `Cell::severity(text, sev)` constructor (semantic shorthand) and `Cell::with_severity(sev)` builder (typed-cell chain, preserves G7 `SortKey`, last-call-wins precedence with `with_style` documented). Bundled `#[non_exhaustive]` on `CellStyle` — matches `Severity` / `NamedColor` precedent from PR #473 (one breaking change beats two).

### Resolved 2026-05-19 — StyledText DX: line primitive + `paragraph` rename

Spec PR #480, plan PR #481, implementation PR #482 (`72b1875`). Two coupled DX gaps in the same `StyledText` / `StyledContent` surface area shipped together.

- **D5** ✅ Line primitive — shipped as `envision::render::styled_line(frame, area, &[StyledInline], theme)` free function in new top-level `src/render.rs` module (mirrors `envision::scroll::render_scrollbar` convention). Re-exported at crate root as `envision::styled_line`. Internal implementation lifts the existing borderless-`StyledText` render path via a one-block `StyledContent` — zero new rendering logic; consumer side collapses from six-types-three-methods to one call. Module + re-export gated behind `display-components` feature for `--no-default-features` builds.
- **D14** ✅ `StyledContent::paragraph(...)` → `StyledContent::line(...)` rename — old method deleted outright (pre-1.0 ruthlessness). Bundled with `StyledBlock::Paragraph(Vec<StyledInline>)` → `StyledBlock::Line(Vec<StyledInline>)` variant rename and private `fn render_paragraph` → `fn render_line` helper rename for source-level coherence. `paragraph` name reserved for future real block-level wrapped-text semantics (lands when a consumer needs it).
- **Migration**: 18 mechanical call-site updates across 3 files (10 in `examples/styling_showcase.rs`, 8 internal across `src/component/styled_text/` including one doctest-example site the plan-time grep missed). All insta snapshots byte-identical pre/post — rename is name-only, no rendering changes.

### Resolved 2026-05-20 — Per-component style overrides

Spec PR #485, plan PR #486, implementation PR #487 (`8201a04`). Two coupled parent-side style hooks shipped together — both restore consumer flexibility previously bottlenecked by closed-enum or border-inheritance constraints. Top-line payoff: G5 unblocks the four-stop severity ramp deferred during D6 + D9 design.

- **G4** ✅ Pane title styling — shipped as `PaneConfig::with_title_style(Style)` builder + `title_style(&self) -> Option<Style>` getter. When set, the pane title renders with the given style; when `None` (default), the title inherits the border style (current behavior). Focus-invariant by design — consumer-set styles aren't silently overridden by focus state. New sibling file `src/component/pane_layout/title_style.rs` houses the impl + inline tests (mirrors the `view_with.rs` split pattern; keeps `mod.rs` under the 1000-line cap). Multi-segment `with_title_spans(Vec<Span>)` deferred per Q-α — lands when a consumer needs multi-style titles.
- **G5** ✅ StatusBar layered coloring — shipped as `StatusBarItem::with_color(Color)` + `with_style_override(Style)` builders with matching getters. Render-time precedence: `style_override > color > style.style(theme)`. **Layered setter semantics, not last-call-wins** — each setter writes its own field idempotently; branched construction (`if user_wants_emphasis { item.with_style_override(s) } else { item }`) keeps the brand color rebuildable. `with_color(c)` produces `Style::default().fg(c)` — clean separation; consumers wanting layered semantics reach for `with_style_override` explicitly.
- **Q-γ payoff — four-stop severity ramp restored**: Consumer-side `severity_status_style` helpers that collapsed Bad+Mild → Warning delete entirely; `StatusBarItem::new(t).with_color(theme.severity_color(sev))` distinguishes all four `Severity` bands on full-palette themes. Three convergence views (D15 cells via `CellStyle::Severity`, D5 banner via `styled_line` + `theme.severity_color`, G5 status segments via `with_color`) reach the same gradient.
- **Field-add safety**: Both `PaneConfig` and `StatusBarItem` had non-public fields (private + `pub(super)`) before this PR, so external consumers can't struct-literal-construct either. Only forward-compat concern was serialization; `#[serde(default)]` on every new field handles round-tripping.

### Resolved 2026-05-20 — `StyledInline` composable styles

Spec PR #489, plan PR #490, implementation PR #491 (`5c44ab4`). The 7-variant `StyledInline` leaf-enum forced single-dimension styling — `Bold + Colored` required two inlines because each leaf captured one dimension. Combinatorial explosion (2^6 = 64 variants for full coverage) ruled out; composable struct shape ships instead.

- **G6** ✅ Composable inline styling — shipped as 3-variant `StyledInline` enum (`Plain | Code | Styled { text, style: InlineStyle }`, `#[non_exhaustive]`) replacing the 7-variant leaf form. New `InlineStyle` struct (`#[non_exhaustive]`) with 6 optional dimensions (`fg`, `bg`, `bold`, `italic`, `underlined`, `strikethrough`) and 7 `const fn` builder methods (`new`, `fg`, `bg`, `bold`, `italic`, `underlined`, `strikethrough`) — usable in `const` contexts. Two-layer constructor surface on `StyledInline`: general-purpose `styled(text, style)` + 5 leaf helpers (`bold`, `italic`, `underlined`, `strikethrough`, `colored`) for single-dimension cases (~80% of usage). `strikethrough: bool` maps to `ratatui::style::Modifier::CROSSED_OUT` (ratatui's naming convention; documented in 5 places — CHANGELOG + field docstring + builder method docstring + render-arm comment + spec).
- **Migration**: All internal envision references migrated mechanically across 6 files (`examples/styling_showcase.rs`, `src/component/styled_text/tests.rs`, `src/component/styled_text/content.rs`, `src/render.rs`, `examples/styled_text.rs`, `src/component/styled_text/mod.rs`). 3-phase additive-first shape (Phase 1 adds surface alongside leaves; Phase 2 mechanical site migration; Phase 3 deletes 5 leaf variants) gives clean bisect granularity. Phase 3's compiler-enforced completeness + post-merge `grep -rn 'StyledInline::Bold\|Italic\|Underline\|Strikethrough\|Colored'` returning 0 matches both confirm migration is complete. Insta snapshots byte-identical pre/post — single-dimension cases route through the new `Styled` arm with equivalent rendering semantics.
- **Top-line payoff — bold-on-banner-values**: leadline's per-op summary banner at `app.rs:412-455` (`build_summary_inlines`) renders 5 value segments (iconnx/ort/ratio/delta/iters) that need bold + severity-color in a single inline run. Pre-G6, the bold half dropped (`Bold(t)` had no color field; `Colored {..}` had no bold field). Post-G6, `StyledInline::styled(value, InlineStyle::new().fg(value_color).bold())` lands the combo — the summary banner reads with weight contrast on value segments and the magnitude of slowdown "jumps" at the user via bold weight in addition to severity color. Snapshot test `snapshot_styled_inline_bold_and_colored_combined` ANSI-asserts both `\x1b[31m` (red) AND `\x1b[1m` (BOLD) appear on the same span — vindication of the combo.

### Resolved 2026-05-24 — Punch-list closure (D10 + D12 + D13)

Three items from the original "small rough edges" punch list close together — D12 via this cadence's new code, D10 and D13 via verification that the originally-flagged ergonomic problems were already resolved on the envision side (D10 through prior docstring work; D13 through silent shipment of `App::on_exit`). One coherent tracking-doc PR per leadline's bundling preference.

- **D10** ✅ `App::handle_event` vs `handle_event_with_state` ambiguity — resolved-via-docs. Current trait at `src/app/model/mod.rs:238-255` carries crystal-clear docstrings: `handle_event` is documented as "Override this for simple stateless event mapping (most apps)"; `handle_event_with_state` is documented as "Override this instead of `handle_event` when you need state for overlay-precedence checks or mode-dependent key bindings. The default implementation delegates to `handle_event`, ignoring state." The original "took source-reading to figure out" discovery problem the brief flagged is gone. Brief's Option A (consolidate to one) was an alternative; the chosen path (keep both, document clearly) is principled and shipped.
- **D12** ✅ StatusBar per-section separator override — shipped as `StatusBarState::with_left_separator(impl Into<String>)` + `with_center_separator(...)` + `with_right_separator(...)` builders with matching getters. Per-side override takes precedence over the existing global `separator` at render time (`state.<side>_separator.as_deref().unwrap_or(&state.separator)` in `status_bar/mod.rs:850-852`). Layered semantics, not last-call-wins — global stays, per-side overrides layer on top. Three new `Option<String>` fields on `StatusBarState` with `#[serde(default)]` for serialization forward-compat. Methods live in new sibling file `src/component/status_bar/per_side_separators.rs` (same multi-module impl pattern as `pane_layout/title_style.rs` from G4; keeps `mod.rs` under 1000-line cap). Spec PR #493, plan PR #494, implementation PR #495 (`9ceaed2`).
- **D13** ✅ Quit/exit hook — already shipped as `App::on_exit(state: &Self::State)` default-no-op trait method at `src/app/model/mod.rs:257-260`, wired into both terminal runtime (`runtime/terminal.rs:210-211`) and virtual runtime (`runtime/mod.rs:782`). Tests pin default-no-op + custom-override behavior at `model/tests.rs:89` and `:208`. The brief's autosave use case is fully supported by overriding `on_exit`. Likely silently landed as side-effect of D1 runtime work or independent runtime maintenance; no envision-side cadence needed for this item.

### Other follow-ups (new from 2026-05-01 conversation)

- **D3** `Column` width tuning is trial-and-error — no doc on "Length for known-width + Min for flex" pattern, no debug output when columns get clipped. Want canonical doctring + render-time clip warning.
- **D8** No multi-view drill-down example — Roster → Enter → Per-op → Esc → Roster pattern of every dashboard. Want a 100-line example showing two views, modal navigation, per-view key hints, state preservation. (`Router` exists but its scope is unclear.)

### Review 2026-06-25 — D3 + D7 + D8 docs-suite spec + plan (from leadline)

leadline reviewed `docs/superpowers/specs/2026-05-24-docs-suite-d3-d7-d8-design.md`
+ plan + amendments (`d8564a9`, `9b27761`). **Design approved for all three.**
Three items to fold in before the impl cadence runs:

- **D3 — layout-split correctness (real bug).** `detect_clipped_columns` must
  be fed the resolved widths from splitting the *same* `widths` vec the
  renderer resolves (which includes the reserved status `Length(2)` slot),
  then mapped back to user columns offset by `has_status as usize`. Plan
  Step 5 splits only `user_column_widths` over the full `area`, so when
  `has_status` is set the resolved figure is too generous by ~2 cells — a
  column clipped in the real render can read as non-clipped in detection, and
  the "resolved N" in the warning text is wrong. The spec's "Render-path
  integration" (split full `widths.clone()`) is the correct shape; the plan
  call site needs to match it. Dedup-per-`(column, area-width)` and `Min(n)`
  detection are endorsed as-is (better than leadline's once-per-render
  suggestion).
- **D7 — naming.** The "Choosing a Harness" table names
  `Runtime::virtual_terminal`; the public surface is
  `Runtime::<App, _>::virtual_builder(w, h).build()`. Name `virtual_builder`
  consistently so the table entry isn't a dead reference. Decision table +
  dependency-free golden-file recipe otherwise approved.
- **D8 — KeyHints + state-aware events.** The brief asked for "key hints
  update per view," and a stateless `handle_event` with global Up/Down ticks
  the roster selection while in the detail screen — the wrong behavior to
  demonstrate. Switch `drilldown.rs` to `handle_event_with_state` gating
  Up/Down to the active `Screen`, and render a per-view `KeyHints` bar
  (Roster: "↑/↓ select · Enter open · q quit"; PerOp: "Esc back · q quit").

leadline owes no migration for D3/D7/D8 (docs/examples only). Post-merge,
leadline will write golden-frame regression tests for `virtual_preview`'s
frames (D7 removal trigger) and enable envision's `tracing` feature in dev +
a subscriber so the D3 clip warning surfaces. Tracked leadline-side in
`notes/envision_gaps.md` (D3/D7/D8 review notes, 2026-06-25).

### Round 2 (same date) — D3 highlight-symbol false-negative

Re-review of the round-1 amendments (`7a271ba` spec / `e994d40` plan)
surfaced one more accuracy bug in D3, recorded leadline-side in
`notes/envision_gaps.md` commit `7fb9383`. D7 and D8 amendments
fully resolved.

- **D3 (still inaccurate).** envision's `Table` is constructed with
  `.highlight_symbol("> ")` at `render.rs:153` and no explicit
  `.highlight_spacing(...)` call, so ratatui's default
  `HighlightSpacing::WhenSelected` applies — ratatui reserves the
  highlight symbol's display width (2 cells) from the
  column-distribution area BEFORE laying out columns whenever
  `state.selected.is_some()` (the normal case). The round-1 amendment
  split over the full `inner_area.width` without subtracting that
  reservation, so resolved widths come out ~2 cells too generous on
  every render with a selected row — the same false-negative the
  amendment set out to eliminate, reintroduced via the highlight
  symbol. The spec's "highlight-symbol does not affect column layout"
  note was wrong and is being corrected. Fix: subtract 2 from the
  column-distribution area's width when `state.selected.is_some()`
  before the `Layout::horizontal::split`. Best-effort diagnostic, so
  not a release blocker, but defeats the accuracy the D3 amendment
  was for.

Round-2 corrections land as spec amendment `381c47f` + plan amendment
`a0304c8` (mirroring the spec contract in Task 1 Step 5).

## Plan of attack (proposed sequencing)

This is a sketch — treat as draft until reviewed.

1. ~~**Current brainstorm PR**: G1 + G3 + D4 + G7 (Table/Sort/Cell unification, ResourceTable merger, sort vocabulary redesign).~~ ✅ shipped 2026-05-02 via PR #461
2. **High-leverage batch — separate PRs each**:
   - ~~D1 (`App::init` args + `Runtime` builder)~~ ✅ shipped 2026-05-02 via PR #465
   - ~~D2 (`PaneLayout::view_with` closure flow)~~ ✅ shipped 2026-05-02 via PR #469
   - ~~D5 (styled-line primitive)~~ ✅ shipped 2026-05-19 via PR #482
   - D7 (snapshot testing docs/example)
3. **Component polish batch**:
   - ~~G2 + D11 (Table chrome / border type hint / chrome_owned flag)~~ ✅ shipped 2026-05-02 via PR #469 (combined with D2)
   - ~~G4 (PaneLayout per-pane title style)~~ ✅ shipped 2026-05-20 via PR #487
   - ~~G5 (StatusBarItem per-item color)~~ ✅ shipped 2026-05-20 via PR #487
   - ~~G6 (StyledInline composable styles)~~ ✅ shipped 2026-05-20 via PR #491
   - ~~D14 (`paragraph` → `line` rename)~~ ✅ shipped 2026-05-19 via PR #482 (combined with D5)
   - ~~D12 (StatusBar per-section separator)~~ ✅ shipped 2026-05-24 via PR #495
4. **Theme system batch**:
   - ~~D6 (severity helper)~~ ✅ shipped 2026-05-08 via PR #473
   - ~~D9 (theme palette accessor)~~ ✅ shipped 2026-05-08 via PR #473
5. **App lifecycle batch**:
   - ~~D10 (handle_event consolidation)~~ ✅ resolved-via-docs at re-verification 2026-05-24 (current docstrings on handle_event vs handle_event_with_state at app/model/mod.rs:238-255 are crystal-clear; original discovery problem gone)
   - ~~D13 (on_quit hook + save_state docs)~~ ✅ already shipped as App::on_exit at app/model/mod.rs:257-260 (silent shipment; verified during D12 re-verification 2026-05-24)
6. **Docs batch**:
   - D3 (column width pattern doc + clip warning)
   - D8 (multi-view drill-down example)

leadline Claude has offered to write focused briefs (like the sort one) for D1, D2, D5, D7 — accept those before scoping each.

## Removal triggers

Each item in `envision_gaps.md` carries a "Removal trigger" field describing the exact leadline workaround to delete once the corresponding envision-side fix lands. After each PR ships, ping leadline to land the matching workaround removal.
