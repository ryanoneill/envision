//! The bump's error-type contract (spec § 2).

use std::convert::Infallible;
use std::fmt;

use envision::CaptureBackend;
use envision::adapter::DualBackend;
use envision::backend::Backend;

fn assert_error_type<B: Backend<Error = E>, E>() {}

// Test 1
#[test]
fn capture_backend_error_is_infallible() {
    assert_error_type::<CaptureBackend, Infallible>();
}

#[derive(Debug)]
struct SizeError;
impl fmt::Display for SizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("size unavailable")
    }
}
impl std::error::Error for SizeError {}

/// A primary whose only failure is `size()`.
struct NoSize(CaptureBackend);

impl Backend for NoSize {
    type Error = SizeError;
    fn draw<'a, I>(&mut self, c: I) -> Result<(), SizeError>
    where
        I: Iterator<Item = (u16, u16, &'a envision::ratatui::buffer::Cell)>,
    {
        let Ok(()) = self.0.draw(c);
        Ok(())
    }
    fn hide_cursor(&mut self) -> Result<(), SizeError> {
        Ok(())
    }
    fn show_cursor(&mut self) -> Result<(), SizeError> {
        Ok(())
    }
    fn get_cursor_position(&mut self) -> Result<envision::ratatui::layout::Position, SizeError> {
        Ok(envision::ratatui::layout::Position::ORIGIN)
    }
    fn set_cursor_position<P: Into<envision::ratatui::layout::Position>>(
        &mut self,
        _p: P,
    ) -> Result<(), SizeError> {
        Ok(())
    }
    fn clear(&mut self) -> Result<(), SizeError> {
        Ok(())
    }
    fn clear_region(&mut self, _c: envision::ratatui::backend::ClearType) -> Result<(), SizeError> {
        Ok(())
    }
    fn size(&self) -> Result<envision::ratatui::layout::Size, SizeError> {
        Err(SizeError)
    }
    fn window_size(&mut self) -> Result<envision::ratatui::backend::WindowSize, SizeError> {
        Err(SizeError)
    }
    fn flush(&mut self) -> Result<(), SizeError> {
        Ok(())
    }
}

// Test 7: DualBackend constructors return the primary's own error type.
#[test]
fn dual_backend_constructors_return_primary_error() {
    let auto: Result<DualBackend<NoSize>, SizeError> =
        DualBackend::with_auto_capture(NoSize(CaptureBackend::new(1, 1)));
    assert!(auto.is_err());
    let built: Result<DualBackend<NoSize>, SizeError> =
        envision::adapter::DualBackendBuilder::new(NoSize(CaptureBackend::new(1, 1))).build();
    assert!(built.is_err());
}
