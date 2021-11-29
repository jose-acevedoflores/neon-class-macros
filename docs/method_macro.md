This macro works by generating a method based on the decorated method so it
satisfies the [`neon export_*`](neon::prelude::ModuleContext) methods.

Methods decorated with this macro can have the following forms:

- Takes a [`FunctionContext`](neon::prelude::FunctionContext) as second argument.\
  **NOTES**
  - For this case, the lifetime of the [`FunctionContext`](neon::prelude::FunctionContext)
    needs to be explicit.
  - Return type can be a [`JsResult`](neon::prelude::JsResult) or a type that is
    convertible to one (TODO).

```rust
# use neon::prelude::{FunctionContext, Finalize, Context, JsResult, JsNumber};
# use neon_class_macros::neon_class;
# #[derive(neon_class_macros::Class)]
# struct Dummy {
#    field: String,
# }
# impl Finalize for Dummy {}
# #[neon_class(impl_block)]
# impl Dummy {
#    pub fn new(arg: String) -> Self {
#        Self {
#            field: arg
#        }
#    }
#[neon_class(method)]
fn a_method<'ctx>(
    &self,
    mut cx: FunctionContext<'ctx>,
    num: u32,
) -> JsResult<'ctx, JsNumber> {
    Ok(cx.number(num * 2))
}
# }
```

- Takes only regular items that can be extracted with supported neon types.
  Return type must be convertible to a [`JsResult`](neon::prelude::JsResult)

```rust
# use neon::prelude::{FunctionContext, Finalize, Context, JsResult, JsNumber};
# use neon_class_macros::neon_class;
# #[derive(neon_class_macros::Class)]
# struct Dummy {
#    field: String,
# }
# impl Finalize for Dummy {}
# #[neon_class(impl_block)]
# impl Dummy {
#    pub fn new(arg: String) -> Self {
#        Self {
#            field: arg
#        }
#    }
#[neon_class(method)]
fn plain_method(&self, num: f64) -> String {
  format!("to-str-{}", num)
}
# }
```
