use super::*;
use crate::input::{Event, KeyCode};

#[derive(Clone, Debug, PartialEq)]
struct TestItem {
    id: u32,
    name: String,
}

fn make_items() -> Vec<TestItem> {
    vec![
        TestItem {
            id: 1,
            name: "Item One".to_string(),
        },
        TestItem {
            id: 2,
            name: "Item Two".to_string(),
        },
        TestItem {
            id: 3,
            name: "Item Three".to_string(),
        },
    ]
}

mod component;
mod events;
mod item;
mod state;
