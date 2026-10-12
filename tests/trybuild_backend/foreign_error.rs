//! A backend whose error wraps a third-party error (std::fmt::Error stands in
//! for "foreign"): the consumer writes a local newtype + From impl.
use envision::adapter::DualBackend;
use envision::backend::Backend;
use envision::prelude::*;
use envision::ratatui::backend::{ClearType, WindowSize};
use envision::ratatui::buffer::Cell;
use envision::ratatui::layout::{Position, Size};
use envision::{CaptureBackend, EnvisionError, Runtime};

#[derive(Debug)]
struct MyError(std::fmt::Error);
impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "backend: {}", self.0)
    }
}
impl std::error::Error for MyError {}
impl From<MyError> for EnvisionError {
    fn from(e: MyError) -> Self {
        EnvisionError::other(e)
    }
}

struct MyBackend(CaptureBackend);
impl Backend for MyBackend {
    type Error = MyError;
    fn draw<'a, I: Iterator<Item = (u16, u16, &'a Cell)>>(&mut self, c: I) -> Result<(), MyError> {
        let Ok(()) = self.0.draw(c);
        Ok(())
    }
    fn hide_cursor(&mut self) -> Result<(), MyError> { Ok(()) }
    fn show_cursor(&mut self) -> Result<(), MyError> { Ok(()) }
    fn get_cursor_position(&mut self) -> Result<Position, MyError> { Ok(Position::ORIGIN) }
    fn set_cursor_position<P: Into<Position>>(&mut self, _p: P) -> Result<(), MyError> { Ok(()) }
    fn clear(&mut self) -> Result<(), MyError> { Ok(()) }
    fn clear_region(&mut self, _c: ClearType) -> Result<(), MyError> { Ok(()) }
    fn size(&self) -> Result<Size, MyError> { Ok(Size::new(4, 2)) }
    fn window_size(&mut self) -> Result<WindowSize, MyError> {
        Ok(WindowSize { columns_rows: Size::new(4, 2), pixels: Size::new(32, 32) })
    }
    fn flush(&mut self) -> Result<(), MyError> { Ok(()) }
}

struct NoopApp;
impl App for NoopApp {
    type State = ();
    type Message = ();
    type Args = ();
    fn init(_: ()) -> ((), Command<()>) { ((), Command::none()) }
    fn update(_: &mut (), _: ()) -> Command<()> { Command::none() }
    fn view(_: &(), _: &mut Frame) {}
}

fn main() -> Result<(), EnvisionError> {
    let mut runtime = Runtime::<NoopApp, _>::builder(MyBackend(CaptureBackend::new(4, 2))).build()?;
    runtime.render()?;
    let dual = DualBackend::new(MyBackend(CaptureBackend::new(4, 2)), CaptureBackend::new(4, 2));
    let mut dual_runtime = Runtime::<NoopApp, _>::builder(dual).build()?;
    dual_runtime.render()?;
    Ok(())
}
