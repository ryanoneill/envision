# App::init args redesign — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace `App::init() -> (State, Command<Msg>)` with `App::init(args: Self::Args) -> (State, Command<Msg>)`, gated by a sealed `OptionalArgs` marker for the `()` shortcut, with `.state()` deleted and a typestate-lite `RuntimeBuilder` → `ConfiguredRuntimeBuilder` split.

**Architecture:** Three logical phases. Phase 1 is purely additive (`OptionalArgs` marker module). Phase 2 is the atomic breaking-change commit — App trait gets `type Args` and new `init(args)` signature, builder is split, `.state()` is deleted, all 157 call sites migrate in lockstep. Phase 3 adds the six test categories from the spec, the trybuild compile-fail, and the CHANGELOG entry.

**Tech Stack:** Rust 2024, MSRV 1.85, ratatui 0.29, sealed-trait pattern, typestate-lite via builder split, `trybuild` for compile-fail tests, `insta` snapshots stay unchanged.

---

## Pre-Execution Gotchas

Read these before starting any task.

1. **Stable Rust constraint:** Associated-type defaults (`type Args = ();` in the trait declaration) are unstable on stable Rust ([rust-lang/rust#29661](https://github.com/rust-lang/rust/issues/29661)). The plan does **not** use associated-type defaults; every consumer writes `type Args = ();` explicitly.

2. **Atomicity:** Phase 2 (Tasks 7–22) is a single coherent breaking-change pass. The App-trait signature change cannot land in a commit that compiles unless every `impl App` is migrated in the same commit. **Do not split Phase 2 across commits where intermediate commits would not compile.** Phase 2 may produce one large commit (~5000 line diff across 100+ files) — this is expected and matches the precedent of G7's `0c9fd71`.

3. **`trybuild` dev-dep:** Task 26 adds a `trybuild` compile-fail test. If `trybuild` is not already in `[dev-dependencies]`, add it (`trybuild = "1"`). Verify with `cargo metadata --format-version 1 | jq '.packages[] | select(.name == "envision") | .dependencies[] | select(.name == "trybuild")'`.

4. **`cargo-nextest` filter syntax:** Tests run via `cargo nextest run`, not `cargo test`. Filter syntax: `cargo nextest run -p envision --lib -E 'test(/^app::model::/)'`. Doc tests still run via `cargo test --doc`.

5. **Signed commits required:** Every commit must be signed (`git commit -S`). If signing fails (`gpg-agent` issue), **do not bypass with `--no-gpg-sign` or `git config commit.gpgsign false`**. Stop and ask the user to resolve their gpg agent.

6. **Doctest count:** ~50 doctests in `src/app/` reference `fn init() -> ...`. They must all be migrated together in Phase 2. Use `grep -n "fn init() ->" src/app/**/*.rs` to find them.

7. **`Component::init()` is not in scope:** `src/component/*/mod.rs` files implement `Component::init() -> Self::State` (different trait). Do NOT migrate those — they have a single-arg signature unrelated to `App::init`. Filter migrations to files whose `init()` returns `(_, Command<_>)`.

8. **84 examples migrate identically:** Every `examples/*.rs` has the same App-impl shape. The migration is a regex find-and-replace per file, but verify each compiles after the global change.

9. **The audit tool (`tools/audit/`) currently scans for `fn init()` patterns** — verify after Phase 2 that scorecard checks still pass.

10. **Plan PR is a sibling to spec PR #463** — branch is `app-init-args-plan`, opens against `main`, parallel to the spec. Implementation is a separate branch (`app-init-args-impl` or similar) that depends on the spec PR being merged first.

---

## File Structure

### Created in this plan

| File | Responsibility |
|---|---|
| `src/app/model/optional_args.rs` | Sealed `Sealed` supertrait (private) + public `OptionalArgs` marker. Implemented only for `()`. |
| `tests/trybuild_app_args/missing_with_args.rs` | trybuild compile-fail fixture: `App` with non-`()` Args, `terminal_builder()?.build()?` chain, asserts the friendly compile error. |
| `tests/trybuild_app_args/missing_with_args.stderr` | trybuild stderr fixture. |
| `tests/trybuild_compile_fail.rs` | trybuild harness entry. |

### Modified in this plan

| File | Changes |
|---|---|
| `src/app/model/mod.rs` | Add `type Args` (no default); change `init` to `init(args: Self::Args)`; remove panicking default; doctest migration. |
| `src/app/model/tests.rs` | All 3 test `App` impls add `type Args = ()` and migrate `init()` → `init(_: ())`. |
| `src/app/runtime/builder.rs` | Add `ConfiguredRuntimeBuilder` struct; split `build()` impls; add `with_args`; delete `state()`; mirror config-shaping methods on both structs; doctest migration; test `App` impl migration. |
| `src/app/runtime/mod.rs` | Doctest migration; remove references to `.state()` in module docs. |
| `src/app/runtime/terminal.rs` | Doctest migration. |
| `src/app/runtime/virtual_terminal.rs` | Doctest migration. |
| `src/app/runtime/tests/mod.rs` | All 6 test `App` impls migrate. |
| `src/app/runtime/tests/async_tests.rs` | All 3 test `App` impls migrate. |
| `src/app/runtime_core/tests.rs` | 1 test `App` impl migrates. |
| `src/app/mod.rs` | Doctest migration; re-export `OptionalArgs`. |
| `src/harness/app_harness/mod.rs` | Doctest migration if any; helper test `App` impls migrate. |
| `src/harness/app_harness/tests.rs` | Test `App` impls migrate. |
| `src/lib.rs` | Doctest migration; consider re-exporting `OptionalArgs` at crate root. |
| `tests/integration.rs`, `tests/integration_async.rs`, `tests/integration_with_state.rs` | Test `App` impls migrate. |
| `benches/runtime.rs` | Bench `App` impl migrates. |
| `examples/*.rs` | All 84 example `App` impls migrate. |
| `Cargo.toml` | Add `trybuild` to `[dev-dependencies]` if absent. |
| `CHANGELOG.md` | Breaking-change entry + migration table. |

---

## Phase 0 — Pre-flight

### Task 1: Verify branch state and tooling

**Files:**
- Read: `Cargo.toml` (verify `trybuild` presence)
- Read: `docs/superpowers/specs/2026-05-02-app-init-args-design.md` (the spec)

- [ ] **Step 1: Confirm working branch**

```bash
git branch --show-current
```
Expected: `app-init-args-impl` (or whatever name was chosen). If on `main` or another branch, stop and create the implementation branch from `main`.

- [ ] **Step 2: Verify spec is merged or readable**

```bash
test -f docs/superpowers/specs/2026-05-02-app-init-args-design.md && echo "spec present"
```
Expected: `spec present`. If not, the spec PR (#463) hasn't merged yet — stop.

- [ ] **Step 3: Check trybuild dev-dependency**

```bash
grep -A1 '^\[dev-dependencies\]' Cargo.toml | head -20 | grep trybuild || echo "MISSING"
```
If `MISSING`, add `trybuild = "1"` to `[dev-dependencies]` in `Cargo.toml` and run `cargo check --tests` to download it.

- [ ] **Step 4: Verify gpg-agent works**

```bash
echo "test" | gpg --clearsign > /dev/null && echo "gpg ok"
```
Expected: `gpg ok`. If signing fails, stop and ask the user to resolve.

- [ ] **Step 5: Pin baseline test count**

```bash
cargo nextest run -p envision --lib 2>&1 | tail -5
cargo test --doc 2>&1 | tail -3
```
Record the pre-change test counts in a scratch note. After Phase 3, verify the new counts match expectations (existing tests still pass, new tests added).

---

## Phase 1 — Additive: `OptionalArgs` sealed marker

This phase is purely additive — adds new code without changing any existing signatures. After Phase 1 the code still compiles unchanged.

### Task 2: Add OptionalArgs module skeleton

**Files:**
- Create: `src/app/model/optional_args.rs`
- Modify: `src/app/model/mod.rs:1-10` (module declaration)

- [ ] **Step 1: Write the failing unit test**

Create the new file with only the test, expecting the marker not to exist yet:

```rust
// src/app/model/optional_args.rs

//! Sealed marker trait gating the no-args shortcut on `RuntimeBuilder::build()`.
//!
//! `OptionalArgs` is implemented only for `()`. When `App::Args = ()`, consumers
//! can call `.build()` without `.with_args(...)`. For any other Args type, the
//! compiler requires `.with_args` to be called.

mod sealed {
    /// Internal supertrait — not part of envision's public surface.
    pub trait Sealed {
        fn default_optional_args() -> Self;
    }

    impl Sealed for () {
        fn default_optional_args() -> Self {}
    }
}

/// Marker trait gating the `Args = ()` shortcut on `RuntimeBuilder::build()`.
///
/// Sealed: consumers cannot extend it. Only `()` implements `OptionalArgs`.
/// Any consumer who wants the no-`with_args` shortcut on `RuntimeBuilder::build()`
/// must declare `type Args = ();` on their `App` impl.
pub trait OptionalArgs: sealed::Sealed {}

impl OptionalArgs for () {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_implements_optional_args() {
        fn assert_impl<T: OptionalArgs>() {}
        assert_impl::<()>();
    }

    #[test]
    fn unit_default_args_is_unit() {
        let _: () = sealed::Sealed::default_optional_args();
    }
}
```

Wire the module into `src/app/model/mod.rs`:

```rust
// Add near the top of src/app/model/mod.rs (after `use` statements):
mod optional_args;
pub use optional_args::OptionalArgs;
```

- [ ] **Step 2: Run tests to verify they pass**

```bash
cargo nextest run -p envision --lib -E 'test(/optional_args/)'
```
Expected: 2 passed.

- [ ] **Step 3: Run clippy and check for warnings**

```bash
cargo clippy --all-features -- -D warnings 2>&1 | tail -10
```
Expected: clean.

- [ ] **Step 4: Commit**

```bash
git add src/app/model/optional_args.rs src/app/model/mod.rs
git commit -S -m "Add OptionalArgs sealed marker (additive, Phase 1)

Sealed marker trait that gates the no-args shortcut on RuntimeBuilder::build().
Implemented only for (). Consumers cannot extend.

Tracks D1 spec, Phase 1 of three.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

### Task 3: Re-export OptionalArgs at crate root

**Files:**
- Modify: `src/lib.rs` (find prelude or top-level re-exports)
- Modify: `src/app/mod.rs` (re-export from app module)

- [ ] **Step 1: Add re-export to `src/app/mod.rs`**

Find the existing `pub use` block for app types (look for `pub use model::App;` or similar). Add:

```rust
pub use model::OptionalArgs;
```

- [ ] **Step 2: Add re-export to `src/lib.rs` prelude (if prelude exists)**

Find `pub mod prelude` in `src/lib.rs`. If `App` is re-exported there, add `OptionalArgs` adjacent:

```rust
pub mod prelude {
    pub use crate::app::{App, OptionalArgs, Command, Runtime};
    // ... existing re-exports unchanged ...
}
```

- [ ] **Step 3: Verify it compiles**

```bash
cargo check --all-features 2>&1 | tail -3
```
Expected: clean.

- [ ] **Step 4: Verify the type appears in docs**

```bash
cargo doc --no-deps --all-features 2>&1 | tail -3 && \
  grep -l "OptionalArgs" target/doc/envision/index.html || echo "ok-html-may-vary"
```
Expected: clean build. (HTML grep is a sanity check; not required for green.)

- [ ] **Step 5: Commit**

```bash
git add src/app/mod.rs src/lib.rs
git commit -S -m "Re-export OptionalArgs from app and prelude (Phase 1)

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Phase 2 — Atomic switch: trait change + builder split + 157-site migration

This is the breaking-change pass. Every task in this phase modifies code in ways that, individually, would break compilation. Tasks 4–25 must all be staged before any commit lands; the final commit in Task 26 is large (~5000 line diff). **Treat Tasks 4–25 as preparation for one atomic commit.**

If working with subagent-driven development, dispatch a single subagent for the entire Phase 2 (not one per task). The atomicity is the point. Each task here is a checkpoint to mark progress, not a commit boundary.

### Task 4: Update App trait — add `type Args`, change `init` signature

**Files:**
- Modify: `src/app/model/mod.rs:180-214` (App trait body)

- [ ] **Step 1: Edit the App trait**

Replace lines 180–212 (the trait declaration through the panicking `init` default impl):

```rust
pub trait App: Sized {
    /// The application state type.
    ///
    /// This should contain all data needed to render the UI.
    /// Deriving `Clone` is recommended but not required.
    type State;

    /// The message type representing all possible events.
    ///
    /// This should be an enum covering all ways the state can change.
    type Message: Send + 'static;

    /// Configuration / dependencies handed in at construction time.
    ///
    /// Apps that need no injected config declare `type Args = ();`.
    /// Common uses: CLI-parsed paths, env-derived URLs, opened DB handles,
    /// preloaded fixture data for tests.
    ///
    /// `Args` is consumed (move semantics) by `init`. Apps that need to
    /// keep args around store the relevant fields in `State` during init.
    type Args;

    /// Initializes the application state from the provided args.
    ///
    /// Called exactly once per `Runtime` construction by the builder.
    /// Args are consumed (move semantics).
    fn init(args: Self::Args) -> (Self::State, Command<Self::Message>);

    // ... update / view / handle_event / handle_event_with_state /
    //     on_exit / should_quit / on_tick — unchanged ...
```

The `update`, `view`, `handle_event`, `handle_event_with_state`, `on_exit`, `should_quit`, and `on_tick` methods stay exactly as they were.

- [ ] **Step 2: Update the module-level doctests at `src/app/model/mod.rs:14-65`**

Replace the two doctest snippets in the module documentation:

```rust
//! ## Standard pattern — `init()` creates the state
//!
//! Use [`Runtime::terminal_builder()`] or [`Runtime::virtual_builder()`].
//! These call [`App::init()`] internally to create the initial state and
//! any startup commands.
//!
//! ```rust
//! # use envision::prelude::*;
//! # struct MyApp;
//! # #[derive(Default, Clone)]
//! # struct MyState;
//! # #[derive(Clone)]
//! # enum MyMsg {}
//! # impl App for MyApp {
//! #     type State = MyState;
//! #     type Message = MyMsg;
//! #     type Args = ();
//! #     fn init(_: ()) -> (MyState, Command<MyMsg>) { (MyState, Command::none()) }
//! #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
//! #     fn view(state: &MyState, frame: &mut Frame) {}
//! # }
//! let mut vt = Runtime::<MyApp, _>::virtual_builder(80, 24).build()?;
//! # Ok::<(), envision::EnvisionError>(())
//! ```
```

Delete the entire "External state pattern — `init()` is bypassed" section (lines ~32-65). Replace it with:

```rust
//! ## Args pattern — passing dependencies into `init`
//!
//! Apps that need injected config declare a non-`()` `Args` type and pass
//! values via [`RuntimeBuilder::with_args`]:
//!
//! ```rust
//! # use envision::prelude::*;
//! # use std::path::PathBuf;
//! # struct MyApp;
//! # #[derive(Clone)]
//! # struct MyArgs { dir: PathBuf }
//! # #[derive(Default, Clone)]
//! # struct MyState { dir: PathBuf }
//! # #[derive(Clone)]
//! # enum MyMsg {}
//! # impl App for MyApp {
//! #     type State = MyState;
//! #     type Message = MyMsg;
//! #     type Args = MyArgs;
//! #     fn init(args: MyArgs) -> (MyState, Command<MyMsg>) {
//! #         (MyState { dir: args.dir }, Command::none())
//! #     }
//! #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
//! #     fn view(state: &MyState, frame: &mut Frame) {}
//! # }
//! let args = MyArgs { dir: PathBuf::from("/tmp/example") };
//! let mut vt = Runtime::<MyApp, _>::virtual_builder(80, 24)
//!     .with_args(args)
//!     .build()?;
//! # Ok::<(), envision::EnvisionError>(())
//! ```
```

- [ ] **Step 3: Update the trait-level docstring at `src/app/model/mod.rs:100-179`**

Replace the "External state pattern" example (lines ~144-179) with the args pattern shown above (adapted to the trait-level format).

- [ ] **Step 4: Don't compile yet — this is preparation**

Tasks 5+ continue the atomic switch. Compilation will fail until they're all done.

---

### Task 5: Update RuntimeBuilder struct — remove `state` field, prep for split

**Files:**
- Modify: `src/app/runtime/builder.rs:157-175` (struct + constructor)

- [ ] **Step 1: Edit the RuntimeBuilder struct**

Replace lines 157–175:

```rust
/// A builder for constructing [`Runtime`] instances.
///
/// Created via [`Runtime::builder()`], [`Runtime::terminal_builder()`],
/// or [`Runtime::virtual_builder()`].
///
/// The builder provides fluent methods to configure:
/// - **Args**: `.with_args(args)` to provide the args passed to `App::init`
/// - **Config**: `.config(config)` to supply a full [`RuntimeConfig`]
/// - **Individual settings**: `.tick_rate()`, `.frame_rate()`, etc.
///
/// `with_args(args)` returns a [`ConfiguredRuntimeBuilder`] — see that type
/// for the post-args build path. Calling `.build()` directly on
/// `RuntimeBuilder` is only available when `A::Args: OptionalArgs`
/// (i.e. `A::Args = ()`).
pub struct RuntimeBuilder<A: App, B: Backend> {
    backend: B,
    config: Option<RuntimeConfig>,
    _phantom: PhantomData<A>,
}

impl<A: App, B: Backend> RuntimeBuilder<A, B> {
    /// Creates a new builder with the given backend.
    pub(crate) fn new(backend: B) -> Self {
        Self {
            backend,
            config: None,
            _phantom: PhantomData,
        }
    }
```

Add `use std::marker::PhantomData;` to the imports if not already present.

The `state` field is gone. The `_phantom: PhantomData<A>` carries the `A` parameter through (since it no longer appears in any field).

---

### Task 6: Add ConfiguredRuntimeBuilder struct

**Files:**
- Modify: `src/app/runtime/builder.rs` (add new struct after `RuntimeBuilder`)

- [ ] **Step 1: Add ConfiguredRuntimeBuilder definition**

Insert immediately after the `RuntimeBuilder` struct (around line 165 after Task 5 edits):

```rust
/// A `RuntimeBuilder` after `with_args` has been called.
///
/// Returned by [`RuntimeBuilder::with_args`]. Carries the args that will be
/// passed to `App::init` plus all configuration set so far. Has its own
/// fluent config-shaping methods (`config`, `tick_rate`, `frame_rate`,
/// `max_messages`, `channel_capacity`) and an unconditionally-available
/// `build()`.
///
/// Most users never name this type — the typestate transition happens
/// implicitly when chaining `.with_args(...).build()`.
pub struct ConfiguredRuntimeBuilder<A: App, B: Backend> {
    backend: B,
    config: Option<RuntimeConfig>,
    args: A::Args,
}
```

---

### Task 7: Add `with_args` to RuntimeBuilder

**Files:**
- Modify: `src/app/runtime/builder.rs` (add method to existing impl block on `RuntimeBuilder`)

- [ ] **Step 1: Add the with_args method**

Inside `impl<A: App, B: Backend> RuntimeBuilder<A, B>`, add after the `new` method:

```rust
/// Provides the args for `App::init`.
///
/// Consumes the `RuntimeBuilder` and produces a [`ConfiguredRuntimeBuilder`]
/// whose `build()` is unconditionally available. Any prior config-shaping
/// calls (`tick_rate`, `frame_rate`, etc.) are preserved.
///
/// # Example
///
/// ```rust
/// # use envision::prelude::*;
/// # struct MyApp;
/// # #[derive(Default, Clone)]
/// # struct MyState;
/// # #[derive(Clone)]
/// # enum MyMsg {}
/// # impl App for MyApp {
/// #     type State = MyState;
/// #     type Message = MyMsg;
/// #     type Args = ();
/// #     fn init(_: ()) -> (MyState, Command<MyMsg>) { (MyState, Command::none()) }
/// #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
/// #     fn view(state: &MyState, frame: &mut Frame) {}
/// # }
/// let runtime = Runtime::<MyApp, _>::virtual_builder(80, 24)
///     .with_args(())
///     .build()?;
/// # Ok::<(), envision::EnvisionError>(())
/// ```
pub fn with_args(self, args: A::Args) -> ConfiguredRuntimeBuilder<A, B> {
    ConfiguredRuntimeBuilder {
        backend: self.backend,
        config: self.config,
        args,
    }
}
```

---

### Task 8: Delete the `.state()` method from RuntimeBuilder

**Files:**
- Modify: `src/app/runtime/builder.rs:177-212` (remove the `state` method and its docstring)

- [ ] **Step 1: Delete `pub fn state(...)` and its preceding `///` docstring**

Find the comment block starting `/// Provides a pre-built initial state, bypassing [\`App::init()\`].` and delete from that line through the closing `}` of the `state` method (around line 212). Net deletion of ~36 lines.

---

### Task 9: Mirror config-shaping methods onto ConfiguredRuntimeBuilder

**Files:**
- Modify: `src/app/runtime/builder.rs` (add impl block after ConfiguredRuntimeBuilder definition)

- [ ] **Step 1: Add the mirrored methods**

Insert the following impl block after the `ConfiguredRuntimeBuilder` struct definition:

```rust
impl<A: App, B: Backend> ConfiguredRuntimeBuilder<A, B> {
    /// Sets the full runtime configuration. Replaces any previously set config.
    pub fn config(mut self, config: RuntimeConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Sets the tick rate (how often to poll for events). Default: 50ms.
    pub fn tick_rate(mut self, rate: Duration) -> Self {
        self.config_mut().tick_rate = rate;
        self
    }

    /// Sets the frame rate (how often to render). Default: 16ms (~60fps).
    pub fn frame_rate(mut self, rate: Duration) -> Self {
        self.config_mut().frame_rate = rate;
        self
    }

    /// Sets the max messages per tick. Default: 100.
    pub fn max_messages(mut self, max: usize) -> Self {
        self.config_mut().max_messages_per_tick = max;
        self
    }

    /// Sets the async message channel capacity. Default: 256.
    pub fn channel_capacity(mut self, capacity: usize) -> Self {
        self.config_mut().message_channel_capacity = capacity;
        self
    }

    fn config_mut(&mut self) -> &mut RuntimeConfig {
        self.config.get_or_insert_with(RuntimeConfig::default)
    }
}
```

This duplicates the bodies of `RuntimeBuilder`'s config-shaping methods. The duplication is acceptable: the bodies are 1–3 lines each, and a shared helper trait would obscure the surface for marginal savings. If future readers find the duplication painful, extracting a `BuilderConfigCommon` private trait is straightforward — defer until then.

---

### Task 10: Add `RuntimeBuilder::build()` for the OptionalArgs path

**Files:**
- Modify: `src/app/runtime/builder.rs:329-369` (replace the existing `build` method)

- [ ] **Step 1: Replace the existing `build()` method**

Find the existing `pub fn build(self) -> error::Result<Runtime<A, B>>` method (currently around line 359). Delete the existing method body and the impl-block-closing brace. Replace the entire `impl<A: App, B: Backend> RuntimeBuilder<A, B> { ... }` block's `build()` portion and following helper with:

```rust
    /// Internal helper, available on both builder types.
    fn config_mut(&mut self) -> &mut RuntimeConfig {
        self.config.get_or_insert_with(RuntimeConfig::default)
    }
}

// `build()` for the no-args path — only available when A::Args: OptionalArgs.
//
// On stable Rust this means A::Args == (). Calling `.build()` for an App
// whose Args is anything other than () fails to compile here, which is the
// compile-time enforcement the redesign promises.
impl<A: App, B: Backend> RuntimeBuilder<A, B>
where
    A::Args: crate::app::OptionalArgs,
{
    /// Builds the [`Runtime`].
    ///
    /// Available only when `A::Args = ()`. For apps with non-`()` args,
    /// call `.with_args(...)` first and then `.build()` on the resulting
    /// [`ConfiguredRuntimeBuilder`].
    pub fn build(self) -> error::Result<Runtime<A, B>> {
        use crate::app::model::optional_args::sealed::Sealed;
        let args = <A::Args as Sealed>::default_optional_args();
        self.with_args(args).build()
    }
}
```

This requires `pub(crate)` on the `sealed` module so the `Sealed` trait is reachable. Adjust `src/app/model/optional_args.rs` if necessary:

```rust
// Was: mod sealed { ... }
// Make: pub(crate) mod sealed { ... }
//   And inside the module:
//   pub trait Sealed { fn default_optional_args() -> Self; }
//   (The trait itself stays pub within the module.)
```

---

### Task 11: Add `ConfiguredRuntimeBuilder::build()`

**Files:**
- Modify: `src/app/runtime/builder.rs` (extend the impl block from Task 9)

- [ ] **Step 1: Add `build()` to the ConfiguredRuntimeBuilder impl block**

Inside the `impl<A: App, B: Backend> ConfiguredRuntimeBuilder<A, B>` block (after `channel_capacity`), add:

```rust
/// Builds the [`Runtime`].
///
/// Calls `App::init(self.args)` to obtain the initial state and startup
/// command, then constructs the `Runtime` with the configured backend
/// and runtime config.
///
/// # Errors
///
/// Returns an error if creating the ratatui `Terminal` with the
/// provided backend fails.
pub fn build(self) -> error::Result<Runtime<A, B>> {
    let (state, init_cmd) = A::init(self.args);
    let config = self.config.unwrap_or_default();
    Runtime::with_backend_state_and_config(self.backend, state, init_cmd, config)
}
```

---

### Task 12: Migrate test App impls in `src/app/runtime/builder.rs::tests`

**Files:**
- Modify: `src/app/runtime/builder.rs:521-545` and the other test impls (~lines 770-820)

- [ ] **Step 1: Migrate `TestApp` impl**

```rust
impl App for TestApp {
    type State = TestState;
    type Message = TestMsg;
    type Args = ();

    fn init(_args: ()) -> (Self::State, Command<Self::Message>) {
        (TestState::default(), Command::none())
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
        match msg {
            TestMsg::Increment => state.count += 1,
            TestMsg::Quit => state.quit = true,
        }
        Command::none()
    }

    fn view(state: &Self::State, frame: &mut ratatui::Frame) {
        let text = format!("Count: {}", state.count);
        frame.render_widget(Paragraph::new(text), frame.area());
    }

    fn should_quit(state: &Self::State) -> bool {
        state.quit
    }
}
```

- [ ] **Step 2: Rewrite tests that called `.state(...)`**

Tests at `test_builder_with_state` (line ~558), `test_builder_with_state_and_config` (line ~586), `test_virtual_builder_with_state` (line ~613), `test_virtual_builder_state_and_config` (line ~683) currently call `.state(state, Command::none())` to inject a pre-built state. Each must be rewritten to use a separate App with `Args` carrying the count, OR converted to a non-state-based test of the new path.

Recommended: replace each with a test that exercises the `with_args` path. For example, replace `test_builder_with_state`:

```rust
struct ArgsApp;

impl App for ArgsApp {
    type State = TestState;
    type Message = TestMsg;
    type Args = TestState;
    fn init(args: TestState) -> (Self::State, Command<Self::Message>) {
        (args, Command::none())
    }
    fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
        if let TestMsg::Increment = msg { state.count += 1 }
        if let TestMsg::Quit = msg { state.quit = true }
        Command::none()
    }
    fn view(state: &Self::State, frame: &mut ratatui::Frame) {
        let text = format!("Count: {}", state.count);
        frame.render_widget(Paragraph::new(text), frame.area());
    }
    fn should_quit(state: &Self::State) -> bool { state.quit }
}

#[test]
fn test_builder_with_args() {
    let backend = CaptureBackend::new(80, 24);
    let state = TestState { count: 42, quit: false };
    let runtime = Runtime::<ArgsApp, _>::builder(backend)
        .with_args(state)
        .build()
        .unwrap();
    assert_eq!(runtime.state().count, 42);
}
```

Apply the same shape to:
- `test_builder_with_state_and_config` → `test_builder_with_args_and_config`
- `test_virtual_builder_with_state` → `test_virtual_builder_with_args`
- `test_virtual_builder_state_and_config` → `test_virtual_builder_args_and_config`

- [ ] **Step 3: Migrate the other two test apps in this file (lines ~772 and ~820)**

Each `impl App for SomeApp` gets `type Args = ();` and `fn init() ->` becomes `fn init(_args: ())`.

---

### Task 13: Migrate doctests in `src/app/runtime/builder.rs`

**Files:**
- Modify: `src/app/runtime/builder.rs` doctests at lines 29, 50, 72, 96, 147, 198, 233, 267, 298, 352, 396, 438, 481

- [ ] **Step 1: Find and update each doctest**

For each line listed, the surrounding doctest contains:

```rust
//! # impl App for MyApp {
//! #     type State = MyState;
//! #     type Message = MyMsg;
//! #     fn init() -> (MyState, Command<MyMsg>) { ... }
//! #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
//! #     fn view(state: &MyState, frame: &mut Frame) {}
//! # }
```

Change each to:

```rust
//! # impl App for MyApp {
//! #     type State = MyState;
//! #     type Message = MyMsg;
//! #     type Args = ();
//! #     fn init(_: ()) -> (MyState, Command<MyMsg>) { ... }
//! #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
//! #     fn view(state: &MyState, frame: &mut Frame) {}
//! # }
```

Also: the doctest at line ~198 (the example for the now-deleted `.state()` method) should be deleted entirely along with its surrounding docstring (Task 8 already handles this if the docstring is fully removed).

The doctest at line 72 (in the module docs) currently says `.state(state, Command::none())` — rewrite to `.with_args(())` plus a note on the args pattern.

---

### Task 14: Migrate doctests in `src/app/runtime/mod.rs`

**Files:**
- Modify: `src/app/runtime/mod.rs:25, 57, 146, 174, 253, 279, 347, 377, 471, 625, 782`

- [ ] **Step 1: Apply the same doctest substitution as Task 13**

Each `impl App for MyApp { ... fn init() -> ... }` block becomes the args version. The doctest at line ~253 / 279 / 471 references `.state()` for state-injection patterns — rewrite those to use `.with_args(...)`.

---

### Task 15: Migrate doctests in `src/app/runtime/terminal.rs` and `virtual_terminal.rs`

**Files:**
- Modify: `src/app/runtime/terminal.rs:88, 240`
- Modify: `src/app/runtime/virtual_terminal.rs:33, 62, 99, 125`

- [ ] **Step 1: Apply doctest substitution**

Same pattern as Task 13.

---

### Task 16: Migrate test App impls in `src/app/runtime/tests/mod.rs`

**Files:**
- Modify: `src/app/runtime/tests/mod.rs:31, 221, 356, 469, 809, 914`

- [ ] **Step 1: For each `fn init()` in the file, migrate the surrounding `impl App` block**

Each line listed is the start of a `fn init() -> ...`. Walk up to the enclosing `impl App` block and add `type Args = ();` after the existing `type Message = ...;` line. Change `fn init() -> (Self::State, ...)` to `fn init(_args: ()) -> (Self::State, ...)`.

If any test currently calls `.state(...)` on a builder, rewrite it to use `.with_args(...)` plus an `Args` type that carries the pre-built state. Apply the pattern from Task 12 Step 2.

---

### Task 17: Migrate test App impls in `src/app/runtime/tests/async_tests.rs`

**Files:**
- Modify: `src/app/runtime/tests/async_tests.rs:155, 345, 394`

- [ ] **Step 1: Same pattern as Task 16**

---

### Task 18: Migrate `src/app/runtime_core/tests.rs`

**Files:**
- Modify: `src/app/runtime_core/tests.rs:29`

- [ ] **Step 1: Add `type Args = ();` and migrate `fn init()` to `fn init(_args: ())`**

---

### Task 19: Migrate `src/app/model/tests.rs`

**Files:**
- Modify: `src/app/model/tests.rs:22, 134, 232`

- [ ] **Step 1: For each line, migrate the enclosing impl App block**

Same pattern. After migration, all 3 test App impls in this file have `type Args = ();` and `fn init(_args: ())`.

---

### Task 20: Migrate `src/app/mod.rs` and `src/lib.rs` doctests

**Files:**
- Modify: `src/app/mod.rs:83`
- Modify: `src/lib.rs` (find the App-impl doctest)

- [ ] **Step 1: Apply doctest substitution**

---

### Task 21: Migrate `src/harness/`

**Files:**
- Modify: `src/harness/app_harness/mod.rs` (any App impl)
- Modify: `src/harness/app_harness/tests.rs` (any App impl)

- [ ] **Step 1: Find App impls in harness files**

```bash
grep -n "impl App for\|fn init()" src/harness/app_harness/*.rs
```

Migrate each impl: add `type Args = ();`, change `fn init()` → `fn init(_args: ())`.

---

### Task 22: Migrate integration tests

**Files:**
- Modify: `tests/integration.rs`
- Modify: `tests/integration_async.rs`
- Modify: `tests/integration_with_state.rs`

- [ ] **Step 1: For each file, find and migrate every `impl App for X`**

If `integration_with_state.rs` exists and tests the now-deleted `.state()` API, it must be rewritten to use `with_args` or renamed/deleted entirely. Recommended: rename to `integration_with_args.rs` and rewrite tests to exercise the `with_args` path.

---

### Task 23: Migrate `benches/runtime.rs`

**Files:**
- Modify: `benches/runtime.rs`

- [ ] **Step 1: Migrate the bench App impl**

Add `type Args = ();`, change `fn init()` → `fn init(_args: ())`.

---

### Task 24: Migrate all examples

**Files:**
- Modify: 84 files in `examples/*.rs` (every example with an `impl App` block)

- [ ] **Step 1: Determine the file list**

```bash
grep -rln "fn init() -> (.*Command" examples/ | sort > /tmp/examples_to_migrate.txt
wc -l /tmp/examples_to_migrate.txt
```
Expected: ~84 files.

- [ ] **Step 2: For each file, apply the migration**

The pattern in every example is identical:

```rust
impl App for FooApp {
    type State = State;
    type Message = Msg;
    fn init() -> (State, Command<Msg>) {
        // ... body unchanged ...
    }
    // ... rest unchanged ...
}
```

Becomes:

```rust
impl App for FooApp {
    type State = State;
    type Message = Msg;
    type Args = ();
    fn init(_args: ()) -> (State, Command<Msg>) {
        // ... body unchanged ...
    }
    // ... rest unchanged ...
}
```

If the example uses `.state(...)` on a builder (rare), rewrite to `.with_args(...)` per Task 12 Step 2.

This task is mechanical but voluminous. If using subagent-driven execution, dispatch one subagent for the entire batch with the file list as input.

- [ ] **Step 3: Verify all examples build**

```bash
cargo build --examples --all-features 2>&1 | tail -10
```
Expected: all 84 examples build clean.

---

### Task 25: Update CHANGELOG

**Files:**
- Modify: `CHANGELOG.md`

- [ ] **Step 1: Add the breaking-change entry**

Add a new top-level section (before the existing top entry):

```markdown
## [Unreleased] — 2026-05-XX

### Breaking changes — App::init takes args

`App::init() -> (State, Command<Msg>)` is replaced with
`App::init(args: Self::Args) -> (State, Command<Msg>)`.

- `App` trait gains `type Args` (no default; explicit `type Args = ();` required for no-args apps).
- The panicking default impl of `init` is deleted; `init` is now required.
- `RuntimeBuilder::state(state, cmd)` is **deleted**. Its role is subsumed by `with_args` plus a real `init` impl.
- New: `RuntimeBuilder::with_args(args) -> ConfiguredRuntimeBuilder<A, B>` carries the args into a typestate-lite builder whose `build()` is unconditionally available.
- `RuntimeBuilder::build()` is now only available when `A::Args: OptionalArgs` (sealed marker, implemented only for `()`). Forgetting `with_args` for non-`()` Args is a compile error.

#### Migration

| Old | New |
|---|---|
| `fn init() -> (State, Command<Msg>)` | `type Args = (); fn init(_args: ()) -> (State, Command<Msg>)` |
| `static GLOBAL: OnceLock<T>; fn init() { GLOBAL.get()... }` | `type Args = MyArgs; fn init(args: MyArgs) { args.field... }` |
| `RuntimeBuilder::state(state, cmd)` | `RuntimeBuilder::with_args(args)`; move state-building into `init` |

Tracks leadline gap D1. See `docs/superpowers/specs/2026-05-02-app-init-args-design.md` and PR #461 (sort migration) for the precedent atomic-migration pattern.
```

---

### Task 26: Verify the atomic switch builds and tests pass

**Files:**
- (no edits — verification step)

- [ ] **Step 1: Full build**

```bash
cargo build --all-features 2>&1 | tail -10
```
Expected: clean.

- [ ] **Step 2: Run nextest**

```bash
cargo nextest run -p envision --lib 2>&1 | tail -10
```
Expected: all existing tests pass; counts within 5 of pre-change baseline.

- [ ] **Step 3: Run doctests**

```bash
cargo test --doc --all-features 2>&1 | tail -10
```
Expected: all doctests pass.

- [ ] **Step 4: Run clippy**

```bash
cargo clippy --all-features --tests --examples -- -D warnings 2>&1 | tail -10
```
Expected: clean.

- [ ] **Step 5: Format check**

```bash
cargo fmt --check 2>&1 | tail -5
```
Expected: no diff.

- [ ] **Step 6: Build all examples**

```bash
cargo build --examples --all-features 2>&1 | tail -10
```
Expected: all examples build.

- [ ] **Step 7: Commit the atomic switch**

This is the large commit. Stage every modified file from Tasks 4–25:

```bash
git add -A  # Verify with `git status` first that ONLY in-scope files are staged.
git status  # Visual confirmation
```

Visually confirm: only `src/app/`, `src/lib.rs`, `src/harness/`, `tests/integration*.rs`, `benches/runtime.rs`, `examples/*.rs`, `CHANGELOG.md`. No accidental files.

```bash
git commit -S -m "Switch App::init to take Self::Args; split RuntimeBuilder; delete .state()

Atomic breaking change. App trait gains \`type Args\` (no default — explicit
\`type Args = ();\` required on stable Rust). \`fn init() -> ...\` becomes
\`fn init(args: Self::Args) -> ...\`. RuntimeBuilder is split into
RuntimeBuilder + ConfiguredRuntimeBuilder; with_args promotes between them.
\`.state()\` deleted; its role subsumed by with_args + a real init impl.

Migrates 157 init call sites across 103 files (84 examples, 12 src,
3 integration tests, 1 bench, 2 harness, 1 lib.rs doc).

Tracks leadline gap D1. Spec: docs/superpowers/specs/2026-05-02-app-init-args-design.md

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Phase 3 — Add the six test categories from the spec

Each task in this phase is a separate small commit. Tests pin the new behavior against future regressions.

### Task 27: Test category #1 — `type Args = ()` works

**Files:**
- Modify: `src/app/model/tests.rs` (add new test)

- [ ] **Step 1: Add the test**

```rust
#[test]
fn test_args_unit_default_works_with_build() {
    use crate::app::Runtime;

    struct UnitArgsApp;
    #[derive(Clone, Default)]
    struct UnitState;
    #[derive(Clone)]
    enum UnitMsg {}

    impl App for UnitArgsApp {
        type State = UnitState;
        type Message = UnitMsg;
        type Args = ();

        fn init(_args: ()) -> (Self::State, Command<Self::Message>) {
            (UnitState, Command::none())
        }

        fn update(_: &mut Self::State, _: Self::Message) -> Command<Self::Message> {
            Command::none()
        }

        fn view(_: &Self::State, _: &mut ratatui::Frame) {}
    }

    let _ = Runtime::<UnitArgsApp, _>::virtual_builder(80, 24).build().unwrap();
    // No with_args needed — () is OptionalArgs.
}
```

- [ ] **Step 2: Run and commit**

```bash
cargo nextest run -p envision --lib -E 'test(test_args_unit_default_works_with_build)'
```
Expected: 1 passed.

```bash
git add src/app/model/tests.rs
git commit -S -m "Test: type Args = () works with .build() (no with_args needed)

Pins the no-args ergonomic shape — the OptionalArgs shortcut.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

### Task 28: Test category #2 — trybuild compile-fail when with_args missing

**Files:**
- Create: `tests/trybuild_compile_fail.rs`
- Create: `tests/trybuild_app_args/missing_with_args.rs`
- Create: `tests/trybuild_app_args/missing_with_args.stderr`

- [ ] **Step 1: Create the trybuild harness**

```rust
// tests/trybuild_compile_fail.rs

#[test]
fn compile_fail_app_args() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/trybuild_app_args/missing_with_args.rs");
}
```

- [ ] **Step 2: Create the failing fixture**

```rust
// tests/trybuild_app_args/missing_with_args.rs

use envision::prelude::*;
use std::path::PathBuf;

struct ArgsApp;
#[derive(Clone, Default)]
struct ArgsState;
#[derive(Clone)]
enum ArgsMsg {}

#[derive(Clone)]
struct MyArgs { _path: PathBuf }

impl App for ArgsApp {
    type State = ArgsState;
    type Message = ArgsMsg;
    type Args = MyArgs;

    fn init(_args: MyArgs) -> (ArgsState, Command<ArgsMsg>) {
        (ArgsState, Command::none())
    }
    fn update(_: &mut ArgsState, _: ArgsMsg) -> Command<ArgsMsg> {
        Command::none()
    }
    fn view(_: &ArgsState, _: &mut ratatui::Frame) {}
}

fn main() {
    // Should fail to compile: A::Args = MyArgs is not OptionalArgs,
    // so RuntimeBuilder::build() is not in scope.
    let _ = Runtime::<ArgsApp, _>::virtual_builder(80, 24).build().unwrap();
}
```

- [ ] **Step 3: Generate the .stderr fixture**

```bash
cargo test --test trybuild_compile_fail 2>&1 | tail -50
```

The first run will fail and write a `.stderr.new` file. Inspect it:

```bash
cat tests/trybuild_app_args/missing_with_args.stderr.new
```

Expected content (approximate; exact compiler text may vary by rustc version):

```text
error[E0599]: no method named `build` found for struct `RuntimeBuilder<ArgsApp, _>` in the current scope
  --> tests/trybuild_app_args/missing_with_args.rs:27:65
   |
27 |     let _ = Runtime::<ArgsApp, _>::virtual_builder(80, 24).build().unwrap();
   |                                                            ^^^^^ method not found in `RuntimeBuilder<ArgsApp, ...>`
   |
   = note: the method `build` exists for the struct `RuntimeBuilder<ArgsApp, _>`, but its trait bounds were not satisfied
   = note: the following trait bounds were not satisfied:
           `<ArgsApp as App>::Args: OptionalArgs`
```

If the output matches expectations, accept it as the expected stderr:

```bash
mv tests/trybuild_app_args/missing_with_args.stderr.new tests/trybuild_app_args/missing_with_args.stderr
```

- [ ] **Step 4: Re-run the test to confirm it passes**

```bash
cargo test --test trybuild_compile_fail 2>&1 | tail -10
```
Expected: 1 passed.

- [ ] **Step 5: Commit**

```bash
git add tests/trybuild_compile_fail.rs tests/trybuild_app_args/
git commit -S -m "Test: trybuild compile-fail when with_args missing for non-() Args

Pins the compile-time enforcement promise — forgetting with_args for
non-() Args is a compile error, not a runtime panic.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

### Task 29: Test category #3 — Custom Args of varying shapes

**Files:**
- Modify: `src/app/runtime/tests/mod.rs` (add new test module or test fn)

- [ ] **Step 1: Add the test**

```rust
#[cfg(test)]
mod custom_args_shapes {
    use super::*;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    // Args with PathBuf, Arc<Mutex>, and Vec<u8>.
    struct CustomArgsApp;
    #[derive(Clone, Default)]
    struct CustomState {
        path: PathBuf,
        counter: Arc<Mutex<u32>>,
        buf_len: usize,
    }
    #[derive(Clone)]
    enum CustomMsg {}

    struct CustomArgs {
        path: PathBuf,
        counter: Arc<Mutex<u32>>,
        buf: Vec<u8>,
    }

    impl App for CustomArgsApp {
        type State = CustomState;
        type Message = CustomMsg;
        type Args = CustomArgs;

        fn init(args: CustomArgs) -> (Self::State, Command<Self::Message>) {
            (CustomState {
                path: args.path,
                counter: args.counter,
                buf_len: args.buf.len(),
            }, Command::none())
        }

        fn update(_: &mut Self::State, _: Self::Message) -> Command<Self::Message> {
            Command::none()
        }

        fn view(_: &Self::State, _: &mut ratatui::Frame) {}
    }

    #[test]
    fn test_custom_args_shapes_move_correctly() {
        let counter = Arc::new(Mutex::new(0_u32));
        let args = CustomArgs {
            path: PathBuf::from("/tmp/fixture"),
            counter: counter.clone(),
            buf: vec![1, 2, 3, 4, 5],
        };

        let runtime = Runtime::<CustomArgsApp, _>::virtual_builder(80, 24)
            .with_args(args)
            .build()
            .unwrap();

        assert_eq!(runtime.state().path, PathBuf::from("/tmp/fixture"));
        assert_eq!(runtime.state().buf_len, 5);
        // Arc semantics survived the move
        assert_eq!(Arc::strong_count(&runtime.state().counter), 2);
    }
}
```

- [ ] **Step 2: Run and commit**

```bash
cargo nextest run -p envision --lib -E 'test(test_custom_args_shapes_move_correctly)'
```
Expected: 1 passed.

```bash
git add src/app/runtime/tests/mod.rs
git commit -S -m "Test: custom Args with PathBuf, Arc<Mutex>, Vec<u8> move correctly

Covers move semantics for non-Clone, non-Default, non-Copy Args types.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

### Task 30: Test category #4 — Multi-Runtime parallelism

**Files:**
- Modify: `src/app/runtime/tests/mod.rs` (add the parallel-runtimes test)

- [ ] **Step 1: Add the test**

```rust
#[test]
fn test_multiple_runtimes_with_distinct_args_in_one_test() {
    use std::path::PathBuf;

    struct MultiApp;
    #[derive(Clone, Default)]
    struct MultiState { dir: PathBuf }
    #[derive(Clone)]
    enum MultiMsg {}

    #[derive(Clone)]
    struct MultiArgs { dir: PathBuf }

    impl App for MultiApp {
        type State = MultiState;
        type Message = MultiMsg;
        type Args = MultiArgs;
        fn init(args: MultiArgs) -> (Self::State, Command<Self::Message>) {
            (MultiState { dir: args.dir }, Command::none())
        }
        fn update(_: &mut Self::State, _: Self::Message) -> Command<Self::Message> {
            Command::none()
        }
        fn view(_: &Self::State, _: &mut ratatui::Frame) {}
    }

    let runtime_a = Runtime::<MultiApp, _>::virtual_builder(80, 24)
        .with_args(MultiArgs { dir: PathBuf::from("/fixture/a") })
        .build()
        .unwrap();

    let runtime_b = Runtime::<MultiApp, _>::virtual_builder(80, 24)
        .with_args(MultiArgs { dir: PathBuf::from("/fixture/b") })
        .build()
        .unwrap();

    assert_eq!(runtime_a.state().dir, PathBuf::from("/fixture/a"));
    assert_eq!(runtime_b.state().dir, PathBuf::from("/fixture/b"));
}
```

- [ ] **Step 2: Run and commit**

```bash
cargo nextest run -p envision --lib -E 'test(test_multiple_runtimes_with_distinct_args_in_one_test)'
```
Expected: 1 passed.

```bash
git add src/app/runtime/tests/mod.rs
git commit -S -m "Test: multiple Runtimes with distinct args in one test

Pins the test-ergonomics unlock that motivates the entire redesign.
Replaces the static-OnceLock pattern that forced serialized tests.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

### Task 31: Test category #5 — `init` called exactly once per Runtime

**Files:**
- Modify: `src/app/runtime/tests/mod.rs`

- [ ] **Step 1: Add the test**

```rust
#[test]
fn test_init_called_exactly_once_per_runtime() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static INIT_COUNTER: AtomicUsize = AtomicUsize::new(0);

    struct CountInitApp;
    #[derive(Clone, Default)]
    struct CountInitState;
    #[derive(Clone)]
    enum CountInitMsg {}

    impl App for CountInitApp {
        type State = CountInitState;
        type Message = CountInitMsg;
        type Args = ();

        fn init(_args: ()) -> (Self::State, Command<Self::Message>) {
            INIT_COUNTER.fetch_add(1, Ordering::SeqCst);
            (CountInitState, Command::none())
        }

        fn update(_: &mut Self::State, _: Self::Message) -> Command<Self::Message> {
            Command::none()
        }

        fn view(_: &Self::State, _: &mut ratatui::Frame) {}
    }

    INIT_COUNTER.store(0, Ordering::SeqCst);

    let _runtime = Runtime::<CountInitApp, _>::virtual_builder(80, 24)
        .build()
        .unwrap();

    assert_eq!(INIT_COUNTER.load(Ordering::SeqCst), 1);
}
```

- [ ] **Step 2: Run and commit**

```bash
cargo nextest run -p envision --lib -E 'test(test_init_called_exactly_once_per_runtime)'
```
Expected: 1 passed.

```bash
git add src/app/runtime/tests/mod.rs
git commit -S -m "Test: App::init is called exactly once per Runtime construction

Pins the lifecycle contract — init runs exactly once per build().

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

### Task 32: Test category #6 — Builder-method preservation across with_args

**Files:**
- Modify: `src/app/runtime/builder.rs` (test module)

- [ ] **Step 1: Add the test**

```rust
#[test]
fn test_tick_rate_preserved_across_with_args() {
    use std::time::Duration;

    let custom_tick = Duration::from_millis(123);
    let runtime = Runtime::<TestApp, _>::virtual_builder(80, 24)
        .tick_rate(custom_tick)  // before with_args
        .with_args(())            // promotes to ConfiguredRuntimeBuilder
        .build()
        .unwrap();

    assert_eq!(runtime.config().tick_rate, custom_tick);
}

#[test]
fn test_tick_rate_set_after_with_args_is_preserved() {
    use std::time::Duration;

    let custom_tick = Duration::from_millis(456);
    let runtime = Runtime::<TestApp, _>::virtual_builder(80, 24)
        .with_args(())
        .tick_rate(custom_tick)  // after with_args — uses ConfiguredRuntimeBuilder method
        .build()
        .unwrap();

    assert_eq!(runtime.config().tick_rate, custom_tick);
}
```

If `Runtime` doesn't currently expose a `config()` accessor, add a `pub(crate)` one or use whatever introspection the `Runtime` struct provides for its config. The point is to verify the value survives.

- [ ] **Step 2: Run and commit**

```bash
cargo nextest run -p envision --lib -E 'test(test_tick_rate_preserved)'
```
Expected: 2 passed.

```bash
git add src/app/runtime/builder.rs
git commit -S -m "Test: builder methods survive with_args promotion

Pins the carry-over guarantee — tick_rate / config / etc. set before
with_args are preserved after promotion to ConfiguredRuntimeBuilder.
Catches future implementer sloppiness where a refactor of with_args
accidentally drops a config field.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Phase 4 — Final verification + cross-reference

### Task 33: Pre-merge verification gate

**Files:**
- (no edits — verification step)

- [ ] **Step 1: Full nextest**

```bash
cargo nextest run -p envision --lib --all-features 2>&1 | tail -10
```
Expected: all pass; count higher than baseline by ~6 (the new tests).

- [ ] **Step 2: Doc tests**

```bash
cargo test --doc --all-features 2>&1 | tail -10
```
Expected: all pass.

- [ ] **Step 3: Trybuild**

```bash
cargo test --test trybuild_compile_fail 2>&1 | tail -10
```
Expected: 1 passed.

- [ ] **Step 4: Clippy**

```bash
cargo clippy --all-features --tests --examples -- -D warnings 2>&1 | tail -10
```
Expected: clean.

- [ ] **Step 5: Format**

```bash
cargo fmt --check
```
Expected: no diff.

- [ ] **Step 6: Examples**

```bash
cargo build --examples --all-features 2>&1 | tail -10
```
Expected: all 84 examples build clean.

- [ ] **Step 7: Doc build**

```bash
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features 2>&1 | tail -5
```
Expected: clean.

- [ ] **Step 8: Audit scorecard**

```bash
./tools/audit/target/release/envision-audit scorecard 2>&1 | tail -15
```
Expected: 9/9 (or no regression vs baseline).

---

### Task 34: Open implementation PR

**Files:**
- (push branch + open PR)

- [ ] **Step 1: Push**

```bash
git push -u origin <impl-branch-name>
```

- [ ] **Step 2: Open PR**

```bash
gh pr create --title "App::init args redesign (D1) — implementation" --body "$(cat <<'EOF'
## Summary

Implements the D1 redesign per spec (PR #463) and plan (PR #<plan-pr>).

- `App::init() -> ...` becomes `App::init(args: Self::Args) -> ...`
- `App` trait gains `type Args` (no default — explicit `type Args = ();` required on stable)
- Sealed `OptionalArgs` marker gates the no-args shortcut on `RuntimeBuilder::build()`
- `RuntimeBuilder` is split into `RuntimeBuilder` + `ConfiguredRuntimeBuilder` (typestate-lite via `with_args` promotion)
- `RuntimeBuilder::state(...)` deleted — its role subsumed by `with_args` + a real `init` impl
- 157 `init()` call sites migrated across 103 files (84 examples, 12 src, 3 integration tests, 1 bench, 2 harness, 1 lib.rs)
- 6 new test categories pin: `()` shortcut, trybuild compile-fail for missing `with_args`, custom Args shapes, multi-Runtime parallelism, `init`-called-once, builder-method carry-over

Tracks leadline gap **D1**.

## Test plan
- [ ] CI green (16 checks expected)
- [ ] Squash-merge per project rule
- [ ] Tracking-doc PR follows: mark D1 ✅ resolved in `docs/customer-feedback/2026-05-01-leadline-gaps.md`
- [ ] Notify leadline to migrate `LeadlineApp` and remove `set_baseline_dir` workaround

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

---

## Self-review checklist (run before declaring this plan done)

- [ ] **Spec coverage:** Every section of `2026-05-02-app-init-args-design.md` maps to one or more tasks.
  - § App trait change → Task 4 ✅
  - § OptionalArgs marker → Tasks 2-3 ✅
  - § RuntimeBuilder split → Tasks 5-7, 9-11 ✅
  - § `with_args` carry-over guarantee → Task 7 (impl) + Task 32 (test) ✅
  - § `.state()` deletion → Task 8 ✅
  - § Migration table including test-mutation case → Task 25 (CHANGELOG) ✅
  - § Test ergonomics unlock → Task 30 ✅
  - § 6 test categories → Tasks 27-32 ✅
  - § Files to touch → all enumerated in Phase 2 ✅

- [ ] **Placeholder scan:** No "TBD", "TODO", "implement later", "fill in details", "similar to Task N", or "appropriate error handling" strings.

- [ ] **Type consistency:** `OptionalArgs` named the same in every task; `ConfiguredRuntimeBuilder` named the same in every task; `with_args` signature consistent.

- [ ] **Atomic-switch boundary clear:** Tasks 4–25 explicitly preparation for one commit (Task 26 is the commit).

- [ ] **Phase 3 tests cover all 6 spec categories:**
  - #1 `type Args = ()` works → Task 27
  - #2 trybuild compile-fail → Task 28
  - #3 Custom Args shapes → Task 29
  - #4 Multi-Runtime parallelism → Task 30
  - #5 `init` called once → Task 31
  - #6 Builder-method carry-over → Task 32
