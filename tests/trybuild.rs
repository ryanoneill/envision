//! Compiler-driven tests (trybuild). All trybuild cases live in this one
//! `TestCases`: separate test binaries share trybuild's project directory
//! (target/tests/trybuild/envision) and race when run in parallel.
//!
//! Cases:
//! - App args design: forgetting `with_args` for non-`()` Args types is a
//!   compile error, not a runtime panic (compile-fail, pinned by `.stderr`).
//! - Backend error contract: a foreign error type wrapped in a local newtype
//!   compiles and runs (positive case, no `.stderr`).
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
fn trybuild_cases() {
    let t = trybuild::TestCases::new();
    // App args design: forgetting `with_args` for non-() Args is a compile error.
    t.compile_fail("tests/trybuild_app_args/missing_with_args.rs");
    // Backend error contract: a foreign error type via a local newtype compiles and runs.
    t.pass("tests/trybuild_backend/foreign_error.rs");
}
