//! Collapsible example -- a single expandable/collapsible section.
//!
//! Demonstrates the Collapsible component with a header that can be toggled
//! to expand or collapse a content area.
//!
//! Run with: cargo run --example collapsible --features display-components

use envision::prelude::*;

/// Application marker type.
struct CollapsibleApp;

/// Application state wrapping a single Collapsible.
#[derive(Clone)]
struct State {
    collapsible: CollapsibleState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Collapsible(CollapsibleMessage),
    Quit,
}

impl App for CollapsibleApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let collapsible = CollapsibleState::new("Advanced Settings").with_content_height(4);

        (State { collapsible }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Collapsible(m) => {
                Collapsible::update(&mut state.collapsible, m);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        // Render the collapsible header and border
        Collapsible::view(
            &state.collapsible,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );

        // Render content inside the content area when expanded
        let content_area = state.collapsible.content_area(chunks[0]);
        if content_area.height > 0 {
            // Inset by 1 on left/bottom for the border
            let inner = Rect::new(
                content_area.x + 1,
                content_area.y,
                content_area.width.saturating_sub(1),
                content_area.height.saturating_sub(1),
            );
            if inner.height > 0 && inner.width > 0 {
                let content = ratatui::widgets::Paragraph::new(
                    "Cache size: 256 MB\nLog level: debug\nMax retries: 3",
                )
                .style(theme.normal_style());
                frame.render_widget(content, inner);
            }
        }

        let expanded_label = if state.collapsible.is_expanded() {
            "expanded"
        } else {
            "collapsed"
        };
        let status = format!(
            " Status: {} | Space/Enter: toggle, Left: collapse, Right: expand, q: quit",
            expanded_label
        );
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
        Collapsible::handle_event(
            &state.collapsible,
            event,
            &EventContext::new().focused(true),
        )
        .map(Msg::Collapsible)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<CollapsibleApp, _>::virtual_builder(60, 12).build()?;

    println!("=== Collapsible Example ===\n");

    // Initial render: section expanded
    vt.tick()?;
    println!("Initial state (expanded):");
    println!("{}\n", vt.display());

    // Collapse the section
    vt.dispatch(Msg::Collapsible(CollapsibleMessage::Collapse));
    vt.tick()?;
    println!("After collapse:");
    println!("{}\n", vt.display());

    // Expand the section
    vt.dispatch(Msg::Collapsible(CollapsibleMessage::Expand));
    vt.tick()?;
    println!("After expand:");
    println!("{}\n", vt.display());

    // Toggle the section
    vt.dispatch(Msg::Collapsible(CollapsibleMessage::Toggle));
    vt.tick()?;
    println!("After toggle:");
    println!("{}\n", vt.display());

    Ok(())
}
