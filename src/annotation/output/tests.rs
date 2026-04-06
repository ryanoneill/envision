use super::*;
use crate::annotation::{Annotate, Annotation, with_annotations};
use crate::backend::CaptureBackend;
use ratatui::Terminal;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;

#[test]
fn test_annotated_output_from_backend_and_registry() {
    let backend = CaptureBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                let widget = Annotate::new(
                    Paragraph::new("Click me"),
                    Annotation::button("submit").with_label("Submit"),
                );
                frame.render_widget(widget, frame.area());
            })
            .unwrap();
    });

    let output = AnnotatedOutput::from_backend_and_registry(terminal.backend(), &registry);

    assert!(output.visual.contains("Click me"));
    assert_eq!(output.annotation_count(), 1);
    assert!(!output.is_empty());

    let btn = &output.annotations[0];
    assert_eq!(btn.widget_type, "Button");
    assert_eq!(btn.id, Some("submit".to_string()));
    assert_eq!(btn.label, Some("Submit".to_string()));
    assert!(!btn.focused);
    assert!(!btn.disabled);
    assert_eq!(btn.area.x, 0);
    assert_eq!(btn.area.y, 0);
    assert_eq!(btn.area.width, 40);
    assert_eq!(btn.area.height, 10);
}

#[test]
fn test_annotated_output_empty() {
    let backend = CaptureBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                frame.render_widget(Paragraph::new("No annotations"), frame.area());
            })
            .unwrap();
    });

    let output = AnnotatedOutput::from_backend_and_registry(terminal.backend(), &registry);

    assert!(output.visual.contains("No annotations"));
    assert_eq!(output.annotation_count(), 0);
    assert!(output.is_empty());
}

#[test]
fn test_annotated_output_multiple_annotations() {
    let backend = CaptureBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                let input = Annotate::new(
                    Paragraph::new("John"),
                    Annotation::input("name")
                        .with_label("Name")
                        .with_focus(true),
                )
                .value("John");

                let btn = Annotate::new(
                    Paragraph::new("Submit"),
                    Annotation::button("submit").with_disabled(true),
                );

                frame.render_widget(input, Rect::new(0, 0, 40, 3));
                frame.render_widget(btn, Rect::new(0, 3, 40, 3));
            })
            .unwrap();
    });

    let output = AnnotatedOutput::from_backend_and_registry(terminal.backend(), &registry);

    assert_eq!(output.annotation_count(), 2);

    let name = output.find_by_id("name").unwrap();
    assert_eq!(name.widget_type, "Input");
    assert!(name.focused);
    assert_eq!(name.value, Some("John".to_string()));

    let submit = output.find_by_id("submit").unwrap();
    assert_eq!(submit.widget_type, "Button");
    assert!(submit.disabled);
}

#[test]
fn test_annotated_output_find_by_type() {
    let backend = CaptureBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                let btn1 = Annotate::new(Paragraph::new("OK"), Annotation::button("ok"));
                let btn2 = Annotate::new(Paragraph::new("Cancel"), Annotation::button("cancel"));
                let input = Annotate::new(Paragraph::new(""), Annotation::input("name"));

                frame.render_widget(btn1, Rect::new(0, 0, 10, 1));
                frame.render_widget(btn2, Rect::new(10, 0, 10, 1));
                frame.render_widget(input, Rect::new(0, 2, 20, 1));
            })
            .unwrap();
    });

    let output = AnnotatedOutput::from_backend_and_registry(terminal.backend(), &registry);

    let buttons = output.find_by_type("Button");
    assert_eq!(buttons.len(), 2);

    let inputs = output.find_by_type("Input");
    assert_eq!(inputs.len(), 1);

    let tables = output.find_by_type("Table");
    assert!(tables.is_empty());
}

#[test]
fn test_annotated_output_focused_annotations() {
    let backend = CaptureBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                let focused_input = Annotate::new(
                    Paragraph::new("focused"),
                    Annotation::input("a").with_focus(true),
                );
                let unfocused_input =
                    Annotate::new(Paragraph::new("unfocused"), Annotation::input("b"));

                frame.render_widget(focused_input, Rect::new(0, 0, 20, 1));
                frame.render_widget(unfocused_input, Rect::new(0, 2, 20, 1));
            })
            .unwrap();
    });

    let output = AnnotatedOutput::from_backend_and_registry(terminal.backend(), &registry);

    let focused = output.focused_annotations();
    assert_eq!(focused.len(), 1);
    assert_eq!(focused[0].id, Some("a".to_string()));
}

#[test]
fn test_annotated_output_from_visual_and_registry() {
    let mut registry = AnnotationRegistry::new();
    registry.register(Rect::new(0, 0, 10, 1), Annotation::button("btn"));

    let output =
        AnnotatedOutput::from_visual_and_registry("Custom visual output".to_string(), &registry);

    assert_eq!(output.visual, "Custom visual output");
    assert_eq!(output.annotation_count(), 1);
}

#[test]
fn test_annotated_output_nested_annotations() {
    let backend = CaptureBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                use crate::annotation::AnnotateContainer;

                let container = AnnotateContainer::new(
                    Paragraph::new("Container Content"),
                    Annotation::container("form"),
                );
                frame.render_widget(container, Rect::new(0, 0, 80, 24));
            })
            .unwrap();
    });

    let output = AnnotatedOutput::from_backend_and_registry(terminal.backend(), &registry);

    assert_eq!(output.annotation_count(), 1);
    let container = output.find_by_id("form").unwrap();
    assert_eq!(container.widget_type, "Container");
    assert_eq!(container.depth, 0);
}

#[test]
fn test_annotated_output_metadata() {
    let backend = CaptureBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                let widget = Annotate::new(
                    Paragraph::new("Custom"),
                    Annotation::custom("datepicker", "birth_date")
                        .with_meta("format", "YYYY-MM-DD")
                        .with_meta("min_year", "1900"),
                );
                frame.render_widget(widget, frame.area());
            })
            .unwrap();
    });

    let output = AnnotatedOutput::from_backend_and_registry(terminal.backend(), &registry);

    let widget = output.find_by_id("birth_date").unwrap();
    assert_eq!(widget.widget_type, "datepicker");
    assert_eq!(
        widget.metadata.get("format"),
        Some(&"YYYY-MM-DD".to_string())
    );
    assert_eq!(widget.metadata.get("min_year"), Some(&"1900".to_string()));
}

#[test]
fn test_annotation_area_from_serializable_rect() {
    let rect = SerializableRect::new(10, 20, 30, 40);
    let area: AnnotationArea = rect.into();

    assert_eq!(area.x, 10);
    assert_eq!(area.y, 20);
    assert_eq!(area.width, 30);
    assert_eq!(area.height, 40);
}

#[test]
fn test_annotation_area_equality() {
    let a = AnnotationArea {
        x: 1,
        y: 2,
        width: 3,
        height: 4,
    };
    let b = AnnotationArea {
        x: 1,
        y: 2,
        width: 3,
        height: 4,
    };
    let c = AnnotationArea {
        x: 5,
        y: 6,
        width: 7,
        height: 8,
    };

    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn test_widget_annotation_selected_state() {
    let backend = CaptureBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                let widget = Annotate::new(
                    Paragraph::new("Selected Item"),
                    Annotation::checkbox("option").with_selected(true),
                );
                frame.render_widget(widget, frame.area());
            })
            .unwrap();
    });

    let output = AnnotatedOutput::from_backend_and_registry(terminal.backend(), &registry);

    let checkbox = output.find_by_id("option").unwrap();
    assert!(checkbox.selected);
}

#[test]
fn test_widget_annotation_expanded_state() {
    let backend = CaptureBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                let widget = Annotate::new(
                    Paragraph::new("Expanded section"),
                    Annotation::container("section").with_expanded(true),
                );
                frame.render_widget(widget, frame.area());
            })
            .unwrap();
    });

    let output = AnnotatedOutput::from_backend_and_registry(terminal.backend(), &registry);

    let section = output.find_by_id("section").unwrap();
    assert_eq!(section.expanded, Some(true));
}

#[cfg(feature = "serialization")]
#[test]
fn test_annotated_output_to_json() {
    let backend = CaptureBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                let widget = Annotate::new(
                    Paragraph::new("Hello"),
                    Annotation::button("btn").with_label("Hello Button"),
                );
                frame.render_widget(widget, frame.area());
            })
            .unwrap();
    });

    let output = AnnotatedOutput::from_backend_and_registry(terminal.backend(), &registry);

    let json = output.to_json().unwrap();
    assert!(json.contains("\"widget_type\":\"Button\""));
    assert!(json.contains("\"id\":\"btn\""));
    assert!(json.contains("\"label\":\"Hello Button\""));
    assert!(json.contains("\"visual\""));
}

#[cfg(feature = "serialization")]
#[test]
fn test_annotated_output_to_json_pretty() {
    let backend = CaptureBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                let widget = Annotate::new(
                    Paragraph::new("Pretty"),
                    Annotation::input("field").with_value("test"),
                );
                frame.render_widget(widget, frame.area());
            })
            .unwrap();
    });

    let output = AnnotatedOutput::from_backend_and_registry(terminal.backend(), &registry);

    let json = output.to_json_pretty().unwrap();
    // Pretty JSON has newlines and indentation
    assert!(json.contains('\n'));
    assert!(json.contains("  "));
    assert!(json.contains("\"widget_type\": \"Input\""));
}

#[cfg(feature = "serialization")]
#[test]
fn test_annotated_output_json_roundtrip() {
    let backend = CaptureBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                let widget = Annotate::new(
                    Paragraph::new("Roundtrip"),
                    Annotation::button("ok")
                        .with_label("OK")
                        .with_focus(true)
                        .with_meta("shortcut", "Enter"),
                );
                frame.render_widget(widget, frame.area());
            })
            .unwrap();
    });

    let output = AnnotatedOutput::from_backend_and_registry(terminal.backend(), &registry);

    let json = output.to_json().unwrap();
    let deserialized: AnnotatedOutput = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.visual, output.visual);
    assert_eq!(deserialized.annotations.len(), output.annotations.len());

    let original = &output.annotations[0];
    let restored = &deserialized.annotations[0];
    assert_eq!(original.widget_type, restored.widget_type);
    assert_eq!(original.id, restored.id);
    assert_eq!(original.label, restored.label);
    assert_eq!(original.focused, restored.focused);
    assert_eq!(original.area, restored.area);
    assert_eq!(original.metadata, restored.metadata);
}

#[cfg(feature = "serialization")]
#[test]
fn test_annotated_output_empty_json() {
    let backend = CaptureBackend::new(10, 5);
    let mut terminal = Terminal::new(backend).unwrap();

    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                frame.render_widget(Paragraph::new(""), frame.area());
            })
            .unwrap();
    });

    let output = AnnotatedOutput::from_backend_and_registry(terminal.backend(), &registry);

    let json = output.to_json().unwrap();
    assert!(json.contains("\"annotations\":[]"));
}

#[test]
fn test_find_by_id_not_found() {
    let backend = CaptureBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                let widget = Annotate::new(Paragraph::new("Hello"), Annotation::button("existing"));
                frame.render_widget(widget, frame.area());
            })
            .unwrap();
    });

    let output = AnnotatedOutput::from_backend_and_registry(terminal.backend(), &registry);
    assert!(output.find_by_id("nonexistent").is_none());
}

#[test]
fn test_capture_backend_to_annotated_output() {
    let backend = CaptureBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                let widget = Annotate::new(
                    Paragraph::new("Test"),
                    Annotation::button("btn").with_label("Test Button"),
                );
                frame.render_widget(widget, frame.area());
            })
            .unwrap();
    });

    let output = terminal.backend().to_annotated_output(&registry);

    assert!(output.visual.contains("Test"));
    assert_eq!(output.annotation_count(), 1);

    let btn = output.find_by_id("btn").unwrap();
    assert_eq!(btn.widget_type, "Button");
    assert_eq!(btn.label, Some("Test Button".to_string()));
}

#[cfg(feature = "serialization")]
#[test]
fn test_capture_backend_to_semantic_json() {
    let backend = CaptureBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                let widget = Annotate::new(
                    Paragraph::new("Semantic"),
                    Annotation::input("email")
                        .with_label("Email")
                        .with_value("test@example.com")
                        .with_focus(true),
                );
                frame.render_widget(widget, frame.area());
            })
            .unwrap();
    });

    let json = terminal.backend().to_semantic_json(&registry);

    // Verify it's valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Check structure
    assert!(parsed.get("visual").is_some());
    assert!(parsed.get("annotations").is_some());

    let annotations = parsed["annotations"].as_array().unwrap();
    assert_eq!(annotations.len(), 1);

    let ann = &annotations[0];
    assert_eq!(ann["widget_type"], "Input");
    assert_eq!(ann["id"], "email");
    assert_eq!(ann["label"], "Email");
    assert_eq!(ann["value"], "test@example.com");
    assert_eq!(ann["focused"], true);
}

#[cfg(feature = "serialization")]
#[test]
fn test_capture_backend_semantic_json_multiple_widgets() {
    let backend = CaptureBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                use crate::annotation::AnnotateContainer;

                let container = AnnotateContainer::new(
                    Paragraph::new("Form"),
                    Annotation::container("login-form").with_label("Login Form"),
                );
                frame.render_widget(container, Rect::new(0, 0, 80, 24));

                let username = Annotate::new(
                    Paragraph::new("admin"),
                    Annotation::input("username")
                        .with_label("Username")
                        .with_value("admin"),
                );
                frame.render_widget(username, Rect::new(5, 5, 30, 1));

                let submit = Annotate::new(
                    Paragraph::new("[Submit]"),
                    Annotation::button("submit").with_label("Submit"),
                );
                frame.render_widget(submit, Rect::new(5, 10, 10, 1));
            })
            .unwrap();
    });

    let json = terminal.backend().to_semantic_json(&registry);
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    let annotations = parsed["annotations"].as_array().unwrap();
    assert_eq!(annotations.len(), 3);

    // First should be the container
    assert_eq!(annotations[0]["widget_type"], "Container");
    assert_eq!(annotations[0]["id"], "login-form");

    // Second should be the input
    assert_eq!(annotations[1]["widget_type"], "Input");
    assert_eq!(annotations[1]["id"], "username");
    assert_eq!(annotations[1]["value"], "admin");

    // Third should be the button
    assert_eq!(annotations[2]["widget_type"], "Button");
    assert_eq!(annotations[2]["id"], "submit");
}

#[test]
fn test_widget_annotation_parent_index() {
    let registry = with_annotations(|| {
        use crate::annotation::with_registry;

        with_registry(|reg| {
            reg.open(Rect::new(0, 0, 80, 24), Annotation::container("parent"));
            reg.register(Rect::new(5, 5, 10, 1), Annotation::button("child"));
            reg.close();
        });
    });

    let backend = CaptureBackend::new(80, 24);
    let output = AnnotatedOutput::from_backend_and_registry(&backend, &registry);

    assert_eq!(output.annotation_count(), 2);

    let parent = output.find_by_id("parent").unwrap();
    assert!(parent.parent_index.is_none());
    assert_eq!(parent.depth, 0);

    let child = output.find_by_id("child").unwrap();
    assert_eq!(child.parent_index, Some(0));
    assert_eq!(child.depth, 1);
}
