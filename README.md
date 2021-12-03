# neon_class_macros

![build](https://github.com/jose-acevedoflores/neon-class-macros/actions/workflows/rust.yml/badge.svg)

This crate aims to simplify some of the interactions with the [neon](https://github.com/neon-bindings/neon) api when it comes to
dealing with mirroring rust structs as JS classes. The inspiration for this crate come mostly from
[napi-rs](https://github.com/napi-rs/napi-rs) and [node-bindgen](https://github.com/infinyon/node-bindgen).

The main goal here is to eliminate some of the boilerplate needed when declaring classes on the rust side while still keeping
the flexibility of accessing the `FunctionContext` struct provided by `neon` for more advanced interactions.

It lets you write something like this in rust:

```rust
use neon::prelude::{Context, Finalize, NeonResult, FunctionContext, ModuleContext};
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

   #[neon_class(method)]
   pub fn method_with_cx(&self, _cx: &mut FunctionContext, num: u32, data: String) {
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

This crate relies heavily on [this fork](https://github.com/NZXTCorp/neon-serde) of the `neon_serde` crate for
serializing and deserializing a decorated method's inputs/outputs.

For more examples checkout out the [`derived_class.rs`](./node_tests/src/derived_class.rs) for the rust side and the
[`derivedClass.test.js`](./node_tests/derivedClass.test.js) for the JS side.

## API

#### `neon_class(impl_block)`

Decorate the `impl` block of the struct you want to export. This macro uses the decorated struct name as the name of the constructor on the JS side.\
This macro generates two methods:

- `to_js_obj`: this associated method can be used to turn `Self` into a `JsValue`. It's the equivalent of calling `new` on the JS side.\
   For an example see [to_js_obj](./docs/to_js_obj.md).
- `register_<your_contructor_name_here>`: This method is used to export the decorated struct as a value on the JS side.\
   This method is only present if there is a method decorated with `neon_class(constructor)`.

#### `neon_class(constructor)`

Decorate one (and only one) of the methods as a constructor. The decorated method:

- Must return a `Result<Self, E>` where `E` implements `Display`.
- Can take `&mut FunctionContext` as first argument. The argument must be named `cx` or `_cx`.

A method decorated as constructor is optional (you still have the `to_js_obj` associated method).

#### `neon_class(method, ...)`

Decorate one or more methods to be included as methods on the JS side. The decorated method:

- Must take `&self`.
- Can take `&mut FunctionContext` as second argument. The argument must be named `cx` or `_cx`.
- Can return a `JsResult` directly (as opposed to a type that will be converted via `neon_serde`) BUT you cannot change the binding.
  This means you cannot do `use neon::prelude::JsResult as <new bind>`, you have to use `JsResult` or the full path `neon::prelude::JsResult`
- Is exposed to the JS side with the same name but with `mixedCase`.

Optional args:

- `throw_on_err`:
  - a method with this arg MUST return a `Result<T, E>` where `E` implements `Display`
  - with this arg the `E` will be shown as a message on the JS side.\
    See [`take_numeric_return_result`](./node_tests/src/derived_class.rs) and the corresponding
    [`takeNumericReturnResult`](./node_tests/derivedClass.test.js) test.

## Build Dependencies

1. node/npm version 14 or newer
2. cargo version 1.56 or newer

## Install Instructions

1. npm install
2. npm run build:node_tests

## Running the tests

1. `cargo test --features for-tests` or use the alias `cargo t`
