//! Compile-fail tests for the App args design.
//!
//! Pins the compile-time enforcement promise — forgetting `with_args` for
//! non-`()` Args types is a compile error, not a runtime panic.

#[test]
fn compile_fail_app_args() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/trybuild_app_args/missing_with_args.rs");
}
