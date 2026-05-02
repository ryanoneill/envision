//! Unified cell type for tabular components.
//!
//! `Cell` is the single cell representation used by `Table` and any
//! tabular component built on `TableRow`. A cell carries display text,
//! optional [`CellStyle`], and an optional [`SortKey`] for typed sorting.
//!
//! # Example
//!
//! ```
//! use envision::component::cell::{Cell, CellStyle, SortKey};
//!
//! let cell = Cell::new("running")
//!     .with_style(CellStyle::Success)
//!     .with_sort_key(SortKey::String("running".into()));
//! assert_eq!(cell.text(), "running");
//! ```

#![allow(dead_code)] // Placeholder during Phase 1; tasks 2–9 fill this in.
