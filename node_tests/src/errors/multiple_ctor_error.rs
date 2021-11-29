use neon::prelude::Finalize;
use neon_class_macros::neon_class;

#[allow(dead_code)]
#[derive(neon_class_macros::Class)]
pub struct TestStruct {
    path_to_exe: String,
}

impl Finalize for TestStruct {}

#[neon_class(impl_block)]
impl TestStruct {
    #[neon_class(constructor)]
    pub fn constructor(path_to_exe: String) -> Result<Self, String> {
        Ok(Self { path_to_exe })
    }

    #[neon_class(constructor)]
    pub fn second_ctor_not_allowed(num: u32) -> Result<Self, String> {
        Ok(Self {
            path_to_exe: format!("{}", num),
        })
    }
}

// Needed for the try_build tests.
#[allow(unused)]
fn main() {}
