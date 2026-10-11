//! Characterization: backend errors reach envision's API classified as before.

use std::io;

use envision::backend::Backend;
use envision::prelude::*;
use envision::ratatui::backend::{ClearType, WindowSize};
use envision::ratatui::buffer::Cell;
use envision::ratatui::layout::{Position, Size};
use envision::{EnvisionError, Runtime};

struct NoopApp;

impl App for NoopApp {
    type State = ();
    type Message = ();
    type Args = ();
    fn init(_: ()) -> ((), Command<()>) {
        ((), Command::none())
    }
    fn update(_: &mut (), _: ()) -> Command<()> {
        Command::none()
    }
    fn view(_: &(), _: &mut Frame) {}
}

/// A backend whose `draw` fails like a closed terminal does.
struct FailingBackend;

impl Backend for FailingBackend {
    type Error = io::Error;

    fn draw<'a, I>(&mut self, _content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        Err(io::Error::new(io::ErrorKind::BrokenPipe, "terminal gone"))
    }
    fn hide_cursor(&mut self) -> io::Result<()> {
        Ok(())
    }
    fn show_cursor(&mut self) -> io::Result<()> {
        Ok(())
    }
    fn get_cursor_position(&mut self) -> io::Result<Position> {
        Ok(Position::ORIGIN)
    }
    fn set_cursor_position<P: Into<Position>>(&mut self, _position: P) -> io::Result<()> {
        Ok(())
    }
    fn clear(&mut self) -> io::Result<()> {
        Ok(())
    }
    fn clear_region(&mut self, _clear_type: ClearType) -> io::Result<()> {
        Ok(())
    }
    fn size(&self) -> io::Result<Size> {
        Ok(Size::new(10, 3))
    }
    fn window_size(&mut self) -> io::Result<WindowSize> {
        Ok(WindowSize {
            columns_rows: Size::new(10, 3),
            pixels: Size::new(80, 48),
        })
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

// Test 2: an io::Error raised by a backend surfaces as EnvisionError::Io.
#[tokio::test]
async fn render_io_error_is_classified_as_io() {
    let mut runtime = Runtime::<NoopApp, _>::builder(FailingBackend)
        .build()
        .unwrap();
    let err = runtime.render().unwrap_err();
    assert!(matches!(err, EnvisionError::Io(ref e) if e.kind() == io::ErrorKind::BrokenPipe));
}

// Test 3: the same error reaches tick() and run_ticks().
#[tokio::test]
async fn tick_propagates_backend_draw_error() {
    let mut runtime = Runtime::<NoopApp, _>::builder(FailingBackend)
        .build()
        .unwrap();
    assert!(matches!(runtime.tick(), Err(EnvisionError::Io(_))));
    assert!(matches!(runtime.run_ticks(3), Err(EnvisionError::Io(_))));
}

// Test 6a: accessors need only `B: Backend`.
fn backend_of<B: Backend>(runtime: &Runtime<NoopApp, B>) -> &B {
    runtime.backend()
}

#[tokio::test]
async fn accessor_needs_only_backend_bound() {
    let runtime = Runtime::<NoopApp, _>::builder(FailingBackend)
        .build()
        .unwrap();
    let _backend: &FailingBackend = backend_of(&runtime);
}

// RF-4: DualBackend<CaptureBackend> is a Runtime backend; both halves are
// Infallible, so it builds, renders and ticks.
#[tokio::test]
async fn dual_backend_of_capture_runs_in_runtime() {
    use envision::CaptureBackend;
    use envision::adapter::DualBackend;
    let dual = DualBackend::new(CaptureBackend::new(10, 3), CaptureBackend::new(10, 3));
    let mut runtime = Runtime::<NoopApp, _>::builder(dual).build().unwrap();
    runtime.render().unwrap();
    runtime.tick().unwrap();
}
