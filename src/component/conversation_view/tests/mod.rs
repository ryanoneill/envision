use super::*;
use crate::input::{Event, Key};

fn focused_state() -> ConversationViewState {
    ConversationViewState::new()
}

fn state_with_messages() -> ConversationViewState {
    let mut state = focused_state();
    state.push_system("Welcome to the conversation.");
    state.push_user("Hello, can you help me?");
    state.push_assistant("Of course! What do you need?");
    state
}

mod events;
mod handles;
mod source;
mod state;
mod types;
