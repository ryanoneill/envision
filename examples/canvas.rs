//! Canvas example -- general-purpose drawing surface.
//!
//! Demonstrates the Canvas component with various shape primitives
//! including lines, rectangles, circles, points, and labels.
//!
//! Run with: cargo run --example canvas --features display-components

use envision::prelude::*;

/// Application marker type.
struct CanvasApp;

/// Application state with a canvas.
#[derive(Clone)]
struct State {
    canvas: CanvasState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Quit,
}

impl App for CanvasApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let canvas = CanvasState::new()
            .with_title("Canvas Drawing")
            .with_bounds(0.0, 100.0, 0.0, 100.0)
            .with_marker(CanvasMarker::Braille)
            .with_shapes(vec![
                // Diagonal line across the canvas
                CanvasShape::Line {
                    x1: 0.0,
                    y1: 0.0,
                    x2: 100.0,
                    y2: 100.0,
                    color: Color::Red,
                },
                // Border rectangle
                CanvasShape::Rectangle {
                    x: 5.0,
                    y: 5.0,
                    width: 90.0,
                    height: 90.0,
                    color: Color::Blue,
                },
                // Center circle
                CanvasShape::Circle {
                    x: 50.0,
                    y: 50.0,
                    radius: 20.0,
                    color: Color::Green,
                },
                // Corner points
                CanvasShape::Points {
                    coords: vec![
                        (10.0, 10.0),
                        (90.0, 10.0),
                        (10.0, 90.0),
                        (90.0, 90.0),
                        (50.0, 50.0),
                    ],
                    color: Color::Yellow,
                },
                // Center label
                CanvasShape::Label {
                    x: 50.0,
                    y: 50.0,
                    text: "Center".to_string(),
                    color: Color::Cyan,
                },
                // Small circle in upper-right
                CanvasShape::Circle {
                    x: 80.0,
                    y: 80.0,
                    radius: 8.0,
                    color: Color::Magenta,
                },
            ]);

        (State { canvas }, Command::none())
    }

    fn update(_state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Quit => Command::quit(),
        }
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        Canvas::view(
            &state.canvas,
            &mut RenderContext::new(frame, frame.area(), &theme),
        );
    }

    fn handle_event(event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            match key.key {
                Key::Char('q') | Key::Esc => Some(Msg::Quit),
                _ => None,
            }
        } else {
            None
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<CanvasApp, _>::virtual_terminal(70, 25)?;

    println!("=== Canvas Example ===\n");

    vt.tick()?;
    println!("Canvas with various shapes (line, rectangle, circle, points, label):");
    println!("{}\n", vt.display());

    Ok(())
}
