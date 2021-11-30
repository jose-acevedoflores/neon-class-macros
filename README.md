This crate aims to simplify some of the interactions with the [neon](https://github.com/neon-bindings/neon) api when it comes to
dealing with mirroring rust structs as JS classes. The inspiration for this crate come mostly from
[napi-rs](https://github.com/napi-rs/napi-rs) and [node-bindgen](https://github.com/infinyon/node-bindgen).

It lets you write something like this in rust:

```rust
use neon::prelude::{Context, Finalize, NeonResult, ModuleContext};
use neon_class_macros::neon_class;

#[derive(neon_class_macros::Class)]
pub struct TestStruct {
   fields: Vec<u32>,
}
impl Finalize for TestStruct {}

#[neon_class(impl_block)]
impl TestStruct {
   #[neon_class(constructor)]
   pub fn ctor() -> Result<Self, String> {
      Ok(Self{
         fields: Vec::new(),
      })
   }

   #[neon_class(method)]
   pub fn method(&self, num: u32, data: String) {
      println!("Do something {:?}-{}-{}", self.fields, num, data)
   }
}

#[neon::main]
fn node_entrypoint(mut cx: ModuleContext) -> NeonResult<()> {

    TestStruct::register_ctor(&mut cx)?;
    Ok(())
}
```

And use it like this in javascript:

```javascript
const mod = require("native.node");
const obj = mod.TestStruct();
obj.method(7, "some_string");
```

This crate relies heavily on this fork of the [`neon_serde`](https://github.com/NZXTCorp/neon-serde) crate for
serializing and deserializing method inputs/outputs.

For more examples checkout out the [`derived_class.rs`](./node_tests/src/derived_class.rs)

## Limitations

- Methods decorated as `neon_class(constructor)` or `neon_class(method)` allow for receiving the `FunctionContext` directly as long as:
  1. for a `constructor`, it's the first argument and is of type `&mut FunctionContext`.
  2. for a `method`, it's the second argument and is of type `FunctionContext`
  3. for both `method` and `constructor`, the argument name is `cx` or `_cx`
- For decorated methods that return a `JsResult` you cannot rename the binding. This means
  you need to use `JsResult` or the full path `neon::prelude::JsResult`.
- A method decorated as `neon_class(constructor)` must return a `Result<String, T>` where `T` implements `Display`.

## System Dependencies

1. node/npm version 14 or newer
2. cargo version 1.56 or newer

## Install Instructions

1. npm install
2. npm run build:node_tests

## Running the tests

1. `cargo test --features -for-tests` or use the alias `cargo t`
