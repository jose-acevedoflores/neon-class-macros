use neon::prelude::{Context, Finalize, FunctionContext, JsPromise, JsResult};
use neon::types::JsString;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
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
#[derive(neon_class_macros::Class)]
pub struct TestStruct {
    path_to_exe: PathBuf,
    dll_path_map: HashMap<String, PathBuf>,
    my_val: RefCell<i32>,
}

impl Finalize for TestStruct {}

#[neon_class_macros::impl_block]
impl TestStruct {
    #[neon_class_macros::constructor]
    pub fn constructor(path_to_exe: String, dll_path_map: DllMap) -> Result<Self, String> {
        let dll_path_map = dll_path_map.try_into()?;

        Ok(Self {
            path_to_exe: path_to_exe.into(),
            dll_path_map,
            my_val: RefCell::new(0),
        })
    }

    #[neon_class_macros::method]
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

    #[neon_class_macros::method]
    fn another_one<'ctx>(
        &self,
        mut cx: FunctionContext<'ctx>,
        num: u32,
        msg: String,
    ) -> JsResult<'ctx, JsString> {
        let res = format!("hehe {}-{}-{:?}", msg, num, self.path_to_exe);
        Ok(cx.string(res))
    }

    #[neon_class_macros::method]
    fn plain_method(&self, num: f64) -> String {
        let p = self.dll_path_map.get("LE_KEY");
        format!(
            "to-str-{}-{}",
            num,
            p.map(|p| p.to_str().unwrap().to_string())
                .unwrap_or("NONE".to_string())
        )
    }

    #[neon_class_macros::method]
    fn method_that_returns_nothing(&self) {
        println!("do something {:?}", self.path_to_exe);
    }

    #[neon_class_macros::method]
    fn take_numeric(&self, u_32: u32, i_32: i32) -> i32 {
        *self.my_val.borrow_mut() = i_32;
        i_32 + u_32 as i32
    }
}

#[allow(unused)]
fn main() {}

pub(crate) fn test(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let chan = cx.channel();
    let (def, p) = cx.promise();

    std::thread::spawn(move || {
        let m = DllMap { map: Vec::new() };
        let ts = TestStruct::constructor("random_path".into(), m).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(2));
        chan.settle_with(def, move |cx| TestStruct::to_js_obj(cx, ts));
    });

    Ok(p)
}
// Hack so this file can be included in the src/lib.rs Examples section.
