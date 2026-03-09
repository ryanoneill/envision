//! System clipboard integration.
//!
//! Provides thread-safe clipboard access using a process-global singleton.
//! On Windows, `arboard::Clipboard::new()` initializes COM. When multiple
//! threads call `Clipboard::new()` concurrently (e.g., during parallel test
//! execution), the concurrent COM initialization can corrupt the heap,
//! causing `STATUS_HEAP_CORRUPTION` (0xc0000374).
//!
//! This module solves the problem by creating exactly one `Clipboard`
//! instance for the entire process, serializing all access through a `Mutex`.
//! Clipboard operations are infrequent (only on user copy/cut/paste), so
//! the serialization has no practical performance impact.

use std::sync::{Mutex, OnceLock};

/// Process-global clipboard singleton.
///
/// `OnceLock` ensures `Clipboard::new()` is called exactly once.
/// `Mutex` serializes all subsequent access.
static CLIPBOARD: OnceLock<Mutex<Option<arboard::Clipboard>>> = OnceLock::new();

/// Runs a closure with access to the global clipboard context.
///
/// Returns `None` if the clipboard is unavailable (headless environments,
/// CI, SSH sessions without a clipboard provider) or if the mutex is poisoned.
fn with_clipboard<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut arboard::Clipboard) -> R,
{
    let mutex = CLIPBOARD.get_or_init(|| Mutex::new(arboard::Clipboard::new().ok()));
    let mut guard = mutex.lock().ok()?;
    guard.as_mut().map(f)
}

/// Attempt to write text to the system clipboard.
///
/// Errors are silently ignored — this is best-effort. Falls back gracefully
/// in headless environments (CI, SSH) where no clipboard provider exists.
pub(crate) fn system_clipboard_set(text: &str) {
    with_clipboard(|cb| {
        let _ = cb.set_text(text);
    });
}

/// Attempt to read text from the system clipboard.
///
/// Returns `None` if the clipboard is unavailable or doesn't contain text.
pub(crate) fn system_clipboard_get() -> Option<String> {
    with_clipboard(|cb| cb.get_text().ok())
        .flatten()
        .filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_set_does_not_panic() {
        // Should not panic even in headless CI environments
        system_clipboard_set("test");
    }

    #[test]
    fn test_clipboard_get_does_not_panic() {
        // Should return None in headless CI, Some in desktop environments
        let _ = system_clipboard_get();
    }

    #[test]
    fn test_clipboard_roundtrip() {
        // Set and get — may return None in headless environments.
        // Cannot assert exact content because the system clipboard is
        // global and other parallel tests may write to it.
        system_clipboard_set("roundtrip_test");
        let _ = system_clipboard_get();
    }

    #[test]
    fn test_with_clipboard_returns_none_gracefully() {
        // Verify the with_clipboard wrapper handles unavailable clipboard
        let result = with_clipboard(|cb| cb.get_text().ok());
        // Result is either Some(Some/None) or None — both are fine
        let _ = result;
    }

    #[test]
    fn test_repeated_access_same_thread() {
        // Verify that repeated access reuses the singleton without issues.
        for _ in 0..100 {
            system_clipboard_set("repeated");
            let _ = system_clipboard_get();
        }
    }
}
