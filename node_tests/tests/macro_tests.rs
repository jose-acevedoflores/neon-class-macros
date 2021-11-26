#[test]
fn try_build() {
    let t = trybuild::TestCases::new();
    t.pass("./src/derived_class.rs");
}
