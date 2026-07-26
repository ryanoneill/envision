# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Breaking Changes

#### Selection accessors completed on `selected_index()`

Cadence A unified selection accessors across six components. This release finishes the getter/setter half: every component whose selection accessor was spelled `selected` now spells it `selected_index`, and every corresponding mutator is `set_selected_index`. The `with_selected` builder leg is untouched and remains deferred.

The motivation is that `selected` was ambiguous about return type. Four different signatures shared the spelling: `Option<usize>` (15 components), `Option<(usize, usize)>` (`heatmap`), bare `usize` (`box_plot`), and a `bool` builder (`annotation::widget`). The `selected_index` / `selected_item` / `value_at_selection` system makes the name predict the type.

**Aliases deleted** (body was exactly `self.selected_index()`) — `accordion`, `dropdown`, `file_browser`, `loading_list`, `menu`, `metrics_dashboard`, `radio_group`, `searchable_list`, `select`, `selectable_list`, `tabs`, `tree`.

**Primary accessors renamed** (no `selected_index()` sibling existed) — `alert_panel`, `diagram`, `multi_progress`, `box_plot`.

**Setters renamed** `set_selected` → `set_selected_index` on all twelve components that had one.

`box_plot::selected_index()` keeps its bare `usize` return, matching the existing `flame_graph::selected_index()`. Whether it should become `Option<usize>` is a separate question, deferred.

`heatmap::selected()` is **unchanged** — it returns `Option<(usize, usize)>` coordinates, not an index, and was disentangled from this pattern during Cadence A.

See `MIGRATION.md` § *v0.17.x to v0.18.0* for the full before/after table.

### Added

#### Harness types available from the prelude

`envision::prelude::*` now re-exports the seven harness types available at the crate root. Previously only `AppHarness`, `MessageSender`, and `TestHarness` were included, so consumers matching on `TrySendError::{Full, Closed}`, destructuring `MessageSendError`, or using `Assertion` / `Snapshot` needed a second explicit import.

Added: `Assertion`, `MessageSendError`, `Snapshot`, `TrySendError`.

### Known Deferred Findings

> Supersedes the Known Deferred Findings block under `[0.17.0]`. The Cadence D item listed there — selection-accessor aliases, including `box_plot` — is **closed** by this release.

- **Selection-index accessors under domain-specific names.** `chart::active_series`, `log_correlation::active_stream`, `diff_viewer::current_hunk`, `step_indicator::active_step_index`, `paginator::current_page`, `breadcrumb::focused_index`. Surveyed during Cadence D and deliberately excluded: each name carries domain meaning that a generic rename would erase. Open question for a future cadence.
- **`box_plot::selected_index()` returns a bare `usize`**, so "no selection" is unrepresentable. Whether it should become `Option<usize>` is a semantic question, deferred — the naming half closed in this release.
- **The `with_selected` builder leg was not renamed.** `menu`, `tabs`, `tree`, `loading_list`, `selectable_list`, `radio_group`, `table`, and `tab_bar` still expose `with_selected(i)` alongside `set_selected_index()` and `selected_index()`, so one type now carries two spellings of the same concept. Renaming it to `with_selected_index` is a second breaking change that belongs in its own cadence, not bolted onto this one.
- **Three `selected_item()` one-line aliases survive** — `file_browser::selected_item()` → `selected_entry()`, `tree::selected_item()` → `selected_node()`, `accordion::selected_item()` → `focused_panel()`. These are the same "pure redundancy" shape this release deleted twelve times, kept only because `selected_item()` is the canonical cross-component spelling Cadence A established. Delete-or-keep is an open question.
- **`accordion::selected_index()` remains a convenience alias for `focused_index()`.** It performs real work (Option-normalizing the empty case) so it is not redundant, but the indirection stands.
- **`compact_str` adoption is sporadic** — 2 non-test source files (`src/component/cell.rs`, `src/backend/cell/mod.rs`). Needs a commit-or-drop decision.
- **Naming outliers** — `is_checked`, `label_text` (the `tab_bar` setter outlier previously grouped here is closed by this release). Plus `restore_terminal` → `restore`, `AppShell` placement in the README component table, five files near the 1000-line cap, and snapshot-coverage concentration in ~20 of 74 components.

## [0.17.0] - 2026-07-26

### Breaking Changes

#### `App::init` takes args; `RuntimeBuilder` split

`App::init() -> (State, Command<Msg>)` is replaced with `App::init(args: Self::Args) -> (State, Command<Msg>)`.

- `App` trait gains `type Args` (no default; explicit `type Args = ();` required for no-args apps on stable Rust).
- The panicking default impl of `init` is deleted; `init` is now required.
- `RuntimeBuilder::state(state, cmd)` is **deleted**. Its role is subsumed by `with_args` plus a real `init` impl.
- `AppHarness::with_state` and `AppHarness::with_state_and_config` are **deleted**; replaced by `AppHarness::with_args` and `AppHarness::with_args_and_config`.
- New: `RuntimeBuilder::with_args(args) -> ConfiguredRuntimeBuilder<A, B>` carries the args into a typestate-lite builder whose `build()` is unconditionally available.
- `RuntimeBuilder::build()` is now only available when `A::Args: OptionalArgs` (sealed marker, implemented only for `()`). Forgetting `with_args` for non-`()` Args is a compile error, not a runtime panic.
- `AppHarness::new` and `AppHarness::with_config` similarly require `A::Args: OptionalArgs`.

Tracks leadline gap D1. See `docs/superpowers/specs/2026-05-02-app-init-args-design.md`. Migration table lifted verbatim into `MIGRATION.md#v016x-to-v0170`.

#### Table sort & cell API redesign

Removed:
- `TableMessage::SortBy`, `TableMessage::AddSort`, `TableMessage::ClearSort` — replaced by explicit primitives.
- `Column::with_comparator` / `Column::comparator` / `SortComparator` / `numeric_comparator` / `date_comparator`.
- `ResourceTable`, `ResourceRow`, `ResourceCell`, `ResourceColumn`, `ResourceTableState`, `ResourceTableMessage`, `ResourceTableOutput`.

Added:
- `Cell { text, style, sort_key }` — unified cell type for all tabular components.
- `SortKey` enum (`String`, `I64`, `U64`, `F64`, `Bool`, `Duration`, `DateTime`, `None`).
- `TableMessage::{SortAsc, SortDesc, SortToggle, SortClear, RemoveSort, AddSortAsc, AddSortDesc, AddSortToggle}` — explicit sort primitives, each carrying the column index.
- `Column::with_default_sort(SortDirection)` — declare per-column natural direction.
- `TableState::with_initial_sort(col, dir)` and `with_initial_sorts(Vec<InitialSort>)`.
- `TableRow::status()` (default `RowStatus::None`) — optional row-status dot column.
- `CellStyle` enum and per-cell styling in `Table` rendering.

Tracks leadline gaps G1 + G3 + G7. See `docs/superpowers/specs/2026-05-02-table-sort-cell-unification-design.md`. Migration table in `MIGRATION.md#v016x-to-v0170`.

#### `FileSortDirection` removed

`file_browser::FileSortDirection` deleted. `file_browser` uses `table::SortDirection` (same 2-variant Ascending/Descending shape). `sort_direction()` getter signature changes to return by value (SortDirection is Copy).

See `MIGRATION.md#v016x-to-v0170` for the full before/after table.

#### `ResourceGaugeState::new` replaced by named-struct + builder

`ResourceGaugeState::new(actual, request, limit)` (three unlabeled positional f64 args) deleted. Replaced by `ResourceGaugeState::default().with_values(ResourceValues { actual, request, limit })` single-call form or fluent builder chain (`with_actual`, `with_request`, `with_limit`).

New public type: `envision::component::ResourceValues { actual, request, limit }`.
New accessor: `state.values() -> ResourceValues` (closes accessor-symmetry gap with `set_values`).

See `MIGRATION.md#v016x-to-v0170`.

#### Component selection accessors unified on `selected_item()`

Six components had divergent accessor shapes for "which is selected?" — literal aliases (`selected_value()` / `selected_row()` / `selected()`), semantic outliers (`active_tab()` on tab_bar), and type-incoherent overloads (heatmap's `selected_value() -> f64` returning the data value at coordinates, not the selection itself). Unified on canonical `selected_item()` returning `Option<&T>` where the concept fits; renamed the semantic outliers to disambiguate:

- `dropdown::selected_value()` — deleted; use `selected_item() -> Option<&str>`
- `select::selected_value()` — deleted; use `selected_item() -> Option<&str>`
- `heatmap::selected_value() -> Option<f64>` — renamed to `value_at_selection() -> Option<f64>` (returns data value at cursor coords; not a selection accessor)
- `tab_bar::selected()` — deleted; use `selected_index()`
- `tab_bar::active_tab() -> Option<&Tab>` — renamed to `selected_item() -> Option<&Tab>`
- `tab_bar::active_tab_mut() -> Option<&mut Tab>` — renamed to `selected_item_mut() -> Option<&mut Tab>`
- `data_grid::selected()` — deleted; use `selected_index()`
- `data_grid::selected_row()` — deleted; use `selected_item()`
- `table::selected()` — deleted; use `selected_index()`
- `table::selected_row()` — deleted; use `selected_item()`
- `tab_bar::set_selected()` — renamed to `set_selected_index()` for symmetry
- `data_grid::set_selected()` — renamed to `set_selected_index()` for symmetry
- `table::set_selected()` — renamed to `set_selected_index()` for symmetry

Closes audit finding #6 (`selected_value` / `selected_item` / `active_tab` divergence). See `MIGRATION.md#v016x-to-v0170` for the full migration table.

#### `MessageSender<M>` replaces `tokio::sync::mpsc::Sender` in AppHarness surface

`AppHarness::message_sender()` now returns a first-party `MessageSender<A::Message>` newtype instead of raw `tokio::sync::mpsc::Sender<A::Message>`. Consumers no longer need `tokio` as a direct dependency to use the accessor. Full tokio Sender semantics preserved through passthrough methods; an explicit `MessageSender::into_inner()` escape hatch is available for consumers needing `reserve` / `send_timeout` / `same_channel` / `downgrade` / `closed()` future.

Closes audit finding #8 (dependency leakage on AppHarness surface). See `MIGRATION.md#v016x-to-v0170` for the migration table.

### Added

#### Chrome ownership protocol (G2 + D2 + D11)

- `PaneLayout::view_with(state, ctx, |pane_id, child_ctx| ...)` — closure-based renderer; envision owns inner-rect computation.
- `RenderContext::chrome_owned` flag — Table, StyledText, and other chrome-drawing components consult it and skip their outer Block when embedded.
- 35+ chrome-drawing components audited to consult the flag (Table, StyledText, LogViewer, ScrollableText, ScrollView, MarkdownRenderer, ConversationView, DataGrid, MetricsDashboard, KeyHints, StatusLog, EventStream, LogCorrelation, TerminalOutput, FileBrowser, FlameGraph, SearchableList, SelectableList, AlertPanel, HelpPanel, TitleCard, BoxPlot, Histogram, Heatmap, Treemap, Diagram, Sparkline, Chart, Timeline, Calendar, UsageDisplay, MultiProgress, StepIndicator, Tabs, TabBar, CodeBlock, DiffViewer, StatusBar, and others).

Tracks leadline gaps G2 + D2 + D11.

#### `App::on_exit` shipped

- `App::on_exit(state: &Self::State)` default-no-op trait method. Wired into both terminal and virtual runtimes. Consumers override for autosave / cleanup.

Tracks leadline gap D13.

#### Theme palette + severity helper (D6 + D9)

- `Severity` enum (`Good | Mild | Bad | Critical`, `#[non_exhaustive]`).
- `Severity::from_thresholds(value, &[(threshold, severity)])` first-match-wins bucketer.
- `Theme::severity_color(Severity) -> Color`, `Theme::severity_style(Severity) -> Style`.
- `NamedColor` enum (26 variants, `#[non_exhaustive]`).
- `Palette` struct — one `Color` field per `NamedColor`.
- `Theme::color(NamedColor) -> Color` accessor.
- Per-palette module extraction (`nord.rs`, `dracula.rs`, `solarized.rs`, `gruvbox.rs`) mirroring `catppuccin.rs`.

Raw `pub const` color constants (`CATPPUCCIN_*`, `NORD0`–`NORD15`, `DRACULA_*`, `SOLARIZED_*`, `GRUVBOX_*`) marked `#[deprecated(since = "0.17.0")]` — accessible during transition window.

#### `CellStyle::Severity(Severity)` (D15)

- Severity-aware cells reach the active theme at render time via `theme.severity_style(*sev)` in `cell_style_to_ratatui`.
- `Cell::severity(text, sev)` constructor + `Cell::with_severity(sev)` builder.
- `CellStyle` gains `#[non_exhaustive]`.

#### StyledText DX: `styled_line` primitive + `paragraph` → `line` rename (D5 + D14)

- `envision::render::styled_line(frame, area, &[StyledInline], theme)` free function (`src/render.rs`). Re-exported at `envision::styled_line`. Module + re-export gated on `display-components`.
- `StyledContent::paragraph(...)` deleted; `StyledContent::line(...)` replaces it. Also `StyledBlock::Paragraph` → `StyledBlock::Line`.

#### Per-component style overrides (G4 + G5)

- `PaneConfig::with_title_style(Style)` + `title_style() -> Option<Style>` — pane title styling independent of border.
- `StatusBarItem::with_color(Color)` + `with_style_override(Style)` — layered semantics, not last-call-wins; restores the four-stop severity ramp for StatusBar consumers.

Sibling file split: `pane_layout/title_style.rs`.

#### `StyledInline` composable styles (G6)

- 3-variant enum: `Plain | Code | Styled { text, style: InlineStyle }`.
- New `InlineStyle` struct with 6 optional dimensions (`fg`, `bg`, `bold`, `italic`, `underlined`, `strikethrough`) and 7 `const fn` builder methods.
- Two-layer constructor surface: `StyledInline::styled(text, style)` + 5 leaf helpers (`bold`, `italic`, `underlined`, `strikethrough`, `colored`).

Removed leaf variants: `Bold`, `Italic`, `Underline`, `Strikethrough`, `Colored`.

#### D3 column clip warning

- `Column::new` docstring gains canonical Length+Min multi-column idiom.
- `pub(crate) detect_clipped_columns(columns, resolved_widths) -> Vec<ClippedColumn>` helper mirrors the full ratatui 0.29 Table width formula (border / selection / column_spacing / has_status offset).
- `RefCell<ClipWarnState>` on `TableState` for interior-mutability dedup keyed by `(column index, area width)`. Terminal-resize re-arm.
- Emission via `tracing::warn!` feature-gated on `tracing`. `impl ClipKind` also gated so default-feature builds stay warning-free.

#### D7 harness compare-and-contrast + golden-file snapshot recipe

- `src/harness` module docs gain "Choosing a Harness" decision table comparing `TestHarness` / `AppHarness` / `Runtime::virtual_builder`.
- `src/harness/snapshot` module docs gain runnable golden-file recipe (`update_golden` / `assert_matches_golden` / `unified_diff`), dependency-free, `insta` linked as upgrade.

#### D8 drilldown example + Router-vs-state-enum docs

- `examples/drilldown.rs` — master+detail pattern via Screen enum, per-view `KeyHints`, `handle_event_with_state` for screen-gated bindings, selection preservation.
- `src/component/router/mod.rs` module docs gain "Choosing Router vs. an in-state enum" section.

#### StatusBar per-side separator overrides (D12)

- `StatusBarState::with_left_separator(...)` / `with_center_separator(...)` / `with_right_separator(...)` — per-side overrides layer on top of the global `separator`.
- Sibling file split: `status_bar/per_side_separators.rs`.

#### `resource_gauge` builder + accessor closure (this cadence, Unit 3)

- New public type: `ResourceValues { actual: f64, request: f64, limit: f64 }` (`Clone`, `Copy`, `Debug`, `Default`, `PartialEq`).
- `ResourceGaugeState::with_values(ResourceValues) -> Self` — named-struct single-call constructor.
- `ResourceGaugeState::with_actual(f64) -> Self` / `with_request(f64) -> Self` / `with_limit(f64) -> Self` — fluent builder.
- `ResourceGaugeState::values(&self) -> ResourceValues` — matching accessor for the existing `set_values` multi-field mutator; closes the audit's 9/9 scorecard gap.
- New `examples/resource_gauge.rs` — K8s pod-quota shape.

#### `MessageSender<M>` + `MessageSendError<T>` + `TrySendError<T>` (this cadence, Unit 2)

New first-party types in `envision::harness` (re-exported at crate root and prelude):

- `MessageSender<M: Send + 'static>` — newtype wrapper around `tokio::sync::mpsc::Sender<M>`. Full passthrough surface: `send()`, `try_send()`, `is_closed()`, `capacity()`, `max_capacity()`, plus `into_inner()` explicit escape hatch. `Clone + Debug + Send + Sync` (when `M: Send`).
- `MessageSendError<T>(pub T)` — returned by `send()` when the receiver has been dropped. Carries the message back so the caller can inspect it. Implements `Debug + Display + Error`.
- `TrySendError<T>::{Full(T), Closed(T)}` — returned by `try_send()`. Preserves tokio's Full/Closed distinction so consumers can retry-on-full or exit-on-closed with a match arm. Includes `into_inner(self) -> T` extractor.

### Changed

#### Chrome ownership protocol

- `Table`, `LogViewer`, `ScrollView`, `ScrollableText`, `MarkdownRenderer`, `ConversationView`, `DataGrid`, `MetricsDashboard` (per-cell only), and 27 others skip their outer Block when `RenderContext::chrome_owned == true`.
- Consumers embedding any of them get correct behavior without further envision changes.

#### `examples/router.rs` refresh (D8)

- Screen-render bodies now use `PaneLayout::view_with` chrome instead of raw `ratatui::widgets::Paragraph + Block::borders`. No behavior change; better envision-component showcase.

### Known Deferred Findings

The 2026-07-04 audit surfaced items that remain deferred beyond v0.17.0:

- **`selected()` alias on ~15 additional components** (accordion, tabs, radio_group, searchable_list, selectable_list, tree, menu, box_plot, loading_list, alert_panel, and others) — these components have `selected() -> Option<usize>` as a literal alias for `selected_index()`, similar to the aliases just removed from tab_bar/data_grid/table in this release but without the additional divergence that motivated their inclusion this round. Scheduled for **Cadence D** (v0.18+): finish the consistency sweep across the remaining alias sites.

If audit findings #6 (selected_value/selected_item/active_tab across 5 originally-flagged components) and #8 (dependency leakage in 8 public signatures) previously appeared here, they were closed by the v0.17.0 consistency-cleanup cadence (see Breaking Changes / Added subsections above).

## [0.16.0] - 2026-04-20

### Breaking

- **`DependencyGraph` removed.** Replaced entirely by `Diagram`, which
  provides Sugiyama hierarchical and Fruchterman-Reingold force-directed
  layout, crossing minimization, spatial navigation, edge following,
  clusters, search, minimap, viewport pan/zoom, and batch buffer
  rendering. `GraphNode`, `GraphEdge`, `GraphOrientation`, and all
  `DependencyGraph*` types are removed. `NodeStatus` is now defined in
  `diagram::types`. (#437–#445, #451)

### Added

- **`Diagram` component** — world-class graph visualization with:
  - Sugiyama hierarchical layout with barycenter crossing minimization
  - Fruchterman-Reingold force-directed layout with incremental stability
  - Spatial keyboard navigation (arrow keys select nearest node)
  - Edge following with multi-target picker
  - Node search (`/` to filter by ID or label)
  - Cluster grouping with expand/collapse
  - Minimap for large graphs
  - Viewport pan/zoom (H/J/K/L, +/-, 0 to fit)
  - Edge styles (solid, dashed, dotted) with arrowheads and corner chars
  - Node shapes (rectangle, rounded rectangle, diamond)
  - Self-loop and bidirectional edge rendering
  - Batch buffer writes for edges (dramatically faster than per-cell widgets)
  - Layout caching (recomputes only on data changes)
  - Performance: 100 nodes in ~250µs
  (#437–#445)
- **24-bit RGB color support** in ANSI parser — `ESC[38;2;r;g;b;m`
  foreground and `ESC[48;2;r;g;b;m` background. (#447)
- **Diagram benchmark** in CI — 10/50/100 nodes at two terminal sizes. (#446)
- **Diagram performance guide** in module docs — layout caching, viewport
  culling, benchmark numbers, force-directed complexity. (#452)
- **Diagram in CHOOSING.md** — component selection guide updated. (#449)

### Fixed

- **`terminal_output` example** now runs as interactive terminal app
  (was virtual terminal with no colors). (#445, #448)

### Internal

- Property-based testing (proptest) for ANSI parser — 6 properties
  including round-trip text preservation and no-panics on arbitrary
  input. (#450)
- Property-based testing for Sugiyama layout — 8 properties including
  no-overlap, bounding box containment, cycle/self-loop handling. (#453)
- Diagram integration and stress tests — 500-node render, navigation,
  edge following, search, full lifecycle. (#454)
- Updated SECURITY.md with ANSI parser hardening details. (#455)
- Audit grade: A (4.09 GPA), scorecard 9/9, 25/25 categories A or above.

## [0.15.1] - 2026-04-18

### Added

- **Audit scorecard** — `envision-audit scorecard` subcommand with 9 pass/fail
  quality metrics for tracking release health. (#429)
- **16 missing getter methods** for accessor symmetry — `role_style()`,
  `status_style()`, `step_style()`, `y_range()`, `filter()`, `counter_value()`,
  `gauge_value()`, `text_value()`, `is_status()`, `color()` on GraphNode,
  GraphEdge, TimelineEvent, TimelineSpan. (#433)
- **`CalendarState` Default impl** — returns January 1970. (#432)
- **107 doc tests** across 14 components bringing public API doc test coverage
  to 100% (1713/1713 methods). (#428, #431, #434, #435)

### Fixed

- **Windows 1.85 CI stack overflow** — merged doctest binary exceeded Windows
  1 MB default stack; set `RUSTDOCFLAGS=-C link-args=/STACK:16777216` for
  Windows doc test step. (#428)
- **6 clippy lints** from newer stable toolchain — `sort_by` → `sort_by_key`,
  `collapsible-match` in status_bar, terminal_output, component_showcase. (#433)
- **Audit tool false positives** — generic manual trait impls
  (`impl<T> PartialEq for FooState<T>`) were not detected; fixed with
  line-by-line scanner. (#432)
- **Audit tool scope** — `read_non_test_sources` now scans all public API
  files (mod.rs + `pub mod` + `pub use` re-export modules), not just mod.rs,
  while correctly excluding private helper modules. (#433, #436)

### Internal

- **8 components refactored** to stay under 1000-line limit by extracting
  State impl blocks into state.rs/builders.rs files: step_indicator,
  conversation_view, chart, table, histogram, box_plot, data_grid,
  usage_display. (#430)
- Scorecard: 9/9 checks passing (files, accessors, doc tests, derives,
  unsafe, clippy).

## [0.15.0] - 2026-04-13

### Breaking

- **Message `Clone` bound removed** from `Component::Message`,
  `Component::Output`, `App::Message`, `Command::subscribe`, and
  `WorkerBuilder::spawn`. The bound was never exercised — removal is
  backward-compatible for users who keep `#[derive(Clone)]`. (#420)
- **`ConversationView::view_from()` removed** along with the public
  `MessageSource` trait. The API was unused by its target customer. (#423)
- **`with_markdown()` and `set_markdown_enabled()` gated on `markdown`
  feature.** Calling them without the feature is now a compile error
  instead of a silent no-op. (#424)
- **12 old `Runtime` constructors deleted.** `new_terminal()`,
  `terminal_with_config()`, `new_terminal_with_state()`,
  `terminal_with_state_and_config()`, `virtual_terminal()`,
  `virtual_terminal_with_config()`, `virtual_terminal_with_state()`,
  `virtual_terminal_with_state_and_config()`, `with_backend()`,
  `with_backend_and_config()`, `with_backend_and_state()`, and
  `with_backend_state_and_config()` (now `pub(crate)`) are removed.
  Use `Runtime::builder()`, `Runtime::terminal_builder()`, or
  `Runtime::virtual_builder()` instead.
- **API consistency sweep — 11 renames:** (#425)
  - `CollapsibleState::expanded()` removed; use `is_expanded()` instead
    (matches accordion/tree/span_tree pattern).
  - `TabBarState::with_active()` renamed to `with_selected()`,
    `active_index()` to `selected_index()`, `active()` to `selected()`,
    `set_active()` to `set_selected()` (matches `TabsState` convention).
  - `LogViewerState::with_regex()` renamed to `with_use_regex()`
    (matches getter `use_regex()` and setter `set_use_regex()`).
  - `MultiProgressState::with_percentages()` renamed to
    `with_show_percentages()` (matches `set_show_percentages()`).
  - `MultiProgressState::with_auto_remove()` renamed to
    `with_auto_remove_completed()` (matches `set_auto_remove_completed()`).
  - `DiffViewerState::with_show_line_numbers()` renamed to
    `with_line_numbers()` (matches code_block, text_area, terminal_output).
  - `ChartState::with_legend()` renamed to `with_show_legend()`
    (matches getter `show_legend()` and setter `set_show_legend()`).
  - `ConversationViewState::with_timestamps()` renamed to
    `with_show_timestamps()` (matches setter `set_show_timestamps()`).
  - `ConversationViewState::with_role_labels()` renamed to
    `with_show_role_labels()` (matches setter `set_show_role_labels()`).
  - `StatusLogState::with_timestamps()` renamed to
    `with_show_timestamps()` (matches setter `set_show_timestamps()`).
  - `LogViewerState::with_timestamps()` renamed to
    `with_show_timestamps()` (matches setter `set_show_timestamps()`).

### Added

- `RuntimeBuilder<A, B>` builder pattern for constructing `Runtime`
  instances. Three entry points: `Runtime::builder(backend)` for any
  backend, `Runtime::terminal_builder()` for real terminals, and
  `Runtime::virtual_builder(w, h)` for virtual terminals. Supports
  `.state()`, `.config()`, `.tick_rate()`, `.frame_rate()`,
  `.max_messages()`, and `.channel_capacity()` builder methods. (#421)
- `envision::terminal::restore()` standalone function for terminal
  cleanup in panic handlers. (#422)
- `docs/CHOOSING.md` component decision tree. (#422)
- `InputMode::{Desktop, Readline}` for `LineInput` keybinding mode.
  Desktop mode (default) uses standard keybindings; Readline mode adds
  Emacs-style shortcuts (Ctrl-A, Ctrl-E, Ctrl-K, etc.). (#426)
- `LineInputMessage::DeleteToEnd` for readline Ctrl-K. (#426)
- `StepIndicatorState::with_step_style(index, style)` per-step-index
  style overrides. (#417)
- `StepIndicatorState::with_status_style(status, style)` per-status
  style overrides (renamed from `with_step_style`). (#417)

---

**Historical entries:** For versions v0.14.x and earlier, see [`CHANGELOG-legacy.md`](CHANGELOG-legacy.md).
