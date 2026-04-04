//! Component, Focusable, and Disableable trait implementations for ChatView.

use ratatui::prelude::*;

use super::message::Focus;
use super::render_helpers;
use super::{ChatView, ChatViewMessage, ChatViewOutput, ChatViewState, ViewContext};
use crate::component::{Component, Disableable, Focusable, TextAreaMessage, TextAreaOutput};
use crate::input::{Event, KeyCode, KeyModifiers};
use crate::theme::Theme;

impl Component for ChatView {
    type State = ChatViewState;
    type Message = ChatViewMessage;
    type Output = ChatViewOutput;

    fn init() -> Self::State {
        ChatViewState::default()
    }

    fn handle_event(state: &Self::State, event: &Event) -> Option<Self::Message> {
        Self::handle_event_with_ctx(
            state,
            event,
            &ViewContext::new()
                .focused(state.focused)
                .disabled(state.disabled),
        )
    }

    fn handle_event_with_ctx(
        state: &Self::State,
        event: &Event,
        ctx: &ViewContext,
    ) -> Option<Self::Message> {
        if !ctx.focused || ctx.disabled {
            return None;
        }

        let key = event.as_key()?;

        match state.focus {
            Focus::Input => {
                let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
                match key.code {
                    KeyCode::Enter if ctrl => Some(ChatViewMessage::Submit),
                    KeyCode::Enter => Some(ChatViewMessage::NewLine),
                    KeyCode::Tab => Some(ChatViewMessage::ToggleFocus),
                    KeyCode::Char(c) if !ctrl => Some(ChatViewMessage::Input(c)),
                    KeyCode::Char('k') if ctrl => Some(ChatViewMessage::DeleteToEnd),
                    KeyCode::Char('u') if ctrl => Some(ChatViewMessage::DeleteToStart),
                    KeyCode::Backspace => Some(ChatViewMessage::Backspace),
                    KeyCode::Delete => Some(ChatViewMessage::Delete),
                    KeyCode::Left => Some(ChatViewMessage::Left),
                    KeyCode::Right => Some(ChatViewMessage::Right),
                    KeyCode::Up => Some(ChatViewMessage::Up),
                    KeyCode::Down => Some(ChatViewMessage::Down),
                    KeyCode::Home if ctrl => Some(ChatViewMessage::InputStart),
                    KeyCode::Home => Some(ChatViewMessage::Home),
                    KeyCode::End if ctrl => Some(ChatViewMessage::InputEnd),
                    KeyCode::End => Some(ChatViewMessage::End),
                    _ => None,
                }
            }
            Focus::History => match key.code {
                KeyCode::Up | KeyCode::Char('k') => Some(ChatViewMessage::ScrollUp),
                KeyCode::Down | KeyCode::Char('j') => Some(ChatViewMessage::ScrollDown),
                KeyCode::Home => Some(ChatViewMessage::ScrollToTop),
                KeyCode::End => Some(ChatViewMessage::ScrollToBottom),
                KeyCode::Tab => Some(ChatViewMessage::ToggleFocus),
                _ => None,
            },
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        if state.disabled {
            return None;
        }

        match msg {
            ChatViewMessage::Input(c) => {
                state.input.update(TextAreaMessage::Insert(c));
                Some(ChatViewOutput::InputChanged(state.input.value()))
            }
            ChatViewMessage::NewLine => {
                state.input.update(TextAreaMessage::NewLine);
                Some(ChatViewOutput::InputChanged(state.input.value()))
            }
            ChatViewMessage::Backspace => {
                if let Some(TextAreaOutput::Changed(_)) =
                    state.input.update(TextAreaMessage::Backspace)
                {
                    Some(ChatViewOutput::InputChanged(state.input.value()))
                } else {
                    None
                }
            }
            ChatViewMessage::Delete => {
                if let Some(TextAreaOutput::Changed(_)) =
                    state.input.update(TextAreaMessage::Delete)
                {
                    Some(ChatViewOutput::InputChanged(state.input.value()))
                } else {
                    None
                }
            }
            ChatViewMessage::Left => {
                state.input.update(TextAreaMessage::Left);
                None
            }
            ChatViewMessage::Right => {
                state.input.update(TextAreaMessage::Right);
                None
            }
            ChatViewMessage::Up => {
                state.input.update(TextAreaMessage::Up);
                None
            }
            ChatViewMessage::Down => {
                state.input.update(TextAreaMessage::Down);
                None
            }
            ChatViewMessage::Home => {
                state.input.update(TextAreaMessage::Home);
                None
            }
            ChatViewMessage::End => {
                state.input.update(TextAreaMessage::End);
                None
            }
            ChatViewMessage::InputStart => {
                state.input.update(TextAreaMessage::TextStart);
                None
            }
            ChatViewMessage::InputEnd => {
                state.input.update(TextAreaMessage::TextEnd);
                None
            }
            ChatViewMessage::DeleteToEnd => {
                if let Some(TextAreaOutput::Changed(_)) =
                    state.input.update(TextAreaMessage::DeleteToEnd)
                {
                    Some(ChatViewOutput::InputChanged(state.input.value()))
                } else {
                    None
                }
            }
            ChatViewMessage::DeleteToStart => {
                if let Some(TextAreaOutput::Changed(_)) =
                    state.input.update(TextAreaMessage::DeleteToStart)
                {
                    Some(ChatViewOutput::InputChanged(state.input.value()))
                } else {
                    None
                }
            }
            ChatViewMessage::Submit => {
                let value = state.input.value();
                if value.trim().is_empty() {
                    return None;
                }
                state.push_user(&value);
                state.input.update(TextAreaMessage::Clear);
                Some(ChatViewOutput::Submitted(value))
            }
            ChatViewMessage::ToggleFocus => {
                match state.focus {
                    Focus::Input => {
                        state.focus = Focus::History;
                        state.input.set_focused(false);
                    }
                    Focus::History => {
                        state.focus = Focus::Input;
                        state.input.set_focused(true);
                    }
                }
                None
            }
            ChatViewMessage::FocusInput => {
                state.focus = Focus::Input;
                state.input.set_focused(true);
                None
            }
            ChatViewMessage::FocusHistory => {
                state.focus = Focus::History;
                state.input.set_focused(false);
                None
            }
            ChatViewMessage::ScrollUp => {
                state.scroll.set_content_length(state.messages.len());
                state.scroll.scroll_up();
                state.auto_scroll = false;
                None
            }
            ChatViewMessage::ScrollDown => {
                state.scroll.set_content_length(state.messages.len());
                state.scroll.scroll_down();
                if state.scroll.at_end() {
                    state.auto_scroll = true;
                }
                None
            }
            ChatViewMessage::ScrollToTop => {
                state.scroll.set_content_length(state.messages.len());
                state.scroll.scroll_to_start();
                state.auto_scroll = false;
                None
            }
            ChatViewMessage::ScrollToBottom => {
                state.scroll.set_content_length(state.messages.len());
                state.scroll.scroll_to_end();
                state.auto_scroll = true;
                None
            }
            ChatViewMessage::ClearInput => {
                state.input.update(TextAreaMessage::Clear);
                Some(ChatViewOutput::InputChanged(String::new()))
            }
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, ctx: &ViewContext) {
        if area.height < 4 {
            return;
        }

        crate::annotation::with_registry(|reg| {
            reg.open(
                area,
                crate::annotation::Annotation::container("chat_view")
                    .with_focus(ctx.focused)
                    .with_disabled(ctx.disabled),
            );
        });

        // Layout: history + input
        let input_h = state.input_height + 2; // +2 for border
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(input_h)])
            .split(area);

        let history_area = chunks[0];
        let input_area = chunks[1];

        render_helpers::render_history(
            state,
            frame,
            history_area,
            theme,
            ctx.focused,
            ctx.disabled,
        );
        render_helpers::render_input(state, frame, input_area, theme, ctx.focused, ctx.disabled);

        crate::annotation::with_registry(|reg| {
            reg.close();
        });
    }
}

impl Focusable for ChatView {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
        if focused && state.focus == Focus::Input {
            state.input.set_focused(true);
        }
    }
}

impl Disableable for ChatView {
    fn is_disabled(state: &Self::State) -> bool {
        state.disabled
    }

    fn set_disabled(state: &mut Self::State, disabled: bool) {
        state.disabled = disabled;
    }
}
