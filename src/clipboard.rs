//! System clipboard integration.
//!
//! Provides thread-safe clipboard access using a thread-local cached context.
//! This avoids repeated COM initialization/teardown on Windows, which can cause
//! heap corruption (`STATUS_HEAP_CORRUPTION`) when multiple threads access the
//! clipboard concurrently (e.g., during parallel test execution).

use std::cell::RefCell;

thread_local! {
    static CLIPBOARD: RefCell<Option<arboard::Clipboard>> =
        RefCell::new(arboard::Clipboard::new().ok());
}

/// Runs a closure with access to the thread-local clipboard context.
///
/// Returns `None` if the clipboard is unavailable (headless environments,
/// CI, SSH sessions without a clipboard provider).
fn with_clipboard<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut arboard::Clipboard) -> R,
{
    CLIPBOARD.with(|cb| cb.borrow_mut().as_mut().map(f))
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
        // The root cause of STATUS_HEAP_CORRUPTION was repeated Clipboard::new()
        // calls. This test verifies that repeated access on the same thread
        // reuses the cached context without issues.
        for _ in 0..100 {
            system_clipboard_set("repeated");
            let _ = system_clipboard_get();
        }
    }
}
