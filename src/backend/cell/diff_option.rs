//! Serializable mirror of ratatui's `CellDiffOption`.

use std::num::NonZeroU16;

use ratatui::buffer::CellDiffOption;

/// How a cell participates in buffer diffing, captured from ratatui.
///
/// `#[non_exhaustive]` — unlike [`SerializableColor`](super::SerializableColor) —
/// because ratatui's `CellDiffOption` has already grown variants and hyperlink
/// support (#125) will exercise them; the mirror keeps room to follow without
/// a breaking change.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serialization", serde(rename_all = "snake_case"))]
#[non_exhaustive]
pub enum SerializableDiffOption {
    /// No special handling.
    #[default]
    None,
    /// Skipped when diffing.
    Skip,
    /// Always re-sent, even if unchanged.
    AlwaysUpdate,
    /// Occupies exactly this many columns regardless of the symbol.
    ForcedWidth(NonZeroU16),
}

impl SerializableDiffOption {
    /// True for [`SerializableDiffOption::None`].
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

impl From<CellDiffOption> for SerializableDiffOption {
    fn from(option: CellDiffOption) -> Self {
        match option {
            CellDiffOption::None => Self::None,
            CellDiffOption::Skip => Self::Skip,
            CellDiffOption::AlwaysUpdate => Self::AlwaysUpdate,
            CellDiffOption::ForcedWidth(w) => Self::ForcedWidth(w),
        }
    }
}
