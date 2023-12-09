use trybuild::TestCases;

#[test]
fn test_compile_fail() {
    let t = TestCases::new();
    t.pass("tests/0*.rs");
    t.compile_fail("tests/failed/*.rs")
}
