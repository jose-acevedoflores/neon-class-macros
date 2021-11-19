#[test]
fn try_build() {
    let t = trybuild::TestCases::new();
    t.pass("derived_class/src/derived_class.rs");
}
