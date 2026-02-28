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
