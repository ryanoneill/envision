//! Example demonstrating the new UI components: Modal, Select, and Menu.
//!
//! This example shows:
//! - Modal: Popup dialogs with overlay backdrop
//! - Select: Dropdown selection component
//! - Menu: Horizontal menu bar
//!
//! Run with: cargo run --example new_components

use envision::component::{
    Component, Focusable, Menu, MenuItem, MenuMessage, MenuOutput, MenuState, Modal, ModalMessage,
    ModalOutput, ModalState, Select, SelectMessage, SelectOutput, SelectState, Toggleable,
};
use envision::prelude::*;
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

/// The application marker type
struct ComponentsApp;

/// Which component is currently focused
#[derive(Clone, Debug, PartialEq)]
enum FocusedComponent {
    Menu,
    Select,
}

/// Application state
#[derive(Clone)]
struct State {
    menu_state: MenuState,
    select_state: SelectState,
    modal_state: ModalState,
    focused: FocusedComponent,
    log: Vec<String>,
}

/// Messages that can modify state
#[derive(Clone, Debug)]
enum Msg {
    // Menu messages
    MenuMsg(MenuMessage),
    MenuOutput(MenuOutput),
    // Select messages
    SelectMsg(SelectMessage),
    SelectOutput(SelectOutput),
    // Modal messages
    ModalMsg(ModalMessage),
    ModalOutput(ModalOutput),
    // Focus switching
    NextFocus,
    PrevFocus,
    // App control
    Quit,
}

impl App for ComponentsApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let menu_state = MenuState::new(vec![
            MenuItem::new("File"),
            MenuItem::new("Edit"),
            MenuItem::new("View"),
            MenuItem::new("Help"),
        ]);

        let select_state = SelectState::new(vec!["Red", "Green", "Blue", "Yellow", "Cyan"]);

        let mut modal_state = ModalState::new("Welcome!", 50, 8);
        modal_state.set_content("Welcome to the new components demo!\n\nPress Tab to switch focus.\nPress Enter on menu or select items.");

        let mut log = Vec::new();
        log.push("App initialized".to_string());

        let state = State {
            menu_state,
            select_state,
            modal_state,
            focused: FocusedComponent::Menu,
            log,
        };

        (state, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            // Menu messages
            Msg::MenuMsg(menu_msg) => {
                if let Some(output) = Menu::update(&mut state.menu_state, menu_msg) {
                    return Command::message(Msg::MenuOutput(output));
                }
            }
            Msg::MenuOutput(MenuOutput::ItemActivated(idx)) => {
                let item = state.menu_state.items()[idx].label.clone();
                state.log.push(format!("Menu activated: {}", item));

                // Show modal with menu item info
                state.modal_state.set_title("Menu Action");
                state
                    .modal_state
                    .set_content(format!("You selected: {}\n\nThis would normally open a submenu or perform an action.", item));
                Modal::show(&mut state.modal_state);
            }

            // Select messages
            Msg::SelectMsg(select_msg) => {
                if let Some(output) = Select::update(&mut state.select_state, select_msg) {
                    return Command::message(Msg::SelectOutput(output));
                }
            }
            Msg::SelectOutput(output) => match output {
                SelectOutput::Changed(Some(idx)) => {
                    let value = state.select_state.options()[idx].to_string();
                    state.log.push(format!("Selected: {}", value));
                }
                SelectOutput::Submitted(idx) => {
                    let value = state.select_state.options()[idx].to_string();
                    state.log.push(format!("Confirmed: {}", value));
                }
                _ => {}
            },

            // Modal messages
            Msg::ModalMsg(modal_msg) => {
                if let Some(output) = Modal::update(&mut state.modal_state, modal_msg) {
                    return Command::message(Msg::ModalOutput(output));
                }
            }
            Msg::ModalOutput(ModalOutput::Closed) => {
                state.log.push("Modal closed".to_string());
            }

            // Focus switching
            Msg::NextFocus => {
                state.focused = match state.focused {
                    FocusedComponent::Menu => FocusedComponent::Select,
                    FocusedComponent::Select => FocusedComponent::Menu,
                };
                update_focus(state);
            }
            Msg::PrevFocus => {
                state.focused = match state.focused {
                    FocusedComponent::Menu => FocusedComponent::Select,
                    FocusedComponent::Select => FocusedComponent::Menu,
                };
                update_focus(state);
            }

            Msg::Quit => {
                return Command::quit();
            }
        }

        // Keep log to last 10 entries
        if state.log.len() > 10 {
            state.log.remove(0);
        }

        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let area = frame.area();

        // Main layout
        let chunks = Layout::vertical([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Menu
            Constraint::Length(8), // Select
            Constraint::Length(3), // Controls
            Constraint::Min(0),    // Log
        ])
        .split(area);

        // Title
        let title = Paragraph::new("New Components Demo")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title, chunks[0]);

        // Menu
        let menu_block = Block::default()
            .borders(Borders::ALL)
            .title("Menu (Tab to focus)")
            .border_style(if state.focused == FocusedComponent::Menu {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });
        let menu_area = menu_block.inner(chunks[1]);
        frame.render_widget(menu_block, chunks[1]);
        Menu::view(&state.menu_state, frame, menu_area);

        // Select
        let select_block = Block::default()
            .borders(Borders::ALL)
            .title("Select Dropdown (Tab to focus, Space to open)")
            .border_style(if state.focused == FocusedComponent::Select {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });
        let select_area = select_block.inner(chunks[2]);
        frame.render_widget(select_block, chunks[2]);
        Select::view(&state.select_state, frame, select_area);

        // Controls
        let controls = Paragraph::new(Line::from(vec![
            Span::styled("[Tab] ", Style::default().fg(Color::Green)),
            Span::raw("Switch Focus  "),
            Span::styled("[←→] ", Style::default().fg(Color::Yellow)),
            Span::raw("Navigate  "),
            Span::styled("[Enter] ", Style::default().fg(Color::Cyan)),
            Span::raw("Activate  "),
            Span::styled("[Esc] ", Style::default().fg(Color::Red)),
            Span::raw("Close Modal  "),
            Span::styled("[Q] ", Style::default().fg(Color::Magenta)),
            Span::raw("Quit"),
        ]))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Controls"));
        frame.render_widget(controls, chunks[3]);

        // Log
        let log_lines: Vec<Line> = state
            .log
            .iter()
            .map(|s| Line::from(format!("  {}", s)))
            .collect();
        let log = Paragraph::new(log_lines)
            .block(Block::default().borders(Borders::ALL).title("Event Log"));
        frame.render_widget(log, chunks[4]);

        // Render modal last (on top)
        Modal::view(&state.modal_state, frame, area);
    }

    fn handle_event(state: &State, event: &SimulatedEvent) -> Option<Msg> {
        use crossterm::event::KeyCode;

        if let Some(key) = event.as_key() {
            // Modal takes priority if visible
            if Modal::is_visible(&state.modal_state) {
                match key.code {
                    KeyCode::Esc | KeyCode::Enter => {
                        return Some(Msg::ModalMsg(ModalMessage::Close));
                    }
                    _ => return None,
                }
            }

            // Global keys
            match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => return Some(Msg::Quit),
                KeyCode::Tab => return Some(Msg::NextFocus),
                KeyCode::BackTab => return Some(Msg::PrevFocus),
                _ => {}
            }

            // Component-specific keys
            match state.focused {
                FocusedComponent::Menu => match key.code {
                    KeyCode::Left => Some(Msg::MenuMsg(MenuMessage::SelectPrevious)),
                    KeyCode::Right => Some(Msg::MenuMsg(MenuMessage::SelectNext)),
                    KeyCode::Enter => Some(Msg::MenuMsg(MenuMessage::Activate)),
                    _ => None,
                },
                FocusedComponent::Select => match key.code {
                    KeyCode::Char(' ') => Some(Msg::SelectMsg(SelectMessage::Toggle)),
                    KeyCode::Up => Some(Msg::SelectMsg(SelectMessage::SelectPrevious)),
                    KeyCode::Down => Some(Msg::SelectMsg(SelectMessage::SelectNext)),
                    KeyCode::Enter => {
                        if state.select_state.is_open() {
                            Some(Msg::SelectMsg(SelectMessage::Confirm))
                        } else {
                            Some(Msg::SelectMsg(SelectMessage::Open))
                        }
                    }
                    KeyCode::Esc => Some(Msg::SelectMsg(SelectMessage::Close)),
                    _ => None,
                },
            }
        } else {
            None
        }
    }
}

/// Update focus state for all components
fn update_focus(state: &mut State) {
    Menu::set_focused(&mut state.menu_state, state.focused == FocusedComponent::Menu);
    Select::set_focused(
        &mut state.select_state,
        state.focused == FocusedComponent::Select,
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a headless runtime for demonstration
    let mut runtime = Runtime::<ComponentsApp, _>::headless(80, 30)?;

    // Simulate some user interactions
    println!("=== New Components Demo ===\n");

    // Initial render with modal shown
    Modal::show(&mut runtime.state_mut().modal_state);
    update_focus(runtime.state_mut());
    runtime.render()?;
    println!("Initial state with welcome modal:");
    println!("{}\n", runtime.backend());

    // Close modal
    runtime.dispatch(Msg::ModalMsg(ModalMessage::Close));
    runtime.render()?;
    println!("After closing modal:");
    println!("{}\n", runtime.backend());

    // Navigate menu
    runtime.dispatch(Msg::MenuMsg(MenuMessage::SelectNext));
    runtime.dispatch(Msg::MenuMsg(MenuMessage::SelectNext));
    runtime.render()?;
    println!("After navigating menu to 'View':");
    println!("{}\n", runtime.backend());

    // Activate menu item
    runtime.dispatch(Msg::MenuMsg(MenuMessage::Activate));
    runtime.render()?;
    println!("After activating menu item (shows modal):");
    println!("{}\n", runtime.backend());

    // Close modal again
    runtime.dispatch(Msg::ModalMsg(ModalMessage::Close));
    runtime.render()?;

    // Switch to select
    runtime.dispatch(Msg::NextFocus);
    runtime.render()?;
    println!("After switching focus to Select:");
    println!("{}\n", runtime.backend());

    // Open select dropdown
    runtime.dispatch(Msg::SelectMsg(SelectMessage::Open));
    runtime.render()?;
    println!("After opening Select dropdown:");
    println!("{}\n", runtime.backend());

    // Navigate and confirm selection
    runtime.dispatch(Msg::SelectMsg(SelectMessage::SelectNext));
    runtime.dispatch(Msg::SelectMsg(SelectMessage::SelectNext));
    runtime.dispatch(Msg::SelectMsg(SelectMessage::Confirm));
    runtime.render()?;
    println!("After selecting 'Blue':");
    println!("{}\n", runtime.backend());

    println!("Demo completed successfully!");
    println!("Event log entries: {}", runtime.state().log.len());

    Ok(())
}
