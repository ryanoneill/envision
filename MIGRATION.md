# Migration Guide

## v0.16.x to v0.17.0

### `App::init` takes args; `RuntimeBuilder` split

`App::init() -> (State, Command<Msg>)` is replaced with `App::init(args: Self::Args) -> (State, Command<Msg>)`. Migration:

| Old | New |
|---|---|
| `fn init() -> (State, Command<Msg>)` | `type Args = (); fn init(_args: ()) -> (State, Command<Msg>)` |
| `static GLOBAL: OnceLock<T>; fn init() { GLOBAL.get()... }` | `type Args = MyArgs; fn init(args: MyArgs) { args.field... }` |
| `RuntimeBuilder::state(state, cmd)` | `RuntimeBuilder::with_args(args)`; move state-building into `init` |
| `AppHarness::with_state(w, h, state, cmd)` | `AppHarness::with_args(w, h, args)`; build state from args inside `init` |
| `AppHarness::with_state_and_config(w, h, state, cmd, cfg)` | `AppHarness::with_args_and_config(w, h, args, cfg)` |

Forgetting `with_args` for a non-`()` Args is now a compile error (via the sealed `OptionalArgs` marker), not a runtime panic.

### Table sort & cell API redesign

`TableMessage::SortBy` / `AddSort` / `ClearSort` are removed and replaced by explicit primitives. `Column::with_comparator` / `comparator` / `SortComparator` are removed and replaced by the reified `Cell { text, style, sort_key }` type + `SortKey` enum. `ResourceTable` and its supporting types are removed in favor of `Table` with an optional row-status column.

| Old | New |
|---|---|
| `TableMessage::SortBy(col)` for header-click intent | `TableMessage::SortToggle(col)` |
| `TableMessage::SortBy(col)` for "always Asc" | `TableMessage::SortAsc(col)` |
| `TableMessage::SortBy(col)` for "always Desc" | `TableMessage::SortDesc(col)` |
| `SortBy(col); SortBy(col)` (init bootstrap to Desc) | `TableState::with_initial_sort(col, Descending)` |
| `TableMessage::AddSort(col)` for tiebreaker click | `TableMessage::AddSortToggle(col)` |
| `TableMessage::AddSort(col)` for "always Asc tiebreaker" | `TableMessage::AddSortAsc(col)` |
| `TableMessage::ClearSort` | `TableMessage::SortClear` |
| `TableMessage::RemoveSort(col)` was already the drop-one-column primitive | `TableMessage::RemoveSort(col)` (unchanged) |
| `Column::with_comparator(numeric_comparator())` | `Cell::number(value)` per cell. Mixed-precision: `Cell::number(value).with_text(format!("{:.2}", value))` |
| `Column::with_comparator(date_comparator())` | `Cell::datetime(value)` per cell |
| `Column::with_comparator(custom_fn)` | `Cell::new(text).with_sort_key(SortKey::...)` per cell |
| `TableRow::cells() -> Vec<String>` | `TableRow::cells() -> Vec<Cell>` (use `Cell::new(s)` or `s.into()`) |
| `ResourceTable*` | `Table` with optional `TableRow::status()` for the status dot |
| `ResourceCell::*` constructors | `Cell::*` (constructors map 1:1) |
| `RowStatus` (formerly in `resource_table`) | `RowStatus` (in `envision::cell`, re-exported at crate root) |

See `docs/superpowers/specs/2026-05-02-table-sort-cell-unification-design.md` for the full design.

### `FileSortDirection` removed; use `table::SortDirection`

`file_browser::FileSortDirection` deleted. `file_browser` now uses `crate::component::table::SortDirection` at every use site (canonical single path — no local re-export at the `file_browser` boundary). The two enums had identical 2-variant Ascending/Descending shape; unification eliminates two-names-for-one-concept.

`SortDirection` also derives `Copy + Default` (where `FileSortDirection` didn't), forcing a getter-shape improvement: `sort_direction()` returns by value.

| Old | New |
|---|---|
| `use envision::component::file_browser::FileSortDirection;` | `use envision::component::SortDirection;` |
| `FileSortDirection::Ascending` | `SortDirection::Ascending` |
| `FileSortDirection::Descending` | `SortDirection::Descending` |
| `FileBrowserOutput::SortChanged(field, FileSortDirection::Ascending)` | `FileBrowserOutput::SortChanged(field, SortDirection::Ascending)` |
| `fn sort_direction(&self) -> &FileSortDirection` | `fn sort_direction(&self) -> SortDirection` (by value; `SortDirection: Copy`) |
| `let dir = *state.sort_direction();` | `let dir = state.sort_direction();` (no deref needed — returns by value) |
| `match state.sort_direction() { FileSortDirection::Ascending => …, FileSortDirection::Descending => … }` | `match state.sort_direction() { SortDirection::Ascending => …, SortDirection::Descending => … }` |

Bonus: `SortDirection::toggle()` is available; use it to replace hand-rolled asc/desc flips.

### `ResourceGaugeState::new` replaced by named-struct + builder

`ResourceGaugeState::new(actual, request, limit)` took three unlabeled positional `f64` arguments; transposing `request` and `limit` was silent. Replaced by two named forms — pick whichever fits your construction site:

**Named-struct single call** (recommended when all three values are known up front — test fixtures, snapshots):

| Old | New |
|---|---|
| `ResourceGaugeState::new(250.0, 500.0, 1000.0)` | `ResourceGaugeState::default().with_values(ResourceValues { actual: 250.0, request: 500.0, limit: 1000.0 })` |

**Fluent builder** (recommended when values are computed independently):

| Old | New |
|---|---|
| `let a = compute_actual(); let r = ..; let l = ..; ResourceGaugeState::new(a, r, l)` | `ResourceGaugeState::default().with_actual(a).with_request(r).with_limit(l)` |

**Accessor symmetry** (this cadence also closes the `set_values`/no-getter gap):

| Old | New |
|---|---|
| `state.actual(); state.request(); state.limit();` (three separate calls) | Still supported. Plus new `state.values() -> ResourceValues` returning all three at once. |
| `state.set_values(a, r, l);` (existing, unchanged) | Existing, unchanged. Getter counterpart is `state.values()`. |

**New type**: `envision::component::ResourceValues { actual, request, limit }` — also available at `envision::prelude::ResourceValues`.

### Component selection accessors unified on `selected_item()`

Six components had divergent shapes for "which is selected?"; unified on canonical `selected_item()` returning `Option<&T>` where the concept fits. Semantic outliers renamed to disambiguate.

| Component | Old | New |
|---|---|---|
| `dropdown` | `state.selected_value()` | `state.selected_item()` (was already the canonical name; the alias is what's deleted) |
| `select` | `state.selected_value()` | `state.selected_item()` (same shape) |
| `heatmap` | `state.selected_value() -> Option<f64>` (returns data value at cursor coordinates) | `state.value_at_selection() -> Option<f64>` (renamed — the `value_` prefix disambiguates from selection-accessor pattern; the coordinate accessor `state.selected() -> Option<(usize, usize)>` is unchanged) |
| `tab_bar` | `state.active_tab() -> Option<&Tab>` | `state.selected_item() -> Option<&Tab>` |
| `tab_bar` | `state.active_tab_mut() -> Option<&mut Tab>` | `state.selected_item_mut() -> Option<&mut Tab>` |
| `tab_bar` | `state.selected() -> Option<usize>` (was: literal alias for `selected_index()`) | `state.selected_index()` |
| `data_grid` | `state.selected()` (was: literal alias for `selected_index()`) | `state.selected_index()` |
| `data_grid` | `state.selected_row()` (was: literal alias for `selected_item()`) | `state.selected_item()` |
| `table` | `state.selected()` (was: literal alias for `selected_index()`) | `state.selected_index()` |
| `table` | `state.selected_row()` (was: literal alias for `selected_item()`) | `state.selected_item()` |
| `tab_bar` | `state.set_selected(idx)` | `state.set_selected_index(idx)` (renamed for symmetry with `selected_index()`) |
| `data_grid` | `state.set_selected(idx)` | `state.set_selected_index(idx)` (renamed for symmetry with `selected_index()`) |
| `table` | `state.set_selected(idx)` | `state.set_selected_index(idx)` (renamed for symmetry with `selected_index()`) |

Grep hint for consumers: search your codebase for `\.selected_value(`, `\.active_tab(`, `\.active_tab_mut(`, `\.selected_row(`, `\.selected(` (the last only on tab_bar / data_grid / table state), and `\.set_selected(` (only on tab_bar / data_grid / table state) — every hit needs to migrate to the new form per the table above.

### `MessageSender<M>` wraps `tokio::sync::mpsc::Sender<M>`

`AppHarness::message_sender()` now returns a first-party `MessageSender<M>` newtype instead of raw `tokio::sync::mpsc::Sender<M>`. Consumer code changes are limited to type spellings; call sites are identical.

| Old | New |
|---|---|
| `let sender: tokio::sync::mpsc::Sender<MyMsg> = harness.message_sender();` | `let sender: envision::MessageSender<MyMsg> = harness.message_sender();` (or use `envision::prelude::MessageSender`) |
| `sender.send(msg).await` — returns `Result<(), tokio::sync::mpsc::error::SendError<MyMsg>>` | `sender.send(msg).await` — returns `Result<(), envision::MessageSendError<MyMsg>>` |
| `sender.try_send(msg)` — returns `Result<(), tokio::sync::mpsc::error::TrySendError<MyMsg>>` | `sender.try_send(msg)` — returns `Result<(), envision::TrySendError<MyMsg>>` |
| `sender.is_closed()` — still available | `sender.is_closed()` — still available (passthrough) |
| `sender.capacity()` — still available | `sender.capacity()` — still available (passthrough) |
| `sender.max_capacity()` — still available | `sender.max_capacity()` — still available (passthrough) |
| `sender.reserve()`, `.send_timeout()`, `.same_channel()`, `.downgrade()`, `.closed()` future | `let tokio_sender = sender.into_inner();` — explicit escape hatch that re-couples to tokio |

`MessageSender<M>` is `Clone + Debug + Send + Sync` (when `M: Send`). Its `send`/`try_send` methods have the same semantics as tokio's Sender (bounded channel, receiver-dropped errors carry the message back).

**Generic parameter note:** `MessageSender<M>` is parameterized on the message type `M`, not on an App type. This means portable helper functions like `fn spawn_watcher<M: Send + 'static>(sender: MessageSender<M>) { ... }` work without depending on envision's `App` trait.

## v0.15.x to v0.16.0

### `DependencyGraph` removed; use `Diagram`

`DependencyGraph` and its supporting types (`GraphNode`, `GraphEdge`, `GraphOrientation`, `NodeStatus`, all `DependencyGraph*` types) are deleted. Replaced by `Diagram`, which provides:

- Sugiyama hierarchical layout with barycenter crossing minimization
- Fruchterman-Reingold force-directed layout
- Spatial keyboard navigation, edge following, search, clusters, minimap
- Viewport pan/zoom, edge styles, node shapes
- Batch buffer rendering and layout caching

`NodeStatus` moved from `dependency_graph::types` to `diagram::types`.

| Old | New |
|---|---|
| `use envision::component::{DependencyGraph, DependencyGraphState};` | `use envision::component::{Diagram, DiagramState};` |
| `DependencyGraphState::new(nodes, edges)` | `DiagramState::hierarchical(nodes, edges)` or `DiagramState::force_directed(nodes, edges)` |
| `GraphNode { id, label }` | `DiagramNode { id, label, .. }` (richer node type) |
| `GraphEdge { from, to }` | `DiagramEdge { from, to, style: EdgeStyle::Solid, .. }` |
| `GraphOrientation::TopDown` | Handled via `LayoutDirection::TopDown` in `LayoutConfig` |
| `use dependency_graph::types::NodeStatus;` | `use diagram::types::NodeStatus;` |

See CHANGELOG `[0.16.0]` entry for the full feature list.

## v0.14.x to v0.15.0

### Message `Clone` bound removed

`Component::Message`, `Component::Output`, and `App::Message` no longer require `Clone`. Existing code with `#[derive(Clone)]` keeps compiling — the derive is now optional, not required.

If you relied on the implied `Clone` bound in generic code (e.g., `fn foo<A: App>() where A::Message: Clone`), remove the bound or add it explicitly.

### Runtime constructors replaced by builder

The 12 `Runtime::new_terminal()` / `virtual_terminal()` / `with_backend()` constructors are removed. Use the builder instead:

```rust
// Before
let rt = Runtime::<MyApp, _>::new_terminal()?;

// After
let rt = Runtime::<MyApp, _>::terminal_builder()?.build()?;
```

```rust
// Before
let rt = Runtime::<MyApp, _>::virtual_terminal(80, 24)?;

// After
let rt = Runtime::<MyApp, _>::virtual_builder(80, 24).build()?;
```

```rust
// Before
let rt = Runtime::<MyApp, _>::with_backend_state_and_config(
    backend, state, cmd, config,
)?;

// After
let rt = Runtime::<MyApp, _>::builder(backend)
    .state(state, cmd)
    .config(config)
    .build()?;
```

### `ConversationView::view_from()` removed

`view_from` and the public `MessageSource` trait are removed. Use the standard `ConversationView::view()` with messages stored in `ConversationViewState`.

### `with_markdown()` requires the `markdown` feature

Calling `with_markdown(true)` or `set_markdown_enabled(true)` without the `markdown` Cargo feature is now a compile error. Add `features = ["markdown"]` to your Cargo.toml or use the `full` feature (included in defaults).

### API consistency renames

| Before | After |
|--------|-------|
| `CollapsibleState::expanded()` | `is_expanded()` |
| `TabBarState::active()` / `active_index()` / `set_active()` / `with_active()` | `selected()` / `selected_index()` / `set_selected()` / `with_selected()` |
| `with_regex()` (log_viewer) | `with_use_regex()` |
| `with_percentages()` (multi_progress) | `with_show_percentages()` |
| `with_auto_remove()` (multi_progress) | `with_auto_remove_completed()` |
| `with_show_line_numbers()` (diff_viewer) | `with_line_numbers()` |
| `with_legend()` (chart) | `with_show_legend()` |
| `with_timestamps()` (conversation_view, status_log, log_viewer) | `with_show_timestamps()` |
| `with_role_labels()` (conversation_view) | `with_show_role_labels()` |

### New features

- `RuntimeBuilder` — `Runtime::terminal_builder()?.theme(t).tick_rate(d).build()?`
- `envision::terminal::restore()` — standalone terminal cleanup for panic hooks
- `InputMode::{Desktop, Readline}` on `LineInput` — `state.with_input_mode(InputMode::Readline)`
- `StepIndicator` per-step-index styles — `state.with_step_style(0, Style::default().fg(Color::Cyan))`
- `docs/CHOOSING.md` — component decision tree

---

**Historical migration paths:** For upgrades from versions v0.13.x and earlier, see [`MIGRATION-legacy.md`](MIGRATION-legacy.md).
