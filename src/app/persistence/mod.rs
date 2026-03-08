//! Session persistence helpers for saving and loading application state.
//!
//! This module provides async convenience functions for serializing application
//! state to JSON files and deserializing it back. All functions require the
//! `serialization` feature.
//!
//! # Example
//!
//! ```rust
//! # tokio_test::block_on(async {
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
//! tokio::fs::create_dir_all(&dir).await.unwrap();
//! let path = dir.join("state.json");
//! let state = AppState { counter: 42, name: "test".into() };
//! let json = serde_json::to_string(&state).unwrap();
//! tokio::fs::write(&path, &json).await.unwrap();
//!
//! // Load it back
//! let loaded: AppState = load_state(&path).await.unwrap();
//! assert_eq!(loaded, state);
//! # tokio::fs::remove_dir_all(&dir).await.unwrap();
//! # });
//! ```

use std::path::Path;

use serde::de::DeserializeOwned;

use crate::error::EnvisionError;

/// Loads application state from a JSON file asynchronously.
///
/// Reads the file at `path` using `tokio::fs`, deserializes it as JSON, and
/// returns the deserialized state.
///
/// # Errors
///
/// Returns [`EnvisionError::Io`] if the file cannot be read (e.g., the file
/// does not exist or permissions are insufficient). Returns
/// [`EnvisionError::Config`] if the file contents cannot be deserialized
/// as valid JSON matching the expected state type.
///
/// # Example
///
/// ```rust
/// # tokio_test::block_on(async {
/// use envision::app::persistence::load_state;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct MyState {
///     count: i32,
/// }
///
/// // Returns EnvisionError::Io for missing files
/// let result: Result<MyState, _> = load_state("/nonexistent/path.json").await;
/// assert!(result.is_err());
/// # });
/// ```
pub async fn load_state<S: DeserializeOwned>(path: impl AsRef<Path>) -> Result<S, EnvisionError> {
    let path = path.as_ref();
    let contents = tokio::fs::read_to_string(path).await?;
    serde_json::from_str(&contents).map_err(|e| {
        EnvisionError::config(
            path.display().to_string(),
            format!("failed to deserialize state: {}", e),
        )
    })
}

#[cfg(test)]
mod tests;
