use super::*;
use crate::component::test_utils;

// =============================================================================
// Construction
// =============================================================================

#[test]
fn test_new() {
    let state = CanvasState::new();
    assert!(state.shapes().is_empty());
    assert_eq!(state.x_bounds(), [0.0, 100.0]);
    assert_eq!(state.y_bounds(), [0.0, 100.0]);
    assert_eq!(state.title(), None);
    assert_eq!(state.marker(), &CanvasMarker::Braille);
}

#[test]
fn test_default() {
    let state = CanvasState::default();
    assert!(state.shapes().is_empty());
    assert_eq!(state.x_bounds(), [0.0, 100.0]);
    assert_eq!(state.y_bounds(), [0.0, 100.0]);
}

#[test]
fn test_default_matches_init() {
    let default_state = CanvasState::default();
    let init_state = Canvas::init();

    assert_eq!(default_state.shapes().len(), init_state.shapes().len());
    assert_eq!(default_state.x_bounds(), init_state.x_bounds());
    assert_eq!(default_state.y_bounds(), init_state.y_bounds());
    assert_eq!(default_state.title(), init_state.title());
    assert_eq!(default_state.marker(), init_state.marker());
}

// =============================================================================
// Builders
// =============================================================================

#[test]
fn test_with_shapes() {
    let shapes = vec![CanvasShape::Circle {
        x: 50.0,
        y: 50.0,
        radius: 10.0,
        color: Color::Red,
    }];
    let state = CanvasState::new().with_shapes(shapes);
    assert_eq!(state.shapes().len(), 1);
}

#[test]
fn test_with_x_bounds() {
    let state = CanvasState::new().with_x_bounds(-50.0, 50.0);
    assert_eq!(state.x_bounds(), [-50.0, 50.0]);
}

#[test]
fn test_with_y_bounds() {
    let state = CanvasState::new().with_y_bounds(0.0, 200.0);
    assert_eq!(state.y_bounds(), [0.0, 200.0]);
}

#[test]
fn test_with_bounds() {
    let state = CanvasState::new().with_bounds(0.0, 200.0, -100.0, 100.0);
    assert_eq!(state.x_bounds(), [0.0, 200.0]);
    assert_eq!(state.y_bounds(), [-100.0, 100.0]);
}

#[test]
fn test_with_title() {
    let state = CanvasState::new().with_title("Test Canvas");
    assert_eq!(state.title(), Some("Test Canvas"));
}

#[test]
fn test_with_marker() {
    let state = CanvasState::new().with_marker(CanvasMarker::Block);
    assert_eq!(state.marker(), &CanvasMarker::Block);
}

#[test]
fn test_with_marker_dot() {
    let state = CanvasState::new().with_marker(CanvasMarker::Dot);
    assert_eq!(state.marker(), &CanvasMarker::Dot);
}

#[test]
fn test_with_marker_half_block() {
    let state = CanvasState::new().with_marker(CanvasMarker::HalfBlock);
    assert_eq!(state.marker(), &CanvasMarker::HalfBlock);
}

// =============================================================================
// Shape operations
// =============================================================================

#[test]
fn test_add_shape() {
    let mut state = CanvasState::new();
    state.add_shape(CanvasShape::Line {
        x1: 0.0,
        y1: 0.0,
        x2: 100.0,
        y2: 100.0,
        color: Color::White,
    });
    assert_eq!(state.shapes().len(), 1);
}

#[test]
fn test_add_multiple_shapes() {
    let mut state = CanvasState::new();
    state.add_shape(CanvasShape::Line {
        x1: 0.0,
        y1: 0.0,
        x2: 100.0,
        y2: 100.0,
        color: Color::White,
    });
    state.add_shape(CanvasShape::Circle {
        x: 50.0,
        y: 50.0,
        radius: 10.0,
        color: Color::Cyan,
    });
    assert_eq!(state.shapes().len(), 2);
}

#[test]
fn test_clear() {
    let mut state = CanvasState::new().with_shapes(vec![
        CanvasShape::Circle {
            x: 50.0,
            y: 50.0,
            radius: 10.0,
            color: Color::Red,
        },
        CanvasShape::Line {
            x1: 0.0,
            y1: 0.0,
            x2: 100.0,
            y2: 100.0,
            color: Color::Green,
        },
    ]);
    assert_eq!(state.shapes().len(), 2);
    state.clear();
    assert!(state.shapes().is_empty());
}

// =============================================================================
// Bounds operations
// =============================================================================

#[test]
fn test_set_x_bounds() {
    let mut state = CanvasState::new();
    state.set_x_bounds(-100.0, 100.0);
    assert_eq!(state.x_bounds(), [-100.0, 100.0]);
}

#[test]
fn test_set_y_bounds() {
    let mut state = CanvasState::new();
    state.set_y_bounds(-50.0, 50.0);
    assert_eq!(state.y_bounds(), [-50.0, 50.0]);
}

// =============================================================================
// Title operations
// =============================================================================

#[test]
fn test_set_title() {
    let mut state = CanvasState::new();
    state.set_title(Some("New Title".into()));
    assert_eq!(state.title(), Some("New Title"));
}

#[test]
fn test_set_title_none() {
    let mut state = CanvasState::new().with_title("Old");
    state.set_title(None);
    assert_eq!(state.title(), None);
}

// =============================================================================
// Marker operations
// =============================================================================

#[test]
fn test_set_marker() {
    let mut state = CanvasState::new();
    state.set_marker(CanvasMarker::Dot);
    assert_eq!(state.marker(), &CanvasMarker::Dot);
}

#[test]
fn test_marker_default() {
    let marker = CanvasMarker::default();
    assert_eq!(marker, CanvasMarker::Braille);
}

// =============================================================================
// Focus/Disabled state
// =============================================================================

// =============================================================================
// Update messages
// =============================================================================

#[test]
fn test_update_add_shape() {
    let mut state = CanvasState::new();
    let output = Canvas::update(
        &mut state,
        CanvasMessage::AddShape(CanvasShape::Circle {
            x: 50.0,
            y: 50.0,
            radius: 10.0,
            color: Color::Green,
        }),
    );
    assert_eq!(output, None);
    assert_eq!(state.shapes().len(), 1);
}

#[test]
fn test_update_set_shapes() {
    let mut state = CanvasState::new();
    state.add_shape(CanvasShape::Circle {
        x: 50.0,
        y: 50.0,
        radius: 10.0,
        color: Color::Red,
    });
    let new_shapes = vec![
        CanvasShape::Line {
            x1: 0.0,
            y1: 0.0,
            x2: 100.0,
            y2: 100.0,
            color: Color::White,
        },
        CanvasShape::Rectangle {
            x: 10.0,
            y: 10.0,
            width: 80.0,
            height: 80.0,
            color: Color::Blue,
        },
    ];
    let output = Canvas::update(&mut state, CanvasMessage::SetShapes(new_shapes));
    assert_eq!(output, None);
    assert_eq!(state.shapes().len(), 2);
}

#[test]
fn test_update_clear() {
    let mut state = CanvasState::new().with_shapes(vec![CanvasShape::Circle {
        x: 50.0,
        y: 50.0,
        radius: 10.0,
        color: Color::Red,
    }]);
    let output = Canvas::update(&mut state, CanvasMessage::Clear);
    assert_eq!(output, None);
    assert!(state.shapes().is_empty());
}

#[test]
fn test_update_set_bounds() {
    let mut state = CanvasState::new();
    let output = Canvas::update(
        &mut state,
        CanvasMessage::SetBounds {
            x: [-50.0, 50.0],
            y: [-25.0, 25.0],
        },
    );
    assert_eq!(output, None);
    assert_eq!(state.x_bounds(), [-50.0, 50.0]);
    assert_eq!(state.y_bounds(), [-25.0, 25.0]);
}

#[test]
fn test_update_set_marker() {
    let mut state = CanvasState::new();
    let output = Canvas::update(
        &mut state,
        CanvasMessage::SetMarker(CanvasMarker::HalfBlock),
    );
    assert_eq!(output, None);
    assert_eq!(state.marker(), &CanvasMarker::HalfBlock);
}

// =============================================================================
// Instance methods
// =============================================================================

#[test]
fn test_instance_update() {
    let mut state = CanvasState::new();
    let output = state.update(CanvasMessage::AddShape(CanvasShape::Circle {
        x: 50.0,
        y: 50.0,
        radius: 10.0,
        color: Color::Red,
    }));
    assert_eq!(output, None);
    assert_eq!(state.shapes().len(), 1);
}

// =============================================================================
// CanvasShape variants
// =============================================================================

#[test]
fn test_shape_line() {
    let shape = CanvasShape::Line {
        x1: 0.0,
        y1: 0.0,
        x2: 100.0,
        y2: 100.0,
        color: Color::Red,
    };
    assert_eq!(
        shape,
        CanvasShape::Line {
            x1: 0.0,
            y1: 0.0,
            x2: 100.0,
            y2: 100.0,
            color: Color::Red,
        }
    );
}

#[test]
fn test_shape_rectangle() {
    let shape = CanvasShape::Rectangle {
        x: 10.0,
        y: 10.0,
        width: 50.0,
        height: 30.0,
        color: Color::Blue,
    };
    assert_eq!(
        shape,
        CanvasShape::Rectangle {
            x: 10.0,
            y: 10.0,
            width: 50.0,
            height: 30.0,
            color: Color::Blue,
        }
    );
}

#[test]
fn test_shape_circle() {
    let shape = CanvasShape::Circle {
        x: 50.0,
        y: 50.0,
        radius: 20.0,
        color: Color::Yellow,
    };
    assert_eq!(
        shape,
        CanvasShape::Circle {
            x: 50.0,
            y: 50.0,
            radius: 20.0,
            color: Color::Yellow,
        }
    );
}

#[test]
fn test_shape_points() {
    let shape = CanvasShape::Points {
        coords: vec![(10.0, 20.0), (30.0, 40.0)],
        color: Color::Green,
    };
    assert_eq!(
        shape,
        CanvasShape::Points {
            coords: vec![(10.0, 20.0), (30.0, 40.0)],
            color: Color::Green,
        }
    );
}

#[test]
fn test_shape_label() {
    let shape = CanvasShape::Label {
        x: 25.0,
        y: 75.0,
        text: "Hello".to_string(),
        color: Color::Magenta,
    };
    assert_eq!(
        shape,
        CanvasShape::Label {
            x: 25.0,
            y: 75.0,
            text: "Hello".to_string(),
            color: Color::Magenta,
        }
    );
}

#[test]
fn test_shape_clone() {
    let shape = CanvasShape::Circle {
        x: 50.0,
        y: 50.0,
        radius: 10.0,
        color: Color::Red,
    };
    let cloned = shape.clone();
    assert_eq!(shape, cloned);
}

// =============================================================================
// CanvasMarker
// =============================================================================

#[test]
fn test_marker_clone() {
    let marker = CanvasMarker::Braille;
    let cloned = marker.clone();
    assert_eq!(marker, cloned);
}

#[test]
fn test_marker_eq() {
    assert_eq!(CanvasMarker::Dot, CanvasMarker::Dot);
    assert_eq!(CanvasMarker::Block, CanvasMarker::Block);
    assert_eq!(CanvasMarker::HalfBlock, CanvasMarker::HalfBlock);
    assert_eq!(CanvasMarker::Braille, CanvasMarker::Braille);
    assert_ne!(CanvasMarker::Dot, CanvasMarker::Block);
}

// =============================================================================
// CanvasMessage
// =============================================================================

#[test]
fn test_message_clone() {
    let msg = CanvasMessage::Clear;
    let cloned = msg.clone();
    assert_eq!(msg, cloned);
}

#[test]
fn test_message_add_shape_clone() {
    let msg = CanvasMessage::AddShape(CanvasShape::Circle {
        x: 50.0,
        y: 50.0,
        radius: 10.0,
        color: Color::Red,
    });
    let cloned = msg.clone();
    assert_eq!(msg, cloned);
}

// =============================================================================
// Rendering
// =============================================================================

#[test]
fn test_render_empty() {
    let state = CanvasState::new();
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Canvas::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
}

#[test]
fn test_render_with_title() {
    let state = CanvasState::new().with_title("Test Canvas");
    let (mut terminal, theme) = test_utils::setup_render(40, 15);
    terminal
        .draw(|frame| {
            Canvas::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_render_with_shapes() {
    let state = CanvasState::new().with_title("Shapes").with_shapes(vec![
        CanvasShape::Line {
            x1: 0.0,
            y1: 0.0,
            x2: 100.0,
            y2: 100.0,
            color: Color::Red,
        },
        CanvasShape::Rectangle {
            x: 10.0,
            y: 10.0,
            width: 30.0,
            height: 30.0,
            color: Color::Blue,
        },
        CanvasShape::Circle {
            x: 70.0,
            y: 70.0,
            radius: 15.0,
            color: Color::Green,
        },
    ]);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Canvas::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_render_focused() {
    let state = CanvasState::new();
    let (mut terminal, theme) = test_utils::setup_render(40, 15);
    terminal
        .draw(|frame| {
            Canvas::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_render_disabled() {
    let state = CanvasState::new()
        .with_title("Disabled Canvas")
        .with_shapes(vec![CanvasShape::Circle {
            x: 50.0,
            y: 50.0,
            radius: 20.0,
            color: Color::Red,
        }]);
    let (mut terminal, theme) = test_utils::setup_render(40, 15);
    terminal
        .draw(|frame| {
            Canvas::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).disabled(true),
            );
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_render_small_area() {
    let state = CanvasState::new().with_title("Small");
    let (mut terminal, theme) = test_utils::setup_render(5, 2);
    terminal
        .draw(|frame| {
            Canvas::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
}

#[test]
fn test_render_tiny_area_no_panic() {
    let state = CanvasState::new();
    let (mut terminal, theme) = test_utils::setup_render(1, 1);
    terminal
        .draw(|frame| {
            Canvas::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
}

#[test]
fn test_render_with_points() {
    let state = CanvasState::new()
        .with_title("Points")
        .with_shapes(vec![CanvasShape::Points {
            coords: vec![(10.0, 10.0), (50.0, 50.0), (90.0, 90.0)],
            color: Color::Yellow,
        }]);
    let (mut terminal, theme) = test_utils::setup_render(40, 15);
    terminal
        .draw(|frame| {
            Canvas::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_render_with_label() {
    let state = CanvasState::new()
        .with_title("Labels")
        .with_shapes(vec![CanvasShape::Label {
            x: 50.0,
            y: 50.0,
            text: "Hello".to_string(),
            color: Color::Cyan,
        }]);
    let (mut terminal, theme) = test_utils::setup_render(40, 15);
    terminal
        .draw(|frame| {
            Canvas::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

// =============================================================================
// Annotation
// =============================================================================

#[test]
fn test_annotation_emitted() {
    use crate::annotation::with_annotations;
    let state = CanvasState::new().with_title("Annotated");
    let (mut terminal, theme) = test_utils::setup_render(40, 15);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                Canvas::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
            })
            .unwrap();
    });
    assert!(registry.get_by_id("canvas").is_some());
}

// =============================================================================
// PartialEq
// =============================================================================

#[test]
fn test_state_partial_eq() {
    let state1 = CanvasState::new().with_title("Test");
    let state2 = CanvasState::new().with_title("Test");
    assert_eq!(state1, state2);
}

#[test]
fn test_state_partial_eq_different() {
    let state1 = CanvasState::new().with_title("A");
    let state2 = CanvasState::new().with_title("B");
    assert_ne!(state1, state2);
}

// =============================================================================
// Builder chaining
// =============================================================================

#[test]
fn test_builder_chaining() {
    let state = CanvasState::new()
        .with_x_bounds(0.0, 200.0)
        .with_y_bounds(-100.0, 100.0)
        .with_title("Full Builder")
        .with_marker(CanvasMarker::HalfBlock)
        .with_shapes(vec![CanvasShape::Line {
            x1: 0.0,
            y1: 0.0,
            x2: 200.0,
            y2: 100.0,
            color: Color::Cyan,
        }]);

    assert_eq!(state.x_bounds(), [0.0, 200.0]);
    assert_eq!(state.y_bounds(), [-100.0, 100.0]);
    assert_eq!(state.title(), Some("Full Builder"));
    assert_eq!(state.marker(), &CanvasMarker::HalfBlock);
    assert_eq!(state.shapes().len(), 1);
}
