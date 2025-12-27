//! CaptureBackend implementation - the core of Envision.
//!
//! This module provides a ratatui `Backend` implementation that captures
//! all rendering operations for inspection, testing, and headless operation.

use std::fmt;
use std::io;

use ratatui::backend::{Backend, ClearType, WindowSize};
use ratatui::buffer::Cell;
use ratatui::layout::{Position, Size};
use serde::{Deserialize, Serialize};

use super::cell::EnhancedCell;
use super::output::OutputFormat;

/// A backend that captures rendered frames for inspection and testing.
///
/// `CaptureBackend` implements ratatui's `Backend` trait, storing all rendering
/// operations in an internal buffer that can be inspected, serialized, or
/// converted to various output formats.
///
/// # Features
///
/// - **Frame capture**: All rendering is captured in an inspectable buffer
/// - **History tracking**: Optionally track multiple frames for diff analysis
/// - **Multiple output formats**: Plain text, ANSI colored, JSON, annotated
/// - **Full serialization**: State can be serialized for snapshots
///
/// # Example
///
/// ```rust
/// use envision::backend::CaptureBackend;
/// use ratatui::Terminal;
/// use ratatui::widgets::Paragraph;
///
/// let backend = CaptureBackend::new(80, 24);
/// let mut terminal = Terminal::new(backend).unwrap();
///
/// terminal.draw(|frame| {
///     frame.render_widget(Paragraph::new("Hello!"), frame.area());
/// }).unwrap();
///
/// // Get plain text output
/// println!("{}", terminal.backend());
///
/// // Get with colors (ANSI)
/// println!("{}", terminal.backend().to_ansi());
/// ```
#[derive(Clone, Debug)]
pub struct CaptureBackend {
    /// The captured cells
    cells: Vec<EnhancedCell>,

    /// Width of the terminal
    width: u16,

    /// Height of the terminal
    height: u16,

    /// Current cursor position
    cursor_position: Position,

    /// Whether the cursor is visible
    cursor_visible: bool,

    /// Current frame number (incremented on each flush)
    current_frame: u64,

    /// History of frame snapshots (if enabled)
    history: Vec<FrameSnapshot>,

    /// Maximum history size (0 = disabled)
    history_capacity: usize,
}

/// A snapshot of a single frame's state.
///
/// This captures the complete state of the backend at a point in time,
/// useful for comparing frames or serializing for testing.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrameSnapshot {
    /// The frame number
    pub frame: u64,

    /// Terminal dimensions
    pub size: (u16, u16),

    /// Cursor state
    pub cursor: CursorSnapshot,

    /// All cells in the buffer
    pub cells: Vec<EnhancedCell>,
}

/// Snapshot of cursor state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CursorSnapshot {
    pub position: (u16, u16),
    pub visible: bool,
}

impl CaptureBackend {
    /// Creates a new CaptureBackend with the specified dimensions.
    pub fn new(width: u16, height: u16) -> Self {
        let size = (width as usize) * (height as usize);
        Self {
            cells: vec![EnhancedCell::new(); size],
            width,
            height,
            cursor_position: Position::new(0, 0),
            cursor_visible: true,
            current_frame: 0,
            history: Vec::new(),
            history_capacity: 0,
        }
    }

    /// Creates a new CaptureBackend with history tracking enabled.
    ///
    /// # Arguments
    ///
    /// * `width` - Terminal width in columns
    /// * `height` - Terminal height in rows
    /// * `history_capacity` - Maximum number of frames to keep in history
    pub fn with_history(width: u16, height: u16, history_capacity: usize) -> Self {
        let mut backend = Self::new(width, height);
        backend.history_capacity = history_capacity;
        backend.history = Vec::with_capacity(history_capacity);
        backend
    }

    /// Returns the current frame number.
    pub fn current_frame(&self) -> u64 {
        self.current_frame
    }

    /// Returns the cell at the given position, if valid.
    pub fn cell(&self, x: u16, y: u16) -> Option<&EnhancedCell> {
        if x < self.width && y < self.height {
            Some(&self.cells[self.index_of(x, y)])
        } else {
            None
        }
    }

    /// Returns a mutable reference to the cell at the given position.
    pub fn cell_mut(&mut self, x: u16, y: u16) -> Option<&mut EnhancedCell> {
        if x < self.width && y < self.height {
            let idx = self.index_of(x, y);
            Some(&mut self.cells[idx])
        } else {
            None
        }
    }

    /// Returns all cells as a slice.
    pub fn cells(&self) -> &[EnhancedCell] {
        &self.cells
    }

    /// Returns the content of a specific row as a string.
    pub fn row_content(&self, y: u16) -> String {
        if y >= self.height {
            return String::new();
        }

        let start = self.index_of(0, y);
        let end = start + self.width as usize;
        self.cells[start..end]
            .iter()
            .map(|c| c.symbol())
            .collect()
    }

    /// Returns all content as a vector of row strings.
    pub fn content_lines(&self) -> Vec<String> {
        (0..self.height).map(|y| self.row_content(y)).collect()
    }

    /// Searches for text in the buffer and returns positions where it appears.
    pub fn find_text(&self, needle: &str) -> Vec<Position> {
        let mut positions = Vec::new();
        for y in 0..self.height {
            let row = self.row_content(y);
            for (x, _) in row.match_indices(needle) {
                positions.push(Position::new(x as u16, y));
            }
        }
        positions
    }

    /// Returns true if the buffer contains the given text.
    pub fn contains_text(&self, needle: &str) -> bool {
        !self.find_text(needle).is_empty()
    }

    /// Creates a snapshot of the current state.
    pub fn snapshot(&self) -> FrameSnapshot {
        FrameSnapshot {
            frame: self.current_frame,
            size: (self.width, self.height),
            cursor: CursorSnapshot {
                position: (self.cursor_position.x, self.cursor_position.y),
                visible: self.cursor_visible,
            },
            cells: self.cells.clone(),
        }
    }

    /// Returns the frame history (if history tracking is enabled).
    pub fn history(&self) -> &[FrameSnapshot] {
        &self.history
    }

    /// Computes the diff between the current frame and the previous one.
    pub fn diff_from_previous(&self) -> Option<FrameDiff> {
        self.history.last().map(|prev| self.diff_from(prev))
    }

    /// Computes the diff between the current state and a snapshot.
    pub fn diff_from(&self, previous: &FrameSnapshot) -> FrameDiff {
        let mut changed_cells = Vec::new();

        for y in 0..self.height.min(previous.size.1) {
            for x in 0..self.width.min(previous.size.0) {
                let idx = self.index_of(x, y);
                let prev_idx = (y as usize) * (previous.size.0 as usize) + (x as usize);

                if idx < self.cells.len() && prev_idx < previous.cells.len() {
                    let current = &self.cells[idx];
                    let prev = &previous.cells[prev_idx];

                    if current != prev {
                        changed_cells.push(CellChange {
                            position: (x, y),
                            old: prev.clone(),
                            new: current.clone(),
                        });
                    }
                }
            }
        }

        FrameDiff {
            from_frame: previous.frame,
            to_frame: self.current_frame,
            changed_cells,
            size_changed: (self.width, self.height) != previous.size,
            cursor_moved: (self.cursor_position.x, self.cursor_position.y)
                != previous.cursor.position,
        }
    }

    /// Renders the buffer to a string using the specified format.
    pub fn render(&self, format: OutputFormat) -> String {
        format.render(self)
    }

    /// Renders the buffer with ANSI color codes.
    pub fn to_ansi(&self) -> String {
        self.render(OutputFormat::Ansi)
    }

    /// Renders the buffer as JSON.
    pub fn to_json(&self) -> String {
        self.render(OutputFormat::Json)
    }

    /// Renders the buffer as JSON (pretty-printed).
    pub fn to_json_pretty(&self) -> String {
        self.render(OutputFormat::JsonPretty)
    }

    /// Converts (x, y) coordinates to a linear index.
    fn index_of(&self, x: u16, y: u16) -> usize {
        (y as usize) * (self.width as usize) + (x as usize)
    }

    /// Converts a linear index to (x, y) coordinates.
    #[allow(dead_code)]
    fn pos_of(&self, index: usize) -> (u16, u16) {
        let x = (index % self.width as usize) as u16;
        let y = (index / self.width as usize) as u16;
        (x, y)
    }

    /// Saves the current state to history (if enabled).
    fn save_to_history(&mut self) {
        if self.history_capacity > 0 {
            if self.history.len() >= self.history_capacity {
                self.history.remove(0);
            }
            self.history.push(self.snapshot());
        }
    }

    /// Returns the width of the terminal.
    pub fn width(&self) -> u16 {
        self.width
    }

    /// Returns the height of the terminal.
    pub fn height(&self) -> u16 {
        self.height
    }

    /// Returns whether the cursor is currently visible.
    pub fn is_cursor_visible(&self) -> bool {
        self.cursor_visible
    }

    /// Returns the current cursor position.
    pub fn cursor_position(&self) -> Position {
        self.cursor_position
    }
}

impl Backend for CaptureBackend {
    fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        for (x, y, cell) in content {
            if x < self.width && y < self.height {
                let idx = self.index_of(x, y);
                self.cells[idx] = EnhancedCell::from_ratatui_cell(cell, self.current_frame);
            }
        }
        Ok(())
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        self.cursor_visible = false;
        Ok(())
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        self.cursor_visible = true;
        Ok(())
    }

    fn get_cursor_position(&mut self) -> io::Result<Position> {
        Ok(self.cursor_position)
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, position: P) -> io::Result<()> {
        self.cursor_position = position.into();
        Ok(())
    }

    fn clear(&mut self) -> io::Result<()> {
        for cell in &mut self.cells {
            cell.reset();
        }
        Ok(())
    }

    fn clear_region(&mut self, clear_type: ClearType) -> io::Result<()> {
        let cells_len = self.cells.len();
        let (start, end) = match clear_type {
            ClearType::All => (0, cells_len),
            ClearType::AfterCursor => {
                let start = self.index_of(self.cursor_position.x, self.cursor_position.y);
                (start, cells_len)
            }
            ClearType::BeforeCursor => {
                let end = self.index_of(self.cursor_position.x, self.cursor_position.y);
                (0, end)
            }
            ClearType::CurrentLine => {
                let start = self.index_of(0, self.cursor_position.y);
                let end = start + self.width as usize;
                (start, end)
            }
            ClearType::UntilNewLine => {
                let start = self.index_of(self.cursor_position.x, self.cursor_position.y);
                let end = self.index_of(0, self.cursor_position.y) + self.width as usize;
                (start, end)
            }
        };

        let end = end.min(cells_len);
        for cell in &mut self.cells[start..end] {
            cell.reset();
        }
        Ok(())
    }

    fn size(&self) -> io::Result<Size> {
        Ok(Size::new(self.width, self.height))
    }

    fn window_size(&mut self) -> io::Result<WindowSize> {
        // For a capture backend, we don't have real pixel dimensions
        // Use reasonable defaults (assuming ~8x16 pixels per cell)
        Ok(WindowSize {
            columns_rows: Size::new(self.width, self.height),
            pixels: Size::new(self.width * 8, self.height * 16),
        })
    }

    fn flush(&mut self) -> io::Result<()> {
        self.save_to_history();
        self.current_frame += 1;
        Ok(())
    }
}

impl fmt::Display for CaptureBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.render(OutputFormat::Plain))
    }
}

/// Represents the difference between two frames.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrameDiff {
    /// Frame number of the previous state
    pub from_frame: u64,

    /// Frame number of the current state
    pub to_frame: u64,

    /// Cells that changed between frames
    pub changed_cells: Vec<CellChange>,

    /// Whether the terminal size changed
    pub size_changed: bool,

    /// Whether the cursor moved
    pub cursor_moved: bool,
}

impl FrameDiff {
    /// Returns true if there are any changes.
    pub fn has_changes(&self) -> bool {
        !self.changed_cells.is_empty() || self.size_changed || self.cursor_moved
    }

    /// Returns the number of cells that changed.
    pub fn changed_count(&self) -> usize {
        self.changed_cells.len()
    }
}

impl fmt::Display for FrameDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Frame {} → {} changes:", self.from_frame, self.to_frame)?;

        if self.size_changed {
            writeln!(f, "  [Size changed]")?;
        }
        if self.cursor_moved {
            writeln!(f, "  [Cursor moved]")?;
        }

        for change in &self.changed_cells {
            writeln!(
                f,
                "  ({},{}) \"{}\" → \"{}\"",
                change.position.0,
                change.position.1,
                change.old.symbol(),
                change.new.symbol()
            )?;
        }

        Ok(())
    }
}

/// A single cell change in a diff.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CellChange {
    /// Position of the changed cell
    pub position: (u16, u16),

    /// Previous cell state
    pub old: EnhancedCell,

    /// New cell state
    pub new: EnhancedCell,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_backend() {
        let backend = CaptureBackend::new(80, 24);
        assert_eq!(backend.width, 80);
        assert_eq!(backend.height, 24);
        assert_eq!(backend.cells.len(), 80 * 24);
        assert_eq!(backend.current_frame, 0);
    }

    #[test]
    fn test_size() {
        let backend = CaptureBackend::new(120, 40);
        let size = backend.size().unwrap();
        assert_eq!(size.width, 120);
        assert_eq!(size.height, 40);
    }

    #[test]
    fn test_cell_access() {
        let mut backend = CaptureBackend::new(10, 10);

        // Modify a cell
        if let Some(cell) = backend.cell_mut(5, 5) {
            cell.set_char('X');
        }

        // Read it back
        let cell = backend.cell(5, 5).unwrap();
        assert_eq!(cell.symbol(), "X");

        // Out of bounds returns None
        assert!(backend.cell(100, 100).is_none());
    }

    #[test]
    fn test_row_content() {
        let mut backend = CaptureBackend::new(10, 5);

        // Set some content in row 2
        for (i, c) in "Hello".chars().enumerate() {
            if let Some(cell) = backend.cell_mut(i as u16, 2) {
                cell.set_char(c);
            }
        }

        let row = backend.row_content(2);
        assert!(row.starts_with("Hello"));
    }

    #[test]
    fn test_find_text() {
        let mut backend = CaptureBackend::new(20, 5);

        // Write "Hello" at position (5, 2)
        for (i, c) in "Hello".chars().enumerate() {
            if let Some(cell) = backend.cell_mut(5 + i as u16, 2) {
                cell.set_char(c);
            }
        }

        let positions = backend.find_text("Hello");
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0], Position::new(5, 2));

        assert!(backend.contains_text("Hello"));
        assert!(!backend.contains_text("Goodbye"));
    }

    #[test]
    fn test_cursor_operations() {
        let mut backend = CaptureBackend::new(80, 24);

        backend.set_cursor_position(Position::new(10, 5)).unwrap();
        assert_eq!(backend.get_cursor_position().unwrap(), Position::new(10, 5));

        backend.hide_cursor().unwrap();
        assert!(!backend.cursor_visible);

        backend.show_cursor().unwrap();
        assert!(backend.cursor_visible);
    }

    #[test]
    fn test_clear() {
        let mut backend = CaptureBackend::new(10, 10);

        // Set some content
        if let Some(cell) = backend.cell_mut(5, 5) {
            cell.set_char('X');
        }

        // Clear
        backend.clear().unwrap();

        // Should be reset
        let cell = backend.cell(5, 5).unwrap();
        assert_eq!(cell.symbol(), " ");
    }

    #[test]
    fn test_flush_increments_frame() {
        let mut backend = CaptureBackend::new(80, 24);
        assert_eq!(backend.current_frame(), 0);

        backend.flush().unwrap();
        assert_eq!(backend.current_frame(), 1);

        backend.flush().unwrap();
        assert_eq!(backend.current_frame(), 2);
    }

    #[test]
    fn test_history_tracking() {
        let mut backend = CaptureBackend::with_history(10, 5, 3);

        // Initial state
        backend.flush().unwrap();
        assert_eq!(backend.history().len(), 1);

        // Add more frames
        backend.flush().unwrap();
        backend.flush().unwrap();
        assert_eq!(backend.history().len(), 3);

        // Should cap at capacity
        backend.flush().unwrap();
        assert_eq!(backend.history().len(), 3);
        assert_eq!(backend.history()[0].frame, 1); // Oldest frame removed
    }

    #[test]
    fn test_diff() {
        let mut backend = CaptureBackend::with_history(10, 5, 2);

        // Initial frame
        backend.flush().unwrap();

        // Modify a cell
        if let Some(cell) = backend.cell_mut(3, 2) {
            cell.set_char('A');
        }

        // Get diff
        let diff = backend.diff_from_previous().unwrap();
        assert!(diff.has_changes());
        assert_eq!(diff.changed_count(), 1);
        assert_eq!(diff.changed_cells[0].position, (3, 2));
        assert_eq!(diff.changed_cells[0].new.symbol(), "A");
    }

    #[test]
    fn test_snapshot_serialization() {
        let backend = CaptureBackend::new(10, 5);
        let snapshot = backend.snapshot();

        let json = serde_json::to_string(&snapshot).unwrap();
        let deserialized: FrameSnapshot = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.frame, snapshot.frame);
        assert_eq!(deserialized.size, snapshot.size);
    }

    #[test]
    fn test_display() {
        let mut backend = CaptureBackend::new(5, 2);

        // Set content
        for (i, c) in "Hello".chars().enumerate() {
            if let Some(cell) = backend.cell_mut(i as u16, 0) {
                cell.set_char(c);
            }
        }
        for (i, c) in "World".chars().enumerate() {
            if let Some(cell) = backend.cell_mut(i as u16, 1) {
                cell.set_char(c);
            }
        }

        let output = backend.to_string();
        assert!(output.contains("Hello"));
        assert!(output.contains("World"));
    }
}
