use super::*;

#[derive(Clone, Debug, PartialEq)]
struct TestState {
    value: i32,
}

#[derive(Clone, Debug, PartialEq)]
enum TestMsg {
    Add(i32),
    Finished,
}

#[test]
fn test_update_result_none() {
    let result: UpdateResult<TestState, TestMsg> = UpdateResult::none();
    assert!(result.state.is_none());
    assert!(result.command.is_none());
}

#[test]
fn test_update_result_state() {
    let result: UpdateResult<TestState, TestMsg> = UpdateResult::state(TestState { value: 42 });
    assert_eq!(result.state.unwrap().value, 42);
    assert!(result.command.is_none());
}

#[test]
fn test_update_result_command() {
    let result: UpdateResult<TestState, TestMsg> =
        UpdateResult::command(Command::message(TestMsg::Finished));
    assert!(result.state.is_none());
    assert!(!result.command.is_none());
}

#[test]
fn test_update_result_with() {
    let result = UpdateResult::with(TestState { value: 10 }, Command::message(TestMsg::Finished));
    assert_eq!(result.state.unwrap().value, 10);
    assert!(!result.command.is_none());
}

#[test]
fn test_fn_update() {
    let updater = FnUpdate::new(|state: &mut TestState, msg: TestMsg| {
        match msg {
            TestMsg::Add(n) => state.value += n,
            TestMsg::Finished => {}
        }
        Command::none()
    });

    let mut state = TestState { value: 0 };
    updater.update(&mut state, TestMsg::Add(5));
    assert_eq!(state.value, 5);

    updater.update(&mut state, TestMsg::Add(3));
    assert_eq!(state.value, 8);
}

#[test]
fn test_map_state() {
    let result: UpdateResult<TestState, TestMsg> = UpdateResult::state(TestState { value: 42 });

    let mapped = result.map_state(|s| s.value);
    assert_eq!(mapped.state, Some(42));
}

#[test]
fn test_and_command() {
    let result: UpdateResult<TestState, TestMsg> = UpdateResult::state(TestState { value: 10 });

    let with_cmd = result.and_command(Command::message(TestMsg::Finished));
    assert!(with_cmd.state.is_some());
    assert!(!with_cmd.command.is_none());
}

#[test]
fn test_and_command_chained() {
    let result: UpdateResult<TestState, TestMsg> = UpdateResult::none()
        .and_command(Command::message(TestMsg::Add(1)))
        .and_command(Command::message(TestMsg::Add(2)));

    // Commands should be batched
    assert!(!result.command.is_none());
}

#[test]
fn test_map_message() {
    #[derive(Clone, Debug, PartialEq)]
    enum OtherMsg {
        Transformed(i32),
    }

    let result: UpdateResult<TestState, TestMsg> =
        UpdateResult::command(Command::message(TestMsg::Add(5)));

    let mapped: UpdateResult<TestState, OtherMsg> = result.map_message(|msg| match msg {
        TestMsg::Add(n) => OtherMsg::Transformed(n),
        TestMsg::Finished => OtherMsg::Transformed(0),
    });

    assert!(mapped.state.is_none());
    assert!(!mapped.command.is_none());
}

#[test]
fn test_update_result_default() {
    let result: UpdateResult<TestState, TestMsg> = UpdateResult::default();
    assert!(result.state.is_none());
    assert!(result.command.is_none());
}

#[test]
fn test_state_ext_updated() {
    let state = TestState { value: 42 };
    let result = state.updated(Command::<TestMsg>::none());

    assert!(result.state.is_some());
    assert_eq!(result.state.unwrap().value, 42);
}

#[test]
fn test_state_ext_unchanged() {
    let state = TestState { value: 100 };
    let result = state.unchanged();

    assert!(result.state.is_some());
    assert_eq!(result.state.unwrap().value, 100);
    assert!(result.command.is_none());
}

#[test]
fn test_update_result_debug() {
    let result: UpdateResult<TestState, TestMsg> = UpdateResult::state(TestState { value: 1 });
    let debug = format!("{:?}", result);
    assert!(debug.contains("UpdateResult"));
    assert!(debug.contains("state"));
}

#[test]
fn test_fn_update_with_command() {
    let updater = FnUpdate::new(|state: &mut TestState, msg: TestMsg| match msg {
        TestMsg::Add(n) => {
            state.value += n;
            Command::message(TestMsg::Finished)
        }
        TestMsg::Finished => Command::none(),
    });

    let mut state = TestState { value: 0 };
    let cmd = updater.update(&mut state, TestMsg::Add(5));

    assert_eq!(state.value, 5);
    assert!(!cmd.is_none());
}

#[test]
fn test_map_state_with_none() {
    let result: UpdateResult<TestState, TestMsg> = UpdateResult::none();
    let mapped = result.map_state(|s| s.value);
    assert!(mapped.state.is_none());
}
