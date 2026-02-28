//! Dual backend implementation.

use std::io;

use ratatui::backend::{Backend, ClearType, WindowSize};
use ratatui::buffer::Cell;
use ratatui::layout::{Position, Size};

use crate::backend::CaptureBackend;

/// A backend that writes to two backends simultaneously.
///
/// This is useful for:
/// - Debugging: See what's being rendered while running normally
/// - Recording: Capture frames for later analysis
/// - Testing: Run with real UI while collecting test data
///
/// The primary backend is typically a real terminal (e.g., CrosstermBackend),
/// while the secondary backend is a CaptureBackend for inspection.
///
/// # Type Parameters
///
/// * `P` - The primary backend type (typically the real terminal)
///
/// # Example
///
/// ```rust,no_run
/// use envision::adapter::DualBackend;
/// use envision::backend::CaptureBackend;
/// use ratatui::backend::CrosstermBackend;
/// use ratatui::Terminal;
/// use std::io::stdout;
///
/// let capture = CaptureBackend::new(80, 24);
/// let crossterm = CrosstermBackend::new(stdout());
/// let dual = DualBackend::new(crossterm, capture);
///
/// let mut terminal = Terminal::new(dual).unwrap();
/// ```
pub struct DualBackend<P: Backend> {
    /// The primary backend (real terminal)
    primary: P,

    /// The capture backend for inspection
    capture: CaptureBackend,

    /// Whether to synchronize sizes
    sync_sizes: bool,
}

impl<P: Backend> DualBackend<P> {
    /// Creates a new dual backend.
    ///
    /// The primary backend is used for actual terminal output,
    /// while the capture backend records all operations.
    pub fn new(primary: P, capture: CaptureBackend) -> Self {
        Self {
            primary,
            capture,
            sync_sizes: true,
        }
    }

    /// Creates a dual backend with automatic capture sizing.
    ///
    /// The capture backend will be created with the same dimensions
    /// as the primary backend.
    pub fn with_auto_capture(primary: P) -> io::Result<Self> {
        let size = primary.size()?;
        let capture = CaptureBackend::new(size.width, size.height);
        Ok(Self {
            primary,
            capture,
            sync_sizes: true,
        })
    }

    /// Creates a dual backend with history tracking.
    pub fn with_history(primary: P, capture: CaptureBackend, sync_sizes: bool) -> Self {
        Self {
            primary,
            capture,
            sync_sizes,
        }
    }

    /// Disables size synchronization between backends.
    ///
    /// By default, the capture backend tries to match the primary's size.
    /// Disable this if you want independent sizing.
    pub fn disable_sync_sizes(mut self) -> Self {
        self.sync_sizes = false;
        self
    }

    /// Returns a reference to the primary backend.
    pub fn primary(&self) -> &P {
        &self.primary
    }

    /// Returns a mutable reference to the primary backend.
    pub fn primary_mut(&mut self) -> &mut P {
        &mut self.primary
    }

    /// Returns a reference to the capture backend.
    pub fn capture(&self) -> &CaptureBackend {
        &self.capture
    }

    /// Returns a mutable reference to the capture backend.
    pub fn capture_mut(&mut self) -> &mut CaptureBackend {
        &mut self.capture
    }

    /// Splits the dual backend into its components.
    pub fn into_inner(self) -> (P, CaptureBackend) {
        (self.primary, self.capture)
    }

    /// Returns the captured content as a string.
    pub fn captured_text(&self) -> String {
        self.capture.to_string()
    }

    /// Returns the captured content with ANSI colors.
    pub fn captured_ansi(&self) -> String {
        self.capture.to_ansi()
    }

    /// Returns true if the captured content contains the given text.
    pub fn contains_text(&self, needle: &str) -> bool {
        self.capture.contains_text(needle)
    }

    /// Returns the current frame number.
    pub fn frame_count(&self) -> u64 {
        self.capture.current_frame()
    }
}

impl<P: Backend> Backend for DualBackend<P> {
    fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        // Collect content so we can iterate twice
        let cells: Vec<_> = content.collect();

        // Draw to primary
        self.primary
            .draw(cells.iter().map(|&(x, y, c)| (x, y, c)))?;

        // Draw to capture
        self.capture.draw(cells.into_iter())?;

        Ok(())
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        self.primary.hide_cursor()?;
        self.capture.hide_cursor()?;
        Ok(())
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        self.primary.show_cursor()?;
        self.capture.show_cursor()?;
        Ok(())
    }

    fn get_cursor_position(&mut self) -> io::Result<Position> {
        // Return primary's cursor position
        self.primary.get_cursor_position()
    }

    fn set_cursor_position<Pos: Into<Position>>(&mut self, position: Pos) -> io::Result<()> {
        let pos = position.into();
        self.primary.set_cursor_position(pos)?;
        self.capture.set_cursor_position(pos)?;
        Ok(())
    }

    fn clear(&mut self) -> io::Result<()> {
        self.primary.clear()?;
        self.capture.clear()?;
        Ok(())
    }

    fn clear_region(&mut self, clear_type: ClearType) -> io::Result<()> {
        self.primary.clear_region(clear_type)?;
        self.capture.clear_region(clear_type)?;
        Ok(())
    }

    fn size(&self) -> io::Result<Size> {
        // Return primary's size
        self.primary.size()
    }

    fn window_size(&mut self) -> io::Result<WindowSize> {
        // Return primary's window size
        self.primary.window_size()
    }

    fn flush(&mut self) -> io::Result<()> {
        self.primary.flush()?;
        self.capture.flush()?;
        Ok(())
    }
}

/// A builder for creating dual backends with various configurations.
#[allow(dead_code)]
pub struct DualBackendBuilder<P: Backend> {
    primary: P,
    width: Option<u16>,
    height: Option<u16>,
    history_capacity: usize,
    sync_sizes: bool,
}

#[allow(dead_code)]
impl<P: Backend> DualBackendBuilder<P> {
    /// Creates a new builder with the given primary backend.
    pub fn new(primary: P) -> Self {
        Self {
            primary,
            width: None,
            height: None,
            history_capacity: 0,
            sync_sizes: true,
        }
    }

    /// Sets the capture backend dimensions.
    ///
    /// If not set, will use the primary backend's size.
    pub fn capture_size(mut self, width: u16, height: u16) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Enables frame history with the given capacity.
    pub fn with_history(mut self, capacity: usize) -> Self {
        self.history_capacity = capacity;
        self
    }

    /// Disables size synchronization.
    pub fn no_sync_sizes(mut self) -> Self {
        self.sync_sizes = false;
        self
    }

    /// Builds the dual backend.
    pub fn build(self) -> io::Result<DualBackend<P>> {
        let size = self.primary.size()?;
        let width = self.width.unwrap_or(size.width);
        let height = self.height.unwrap_or(size.height);

        let capture = if self.history_capacity > 0 {
            CaptureBackend::with_history(width, height, self.history_capacity)
        } else {
            CaptureBackend::new(width, height)
        };

        Ok(DualBackend {
            primary: self.primary,
            capture,
            sync_sizes: self.sync_sizes,
        })
    }
}

#[cfg(test)]
mod tests;
