//! Overlay action types.

/// The result of an overlay handling an input event.
///
/// This tells the runtime what to do after an overlay processes an event.
pub enum OverlayAction<M> {
    /// Event was consumed by the overlay, stop propagation.
    Consumed,
    /// Event produced a message; keep the overlay and dispatch it through `update()`.
    KeepAndMessage(M),
    /// Dismiss (pop) this overlay, event is consumed.
    Dismiss,
    /// Dismiss this overlay and dispatch a message.
    DismissWithMessage(M),
    /// Pass the event to the next overlay or to the app.
    Propagate,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlay_action_variants() {
        // Verify all variants can be constructed
        let _consumed: OverlayAction<i32> = OverlayAction::Consumed;
        let _message: OverlayAction<i32> = OverlayAction::KeepAndMessage(42);
        let _dismiss: OverlayAction<i32> = OverlayAction::Dismiss;
        let _dismiss_with: OverlayAction<i32> = OverlayAction::DismissWithMessage(42);
        let _propagate: OverlayAction<i32> = OverlayAction::Propagate;
    }
}
