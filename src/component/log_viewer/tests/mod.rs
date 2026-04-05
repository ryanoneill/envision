use super::*;
use crate::component::test_utils;

fn sample_state() -> LogViewerState {
    let mut state = LogViewerState::new();
    state.push_info("Server started");
    state.push_success("Connected to database");
    state.push_warning("Disk space low");
    state.push_error("Connection timeout");
    state
}

fn focused_state() -> LogViewerState {
    sample_state()
}

mod edge_cases;
mod events;
mod scrolling;
mod state;
mod update;
mod view;
