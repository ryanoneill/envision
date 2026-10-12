//! Positive compile tests (no .stderr files: they must simply compile and run).
#[test]
#[cfg_attr(
    tarpaulin,
    ignore = "trybuild invokes cargo; not meaningful under coverage instrumentation"
)]
fn backend_contracts_compile() {
    let t = trybuild::TestCases::new();
    t.pass("tests/trybuild_backend/foreign_error.rs");
}
