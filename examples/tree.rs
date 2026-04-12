//! Tree example — hierarchical tree view with expand/collapse.
//!
//! Demonstrates the Tree component with keyboard-driven navigation,
//! node expansion and collapse, and selection tracking.
//!
//! Run with: cargo run --example tree

use envision::prelude::*;

/// Application marker type.
struct TreeApp;

/// Application state wrapping a single Tree.
#[derive(Clone)]
struct State {
    tree: TreeState<String>,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Tree(TreeMessage),
    Quit,
}

impl App for TreeApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let mut src = TreeNode::new_expanded("src", "src".into());

        let mut components = TreeNode::new_expanded("components", "components".into());
        components.add_child(TreeNode::new("button.rs", "button.rs".into()));
        components.add_child(TreeNode::new("input.rs", "input.rs".into()));
        components.add_child(TreeNode::new("table.rs", "table.rs".into()));
        src.add_child(components);

        let mut utils = TreeNode::new("utils", "utils".into());
        utils.add_child(TreeNode::new("helpers.rs", "helpers.rs".into()));
        utils.add_child(TreeNode::new("format.rs", "format.rs".into()));
        src.add_child(utils);

        src.add_child(TreeNode::new("main.rs", "main.rs".into()));
        src.add_child(TreeNode::new("lib.rs", "lib.rs".into()));

        let mut tests = TreeNode::new("tests", "tests".into());
        tests.add_child(TreeNode::new("integration.rs", "integration.rs".into()));
        tests.add_child(TreeNode::new("unit.rs", "unit.rs".into()));

        let tree = TreeState::new(vec![src, tests]);

        (State { tree }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Tree(m) => {
                Tree::update(&mut state.tree, m);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        Tree::view(
            &state.tree,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );

        let selected = state
            .tree
            .selected_item()
            .map(|n| n.label().to_string())
            .unwrap_or_else(|| "None".into());
        let status = format!(" Selected: {}", selected);
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            chunks[1],
        );
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            if matches!(key.code, Key::Char('q') | Key::Esc) {
                return Some(Msg::Quit);
            }
        }
        Tree::handle_event(&state.tree, event, &EventContext::new().focused(true)).map(Msg::Tree)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<TreeApp, _>::virtual_terminal(50, 15)?;

    println!("=== Tree Example ===\n");

    // Initial render: expanded tree
    vt.tick()?;
    println!("Initial tree (src expanded, utils collapsed):");
    println!("{}\n", vt.display());

    // Navigate down to select "components"
    vt.dispatch(Msg::Tree(TreeMessage::Down));
    vt.tick()?;
    println!("After Down (selected: components):");
    println!("{}\n", vt.display());

    // Collapse components
    vt.dispatch(Msg::Tree(TreeMessage::Collapse));
    vt.tick()?;
    println!("After Collapse (components collapsed):");
    println!("{}\n", vt.display());

    // Expand it again
    vt.dispatch(Msg::Tree(TreeMessage::Expand));
    vt.tick()?;
    println!("After Expand (components re-expanded):");
    println!("{}\n", vt.display());

    Ok(())
}
