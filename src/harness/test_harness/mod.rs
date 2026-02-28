//! Core test harness implementation.

use std::io;

use ratatui::Terminal;

use crate::annotation::{with_annotations, AnnotationRegistry, RegionInfo, WidgetType};
use crate::backend::CaptureBackend;
use crate::input::{Event, EventQueue};

use super::assertions::{Assertion, AssertionError, AssertionResult};
use super::snapshot::Snapshot;

/// Test harness for headless TUI testing.
///
/// The harness provides a unified interface for:
/// - Rendering UI to a capture backend
/// - Simulating user input
/// - Querying rendered content and annotations
/// - Making assertions about UI state
///
/// # Example
///
/// ```rust,no_run
/// use envision::harness::TestHarness;
/// use ratatui::widgets::Paragraph;
///
/// let mut harness = TestHarness::new(80, 24);
///
/// harness.render(|frame| {
///     frame.render_widget(Paragraph::new("Test"), frame.area());
/// });
///
/// assert!(harness.contains("Test"));
/// ```
pub struct TestHarness {
    terminal: Terminal<CaptureBackend>,
    events: EventQueue,
    annotations: AnnotationRegistry,
    frame_count: u64,
}

impl TestHarness {
    /// Creates a new test harness with the given dimensions.
    pub fn new(width: u16, height: u16) -> Self {
        let backend = CaptureBackend::new(width, height);
        let terminal = Terminal::new(backend).expect("Failed to create terminal");

        Self {
            terminal,
            events: EventQueue::new(),
            annotations: AnnotationRegistry::new(),
            frame_count: 0,
        }
    }

    /// Returns the width of the terminal.
    pub fn width(&self) -> u16 {
        self.terminal.backend().width()
    }

    /// Returns the height of the terminal.
    pub fn height(&self) -> u16 {
        self.terminal.backend().height()
    }

    /// Returns the current frame count.
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    // -------------------------------------------------------------------------
    // Rendering
    // -------------------------------------------------------------------------

    /// Renders a frame using the provided closure.
    ///
    /// This collects annotations during rendering and increments the frame count.
    pub fn render<F>(&mut self, f: F) -> io::Result<()>
    where
        F: FnOnce(&mut ratatui::Frame),
    {
        self.annotations = with_annotations(|| {
            self.terminal.draw(f).expect("Failed to draw");
        });
        self.frame_count += 1;
        Ok(())
    }

    /// Returns the current terminal content as plain text.
    pub fn screen(&self) -> String {
        self.terminal.backend().to_string()
    }

    /// Returns the content of a specific row.
    pub fn row(&self, y: u16) -> String {
        self.terminal.backend().row_content(y)
    }

    /// Returns a snapshot of the current frame.
    pub fn snapshot(&self) -> Snapshot {
        Snapshot::new(self.terminal.backend().snapshot(), self.annotations.clone())
    }

    /// Returns the cell at the given position, or `None` if out of bounds.
    ///
    /// Use this to assert on cell styling:
    /// ```ignore
    /// let cell = harness.cell_at(5, 3).unwrap();
    /// assert_eq!(cell.fg, SerializableColor::Green);
    /// ```
    pub fn cell_at(&self, x: u16, y: u16) -> Option<&crate::backend::EnhancedCell> {
        self.terminal.backend().cell(x, y)
    }

    /// Returns a reference to the backend.
    pub fn backend(&self) -> &CaptureBackend {
        self.terminal.backend()
    }

    /// Returns a mutable reference to the backend.
    pub fn backend_mut(&mut self) -> &mut CaptureBackend {
        self.terminal.backend_mut()
    }

    // -------------------------------------------------------------------------
    // Input Simulation
    // -------------------------------------------------------------------------

    /// Returns a reference to the event queue.
    pub fn events(&self) -> &EventQueue {
        &self.events
    }

    /// Returns a mutable reference to the event queue.
    pub fn events_mut(&mut self) -> &mut EventQueue {
        &mut self.events
    }

    /// Queues a single event.
    pub fn push_event(&mut self, event: Event) {
        self.events.push(event);
    }

    /// Pops the next event from the queue.
    pub fn pop_event(&mut self) -> Option<Event> {
        self.events.pop()
    }

    /// Types a string as keyboard input.
    pub fn type_str(&mut self, s: &str) {
        self.events.type_str(s);
    }

    /// Simulates pressing Enter.
    pub fn enter(&mut self) {
        self.events.enter();
    }

    /// Simulates pressing Escape.
    pub fn escape(&mut self) {
        self.events.escape();
    }

    /// Simulates pressing Tab.
    pub fn tab(&mut self) {
        self.events.tab();
    }

    /// Simulates `Ctrl+<key>`.
    pub fn ctrl(&mut self, c: char) {
        self.events.ctrl(c);
    }

    /// Simulates a mouse click at the given position.
    pub fn click(&mut self, x: u16, y: u16) {
        self.events.click(x, y);
    }

    /// Clears all pending events.
    pub fn clear_events(&mut self) {
        self.events.clear();
    }

    // -------------------------------------------------------------------------
    // Annotations
    // -------------------------------------------------------------------------

    /// Returns a reference to the annotation registry.
    pub fn annotations(&self) -> &AnnotationRegistry {
        &self.annotations
    }

    /// Returns the region at the given position.
    pub fn region_at(&self, x: u16, y: u16) -> Option<&RegionInfo> {
        self.annotations.region_at(x, y)
    }

    /// Finds regions by id.
    pub fn find_by_id(&self, id: &str) -> Vec<&RegionInfo> {
        self.annotations.find_by_id(id)
    }

    /// Gets the first region with the given id.
    pub fn get_by_id(&self, id: &str) -> Option<&RegionInfo> {
        self.annotations.get_by_id(id)
    }

    /// Finds regions by widget type.
    pub fn find_by_type(&self, widget_type: &WidgetType) -> Vec<&RegionInfo> {
        self.annotations.find_by_type(widget_type)
    }

    /// Returns the currently focused region.
    pub fn focused(&self) -> Option<&RegionInfo> {
        self.annotations.focused_region()
    }

    /// Returns all interactive regions.
    pub fn interactive(&self) -> Vec<&RegionInfo> {
        self.annotations.interactive_regions()
    }

    /// Clicks on a widget by id.
    ///
    /// Returns true if the widget was found and clicked.
    pub fn click_on(&mut self, id: &str) -> bool {
        if let Some(region) = self.annotations.get_by_id(id) {
            let x = region.area.x + region.area.width / 2;
            let y = region.area.y + region.area.height / 2;
            self.click(x, y);
            true
        } else {
            false
        }
    }

    // -------------------------------------------------------------------------
    // Content Queries
    // -------------------------------------------------------------------------

    /// Returns true if the screen contains the given text.
    pub fn contains(&self, needle: &str) -> bool {
        self.terminal.backend().contains_text(needle)
    }

    /// Finds the first position of text on screen.
    pub fn find_text(&self, needle: &str) -> Option<(u16, u16)> {
        self.terminal
            .backend()
            .find_text(needle)
            .first()
            .map(|p| (p.x, p.y))
    }

    /// Finds all positions of text on screen.
    pub fn find_all_text(&self, needle: &str) -> Vec<(u16, u16)> {
        self.terminal
            .backend()
            .find_text(needle)
            .iter()
            .map(|p| (p.x, p.y))
            .collect()
    }

    /// Returns the content within a rectangular region.
    pub fn region_content(&self, x: u16, y: u16, width: u16, height: u16) -> String {
        let mut lines = Vec::new();
        for row in y..y.saturating_add(height) {
            let row_content = self.row(row);
            let start = x as usize;
            let end = (x + width) as usize;
            if start < row_content.len() {
                let end = end.min(row_content.len());
                lines.push(row_content[start..end].to_string());
            }
        }
        lines.join("\n")
    }

    // -------------------------------------------------------------------------
    // Assertions
    // -------------------------------------------------------------------------

    /// Asserts that the screen contains the given text.
    ///
    /// # Panics
    ///
    /// Panics if the text is not found.
    pub fn assert_contains(&self, needle: &str) {
        if !self.contains(needle) {
            panic!(
                "Expected screen to contain '{}', but it was not found.\n\nScreen:\n{}",
                needle,
                self.screen()
            );
        }
    }

    /// Asserts that the screen does not contain the given text.
    ///
    /// # Panics
    ///
    /// Panics if the text is found.
    pub fn assert_not_contains(&self, needle: &str) {
        if self.contains(needle) {
            panic!(
                "Expected screen to NOT contain '{}', but it was found.\n\nScreen:\n{}",
                needle,
                self.screen()
            );
        }
    }

    /// Asserts that a widget with the given id exists.
    ///
    /// # Panics
    ///
    /// Panics if the widget is not found.
    pub fn assert_widget_exists(&self, id: &str) {
        if self.get_by_id(id).is_none() {
            panic!(
                "Expected widget with id '{}' to exist, but it was not found.\n\nAnnotations:\n{}",
                id,
                self.annotations.format_tree()
            );
        }
    }

    /// Asserts that a widget with the given id does not exist.
    ///
    /// # Panics
    ///
    /// Panics if the widget is found.
    pub fn assert_widget_not_exists(&self, id: &str) {
        if self.get_by_id(id).is_some() {
            panic!(
                "Expected widget with id '{}' to NOT exist, but it was found.\n\nAnnotations:\n{}",
                id,
                self.annotations.format_tree()
            );
        }
    }

    /// Asserts that a widget with the given id is focused.
    ///
    /// # Panics
    ///
    /// Panics if the widget is not focused or doesn't exist.
    pub fn assert_focused(&self, id: &str) {
        match self.get_by_id(id) {
            Some(region) if region.annotation.focused => {}
            Some(_) => panic!(
                "Expected widget '{}' to be focused, but it is not.\n\nAnnotations:\n{}",
                id,
                self.annotations.format_tree()
            ),
            None => panic!(
                "Expected widget '{}' to be focused, but it doesn't exist.\n\nAnnotations:\n{}",
                id,
                self.annotations.format_tree()
            ),
        }
    }

    /// Runs an assertion and returns the result.
    pub fn assert(&self, assertion: Assertion) -> AssertionResult {
        assertion.check(self)
    }

    /// Runs multiple assertions and returns all results.
    pub fn assert_all(&self, assertions: Vec<Assertion>) -> Vec<AssertionResult> {
        assertions.into_iter().map(|a| self.assert(a)).collect()
    }

    /// Runs multiple assertions, returning the first failure if any.
    pub fn assert_all_ok(&self, assertions: Vec<Assertion>) -> Result<(), AssertionError> {
        for assertion in assertions {
            self.assert(assertion)?;
        }
        Ok(())
    }
}

impl Default for TestHarness {
    fn default() -> Self {
        Self::new(80, 24)
    }
}

#[cfg(test)]
mod tests;
