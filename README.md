This crate aims to simplify some of the interactions with the [neon](https://github.com/neon-bindings/neon) api when it comes to
dealing with mirroring rust structs as JS classes. The inspiration for this crate come mostly from
[napi-rs](https://github.com/napi-rs/napi-rs).

It lets you write something like this in rust:

```rust
use neon::prelude::{Context, Finalize, NeonResult, ModuleContext};

#[derive(neon_class_macros::Class)]
pub struct TestStruct {
   fields: Vec<u32>,
}
impl Finalize for TestStruct {}

#[neon_class_macros::impl_block]
impl TestStruct {
   #[neon_class_macros::constructor]
   pub fn ctor() -> Result<Self, String> {
      Ok(Self{
         fields: Vec::new(),
      })
   }

   #[neon_class_macros::method]
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

For more examples checkout out the [`derived_class.rs`](./node_tests/src/derived_class.rs)

## System Dependencies

1. node/npm version 14 or newer
2. cargo version 1.56 or newer

## Install Instructions

1. npm install
2. npm run build:node_tests

## Running the tests

1. `cargo test --features -for-tests` or use the alias `cargo t`
