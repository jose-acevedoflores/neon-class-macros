use neon::prelude::{Context, FunctionContext, JsPromise, JsResult, ModuleContext, NeonResult};
use neon::types::JsUndefined;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Serialize, Debug, Deserialize)]
struct KV {
    k: String,
    v: String,
}

#[derive(Serialize, Deserialize)]
/// Contain a list of key/value pairs.
struct DllMap {
    map: Vec<KV>,
}

impl TryFrom<DllMap> for HashMap<String, PathBuf> {
    type Error = String;

    fn try_from(value: DllMap) -> Result<Self, Self::Error> {
        let mut m = HashMap::new();
        for KV { k, v } in value.map {
            m.insert(k, v.into());
        }
        Ok(m)
    }
}
#[allow(dead_code)]
#[derive(neon_macros::Class)]
pub struct TestStruct {
    path_to_exe: PathBuf,
    dll_path_map: HashMap<String, PathBuf>,
}

impl TestStruct {
    #[neon_macros::constructor(expose)]
    fn constructor(path_to_exe: String, dll_path_map: DllMap) -> Result<Self, String> {
        let dll_path_map = dll_path_map.try_into()?;

        Ok(Self {
            path_to_exe: path_to_exe.into(),
            dll_path_map,
        })
    }

    #[neon_macros::method]
    fn start_camel<'ctx>(
        &self,
        mut cx: FunctionContext<'ctx>,
        num: u32,
    ) -> JsResult<'ctx, JsPromise> {
        let chan = cx.channel();
        let (def, p) = cx.promise();

        std::thread::spawn(move || {
            chan.settle_with(def, move |cx| Ok(cx.number(num * 2)));
        });

        Ok(p)
    }
}

#[allow(unused)]
fn main() {}

fn test(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    Ok(cx.undefined())
}

#[neon::main]
fn node_entrypoint(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("test", test)?;

    Ok(())
}
