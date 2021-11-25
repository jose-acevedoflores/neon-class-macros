use crate::derived_class::test;
use neon::prelude::{ModuleContext, NeonResult};

mod derived_class;

#[neon::main]
fn node_entrypoint(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("test", test)?;

    derived_class::TestStruct::register_constructor(&mut cx)?;
    Ok(())
}
// Hack so this file can be included in the src/lib.rs Examples section.
