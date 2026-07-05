# Consistency-cleanup cadence (v0.17.0 pre-release, Cadence A) — design

## Purpose

Close two trust-eroding findings from the 2026-07-05 audit — the `selected_value` / `selected_item` / `active_tab` accessor divergence across five components, and the one high-value dep-leakage site (`tokio::sync::mpsc::Sender` in the AppHarness surface) — before cutting v0.17.0. Both are breaking changes; both bundle cleanly into the same MIGRATION.md v0.16→v0.17 section already established by the release-readiness cadence.

- **Audit source:** In-session `/audit` at 2026-07-05 morning ranked these as the top-2 trust-eroders (finding #1 and #2 in that report). Prior Fable audit 2026-07-04 flagged them as deferred (findings #6 and #8); the release-readiness cadence honestly documented them in CHANGELOG's Known Deferred Findings block. This cadence closes both.
- **Terminal state:** All five components expose a single canonical `selected_item()` accessor; the AppHarness `message_sender()` returns a first-party `MessageSender<A>` newtype instead of raw `tokio::sync::mpsc::Sender`; CHANGELOG's Known Deferred Findings block is trimmed to only the still-deferred items; MIGRATION.md v0.16→v0.17 gets a new section covering all the renames.

## Scope

Three code-shaped units:

| Unit | What | Files primarily touched |
|---|---|---|
| 1 | Consistency sweep — canonical `selected_item()` across 5 components; delete literal aliases; rename semantically distinct accessors to disambiguate | `dropdown/mod.rs`, `select/mod.rs`, `heatmap/mod.rs`, `tab_bar/mod.rs`, `data_grid/state.rs`, plus tests and any example usage |
| 2 | `MessageSender<A>` newtype wrapping `tokio::sync::mpsc::Sender<A::Message>` + `Position` cosmetic at `virtual_terminal.rs:147` | `src/harness/message_sender.rs` (new), `src/harness/mod.rs`, `src/harness/app_harness/mod.rs`, `src/app/runtime/virtual_terminal.rs` |
| 3 | CHANGELOG + MIGRATION.md updates — new v0.16→v0.17 subsections, trimmed Known Deferred Findings block | `CHANGELOG.md`, `MIGRATION.md` |

Out of scope (explicitly):

- The `Color` / `Style` / `Widget` / `Cell` / `Rect` uses the prior audit named as "dep leakage" but which review shows are legitimate escape hatches or already routed through envision's own re-exports. Explicit list + justification in the CHANGELOG update.
- Deprecation shims. Pre-1.0, delete-outright pattern matches D5/D14/G7/D12/D3/D8/resource_gauge/FileSortDirection precedent.
- `compact_str` sporadic-adoption commitment decision (finding #4). Its own cadence when the time comes.
- 22 examples that `use ratatui::...` directly (finding #5). Cosmetic; skip until it matters.
- Doc-hygiene split of CHANGELOG.md and MIGRATION.md over 1000-line human cap (finding #3). Cadence B, separate brainstorm.

## Cadence structure

Standard 4-PR pattern:

1. Spec PR (this)
2. Plan PR
3. Impl PR — three signed commits (Unit 1 / Unit 2 / Unit 3 = CHANGELOG + MIGRATION.md)
4. Tracking-doc PR — check in `docs/audits/2026-07-05-post-consistency-cleanup.md` verification record; mark audit findings #6 and #8 CLOSED

No re-audit gate this time. The audit changes needed to close the findings are mechanical and the plan's verification gauntlet is sufficient. A re-audit is dispatched from the main session only if a genuine question emerges from the impl review.

## Unit 1 — Consistency sweep on `selected_item()`

Files touched: `src/component/dropdown/mod.rs`, `src/component/select/mod.rs`, `src/component/heatmap/mod.rs`, `src/component/tab_bar/mod.rs`, `src/component/data_grid/state.rs`, plus their `tests.rs`/`tests/` files, plus any test in `tests/` or example in `examples/` that references a deleted accessor.

### Per-component changes

**dropdown** (`src/component/dropdown/mod.rs:251,269`):

| Before | After |
|---|---|
| `selected_value() -> Option<&str>` — literal alias | *deleted* |
| `selected_item() -> Option<&str>` | *unchanged; only accessor* |

**select** (`src/component/select/mod.rs:239,257`):

Same as dropdown — both accessors are literal aliases returning `Option<&str>`. Delete `selected_value()`; keep `selected_item()`.

**heatmap** (`src/component/heatmap/mod.rs:466`):

| Before | After |
|---|---|
| `selected_value() -> Option<f64>` (returns the DATA VALUE at the cursor coordinates — not "the item that is selected") | `selected_cell_value() -> Option<f64>` (same body; renamed to signal it's a value at coordinates, not a collection-selection accessor) |

Rationale: the return type (`f64`) already telegraphs that this isn't the selected-item pattern. The rename makes the semantics explicit. Follow-up docstring update in the same commit: emphasize that this is the value under the cursor, not "which cell is selected" (that is what `selected(&self) -> Option<(usize, usize)>` at the top of the state's accessor block returns).

**tab_bar** (`src/component/tab_bar/mod.rs:297,322,336,351`):

| Before | After |
|---|---|
| `selected_index() -> Option<usize>` | *unchanged* |
| `selected() -> Option<usize>` — literal alias for `selected_index()` | *deleted* |
| `active_tab() -> Option<&Tab>` | `selected_item() -> Option<&Tab>` |
| `active_tab_mut() -> Option<&mut Tab>` | `selected_item_mut() -> Option<&mut Tab>` |

Rationale: `active_tab` is a semantic outlier — every other component with "which one is picked?" uses `selected_item()`. Rename is a one-word substitution across the accessor bodies + docstrings.

**data_grid** (`src/component/data_grid/state.rs:119,143,167,191`):

| Before | After |
|---|---|
| `selected_index() -> Option<usize>` | *unchanged* |
| `selected() -> Option<usize>` — literal alias for `selected_index()` | *deleted* |
| `selected_row() -> Option<&T>` — literal alias for `selected_item()` | *deleted* |
| `selected_item() -> Option<&T>` | *unchanged* |

Also unchanged: `set_selected(Option<usize>)` mutator; `selected_column() -> usize` (different concept — column cursor position, not a selection alias).

### Migration table (goes into MIGRATION.md at Unit 3)

```markdown
### Component selection accessors unified on `selected_item()`

Every component with a "which is selected?" accessor now uses the same
canonical shape: `selected_item() -> Option<&T>` (or `Option<&str>` for
string-keyed components). Literal aliases and semantic outliers are removed.

| Component | Old | New |
|---|---|---|
| `dropdown` | `state.selected_value()` | `state.selected_item()` |
| `select` | `state.selected_value()` | `state.selected_item()` |
| `heatmap` | `state.selected_value() -> Option<f64>` (returns data value at cursor) | `state.selected_cell_value() -> Option<f64>` (renamed — this was never a "selected item" accessor) |
| `tab_bar` | `state.active_tab() -> Option<&Tab>` | `state.selected_item() -> Option<&Tab>` |
| `tab_bar` | `state.active_tab_mut() -> Option<&mut Tab>` | `state.selected_item_mut() -> Option<&mut Tab>` |
| `tab_bar` | `state.selected()` (was: literal alias for `selected_index()`) | `state.selected_index()` |
| `data_grid` | `state.selected()` (was: literal alias for `selected_index()`) | `state.selected_index()` |
| `data_grid` | `state.selected_row()` (was: literal alias for `selected_item()`) | `state.selected_item()` |
```

## Unit 2 — `MessageSender<A>` newtype + Position cosmetic

Files touched: new `src/harness/message_sender.rs`, `src/harness/mod.rs`, `src/harness/app_harness/mod.rs`, `src/app/runtime/virtual_terminal.rs`.

### `MessageSender<A>` design

New file `src/harness/message_sender.rs` (~50 lines):

```rust
//! `MessageSender<A>` — first-party wrapper around the async message channel
//! that carries `A::Message` between the AppHarness and its Runtime.

use tokio::sync::mpsc;

/// Hands the caller a way to inject messages into the AppHarness's Runtime
/// asynchronously — from subscription callbacks, spawned tasks, or any other
/// non-App-loop code path.
///
/// Wraps `tokio::sync::mpsc::Sender<A::Message>` so envision consumers don't
/// need `tokio` as a direct dependency to use `AppHarness::message_sender()`.
/// The Sender's semantics are preserved (bounded, cloneable, send returns
/// `Result` on receiver-dropped) but the tokio-specific error type is
/// wrapped in [`MessageSendError`].
pub struct MessageSender<A: crate::app::App> {
    inner: mpsc::Sender<A::Message>,
}

impl<A: crate::app::App> MessageSender<A> {
    pub(crate) fn new(inner: mpsc::Sender<A::Message>) -> Self {
        Self { inner }
    }

    /// Sends a message into the AppHarness. Returns an error only when the
    /// AppHarness (and hence the receiver) has been dropped.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use envision::prelude::*;
    /// # async fn example<A: App>(sender: MessageSender<A>, msg: A::Message) {
    /// sender.send(msg).await.expect("harness still alive");
    /// # }
    /// ```
    pub async fn send(&self, msg: A::Message) -> Result<(), MessageSendError<A::Message>> {
        self.inner.send(msg).await.map_err(|e| MessageSendError(e.0))
    }

    /// Attempts to send a message without waiting. Returns an error if the
    /// channel is full or the AppHarness has been dropped.
    pub fn try_send(&self, msg: A::Message) -> Result<(), TrySendError<A::Message>> {
        self.inner.try_send(msg).map_err(TrySendError::from_tokio)
    }
}

impl<A: crate::app::App> Clone for MessageSender<A> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

impl<A: crate::app::App> std::fmt::Debug for MessageSender<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageSender").finish_non_exhaustive()
    }
}

/// Error returned by [`MessageSender::send`] when the AppHarness receiver
/// has been dropped. Carries the message back so the caller can inspect it.
#[derive(Debug)]
pub struct MessageSendError<T>(pub T);

impl<T> std::fmt::Display for MessageSendError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "message sender: receiver dropped")
    }
}

impl<T: std::fmt::Debug> std::error::Error for MessageSendError<T> {}

/// Error returned by [`MessageSender::try_send`].
#[derive(Debug)]
pub enum TrySendError<T> {
    /// Channel is full; the message was NOT sent. Try again later.
    Full(T),
    /// AppHarness receiver has been dropped; the message was NOT sent.
    Closed(T),
}

impl<T> TrySendError<T> {
    fn from_tokio(err: mpsc::error::TrySendError<T>) -> Self {
        match err {
            mpsc::error::TrySendError::Full(t) => TrySendError::Full(t),
            mpsc::error::TrySendError::Closed(t) => TrySendError::Closed(t),
        }
    }
}

impl<T> std::fmt::Display for TrySendError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Full(_) => write!(f, "message sender: channel full"),
            Self::Closed(_) => write!(f, "message sender: receiver dropped"),
        }
    }
}

impl<T: std::fmt::Debug> std::error::Error for TrySendError<T> {}
```

Re-exported at:
- `src/harness/mod.rs`: `pub use message_sender::{MessageSender, MessageSendError, TrySendError};`
- `src/lib.rs` prelude: `pub use crate::harness::{AppHarness, TestHarness, MessageSender};` (extending the existing `harness::` re-export line at src/lib.rs:463)

### `app_harness/mod.rs` change

At `src/harness/app_harness/mod.rs:264`, the current signature:

```rust
pub fn message_sender(&self) -> tokio::sync::mpsc::Sender<A::Message> {
    self.runtime.message_sender()
}
```

Becomes:

```rust
pub fn message_sender(&self) -> MessageSender<A> {
    MessageSender::new(self.runtime.message_sender())
}
```

Consumer code changes from `sender.send(msg).await.unwrap()` to `sender.send(msg).await.unwrap()` — the call shape is identical, only the type flowing out of `message_sender()` is different. Downstream consumers who spelled the type explicitly must update `tokio::sync::mpsc::Sender<Msg>` → `MessageSender<A>`.

### `virtual_terminal.rs` Position cosmetic

At `src/app/runtime/virtual_terminal.rs:147`, the current signature:

```rust
pub fn find_text(&self, needle: &str) -> Vec<ratatui::layout::Position> {
```

Becomes:

```rust
pub fn find_text(&self, needle: &str) -> Vec<crate::layout::Position> {
```

Zero runtime change (`crate::layout::Position` is a re-export of `ratatui::layout::Position` at `src/layout/mod.rs`). The change removes a direct `ratatui::` path from the public API surface, matching how every other envision file references `Position` via envision's own re-export.

Not called out in MIGRATION.md — the returned type is IDENTICAL (Position is a re-export, not a newtype). Purely internal path canonicalization.

### Migration table addition

```markdown
### `MessageSender<A>` wraps `tokio::sync::mpsc::Sender<A::Message>`

`AppHarness::message_sender()` now returns a first-party `MessageSender<A>`
newtype instead of raw `tokio::sync::mpsc::Sender<A::Message>`. Consumer code
changes are limited to type spellings; call sites are identical.

| Old | New |
|---|---|
| `let sender: tokio::sync::mpsc::Sender<MyMsg> = harness.message_sender();` | `let sender: envision::MessageSender<MyApp> = harness.message_sender();` (or use `envision::prelude::MessageSender`) |
| `sender.send(msg).await` — returns `Result<(), tokio::sync::mpsc::error::SendError<MyMsg>>` | `sender.send(msg).await` — returns `Result<(), envision::MessageSendError<MyMsg>>` |
| `sender.try_send(msg)` — returns `Result<(), tokio::sync::mpsc::error::TrySendError<MyMsg>>` | `sender.try_send(msg)` — returns `Result<(), envision::TrySendError<MyMsg>>` |

`MessageSender<A>` is `Clone`. Its send/try_send methods have the same
semantics as tokio's Sender (bounded channel, receiver-dropped errors carry
the message back).
```

## Unit 3 — CHANGELOG + MIGRATION.md updates

Files touched: `CHANGELOG.md`, `MIGRATION.md`.

### CHANGELOG changes

Under `## [Unreleased]`:

- **`### Breaking Changes`** gets two new `#### ` sub-sub-sections:
  - `#### Component selection accessors unified on selected_item()` — narrative + migration table cross-reference
  - `#### MessageSender<A> replaces tokio::sync::mpsc::Sender in AppHarness surface` — narrative + migration table cross-reference

- **`### Added`** gets one new `#### ` sub-sub-section:
  - `#### MessageSender<A> + MessageSendError + TrySendError` — new types

- **`### Known Deferred Findings`** block updated: findings #6 and #8 REMOVED (this cadence closes both). Only pre-existing text that's still true stays. If the section becomes empty, delete the section header entirely. If any deferred findings from the earlier audit are still valid, they stay.

### MIGRATION.md changes

Under `## v0.16.x to v0.17.0`, append two new subsections at the bottom (after the ResourceGaugeState/FileSortDirection tables from the release-readiness cadence):

- `### Component selection accessors unified on selected_item()` — table from Unit 1
- `### MessageSender<A> wraps tokio::sync::mpsc::Sender<A::Message>` — table from Unit 2

## Testing strategy

- Every unit test that references a deleted accessor migrates to the new name (mechanical rename).
- Every doc-test that references a deleted accessor updates.
- Grep-verifiable success: `grep -rn 'active_tab\b\|selected_value\b' src/ tests/ examples/` returns hits ONLY at the `heatmap::selected_cell_value` site (which contains `selected_value` as a substring in its old-alias-name comment if any, otherwise zero hits) and CHANGELOG/MIGRATION migration tables. The 5 renamed accessors produce zero call-site hits at their old names.
- `MessageSender<A>` unit tests in the same style as EnvisionError tests (round-trip through send/try_send + error variants).
- Full verification gauntlet unchanged from prior cadences:
  - `cargo fmt --check`
  - `cargo clippy --all-features -- -D warnings`
  - `cargo nextest run --all-features`
  - `cargo test --all-features --doc`
  - `cargo build --no-default-features`
  - `cargo test --no-default-features --no-run` (D8 lesson)
  - `cargo build --examples --all-features`
  - `cargo doc --no-deps --all-features` (zero intra-doc-link warnings)
  - `./tools/audit/target/release/envision-audit all` — expect 9/9 scorecard (accessor-symmetry gaps stay at 0)

## Success criteria

1. All 5 component-consistency renames landed; grep-verified zero hits on `active_tab\b`, `selected_value\b` (except where the substring appears in `selected_cell_value`), and `selected_row\b`.
2. `AppHarness::message_sender()` returns `MessageSender<A>`; grep-verified zero direct `tokio::sync::mpsc::Sender` references in envision's public API surface (excluding `src/harness/message_sender.rs` itself and the private path inside app_harness).
3. `virtual_terminal.rs:147` returns `crate::layout::Position`; grep-verified zero `ratatui::layout::Position` references in public signatures.
4. CHANGELOG's Known Deferred Findings block reflects the closure of findings #6 and #8.
5. MIGRATION.md v0.16→v0.17 has both new migration tables.
6. Full verification gauntlet clean; audit scorecard 9/9 preserved.

## Risk register

- **`MessageSender<A>` generic parameter is `A: App`, not `A::Message`.** This is a deliberate choice — mirrors the existing `AppHarness<A>` and `Runtime<A, B>` shape, and lets us extend the type later (e.g., adding a `sender.send_all(msgs)` batch method) without breaking consumer signatures. Alternative was `MessageSender<M>` parameterized on the message type; rejected because it splits from the rest of the harness surface.
- **`tokio::sync::mpsc::error::TrySendError` has two variants in tokio's actual API (`Full`, `Closed`).** Verify at impl time — the spec's `TrySendError<T>` mirrors those two variants. If tokio's API has drifted since (unlikely at tokio 1.x), adjust.
- **Heatmap `selected_cell_value()` rename readers can miss.** Mitigation: the docstring on the renamed method explicitly states "renamed from `selected_value()` in v0.17.0 — see MIGRATION.md."
- **data_grid consumers who spelled `selected()` or `selected_row()` explicitly.** They see the compile error and the migration table names the exact rename. Same pattern as prior cadences.
- **Ripple effect through tests.** ~10-20 test sites per renamed accessor across the 5 components. Mechanical grep-and-migrate; verified via the grep-clean assertion.
- **`MessageSender<A>` uses `A: crate::app::App` bound — the `App` trait's re-export path.** Verify the path in impl. If `crate::app::App` doesn't resolve inside `src/harness/message_sender.rs`, use whatever the actual path is (likely just `crate::App` via a lib.rs re-export).

## Open questions

None. Design decisions resolved during brainstorm:
- Scope: findings #1 (consistency) + #2 (dep-leakage) bundled as one cadence
- Canonical accessor: `selected_item() -> Option<&T>` across all 5 components
- Heatmap rename: `selected_cell_value()` (semantically distinct, disambiguates from selection-accessor pattern)
- tab_bar: rename `active_tab` → `selected_item`; drop `selected` alias
- data_grid: keep `selected_index` + `selected_item` + `set_selected` + `selected_column`; drop `selected` + `selected_row` aliases
- Dep-leakage real scope: only `MessageSender<A>` newtype worth adding + the Position cosmetic
- Deprecate vs delete: delete outright (pre-1.0 precedent)
- MIGRATION.md: append two new subsections to v0.16→v0.17
- CHANGELOG Known Deferred Findings: remove findings #6 and #8 (closed by this cadence)

## Reference

- **2026-07-05 audit** (in-session `/audit`): A- (3.81 GPA); findings #1 and #2 in the trust-eroders list; the release-hygiene work landed successfully but consistency + dep-leakage remained.
- **2026-07-04 pre-release-hygiene audit** (Fable): A- (3.62); called these out as deferred findings #6 (selected_value incoherence) and #8 (dep leakage in 8 signatures).
- **2026-07-05 post-release-hygiene audit** (Fable): A (3.91); confirmed findings #6 and #8 as documented deferrals.
- **Precedent for delete-outright pre-1.0:** D5 paragraph→line rename, D14 same, resource_gauge::new deletion, FileSortDirection deletion (release-readiness cadence). Pattern is: outright deletion + MIGRATION.md table + no `#[deprecated]` shim.
- **Cadence pattern:** brainstorm → spec PR → plan PR → impl PR → tracking-doc PR (established across 11+ prior cadences).
- **After Cadence A merges, Cadence B is the doc-hygiene split of CHANGELOG.md + MIGRATION.md** (finding #3 from the same audit). Separate brainstorm.
