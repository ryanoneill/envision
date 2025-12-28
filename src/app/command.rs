//! Commands for handling side effects in TEA applications.
//!
//! Commands represent side effects that should be executed after
//! an update. They're the bridge between pure state updates and
//! the outside world (IO, network, etc.).

/// A command that can produce messages or perform side effects.
///
/// Commands are returned from `update` functions to trigger
/// asynchronous operations or batch multiple messages.
#[derive(Default)]
pub struct Command<M> {
    actions: Vec<CommandAction<M>>,
}

enum CommandAction<M> {
    /// A message to dispatch immediately
    Message(M),

    /// A batch of messages to dispatch
    Batch(Vec<M>),

    /// Quit the application
    Quit,

    /// A callback to execute (for sync side effects)
    Callback(Box<dyn FnOnce() -> Option<M> + Send + 'static>),
}

impl<M> Command<M> {
    /// Creates an empty command (no-op).
    pub fn none() -> Self {
        Self {
            actions: Vec::new(),
        }
    }

    /// Returns true if this command has no actions.
    pub fn is_none(&self) -> bool {
        self.actions.is_empty()
    }

    /// Creates a command that dispatches a single message.
    pub fn message(msg: M) -> Self {
        Self {
            actions: vec![CommandAction::Message(msg)],
        }
    }

    /// Creates a command that dispatches multiple messages.
    pub fn batch(messages: impl IntoIterator<Item = M>) -> Self {
        let msgs: Vec<M> = messages.into_iter().collect();
        if msgs.is_empty() {
            Self::none()
        } else {
            Self {
                actions: vec![CommandAction::Batch(msgs)],
            }
        }
    }

    /// Creates a command that quits the application.
    pub fn quit() -> Self {
        Self {
            actions: vec![CommandAction::Quit],
        }
    }

    /// Creates a command from a synchronous callback.
    ///
    /// The callback will be executed and may optionally return a message.
    pub fn perform<F>(f: F) -> Self
    where
        F: FnOnce() -> Option<M> + Send + 'static,
    {
        Self {
            actions: vec![CommandAction::Callback(Box::new(f))],
        }
    }

    /// Combines multiple commands into one.
    pub fn combine(commands: impl IntoIterator<Item = Command<M>>) -> Self {
        let mut actions = Vec::new();
        for cmd in commands {
            actions.extend(cmd.actions);
        }
        Self { actions }
    }

    /// Appends another command to this one.
    pub fn and(mut self, other: Command<M>) -> Self {
        self.actions.extend(other.actions);
        self
    }

    /// Maps the message type to a different type.
    pub fn map<N, F>(self, f: F) -> Command<N>
    where
        F: Fn(M) -> N + Clone + Send + 'static,
        M: 'static,
        N: 'static,
    {
        let actions = self
            .actions
            .into_iter()
            .map(|action| match action {
                CommandAction::Message(m) => CommandAction::Message(f(m)),
                CommandAction::Batch(msgs) => {
                    CommandAction::Batch(msgs.into_iter().map(|m| f.clone()(m)).collect())
                }
                CommandAction::Quit => CommandAction::Quit,
                CommandAction::Callback(cb) => {
                    let f = f.clone();
                    CommandAction::Callback(Box::new(move || cb().map(|m| f(m))))
                }
            })
            .collect();

        Command { actions }
    }
}

impl<M: Clone> Clone for Command<M> {
    fn clone(&self) -> Self {
        // Note: Callbacks and Async can't be cloned, so we only clone Message/Batch/Quit
        let actions = self
            .actions
            .iter()
            .filter_map(|action| match action {
                CommandAction::Message(m) => Some(CommandAction::Message(m.clone())),
                CommandAction::Batch(msgs) => Some(CommandAction::Batch(msgs.clone())),
                CommandAction::Quit => Some(CommandAction::Quit),
                _ => None, // Skip non-clonable actions
            })
            .collect();

        Self { actions }
    }
}

impl<M> std::fmt::Debug for Command<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command")
            .field("action_count", &self.actions.len())
            .finish()
    }
}

/// Handles execution of commands.
///
/// This is used by the runtime to process commands returned from updates.
pub struct CommandHandler<M> {
    pending_messages: Vec<M>,
    should_quit: bool,
}

impl<M> CommandHandler<M> {
    /// Creates a new command handler.
    pub fn new() -> Self {
        Self {
            pending_messages: Vec::new(),
            should_quit: false,
        }
    }

    /// Executes a command and collects any resulting messages.
    pub fn execute(&mut self, command: Command<M>) {
        for action in command.actions {
            match action {
                CommandAction::Message(m) => {
                    self.pending_messages.push(m);
                }
                CommandAction::Batch(msgs) => {
                    self.pending_messages.extend(msgs);
                }
                CommandAction::Quit => {
                    self.should_quit = true;
                }
                CommandAction::Callback(cb) => {
                    if let Some(m) = cb() {
                        self.pending_messages.push(m);
                    }
                }
            }
        }
    }

    /// Takes all pending messages.
    pub fn take_messages(&mut self) -> Vec<M> {
        std::mem::take(&mut self.pending_messages)
    }

    /// Returns true if a quit command was executed.
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Resets the quit flag.
    pub fn reset_quit(&mut self) {
        self.should_quit = false;
    }
}

impl<M> Default for CommandHandler<M> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {
        A,
        B,
        C,
    }

    #[test]
    fn test_command_none() {
        let cmd: Command<TestMsg> = Command::none();
        assert!(cmd.is_none());
    }

    #[test]
    fn test_command_message() {
        let cmd = Command::message(TestMsg::A);
        assert!(!cmd.is_none());
    }

    #[test]
    fn test_command_batch() {
        let cmd = Command::batch([TestMsg::A, TestMsg::B, TestMsg::C]);
        assert!(!cmd.is_none());
    }

    #[test]
    fn test_command_handler_message() {
        let mut handler = CommandHandler::new();
        handler.execute(Command::message(TestMsg::A));

        let messages = handler.take_messages();
        assert_eq!(messages, vec![TestMsg::A]);
    }

    #[test]
    fn test_command_handler_batch() {
        let mut handler = CommandHandler::new();
        handler.execute(Command::batch([TestMsg::A, TestMsg::B]));

        let messages = handler.take_messages();
        assert_eq!(messages, vec![TestMsg::A, TestMsg::B]);
    }

    #[test]
    fn test_command_handler_quit() {
        let mut handler: CommandHandler<TestMsg> = CommandHandler::new();
        assert!(!handler.should_quit());

        handler.execute(Command::quit());
        assert!(handler.should_quit());
    }

    #[test]
    fn test_command_combine() {
        let cmd = Command::combine([
            Command::message(TestMsg::A),
            Command::message(TestMsg::B),
        ]);

        let mut handler = CommandHandler::new();
        handler.execute(cmd);

        let messages = handler.take_messages();
        assert_eq!(messages, vec![TestMsg::A, TestMsg::B]);
    }

    #[test]
    fn test_command_and() {
        let cmd = Command::message(TestMsg::A).and(Command::message(TestMsg::B));

        let mut handler = CommandHandler::new();
        handler.execute(cmd);

        let messages = handler.take_messages();
        assert_eq!(messages, vec![TestMsg::A, TestMsg::B]);
    }

    #[test]
    fn test_command_perform() {
        let cmd = Command::perform(|| Some(TestMsg::A));

        let mut handler = CommandHandler::new();
        handler.execute(cmd);

        let messages = handler.take_messages();
        assert_eq!(messages, vec![TestMsg::A]);
    }

    #[test]
    fn test_command_perform_none() {
        let cmd: Command<TestMsg> = Command::perform(|| None);

        let mut handler = CommandHandler::new();
        handler.execute(cmd);

        let messages = handler.take_messages();
        assert!(messages.is_empty());
    }

    #[test]
    fn test_command_map() {
        #[derive(Clone, Debug, PartialEq)]
        enum OuterMsg {
            Inner(TestMsg),
        }

        let cmd = Command::message(TestMsg::A);
        let mapped = cmd.map(OuterMsg::Inner);

        let mut handler = CommandHandler::new();
        handler.execute(mapped);

        let messages = handler.take_messages();
        assert_eq!(messages, vec![OuterMsg::Inner(TestMsg::A)]);
    }

    #[test]
    fn test_command_batch_empty() {
        let cmd: Command<TestMsg> = Command::batch(Vec::new());
        assert!(cmd.is_none());
    }

    #[test]
    fn test_command_clone_message() {
        let cmd = Command::message(TestMsg::A);
        let cloned = cmd.clone();

        let mut handler = CommandHandler::new();
        handler.execute(cloned);

        let messages = handler.take_messages();
        assert_eq!(messages, vec![TestMsg::A]);
    }

    #[test]
    fn test_command_clone_batch() {
        let cmd = Command::batch([TestMsg::A, TestMsg::B]);
        let cloned = cmd.clone();

        let mut handler = CommandHandler::new();
        handler.execute(cloned);

        let messages = handler.take_messages();
        assert_eq!(messages, vec![TestMsg::A, TestMsg::B]);
    }

    #[test]
    fn test_command_clone_quit() {
        let cmd: Command<TestMsg> = Command::quit();
        let cloned = cmd.clone();

        let mut handler = CommandHandler::new();
        handler.execute(cloned);

        assert!(handler.should_quit());
    }

    #[test]
    fn test_command_clone_callback_skipped() {
        let cmd = Command::perform(|| Some(TestMsg::A));
        let cloned = cmd.clone();

        // Callbacks can't be cloned, so cloned should have no actions
        assert!(cloned.is_none());
    }

    #[test]
    fn test_command_debug() {
        let cmd = Command::message(TestMsg::A);
        let debug_str = format!("{:?}", cmd);

        assert!(debug_str.contains("Command"));
        assert!(debug_str.contains("action_count"));
        assert!(debug_str.contains("1"));
    }

    #[test]
    fn test_command_map_batch() {
        #[derive(Clone, Debug, PartialEq)]
        enum OuterMsg {
            Inner(TestMsg),
        }

        let cmd = Command::batch([TestMsg::A, TestMsg::B]);
        let mapped = cmd.map(OuterMsg::Inner);

        let mut handler = CommandHandler::new();
        handler.execute(mapped);

        let messages = handler.take_messages();
        assert_eq!(
            messages,
            vec![OuterMsg::Inner(TestMsg::A), OuterMsg::Inner(TestMsg::B)]
        );
    }

    #[test]
    fn test_command_map_quit() {
        #[derive(Clone, Debug, PartialEq)]
        enum OuterMsg {
            Inner(TestMsg),
        }

        let cmd: Command<TestMsg> = Command::quit();
        let mapped: Command<OuterMsg> = cmd.map(OuterMsg::Inner);

        let mut handler = CommandHandler::new();
        handler.execute(mapped);

        assert!(handler.should_quit());
    }

    #[test]
    fn test_command_map_callback() {
        #[derive(Clone, Debug, PartialEq)]
        enum OuterMsg {
            Inner(TestMsg),
        }

        let cmd = Command::perform(|| Some(TestMsg::A));
        let mapped = cmd.map(OuterMsg::Inner);

        let mut handler = CommandHandler::new();
        handler.execute(mapped);

        let messages = handler.take_messages();
        assert_eq!(messages, vec![OuterMsg::Inner(TestMsg::A)]);
    }

    #[test]
    fn test_command_map_callback_none() {
        #[derive(Clone, Debug, PartialEq)]
        enum OuterMsg {
            Inner(TestMsg),
        }

        let cmd: Command<TestMsg> = Command::perform(|| None);
        let mapped: Command<OuterMsg> = cmd.map(OuterMsg::Inner);

        let mut handler = CommandHandler::new();
        handler.execute(mapped);

        let messages = handler.take_messages();
        assert!(messages.is_empty());
    }

    #[test]
    fn test_command_handler_reset_quit() {
        let mut handler: CommandHandler<TestMsg> = CommandHandler::new();
        handler.execute(Command::quit());
        assert!(handler.should_quit());

        handler.reset_quit();
        assert!(!handler.should_quit());
    }

    #[test]
    fn test_command_handler_default() {
        let handler: CommandHandler<TestMsg> = CommandHandler::default();
        assert!(!handler.should_quit());
        assert!(handler.pending_messages.is_empty());
    }
}
