//! Session persistence helpers for saving and loading application state.
//!
//! This module provides convenience functions for serializing application state
//! to JSON files and deserializing it back. All functions require the
//! `serialization` feature.
//!
//! # Example
//!
//! ```rust
//! use envision::app::persistence::load_state;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize, Deserialize, Default, PartialEq, Debug)]
//! struct AppState {
//!     counter: i32,
//!     name: String,
//! }
//!
//! // Save state to a file
//! let dir = std::env::temp_dir().join("envision_doc_test");
//! std::fs::create_dir_all(&dir).unwrap();
//! let path = dir.join("state.json");
//! let state = AppState { counter: 42, name: "test".into() };
//! let json = serde_json::to_string(&state).unwrap();
//! std::fs::write(&path, &json).unwrap();
//!
//! // Load it back
//! let loaded: AppState = load_state(&path).unwrap();
//! assert_eq!(loaded, state);
//! # std::fs::remove_dir_all(&dir).unwrap();
//! ```

use std::path::Path;

use serde::de::DeserializeOwned;

use crate::error::EnvisionError;

/// Loads application state from a JSON file.
///
/// Reads the file at `path`, deserializes it as JSON, and returns the
/// deserialized state. Returns [`EnvisionError::Io`] for file system errors
/// and [`EnvisionError::Config`] for deserialization errors.
///
/// # Example
///
/// ```rust
/// use envision::app::persistence::load_state;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct MyState {
///     count: i32,
/// }
///
/// // Returns EnvisionError::Io for missing files
/// let result: Result<MyState, _> = load_state("/nonexistent/path.json");
/// assert!(result.is_err());
/// ```
pub fn load_state<S: DeserializeOwned>(path: impl AsRef<Path>) -> Result<S, EnvisionError> {
    let path = path.as_ref();
    let contents = std::fs::read_to_string(path)?;
    serde_json::from_str(&contents).map_err(|e| {
        EnvisionError::config(
            path.display().to_string(),
            format!("failed to deserialize state: {}", e),
        )
    })
}

#[cfg(test)]
mod tests;
