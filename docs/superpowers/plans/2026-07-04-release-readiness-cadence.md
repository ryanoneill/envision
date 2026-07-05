# Release-readiness cadence (v0.17.0 pre-release) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close all 5 blocking findings from the 2026-07-04 Fable audit (`A-`, 3.62 GPA) — README compile-broken vs current App API, stacked `[Unreleased]` CHANGELOG blocks, stale MIGRATION.md, dual sort systems in `file_browser`, and `resource_gauge` accessor/constructor asymmetries — so v0.17.0 ships with the 11 weeks of unreleased API-quality work AND with restored release hygiene.

**Architecture:** Three independent code-shaped units bundled into one impl PR with three logically-separated signed commits:
- Unit 1 (release hygiene): `README.md` + `CHANGELOG.md` + `MIGRATION.md` + `src/lib.rs` (`include_str!` wire).
- Unit 2 (file_browser sort unification): delete `FileSortDirection` at `src/component/file_browser/types.rs:314-323`, canonicalize on `crate::component::table::SortDirection` at every use site (Option A — no local re-export). Getter signature changes shape (`Copy` allows by-value return).
- Unit 3 (resource_gauge closure): delete positional `new(f64, f64, f64)`, introduce `ResourceValues` named-struct + `with_values` single-call + fluent-builder methods + `values()` getter (restores 9/9 scorecard); add `examples/resource_gauge.rs`.

**Tech Stack:** Rust (edition 2024, MSRV 1.85), ratatui 0.29, tokio, cargo-nextest, insta snapshots, tempfile in dev-deps.

## Global Constraints

Every task's requirements implicitly include this section. Copy verbatim from the spec:

- Signed commits required. If `git commit -S` fails, STOP and ask the user — never bypass with `--no-gpg-sign`.
- Files must stay under 1000 lines.
- No clippy warnings allowed on either default-features (tracing OFF) or `--all-features`: `cargo clippy --all-features -- -D warnings` must be clean.
- No dead-code or unused-import warnings on any feature combination.
- `cargo doc --no-deps --all-features` must produce zero intra-doc-link warnings.
- `cargo build --no-default-features` must pass AND `cargo test --no-default-features --no-run` must pass (D8 lesson — the `--no-run` form catches example-gating drift the `--build` form misses).
- Audit scorecard must be 9/9 after Task 3 (`resource_gauge` accessor-symmetry gap closed).
- README code blocks must be self-sufficient from the visible text alone. Visible `use` imports required; only `# fn main() { }` may be hidden. `ignore` is banned unless justified inline.
- File_browser: canonical import path is `envision::component::table::SortDirection`. Option A — NO local re-export at the `file_browser` boundary.
- resource_gauge: `values()` returns `ResourceValues` struct, NOT `(f64, f64, f64)` tuple. The tuple recreates the exact positional-f64 hazard the constructor deletion fixes.
- Every new `pub fn` gets a `# Example` doc-test (audit-coverage discipline from G4+G5+G6 regressions).
- Feature-gate any impl block whose methods are only called under a feature (`impl ClipKind` lesson from D3).
- Always merge `origin/main` into the impl branch before pushing PR (Task 5).

---

## Pre-execution gotchas (read once before Task 1)

- **`AppHarness::new(width, height)` returns `Result`** — the README testing snippet needs `.unwrap()` (or `?` inside a `# fn main() -> Result<..>`). Signature at `src/harness/app_harness/mod.rs:92`.
- **`AppHarness::dispatch(msg)` takes ownership of the message** (`msg: A::Message` at `app_harness/mod.rs:251`). Not a reference. Message enum must be `Clone` if the same value is dispatched twice.
- **`AppHarness::render()` returns `Result<()>`** (`app_harness/mod.rs:383`). Needs `.unwrap()` or `?`.
- **`AppHarness::contains_text(needle: &str) -> bool`** (`app_harness/mod.rs:407`); **`AppHarness::assert_contains(needle: &str)`** (`app_harness/mod.rs:430`) — prefer `assert_contains` in test-style snippets, it panics with a diff on failure.
- **`FileSortDirection` has 24 references across 6 files** (spec's Unit 2 Migration Surface). Systematic grep-verifiable after migration: `grep -rn 'FileSortDirection' src/ examples/ tests/` returns zero hits.
- **`ResourceGaugeState::new` has 76 call sites** — mostly in `src/component/resource_gauge/tests.rs`. Grep-verifiable: `grep -rn 'ResourceGaugeState::new\b\|resource_gauge::new\b' src/ examples/ tests/` returns zero hits.
- **`ResourceValues` re-export at crate root goes in `src/lib.rs` prelude block** (approximately lines 458-459 — `pub use crate::component::*;` already re-exports it if it's public in `component/mod.rs`; verify at impl time). No separate `pub use` needed if the module re-export catches it.
- **`table::SortDirection` derives `Copy + Default`** (types.rs:439). This causes: (a) `sort_direction()` getter should return by-value not by-reference, (b) toggle branches collapse to `.toggle()`, (c) field initializer `sort_direction: SortDirection::Ascending` becomes redundant with `Default` (impl-time judgment).
- **CHANGELOG's three `[Unreleased]` topic labels are load-bearing.** Preserve them as `#### <topic>` sub-sub-sections under Keep-a-Changelog `### <kind>` subsections. See Task 1 Step 6 for the concrete before/after shape.
- **`examples/resource_gauge.rs` MUST have a `[[example]]` declaration in `Cargo.toml`** with `required-features = ["data-components"]`. Without this, `cargo test --no-default-features` tries to build the example and fails with undeclared-type errors (D8 lesson from the drilldown example).

## File structure

Files created:
- `src/component/resource_gauge/values.rs` — new sibling module housing the `ResourceValues` struct. Keeps `mod.rs` from creeping toward the 1000-line cap. (Task 3, Step 1.)
- `examples/resource_gauge.rs` — new ~150-line K8s pod-quota example. (Task 3, Step 10.)

Files modified:
- `README.md` — Feature Flags section added, TEA + testing examples fixed, 73→74 component count. (Task 1, Steps 1-4.)
- `src/lib.rs` — `#[cfg(doctest)] pub struct ReadmeDoctests;` gated re-import block. (Task 1, Step 5.) Also re-export of `ResourceValues` if module re-export doesn't catch it (Task 3, Step 3.)
- `CHANGELOG.md` — three `[Unreleased]` blocks consolidated into one with topic-preserved `####` subsections + "Known Deferred Findings" block. (Task 1, Step 6.)
- `MIGRATION.md` — `v0.15.x to v0.16.0` (small) + comprehensive `v0.16.x to v0.17.0` sections backfilled. Unit 2 + Unit 3 tables placeholder in Task 1, filled in Task 4. (Task 1, Step 7; Task 4, Step 1.)
- `src/component/file_browser/types.rs` — delete `FileSortDirection` enum + update `FileBrowserOutput::SortChanged` signature. (Task 2, Steps 1-2.)
- `src/component/file_browser/mod.rs` — state field type + initializer + `with_sort_direction` param + `sort_direction()` return type (by-value) + toggle collapse + sort comparator branches + 2 docstring examples. (Task 2, Steps 3-9.)
- `src/component/file_browser/tests.rs` — 8 test-site references update + one getter-deref change. (Task 2, Step 10.)
- `src/component/file_browser/helper_tests.rs` — no change (the `debug.contains("sort_direction")` string check at line 99 still passes). (Task 2, Step 11 verification only.)
- `src/component/mod.rs` — remove `FileSortDirection` from the re-export list at line 379. (Task 2, Step 2.)
- `src/component/resource_gauge/mod.rs` — delete `new(f64, f64, f64)` at line 139, add `with_values` + `with_actual` + `with_request` + `with_limit` + `values()` + module import of `ResourceValues`. (Task 3, Steps 2 and 4-8.)
- `src/component/resource_gauge/tests.rs` — 76 mechanical migration sites (implementer picks between `with_values(ResourceValues { .. })` and fluent-builder form per site). (Task 3, Step 9.)
- `Cargo.toml` — new `[[example]]` block for `drilldown` sibling: `name = "resource_gauge"`, `required-features = ["data-components"]`. (Task 3, Step 11.)

---

## Task 1: Unit 1 — Release hygiene

**Files:**
- Modify: `README.md` (3 code-block fixes + Feature Flags section insert + component-count fix)
- Modify: `src/lib.rs` (add `#[cfg(doctest)] pub struct ReadmeDoctests;` block)
- Modify: `CHANGELOG.md` (collapse three `[Unreleased]` blocks into one + add "Known Deferred Findings")
- Modify: `MIGRATION.md` (backfill `v0.15→v0.16` + start `v0.16→v0.17` with the already-committed breaking changes; Units 2+3 additions are Task 4)

**Interfaces:**
- Consumes: nothing from earlier tasks; this is Task 1.
- Produces: the `#[cfg(doctest)] pub struct ReadmeDoctests;` gate at `src/lib.rs`, and the consolidated `[Unreleased]` block shape at CHANGELOG line 8. Tasks 2 and 3 will slot their CHANGELOG entries into the sub-sub-sections established here; Task 4 finalizes the tables.

### Step 1: Fix README TEA example — `type Args` + `fn init(_args: ())`

- [ ] Open `README.md`. Locate the TEA counter example (currently around lines 55-91). The `impl App for MyApp` block currently reads:

```rust
impl App for MyApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        (State::default(), Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Increment => state.count += 1,
            Msg::Decrement => state.count -= 1,
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let text = format!("Count: {}", state.count);
        frame.render_widget(Paragraph::new(text), frame.area());
    }
}
```

- [ ] Replace with the current-API-compatible form:

```rust
impl App for MyApp {
    type State = State;
    type Message = Msg;
    type Args = ();

    fn init(_args: ()) -> (State, Command<Msg>) {
        (State::default(), Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Increment => state.count += 1,
            Msg::Decrement => state.count -= 1,
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let text = format!("Count: {}", state.count);
        frame.render_widget(Paragraph::new(text), frame.area());
    }
}
```

Only two changes: `type Args = ();` line added, `fn init()` becomes `fn init(_args: ())`.

- [ ] Verify the code block above the `impl` block is also self-sufficient — needs visible imports for `envision::prelude::*` and `ratatui::widgets::Paragraph`. If the current README already has these visible at the top of the block, no change. If they're absent, add them as the first lines of the block.

### Step 2: Fix README "Testing with Runtime" example — use `AppHarness` idiom

- [ ] Locate the "Testing with Runtime" section (currently around lines 100-115). The current block reads:

```rust
use envision::prelude::*;
use ratatui::widgets::Paragraph;

// Given the MyApp defined above:
#[test]
fn test_my_app() {
    let mut runtime = Runtime::<MyApp>::virtual_terminal(80, 24).unwrap();

    runtime.dispatch(Msg::Increment);
    runtime.dispatch(Msg::Increment);
    runtime.render().unwrap();

    assert!(runtime.contains_text("Count: 2"));
}
```

All four `runtime.*` method calls are wrong — those methods live on `AppHarness`, not on `Runtime`. Rename the section heading to "Testing with AppHarness" and replace the code:

```rust
use envision::prelude::*;

#[test]
fn test_my_app() {
    let mut harness = AppHarness::<MyApp>::new(80, 24).unwrap();

    harness.dispatch(Msg::Increment);
    harness.dispatch(Msg::Increment);
    harness.render().unwrap();

    harness.assert_contains("Count: 2");
}
```

Key changes:
- `Runtime::<MyApp>::virtual_terminal(80, 24)` → `AppHarness::<MyApp>::new(80, 24)` (the correct constructor; signature at `src/harness/app_harness/mod.rs:92`).
- `runtime.*` → `harness.*` throughout.
- `assert!(runtime.contains_text(..))` → `harness.assert_contains(..)` (the direct assertion form panics with a diff on failure; better UX than a bare `assert!` — signature at `app_harness/mod.rs:430`).
- The `use ratatui::widgets::Paragraph;` import is unused in the test snippet — drop it.
- `AppHarness` is already re-exported in `envision::prelude::*` (verified at `src/lib.rs:463`), so no separate `use envision::harness::AppHarness;` needed.

### Step 3: Fix "Test Harness for Custom Rendering" block if it exists

- [ ] Locate the "Test Harness for Custom Rendering" section (currently starts around line 117). Verify the `TestHarness` example still compiles against current API. Read the block and check:

```rust
use envision::harness::TestHarness;
use ratatui::widgets::Paragraph;

let mut harness = TestHarness::new(80, 24);
// ... more code
```

- [ ] If the block references `envision::harness::TestHarness` but `TestHarness` is now also in the prelude (verified at `src/lib.rs:463`), leave the block as-is if the imports match, or migrate the `envision::harness::TestHarness` line to `use envision::prelude::*;` to match the AppHarness snippet style. Judgment call — prefer consistency with the AppHarness snippet.
- [ ] Verify the block's method calls (whatever they are) exist on `src/harness/test_harness/mod.rs` — if any are stale, fix them the same way as Step 2.

### Step 4: Add README Feature Flags section

- [ ] Locate the "Installation" section (near the top of the README, right before "Quick Start"). Immediately after the Installation section's closing paragraph, insert:

```markdown
## Feature Flags

Envision is feature-gated so consumers can opt out of the parts they don't need. Default features enable serialization and all component groups.

| Flag | On by default | What it turns on |
|---|---|---|
| `full` | yes | All component groups + clipboard + markdown + regex (convenience alias) |
| `input-components` | yes (via `full`) | Interactive input widgets (`LineInput`, `TextArea`, `Dropdown`, `Select`, ...) |
| `data-components` | yes (via `full`) | Data-display widgets (`Table`, `DataGrid`, `Tree`, `ResourceGauge`, ...) |
| `display-components` | yes (via `full`) | Text and chart widgets (`StyledText`, `Chart`, `Sparkline`, ...) |
| `navigation-components` | yes (via `full`) | `PaneLayout`, `Router`, `TabBar`, `KeyHints` |
| `overlay-components` | yes (via `full`) | Overlay stack primitives |
| `compound-components` | yes (via `full`) | Higher-level compositions (`Diagram`, `LogViewer`, `ConversationView`) |
| `serialization` | yes | `serde::Serialize`/`Deserialize` on component state |
| `tracing` | no | Emits `tracing::warn!` diagnostics (e.g., table column clip warnings) |
| `clipboard` | yes (via `full`) | `arboard`-backed clipboard on `TextArea` |
| `markdown` | yes (via `full`) | Markdown rendering in `StyledText` |
| `regex` | yes (via `full`) | Regex search in `EventStream`, `LogViewer` |
| `test-utils` | no | `AppHarness` async test utilities (`advance_time`, `wait_for`) at the crate boundary for downstream tests |

To opt out of everything and only pull in specific groups:

```toml
[dependencies]
envision = { version = "0.17", default-features = false, features = ["data-components", "display-components"] }
```
```

- [ ] Verify each flag row corresponds to a real entry in `Cargo.toml` `[features]` block (lines 15-47). If Cargo.toml has been touched since the audit and the flag list differs, adjust the table to match; the source of truth is `Cargo.toml`.

### Step 5: Fix component count 73 → 74 with sync comment

- [ ] Grep `README.md` for occurrences of `73`:

```bash
grep -n "73" README.md
```

Expected: three hits (lines 13, 157, 323 approximately — verify against current README):
- Line 13 (or nearest): "Component Library - 73 ready-to-use UI components following TEA pattern"
- Line 157 (or nearest): "Envision provides a comprehensive library of 73 reusable UI components"
- Line 323 (or nearest): "`component` | 73 reusable UI components with `Component`, `Toggleable` traits"

- [ ] For each occurrence, replace `73` with `74` and add an HTML sync comment on the line above:

```markdown
<!-- component-count: keep in sync with docs/CHOOSING.md -->
- **Component Library** - 74 ready-to-use UI components following TEA pattern
```

The HTML comment renders as blank in markdown but is grep-catchable during future audits.

- [ ] Verify by running `grep -n "73" README.md` — should return zero hits (or, if any legitimate "73" appears elsewhere like a version number, exempt it). And `grep -n "74" README.md` should return at least three.

### Step 6: Add `#[cfg(doctest)] pub struct ReadmeDoctests;` to `src/lib.rs`

- [ ] Open `src/lib.rs`. Locate the `pub mod prelude {` block (approximately line 430).
- [ ] Immediately above the `pub mod prelude {` line, insert:

```rust
/// Compiles the code blocks in `README.md` as doctests so they can't rot silently.
///
/// The `#[cfg(doctest)]` gate keeps this out of both the normal build and the public
/// API surface — the struct exists only when `cargo test --doc` is running.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
pub struct ReadmeDoctests;
```

- [ ] Save. Run:

```bash
cargo test --doc --all-features 2>&1 | tail -20
```

Expected: doc-tests pass. If any README code block fails to compile, read the failure output, fix the block in `README.md` (add visible `use` imports as needed — the visible-imports rule applies here), and re-run until clean.

- [ ] Also verify:

```bash
cargo doc --no-deps --all-features 2>&1 | grep -iE "warning|error" | head -10
```

Expected: no new warnings introduced by the `include_str!` block (the `#[cfg(doctest)]` gate keeps the struct out of rustdoc's public API surface).

### Step 7: Consolidate CHANGELOG's three `[Unreleased]` blocks

- [ ] Open `CHANGELOG.md`. Verify the three existing `## [Unreleased]` headers at approximately lines 8, 362, 389.

- [ ] Prepare the new consolidated block using the following exact shape. Do NOT lose content — every bullet from the old blocks must land under a `####` sub-sub-section:

```markdown
## [Unreleased]

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
- `Column::with_comparator` / `Column::comparator` / `SortComparator` / `SortDirection::Toggle`.

Added:
- `Column::with_default_sort(SortDirection)` — declarative default column sort.
- `TableState::with_initial_sort(InitialSort)` — construction-time sort state.
- `TableMessage::SortAsc { column }` / `SortDesc { column }` / `RemoveSort { column }` / `Clear` — explicit primitives.
- Reified `Cell { text, style, sort_key }` type replacing the string-and-comparator model.
- `TableRow::cells(&self) -> Vec<Cell>` with `SortKey` variant enum for typed sort keys.

Tracks leadline gaps G1 + G3 + G7. See `docs/superpowers/specs/2026-05-01-table-sort-cell-unification-design.md`. Migration table in `MIGRATION.md#v016x-to-v0170`.

#### `FileSortDirection` removed

`file_browser::FileSortDirection` deleted. `file_browser` uses `table::SortDirection` (same 2-variant Ascending/Descending shape). `sort_direction()` getter signature changes to return by value (SortDirection is Copy).

See `MIGRATION.md#v016x-to-v0170` for the full before/after table.

#### `ResourceGaugeState::new` replaced by named-struct + builder

`ResourceGaugeState::new(actual, request, limit)` (three unlabeled positional f64 args) deleted. Replaced by `ResourceGaugeState::default().with_values(ResourceValues { actual, request, limit })` single-call form or fluent builder chain (`with_actual`, `with_request`, `with_limit`).

New public type: `envision::component::ResourceValues { actual, request, limit }`.
New accessor: `state.values() -> ResourceValues` (closes accessor-symmetry gap with `set_values`).

See `MIGRATION.md#v016x-to-v0170`.

### Added

#### Chrome ownership protocol (G2 + D2 + D11)

- `PaneLayout::view_with(state, ctx, |pane_id, child_ctx| ...)` — closure-based renderer; envision owns inner-rect computation.
- `RenderContext::chrome_owned` flag — Table, StyledText, and other chrome-drawing components consult it and skip their outer Block when embedded.
- 35 chrome-drawing components audited to consult the flag.

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

75 raw `pub const` color constants marked `#[deprecated(since = "0.17.0")]` — accessible during transition window.

#### `CellStyle::Severity(Severity)` (D15)

- Severity-aware cells reach the active theme at render time via `theme.severity_style(*sev)` in `cell_style_to_ratatui`.
- `Cell::severity(text, sev)` constructor + `Cell::with_severity(sev)` builder.
- `CellStyle` gains `#[non_exhaustive]`.

#### StyledText DX: `styled_line` primitive + `paragraph` → `line` rename (D5 + D14)

- `envision::render::styled_line(frame, area, &[StyledInline], theme)` free function (`src/render.rs`).
- Re-exported at `envision::styled_line`. Module + re-export gated on `display-components`.
- `StyledContent::paragraph(...)` deleted; `StyledContent::line(...)` replaces it. Also `StyledBlock::Paragraph` → `StyledBlock::Line`.

#### Per-component style overrides (G4 + G5)

- `PaneConfig::with_title_style(Style)` + `title_style() -> Option<Style>` — pane title styling independent of border.
- `StatusBarItem::with_color(Color)` + `with_style_override(Style)` — layered semantics, not last-call-wins.

Sibling file split: `pane_layout/title_style.rs`. Restores four-stop severity ramp for StatusBar consumers.

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

### Changed

#### Chrome ownership protocol

- `Table`, `LogViewer`, `ScrollView`, `ScrollableText`, `MarkdownRenderer`, `ConversationView`, `DataGrid`, `MetricsDashboard` (per-cell only), and 27 others skip their outer Block when `RenderContext::chrome_owned == true`.
- Consumers embedding any of them get correct behavior without further envision changes.

#### `examples/router.rs` refresh (D8)

- Screen-render bodies now use `PaneLayout::view_with` chrome instead of raw `ratatui::widgets::Paragraph + Block::borders`. No behavior change; better envision-component showcase.

### Known Deferred Findings

The 2026-07-04 audit (Fable) surfaced two API incoherences deliberately deferred beyond v0.17.0. Both are tracked as follow-up cadences and will be addressed in v0.18.0 or later:

- **`selected_value` / `selected_item` / `active_tab` accessor shape divergence** across `dropdown`, `select`, `heatmap`, `tab_bar`, and `data_grid`. `dropdown::selected_value()` and `dropdown::selected_item()` are literal `&str` aliases; `heatmap::selected_value()` returns `f64` (type-incoherent with the string variant); `tab_bar` uses `active_tab()` instead of `selected_item()`; `data_grid` has four selection accessors (`selected`, `selected_index`, `selected_row`, `selected_item`). Requires a dedicated consistency-sweep cadence.
- **Dependency leakage in 8 public signatures** (`ratatui::layout::Position`, `ratatui::buffer::Cell`, `ratatui::style::Color`, `ratatui::style::Style`, `ratatui::widgets::Widget`, `tokio::sync::mpsc::Sender` at `harness/app_harness/mod.rs:264`, plus 2 others). Architectural discussion; not release-blocking.
```

- [ ] Open `CHANGELOG.md`. Delete the three existing `## [Unreleased]` blocks (from line 8 through the last one, up to but not including `## [0.16.0]`). Insert the consolidated block above at line 8.
- [ ] Verify:

```bash
grep -n "^## \[Unreleased\]" CHANGELOG.md
```

Expected: exactly one hit at line 8.

### Step 8: Backfill MIGRATION.md — v0.15→v0.16 + start v0.16→v0.17

- [ ] Open `MIGRATION.md`. The file currently starts:

```markdown
# Migration Guide

## v0.14.x to v0.15.0
```

- [ ] Insert two new sections between the header and the `## v0.14.x to v0.15.0` line:

```markdown
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

`TableMessage::SortBy` / `AddSort` / `ClearSort` are removed and replaced by explicit primitives. `Column::with_comparator` / `comparator` / `SortComparator` are removed and replaced by the reified `Cell { text, style, sort_key }` type + `SortKey` enum.

| Old | New |
|---|---|
| `TableMessage::SortBy(column)` — cycles asc/desc/none | `TableMessage::SortAsc { column }` / `SortDesc { column }` / `RemoveSort { column }` (explicit) |
| `TableMessage::AddSort(column)` | `TableMessage::SortAsc { column }` after `TableMessage::Clear` |
| `TableMessage::ClearSort` | `TableMessage::Clear` |
| `impl TableRow for Row { fn cells(&self) -> Vec<String> { ... } fn comparator(col: usize) -> ... }` | `impl TableRow for Row { fn cells(&self) -> Vec<Cell> { vec![Cell::new(...), ...] } }` (sort_key encoded in Cell) |
| `Column::with_comparator(...)` | `Column::with_default_sort(SortDirection::Ascending)` (declarative) |
| `TableState::sort_by(col, dir)` | `TableState::with_initial_sort(InitialSort::new(col, dir))` (construction-time) |

Placeholder for FileSortDirection migration (filled by Task 4 after Unit 2 lands).

Placeholder for ResourceGaugeState migration (filled by Task 4 after Unit 3 lands).

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
```

Note the two `Placeholder for ...` lines under `v0.16.x to v0.17.0` — those get filled by Task 4 after Units 2 and 3 land.

- [ ] Save. Verify:

```bash
grep -n "^## v0\.\(14\|15\|16\)" MIGRATION.md
```

Expected: three hits — `## v0.16.x to v0.17.0`, `## v0.15.x to v0.16.0`, `## v0.14.x to v0.15.0`, in that order (newest first).

### Step 9: Full verification for Task 1

- [ ] Run:

```bash
cargo fmt --check
```

Expected: no output (clean formatting).

- [ ] Run:

```bash
cargo clippy --all-features -- -D warnings 2>&1 | tail -10
```

Expected: no warnings.

- [ ] Run:

```bash
cargo test --all-features --doc 2>&1 | tail -20
```

Expected: all doc tests pass, including the new README code blocks compiled via `include_str!`. Count should increase by the number of ` ```rust` blocks in the README.

- [ ] Run:

```bash
cargo doc --no-deps --all-features 2>&1 | grep -iE "warning|error" | head -10
```

Expected: no new warnings from the `include_str!` block or the CHANGELOG/MIGRATION.md changes (those aren't Rust code — no impact on cargo doc).

- [ ] Run:

```bash
cargo build --no-default-features 2>&1 | tail -5
```

Expected: clean build.

### Step 10: Commit Task 1

- [ ] Stage:

```bash
git add README.md src/lib.rs CHANGELOG.md MIGRATION.md
```

- [ ] Commit with a signed HEREDOC message:

```bash
git commit -S -m "$(cat <<'EOF'
release-hygiene: README examples fixed + include_str! wire + Feature Flags + CHANGELOG collapse + MIGRATION.md backfill

Address 3 of 5 Fable audit findings (2026-07-04) that block v0.17.0
release readiness:

README (finding #1):
- TEA counter example: type Args = (); + fn init(_args: ())
- Testing example: replaced bogus Runtime::virtual_terminal +
  runtime.dispatch/render/contains_text calls (which don't exist) with
  the correct AppHarness idiom: AppHarness::new(w, h).unwrap() +
  harness.dispatch(msg) + harness.render().unwrap() +
  harness.assert_contains(...)
- New "Feature Flags" section between Installation and Quick Start
  listing all 14 flags with default-features=false example
- 73 -> 74 component count fix (3 sites) with HTML sync comment for
  drift-catching

include_str! doctest wiring (finding #1 mechanism):
- New #[cfg(doctest)] pub struct ReadmeDoctests; block in src/lib.rs
- README code blocks now compiled by cargo test --doc
- Copy-paste fidelity discipline: visible imports required; only
  # fn main() { } may be hidden. No # use envision::prelude::*;
  invisibly injecting scope

CHANGELOG (finding #2):
- Three stacked [Unreleased] blocks collapsed into one
- Topic labels preserved as #### sub-sub-sections under
  Keep-a-Changelog ### kinds (Breaking Changes, Added, Changed)
- New "Known Deferred Findings" block making audit findings #6
  (selected_value incoherence) and #8 (dep leakage) visible to
  consumers reading release notes rather than only in this spec

MIGRATION.md (finding #3):
- New v0.16.x -> v0.17.0 section covering App::init args + Table
  sort/cell redesign (migration tables lifted from CHANGELOG)
- New v0.15.x -> v0.16.0 section covering DependencyGraph -> Diagram
- Placeholder rows for FileSortDirection and ResourceGaugeState::new
  migrations, filled by Task 4 after Units 2 and 3 land

Verification: cargo fmt clean; cargo clippy --all-features -D warnings
clean; cargo test --all-features --doc adds N passing tests for the
README blocks; cargo doc --no-deps clean; cargo build
--no-default-features clean.
EOF
)"
```

- [ ] Verify signature:

```bash
git log --show-signature -1 HEAD 2>&1 | head -5
```

Expected: `Good signature from "Ryan O'Neill ..."`.

---

## Task 2: Unit 2 — file_browser sort unification

**Files:**
- Modify: `src/component/file_browser/types.rs` (delete `FileSortDirection` enum; update `FileBrowserOutput::SortChanged` signature)
- Modify: `src/component/file_browser/mod.rs` (state field + initializer + `with_sort_direction` param + `sort_direction()` return type + toggle + sort comparator + 2 docstring examples)
- Modify: `src/component/file_browser/tests.rs` (8 test-site references + one getter-deref)
- Modify: `src/component/mod.rs` (remove `FileSortDirection` from re-export at line 379)

**Interfaces:**
- Consumes: Task 1's consolidated `[Unreleased]` block layout at CHANGELOG line 8 (Unit 2's entry is already under the `#### FileSortDirection removed` sub-sub-section).
- Produces: `sort_direction()` returns `SortDirection` by value (was `&FileSortDirection`); `FileBrowserOutput::SortChanged(FileSortField, SortDirection)` (was `SortChanged(FileSortField, FileSortDirection)`); `envision::component::file_browser::FileSortDirection` no longer exists. Task 4 will fill the MIGRATION.md placeholder for Unit 2.

### Step 1: Add `use crate::component::table::SortDirection;` in `types.rs`

- [ ] Open `src/component/file_browser/types.rs`. Locate the top-of-file imports. Add:

```rust
use crate::component::table::SortDirection;
```

- [ ] Save. The compiler will now know the type; Step 2 renames the existing `FileSortDirection` uses.

### Step 2: Delete `FileSortDirection` enum + update `FileBrowserOutput::SortChanged`

- [ ] In `src/component/file_browser/types.rs`, locate line 60:

```rust
    SortChanged(FileSortField, FileSortDirection),
```

Replace with:

```rust
    SortChanged(FileSortField, SortDirection),
```

- [ ] Locate lines 314-323 (the `FileSortDirection` enum definition):

```rust
/// Sort direction.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum FileSortDirection {
    /// Ascending order.
    Ascending,
    /// Descending order.
    Descending,
}
```

Delete the entire block (all 12 lines including the docstring). Verify no dangling blank lines are left behind (adjust spacing to match surrounding code).

- [ ] Save.

- [ ] Run:

```bash
cargo check --all-features 2>&1 | tail -20
```

Expected: errors like `cannot find type FileSortDirection in this scope` in `mod.rs` and `tests.rs`. That's the compiler walking us through the remaining sites — those are Steps 3-11.

### Step 3: Update `mod.rs` imports + state field type

- [ ] Open `src/component/file_browser/mod.rs`. Locate the top-of-file imports. Verify or add:

```rust
use crate::component::file_browser::types::*;  // if this line exists, FileSortDirection was pulled via glob — check
use crate::component::table::SortDirection;
```

If the file already uses `use crate::component::file_browser::types::*;` glob import, the removal of `FileSortDirection` from `types.rs` means it's gone from the glob — a good thing. Add the `use crate::component::table::SortDirection;` line explicitly so the type is in scope.

- [ ] Locate line 75:

```rust
    sort_direction: FileSortDirection,
```

Replace with:

```rust
    sort_direction: SortDirection,
```

- [ ] Locate line 96 (initializer, inside a `Default` impl or `new`-style constructor):

```rust
            sort_direction: FileSortDirection::Ascending,
```

Judgment: keep as-is (the struct's construction is explicit field-by-field, per file_browser convention). Just retype the enum name:

```rust
            sort_direction: SortDirection::Ascending,
```

- [ ] Save.

### Step 4: Update `with_sort_direction` builder method signature

- [ ] Locate `with_sort_direction` (around line 269):

```rust
    pub fn with_sort_direction(mut self, direction: FileSortDirection) -> Self {
```

Replace with:

```rust
    pub fn with_sort_direction(mut self, direction: SortDirection) -> Self {
```

- [ ] Locate the docstring example just above (around lines 262-267):

```rust
    /// use envision::component::file_browser::{FileEntry, FileBrowserState, FileSortDirection};
    ///
    /// let state = FileBrowserState::new("/", vec![
    ///     FileEntry::new("a.txt", false),
    /// ]).with_sort_direction(FileSortDirection::Descending);
    /// assert_eq!(state.sort_direction(), &FileSortDirection::Descending);
```

Replace with:

```rust
    /// use envision::component::file_browser::{FileEntry, FileBrowserState};
    /// use envision::component::table::SortDirection;
    ///
    /// let state = FileBrowserState::new("/", vec![
    ///     FileEntry::new("a.txt", false),
    /// ]).with_sort_direction(SortDirection::Descending);
    /// assert_eq!(state.sort_direction(), SortDirection::Descending);
```

Two changes: (a) removed `FileSortDirection` from the `use` line + added a separate `use envision::component::table::SortDirection;` line (Option A canonical path). (b) The assertion changed from `&FileSortDirection::Descending` to `SortDirection::Descending` — because Step 5 makes `sort_direction()` return by value.

### Step 5: Change `sort_direction()` getter to return by value

- [ ] Locate `sort_direction` getter (around line 533):

```rust
    pub fn sort_direction(&self) -> &FileSortDirection {
        &self.sort_direction
    }
```

Replace with:

```rust
    pub fn sort_direction(&self) -> SortDirection {
        self.sort_direction
    }
```

(Two changes: return type `&FileSortDirection` → `SortDirection`; body `&self.sort_direction` → `self.sort_direction` — auto-copy since `SortDirection: Copy`.)

- [ ] Locate the docstring example just above (around lines 527-530):

```rust
    /// use envision::component::file_browser::{FileEntry, FileBrowserState, FileSortDirection};
    ///
    /// let state = FileBrowserState::new("/", vec![
    ///     FileEntry::new("a.txt", false),
    /// ]).with_sort_direction(FileSortDirection::Descending);
    /// assert_eq!(state.sort_direction(), &FileSortDirection::Descending);
```

Replace with:

```rust
    /// use envision::component::file_browser::{FileEntry, FileBrowserState};
    /// use envision::component::table::SortDirection;
    ///
    /// let state = FileBrowserState::new("/", vec![
    ///     FileEntry::new("a.txt", false),
    /// ]).with_sort_direction(SortDirection::Descending);
    /// assert_eq!(state.sort_direction(), SortDirection::Descending);
```

### Step 6: Update sort comparator branches at lines 646-647

- [ ] Locate lines 646-647:

```rust
                FileSortDirection::Ascending => ord,
                FileSortDirection::Descending => ord.reverse(),
```

Replace with:

```rust
                SortDirection::Ascending => ord,
                SortDirection::Descending => ord.reverse(),
```

(Mechanical rename — semantics identical.)

### Step 7: Collapse toggle branches at lines 927-928 to `.toggle()`

- [ ] Locate lines 927-928. Current shape is likely:

```rust
                let new_direction = match state.sort_direction {
                    FileSortDirection::Ascending => FileSortDirection::Descending,
                    FileSortDirection::Descending => FileSortDirection::Ascending,
                };
```

Replace with:

```rust
                let new_direction = state.sort_direction.toggle();
```

(The `toggle()` method already exists on `SortDirection` at `src/component/table/types.rs:458`. This removes 4 lines of dead-simple logic.)

- [ ] Save.

### Step 8: Verify `mod.rs` compiles

- [ ] Run:

```bash
cargo check --all-features 2>&1 | tail -20
```

Expected: errors now only in `tests.rs` (and possibly `src/component/mod.rs:379`). If `mod.rs` still emits errors, walk through Steps 3-7 to find the missed site.

### Step 9: Remove `FileSortDirection` from `src/component/mod.rs:379`

- [ ] Open `src/component/mod.rs`. Locate line 379:

```rust
    FileSortDirection, FileSortField, SelectionMode,
```

Replace with:

```rust
    FileSortField, SelectionMode,
```

(Removes `FileSortDirection` from the re-export list — the type no longer exists.)

- [ ] Save.

### Step 10: Migrate `file_browser/tests.rs` — 8 references + getter deref

- [ ] Open `src/component/file_browser/tests.rs`. Locate the top-of-file imports. Add:

```rust
use crate::component::table::SortDirection;
```

If `tests.rs` uses a glob `use super::*;` or `use crate::component::file_browser::*;`, the glob no longer pulls `FileSortDirection` (deleted) but does still pull `FileSortField`. So the explicit `SortDirection` import is required.

- [ ] Systematic search-replace within `tests.rs`. For each of the following patterns, apply the fix:

**Pattern 1** — Type name in assertions and construction (lines 72, 95, 96, 196, 530, 543 approximately):

```rust
FileSortDirection::Ascending  →  SortDirection::Ascending
FileSortDirection::Descending →  SortDirection::Descending
```

**Pattern 2** — Getter-deref shape change (line 72 and 96 approximately). Old:

```rust
    assert_eq!(*state.sort_direction(), FileSortDirection::Ascending);
```

New (drop the `*` deref — `sort_direction()` now returns by value):

```rust
    assert_eq!(state.sort_direction(), SortDirection::Ascending);
```

Same fix at line 96:

```rust
    assert_eq!(*state.sort_direction(), FileSortDirection::Descending);
```

→

```rust
    assert_eq!(state.sort_direction(), SortDirection::Descending);
```

**Pattern 3** — Test function names at lines 93 and 536 (`test_with_sort_direction` and `test_toggle_sort_direction`) stay as-is — they're internal names, not tied to the enum.

**Pattern 4** — `FileBrowserOutput::SortChanged(FileSortField::Size, FileSortDirection::Ascending)` at line 530 area:

```rust
        Some(FileBrowserOutput::SortChanged(
            FileSortField::Size,
            FileSortDirection::Ascending
        ))
```

→

```rust
        Some(FileBrowserOutput::SortChanged(
            FileSortField::Size,
            SortDirection::Ascending
        ))
```

Same at line 543:

```rust
        Some(FileBrowserOutput::SortChanged(
            FileSortField::Name,
            FileSortDirection::Descending
        ))
```

→

```rust
        Some(FileBrowserOutput::SortChanged(
            FileSortField::Name,
            SortDirection::Descending
        ))
```

- [ ] Save.

- [ ] Grep to verify no remaining `FileSortDirection` references in `tests.rs`:

```bash
grep -n "FileSortDirection" src/component/file_browser/tests.rs
```

Expected: no output.

### Step 11: Verify `helper_tests.rs` still passes (no change)

- [ ] Open `src/component/file_browser/helper_tests.rs` and check line 99:

```rust
    assert!(debug.contains("sort_direction"));
```

Expected: unchanged. The debug-format check is for the field name `sort_direction`, not the type name — still valid.

- [ ] Do NOT modify this file.

### Step 12: Grep verify no remaining `FileSortDirection` across the tree

- [ ] Run:

```bash
grep -rn 'FileSortDirection' src/ examples/ tests/
```

Expected: zero hits. If any remain, walk them through Steps 3-10.

### Step 13: Full verification for Task 2

- [ ] Run:

```bash
cargo check --all-features 2>&1 | tail -5
```

Expected: clean.

- [ ] Run:

```bash
cargo nextest run --all-features -E 'test(file_browser)' 2>&1 | tail -15
```

Expected: all file_browser tests pass.

- [ ] Run:

```bash
cargo test --all-features --doc file_browser 2>&1 | tail -10
```

Expected: doc tests (including the two docstring examples we updated in Steps 4 and 5) pass.

- [ ] Run:

```bash
cargo clippy --all-features -- -D warnings 2>&1 | tail -5
```

Expected: clean.

- [ ] Run:

```bash
cargo build --no-default-features 2>&1 | tail -3
```

Expected: clean.

### Step 14: Commit Task 2

- [ ] Stage:

```bash
git add src/component/file_browser/types.rs src/component/file_browser/mod.rs src/component/file_browser/tests.rs src/component/mod.rs
```

- [ ] Commit:

```bash
git commit -S -m "$(cat <<'EOF'
file-browser-sort: delete FileSortDirection; canonicalize on table::SortDirection

Address Fable audit finding #4 (dual sort systems). file_browser's
FileSortDirection was a 2-variant Ascending/Descending enum identical
in shape to table::SortDirection — same variants, same PartialEq/Eq
derives, same semantic role. Two names for one concept is exactly what
the G1/G3/G7 sort/cell unification cadence was supposed to eliminate;
file_browser was the one holdout.

Deletion + full migration to canonical table::SortDirection at every
use site (Option A per spec — NO local re-export at the file_browser
boundary, which would perpetuate the "same type under two paths"
pattern the audit finding calls out).

Getter signature also changes: sort_direction() returns SortDirection
by value (was &FileSortDirection). SortDirection: Copy makes by-value
return the idiomatic form; the reference was unnecessary. Third
breaking change on file_browser, documented in MIGRATION.md.

Toggle branches at mod.rs:927-928 collapse to
state.sort_direction.toggle() (the toggle() method already exists on
table::SortDirection at types.rs:458). Four lines of hand-rolled
match dropped.

24 references across 6 files migrated:
- src/component/file_browser/types.rs — enum deleted, SortChanged
  variant type updated
- src/component/file_browser/mod.rs — state field + initializer +
  with_sort_direction param + sort_direction() return type +
  toggle collapse + sort comparator branches + 2 docstring examples
- src/component/file_browser/tests.rs — 8 test-site references +
  getter deref shape change
- src/component/mod.rs — FileSortDirection removed from re-export list
- src/component/file_browser/helper_tests.rs — unchanged (debug-string
  check is for the field name, not the type name)

Grep verified: `grep -rn 'FileSortDirection' src/ examples/ tests/`
returns zero hits post-migration.

Verification: cargo check clean; cargo nextest run file_browser passes;
cargo test --doc file_browser passes; cargo clippy --all-features
-D warnings clean; cargo build --no-default-features clean.

Sets up MIGRATION.md fill-in at Task 4.
EOF
)"
```

- [ ] Verify signature.

---

## Task 3: Unit 3 — resource_gauge closure

**Files:**
- Create: `src/component/resource_gauge/values.rs` (new sibling module for the `ResourceValues` struct)
- Modify: `src/component/resource_gauge/mod.rs` (declare submodule; delete positional `new`; add builder methods; add `values()` accessor)
- Modify: `src/component/resource_gauge/tests.rs` (76 mechanical migration sites)
- Modify: `src/lib.rs` (verify `ResourceValues` re-export catches through `pub use crate::component::*;`)
- Modify: `Cargo.toml` (add `[[example]]` block for `resource_gauge`)
- Create: `examples/resource_gauge.rs` (~150 lines, K8s pod-quota shape)

**Interfaces:**
- Consumes: nothing from Task 2 (independent unit).
- Produces: new `envision::component::ResourceValues` struct; new `ResourceGaugeState::with_values(ResourceValues) -> Self`, `with_actual(f64) -> Self`, `with_request(f64) -> Self`, `with_limit(f64) -> Self`, `values(&self) -> ResourceValues`; deletion of `ResourceGaugeState::new(f64, f64, f64)`. Task 4 will fill the MIGRATION.md placeholder for Unit 3.

### Step 1: Create sibling `values.rs` housing the `ResourceValues` struct

- [ ] Create `src/component/resource_gauge/values.rs` with:

```rust
//! `ResourceValues` — named-fields carrier for the three resource_gauge values.

/// Named-fields carrier for the three resource-gauge values (`actual`, `request`,
/// `limit`).
///
/// Replaces the previous positional `(f64, f64, f64)` triple on the constructor
/// and accessor surface — struct-literal construction (`ResourceValues { actual,
/// request, limit }`) and named destructuring (`let ResourceValues { actual,
/// request, limit } = state.values();`) both eliminate the "silently transpose
/// `request` and `limit`" hazard.
///
/// # Example
///
/// ```rust
/// use envision::component::ResourceValues;
///
/// let vals = ResourceValues {
///     actual: 250.0,
///     request: 500.0,
///     limit: 1000.0,
/// };
/// assert_eq!(vals.actual, 250.0);
/// assert_eq!(vals.request, 500.0);
/// assert_eq!(vals.limit, 1000.0);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serialization", derive(serde::Serialize, serde::Deserialize))]
pub struct ResourceValues {
    /// Current in-use value (e.g., current CPU consumption).
    pub actual: f64,
    /// Requested value (e.g., K8s pod resource request).
    pub request: f64,
    /// Hard limit (e.g., K8s pod resource limit).
    pub limit: f64,
}
```

- [ ] Save. Note that this file is only ~30 lines — well under the 1000-line cap.

### Step 2: Add module declaration + import in `src/component/resource_gauge/mod.rs`

- [ ] Open `src/component/resource_gauge/mod.rs`. Locate the top-of-file `mod` declarations (or the position where sibling modules would go). Add:

```rust
mod values;

pub use values::ResourceValues;
```

If a sibling `mod` block already exists, insert into it. Place `pub use values::ResourceValues;` alongside any other public re-exports so the type is reachable as `envision::component::resource_gauge::ResourceValues` — which then means `envision::component::ResourceValues` (via `pub use crate::component::*;`) and `envision::prelude::ResourceValues` (via `pub use crate::component::*;` in the prelude at `src/lib.rs:459`).

- [ ] Save.

### Step 3: Verify `ResourceValues` reachable from crate root

- [ ] Run:

```bash
cargo check --all-features 2>&1 | tail -5
```

Expected: clean.

- [ ] Verify the re-export chain reaches the crate root. Write a temporary test at the bottom of `src/component/resource_gauge/values.rs`:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn resource_values_reachable_from_prelude() {
        // Compile-check only — this must resolve via prelude.
        let _vals: crate::component::ResourceValues = crate::component::ResourceValues::default();
        let _vals: crate::prelude::ResourceValues = crate::prelude::ResourceValues::default();
    }
}
```

- [ ] Run:

```bash
cargo test --all-features resource_values_reachable_from_prelude 2>&1 | tail -5
```

Expected: pass. If `crate::component::ResourceValues` doesn't resolve, verify Step 2's `pub use` line. If `crate::prelude::ResourceValues` doesn't resolve, verify `src/lib.rs:459` (`pub use crate::component::*;`) captures the new type — it should, because the module re-export is glob-based. If it doesn't, add `pub use crate::component::resource_gauge::ResourceValues;` explicitly in the prelude block at `src/lib.rs`.

- [ ] Once verified, delete the temporary test.

### Step 4: Delete positional `new(f64, f64, f64)` at `mod.rs:139`

- [ ] Locate the `pub fn new` at approximately line 139 of `src/component/resource_gauge/mod.rs`:

```rust
    /// Creates a new ResourceGauge with the given values.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_gauge::ResourceGaugeState;
    ///
    /// let state = ResourceGaugeState::new(100.0, 200.0, 500.0);
    /// assert_eq!(state.actual(), 100.0);
    /// assert_eq!(state.request(), 200.0);
    /// assert_eq!(state.limit(), 500.0);
    /// ```
    pub fn new(actual: f64, request: f64, limit: f64) -> Self {
        Self {
            actual,
            request,
            limit,
            ..Default::default()
        }
    }
```

Delete the entire block including the docstring. Do not leave a stub or `#[deprecated]` — the spec calls for outright removal (breaking change).

- [ ] Save. Expect `cargo check` to now show ~76 errors in `tests.rs` — those are Step 9.

### Step 5: Add `with_values(ResourceValues) -> Self` builder

- [ ] In `src/component/resource_gauge/mod.rs`, immediately after where `new` was, add:

```rust
    /// Sets all three values from a named struct in a single call.
    ///
    /// The struct-literal form (`ResourceValues { actual, request, limit }`)
    /// names each field at construction, matching the intent of the removed
    /// positional `new(a, r, l)` without its transposition hazard.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ResourceGaugeState, ResourceValues};
    ///
    /// let state = ResourceGaugeState::default().with_values(ResourceValues {
    ///     actual: 250.0,
    ///     request: 500.0,
    ///     limit: 1000.0,
    /// });
    /// assert_eq!(state.actual(), 250.0);
    /// ```
    pub fn with_values(mut self, values: ResourceValues) -> Self {
        self.actual = values.actual;
        self.request = values.request;
        self.limit = values.limit;
        self
    }
```

### Step 6: Add `with_actual` + `with_request` + `with_limit` fluent builders

- [ ] Immediately after `with_values`, add three fluent methods:

```rust
    /// Sets the actual (in-use) value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ResourceGaugeState;
    ///
    /// let state = ResourceGaugeState::default().with_actual(250.0);
    /// assert_eq!(state.actual(), 250.0);
    /// ```
    pub fn with_actual(mut self, actual: f64) -> Self {
        self.actual = actual;
        self
    }

    /// Sets the requested value (e.g., K8s pod resource request).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ResourceGaugeState;
    ///
    /// let state = ResourceGaugeState::default().with_request(500.0);
    /// assert_eq!(state.request(), 500.0);
    /// ```
    pub fn with_request(mut self, request: f64) -> Self {
        self.request = request;
        self
    }

    /// Sets the hard limit (e.g., K8s pod resource limit).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ResourceGaugeState;
    ///
    /// let state = ResourceGaugeState::default().with_limit(1000.0);
    /// assert_eq!(state.limit(), 1000.0);
    /// ```
    pub fn with_limit(mut self, limit: f64) -> Self {
        self.limit = limit;
        self
    }
```

### Step 7: Add `values() -> ResourceValues` accessor

- [ ] Locate `set_values(&mut self, actual: f64, request: f64, limit: f64)` at approximately `mod.rs:503`. Immediately BEFORE (or after — position for reviewer scannability) `set_values`, add:

```rust
    /// Returns all three values as a named struct.
    ///
    /// Complements [`set_values`](Self::set_values); closes the audit
    /// scorecard accessor-symmetry gap flagged at v0.16.0.
    ///
    /// Returning `ResourceValues` (not a bare `(f64, f64, f64)` tuple) keeps
    /// destructuring safe — `let ResourceValues { actual, request, limit } =
    /// state.values()` binds each field by name, so callers can't silently
    /// transpose `request` and `limit`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ResourceGaugeState, ResourceValues};
    ///
    /// let state = ResourceGaugeState::default()
    ///     .with_actual(250.0)
    ///     .with_request(500.0)
    ///     .with_limit(1000.0);
    /// let ResourceValues { actual, request, limit } = state.values();
    /// assert_eq!(actual, 250.0);
    /// assert_eq!(request, 500.0);
    /// assert_eq!(limit, 1000.0);
    /// ```
    pub fn values(&self) -> ResourceValues {
        ResourceValues {
            actual: self.actual,
            request: self.request,
            limit: self.limit,
        }
    }
```

- [ ] Save.

### Step 8: Verify `mod.rs` compiles + line count under 1000

- [ ] Run:

```bash
cargo check --all-features 2>&1 | tail -20
```

Expected: errors only in `tests.rs` (76 sites). If `mod.rs` still emits errors, walk through Steps 4-7.

- [ ] Verify line count:

```bash
wc -l src/component/resource_gauge/mod.rs
```

Expected: line count under 1000 (baseline was ~654, adding ~120 lines of builder methods lands at ~774 — comfortably under). If it exceeds 995, mitigation is to move the builder methods to a sibling file `src/component/resource_gauge/builders.rs` — but this is unlikely to trigger.

### Step 9: Migrate 76 `ResourceGaugeState::new` call sites in `tests.rs`

- [ ] Open `src/component/resource_gauge/tests.rs`. Locate each `ResourceGaugeState::new(a, r, l)` call. For each site, apply one of two forms per site's intent — implementer picks per site to minimize diff noise:

**Form A — named-struct single call** (recommended for test fixtures where all three values are known up front):

Old:
```rust
let state = ResourceGaugeState::new(100.0, 200.0, 500.0);
```

New:
```rust
let state = ResourceGaugeState::default().with_values(ResourceValues {
    actual: 100.0,
    request: 200.0,
    limit: 500.0,
});
```

**Form B — fluent builder** (recommended when values are computed independently):

Old:
```rust
let mut state = ResourceGaugeState::new(0.0, 100.0, 200.0);
```

New:
```rust
let mut state = ResourceGaugeState::default()
    .with_actual(0.0)
    .with_request(100.0)
    .with_limit(200.0);
```

- [ ] Add the `ResourceValues` import to `tests.rs` if using Form A anywhere (top of file):

```rust
use crate::component::ResourceValues;
```

Or use the local `use super::*;`-plus-`use crate::component::ResourceValues;` combo per the file's existing style.

- [ ] Migrate every site. There are approximately 76 references — the exact number can be verified with `grep -c 'ResourceGaugeState::new\b' src/component/resource_gauge/tests.rs`.

- [ ] Save.

- [ ] Grep-verify no remaining `ResourceGaugeState::new` sites across the tree:

```bash
grep -rn 'ResourceGaugeState::new\b\|resource_gauge::new\b' src/ examples/ tests/
```

Expected: zero hits.

- [ ] Run:

```bash
cargo nextest run --all-features -E 'test(resource_gauge)' 2>&1 | tail -15
```

Expected: all resource_gauge tests pass.

### Step 10: Create `examples/resource_gauge.rs` — K8s pod-quota demo

- [ ] Create `examples/resource_gauge.rs`. Aim for ~150 lines demonstrating the component with realistic K8s-style values. The example uses the new builder surface and the D6+D9 severity color mapping:

```rust
//! resource_gauge example — K8s pod-quota shape.
//!
//! Demonstrates the ResourceGauge component with a realistic Kubernetes
//! pod-quota scenario. Each row shows a pod's CPU (millicores) and memory (MB)
//! consumption against its request and limit.
//!
//! Surface exercised:
//! - ResourceGaugeState::default() + with_values(ResourceValues { .. }) builder
//! - Severity color mapping via theme.severity_color(Severity)
//! - Table<Pod> rendering multiple gauge cells per row
//!
//! Run with: cargo run --example resource_gauge --features data-components

use envision::component::{ResourceGaugeState, ResourceValues};
use envision::prelude::*;

/// A pod's resource snapshot at one point in time.
#[derive(Clone, Debug)]
struct Pod {
    name: String,
    cpu: ResourceValues,     // millicores
    memory: ResourceValues,  // MB
}

struct QuotaApp;

#[derive(Clone)]
struct State {
    pods: Vec<Pod>,
    selected: usize,
}

#[derive(Clone, Debug)]
enum Msg {
    Next,
    Prev,
    Quit,
}

impl App for QuotaApp {
    type State = State;
    type Message = Msg;
    type Args = ();

    fn init(_args: ()) -> (State, Command<Msg>) {
        let pods = vec![
            Pod {
                name: "api-server-x7fk2".into(),
                cpu: ResourceValues { actual: 350.0, request: 500.0, limit: 1000.0 },
                memory: ResourceValues { actual: 128.0, request: 256.0, limit: 512.0 },
            },
            Pod {
                name: "worker-queue-a3bcd".into(),
                cpu: ResourceValues { actual: 920.0, request: 1000.0, limit: 1000.0 },
                memory: ResourceValues { actual: 480.0, request: 512.0, limit: 512.0 },
            },
            Pod {
                name: "ingress-nginx-9k2s0".into(),
                cpu: ResourceValues { actual: 45.0, request: 100.0, limit: 500.0 },
                memory: ResourceValues { actual: 64.0, request: 128.0, limit: 256.0 },
            },
        ];
        (
            State {
                pods,
                selected: 0,
            },
            Command::none(),
        )
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Next => {
                state.selected = (state.selected + 1).min(state.pods.len().saturating_sub(1));
            }
            Msg::Prev => {
                state.selected = state.selected.saturating_sub(1);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        use envision::component::resource_gauge::ResourceGauge;
        let area = frame.area();
        let theme = Theme::default();

        // Show a full-frame gauge for the currently selected pod's CPU usage.
        let pod = &state.pods[state.selected];
        let gauge_state = ResourceGaugeState::default()
            .with_values(pod.cpu)
            .with_title(format!(" {} — CPU (millicores) ", pod.name))
            .with_units("m".into())
            .with_show_legend(true);

        let mut ctx = RenderContext::new(frame, area, &theme).focused(true);
        <ResourceGauge as Component>::view(&gauge_state, &mut ctx);
    }

    fn handle_event(event: &Event) -> Option<Msg> {
        let key = event.as_key()?;
        match key.code {
            Key::Down | Key::Char('j') => Some(Msg::Next),
            Key::Up | Key::Char('k') => Some(Msg::Prev),
            Key::Char('q') | Key::Esc => Some(Msg::Quit),
            _ => None,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<QuotaApp, _>::virtual_builder(80, 12).build()?;

    println!("=== ResourceGauge Example ===\n");

    vt.tick()?;
    println!("Pod 1 (api-server, 350/500/1000 millicores CPU):");
    println!("{}\n", vt.display());

    vt.dispatch(Msg::Next);
    vt.tick()?;
    println!("Pod 2 (worker-queue, 920/1000/1000 — near limit):");
    println!("{}\n", vt.display());

    vt.dispatch(Msg::Next);
    vt.tick()?;
    println!("Pod 3 (ingress-nginx, 45/100/500 — comfortably under):");
    println!("{}\n", vt.display());

    Ok(())
}
```

- [ ] Save. Note that the example uses `ResourceGaugeState`'s builder surface AND the new `ResourceValues` type — exercises everything that ships in this Unit.

- [ ] If `ResourceGaugeState` doesn't have `with_title`, `with_units`, or `with_show_legend` builders (verify against `mod.rs`), adjust the example to use whatever the current builder surface offers. The specific setter names in the example are illustrative; the core new API being exercised is `.with_values(ResourceValues { .. })`.

### Step 11: Add `[[example]]` block in `Cargo.toml`

- [ ] Open `Cargo.toml`. Locate the block for the existing `drilldown` example (added in the D8 cadence). Immediately after that block, add:

```toml
[[example]]
name = "resource_gauge"
required-features = ["data-components"]
```

- [ ] Save.

- [ ] Verify:

```bash
cargo build --example resource_gauge --features data-components 2>&1 | tail -5
```

Expected: clean build.

- [ ] Run the example end-to-end:

```bash
cargo run --example resource_gauge --features data-components 2>&1 | head -20
```

Expected: three sections of output showing rendered ResourceGauge frames for each pod.

### Step 12: Full verification for Task 3

- [ ] Run:

```bash
cargo fmt --check
```

Expected: clean.

- [ ] Run:

```bash
cargo clippy --all-features -- -D warnings 2>&1 | tail -10
```

Expected: clean.

- [ ] Run:

```bash
cargo nextest run --all-features 2>&1 | tail -10
```

Expected: all tests pass (including the 76 migrated tests).

- [ ] Run:

```bash
cargo test --all-features --doc 2>&1 | tail -10
```

Expected: all doc tests pass (including the 5 new `# Example` blocks on `with_values`, `with_actual`, `with_request`, `with_limit`, `values()` + the `ResourceValues` struct docstring).

- [ ] Run:

```bash
cargo build --no-default-features 2>&1 | tail -3
```

Expected: clean.

- [ ] Run (the D8 lesson — catches example-gating drift):

```bash
cargo test --no-default-features --no-run 2>&1 | tail -5
```

Expected: clean. The `[[example]]` block with `required-features = ["data-components"]` prevents this from trying to build `resource_gauge.rs` without the required features.

- [ ] Run:

```bash
cargo build --examples --all-features 2>&1 | tail -3
```

Expected: clean. Verifies the new example is included in the all-features example build.

- [ ] Run:

```bash
cargo doc --no-deps --all-features 2>&1 | grep -iE "warning|error" | head -10
```

Expected: zero intra-doc-link warnings.

- [ ] Run:

```bash
./tools/audit/target/release/envision-audit all 2>&1 | grep -iE "scorecard|baseline|resource" | head -10
```

Expected: scorecard 9/9 (resource_gauge accessor-symmetry gap closed).

### Step 13: Commit Task 3

- [ ] Stage:

```bash
git add src/component/resource_gauge/mod.rs src/component/resource_gauge/values.rs src/component/resource_gauge/tests.rs examples/resource_gauge.rs Cargo.toml
```

- [ ] Commit:

```bash
git commit -S -m "$(cat <<'EOF'
resource-gauge-closure: builder + values() + example (audit finding #5)

Address Fable audit finding #5 (resource_gauge asymmetries). Three
related fixes ship together:

1. Delete positional new(actual, request, limit) at mod.rs:139. Three
   unlabeled f64 args where transposing request/limit was silent — the
   exact ergonomic hazard the audit flagged. Pre-1.0 breaking change.

2. Introduce ResourceValues struct in new sibling module
   src/component/resource_gauge/values.rs. Named-fields carrier
   (actual, request, limit) with #[derive(Clone, Copy, Debug, Default,
   PartialEq)] + serde-gated Serialize/Deserialize. Re-exported at
   crate root via the existing pub use crate::component::*; chain.

3. Add ResourceGaugeState builder surface + closure of the accessor
   symmetry gap:
   - with_values(ResourceValues) -> Self — single-call struct-literal
     form, matches the intent of the removed new() but with named
     fields
   - with_actual(f64), with_request(f64), with_limit(f64) — fluent
     builder for when values are computed independently
   - values(&self) -> ResourceValues — matching accessor for the
     existing set_values multi-field mutator; closes the 9/9 -> 8/9
     scorecard regression flagged at v0.16.0

76 ResourceGaugeState::new call sites migrated in
src/component/resource_gauge/tests.rs. Implementer picked between
with_values struct-literal form (test fixtures) and fluent builder
(computed-value sites) per site to minimize diff noise.

New example: examples/resource_gauge.rs (~150 lines) demonstrates
K8s pod-quota scenario. Cargo.toml [[example]] block added with
required-features = ["data-components"] (D8 lesson — no example
declaration means cargo test --no-default-features tries to build
the example and fails on undeclared-type errors).

Every new pub fn has a # Example doc-test (audit-coverage discipline).

Grep verified: `grep -rn 'ResourceGaugeState::new\b\|resource_gauge::new\b'
src/ examples/ tests/` returns zero hits post-migration.

Verification: cargo fmt clean; cargo clippy --all-features -D warnings
clean; cargo nextest run --all-features passes; cargo test --doc passes
(5 new # Example blocks); cargo build --no-default-features clean;
cargo test --no-default-features --no-run clean; cargo build --examples
--all-features includes resource_gauge; audit scorecard 9/9 restored.

Sets up MIGRATION.md fill-in at Task 4.
EOF
)"
```

- [ ] Verify signature.

---

## Task 4: MIGRATION.md fill-ins + full verification gauntlet

**Files:**
- Modify: `MIGRATION.md` (fill the placeholder Unit 2 + Unit 3 sections with the final tables)

**Interfaces:**
- Consumes: The `[Unreleased]` block already covers Unit 2 (`#### FileSortDirection removed` sub-sub-section) and Unit 3 (`#### ResourceGaugeState::new replaced by named-struct + builder`) from Task 1. `MIGRATION.md` gets the corresponding cross-referenced tables in Task 4.
- Produces: MIGRATION.md fully complete for v0.16→v0.17 with tables for every breaking change in the `[Unreleased]` block.

### Step 1: Fill the FileSortDirection MIGRATION.md placeholder

- [ ] Open `MIGRATION.md`. Locate the "Placeholder for FileSortDirection migration" line under `## v0.16.x to v0.17.0`. Replace with:

```markdown
### `FileSortDirection` removed; use `table::SortDirection`

`file_browser::FileSortDirection` deleted. `file_browser` now uses `crate::component::table::SortDirection` at every use site (canonical single path — no local re-export at the `file_browser` boundary). The two enums had identical 2-variant Ascending/Descending shape; unification eliminates two-names-for-one-concept.

`SortDirection` also derives `Copy + Default` (where `FileSortDirection` didn't), forcing a getter-shape improvement: `sort_direction()` returns by value.

| Old | New |
|---|---|
| `use envision::component::file_browser::FileSortDirection;` | `use envision::component::table::SortDirection;` |
| `FileSortDirection::Ascending` | `SortDirection::Ascending` |
| `FileSortDirection::Descending` | `SortDirection::Descending` |
| `FileBrowserOutput::SortChanged(field, FileSortDirection::Ascending)` | `FileBrowserOutput::SortChanged(field, SortDirection::Ascending)` |
| `fn sort_direction(&self) -> &FileSortDirection` | `fn sort_direction(&self) -> SortDirection` (by value; `SortDirection: Copy`) |
| `let dir = *state.sort_direction();` | `let dir = state.sort_direction();` (no deref needed — returns by value) |
| `match state.sort_direction() { FileSortDirection::Ascending => …, FileSortDirection::Descending => … }` | `match state.sort_direction() { SortDirection::Ascending => …, SortDirection::Descending => … }` |

Bonus: `SortDirection::toggle()` is available; use it to replace hand-rolled asc/desc flips.
```

### Step 2: Fill the ResourceGaugeState MIGRATION.md placeholder

- [ ] Locate the "Placeholder for ResourceGaugeState migration" line. Replace with:

```markdown
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
```

- [ ] Save.

### Step 3: Full-tree verification gauntlet

- [ ] `cargo fmt --check`

Expected: clean.

- [ ] `cargo clippy --all-features -- -D warnings`

Expected: no warnings.

- [ ] `cargo nextest run --all-features`

Expected: all tests pass. Note the migrated tests from Task 3.

- [ ] `cargo test --all-features --doc`

Expected: all doc tests pass — including the new README code blocks (Task 1), the file_browser docstring examples (Task 2), and the resource_gauge doc examples (Task 3).

- [ ] `cargo build --no-default-features`

Expected: clean.

- [ ] `cargo test --no-default-features --no-run`

Expected: clean. Catches example-gating drift (D8 lesson — `resource_gauge.rs` is gated behind `data-components`).

- [ ] `cargo build --examples --all-features`

Expected: clean. Verifies `drilldown` and `resource_gauge` both build.

- [ ] `cargo doc --no-deps --all-features 2>&1 | grep -iE "warning|error" | head -10`

Expected: zero intra-doc-link warnings.

- [ ] `./tools/audit/target/release/envision-audit all 2>&1 | grep -iE "scorecard|baseline" | head -10`

Expected: 9/9 scorecard (resource_gauge accessor-symmetry closed; other scorecard items unchanged).

### Step 4: Commit Task 4

- [ ] Stage:

```bash
git add MIGRATION.md
```

- [ ] Commit:

```bash
git commit -S -m "$(cat <<'EOF'
MIGRATION.md: fill Unit 2 + Unit 3 sections; verification gauntlet clean

Complete the v0.16.x -> v0.17.0 MIGRATION.md section by replacing the
two placeholder rows with full before/after migration tables:

- FileSortDirection removed section: import path change, enum variant
  path change, FileBrowserOutput signature change, sort_direction()
  return-type shape change (by-value for Copy types), getter-deref
  removal, and mention of SortDirection::toggle() as the replacement
  for hand-rolled asc/desc flips.
- ResourceGaugeState::new replaced section: named-struct form,
  fluent-builder form, values() accessor closing the set_values gap,
  and new ResourceValues type at both envision::component:: and
  envision::prelude:: paths.

Verification gauntlet run in full and clean:
- cargo fmt --check
- cargo clippy --all-features -- -D warnings
- cargo nextest run --all-features
- cargo test --all-features --doc
- cargo build --no-default-features
- cargo test --no-default-features --no-run (D8 lesson)
- cargo build --examples --all-features
- cargo doc --no-deps --all-features (zero intra-doc-link warnings)
- audit tool: 9/9 scorecard (resource_gauge asymmetry closed)

Final CHANGELOG [Unreleased] block content is unchanged from Task 1's
Step 6 — it already includes the FileSortDirection and
ResourceGaugeState::new sub-sub-sections plus the Known Deferred
Findings block. Only MIGRATION.md tables are added here.

Impl PR ready to open at Task 5.
EOF
)"
```

- [ ] Verify signature.

---

## Task 5: Push impl branch + open impl PR

**Files:** none directly — this is a mechanical push + `gh pr create`.

**Interfaces:**
- Consumes: three signed impl commits from Tasks 1-3 plus the MIGRATION.md fill-in from Task 4.
- Produces: open PR against `main`.

### Step 1: Confirm branch state

- [ ] Run:

```bash
git log --oneline -5
```

Expected (in order, most recent first):
- Task 4 (MIGRATION.md fill-in) commit
- Task 3 (resource-gauge-closure) commit
- Task 2 (file-browser-sort) commit
- Task 1 (release-hygiene) commit
- Whatever the branch parent was (should be at or near current `main`, depending on when the impl branch was created)

If any commit is missing or misordered, STOP and reconcile before pushing.

### Step 2: Merge latest `origin/main` into the impl branch

- [ ] Run:

```bash
git fetch origin main
git merge origin/main --no-ff -S -m "Merge origin/main into release-readiness-impl"
```

If merge conflicts, resolve them (most likely candidates: `CHANGELOG.md` if a hotfix has landed since branch creation; `Cargo.toml` if a version bump snuck in).

If signing the merge commit fails, STOP and ask the user.

### Step 3: Push impl branch

- [ ] Run:

```bash
git push -u origin release-readiness-impl
```

### Step 4: Open impl PR

- [ ] Run:

```bash
gh pr create --title "Impl: release-readiness cadence (v0.17.0 pre-release)" --body "$(cat <<'EOF'
## Summary

Closes all 5 blocking findings from Fable's 2026-07-04 audit (`A-`, 3.62 GPA — regressed from v0.15.1's `A`, 4.02 GPA) as one coordinated cadence before cutting v0.17.0. Three units shipped as three logically-separated signed commits + MIGRATION.md fill-in:

- **Unit 1 — release-hygiene**: README examples fixed (TEA snippet gains `type Args = ();` + `fn init(_args: ())`; testing snippet replaced with correct AppHarness idiom); `#[cfg(doctest)] pub struct ReadmeDoctests;` block in `src/lib.rs` wires README code blocks into `cargo test --doc`; visible imports (no hidden `# use ...` — real users copy-paste); Feature Flags section added; 73 → 74 component count fix; three stacked `[Unreleased]` CHANGELOG blocks consolidated into one with topic-preserved `#### ` sub-sub-sections under Keep-a-Changelog `### ` kinds; "Known Deferred Findings" block making audit findings #6 and #8 visible in release notes; MIGRATION.md backfilled with `v0.15.x to v0.16.0` (small — `DependencyGraph → Diagram`) and the shell of `v0.16.x to v0.17.0`.

- **Unit 2 — file-browser-sort**: `FileSortDirection` deleted at `src/component/file_browser/types.rs:314-323`. `crate::component::table::SortDirection` used at every reference (Option A — no local re-export). `SortDirection` derives `Copy + Default` so `sort_direction()` returns by value (was `&FileSortDirection`) — third breaking change on `file_browser`, documented in MIGRATION.md. Toggle branches collapse to `state.sort_direction.toggle()`. 24 references migrated across 6 files.

- **Unit 3 — resource-gauge-closure**: `ResourceGaugeState::new(f64, f64, f64)` deleted. New `ResourceValues { actual, request, limit }` named-fields struct in sibling `src/component/resource_gauge/values.rs` (`Clone + Copy + Debug + Default + PartialEq` + serde-gated). New `with_values(ResourceValues)` single-call constructor + fluent `with_actual`/`with_request`/`with_limit` builders + `values() -> ResourceValues` accessor (closes 9/9 → 8/9 scorecard regression from v0.16.0). 76 `new` call sites migrated. New `examples/resource_gauge.rs` (~150 lines, K8s pod-quota shape); `Cargo.toml` `[[example]]` gated on `data-components`.

## Spec / plan

- Spec: `docs/superpowers/specs/2026-07-04-release-readiness-cadence-design.md` (PR #502)
- Plan: `docs/superpowers/plans/2026-07-04-release-readiness-cadence.md` (open the plan PR before this impl PR and reference the resulting PR number here at impl PR creation time)

## Design decisions from spec brainstorm + adversarial review

- file_browser: **Option A** (full migration to `table::SortDirection` at every use site) — no local re-export at file_browser boundary. Option B would perpetuate the "same type under two paths" pattern the audit finding calls out.
- resource_gauge: **`ResourceValues` struct** for both `with_values()` construction AND `values()` accessor — NOT a `(f64, f64, f64)` tuple. The tuple would recreate the exact positional-f64 hazard the constructor deletion fixes.
- resource_gauge: **breaking replace** (delete positional `new`) — cleaner than dual-entry; pre-1.0 with two Breaking cadences already queued.
- CHANGELOG: **single `[Unreleased]` block that `/release` renames at release time** — follows Keep a Changelog convention.
- README doctest wiring: **`#[cfg(doctest)] pub struct ReadmeDoctests;`** — NOT crate-root `#![doc = include_str!(..)]`, which would replace the curated `//!` module docs on docs.rs.
- README code block shape: **visible imports** (no hidden `# use ...`). Copy-paste fidelity.

## Test plan

- [x] Unit 1: `cargo test --doc --all-features` picks up new README doctests (was: they never compiled)
- [x] Unit 2: 24 `FileSortDirection` references migrated to `table::SortDirection`; `grep -rn 'FileSortDirection' src/ examples/ tests/` returns zero hits
- [x] Unit 3: 76 `ResourceGaugeState::new` sites migrated; `grep -rn 'ResourceGaugeState::new\b' src/ examples/ tests/` returns zero hits
- [x] Unit 3: new `examples/resource_gauge.rs` builds via `cargo build --example resource_gauge --features data-components`
- [x] `cargo fmt --check` — clean
- [x] `cargo clippy --all-features -- -D warnings` — clean
- [x] `cargo nextest run --all-features` — all tests pass
- [x] `cargo test --all-features --doc` — all doc tests pass (README + resource_gauge builder docs)
- [x] `cargo build --no-default-features` — clean
- [x] `cargo test --no-default-features --no-run` — clean (D8 lesson)
- [x] `cargo build --examples --all-features` — includes `drilldown` and `resource_gauge`
- [x] `cargo doc --no-deps --all-features` — zero intra-doc-link warnings
- [x] `./tools/audit/target/release/envision-audit all` — **9/9 scorecard restored**

Next steps after this PR merges: tracking-doc PR closing the release-readiness milestone; Fable re-audit gate; `/release minor` cutting v0.17.0.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

Expected: PR URL returned. Note the PR number for the tracking-doc PR's reference.

### Step 5: CI watch

- [ ] Run `gh pr checks <PR_NUMBER>` periodically until all required checks complete.

If any check fails:
- Read the failure log: `gh run view <RUN_ID> --log-failed`
- Diagnose the root cause
- Fix in a follow-up signed commit on the same branch
- Push

The Coverage check has a known tarpaulin flake pattern (see the D3+D7+D8 cadence session) — if it fails with "Failed to run tests: Error: Timed out waiting for test response" AFTER `test result: ok. NNNN passed`, treat as infra flake and retrigger via `gh run rerun <RUN_ID> --failed`. Coverage is not in the required-checks list per branch protection.

Do not attempt to merge until all required checks pass.

### Step 6: Merge after approval

- [ ] After the tracking-doc PR + Fable re-audit gate are cleared:

```bash
gh pr merge <PR_NUMBER> --squash --delete-branch
```

- [ ] After merge, the impl is complete. Next: tracking-doc PR (NOT part of this plan; opened on a separate branch after this PR lands).

---

## Out of scope for this plan

- Tracking-doc PR closure (separate branch + PR after impl merges — marks all 5 findings resolved in a new `docs/audits/2026-07-XX-post-release-hygiene.md`, includes the Fable re-audit report).
- Fable re-audit (dispatched from the main session, not from within the impl flow).
- `/release minor` for v0.17.0 (dispatched after tracking-doc PR merges and re-audit confirms grade ≥ A).
- Feature work for v0.17.0+ (K8s-driven components — separate roadmap track).
- Audit findings #6 (`selected_value` incoherence) and #8 (dep leakage) — explicitly deferred to future cadences per the CHANGELOG `Known Deferred Findings` block.

## Recovery patterns from prior cadences

- **`git commit -S` fails** → STOP. Ask the user. Never bypass with `--no-gpg-sign`.
- **`cargo fmt --check` drifts mid-task** → Run `cargo fmt`, stage, add a small follow-up signed commit (e.g., `fmt: cargo fmt drift after Task N`). Don't amend.
- **`ResourceValues` re-export doesn't reach the prelude via glob** → Add `pub use crate::component::resource_gauge::ResourceValues;` explicitly in the prelude block at `src/lib.rs`.
- **README block fails doctest after include_str! wire** → Read the exact rustdoc error, add the missing visible imports to the block, re-run `cargo test --doc`. Do NOT add hidden `# use ...` lines.
- **File_browser initializer at mod.rs:96 can drop to `Default::default()`** → judgment call at impl-time. If the surrounding `Default` impl uses explicit field-by-field construction (the plan's default assumption), leave as `sort_direction: SortDirection::Ascending`. If it uses `Default::default()` chain, drop the line (Ascending is `#[default]` on `SortDirection`).
- **Coverage CI check flakes with tarpaulin timeout after tests pass** → `gh run rerun <RUN_ID> --failed`. Coverage is not required for merge per branch protection.
- **Audit tool reports scorecard < 9/9 after Task 3** → the resource_gauge asymmetry is the one restored gap. If a NEW gap surfaces, examine the audit output for which accessor/mutator pair is unbalanced; add the missing symmetry method to close it.
- **`cargo test --no-default-features --no-run` fails on `resource_gauge` example** → Cargo.toml `[[example]]` block is missing or has wrong `required-features`. Add or fix.

## Reference

- Spec: `docs/superpowers/specs/2026-07-04-release-readiness-cadence-design.md` (PR #502; commits `c93c0e2` + `169d291`).
- Fable audit report (2026-07-04): `A-`, 3.62 GPA. Findings summarized in spec's Scope section. Report will be checked in at `docs/audits/2026-07-04-pre-release-hygiene.md` as part of the tracking-doc PR.
- Prior 10 leadline cadence pattern (May 2026): brainstorm → spec PR → plan PR → impl PR → tracking-doc PR. Same 4-PR pattern here, plus re-audit gate before release.
- D3+D7+D8 plan (`docs/superpowers/plans/2026-05-24-docs-suite-d3-d7-d8.md`): style precedent for this plan's shape.
- CLAUDE.md project rules: PRs required; signed commits; squash-merge; merge `origin/main` before push; files under 1000 lines; no clippy warnings; no TODOs without tracking doc.
