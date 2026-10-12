# Migration Guide

## v0.17.x to v0.18.0

Minimum supported Rust version is now 1.88, required by ratatui 0.30 (see *ratatui 0.30 upgrade* below).

### ratatui 0.30 upgrade

The sections below follow the order in which a crate upgrading from v0.17 meets each change: dependency resolution first, then compile errors, then behaviour.

#### 1. Minimum versions

| | v0.17 | v0.18 |
|---|---|---|
| Rust (MSRV) | 1.85 | **1.88** |
| `ratatui` | 0.29 | **0.30.2** |
| `crossterm` | 0.28 | **0.29** |

If your crate also depends on ratatui 0.29 directly, Cargo builds two ratatuis and types stop matching:

```text
expected `ratatui::Frame`, found `ratatui::Frame`
note: perhaps two different versions of crate `ratatui` are being used?
```

(abridged)

Either fix works:

```rust
// Fix A (preferred): import ratatui through envision; it is always envision's ratatui.
use envision::ratatui::widgets::Paragraph; // was: use ratatui::widgets::Paragraph;
```

```toml
# Fix B: keep a direct dependency, with a requirement compatible with envision's (>= 0.30.2, < 0.31).
ratatui = "0.30.2" # was: "0.29"
```

**A direct `crossterm` dependency must move to 0.29 too**, or be replaced by `envision::ratatui::crossterm`. Unlike ratatui, a leftover crossterm 0.28 compiles silently: Cargo builds both crossterms, each with its own event reader and raw-mode state. That matters for `TerminalHook` code that calls `crossterm::execute!` or reads events, which would then talk to a different crossterm than envision's terminal.

```toml
crossterm = "0.29" # was: "0.28"
```

**Third-party ratatui widget crates** (e.g. `tui-*` crates, `ratatui-image`) must release a version that supports ratatui 0.30. `envision::ratatui` cannot change the ratatui requirement inside another crate; until it supports 0.30, its widgets hit the two-versions error above.

ratatui's own changes are listed in its [0.30 breaking-changes notes](https://github.com/ratatui/ratatui/blob/main/BREAKING-CHANGES.md#v0300).

**Policy:** from v0.18.0 on, every ratatui 0.x bump is a breaking envision release.

#### 2. Security: `lru`

This release removes RUSTSEC-2024-0436 (`paste`), RUSTSEC-2026-0002 and RUSTSEC-2026-0253 (`lru`) from envision's tree. ratatui-core 0.1.2 still admits `lru` 0.18.0 and 0.18.1, which RUSTSEC-2026-0253 covers. Make sure your lockfile has 0.18.2 or later:

```bash
cargo update -p lru
```

The advisory's code path (`LruCache::pop()` panicking during drop) is unreachable through ratatui, which only calls `get_or_insert`, `put` and `resize` on caches of plain data; see the CHANGELOG *Security* section for the evidence.

#### 3. Backend errors

`Backend` now has `type Error`. `CaptureBackend::Error` is `Infallible`, so `?` into an `io::Result` no longer compiles. Destructure instead:

```rust
// v0.17
fn size_of(b: &CaptureBackend) -> std::io::Result<u16> { Ok(b.size()?.width) }
// v0.18 — CaptureBackend cannot fail
fn size_of(b: &CaptureBackend) -> u16 { let Ok(size) = b.size(); size.width }
```

The same applies to `Terminal<CaptureBackend>`:

```rust
// v0.17
let mut terminal = Terminal::new(CaptureBackend::new(80, 24)).unwrap();
terminal.draw(|frame| { /* ... */ }).unwrap();
// v0.18
let Ok(mut terminal) = Terminal::new(CaptureBackend::new(80, 24));
let Ok(_) = terminal.draw(|frame| { /* ... */ });
```

`?` on these results still works inside a function returning `envision::Result`, because `EnvisionError: From<Infallible>`. `TestHarness::render` keeps its `envision::Result<()>` return type, but never returns `Err`.

**Custom backends** declare their error type:

```rust
// v0.17
impl Backend for MyBackend {
    fn size(&self) -> std::io::Result<Size> { /* ... */ }
    // ...
}
// v0.18
impl Backend for MyBackend {
    type Error = std::io::Error;
    fn size(&self) -> Result<Size, Self::Error> { /* ... */ }
    // ...
}
```

`Backend` is now re-exported as `envision::backend::Backend`.

**Runtime bound.** Six public methods, `Runtime::render`, `Runtime::tick`, `Runtime::run`, `Runtime::run_ticks`, `RuntimeBuilder::build` and `ConfiguredRuntimeBuilder::build`, require `EnvisionError: From<B::Error>`. `std::io::Error` (crossterm, termion, termwiz, termina) already satisfies it. Generic code that calls one of these methods must repeat the bound:

```rust
// v0.17
fn redraw<B: Backend>(runtime: &mut Runtime<MyApp, B>) -> envision::Result<()> {
    runtime.render()
}
// v0.18 — error[E0277] without the `where` clause
fn redraw<B: Backend>(runtime: &mut Runtime<MyApp, B>) -> envision::Result<()>
where
    EnvisionError: From<B::Error>,
{
    runtime.render()
}
```

If your backend's error type comes from another crate, the orphan rule (E0117) stops you implementing `From` for it. Wrap it in a local newtype and return that from your backend:

```rust
#[derive(Debug)]
struct MyError(std::fmt::Error); // a foreign error type, wrapped locally

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for MyError {}

impl From<MyError> for EnvisionError {
    fn from(e: MyError) -> Self {
        EnvisionError::other(e)
    }
}

impl Backend for MyBackend {
    type Error = MyError;
    // ...
}
```

If the backend itself is third-party, wrap it in a local newtype backend whose methods forward to it and map its errors into your error newtype.

**`DualBackend`.** `DualBackend<P>::Error` is `P::Error`. `DualBackend::with_auto_capture` and `DualBackendBuilder::build` now return `Result<_, P::Error>`:

```rust
// v0.17: envision::Result<DualBackend<P>>
// v0.18: Result<DualBackend<P>, P::Error>
let dual = DualBackend::with_auto_capture(CrosstermBackend::new(stdout()))?;
```

`?` keeps working where `P::Error` converts into your function's error type: unchanged for `io::Error` into `envision::Result` or `io::Result`. A custom primary needs the `From` impl above.

#### 4. `EnvisionError` is `#[non_exhaustive]`

```rust
// v0.17 — exhaustive; v0.18: error[E0004]
match err {
    EnvisionError::Io(_) => {}
    EnvisionError::Render { .. } => {}
    EnvisionError::Config { .. } => {}
    EnvisionError::Subscription { .. } => {}
    EnvisionError::Other(_) => {}
}
// v0.18
match err {
    EnvisionError::Io(_) => {}
    EnvisionError::Render { .. } => {}
    EnvisionError::Config { .. } => {}
    EnvisionError::Subscription { .. } => {}
    EnvisionError::Other(_) => {}
    _ => {}
}
```

#### 5. `EnhancedCell`

Fields are private; use the accessors. `skip` is replaced by `diff_option`.

| v0.17 | v0.18 |
|---|---|
| `cell.fg` | `cell.fg()` |
| `cell.bg` | `cell.bg()` |
| `cell.modifiers` | `cell.modifiers()` |
| `cell.underline_color` | `cell.underline_color()` |
| `cell.last_modified_frame` | `cell.last_modified_frame()` |
| `cell.skip` | `cell.diff_option() == SerializableDiffOption::Skip` |
| `cell.skip = true` | `cell.set_diff_option(SerializableDiffOption::Skip)` |
| `cell.fg = c` | `cell.set_style(Style::default().fg(c.into()))` |
| `cell.bg = c` | `cell.set_style(Style::default().bg(c.into()))` |
| `cell.underline_color = Some(c)` | `cell.set_style(Style::default().underline_color(c.into()))` |

`set_style` merges into the cell rather than replacing it: it changes only the colours the `Style` sets, adds `add_modifier` and removes `sub_modifier`, and never clears `underline_color`. So some v0.17 field writes have no one-line equivalent:

- `cell.modifiers = m`: clear, then add, in two calls: `cell.set_style(Style::default().remove_modifier(Modifier::all()));` then `cell.set_style(Style::default().add_modifier(m.into()));`.
- `cell.underline_color = None`: not possible on an existing cell. Build a fresh one instead (`EnhancedCell::new()` or `EnhancedCell::with_symbol(..)`, whose underline colour is `None`), then `set_style` the rest.
- `cell.last_modified_frame = n`: no longer possible. The frame stamp is written only by `CaptureBackend`; the field is now encapsulated, and its setter is crate-internal.

```rust
// v0.17
cell.fg == SerializableColor::Red && !cell.skip
// v0.18
cell.fg() == SerializableColor::Red && cell.diff_option() != SerializableDiffOption::Skip
```

`SerializableColor`, `SerializableModifier` and `SerializableDiffOption` are exported from `envision::backend` and the crate root; `SerializableDiffOption` is also in the prelude. `SerializableDiffOption` is `#[non_exhaustive]`, so a `match` on it needs a `_ =>` arm.

**JSON golden files.** v0.17 files, including `"skip": true` cells, still load without loss. New files omit `skip` and write `diff_option` only when it is not `none` (`"skip"`, `"always_update"`, `{"forced_width": n}`), so every v0.17 JSON golden changes once. To update them:

1. Run your tests. `SnapshotTest` writes `<name>.json.new` beside each golden that no longer matches.
2. Diff each `.new` file against its golden. If the only change is the removed `"skip":false` entries, move the `.new` file into place (or rerun that test with `with_update(true)`).
3. Investigate any other change; it is probably one of the rendering changes in section 6. Do not regenerate every golden at once, which would hide them.

#### 6. Rendering

- **Charts and canvases** shift by up to one braille sub-cell (ratatui now rounds grid coordinates), and lines with an out-of-bounds endpoint are now drawn clipped instead of dropped. Review your own chart snapshots with `cargo insta review`.
- **Wide glyphs.** `CaptureBackend` now clears the columns a wide glyph covers, as a terminal does. Single-frame captures are unchanged; multi-frame captures with CJK or emoji may differ from v0.17, where they could show stale characters.
- **Widths.** `EnhancedCell::symbol_width()` returns ratatui's width. It differs from v0.17 only where ratatui and `unicode-width` (now 0.2.2) disagree, such as halfwidth katakana with a voiced sound mark (`"ﾊﾟ"`: 2, was 1). envision's component text-layout width maths (e.g. `line_input`, `text_area`, `tab_bar`) still uses `unicode-width`, so it can disagree with ratatui on such sequences.

#### 7. Re-exports and ratatui features

New: `envision::ratatui` (including `envision::ratatui::crossterm`), `envision::backend::Backend`, `HorizontalAlignment` / `VerticalAlignment` in `envision::layout` and the prelude, and `SerializableDiffOption` / `SerializableColor` / `SerializableModifier` in the prelude.

envision no longer enables ratatui's `widget-calendar` and `macros` features.

- **First choice for all ratatui types:** `envision::ratatui`.
- **To use the trimmed features**, add a direct dependency compatible with envision's; Cargo unifies it, so the feature also appears under `envision::ratatui`:

  ```toml
  ratatui = { version = "0.30.2", features = ["widget-calendar"] } # or ["macros"]
  ```

  A requirement that resolves to a different ratatui brings back the two-versions error from section 1.
- If you already use `ratatui = { …, default-features = false }` and relied on envision enabling either feature, name it explicitly now.

**Supported ratatui features.** envision enables `crossterm_0_29`, `underline-color` and `layout-cache` (plus `serde` with `serialization`). Additive features such as `widget-calendar` and `macros` are compatible. `scrolling-regions` is not yet supported: enabling it anywhere in your dependency graph makes envision fail to compile (E0046 — `CaptureBackend`/`DualBackend` lack `scroll_region_up`/`scroll_region_down`), as was already the case in v0.17. `unstable-*` features are untested.

#### 8. Upstream changes visible through envision

Through envision's own re-exports:

- `Frame::render_widget_ref` was removed (it is behind ratatui's `unstable-widget-ref` feature); use `render_widget(&widget, area)`.
- `Terminal::new` and `Terminal::draw` return `B::Error` (section 3).
- `Alignment` is a type alias for `HorizontalAlignment`.
- ratatui's `Style` serialization omits `None` colours and empty modifiers, so serialized `KeyHintsState` and `TitleCardState` JSON is shorter. Old JSON still loads.

Only through `envision::ratatui`:

- `ListState` is `Copy`; `.clone()` on it now trips `clippy::clone_on_copy`.
- `Flex::SpaceAround` changed meaning, and `Flex::SpaceEvenly` was added. 0.29's `SpaceAround` layout is now spelled `SpaceEvenly`.

### Selection accessors completed on `selected_index()`

Every component whose selection accessor was spelled `selected` now spells it `selected_index`; every corresponding mutator is `set_selected_index`.

| Component | Old | New |
|---|---|---|
| `accordion` | `state.selected()` | `state.selected_index()` |
| `alert_panel` | `state.selected()` | `state.selected_index()` |
| `box_plot` | `state.selected()` | `state.selected_index()` |
| `box_plot` | `state.set_selected(i)` | `state.set_selected_index(i)` |
| `diagram` | `state.selected()` | `state.selected_index()` |
| `dropdown` | `state.selected()` | `state.selected_index()` |
| `dropdown` | `state.set_selected(i)` | `state.set_selected_index(i)` |
| `file_browser` | `state.selected()` | `state.selected_index()` |
| `loading_list` | `state.selected()` | `state.selected_index()` |
| `loading_list` | `state.set_selected(i)` | `state.set_selected_index(i)` |
| `menu` | `state.selected()` | `state.selected_index()` |
| `menu` | `state.set_selected(i)` | `state.set_selected_index(i)` |
| `metrics_dashboard` | `state.selected()` | `state.selected_index()` |
| `metrics_dashboard` | `state.set_selected(i)` | `state.set_selected_index(i)` |
| `multi_progress` | `state.selected()` | `state.selected_index()` |
| `multi_progress` | `state.set_selected(i)` | `state.set_selected_index(i)` |
| `radio_group` | `state.selected()` | `state.selected_index()` |
| `radio_group` | `state.set_selected(i)` | `state.set_selected_index(i)` |
| `searchable_list` | `state.selected()` | `state.selected_index()` |
| `searchable_list` | `state.set_selected(i)` | `state.set_selected_index(i)` |
| `select` | `state.selected()` | `state.selected_index()` |
| `select` | `state.set_selected(i)` | `state.set_selected_index(i)` |
| `selectable_list` | `state.selected()` | `state.selected_index()` |
| `selectable_list` | `state.set_selected(i)` | `state.set_selected_index(i)` |
| `tabs` | `state.selected()` | `state.selected_index()` |
| `tabs` | `state.set_selected(i)` | `state.set_selected_index(i)` |
| `tree` | `state.selected()` | `state.selected_index()` |
| `tree` | `state.set_selected(i)` | `state.set_selected_index(i)` |

Return types are unchanged in every case, including `box_plot`, whose accessor stays a bare `usize`.

**Finding your call sites.** Search for `.selected()` and `.set_selected(` on the sixteen components above.

`heatmap::selected()` returns `Option<(usize, usize)>` coordinates and is **unchanged** — do not migrate it.

Because these are renames, `cargo build` identifies every site and the compiler error names the receiver type, which disambiguates `heatmap` from the rest. Note that method references (`SelectState::selected`, or `.map(SelectState::selected)`) will not match a `.selected()` grep but will still fail to compile.

### Harness types available from the prelude

`envision::prelude::*` now re-exports `Assertion`, `MessageSendError`, `Snapshot`, and `TrySendError` alongside the three it already provided. The explicit `use envision::{Assertion, MessageSendError, Snapshot, TrySendError};` import is no longer required.

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
