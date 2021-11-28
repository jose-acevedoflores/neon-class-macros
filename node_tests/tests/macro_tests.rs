#[test]
fn try_build() {
    let t = trybuild::TestCases::new();
    t.pass("./src/derived_class.rs");
}

#[test]
fn enforce_one_constructor() {
    let t = trybuild::TestCases::new();
    t.compile_fail("./src/errors/multiple_ctor_error.rs");
}
