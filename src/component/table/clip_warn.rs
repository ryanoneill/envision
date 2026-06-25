//! Transient observability state for clip-warning dedup.
//!
//! Kept in a sibling module to keep `mod.rs` from creeping toward the
//! 1000-line cap. Used exclusively from the render path; not exposed
//! beyond `pub(super)`.

use std::collections::HashSet;

#[derive(Default, Debug, Clone)]
pub(super) struct ClipWarnState {
    pub(super) last_area_width: Option<u16>,
    pub(super) warned_cols: HashSet<usize>,
}
