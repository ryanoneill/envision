//! Compile-fail fixture: forgetting `.with_args(...)` for a non-`()` Args type.
//!
//! When `App::Args` is anything other than `()`, calling `.build()` directly
//! on `RuntimeBuilder` is unavailable: the `OptionalArgs` trait bound on
//! `RuntimeBuilder::build` is not satisfied. The user must call
//! `.with_args(...)` to obtain a `ConfiguredRuntimeBuilder`, whose `build`
//! is unconditionally available.

use envision::prelude::*;
use std::path::PathBuf;

struct ArgsApp;
#[derive(Clone, Default)]
struct ArgsState;
#[derive(Clone)]
enum ArgsMsg {}

#[derive(Clone)]
struct MyArgs {
    _path: PathBuf,
}

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
    let _ = Runtime::<ArgsApp, _>::virtual_builder(80, 24)
        .build()
        .unwrap();
}
