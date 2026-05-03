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

- **D2** `PaneLayout` consumer flow has magic-number coupling — `state.layout` → `PaneLayout::view` → `rect.inner(Margin{1,1})`. Hardcoded `Margin{1,1}` knowledge breaks silently if envision changes border thickness. Want `PaneLayout::view_with(state, ctx, |pane_id, child_ctx| ...)` — envision computes inner rects.
- **D5** No "render styled Line into Rect" primitive — six types and three method calls to draw a single styled line. Want `envision::render::line(frame, area, line, theme)` or a tiny `StyledLine` component.
- **D7** View-snapshot testing is undocumented — `AppHarness`/`TestHarness` exist but no docs explaining when to reach for them. Want documented "render at W×H, dispatch event sequence, snapshot cell buffer" pattern.

### Other follow-ups (pre-existing gaps)

- **G2 / D11** Table inner border style + chrome ownership — `Table` renders square borders inside a rounded outer `Block`, no way to suppress inner chrome. Want `BorderType` hint on `Table`, or a `chrome_owned: true` flag on `RenderContext` so children skip their own chrome.
- **G4** `PaneLayout` per-pane title style — title inherits border style; no `PaneConfig::with_title_style(Style)` or pre-styled `Vec<Span>` form.
- **G5** `StatusBarItem::with_style(StatusBarStyle)` enum is closed — no per-item arbitrary `Color` or full `Style` override. Want `with_color(Color)` and `with_style_override(Style)`.
- **G6** `StyledInline` cannot combine color + modifier — leaf variants only (`Bold`, `Italic`, `Colored{..}`). Want a single composable `Styled { text, style: InlineStyle }` variant with the leaf forms as constructors.

### Other follow-ups (new from 2026-05-01 conversation)

- **D3** `Column` width tuning is trial-and-error — no doc on "Length for known-width + Min for flex" pattern, no debug output when columns get clipped. Want canonical doctring + render-time clip warning.
- **D6** No severity helper in `Theme` — every dashboard rewrites `severity_color_for_ratio` / `severity_status_style`. Want `theme.severity_style(value, &[(threshold, Severity)])` and a `Severity::Good|Mild|Bad|Critical` enum that maps through the theme. Bonus: makes severity coloring consistent across status bar, cells, gauges.
- **D8** No multi-view drill-down example — Roster → Enter → Per-op → Esc → Roster pattern of every dashboard. Want a 100-line example showing two views, modal navigation, per-view key hints, state preservation. (`Router` exists but its scope is unclear.)
- **D9** `Theme` color access is uneven — for colors outside named slots, consumers reach past the `Theme` abstraction to raw `CATPPUCCIN_*` constants, breaking theme-swap. Want `theme.color(NamedColor::Lavender)` enum-keyed or `theme.palette().lavender()` accessor.
- **D10** `App::handle_event` vs `handle_event_with_state` — both exist on the trait, unclear which is canonical and when to override which. Want consolidation to one method or much clearer doc.
- **D12** `StatusBarState::with_separator` is global per-bar — no per-section override. Want per-section separator config or per-item-trailing-separator property.
- **D13** No quit hook — `Command::quit()` exists but no `App::on_quit(state) -> Result<()>` for autosave. Relationship between `load_state` re-export and quit is undocumented. Want documented `on_quit` lifecycle hook.
- **D14** `StyledContent::paragraph(...)` produces a single line, not a wrapped paragraph — name conflicts with intuition. Want rename to `line(...)`; reserve `paragraph` for wrapped block-level text.

## Plan of attack (proposed sequencing)

This is a sketch — treat as draft until reviewed.

1. ~~**Current brainstorm PR**: G1 + G3 + D4 + G7 (Table/Sort/Cell unification, ResourceTable merger, sort vocabulary redesign).~~ ✅ shipped 2026-05-02 via PR #461
2. **High-leverage batch — separate PRs each**:
   - ~~D1 (`App::init` args + `Runtime` builder)~~ ✅ shipped 2026-05-02 via PR #465
   - D2 (`PaneLayout::view_with` closure flow)
   - D5 (styled-line primitive)
   - D7 (snapshot testing docs/example)
3. **Component polish batch**:
   - G2 + D11 (Table chrome / border type hint / chrome_owned flag)
   - G4 (PaneLayout per-pane title style)
   - G5 (StatusBarItem per-item color)
   - G6 (StyledInline composable styles)
   - D14 (`paragraph` → `line` rename)
   - D12 (StatusBar per-section separator)
4. **Theme system batch**:
   - D6 (severity helper)
   - D9 (theme palette accessor)
5. **App lifecycle batch**:
   - D10 (handle_event consolidation)
   - D13 (on_quit hook + save_state docs)
6. **Docs batch**:
   - D3 (column width pattern doc + clip warning)
   - D8 (multi-view drill-down example)

leadline Claude has offered to write focused briefs (like the sort one) for D1, D2, D5, D7 — accept those before scoping each.

## Removal triggers

Each item in `envision_gaps.md` carries a "Removal trigger" field describing the exact leadline workaround to delete once the corresponding envision-side fix lands. After each PR ships, ping leadline to land the matching workaround removal.
