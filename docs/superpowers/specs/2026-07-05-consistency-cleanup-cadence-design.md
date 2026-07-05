# Consistency-cleanup cadence (v0.17.0 pre-release, Cadence A) — design

## Purpose

Close two trust-eroding findings from the 2026-07-05 audit — the `selected_value` / `selected_item` / `active_tab` accessor divergence across five components, and the one high-value dep-leakage site (`tokio::sync::mpsc::Sender` in the AppHarness surface) — before cutting v0.17.0. Both are breaking changes; both bundle cleanly into the same MIGRATION.md v0.16→v0.17 section already established by the release-readiness cadence.

- **Audit source:** In-session `/audit` at 2026-07-05 morning ranked these as the top-2 trust-eroders (finding #1 and #2 in that report). Prior Fable audit 2026-07-04 flagged them as deferred (findings #6 and #8); the release-readiness cadence honestly documented them in CHANGELOG's Known Deferred Findings block. This cadence closes both.
- **Terminal state:** All five components expose a single canonical `selected_item()` accessor; the AppHarness `message_sender()` returns a first-party `MessageSender<A>` newtype instead of raw `tokio::sync::mpsc::Sender`; CHANGELOG's Known Deferred Findings block is trimmed to only the still-deferred items; MIGRATION.md v0.16→v0.17 gets a new section covering all the renames.

## Scope

Three code-shaped units:

| Unit | What | Files primarily touched |
|---|---|---|
| 1 | Consistency sweep — canonical `selected_item()` across **6 components** (adversarial review M3: Table has byte-identical divergence to data_grid; leaving it out makes the "consistency sweep" framing dishonest, so Table is IN scope); delete literal aliases; rename semantically distinct accessors to disambiguate | `dropdown/mod.rs`, `select/mod.rs`, `heatmap/mod.rs`, `tab_bar/mod.rs`, `data_grid/state.rs`, `table/state.rs`, plus internal component-body call sites, tests, and any example usage |
| 2 | `MessageSender<M>` newtype wrapping `tokio::sync::mpsc::Sender<M>` + `Position` cosmetic at `virtual_terminal.rs:147` | `src/harness/message_sender.rs` (new), `src/harness/mod.rs`, `src/harness/app_harness/mod.rs`, `src/app/runtime/virtual_terminal.rs` |
| 3 | CHANGELOG + MIGRATION.md updates — new v0.16→v0.17 subsections, trimmed Known Deferred Findings block | `CHANGELOG.md`, `MIGRATION.md` |

Out of scope (explicitly):

- The `Color` / `Style` / `Widget` / `Cell` / `Rect` uses the prior audit named as "dep leakage" but which review shows are legitimate escape hatches or already routed through envision's own re-exports. Explicit list + justification in the CHANGELOG update.
- Deprecation shims. Pre-1.0, delete-outright pattern matches D5/D14/G7/D12/D3/D8/resource_gauge/FileSortDirection precedent.
- `compact_str` sporadic-adoption commitment decision (finding #4). Its own cadence when the time comes.
- 22 examples that `use ratatui::...` directly (finding #5). Cosmetic; skip until it matters.
- Doc-hygiene split of CHANGELOG.md and MIGRATION.md over 1000-line human cap (finding #3). Cadence B, separate brainstorm.
- **Extending the `selected()` alias removal beyond the 6 in-scope components** (adversarial review A3). Envision has 15+ other components with `selected() -> Option<usize>` as an alias for `selected_index()` — accordion, tabs, radio_group, searchable_list, selectable_list, tree, menu, box_plot, loading_list, alert_panel, and others. Trimming ALL of them in one cadence is a scope leap that would compound migration surface without proportional benefit — the audit specifically named the 6 in this scope because they carry the additional `selected_value` / `selected_row` / `active_tab` divergence. The remaining 15+ components' `selected()` aliases are candidates for **Cadence D (future v0.18+): finish the consistency sweep across all `selected_index` alias sites**. Committed as follow-up; explicitly not in this cadence's scope.

## Cadence structure

Standard 4-PR pattern:

1. Spec PR (this)
2. Plan PR
3. Impl PR — three signed commits (Unit 1 / Unit 2 / Unit 3 = CHANGELOG + MIGRATION.md)
4. Tracking-doc PR — check in `docs/audits/2026-07-05-post-consistency-cleanup.md` verification record; mark audit findings #6 and #8 CLOSED

No re-audit gate this time. The audit changes needed to close the findings are mechanical and the plan's verification gauntlet is sufficient. A re-audit is dispatched from the main session only if a genuine question emerges from the impl review.

## Unit 1 — Consistency sweep on `selected_item()`

Files touched: `src/component/dropdown/mod.rs`, `src/component/select/mod.rs`, `src/component/heatmap/mod.rs`, `src/component/tab_bar/mod.rs`, `src/component/data_grid/state.rs`, `src/component/table/state.rs`, plus their `tests.rs`/`tests/` files, plus any test in `tests/` or example in `examples/` that references a deleted accessor.

### Internal call sites requiring migration alongside the alias deletions

Per adversarial review A1, several components call their own soon-to-be-deleted aliases from inside their `view()` bodies. These must be migrated in the SAME commit that deletes the aliases, otherwise the crate fails to compile:

- `src/component/select/mod.rs:526,547` — `state.selected_value()` calls inside `view()`
- `src/component/dropdown/mod.rs:693,722,729` — `state.selected_value()` calls inside `view()`
- Any equivalent internal `state.selected()` / `state.selected_row()` / `state.active_tab()` calls inside data_grid / table / tab_bar `view()` — enumerate at impl time via `grep -rn '\.selected_value\|\.active_tab\|\.selected_row\b\|\.selected\b' src/component/` (post the alias deletions).

### Tautology tests to DELETE (not rename)

Per adversarial review A2, some tests exist specifically to verify that a literal alias returns the same value as its canonical counterpart:

- `src/component/data_grid/tests.rs:87` — `assert_eq!(state.selected_item(), state.selected_row())` (verifies alias) → **delete** after `selected_row` alias is gone
- `src/component/select/tests.rs:423` — similar shape for `selected_value` / `selected_item` → **delete**
- `src/component/dropdown/tests.rs:770` — similar shape → **delete**
- Any tests that specifically exist to prove alias-equivalence for `tab_bar::selected()` vs `selected_index()`, `data_grid::selected()` vs `selected_index()`, `table::selected()` vs `selected_index()`, `table::selected_row()` vs `selected_item()` → **delete**

Once the aliases are gone, these tests become tautologies (`assert_eq!(x, x)`) or dangling. Deletion is the correct action; renaming to the new accessor would create a genuinely tautological test.

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
| `selected_value() -> Option<f64>` (returns the DATA VALUE at the cursor coordinates — not "the item that is selected") | `value_at_selection() -> Option<f64>` (same body; renamed per adversarial review S2 — `selected_cell_value()` still sorts under `selected_*` in IDE autocomplete, defeating the purpose of the disambiguation) |

Rationale: heatmap already has `selected(&self) -> Option<(usize, usize)>` returning the cursor coordinates. `value_at_selection()` reads naturally as "the value AT the selection (which is a coordinate pair)" — the noun `value` sorts distinctly from `selected_*` in autocomplete, and the phrase makes the relationship between the two accessors explicit. Adversarial review's alternative `selected_cell_value()` was rejected because the `selected_` prefix would still group it visually with the collection-selection accessors this cadence is trying to disentangle.

Follow-up docstring update in the same commit: emphasize that this is the value under the cursor, and cross-link `selected()` as the coordinate-pair companion.

**tab_bar** (`src/component/tab_bar/mod.rs:297,322,336,351`):

| Before | After |
|---|---|
| `selected_index() -> Option<usize>` | *unchanged* |
| `selected() -> Option<usize>` — literal alias for `selected_index()` | *deleted* |
| `active_tab() -> Option<&Tab>` | `selected_item() -> Option<&Tab>` |
| `active_tab_mut() -> Option<&mut Tab>` | `selected_item_mut() -> Option<&mut Tab>` |

Rationale: `active_tab` is a semantic outlier — every other component with "which one is picked?" uses `selected_item()`. Rename is a one-word substitution across the accessor bodies + docstrings.

Tradeoff acknowledgment (adversarial review S3): `active_tab()` telegraphs the concrete return type (`Tab`) in the name; a downstream consumer typing `state.` in an IDE would recognize it faster than `selected_item()`. Consistency wins here — 15+ components use `selected_item()` and learning one pattern beats memorizing per-component names — but the discoverability tradeoff is real and worth acknowledging. Not a reason to change the design.

**data_grid** (`src/component/data_grid/state.rs:119,143,167,191`):

| Before | After |
|---|---|
| `selected_index() -> Option<usize>` | *unchanged* |
| `selected() -> Option<usize>` — literal alias for `selected_index()` | *deleted* |
| `selected_row() -> Option<&T>` — literal alias for `selected_item()` | *deleted* |
| `selected_item() -> Option<&T>` | *unchanged* |

Also unchanged: `set_selected(Option<usize>)` mutator; `selected_column() -> usize` (different concept — column cursor position, not a selection alias).

Struct field `selected_row: Option<usize>` on `DataGridState` (~15 internal references per adversarial review M1) is renamed to `selected_row_index: Option<usize>` in the same commit — retaining the internal semantic while breaking the grep-collision with the deleted method name. Field is private (`selected_row` not `pub selected_row`), so the rename is a search-and-replace inside `data_grid/state.rs` + `data_grid/mod.rs` with zero downstream impact.

**table** (`src/component/table/state.rs:246,270,295,323`) — **NEW: added per adversarial review M3:**

Table exposes byte-identical divergence to data_grid:

| Before | After |
|---|---|
| `selected_index() -> Option<usize>` | *unchanged* |
| `selected() -> Option<usize>` — literal alias for `selected_index()` | *deleted* |
| `selected_row() -> Option<&T>` — literal alias for `selected_item()` | *deleted* |
| `selected_item() -> Option<&T>` | *unchanged* |

Also unchanged: `set_selected(Option<usize>)` mutator at `:490`.

Rationale: leaving Table with `selected()` + `selected_row()` while removing them from data_grid would make the "consistency sweep" framing dishonest. A consumer reading MIGRATION.md and seeing data_grid's `selected_row()` drop would reasonably assume Table's did too, then be surprised. Fixing both together is the honest scope.

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
| `heatmap` | `state.selected_value() -> Option<f64>` (returns data value at cursor) | `state.value_at_selection() -> Option<f64>` (renamed — this was never a "selected item" accessor; the noun `value` sorts distinctly from `selected_*` in IDE autocomplete) |
| `tab_bar` | `state.active_tab() -> Option<&Tab>` | `state.selected_item() -> Option<&Tab>` |
| `tab_bar` | `state.active_tab_mut() -> Option<&mut Tab>` | `state.selected_item_mut() -> Option<&mut Tab>` |
| `tab_bar` | `state.selected()` (was: literal alias for `selected_index()`) | `state.selected_index()` |
| `data_grid` | `state.selected()` (was: literal alias for `selected_index()`) | `state.selected_index()` |
| `data_grid` | `state.selected_row()` (was: literal alias for `selected_item()`) | `state.selected_item()` |
| `table` | `state.selected()` (was: literal alias for `selected_index()`) | `state.selected_index()` |
| `table` | `state.selected_row()` (was: literal alias for `selected_item()`) | `state.selected_item()` |
```

The `heatmap` row updates to reference the actually-chosen destination name (`value_at_selection()`) not the earlier draft (`selected_cell_value()`).

## Unit 2 — `MessageSender<A>` newtype + Position cosmetic

Files touched: new `src/harness/message_sender.rs`, `src/harness/mod.rs`, `src/harness/app_harness/mod.rs`, `src/app/runtime/virtual_terminal.rs`.

### `MessageSender<M>` design (parameterized on message type, per adversarial review S1)

**Parameter choice:** `MessageSender<M: Send + 'static>` — parameterized on the message type, NOT `<A: App>`. Adversarial review S1 correctly noted that `<A>` propagates an envision-specific `App: App` bound onto every downstream helper function that touches a sender. `<M: Send + 'static>` uses only `std` bounds, so consumers can write portable helper functions like `fn spawn_watcher<M: Send + 'static>(sender: MessageSender<M>) { ... }` without depending on envision's trait system.

New file `src/harness/message_sender.rs` (~120 lines with expanded API surface per adversarial review M4):

```rust
//! `MessageSender<M>` — first-party wrapper around the async message channel
//! that carries messages into an [`AppHarness`](crate::harness::AppHarness).

use tokio::sync::mpsc;

/// Hands the caller a way to inject messages into the AppHarness's Runtime
/// asynchronously — from subscription callbacks, spawned tasks, or any other
/// non-App-loop code path.
///
/// Wraps `tokio::sync::mpsc::Sender<M>` so envision consumers don't need
/// `tokio` as a direct dependency to use `AppHarness::message_sender()`.
/// The Sender's semantics are preserved (bounded, cloneable, send returns
/// `Result` on receiver-dropped) and its full API surface is available —
/// send/try_send with first-party error types, plus non-mutating queries
/// (`is_closed`, `capacity`, `max_capacity`) as passthroughs. Consumers
/// needing tokio-specific functionality beyond what's exposed can call
/// [`into_inner`](Self::into_inner) as an explicit escape hatch.
pub struct MessageSender<M> {
    inner: mpsc::Sender<M>,
}

impl<M> MessageSender<M> {
    pub(crate) fn new(inner: mpsc::Sender<M>) -> Self {
        Self { inner }
    }

    /// Sends a message into the AppHarness. Returns an error only when the
    /// AppHarness (and hence the receiver) has been dropped.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use envision::prelude::*;
    /// # async fn example<M: Send + 'static>(sender: MessageSender<M>, msg: M) {
    /// sender.send(msg).await.expect("harness still alive");
    /// # }
    /// ```
    pub async fn send(&self, msg: M) -> Result<(), MessageSendError<M>> {
        self.inner.send(msg).await.map_err(|e| MessageSendError(e.0))
    }

    /// Attempts to send a message without waiting. Returns an error if the
    /// channel is full or the AppHarness has been dropped.
    pub fn try_send(&self, msg: M) -> Result<(), TrySendError<M>> {
        self.inner.try_send(msg).map_err(TrySendError::from_tokio)
    }

    /// Returns `true` if the receiver end of the channel has been dropped.
    ///
    /// Useful for `spawn_watcher`-style loops that want to exit before
    /// wasting work on messages that would fail to send.
    pub fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }

    /// Returns the current available capacity of the channel — the number
    /// of messages that can be enqueued without blocking or hitting a
    /// `TrySendError::Full`.
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Returns the maximum capacity of the channel (the bound configured
    /// at AppHarness construction).
    pub fn max_capacity(&self) -> usize {
        self.inner.max_capacity()
    }

    /// Explicit escape hatch: consumes the wrapper and returns the underlying
    /// `tokio::sync::mpsc::Sender<M>` for consumers who need tokio-specific
    /// functionality (`reserve`, `send_timeout`, `same_channel`, `downgrade`,
    /// or a `closed()` Future) that this wrapper deliberately doesn't
    /// expose to keep the default surface minimal.
    ///
    /// Using this method re-couples your code to the tokio dep; it's an
    /// escape hatch by design, not a routine call.
    pub fn into_inner(self) -> mpsc::Sender<M> {
        self.inner
    }
}

impl<M> Clone for MessageSender<M> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

impl<M> std::fmt::Debug for MessageSender<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageSender").finish_non_exhaustive()
    }
}

// Ensure MessageSender<M> is Send + Sync when M is (mirrors tokio's Sender).
// Adversarial review A4: downstream consumers spawn the sender into tokio
// tasks, so Send + Sync + 'static must be preserved.
// Verified at compile time by the unit tests in this module.

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
pub fn message_sender(&self) -> MessageSender<A::Message> {
    MessageSender::new(self.runtime.message_sender())
}
```

Consumer code changes from `sender.send(msg).await.unwrap()` to `sender.send(msg).await.unwrap()` — the call shape is identical, only the type flowing out of `message_sender()` is different. Downstream consumers who spelled the type explicitly must update `tokio::sync::mpsc::Sender<Msg>` → `MessageSender<Msg>` (parameterized on the message type per adversarial review S1, not on the App type).

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
### `MessageSender<M>` wraps `tokio::sync::mpsc::Sender<M>`

`AppHarness::message_sender()` now returns a first-party `MessageSender<M>`
newtype instead of raw `tokio::sync::mpsc::Sender<M>`. Consumer code
changes are limited to type spellings; call sites are identical.

| Old | New |
|---|---|
| `let sender: tokio::sync::mpsc::Sender<MyMsg> = harness.message_sender();` | `let sender: envision::MessageSender<MyMsg> = harness.message_sender();` (or use `envision::prelude::MessageSender`) |
| `sender.send(msg).await` — returns `Result<(), tokio::sync::mpsc::error::SendError<MyMsg>>` | `sender.send(msg).await` — returns `Result<(), envision::MessageSendError<MyMsg>>` |
| `sender.try_send(msg)` — returns `Result<(), tokio::sync::mpsc::error::TrySendError<MyMsg>>` | `sender.try_send(msg)` — returns `Result<(), envision::TrySendError<MyMsg>>` |
| `sender.is_closed()` — still available | `sender.is_closed()` — still available (passthrough) |
| `sender.capacity()` — still available | `sender.capacity()` — still available (passthrough) |
| `sender.max_capacity()` — still available | `sender.max_capacity()` — still available (passthrough) |
| `sender.reserve()`, `.send_timeout()`, `.same_channel()`, `.downgrade()`, `.closed()` | `let tokio_sender = sender.into_inner();` — explicit escape hatch that re-couples to tokio |

`MessageSender<M>` is `Clone + Debug + Send + Sync` (when `M: Send`). Its
send/try_send methods have the same semantics as tokio's Sender (bounded
channel, receiver-dropped errors carry the message back).
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

- Every unit test that references a deleted accessor migrates to the new name (mechanical rename), EXCEPT the alias-equivalence sanity tests enumerated above under "Tautology tests to DELETE" — those are deleted, not renamed.
- Every doc-test that references a deleted accessor updates.
- **Grep-verifiable success** (rewritten per adversarial review M1+M2 — token-boundary grep collides with unrelated fields and downstream example state):
  - `grep -rn '\.selected_value(' src/ tests/ examples/` returns hits ONLY in `CHANGELOG.md` / `MIGRATION.md` migration-table prose. Zero method-call hits.
  - `grep -rn '\.active_tab(' src/ tests/ examples/` returns hits ONLY in `CHANGELOG.md` / `MIGRATION.md` migration-table prose. Zero method-call hits.
  - `grep -rn '\.active_tab_mut(' src/ tests/ examples/` — same shape.
  - `grep -rn '\.selected_row(' src/ tests/ examples/` returns ONLY in migration-table prose. Zero method-call hits.
  - `grep -rn '\.selected()' src/ tests/ examples/` requires closer inspection — some legitimate uses may exist on components NOT in this cadence's scope (accordion, tabs, radio_group, etc. per adversarial review A3 — see Out of scope + Follow-up). Verify at impl time that all `.selected()` hits within the 6-component scope of Unit 1 (dropdown/select/heatmap/tab_bar/data_grid/table) are zero.
  - The `DataGridState`'s renamed field `selected_row_index: Option<usize>` intentionally shows up in `grep 'selected_row'` results but not in `grep '\.selected_row('` — the two forms are distinct.
- **`MessageSender<M>` unit tests** in the same style as EnvisionError tests: round-trip through send/try_send + error variants + `is_closed()`/`capacity()`/`max_capacity()` passthroughs + `into_inner()` escape hatch. Plus a compile-time `assert_impl_all!(MessageSender<u32>: Send + Sync + Clone)` gate (uses `static_assertions` crate — verify at impl time that it's already in dev-dependencies; if not, either add it or write an inline `fn _assert_send<T: Send>() {}` shim).
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

- **`MessageSender<M>` generic parameter is `M: Send + 'static`, not `A: App`.** Amended per adversarial review S1 — the earlier `<A: App>` draft would have propagated envision's `App` trait bound onto every downstream helper function that touches a sender. `<M: Send + 'static>` uses only `std` bounds, so consumers can write portable helper functions without depending on envision's trait system.
- **`tokio::sync::mpsc::error::TrySendError` has two variants in tokio's actual API (`Full`, `Closed`).** Verify at impl time — the spec's `TrySendError<T>` mirrors those two variants. If tokio's API has drifted since (unlikely at tokio 1.x), adjust.
- **Heatmap `value_at_selection()` rename readers can miss.** Mitigation: the docstring on the renamed method explicitly states "renamed from `selected_value()` in v0.17.0 — see MIGRATION.md." Prior draft used `selected_cell_value()` (adversarial review S2 pointed out this still sorted under `selected_*` in IDE autocomplete, defeating the disambiguation).
- **data_grid + table consumers who spelled `selected()` or `selected_row()` explicitly.** They see the compile error and the migration table names the exact rename. Same pattern as prior cadences.
- **Ripple effect through tests.** ~10-25 test sites per renamed accessor across the 6 components (added Table per adversarial review M3). Mechanical grep-and-migrate; verified via the callsite-form grep assertions above.
- **Split-vs-bundle regret risk (adversarial review S4).** 10 breaking renames in this cadence are literal-alias deletions (zero regret risk) except 3 semantic renames: `heatmap::selected_value → value_at_selection`, `tab_bar::active_tab → selected_item`, `tab_bar::active_tab_mut → selected_item_mut`. If any of those three semantic renames turn out wrong post-release, we'd have to re-break in v0.18. Judgment call: ship as-spec'd because (a) `value_at_selection` addresses S2 more cleanly than the earlier draft, (b) `selected_item` for tab_bar is the framework-consistency win — but log this as the actual regret-risk surface if a re-review of the pattern is ever needed.
- **`MessageSender<M>` uses `crate::app::App` bound — NOT applicable anymore.** Amended per S1: no App bound propagation. Just `M: Send + 'static`.

## Open questions

None. Design decisions resolved through two review rounds:

**Brainstorm (2026-07-05):**
- Scope: findings #1 (consistency) + #2 (dep-leakage) bundled as one cadence
- Canonical accessor: `selected_item() -> Option<&T>` across the audited components
- tab_bar: rename `active_tab` → `selected_item`; drop `selected` alias
- data_grid: keep `selected_index` + `selected_item` + `set_selected` + `selected_column`; drop `selected` + `selected_row` aliases
- Dep-leakage real scope: only `MessageSender<M>` newtype worth adding + the Position cosmetic
- Deprecate vs delete: delete outright (pre-1.0 precedent)
- MIGRATION.md: append two new subsections to v0.16→v0.17
- CHANGELOG Known Deferred Findings: remove findings #6 and #8 (closed by this cadence)

**Adversarial user-persona review (2026-07-05):** four must-fix design bugs + five should-consider items + five additional angles, all folded into this spec via inline amendments. Concrete upgrades from that review:

- **M1+M2 grep gates rewritten** to callsite forms (`\.selected_row(` etc.) instead of token-boundary matches, avoiding collisions with unrelated fields (`data_grid.selected_row: Option<usize>` field ~15 refs) and downstream example state (`chat_client.rs` has its own `active_tab: usize` field).
- **M3: Table added to scope.** Table has byte-identical `selected() + selected_row()` alias pattern to data_grid — leaving it out would make the "consistency sweep" framing dishonest.
- **M4: `MessageSender<M>` API surface expanded** to include `is_closed()` + `capacity()` + `max_capacity()` passthroughs + `into_inner()` explicit escape hatch. Prior draft exposed only `send/try_send`, which would have been a regression for consumers using tokio Sender's is-closed loop pattern.
- **S1: Parameter switch** `MessageSender<A: App>` → `MessageSender<M: Send + 'static>` — avoids propagating envision-specific `App` trait bound onto downstream helper functions.
- **S2: Heatmap rename** `selected_cell_value` → `value_at_selection` — the `value_` prefix sorts distinctly from `selected_*` in IDE autocomplete, which is exactly what the disambiguation was supposed to achieve.
- **S3: tab_bar tradeoff acknowledgment** — spec now names the discoverability-vs-consistency tradeoff explicitly rather than asserting the rename is a simple wart.
- **A1: Internal component-body call sites enumerated** — `select/mod.rs:526,547`, `dropdown/mod.rs:693,722,729` need migration in the SAME commit as the alias deletions or the crate fails to compile.
- **A2: Tautology tests DELETED not renamed** — enumerated the ~4 alias-equivalence sanity tests that would become `assert_eq!(x, x)` after the aliases are gone.
- **A3: Cadence D committed as follow-up** — the `selected()` alias exists on 15+ other components; this cadence intentionally scopes to the 6 with the additional divergences the audit named; the remaining alias-only components are scheduled for Cadence D (v0.18+).
- **A4: Send + Sync + 'static** — `MessageSender<M>` inherits from tokio's Sender when `M: Send`; verified via `assert_impl_all!` compile-time gate in unit tests.
- **A5: Prelude gate** verified — `TestHarness` at `src/lib.rs:476` is unconditional (no `#[cfg(feature = "test-utils")]`), so `MessageSender` goes unconditional too.

## Reference

- **2026-07-05 audit** (in-session `/audit`): A- (3.81 GPA); findings #1 and #2 in the trust-eroders list; the release-hygiene work landed successfully but consistency + dep-leakage remained.
- **2026-07-04 pre-release-hygiene audit** (Fable): A- (3.62); called these out as deferred findings #6 (selected_value incoherence) and #8 (dep leakage in 8 signatures).
- **2026-07-05 post-release-hygiene audit** (Fable): A (3.91); confirmed findings #6 and #8 as documented deferrals.
- **Precedent for delete-outright pre-1.0:** D5 paragraph→line rename, D14 same, resource_gauge::new deletion, FileSortDirection deletion (release-readiness cadence). Pattern is: outright deletion + MIGRATION.md table + no `#[deprecated]` shim.
- **Cadence pattern:** brainstorm → spec PR → plan PR → impl PR → tracking-doc PR (established across 11+ prior cadences).
- **After Cadence A merges, Cadence B is the doc-hygiene split of CHANGELOG.md + MIGRATION.md** (finding #3 from the same audit). Separate brainstorm.
