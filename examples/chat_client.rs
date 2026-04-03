#![allow(
    clippy::collapsible_if,
    clippy::single_match,
    clippy::type_complexity,
    clippy::collapsible_match
)]
// =============================================================================
// Chat Client — Reference Application #3
// =============================================================================
//
// An AI chat client demonstrating the "Claude Code" component suite:
//
// - **ConversationView**: Structured message thread with roles
// - **MessageHandle**: Stable streaming identity
// - **CommandPalette**: Slash commands (/help, /clear, /model)
// - **TabBar**: Multiple conversation tabs
// - **TextArea**: Multi-line input
// - **StatusBar**: Token count and model info
// - **Markdown**: Rendered assistant responses
// - **CodeBlock**: Syntax-highlighted code in responses
//
// Run: cargo run --example chat_client --features full
//
// Layout:
// ┌ Chat 1 │ Chat 2 • │ + ────────────────────────┐
// │ ● User                                         │
// │ How do I sort a vector?                         │
// │                                                 │
// │ ◆ Assistant                                     │
// │ Here's how to sort in Rust:                     │
// │ │ let mut v = vec![3, 1, 2];                    │
// │ │ v.sort();                                     │
// ├─────────────────────────────────────────────────┤
// │ > Type a message...                             │
// ├─────────────────────────────────────────────────┤
// │ Model: claude │ Messages: 4 │ Ctrl+P: commands  │
// └─────────────────────────────────────────────────┘

use envision::prelude::*;

// ---------------------------------------------------------------------------
// Focus
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Focus {
    Conversation,
    Input,
}

// ---------------------------------------------------------------------------
// App message
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
enum Msg {
    Conv(ConversationViewMessage),
    Input(TextAreaMessage),
    TabBar(TabBarMessage),
    Palette(CommandPaletteMessage),

    SubmitInput,
    FocusToggle,
    TogglePalette,
    NewTab,
    CloseTab,
    SimulateResponse,
    Quit,
}

// ---------------------------------------------------------------------------
// Conversation state per tab
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct Conversation {
    view: ConversationViewState,
    /// Handle to the currently streaming message (if any)
    streaming_handle: Option<MessageHandle>,
}

impl Conversation {
    fn new(title: &str) -> Self {
        Self {
            view: ConversationViewState::new()
                .with_title(title)
                .with_markdown(true)
                .with_role_labels(true),
            streaming_handle: None,
        }
    }
}

// ---------------------------------------------------------------------------
// App state
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct State {
    conversations: Vec<Conversation>,
    active_tab: usize,
    tab_bar: TabBarState,
    input: TextAreaState,
    palette: CommandPaletteState,
    status: StatusBarState,
    focus: FocusManager<Focus>,
    message_count: usize,
}

impl State {
    fn active_conv(&self) -> &Conversation {
        &self.conversations[self.active_tab]
    }
    fn active_conv_mut(&mut self) -> &mut Conversation {
        &mut self.conversations[self.active_tab]
    }
}

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

struct ChatClient;

impl App for ChatClient {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let palette_items = vec![
            PaletteItem::new("help", "Show Help")
                .with_shortcut("F1")
                .with_category("General"),
            PaletteItem::new("clear", "Clear Conversation")
                .with_shortcut("Ctrl+L")
                .with_category("Actions"),
            PaletteItem::new("new", "New Conversation")
                .with_shortcut("Ctrl+N")
                .with_category("Actions"),
            PaletteItem::new("close", "Close Tab")
                .with_shortcut("Ctrl+W")
                .with_category("Actions"),
            PaletteItem::new("quit", "Quit")
                .with_shortcut("Ctrl+Q")
                .with_category("General"),
        ];

        let mut status = StatusBarState::new();
        status.set_left(vec![StatusBarItem::new("Model: claude-3")]);
        status.set_center(vec![StatusBarItem::new("Messages: 0")]);
        status.set_right(vec![StatusBarItem::new(
            "Tab: switch │ Ctrl+P: commands │ Ctrl+Enter: send",
        )]);

        let initial_conv = Conversation::new("Chat 1");
        let tab_bar = TabBarState::new(vec![Tab::new("chat-1", "Chat 1").with_closable(true)]);

        let state = State {
            conversations: vec![initial_conv],
            active_tab: 0,
            tab_bar,
            input: TextAreaState::with_placeholder("Type a message... (Ctrl+Enter to send)"),
            palette: CommandPaletteState::new(palette_items)
                .with_title("Commands")
                .with_placeholder("Type a command..."),
            status,
            focus: FocusManager::with_initial_focus(vec![Focus::Input, Focus::Conversation]),
            message_count: 0,
        };

        (state, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Conv(m) => {
                ConversationView::update(&mut state.active_conv_mut().view, m);
            }
            Msg::Input(m) => {
                TextArea::update(&mut state.input, m);
            }
            Msg::TabBar(m) => {
                if let Some(output) = TabBar::update(&mut state.tab_bar, m) {
                    match output {
                        TabBarOutput::TabSelected(idx) => {
                            if idx < state.conversations.len() {
                                state.active_tab = idx;
                                update_status(state);
                            }
                        }
                        TabBarOutput::TabClosed(_idx) => {
                            // Remove conversation
                            if state.conversations.len() > 1
                                && state.active_tab < state.conversations.len()
                            {
                                state.conversations.remove(state.active_tab);
                                if state.active_tab >= state.conversations.len() {
                                    state.active_tab = state.conversations.len().saturating_sub(1);
                                }
                                state.tab_bar.set_active(Some(state.active_tab));
                            }
                        }
                        _ => {}
                    }
                }
            }
            Msg::Palette(m) => {
                if let Some(output) = CommandPalette::update(&mut state.palette, m) {
                    match output {
                        CommandPaletteOutput::Selected(item) => match item.id.as_str() {
                            "clear" => {
                                state.active_conv_mut().view.clear_messages();
                                state.message_count = 0;
                                update_status(state);
                            }
                            "new" => return Self::update(state, Msg::NewTab),
                            "close" => return Self::update(state, Msg::CloseTab),
                            "quit" => return Command::quit(),
                            "help" => {
                                state.active_conv_mut().view.push_system(
                                    "**Help**: Type a message and press Ctrl+Enter to send. \
                                     Use Ctrl+P for commands, Tab to switch focus.",
                                );
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }

            Msg::SubmitInput => {
                let text = state.input.value();
                if !text.trim().is_empty() {
                    // Push user message
                    state.active_conv_mut().view.push_user(&text);
                    state.input.set_value("");
                    state.message_count += 1;

                    // Mark tab as modified
                    if let Some(tab) = state.tab_bar.active_tab_mut() {
                        tab.set_modified(true);
                    }

                    update_status(state);

                    // Simulate assistant response
                    return Command::message(Msg::SimulateResponse);
                }
            }

            Msg::SimulateResponse => {
                let response = generate_response(state.message_count);

                // Push assistant message with handle for streaming
                let handle = state.active_conv_mut().view.push_assistant("");
                state.active_conv_mut().streaming_handle = Some(handle);

                // Simulate "streaming" by setting the full content immediately
                // (In a real app, you'd append incrementally via update_by_handle)
                state
                    .active_conv_mut()
                    .view
                    .update_by_handle(handle, |msg| {
                        msg.set_blocks(vec![MessageBlock::text(&response)]);
                    });
                state.active_conv_mut().streaming_handle = None;

                state.message_count += 1;
                update_status(state);
            }

            Msg::FocusToggle => {
                state.focus.focus_next();
            }

            Msg::TogglePalette => {
                if state.palette.is_visible() {
                    state.palette.dismiss();
                } else {
                    state.palette.show();
                }
            }

            Msg::NewTab => {
                let idx = state.conversations.len() + 1;
                let name = format!("Chat {}", idx);
                let conv = Conversation::new(&name);
                state.conversations.push(conv);

                TabBar::update(
                    &mut state.tab_bar,
                    TabBarMessage::AddTab(
                        Tab::new(format!("chat-{}", idx), &name).with_closable(true),
                    ),
                );
                state.active_tab = state.conversations.len() - 1;
                state.tab_bar.set_active(Some(state.active_tab));
            }

            Msg::CloseTab => {
                if state.conversations.len() > 1 {
                    TabBar::update(
                        &mut state.tab_bar,
                        TabBarMessage::CloseTab(state.active_tab),
                    );
                    state.conversations.remove(state.active_tab);
                    if state.active_tab >= state.conversations.len() {
                        state.active_tab = state.conversations.len().saturating_sub(1);
                    }
                    state.tab_bar.set_active(Some(state.active_tab));
                }
            }

            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();

        // Layout: tab bar + conversation + input + status
        let chunks = Layout::vertical([
            Constraint::Length(1), // Tab bar
            Constraint::Min(0),    // Conversation view
            Constraint::Length(3), // Input area
            Constraint::Length(1), // Status bar
        ])
        .split(area);

        let conv_focused = state.focus.is_focused(&Focus::Conversation);
        let input_focused = state.focus.is_focused(&Focus::Input);

        // Tab bar
        TabBar::view(
            &state.tab_bar,
            frame,
            chunks[0],
            &theme,
            &ViewContext::new().focused(true),
        );

        // Conversation
        ConversationView::view(
            &state.active_conv().view,
            frame,
            chunks[1],
            &theme,
            &ViewContext::new().focused(conv_focused),
        );

        // Input
        TextArea::view(
            &state.input,
            frame,
            chunks[2],
            &theme,
            &ViewContext::new().focused(input_focused),
        );

        // Status
        StatusBar::view(
            &state.status,
            frame,
            chunks[3],
            &theme,
            &ViewContext::default(),
        );

        // Command palette overlay (render last)
        if state.palette.is_visible() {
            CommandPalette::view(
                &state.palette,
                frame,
                area,
                &theme,
                &ViewContext::new().focused(true),
            );
        }
    }

    fn handle_event(_event: &Event) -> Option<Msg> {
        None
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        let key = event.as_key()?;
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        // Command palette gets priority
        if state.palette.is_visible() {
            return CommandPalette::handle_event(&state.palette, event).map(Msg::Palette);
        }

        // Global shortcuts
        match key.code {
            KeyCode::Char('q') if ctrl => return Some(Msg::Quit),
            KeyCode::Char('p') if ctrl => return Some(Msg::TogglePalette),
            KeyCode::Char('n') if ctrl => return Some(Msg::NewTab),
            KeyCode::Char('w') if ctrl => return Some(Msg::CloseTab),
            KeyCode::Char('l') if ctrl => {
                state
                    .conversations
                    .get(state.active_tab)
                    .map(|_| Msg::Palette(CommandPaletteMessage::Confirm));
            }
            KeyCode::Tab => return Some(Msg::FocusToggle),
            KeyCode::Esc => return Some(Msg::Quit),
            _ => {}
        }

        // Input-focused: Ctrl+Enter submits, other keys go to TextArea
        if state.focus.is_focused(&Focus::Input) {
            if ctrl && key.code == KeyCode::Enter {
                return Some(Msg::SubmitInput);
            }
            return TextArea::handle_event(&state.input, event).map(Msg::Input);
        }

        // Conversation-focused: route to ConversationView
        if state.focus.is_focused(&Focus::Conversation) {
            return ConversationView::handle_event(&state.active_conv().view, event).map(Msg::Conv);
        }

        // Fall through: route to TabBar for left/right tab switching
        TabBar::handle_event(&state.tab_bar, event).map(Msg::TabBar)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn update_status(state: &mut State) {
    state.status.set_center(vec![StatusBarItem::new(format!(
        "Messages: {} │ Tab: {}",
        state.message_count,
        state.active_tab + 1,
    ))]);
}

fn generate_response(msg_num: usize) -> String {
    let responses = [
        "Here's how to sort a vector in Rust:\n\n```rust\nlet mut v = vec![3, 1, 2];\nv.sort();\nprintln!(\"{:?}\", v); // [1, 2, 3]\n```\n\nYou can also use `sort_by` for custom ordering.",
        "The **ownership** system in Rust prevents data races at compile time. Key rules:\n\n1. Each value has exactly one owner\n2. When the owner goes out of scope, the value is dropped\n3. You can have either one `&mut` reference OR many `&` references",
        "To handle errors in Rust, use the `Result<T, E>` type:\n\n```rust\nfn read_file(path: &str) -> Result<String, std::io::Error> {\n    std::fs::read_to_string(path)\n}\n\nmatch read_file(\"data.txt\") {\n    Ok(content) => println!(\"{}\", content),\n    Err(e) => eprintln!(\"Error: {}\", e),\n}\n```",
        "For async operations, Rust uses `async/await` with the **tokio** runtime:\n\n```rust\n#[tokio::main]\nasync fn main() {\n    let response = reqwest::get(\"https://api.example.com\")\n        .await\n        .unwrap();\n    println!(\"Status: {}\", response.status());\n}\n```",
    ];
    responses[msg_num % responses.len()].to_string()
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() -> envision::Result<()> {
    let mut vt = Runtime::<ChatClient, _>::virtual_terminal(90, 28)?;

    // Simulate a conversation
    // Push a user message and trigger the simulated response
    TextArea::update(
        &mut vt.state_mut().input,
        TextAreaMessage::SetValue("How do I sort a vector in Rust?".to_string()),
    );
    ChatClient::update(vt.state_mut(), Msg::SubmitInput);
    // The SubmitInput returns Command::message(SimulateResponse) which the
    // virtual terminal doesn't process, so dispatch it manually.
    ChatClient::update(vt.state_mut(), Msg::SimulateResponse);
    vt.tick()?;

    // Push another user message and response
    TextArea::update(
        &mut vt.state_mut().input,
        TextAreaMessage::SetValue("What about ownership?".to_string()),
    );
    ChatClient::update(vt.state_mut(), Msg::SubmitInput);
    ChatClient::update(vt.state_mut(), Msg::SimulateResponse);
    vt.tick()?;

    println!("Chat Client — Reference Application");
    println!("====================================");
    println!();
    println!("{}", vt.display());
    println!();
    println!("This demonstrates:");
    println!("  - ConversationView with markdown rendering");
    println!("  - MessageHandle for streaming identity");
    println!("  - TabBar with closable conversation tabs");
    println!("  - TextArea multi-line input");
    println!("  - CommandPalette for slash commands");
    println!("  - FocusManager + ViewContext focus routing");
    println!("  - StatusBar with message count");

    Ok(())
}
