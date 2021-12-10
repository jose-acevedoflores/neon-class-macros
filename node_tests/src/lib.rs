use crate::derived_class::{register_standalone_function, register_test};
use neon::prelude::{ModuleContext, NeonResult};

mod derived_class;

// This is not really a feature, used just to signal the IDE to include the source files.
#[cfg(feature = "error_try_builds")]
mod errors;

#[neon::main]
fn node_entrypoint(mut cx: ModuleContext) -> NeonResult<()> {
    register_test(&mut cx)?;
    register_standalone_function(&mut cx)?;
    derived_class::TestStruct::register_constructor(&mut cx)?;
    derived_class::TestStruct2::register_constructor_with_cx(&mut cx)?;
    Ok(())
}
// Hack so this file can be included in the src/lib.rs Examples section.
