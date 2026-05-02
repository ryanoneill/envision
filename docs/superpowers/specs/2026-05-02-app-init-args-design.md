# App::init args redesign — design spec

**Date:** 2026-05-02
**Status:** Approved; implementation plan to follow.
**Tracks:** leadline gap **D1** in `docs/customer-feedback/2026-05-01-leadline-gaps.md`
**Source brief:** `~/workspace/ryanoneill/rust-ai-explorations/notes/envision_app_init_args_redesign.md`
(leadline commit `c29d1cf`, signed)

---

## TL;DR

`App::init() -> (State, Command<Msg>)` takes no arguments. Every consumer that needs injected
config (CLI args, file paths, opened DB handles, test fixtures) is forced into one of two
wrong shapes: a `static OnceLock<T>` that panics on second `set()` and breaks parallel tests,
or `RuntimeBuilder::state(s, cmd)` which makes `App::init` a panicking unimplemented method.

Replace `init()` with `init(args: Self::Args)` keyed by an associated type on `App`.
Gate the no-args shortcut on `RuntimeBuilder::build()` with a sealed `OptionalArgs` marker
trait (implemented only for `()`), and use a typestate-lite split where `with_args` promotes
`RuntimeBuilder<A, B>` into a separate `ConfiguredRuntimeBuilder<A, B>` whose `build()` is
unconditionally available. `RuntimeBuilder::state(s, cmd)` is deleted — its only role is
subsumed by `with_args` + a real `init` impl.

Compile-time enforcement on stable Rust. Forgetting `.with_args(...)` for a non-`()` Args
type fails to compile, not at runtime.

---

## Goals

1. **Channel injected config through the framework** — typed, per-impl, compiler-checked.
2. **Make `init` always meaningful** — delete the panicking default; every `App` impl owns
   its own initialization.
3. **Unlock parallel testing** — multiple `Runtime` instances with different args in one
   process, no shared mutable global.
4. **Compile-time enforcement** — forgetting `.with_args` for non-`()` Args is a compile
   error, not a runtime panic.
5. **Zero consumer constraint on `Args`** — no `Default` bound, no `Clone` bound, no
   `'static` bound beyond what `init` actually needs.
6. **Single atomic migration** — one breaking-change PR, no shim, no deprecation cycle.
   envision is pre-1.0 and ruthless about API ripping.

## Non-goals

- **Async `init`.** Apps that want async work in initialization do it in `main()` and pass
  the resolved value as part of `Args`. Async init is a much bigger change to the runtime
  shape; deferred.
- **`Default` auto-derive for trivial apps.** A convenience macro that generates
  `fn init(_: Self::Args) -> (Self::State::default(), Command::none())` could be useful but
  is out of scope. Add later if there's demand.
- **Runtime-set args.** Args are passed once at builder time and consumed by `init`. There's
  no API for replacing args on a running `Runtime`.

---

## Design

### App trait change (`src/app/model/mod.rs`)

```rust
pub trait App: Sized {
    type State;
    type Message: Send + 'static;

    /// Configuration / dependencies handed in at construction time.
    /// Apps that need no injected config declare `type Args = ();`.
    /// Common uses: CLI-parsed paths, env-derived URLs, opened DB handles,
    /// preloaded fixture data for tests.
    type Args;

    /// Initializes the application state from the provided args.
    ///
    /// Called exactly once per `Runtime` construction by the builder.
    /// Args are consumed (move semantics); store anything you need to keep
    /// in `State`.
    fn init(args: Self::Args) -> (Self::State, Command<Self::Message>);

    fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message>;
    fn view(state: &Self::State, frame: &mut Frame);
    // handle_event / handle_event_with_state / on_exit / should_quit / on_tick — unchanged
}
```

The panicking default impl of `init` is **deleted**. Every `App` impl declares both
`type Args = ...` and `fn init(args)`. The compiler drives the migration.

#### Stable-Rust constraint

Associated-type defaults (`type Args = ();` in the trait declaration) are unstable on stable
Rust ([rust-lang/rust#29661](https://github.com/rust-lang/rust/issues/29661)). Every consumer
must declare `type Args = X;` explicitly, even when `X = ()`. This is a one-line-per-impl
mechanical addition, compiler-driven.

#### Why associated type, not generic

`type Args` is per-impl, not per-call. Each `App` declares its own args shape; consumers
don't have to thread a generic parameter through every `Runtime<A>` reference they hold.
This matches `iced::Application::Flags` and similar prior art.

#### Why move semantics

`init` consumes `args`. Most cases want this — args are constructed by `main()` and used
once. Apps that want to keep args around store the relevant fields in `State` during init.
No `Clone` bound on `Args`.

---

### OptionalArgs sealed marker (`src/app/model/optional_args.rs`)

```rust
mod sealed {
    pub trait Sealed {
        fn default_optional_args() -> Self;
    }

    impl Sealed for () {
        fn default_optional_args() -> Self {}
    }
}

pub trait OptionalArgs: sealed::Sealed {}

impl OptionalArgs for () {}
```

**Sealed:** consumers cannot extend `OptionalArgs` to their own types. The only way to get
the no-`with_args` shortcut on `RuntimeBuilder::build()` is to declare `type Args = ();`.

**`default_optional_args` lives in the sealed supertrait** so it's not part of envision's
public surface. Consumers see `OptionalArgs` as an empty marker; the
`default_optional_args` accessor is internal.

Module is `pub(crate)` and re-exported as `pub use optional_args::OptionalArgs;` from
`src/app/mod.rs`. The marker trait needs to be public so it can appear in the
`where A::Args: OptionalArgs` bound on `RuntimeBuilder::build()`. The sealed supertrait
stays private.

---

### RuntimeBuilder split (`src/app/runtime/builder.rs`)

The builder is split into two structs to express the typestate transition without falling
into Rust's no-overlapping-impls rule. `RuntimeBuilder` is the entry struct returned by
`Runtime::terminal_builder()` / `Runtime::virtual_builder()`. `with_args` consumes it and
produces `ConfiguredRuntimeBuilder` whose `build()` is unconditionally available.

```rust
pub struct RuntimeBuilder<A: App, B: Backend> {
    backend: B,
    config: Option<RuntimeConfig>,
}

pub struct ConfiguredRuntimeBuilder<A: App, B: Backend> {
    backend: B,
    config: Option<RuntimeConfig>,
    args: A::Args,
}
```

Note: neither struct carries an `Option<A::Args>` field. The presence of args is encoded
in the type itself — `ConfiguredRuntimeBuilder` always has them, `RuntimeBuilder` never
does. This is the typestate.

#### `with_args` consumes and promotes

```rust
impl<A: App, B: Backend> RuntimeBuilder<A, B> {
    pub fn with_args(self, args: A::Args) -> ConfiguredRuntimeBuilder<A, B> {
        ConfiguredRuntimeBuilder {
            backend: self.backend,
            config: self.config,
            args,
        }
    }
}
```

**Carry-over guarantee:** `with_args` consumes the `RuntimeBuilder` and constructs
`ConfiguredRuntimeBuilder { backend: self.backend, config: self.config, args }`. Any
`.config(...)` / `.tick_rate(...)` / `.frame_rate(...)` / `.max_messages(...)` /
`.channel_capacity(...)` calls made before `.with_args` are preserved. This avoids a class
of "user calls `tick_rate` before `with_args`, value silently lost" bugs.

#### Config-shaping methods on both structs

`config`, `tick_rate`, `frame_rate`, `max_messages`, `channel_capacity` exist on both
`RuntimeBuilder` and `ConfiguredRuntimeBuilder`. Consumers may chain them in either order
relative to `with_args`. To avoid duplicating implementations, both delegate to a private
helper (e.g. `BuilderConfigCommon` trait or shared inherent methods on a common helper
struct). Implementation choice deferred to the plan.

#### `build()` — two impls, one per state

```rust
// No-args path: RuntimeBuilder::build() only available when A::Args: OptionalArgs.
impl<A: App, B: Backend> RuntimeBuilder<A, B>
where
    A::Args: OptionalArgs,
{
    pub fn build(self) -> error::Result<Runtime<A, B>> {
        let args = <A::Args as sealed::Sealed>::default_optional_args();
        self.with_args(args).build()
    }
}

// Args path: ConfiguredRuntimeBuilder::build() always available, no bound.
impl<A: App, B: Backend> ConfiguredRuntimeBuilder<A, B> {
    pub fn build(self) -> error::Result<Runtime<A, B>> {
        let (state, init_cmd) = A::init(self.args);
        Runtime::with_backend_state_and_config(
            self.backend,
            state,
            init_cmd,
            self.config.unwrap_or_default(),
        )
    }
}
```

**Compile-time outcomes:**

| Consumer pattern | `A::Args` | Behavior |
|---|---|---|
| `terminal_builder()?.build()?` | `()` | ✅ Compiles. `with_args(())` is implicit. |
| `terminal_builder()?.with_args(x).build()?` | `()` or anything | ✅ Compiles. |
| `terminal_builder()?.build()?` | not `()` | ❌ Compile error: `method 'build' not found...; trait bound 'OptionalArgs' is not satisfied for MyArgs`. |
| `terminal_builder()?.with_args(wrong_type).build()?` | not `()` | ❌ Compile error: type mismatch. |

The error message for the missing-`with_args` case is a standard Rust compiler diagnostic
("method not found because trait bound not satisfied"). Quality-of-error is acceptable
without `#[diagnostic::on_unimplemented]`; can be enhanced later if user feedback flags it.

#### `.state()` deletion

`RuntimeBuilder::state(state, init_cmd)` at `src/app/runtime/builder.rs:209` is **removed**.
Its only role — bypass `init` to inject pre-built state from external sources — is fully
subsumed by `with_args` + a real `init` impl. Consumers migrate as follows:

```rust
// Before
let state = build_state(deps);
let runtime = Runtime::<MyApp, _>::terminal_builder()?
    .state(state, Command::none())
    .build()?;

// After
let runtime = Runtime::<MyApp, _>::terminal_builder()?
    .with_args(MyArgs { deps })
    .build()?;
// And in App::init:
fn init(args: MyArgs) -> (MyState, Command<MyMsg>) {
    (build_state(args.deps), Command::none())
}
```

For startup `Command`s previously passed as the second arg of `.state(s, cmd)`, return them
as the second tuple element from `init`.

---

## Migration

### App impl migration

```rust
// Before
impl App for MyApp {
    type State = MyState;
    type Message = MyMsg;
    fn init() -> (MyState, Command<MyMsg>) {
        // ... reads from a static or hardcodes config ...
    }
}

// After (no-args case — envision's own examples + most existing apps)
impl App for MyApp {
    type State = MyState;
    type Message = MyMsg;
    type Args = ();
    fn init(_args: ()) -> (MyState, Command<MyMsg>) {
        // ... unchanged ...
    }
}

// After (args case — leadline)
impl App for LeadlineApp {
    type State = State;
    type Message = Msg;
    type Args = LeadlineArgs;
    fn init(args: LeadlineArgs) -> (State, Command<Msg>) {
        let LeadlineArgs { baseline_dir } = args;
        // ... uses baseline_dir directly, no static lookup ...
    }
}
```

### Entry-point migration

```rust
// Before — no-args
Runtime::<MyApp, _>::terminal_builder()?.build()?.run_terminal_blocking()?;

// After — no-args (unchanged shape)
Runtime::<MyApp, _>::terminal_builder()?.build()?.run_terminal_blocking()?;

// Before — leadline static-injection dance
set_baseline_dir(baseline_dir);
Runtime::<LeadlineApp, _>::terminal_builder()?.build()?.run_terminal_blocking()?;

// After — leadline with args
Runtime::<LeadlineApp, _>::terminal_builder()?
    .with_args(LeadlineArgs { baseline_dir })
    .build()?
    .run_terminal_blocking()?;
```

### Migration table

| Old | New |
|---|---|
| `fn init() -> (State, Command<Msg>)` | `type Args = (); fn init(_args: ()) -> (State, Command<Msg>)` (no-args case) |
| `static GLOBAL: OnceLock<T> = ...; fn init() { GLOBAL.get()... }` | `type Args = MyArgs; fn init(args: MyArgs) { args.field... }` |
| `RuntimeBuilder::state(state, cmd)` | `RuntimeBuilder::with_args(args)`; move state-building into `init` |
| `App::init` panic default | Required method; compile error if not implemented |
| Test code that mutates pre-built state before `.state(...)` | Introduce an enum-shaped `Args` variant (e.g. `enum TestArgs { Fresh(ProductionArgs), Prebuilt(State) }`) **or** refactor the mutations into `init`. The framework deliberately doesn't carry forward an escape hatch for this; a clean `Args`-driven test setup is more reproducible than ad-hoc state mutation. |

---

## Test ergonomics — the unlock

The static-`OnceLock` pattern leadline currently uses is fundamentally hostile to parallel
testing:

```rust
// Before — second test panics on the second OnceLock::set
#[test]
fn test_renders_table_a() {
    set_baseline_dir(fixture_a());  // panics if any other test ran first
    let mut vt = Runtime::<App, _>::virtual_builder(80, 24).build()?;
    // ...
}
```

After the redesign, each test owns its args:

```rust
#[test]
fn test_renders_table_a() {
    let mut vt = Runtime::<App, _>::virtual_builder(80, 24)
        .with_args(AppArgs { baseline_dir: fixture_a() })
        .build()?;
    // ...
}

#[test]
fn test_renders_table_b() {
    let mut vt = Runtime::<App, _>::virtual_builder(80, 24)
        .with_args(AppArgs { baseline_dir: fixture_b() })
        .build()?;
    // runs in parallel with test_renders_table_a; no shared state
}
```

This isn't a marginal ergonomic improvement — it's the difference between "tests can run in
parallel" and "tests must serialize through a global." Multiplies across every consumer that
wants to test their TUI under different fixtures.

---

## What leadline deletes once this lands

Static, setter, getter — all gone:

```rust
// Removed from leadline/src/app.rs
static BASELINE_DIR: OnceLock<PathBuf> = OnceLock::new();
pub fn set_baseline_dir(dir: PathBuf) { ... }
fn baseline_dir() -> PathBuf { ... }
```

`set_baseline_dir(...)` calls in `leadline/src/main.rs` and
`leadline/examples/virtual_preview.rs` replaced with `.with_args(LeadlineArgs { baseline_dir })`.
Net delete on leadline side: ~25 lines of global-state plumbing.

---

## Files to touch

| File | Change |
|---|---|
| `src/app/model/mod.rs` | Add `type Args` (no default); change `init` signature; remove panicking default. |
| `src/app/model/optional_args.rs` *(new)* | Sealed `Sealed` supertrait + public `OptionalArgs` marker; impl for `()`. |
| `src/app/model/mod.rs` (re-export) | `pub use optional_args::OptionalArgs;` |
| `src/app/runtime/builder.rs` | Split `RuntimeBuilder` and add `ConfiguredRuntimeBuilder`. Add `with_args`. Delete `state(...)`. Update `build()` impls per typestate. Mirror config-shaping methods. |
| `src/app/runtime/mod.rs` | Update doc comments referencing `.state()` / `init()`. |
| `src/app/model/tests.rs` | Update existing tests; add five new test categories (see below). |
| `src/app/runtime/tests/*.rs` | Migrate every test `App` impl: `type Args = ();` + `fn init(_: ())`. Tests using `.state(...)` rewritten to use `.with_args(...)` plus a real `init` impl. |
| `examples/*.rs` | Every example `App` impl gets `type Args = ();` and `fn init(_: ())`. ~30 examples. |
| Doc tests across `src/app/`, `src/lib.rs`, `src/component/*` | Every `impl App` in a doc test gets `type Args = ();` and `_: ()` arg. |
| `tests/integration.rs`, `tests/integration_stress.rs` | Migrate any `App` impls. |
| `CHANGELOG.md` | Breaking-change entry + migration table. |
| `tools/audit/` | If any audit checks reference `init()` signature, update. |

Approximate scope: 234 `fn init() ->` call sites identified by grep; mostly mechanical
one-line edits. Same scale as G7's `TableRow::cells()` migration.

---

## Tests envision should add

In `src/app/model/tests.rs` and `src/app/runtime/tests/`:

1. **`type Args = ()` works** — explicit-`()` declaration; `terminal_builder()?.build()?`
   succeeds without `with_args`. Pin the no-args ergonomic shape.

2. **Compile-fail when `with_args` missing for non-`()` Args** — `trybuild` test with a
   non-`()` Args type and a builder chain that omits `with_args`. Expected error: "method
   `build` not found... `OptionalArgs` not implemented for ...".

3. **Custom Args of varying shapes** — `PathBuf`, `Arc<Mutex<...>>`-shaped mock, `Vec<u8>`
   fixture buffer. Verify move semantics work for non-`Clone`, non-`Copy`, non-`Default`
   Args types.

4. **Multi-`Runtime` parallelism** — single `#[test]` constructs two virtual `Runtime`s with
   different args, asserts each gets its own state. Pins the test-ergonomics unlock that
   motivates the entire redesign.

5. **`init` called exactly once per `Runtime`** — instrument with `Arc<AtomicUsize>` counter
   inside `init`; build a `Runtime`; verify counter == 1. Pin the lifecycle contract.

6. **Builder-method preservation across `with_args`** *(implied by carry-over guarantee)* —
   call `.tick_rate(d)` before `.with_args(args)`, build, assert resulting `Runtime` config
   carries the tick rate. Pin the carry-over invariant against future regressions.

---

## Risks & open questions

### Risks

- **Doc-test churn.** ~234 init sites across the codebase. Mechanical but voluminous;
  cargo-driven find-and-replace plus per-file review will be needed. Mitigated by the same
  subagent-driven approach used for G7's `cells()` migration.

- **Public-trait surface change.** `App::Args` becomes part of the trait surface. Adding it
  is breaking; the migration must be atomic (no shim). Pre-1.0 + ruthless API discipline
  makes this acceptable.

- **`ConfiguredRuntimeBuilder` discoverability.** Users may not realize the second type
  exists until they try to extract a builder mid-chain. Documented in `App` and
  `RuntimeBuilder` module docs; a doc-test demonstrates the chain. Most users never need to
  name the type.

### Decisions resolved during brainstorming

| Question | Resolution |
|---|---|
| Marker trait vs `Default` bound vs typestate | Sealed marker trait (`OptionalArgs`) with typestate-lite via `with_args` promotion to `ConfiguredRuntimeBuilder`. Stable Rust, compile-time enforcement, no consumer constraint. |
| Naming (`Args` / `Config` / `InitArgs` / `Bootstrap`) | `Args`. `Config` collides with `RuntimeConfig`; `InitArgs` is verbose; `Bootstrap` is novel. |
| Should `init` be `async`? | No — sync only. Async work goes in `main()`, resolved values pass via `Args`. Async init is a much larger change, deferred. |
| Drop `panic!` default for `init` or keep a different default? | Drop. `init` becomes required. Convenience `Default`-impl auto-derive for trivial apps deferred to a possible future macro. |

### Open implementation questions for the plan

These don't affect the spec but the plan must decide:

- Where to put the shared config-shaping helper between `RuntimeBuilder` and
  `ConfiguredRuntimeBuilder` (private trait, free fn, or `&mut Option<RuntimeConfig>` accessor).
- Whether to add `#[diagnostic::on_unimplemented]` to `OptionalArgs` for a friendlier error
  message on the missing-`with_args` case (Rust 1.78+, stable).
- Whether the `with_args(()).build()` shortcut on `ConfiguredRuntimeBuilder` is worth
  documenting as a way for users to *force* the args path even with `Args = ()`.

---

## Cadence

Same 4-PR cadence as G7:

1. **PR α** — this design spec (`docs/superpowers/specs/2026-05-02-app-init-args-design.md`).
2. **PR β** — implementation plan (`docs/superpowers/plans/2026-05-02-app-init-args.md`).
3. **PR γ** — implementation. Single atomic breaking-change PR. Subagent-driven if the plan
   decomposes cleanly into independent tasks (likely, given the file-by-file shape of the
   migration); inline otherwise.
4. **Tracking-doc PR** — mark D1 ✅ resolved in
   `docs/customer-feedback/2026-05-01-leadline-gaps.md` once γ lands.

---

## Related context

- leadline's customer-feedback inventory:
  `docs/customer-feedback/2026-05-01-leadline-gaps.md` (item D1)
- leadline-side gaps tracking: `~/workspace/ryanoneill/rust-ai-explorations/notes/envision_gaps.md`
- The `App::init` panic default: `src/app/model/mod.rs:205-214`
- The `.state()` escape hatch: `src/app/runtime/builder.rs:209`
- The `RuntimeBuilder::build()` path: `src/app/runtime/builder.rs:359`
- Prior atomic-migration playbook: G1+G3+G7 spec
  (`docs/superpowers/specs/2026-05-02-table-sort-cell-unification-design.md`),
  plan (`docs/superpowers/plans/2026-05-02-table-sort-cell-unification.md`), implementation
  PR #461 (`235bcae`).

D1 is the first of the four open D-gaps. Every other init-touching gap (D2 PaneLayout closure
flow, D5 styled-line primitive, D7 snapshot testing docs) becomes easier to brief once apps
have a clean way to take args.
