//! Compile-fail tests for the App args design.
//!
//! Pins the compile-time enforcement promise — forgetting `with_args` for
//! non-`()` Args types is a compile error, not a runtime panic.
//!
//! Skipped under `cargo tarpaulin`: trybuild compiles a separate test crate
//! with cargo, which contributes no line coverage and can exceed the coverage
//! run's per-test timeout. It runs in every regular test job. See the
//! "Coverage" section of CONTRIBUTING.md.

#[test]
#[cfg_attr(
    tarpaulin,
    ignore = "trybuild invokes cargo; not meaningful under coverage instrumentation"
)]
fn compile_fail_app_args() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/trybuild_app_args/missing_with_args.rs");
}
