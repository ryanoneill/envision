//! Snapshot testing support.

use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::annotation::AnnotationRegistry;
use crate::backend::FrameSnapshot;

/// Format for snapshot output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SnapshotFormat {
    /// Plain text (screen content only)
    #[default]
    Plain,

    /// ANSI-colored text
    Ansi,

    /// JSON with full metadata
    Json,

    /// JSON (pretty-printed)
    JsonPretty,
}

/// A complete snapshot of UI state.
///
/// Includes both the rendered frame and annotation data.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    /// The captured frame data
    pub frame: FrameSnapshot,

    /// Annotations for this frame
    pub annotations: AnnotationRegistry,
}

impl Snapshot {
    /// Creates a new snapshot.
    pub fn new(frame: FrameSnapshot, annotations: AnnotationRegistry) -> Self {
        Self { frame, annotations }
    }

    /// Returns the plain text representation.
    pub fn to_plain(&self) -> String {
        self.frame.to_plain()
    }

    /// Returns the ANSI-colored representation.
    pub fn to_ansi(&self) -> String {
        self.frame.to_ansi()
    }

    /// Returns the JSON representation.
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }

    /// Returns the pretty-printed JSON representation.
    pub fn to_json_pretty(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    /// Formats the snapshot according to the specified format.
    pub fn format(&self, format: SnapshotFormat) -> String {
        match format {
            SnapshotFormat::Plain => self.to_plain(),
            SnapshotFormat::Ansi => self.to_ansi(),
            SnapshotFormat::Json => self.to_json().unwrap_or_default(),
            SnapshotFormat::JsonPretty => self.to_json_pretty().unwrap_or_default(),
        }
    }

    /// Writes the snapshot to a file.
    pub fn write_to_file(&self, path: impl AsRef<Path>, format: SnapshotFormat) -> io::Result<()> {
        let content = self.format(format);
        std::fs::write(path, content)
    }

    /// Loads a snapshot from a JSON file.
    pub fn load_from_file(path: impl AsRef<Path>) -> io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    /// Compares this snapshot to another.
    pub fn diff(&self, other: &Snapshot) -> SnapshotDiff {
        SnapshotDiff::compute(self, other)
    }

    /// Returns true if this snapshot matches another exactly.
    pub fn matches(&self, other: &Snapshot) -> bool {
        self.to_plain() == other.to_plain()
    }

    /// Returns a formatted tree of annotations.
    pub fn annotation_tree(&self) -> String {
        self.annotations.format_tree()
    }

    /// Returns the number of annotations.
    pub fn annotation_count(&self) -> usize {
        self.annotations.len()
    }
}

/// Difference between two snapshots.
#[derive(Debug, Clone)]
pub struct SnapshotDiff {
    /// Lines that differ
    pub changed_lines: Vec<LineDiff>,

    /// Whether the annotations differ
    pub annotations_differ: bool,

    /// Number of changed lines
    pub changes: usize,
}

/// A single line difference.
#[derive(Debug, Clone)]
pub struct LineDiff {
    /// Line number (0-indexed)
    pub line: usize,

    /// Content in the first snapshot
    pub left: String,

    /// Content in the second snapshot
    pub right: String,
}

impl SnapshotDiff {
    /// Computes the diff between two snapshots.
    pub fn compute(left: &Snapshot, right: &Snapshot) -> Self {
        let left_plain = left.to_plain();
        let right_plain = right.to_plain();

        let left_lines: Vec<&str> = left_plain.lines().collect();
        let right_lines: Vec<&str> = right_plain.lines().collect();

        let max_lines = left_lines.len().max(right_lines.len());
        let mut changed_lines = Vec::new();

        for i in 0..max_lines {
            let l = left_lines.get(i).copied().unwrap_or("");
            let r = right_lines.get(i).copied().unwrap_or("");

            if l != r {
                changed_lines.push(LineDiff {
                    line: i,
                    left: l.to_string(),
                    right: r.to_string(),
                });
            }
        }

        let annotations_differ = left.annotations.format_tree() != right.annotations.format_tree();

        Self {
            changes: changed_lines.len(),
            changed_lines,
            annotations_differ,
        }
    }

    /// Returns true if the snapshots are identical.
    pub fn is_empty(&self) -> bool {
        self.changes == 0 && !self.annotations_differ
    }

    /// Formats the diff for display.
    pub fn format(&self) -> String {
        let mut output = String::new();

        if self.changed_lines.is_empty() && !self.annotations_differ {
            output.push_str("No differences\n");
            return output;
        }

        if !self.changed_lines.is_empty() {
            output.push_str(&format!("Changed lines ({}):\n", self.changes));
            for diff in &self.changed_lines {
                output.push_str(&format!("  Line {}:\n", diff.line + 1));
                output.push_str(&format!("    - {}\n", diff.left));
                output.push_str(&format!("    + {}\n", diff.right));
            }
        }

        if self.annotations_differ {
            output.push_str("Annotations differ\n");
        }

        output
    }
}

/// Asserts that two snapshots match.
///
/// # Panics
///
/// Panics with a diff if the snapshots differ.
#[allow(dead_code)]
pub fn assert_snapshot_eq(left: &Snapshot, right: &Snapshot) {
    let diff = left.diff(right);
    if !diff.is_empty() {
        panic!("Snapshots differ:\n{}", diff.format());
    }
}

/// Asserts that a snapshot matches an expected string.
///
/// # Panics
///
/// Panics if the snapshot's plain text doesn't match.
#[allow(dead_code)]
pub fn assert_snapshot_text(snapshot: &Snapshot, expected: &str) {
    let actual = snapshot.to_plain();
    if actual != expected {
        panic!(
            "Snapshot text differs:\n\nExpected:\n{}\n\nActual:\n{}",
            expected, actual
        );
    }
}

/// Helper for snapshot testing with file storage.
#[derive(Debug)]
#[allow(dead_code)]
pub struct SnapshotTest {
    /// Directory for snapshot files
    pub snapshot_dir: std::path::PathBuf,

    /// Format for snapshot files
    pub format: SnapshotFormat,

    /// Whether to update snapshots
    pub update: bool,
}

#[allow(dead_code)]
impl SnapshotTest {
    /// Creates a new snapshot test helper.
    pub fn new(snapshot_dir: impl AsRef<Path>) -> Self {
        Self {
            snapshot_dir: snapshot_dir.as_ref().to_path_buf(),
            format: SnapshotFormat::Plain,
            update: false,
        }
    }

    /// Sets the snapshot format.
    pub fn with_format(mut self, format: SnapshotFormat) -> Self {
        self.format = format;
        self
    }

    /// Enables update mode (overwrites existing snapshots).
    pub fn with_update(mut self, update: bool) -> Self {
        self.update = update;
        self
    }

    /// Returns the path for a snapshot file.
    pub fn snapshot_path(&self, name: &str) -> std::path::PathBuf {
        let ext = match self.format {
            SnapshotFormat::Plain => "txt",
            SnapshotFormat::Ansi => "ansi",
            SnapshotFormat::Json | SnapshotFormat::JsonPretty => "json",
        };
        self.snapshot_dir.join(format!("{}.{}", name, ext))
    }

    /// Asserts that a snapshot matches the stored version.
    ///
    /// If update mode is enabled, overwrites the stored version.
    pub fn assert(&self, name: &str, snapshot: &Snapshot) -> io::Result<()> {
        let path = self.snapshot_path(name);

        if self.update || !path.exists() {
            std::fs::create_dir_all(&self.snapshot_dir)?;
            snapshot.write_to_file(&path, self.format)?;
            return Ok(());
        }

        let expected = std::fs::read_to_string(&path)?;
        let actual = snapshot.format(self.format);

        if actual != expected {
            // Write actual to a .new file for comparison
            let new_path = path.with_extension(format!(
                "{}.new",
                path.extension().unwrap_or_default().to_string_lossy()
            ));
            std::fs::write(&new_path, &actual)?;

            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Snapshot '{}' differs. New snapshot written to {:?}",
                    name, new_path
                ),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::TestHarness;
    use ratatui::widgets::Paragraph;

    #[test]
    fn test_snapshot_creation() {
        let mut harness = TestHarness::new(40, 5);
        harness.render(|frame| {
            frame.render_widget(Paragraph::new("Test"), frame.area());
        }).unwrap();

        let snapshot = harness.snapshot();
        assert!(snapshot.to_plain().contains("Test"));
    }

    #[test]
    fn test_snapshot_formats() {
        let mut harness = TestHarness::new(20, 2);
        harness.render(|frame| {
            frame.render_widget(Paragraph::new("Hello"), frame.area());
        }).unwrap();

        let snapshot = harness.snapshot();

        // Plain
        let plain = snapshot.format(SnapshotFormat::Plain);
        assert!(plain.contains("Hello"));

        // JSON
        let json = snapshot.format(SnapshotFormat::Json);
        assert!(json.starts_with("{"));

        // JSON Pretty
        let json_pretty = snapshot.format(SnapshotFormat::JsonPretty);
        assert!(json_pretty.contains("\n"));
    }

    #[test]
    fn test_snapshot_diff() {
        let mut harness1 = TestHarness::new(20, 2);
        harness1.render(|frame| {
            frame.render_widget(Paragraph::new("Hello"), frame.area());
        }).unwrap();

        let mut harness2 = TestHarness::new(20, 2);
        harness2.render(|frame| {
            frame.render_widget(Paragraph::new("World"), frame.area());
        }).unwrap();

        let snap1 = harness1.snapshot();
        let snap2 = harness2.snapshot();

        let diff = snap1.diff(&snap2);
        assert!(!diff.is_empty());
        assert!(!diff.changed_lines.is_empty());
    }

    #[test]
    fn test_snapshot_matches() {
        let mut harness1 = TestHarness::new(20, 2);
        harness1.render(|frame| {
            frame.render_widget(Paragraph::new("Same"), frame.area());
        }).unwrap();

        let mut harness2 = TestHarness::new(20, 2);
        harness2.render(|frame| {
            frame.render_widget(Paragraph::new("Same"), frame.area());
        }).unwrap();

        let snap1 = harness1.snapshot();
        let snap2 = harness2.snapshot();

        assert!(snap1.matches(&snap2));
    }

    #[test]
    fn test_snapshot_diff_format() {
        let mut harness1 = TestHarness::new(20, 2);
        harness1.render(|frame| {
            frame.render_widget(Paragraph::new("A"), frame.area());
        }).unwrap();

        let mut harness2 = TestHarness::new(20, 2);
        harness2.render(|frame| {
            frame.render_widget(Paragraph::new("B"), frame.area());
        }).unwrap();

        let diff = harness1.snapshot().diff(&harness2.snapshot());
        let formatted = diff.format();

        assert!(formatted.contains("Changed lines"));
        assert!(formatted.contains("Line 1"));
    }

    #[test]
    fn test_empty_diff() {
        let mut harness = TestHarness::new(20, 2);
        harness.render(|frame| {
            frame.render_widget(Paragraph::new("Test"), frame.area());
        }).unwrap();

        let snap = harness.snapshot();
        let diff = snap.diff(&snap);

        assert!(diff.is_empty());
        assert_eq!(diff.format(), "No differences\n");
    }

    #[test]
    fn test_snapshot_serialization() {
        let mut harness = TestHarness::new(20, 2);
        harness.render(|frame| {
            frame.render_widget(Paragraph::new("Serialize"), frame.area());
        }).unwrap();

        let snapshot = harness.snapshot();
        let json = snapshot.to_json().unwrap();

        let deserialized: Snapshot = serde_json::from_str(&json).unwrap();
        assert!(deserialized.matches(&snapshot));
    }
}
