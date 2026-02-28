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

#[test]
fn test_frame_snapshot_row_content_out_of_bounds() {
    let backend = CaptureBackend::new(10, 5);
    let snapshot = backend.snapshot();

    // Row beyond height should return empty string
    assert_eq!(snapshot.row_content(10), "");
    assert_eq!(snapshot.row_content(100), "");
}

#[test]
fn test_frame_snapshot_to_plain() {
    let mut backend = CaptureBackend::new(5, 2);
    for (i, c) in "Hello".chars().enumerate() {
        if let Some(cell) = backend.cell_mut(i as u16, 0) {
            cell.set_char(c);
        }
    }

    let snapshot = backend.snapshot();
    let plain = snapshot.to_plain();

    assert!(plain.contains("Hello"));
}

#[test]
fn test_frame_snapshot_to_ansi() {
    use crate::backend::cell::SerializableColor;

    let mut backend = CaptureBackend::new(5, 2);

    // Set colored content
    if let Some(cell) = backend.cell_mut(0, 0) {
        cell.set_char('R');
        cell.fg = SerializableColor::Red;
    }
    if let Some(cell) = backend.cell_mut(1, 0) {
        cell.set_char('G');
        cell.fg = SerializableColor::Green;
        cell.bg = SerializableColor::Blue;
    }

    let snapshot = backend.snapshot();
    let ansi = snapshot.to_ansi();

    assert!(ansi.contains("R"));
    assert!(ansi.contains("\x1b[31m")); // Red fg
    assert!(ansi.contains("\x1b[32m")); // Green fg
    assert!(ansi.contains("\x1b[44m")); // Blue bg
    assert!(ansi.contains("\x1b[0m")); // Reset
}

#[test]
fn test_frame_snapshot_to_ansi_with_modifiers() {
    use crate::backend::cell::SerializableModifier;

    let mut backend = CaptureBackend::new(5, 1);

    if let Some(cell) = backend.cell_mut(0, 0) {
        cell.set_char('B');
        cell.modifiers = SerializableModifier {
            bold: true,
            ..Default::default()
        };
    }

    let snapshot = backend.snapshot();
    let ansi = snapshot.to_ansi();

    assert!(ansi.contains("B"));
    assert!(ansi.contains("\x1b[1m")); // Bold
}

#[test]
fn test_frame_snapshot_contains_text() {
    let mut backend = CaptureBackend::new(10, 3);

    for (i, c) in "Hello".chars().enumerate() {
        if let Some(cell) = backend.cell_mut(i as u16, 1) {
            cell.set_char(c);
        }
    }

    let snapshot = backend.snapshot();
    assert!(snapshot.contains_text("Hello"));
    assert!(!snapshot.contains_text("Goodbye"));
}

#[test]
fn test_cell_mut_out_of_bounds() {
    let mut backend = CaptureBackend::new(10, 10);
    assert!(backend.cell_mut(100, 100).is_none());
    assert!(backend.cell_mut(10, 0).is_none()); // At boundary
    assert!(backend.cell_mut(0, 10).is_none()); // At boundary
}

#[test]
fn test_cells_accessor() {
    let backend = CaptureBackend::new(5, 3);
    let cells = backend.cells();
    assert_eq!(cells.len(), 15);
}

#[test]
fn test_row_content_out_of_bounds() {
    let backend = CaptureBackend::new(10, 5);
    assert_eq!(backend.row_content(10), "");
    assert_eq!(backend.row_content(100), "");
}

#[test]
fn test_content_lines() {
    let mut backend = CaptureBackend::new(5, 3);

    for (i, c) in "AAA".chars().enumerate() {
        if let Some(cell) = backend.cell_mut(i as u16, 0) {
            cell.set_char(c);
        }
    }

    let lines = backend.content_lines();
    assert_eq!(lines.len(), 3);
    assert!(lines[0].starts_with("AAA"));
}

#[test]
fn test_to_json() {
    let mut backend = CaptureBackend::new(3, 2);
    if let Some(cell) = backend.cell_mut(0, 0) {
        cell.set_char('X');
    }

    let json = backend.to_json();
    assert!(json.starts_with("{"));
    assert!(json.contains("\"width\":3"));
}

#[test]
fn test_to_json_pretty() {
    let backend = CaptureBackend::new(3, 2);
    let json = backend.to_json_pretty();

    // Pretty JSON has newlines and indentation
    assert!(json.contains("\n"));
    assert!(json.contains("  "));
}

#[test]
fn test_clear_region_all() {
    let mut backend = CaptureBackend::new(5, 3);

    // Set content
    if let Some(cell) = backend.cell_mut(2, 1) {
        cell.set_char('X');
    }

    backend.clear_region(ClearType::All).unwrap();
    assert_eq!(backend.cell(2, 1).unwrap().symbol(), " ");
}

#[test]
fn test_clear_region_after_cursor() {
    let mut backend = CaptureBackend::new(5, 3);

    // Fill with X
    for y in 0..3 {
        for x in 0..5 {
            if let Some(cell) = backend.cell_mut(x, y) {
                cell.set_char('X');
            }
        }
    }

    // Set cursor in middle
    backend.set_cursor_position(Position::new(2, 1)).unwrap();

    // Clear after cursor
    backend.clear_region(ClearType::AfterCursor).unwrap();

    // Before cursor should still be X
    assert_eq!(backend.cell(0, 0).unwrap().symbol(), "X");
    assert_eq!(backend.cell(1, 1).unwrap().symbol(), "X");

    // After cursor should be cleared
    assert_eq!(backend.cell(3, 1).unwrap().symbol(), " ");
    assert_eq!(backend.cell(0, 2).unwrap().symbol(), " ");
}

#[test]
fn test_clear_region_before_cursor() {
    let mut backend = CaptureBackend::new(5, 3);

    // Fill with X
    for y in 0..3 {
        for x in 0..5 {
            if let Some(cell) = backend.cell_mut(x, y) {
                cell.set_char('X');
            }
        }
    }

    // Set cursor in middle
    backend.set_cursor_position(Position::new(2, 1)).unwrap();

    // Clear before cursor
    backend.clear_region(ClearType::BeforeCursor).unwrap();

    // Before cursor should be cleared
    assert_eq!(backend.cell(0, 0).unwrap().symbol(), " ");
    assert_eq!(backend.cell(1, 1).unwrap().symbol(), " ");

    // At and after cursor should still be X
    assert_eq!(backend.cell(2, 1).unwrap().symbol(), "X");
    assert_eq!(backend.cell(3, 1).unwrap().symbol(), "X");
}

#[test]
fn test_clear_region_current_line() {
    let mut backend = CaptureBackend::new(5, 3);

    // Fill with X
    for y in 0..3 {
        for x in 0..5 {
            if let Some(cell) = backend.cell_mut(x, y) {
                cell.set_char('X');
            }
        }
    }

    // Set cursor on row 1
    backend.set_cursor_position(Position::new(2, 1)).unwrap();

    // Clear current line
    backend.clear_region(ClearType::CurrentLine).unwrap();

    // Row 0 should still be X
    assert_eq!(backend.cell(0, 0).unwrap().symbol(), "X");

    // Row 1 should be cleared
    assert_eq!(backend.cell(0, 1).unwrap().symbol(), " ");
    assert_eq!(backend.cell(4, 1).unwrap().symbol(), " ");

    // Row 2 should still be X
    assert_eq!(backend.cell(0, 2).unwrap().symbol(), "X");
}

#[test]
fn test_clear_region_until_newline() {
    let mut backend = CaptureBackend::new(5, 3);

    // Fill with X
    for y in 0..3 {
        for x in 0..5 {
            if let Some(cell) = backend.cell_mut(x, y) {
                cell.set_char('X');
            }
        }
    }

    // Set cursor in middle of row 1
    backend.set_cursor_position(Position::new(2, 1)).unwrap();

    // Clear until newline (rest of the line)
    backend.clear_region(ClearType::UntilNewLine).unwrap();

    // Before cursor on same line should still be X
    assert_eq!(backend.cell(0, 1).unwrap().symbol(), "X");
    assert_eq!(backend.cell(1, 1).unwrap().symbol(), "X");

    // At and after cursor on same line should be cleared
    assert_eq!(backend.cell(2, 1).unwrap().symbol(), " ");
    assert_eq!(backend.cell(4, 1).unwrap().symbol(), " ");

    // Other rows should still be X
    assert_eq!(backend.cell(0, 0).unwrap().symbol(), "X");
    assert_eq!(backend.cell(0, 2).unwrap().symbol(), "X");
}

#[test]
fn test_window_size() {
    let mut backend = CaptureBackend::new(80, 24);
    let window = backend.window_size().unwrap();

    assert_eq!(window.columns_rows.width, 80);
    assert_eq!(window.columns_rows.height, 24);
    // Pixels are calculated as 8x16 per cell
    assert_eq!(window.pixels.width, 640);
    assert_eq!(window.pixels.height, 384);
}

#[test]
fn test_width_and_height() {
    let backend = CaptureBackend::new(100, 50);
    assert_eq!(backend.width(), 100);
    assert_eq!(backend.height(), 50);
}

#[test]
fn test_is_cursor_visible() {
    let mut backend = CaptureBackend::new(80, 24);
    assert!(backend.is_cursor_visible());

    backend.hide_cursor().unwrap();
    assert!(!backend.is_cursor_visible());

    backend.show_cursor().unwrap();
    assert!(backend.is_cursor_visible());
}

#[test]
fn test_cursor_position_accessor() {
    let mut backend = CaptureBackend::new(80, 24);
    backend.set_cursor_position(Position::new(15, 10)).unwrap();

    assert_eq!(backend.cursor_position(), Position::new(15, 10));
}

#[test]
fn test_frame_diff_display() {
    let mut backend = CaptureBackend::with_history(10, 5, 2);

    // Initial frame
    backend.flush().unwrap();

    // Modify cells and cursor
    if let Some(cell) = backend.cell_mut(3, 2) {
        cell.set_char('A');
    }
    backend.set_cursor_position(Position::new(5, 3)).unwrap();

    // Get diff
    let diff = backend.diff_from_previous().unwrap();
    let display = format!("{}", diff);

    assert!(display.contains("Frame 0 → 1 changes:"));
    assert!(display.contains("[Cursor moved]"));
    assert!(display.contains("(3,2)"));
}

#[test]
fn test_frame_diff_display_size_changed() {
    // Create a diff with size_changed = true
    let diff = FrameDiff {
        from_frame: 0,
        to_frame: 1,
        changed_cells: vec![],
        size_changed: true,
        cursor_moved: false,
    };

    let display = format!("{}", diff);
    assert!(display.contains("[Size changed]"));
}

#[test]
fn test_diff_from_no_history() {
    let backend = CaptureBackend::new(10, 5);
    // No history, so diff_from_previous should return None
    assert!(backend.diff_from_previous().is_none());
}

#[test]
fn test_draw_out_of_bounds() {
    let mut backend = CaptureBackend::new(5, 5);

    // Create a cell
    let mut cell = Cell::default();
    cell.set_char('X');

    // Draw at valid position
    let content = vec![(2_u16, 2_u16, &cell)];
    backend.draw(content.into_iter()).unwrap();
    assert_eq!(backend.cell(2, 2).unwrap().symbol(), "X");

    // Draw at out-of-bounds position (should be ignored)
    let content = vec![(100_u16, 100_u16, &cell)];
    backend.draw(content.into_iter()).unwrap();
    // No crash, operation was ignored
}

#[test]
fn test_frame_diff_has_changes() {
    let diff_empty = FrameDiff {
        from_frame: 0,
        to_frame: 1,
        changed_cells: vec![],
        size_changed: false,
        cursor_moved: false,
    };
    assert!(!diff_empty.has_changes());

    let diff_with_cursor = FrameDiff {
        from_frame: 0,
        to_frame: 1,
        changed_cells: vec![],
        size_changed: false,
        cursor_moved: true,
    };
    assert!(diff_with_cursor.has_changes());

    let diff_with_size = FrameDiff {
        from_frame: 0,
        to_frame: 1,
        changed_cells: vec![],
        size_changed: true,
        cursor_moved: false,
    };
    assert!(diff_with_size.has_changes());
}

#[test]
fn test_snapshot_with_truncated_cells() {
    // Test edge case where cells might be truncated (via deserialization)
    // This simulates corrupted data or version mismatch scenarios
    let modifiers = r#"{"bold":false,"dim":false,"italic":false,"underlined":false,"slow_blink":false,"rapid_blink":false,"reversed":false,"hidden":false,"crossed_out":false}"#;
    let cell = format!(
        r#"{{"symbol":" ","fg":"reset","bg":"reset","modifiers":{},"underline_color":null,"last_modified_frame":0,"skip":false}}"#,
        modifiers
    );
    let json = format!(
        r#"{{"frame":0,"size":[5,3],"cursor":{{"position":[0,0],"visible":true}},"cells":[{},{},{},{},{}]}}"#,
        cell, cell, cell, cell, cell
    );

    let modified_snapshot: FrameSnapshot = serde_json::from_str(&json).unwrap();

    // row_content should handle truncated cells gracefully
    let row = modified_snapshot.row_content(2);
    assert_eq!(row, ""); // Row 2 starts at index 10, but we only have 5 cells
}

#[test]
fn test_to_ansi_method() {
    let mut backend = CaptureBackend::new(5, 1);
    if let Some(cell) = backend.cell_mut(0, 0) {
        cell.set_char('T');
    }

    let ansi = backend.to_ansi();
    assert!(ansi.contains("T"));
}
