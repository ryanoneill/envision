//! Tests for args-flow plumbing through `Runtime` construction.
//!
//! These tests pin the behavior introduced by the D1 (App::Args)
//! initialization-arguments work:
//!
//! - Custom `Args` shapes (`PathBuf`, `Arc<Mutex>`, `Vec<u8>`) move correctly
//!   through `with_args` into `App::init`.
//! - Multiple `Runtime`s with distinct `Args` can coexist in a single test.
//! - `App::init` is called exactly once per `Runtime` construction.
//!
//! Extracted from `tests/mod.rs` to keep that file under the project's
//! 1000-line ceiling. No behavior change.

use super::*;

// =========================================================================
// init lifecycle — Test category #5 from the spec
// =========================================================================

#[test]
fn test_init_called_exactly_once_per_runtime() {
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

// =========================================================================
// Multi-Runtime parallelism — Test category #4 from the spec
// =========================================================================

#[test]
fn test_multiple_runtimes_with_distinct_args_in_one_test() {
    use std::path::PathBuf;

    struct MultiApp;
    #[derive(Clone, Default)]
    struct MultiState {
        dir: PathBuf,
    }
    #[derive(Clone)]
    enum MultiMsg {}

    #[derive(Clone)]
    struct MultiArgs {
        dir: PathBuf,
    }

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
        .with_args(MultiArgs {
            dir: PathBuf::from("/fixture/a"),
        })
        .build()
        .unwrap();

    let runtime_b = Runtime::<MultiApp, _>::virtual_builder(80, 24)
        .with_args(MultiArgs {
            dir: PathBuf::from("/fixture/b"),
        })
        .build()
        .unwrap();

    assert_eq!(runtime_a.state().dir, PathBuf::from("/fixture/a"));
    assert_eq!(runtime_b.state().dir, PathBuf::from("/fixture/b"));
}

// =========================================================================
// Custom Args shapes — Test category #3 from the spec
// =========================================================================

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
            (
                CustomState {
                    path: args.path,
                    counter: args.counter,
                    buf_len: args.buf.len(),
                },
                Command::none(),
            )
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
