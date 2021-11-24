use neon::prelude::{Context, Finalize, FunctionContext, JsPromise, JsResult};
use neon::types::{JsNumber, JsString};
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
pub struct DllMap {
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

impl Finalize for TestStruct {}

#[neon_macros::impl_block]
impl TestStruct {
    #[neon_macros::constructor]
    pub fn constructor(path_to_exe: String, dll_path_map: DllMap) -> Result<Self, String> {
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

    #[neon_macros::method]
    fn another_one<'ctx>(
        &self,
        mut cx: FunctionContext<'ctx>,
        num: u32,
        msg: String,
    ) -> JsResult<'ctx, JsString> {
        let res = format!("hehe {}-{}-{:?}", msg, num, self.path_to_exe);
        Ok(cx.string(res))
    }
}

#[allow(unused)]
fn main() {}

pub(crate) fn test(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(cx.number(4))
}
