#![cfg(feature = "derive")]

#[test]
fn compile_failures_are_stable() {
    let cases = trybuild::TestCases::new();

    cases.compile_fail("tests/ui/*.rs");
}
