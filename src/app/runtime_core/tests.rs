use super::*;

use crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;

use crate::backend::CaptureBackend;
use crate::input::Event;
use crate::overlay::{Overlay, OverlayAction};
use crate::theme::Theme;

// ========== Test App ==========

struct TestApp;

#[derive(Clone, Default)]
struct TestState {
    value: String,
}

#[derive(Clone, Debug, PartialEq)]
enum TestMsg {
    Set(String),
    FromOverlay(String),
}

impl App for TestApp {
    type State = TestState;
    type Message = TestMsg;

    fn init() -> (Self::State, super::super::command::Command<Self::Message>) {
        (TestState::default(), super::super::command::Command::none())
    }

    fn update(
        state: &mut Self::State,
        msg: Self::Message,
    ) -> super::super::command::Command<Self::Message> {
        match msg {
            TestMsg::Set(v) => state.value = v,
            TestMsg::FromOverlay(v) => state.value = format!("overlay:{v}"),
        }
        super::super::command::Command::none()
    }

    fn view(state: &Self::State, frame: &mut ratatui::Frame) {
        let text = if state.value.is_empty() {
            "Hello".to_string()
        } else {
            state.value.clone()
        };
        frame.render_widget(Paragraph::new(text), frame.area());
    }

    fn handle_event(event: &Event) -> Option<Self::Message> {
        if let Some(key) = event.as_key() {
            if let KeyCode::Char(c) = key.code {
                return Some(TestMsg::Set(c.to_string()));
            }
        }
        None
    }
}

// ========== Test Overlays ==========

struct ConsumingOverlay;

impl Overlay<TestMsg> for ConsumingOverlay {
    fn handle_event(&mut self, _event: &Event) -> OverlayAction<TestMsg> {
        OverlayAction::Consumed
    }

    fn view(&self, _frame: &mut ratatui::Frame, _area: Rect, _theme: &Theme) {}
}

struct MessageOverlay {
    msg: String,
}

impl Overlay<TestMsg> for MessageOverlay {
    fn handle_event(&mut self, _event: &Event) -> OverlayAction<TestMsg> {
        OverlayAction::Message(TestMsg::FromOverlay(self.msg.clone()))
    }

    fn view(&self, _frame: &mut ratatui::Frame, _area: Rect, _theme: &Theme) {}
}

struct DismissOverlay;

impl Overlay<TestMsg> for DismissOverlay {
    fn handle_event(&mut self, _event: &Event) -> OverlayAction<TestMsg> {
        OverlayAction::Dismiss
    }

    fn view(&self, _frame: &mut ratatui::Frame, _area: Rect, _theme: &Theme) {}
}

struct DismissWithMsgOverlay {
    msg: String,
}

impl Overlay<TestMsg> for DismissWithMsgOverlay {
    fn handle_event(&mut self, _event: &Event) -> OverlayAction<TestMsg> {
        OverlayAction::DismissWithMessage(TestMsg::FromOverlay(self.msg.clone()))
    }

    fn view(&self, _frame: &mut ratatui::Frame, _area: Rect, _theme: &Theme) {}
}

struct PropagateOverlay;

impl Overlay<TestMsg> for PropagateOverlay {
    fn handle_event(&mut self, _event: &Event) -> OverlayAction<TestMsg> {
        OverlayAction::Propagate
    }

    fn view(&self, _frame: &mut ratatui::Frame, _area: Rect, _theme: &Theme) {}
}

// ========== Helper ==========

fn new_core() -> RuntimeCore<TestApp, CaptureBackend> {
    let (state, _cmd) = TestApp::init();
    let backend = CaptureBackend::new(40, 10);
    let terminal = ratatui::Terminal::new(backend).unwrap();

    RuntimeCore {
        state,
        terminal,
        events: EventQueue::new(),
        overlay_stack: OverlayStack::new(),
        theme: Theme::default(),
        should_quit: false,
        max_messages_per_tick: 100,
    }
}

// ========== Tests ==========

#[test]
fn test_render_succeeds() {
    let mut core = new_core();
    core.render().unwrap();

    let output = core.terminal.backend().to_string();
    assert!(output.contains("Hello"));
}

#[test]
fn test_process_event_no_event() {
    let mut core = new_core();
    let result = core.process_event();
    assert!(matches!(result, ProcessEventResult::NoEvent));
}

#[test]
fn test_process_event_no_overlay_dispatches() {
    let mut core = new_core();
    core.events.push(Event::char('x'));

    let result = core.process_event();
    assert!(matches!(result, ProcessEventResult::Dispatch(TestMsg::Set(ref s)) if s == "x"));
}

#[test]
fn test_process_event_no_overlay_unhandled_event() {
    let mut core = new_core();
    core.events.push(Event::Resize(80, 24));

    let result = core.process_event();
    assert!(matches!(result, ProcessEventResult::Consumed));
}

#[test]
fn test_process_event_consuming_overlay() {
    let mut core = new_core();
    core.push_overlay(Box::new(ConsumingOverlay));
    core.events.push(Event::char('x'));

    let result = core.process_event();
    assert!(matches!(result, ProcessEventResult::Consumed));
}

#[test]
fn test_process_event_message_overlay() {
    let mut core = new_core();
    core.push_overlay(Box::new(MessageOverlay {
        msg: "hello".to_string(),
    }));
    core.events.push(Event::char('x'));

    let result = core.process_event();
    assert!(
        matches!(result, ProcessEventResult::Dispatch(TestMsg::FromOverlay(ref s)) if s == "hello")
    );
    assert_eq!(core.overlay_count(), 1);
}

#[test]
fn test_process_event_dismiss_overlay() {
    let mut core = new_core();
    core.push_overlay(Box::new(DismissOverlay));
    assert_eq!(core.overlay_count(), 1);

    core.events.push(Event::char('x'));
    let result = core.process_event();

    assert!(matches!(result, ProcessEventResult::Consumed));
    assert_eq!(core.overlay_count(), 0);
}

#[test]
fn test_process_event_dismiss_with_message_overlay() {
    let mut core = new_core();
    core.push_overlay(Box::new(DismissWithMsgOverlay {
        msg: "bye".to_string(),
    }));
    assert_eq!(core.overlay_count(), 1);

    core.events.push(Event::char('x'));
    let result = core.process_event();

    assert!(
        matches!(result, ProcessEventResult::Dispatch(TestMsg::FromOverlay(ref s)) if s == "bye")
    );
    assert_eq!(core.overlay_count(), 0);
}

#[test]
fn test_process_event_propagate_overlay() {
    let mut core = new_core();
    core.push_overlay(Box::new(PropagateOverlay));
    core.events.push(Event::char('z'));

    let result = core.process_event();
    assert!(matches!(result, ProcessEventResult::Dispatch(TestMsg::Set(ref s)) if s == "z"));
    assert_eq!(core.overlay_count(), 1);
}

#[test]
fn test_push_and_pop_overlay() {
    let mut core = new_core();
    assert!(!core.has_overlays());
    assert_eq!(core.overlay_count(), 0);

    core.push_overlay(Box::new(ConsumingOverlay));
    assert!(core.has_overlays());
    assert_eq!(core.overlay_count(), 1);

    core.push_overlay(Box::new(PropagateOverlay));
    assert_eq!(core.overlay_count(), 2);

    let popped = core.pop_overlay();
    assert!(popped.is_some());
    assert_eq!(core.overlay_count(), 1);

    let popped = core.pop_overlay();
    assert!(popped.is_some());
    assert_eq!(core.overlay_count(), 0);
    assert!(!core.has_overlays());

    assert!(core.pop_overlay().is_none());
}

#[test]
fn test_clear_overlays() {
    let mut core = new_core();
    core.push_overlay(Box::new(ConsumingOverlay));
    core.push_overlay(Box::new(PropagateOverlay));
    core.push_overlay(Box::new(DismissOverlay));
    assert_eq!(core.overlay_count(), 3);

    core.clear_overlays();
    assert_eq!(core.overlay_count(), 0);
    assert!(!core.has_overlays());
}
