Builds a JS object from `obj`

#### to_js_obj

```rust
use neon::prelude::{FunctionContext, Finalize, Context, JsResult, JsObject, JsString};

#[derive(neon_macros::Class)]
struct Dummy {
    field: String,
}

impl Finalize for Dummy {}

#[neon_macros::impl_block]
impl Dummy {
    pub fn new(arg: String) -> Self {
        Self {
            field: arg
        }
    }

    #[neon_macros::method]
    fn a_method<'ctx>(
        &self,
        mut cx: FunctionContext<'ctx>,
        num: u32,
        msg: String,
    ) -> JsResult<'ctx, JsString> {
        let res = format!("hehe {}-{}-{:?}", msg, num, self.field);
        Ok(cx.string(res))
    }
}

fn some_neon_fn(mut cx: FunctionContext) -> JsResult<JsObject> {
    let obj = Dummy::new("s".to_string());
    Dummy::to_js_obj(&mut cx, obj)
}

```

Now on the JS side

```js
const addon = require("addon");

const dummy = addon.someNeonFn();
const result = dummy.aMethod(34, "hallo");
```

#### const THIS

Adds a `const THIS: &str ...` variable to reference the `this` object.\
**TODO:"User can provide their own"** As long as there is a THIS const present
in the `impl` block the other macros should use that.
