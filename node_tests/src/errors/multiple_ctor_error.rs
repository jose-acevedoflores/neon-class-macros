use neon::prelude::Finalize;

#[allow(dead_code)]
#[derive(neon_class_macros::Class)]
pub struct TestStruct {
    path_to_exe: String,
}

impl Finalize for TestStruct {}

#[neon_class_macros::impl_block]
impl TestStruct {
    #[neon_class_macros::constructor]
    pub fn constructor(path_to_exe: String) -> Result<Self, String> {
        Ok(Self { path_to_exe })
    }

    #[neon_class_macros::constructor]
    pub fn second_ctor_not_allowed(num: u32) -> Result<Self, String> {
        Ok(Self {
            path_to_exe: format!("{}", num),
        })
    }
}

// Needed for the try_build tests.
#[allow(unused)]
fn main() {}
