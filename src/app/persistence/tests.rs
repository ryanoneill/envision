use std::io::Write;

use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

use super::*;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct TestState {
    counter: i32,
    name: String,
}

#[tokio::test]
async fn test_load_state_success() {
    let state = TestState {
        counter: 42,
        name: "hello".into(),
    };
    let json = serde_json::to_string(&state).unwrap();

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(json.as_bytes()).unwrap();

    let loaded: TestState = load_state(file.path()).await.unwrap();
    assert_eq!(loaded, state);
}

#[tokio::test]
async fn test_load_state_file_not_found() {
    let result: Result<TestState, _> = load_state("/nonexistent/path/state.json").await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, EnvisionError::Io(_)));
}

#[tokio::test]
async fn test_load_state_invalid_json() {
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(b"not valid json {{{").unwrap();

    let result: Result<TestState, _> = load_state(file.path()).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, EnvisionError::Config { .. }));
}

#[tokio::test]
async fn test_load_state_wrong_shape() {
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(b"{\"x\": 1, \"y\": 2}").unwrap();

    let result: Result<TestState, _> = load_state(file.path()).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, EnvisionError::Config { .. }));
}

#[tokio::test]
async fn test_load_state_error_message_contains_path() {
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(b"invalid").unwrap();
    let path_str = file.path().display().to_string();

    let result: Result<TestState, _> = load_state(file.path()).await;
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains(&path_str),
        "error message should contain path, got: {}",
        msg
    );
}

#[tokio::test]
async fn test_load_state_empty_file() {
    let file = NamedTempFile::new().unwrap();

    let result: Result<TestState, _> = load_state(file.path()).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, EnvisionError::Config { .. }));
}
