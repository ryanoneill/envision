//! Treemap example -- hierarchical proportional data visualization.
//!
//! Demonstrates the Treemap component showing disk usage data.
//! Navigate with arrow keys / hjkl, Enter to zoom in, Escape to zoom out.
//!
//! Run with: cargo run --example treemap --features compound-components

use envision::prelude::*;

/// Application marker type.
struct TreemapApp;

/// Application state.
#[derive(Clone)]
struct State {
    treemap: TreemapState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Quit,
    Treemap(TreemapMessage),
}

fn build_disk_usage() -> TreemapNode {
    TreemapNode::new("disk", 0.0)
        .with_child(
            TreemapNode::new("src", 0.0)
                .with_color(Color::Blue)
                .with_child(TreemapNode::new("main.rs", 45.0).with_color(Color::Cyan))
                .with_child(TreemapNode::new("lib.rs", 32.0).with_color(Color::LightBlue))
                .with_child(
                    TreemapNode::new("component", 0.0)
                        .with_color(Color::Blue)
                        .with_child(TreemapNode::new("mod.rs", 28.0).with_color(Color::Cyan))
                        .with_child(
                            TreemapNode::new("treemap.rs", 22.0).with_color(Color::LightBlue),
                        )
                        .with_child(
                            TreemapNode::new("heatmap.rs", 18.0).with_color(Color::LightCyan),
                        ),
                ),
        )
        .with_child(
            TreemapNode::new("docs", 0.0)
                .with_color(Color::Green)
                .with_child(TreemapNode::new("guide.md", 15.0).with_color(Color::LightGreen))
                .with_child(TreemapNode::new("api.md", 12.0).with_color(Color::Green)),
        )
        .with_child(
            TreemapNode::new("tests", 0.0)
                .with_color(Color::Yellow)
                .with_child(TreemapNode::new("integration.rs", 20.0).with_color(Color::LightYellow))
                .with_child(TreemapNode::new("unit.rs", 14.0).with_color(Color::Yellow)),
        )
        .with_child(TreemapNode::new("Cargo.toml", 8.0).with_color(Color::Magenta))
        .with_child(TreemapNode::new("README.md", 5.0).with_color(Color::LightMagenta))
}

impl App for TreemapApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let treemap = TreemapState::new()
            .with_root(build_disk_usage())
            .with_title("Disk Usage (KB)")
            .with_show_values(true);

        (State { treemap }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Quit => Command::quit(),
            Msg::Treemap(treemap_msg) => {
                state.treemap.update(treemap_msg);
                Command::none()
            }
        }
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        Treemap::view(&state.treemap, &mut RenderContext::new(frame, area, &theme));
    }

    fn handle_event(event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            if let Key::Char('q') = key.key {
                return Some(Msg::Quit);
            }
        }
        None
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(msg) =
            Treemap::handle_event(&state.treemap, event, &EventContext::new().focused(true))
        {
            return Some(Msg::Treemap(msg));
        }
        None
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<TreemapApp, _>::virtual_terminal(60, 16)?;

    println!("=== Treemap Example ===\n");

    // Initial render.
    vt.tick()?;
    println!("Disk usage treemap:");
    println!("{}\n", vt.display());

    // Navigate to next sibling.
    vt.send(Event::key(Key::Right));
    vt.tick()?;
    println!("After selecting next sibling:");
    println!("{}\n", vt.display());

    // Zoom into first node (src).
    vt.send(Event::key(Key::Left));
    vt.tick()?;
    vt.send(Event::key(Key::Enter));
    vt.tick()?;
    println!("After zooming into 'src':");
    println!("{}\n", vt.display());

    // Zoom out.
    vt.send(Event::key(Key::Esc));
    vt.tick()?;
    println!("After zooming out:");
    println!("{}\n", vt.display());

    Ok(())
}
