use neon::prelude::Finalize;
use neon_class_macros::neon_class as cant_rename;

#[allow(dead_code)]
#[derive(neon_class_macros::Class)]
pub struct TestStruct {
    path_to_exe: String,
}

impl Finalize for TestStruct {}

#[cant_rename(impl_block)]
impl TestStruct {
    #[cant_rename(constructor)]
    pub fn constructor(path_to_exe: String) -> Result<Self, String> {
        Ok(Self { path_to_exe })
    }
}

// Needed for the try_build tests.
#[allow(unused)]
fn main() {}
