//! SerializableDiffOption mirror + lossless legacy JSON (spec § 4a/4b).

use std::num::NonZeroU16;

use envision::EnhancedCell;
use envision::backend::SerializableDiffOption;
use envision::ratatui::buffer::{Cell, CellDiffOption};

fn captured(cell: &Cell) -> SerializableDiffOption {
    EnhancedCell::from_ratatui_cell(cell, 0).diff_option()
}

// Test 8: each upstream variant maps across.
#[test]
fn upstream_variants_map_across() {
    let mut c = Cell::new("x");
    assert_eq!(captured(&c), SerializableDiffOption::None);
    c.set_diff_option(CellDiffOption::Skip);
    assert_eq!(captured(&c), SerializableDiffOption::Skip);
    c.set_diff_option(CellDiffOption::AlwaysUpdate);
    assert_eq!(captured(&c), SerializableDiffOption::AlwaysUpdate);
    let w = NonZeroU16::new(3).unwrap();
    c.set_diff_option(CellDiffOption::ForcedWidth(w));
    assert_eq!(captured(&c), SerializableDiffOption::ForcedWidth(w));
}

// Test 8: deprecated skip == true with diff_option None maps to Skip (upstream is_skip rule).
#[test]
#[allow(deprecated)] // exercising the deprecated upstream field on purpose
fn deprecated_skip_maps_to_skip() {
    let mut c = Cell::new("x");
    c.set_skip(true);
    assert_eq!(captured(&c), SerializableDiffOption::Skip);
}

// Test 8 / RF-3 (capture side): upstream honours deprecated `skip` only when
// `diff_option` is None, so an explicit option wins.
#[test]
#[allow(deprecated)] // exercising the deprecated upstream field on purpose
fn explicit_option_beats_deprecated_skip() {
    let mut c = Cell::new("x");
    c.set_skip(true);
    c.set_diff_option(CellDiffOption::AlwaysUpdate);
    assert_eq!(captured(&c), SerializableDiffOption::AlwaysUpdate);
}

#[cfg(feature = "serialization")]
mod json {
    use super::*;
    use envision::FrameSnapshot;

    const MODS: &str = r#"{"bold":false,"dim":false,"italic":false,"underlined":false,"slow_blink":false,"rapid_blink":false,"reversed":false,"hidden":false,"crossed_out":false}"#;

    fn cell_json(extra: &str) -> String {
        format!(
            r#"{{"symbol":"x","fg":"reset","bg":"reset","modifiers":{MODS},"underline_color":null,"last_modified_frame":0{extra}}}"#
        )
    }

    fn load(extra: &str) -> EnhancedCell {
        serde_json::from_str(&cell_json(extra)).unwrap()
    }

    // Test 9
    #[test]
    fn every_variant_round_trips() {
        for opt in [
            SerializableDiffOption::None,
            SerializableDiffOption::Skip,
            SerializableDiffOption::AlwaysUpdate,
            SerializableDiffOption::ForcedWidth(NonZeroU16::new(2).unwrap()),
        ] {
            let mut cell = EnhancedCell::new();
            cell.set_diff_option(opt);
            let json = serde_json::to_string(&cell).unwrap();
            let back: EnhancedCell = serde_json::from_str(&json).unwrap();
            assert_eq!(back.diff_option(), opt, "{json}");
        }
    }

    #[test]
    fn wire_format_is_snake_case() {
        let mut cell = EnhancedCell::new();
        cell.set_diff_option(SerializableDiffOption::AlwaysUpdate);
        assert!(
            serde_json::to_string(&cell)
                .unwrap()
                .contains(r#""diff_option":"always_update""#)
        );
        cell.set_diff_option(SerializableDiffOption::ForcedWidth(
            NonZeroU16::new(2).unwrap(),
        ));
        assert!(
            serde_json::to_string(&cell)
                .unwrap()
                .contains(r#""diff_option":{"forced_width":2}"#)
        );
    }

    #[test]
    fn ordinary_cells_omit_diff_option_and_skip() {
        let json = serde_json::to_string(&EnhancedCell::new()).unwrap();
        assert!(!json.contains("diff_option"), "{json}");
        assert!(!json.contains("skip"), "{json}");
    }

    #[test]
    fn legacy_skip_false_loads_as_none() {
        assert_eq!(
            load(r#","skip":false"#).diff_option(),
            SerializableDiffOption::None
        );
    }

    #[test]
    fn legacy_skip_true_loads_as_skip() {
        assert_eq!(
            load(r#","skip":true"#).diff_option(),
            SerializableDiffOption::Skip
        );
    }

    // RF-3: skip is honoured only when diff_option is None.
    #[test]
    fn legacy_skip_with_explicit_option_keeps_option() {
        let cell = load(r#","skip":true,"diff_option":"always_update""#);
        assert_eq!(cell.diff_option(), SerializableDiffOption::AlwaysUpdate);
    }

    #[test]
    fn forced_width_zero_is_rejected() {
        assert!(
            serde_json::from_str::<EnhancedCell>(&cell_json(
                r#","diff_option":{"forced_width":0}"#
            ))
            .is_err()
        );
    }

    #[test]
    fn hostile_forced_width_is_inert() {
        let cell = cell_json(r#","diff_option":{"forced_width":65535}"#);
        let json = format!(
            r#"{{"frame":0,"size":[3,1],"cursor":{{"position":[0,0],"visible":true}},"cells":[{cell},{cell},{cell}]}}"#
        );
        let snapshot: FrameSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(snapshot.row_content(0).chars().count(), 3);
        let _ = snapshot.to_plain();
        // Output size must not scale with the hostile width (the draw path is
        // covered in tests/capture_wide_glyph.rs).
        assert!(snapshot.to_ansi().len() < 4096);
    }
}
