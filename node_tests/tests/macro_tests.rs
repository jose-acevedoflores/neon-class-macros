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

#[test]
fn check_rename_of_neon_class_macro_gives_error() {
    let t = trybuild::TestCases::new();
    t.compile_fail("./src/errors/rename_macro_error.rs");
}
