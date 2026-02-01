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
pub struct DualBackend<P: Backend<Error = io::Error>> {
    /// The primary backend (real terminal)
    primary: P,

    /// The capture backend for inspection
    capture: CaptureBackend,

    /// Whether to synchronize sizes
    sync_sizes: bool,
}

impl<P: Backend<Error = io::Error>> DualBackend<P> {
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

impl<P: Backend<Error = io::Error>> Backend for DualBackend<P> {
    type Error = io::Error;

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
pub struct DualBackendBuilder<P: Backend<Error = io::Error>> {
    primary: P,
    width: Option<u16>,
    height: Option<u16>,
    history_capacity: usize,
    sync_sizes: bool,
}

#[allow(dead_code)]
impl<P: Backend<Error = io::Error>> DualBackendBuilder<P> {
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
mod tests {
    use super::*;

    #[test]
    fn test_dual_backend_new() {
        let primary = CaptureBackend::new(80, 24);
        let capture = CaptureBackend::new(80, 24);
        let dual = DualBackend::new(primary, capture);

        assert_eq!(dual.capture().width(), 80);
        assert_eq!(dual.capture().height(), 24);
    }

    #[test]
    fn test_dual_backend_draw() {
        let primary = CaptureBackend::new(10, 5);
        let capture = CaptureBackend::new(10, 5);
        let mut dual = DualBackend::new(primary, capture);

        // Create a cell
        let mut cell = Cell::default();
        cell.set_char('X');

        // Draw to dual backend
        let content = vec![(5_u16, 2_u16, &cell)];
        dual.draw(content.into_iter()).unwrap();

        // Both backends should have the content
        assert_eq!(dual.primary().cell(5, 2).unwrap().symbol(), "X");
        assert_eq!(dual.capture().cell(5, 2).unwrap().symbol(), "X");
    }

    #[test]
    fn test_dual_backend_cursor() {
        let primary = CaptureBackend::new(80, 24);
        let capture = CaptureBackend::new(80, 24);
        let mut dual = DualBackend::new(primary, capture);

        dual.set_cursor_position(Position::new(10, 5)).unwrap();
        assert_eq!(dual.get_cursor_position().unwrap(), Position::new(10, 5));
        assert_eq!(dual.capture().cursor_position(), Position::new(10, 5));

        dual.hide_cursor().unwrap();
        assert!(!dual.capture().is_cursor_visible());

        dual.show_cursor().unwrap();
        assert!(dual.capture().is_cursor_visible());
    }

    #[test]
    fn test_dual_backend_clear() {
        let primary = CaptureBackend::new(10, 5);
        let capture = CaptureBackend::new(10, 5);
        let mut dual = DualBackend::new(primary, capture);

        // Set some content
        let mut cell = Cell::default();
        cell.set_char('A');
        dual.draw(vec![(3_u16, 2_u16, &cell)].into_iter()).unwrap();

        // Clear
        dual.clear().unwrap();

        assert_eq!(dual.capture().cell(3, 2).unwrap().symbol(), " ");
    }

    #[test]
    fn test_dual_backend_flush() {
        let primary = CaptureBackend::new(80, 24);
        let capture = CaptureBackend::new(80, 24);
        let mut dual = DualBackend::new(primary, capture);

        assert_eq!(dual.frame_count(), 0);

        dual.flush().unwrap();
        assert_eq!(dual.frame_count(), 1);

        dual.flush().unwrap();
        assert_eq!(dual.frame_count(), 2);
    }

    #[test]
    fn test_dual_backend_size() {
        let primary = CaptureBackend::new(120, 40);
        let capture = CaptureBackend::new(80, 24);
        let dual = DualBackend::new(primary, capture);

        // Size comes from primary
        let size = dual.size().unwrap();
        assert_eq!(size.width, 120);
        assert_eq!(size.height, 40);
    }

    #[test]
    fn test_dual_backend_text_queries() {
        let primary = CaptureBackend::new(20, 5);
        let capture = CaptureBackend::new(20, 5);
        let mut dual = DualBackend::new(primary, capture);

        // Set some text
        for (i, c) in "Hello".chars().enumerate() {
            let mut cell = Cell::default();
            cell.set_char(c);
            dual.draw(vec![(i as u16, 0_u16, &cell)].into_iter())
                .unwrap();
        }

        assert!(dual.contains_text("Hello"));
        assert!(!dual.contains_text("Goodbye"));
        assert!(dual.captured_text().contains("Hello"));
    }

    #[test]
    fn test_dual_backend_into_inner() {
        let primary = CaptureBackend::new(80, 24);
        let capture = CaptureBackend::new(80, 24);
        let dual = DualBackend::new(primary, capture);

        let (p, c) = dual.into_inner();
        assert_eq!(p.width(), 80);
        assert_eq!(c.width(), 80);
    }

    #[test]
    fn test_dual_backend_builder() {
        let primary = CaptureBackend::new(80, 24);
        let dual = DualBackendBuilder::new(primary)
            .capture_size(100, 50)
            .with_history(5)
            .build()
            .unwrap();

        assert_eq!(dual.capture().width(), 100);
        assert_eq!(dual.capture().height(), 50);
    }

    #[test]
    fn test_dual_backend_with_auto_capture() {
        let primary = CaptureBackend::new(80, 24);
        let dual = DualBackend::with_auto_capture(primary).unwrap();

        assert_eq!(dual.capture().width(), 80);
        assert_eq!(dual.capture().height(), 24);
    }

    #[test]
    fn test_dual_backend_with_history() {
        let primary = CaptureBackend::new(80, 24);
        let capture = CaptureBackend::with_history(80, 24, 5);
        let dual = DualBackend::with_history(primary, capture, true);

        assert_eq!(dual.capture().width(), 80);
    }

    #[test]
    fn test_dual_backend_disable_sync_sizes() {
        let primary = CaptureBackend::new(80, 24);
        let capture = CaptureBackend::new(80, 24);
        let dual = DualBackend::new(primary, capture).disable_sync_sizes();

        // sync_sizes is now false, we can verify by checking construction worked
        assert_eq!(dual.capture().width(), 80);
    }

    #[test]
    fn test_dual_backend_primary_mut() {
        let primary = CaptureBackend::new(80, 24);
        let capture = CaptureBackend::new(80, 24);
        let mut dual = DualBackend::new(primary, capture);

        // Access mutable reference to primary
        let primary = dual.primary_mut();
        assert_eq!(primary.width(), 80);
    }

    #[test]
    fn test_dual_backend_capture_mut() {
        let primary = CaptureBackend::new(80, 24);
        let capture = CaptureBackend::new(80, 24);
        let mut dual = DualBackend::new(primary, capture);

        // Access mutable reference to capture and modify
        let capture = dual.capture_mut();
        capture.set_cursor_position(Position::new(5, 5)).unwrap();

        assert_eq!(dual.capture().cursor_position(), Position::new(5, 5));
    }

    #[test]
    fn test_dual_backend_captured_ansi() {
        use ratatui::style::Color;

        let primary = CaptureBackend::new(20, 5);
        let capture = CaptureBackend::new(20, 5);
        let mut dual = DualBackend::new(primary, capture);

        // Set some colored text
        let mut cell = Cell::default();
        cell.set_char('R');
        cell.set_fg(Color::Red);
        dual.draw(vec![(0_u16, 0_u16, &cell)].into_iter()).unwrap();

        let ansi = dual.captured_ansi();
        assert!(ansi.contains("R"));
        // ANSI output should include escape codes for red
        assert!(ansi.contains("\x1b[31m"));
    }

    #[test]
    fn test_dual_backend_clear_region() {
        let primary = CaptureBackend::new(10, 5);
        let capture = CaptureBackend::new(10, 5);
        let mut dual = DualBackend::new(primary, capture);

        // Set some content
        let mut cell = Cell::default();
        cell.set_char('X');
        dual.draw(vec![(5_u16, 2_u16, &cell)].into_iter()).unwrap();

        // Clear using ClearType::All
        dual.clear_region(ClearType::All).unwrap();

        assert_eq!(dual.capture().cell(5, 2).unwrap().symbol(), " ");
    }

    #[test]
    fn test_dual_backend_window_size() {
        let primary = CaptureBackend::new(80, 24);
        let capture = CaptureBackend::new(80, 24);
        let mut dual = DualBackend::new(primary, capture);

        let window = dual.window_size().unwrap();
        assert_eq!(window.columns_rows.width, 80);
        assert_eq!(window.columns_rows.height, 24);
    }

    #[test]
    fn test_dual_backend_builder_no_sync_sizes() {
        let primary = CaptureBackend::new(80, 24);
        let dual = DualBackendBuilder::new(primary)
            .no_sync_sizes()
            .build()
            .unwrap();

        // sync_sizes is false but we can verify builder worked
        assert_eq!(dual.capture().width(), 80);
    }

    #[test]
    fn test_dual_backend_builder_no_history() {
        let primary = CaptureBackend::new(80, 24);
        let dual = DualBackendBuilder::new(primary)
            .capture_size(60, 20)
            .build()
            .unwrap();

        // No history, just custom size
        assert_eq!(dual.capture().width(), 60);
        assert_eq!(dual.capture().height(), 20);
    }
}
