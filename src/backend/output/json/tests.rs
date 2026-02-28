use super::*;
use crate::backend::cell::SerializableColor;

#[test]
fn test_json_render() {
    let mut backend = CaptureBackend::new(10, 3);

    // Set some content
    for (i, c) in "Hello".chars().enumerate() {
        if let Some(cell) = backend.cell_mut(i as u16, 0) {
            cell.set_char(c);
        }
    }

    let output = render(&backend, false);

    // Should be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["size"]["width"], 10);
    assert_eq!(parsed["size"]["height"], 3);
    assert!(parsed["lines"][0].as_str().unwrap().starts_with("Hello"));
}

#[test]
fn test_json_render_pretty() {
    let backend = CaptureBackend::new(5, 2);
    let output = render(&backend, true);

    // Pretty output should have newlines
    assert!(output.contains('\n'));

    // Should still be valid JSON
    let _: serde_json::Value = serde_json::from_str(&output).unwrap();
}

#[test]
fn test_json_styled_cells() {
    let mut backend = CaptureBackend::new(5, 1);

    // Add a styled cell
    if let Some(cell) = backend.cell_mut(0, 0) {
        cell.set_char('X');
        cell.fg = SerializableColor::Red;
        cell.modifiers.bold = true;
    }

    let output = render(&backend, false);
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

    // Should have one styled cell
    let styled = &parsed["styled_cells"];
    assert_eq!(styled.as_array().unwrap().len(), 1);
    assert_eq!(styled[0]["symbol"], "X");
    assert_eq!(styled[0]["bold"], true);
}

#[test]
fn test_json_lines_only() {
    let mut backend = CaptureBackend::new(5, 2);

    for (i, c) in "Hi".chars().enumerate() {
        if let Some(cell) = backend.cell_mut(i as u16, 0) {
            cell.set_char(c);
        }
    }

    let output = render_lines_only(&backend);
    let parsed: Vec<String> = serde_json::from_str(&output).unwrap();

    assert_eq!(parsed.len(), 2);
    assert!(parsed[0].starts_with("Hi"));
}
