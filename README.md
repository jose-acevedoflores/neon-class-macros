This crate aims to simplify some of the interactions with the [neon](https://github.com/neon-bindings/neon) api when it comes to
dealing with mirroring rust structs as JS classes. The inspiration for this crate come mostly from
[napi-rs](https://github.com/napi-rs/napi-rs).

## Usage

There are three steps to using this crate:

1. Annotate the `struct` you want to export as a class:

```rust
// ------------------ Step 1 ---------------
use neon::prelude::Finalize;

#[derive(neon_class_macros::Class)]
pub struct TestStruct {
    fields: (),
}
impl Finalize for TestStruct {}
```

2. Annotate the impl block for the `struct` and the method that
   will serve as the constructor on the JS side:

```rust
// ------------------ Step 1 ---------------
use neon::prelude::{Context, Finalize};

#[derive(neon_class_macros::Class)]
pub struct TestStruct {
    fields: Vec<u32>,
}
impl Finalize for TestStruct {}
// ------------------ Step 2 ---------------
#[neon_class_macros::impl_block]
impl TestStruct {
    #[neon_class_macros::constructor]
    pub fn ctor() -> Result<Self, String> {
        Ok(Self{
            fields: Vec::new(),
        })
    }
}
```

3. Annotate the methods you want to expose to the JS side:

```rust
// ------------------ Step 1 ---------------
use neon::prelude::{Context, Finalize};

#[derive(neon_class_macros::Class)]
pub struct TestStruct {
    fields: Vec<u32>,
}
impl Finalize for TestStruct {}
// ------------------ Step 2 ---------------
#[neon_class_macros::impl_block]
impl TestStruct {
    #[neon_class_macros::constructor]
    pub fn ctor() -> Result<Self, String> {
        Ok(Self{
            fields: Vec::new(),
        })
    }
// ------------------ Step 3 ---------------
   #[neon_class_macros::method]
   pub fn method(&self, num: u32, data: String) {
      println!("Do something {:?}-{}-{}", self.fields, num, data)
   }
}
```

4. Lastly, register the class with neon:

```rust
// ------------------ Step 1 ---------------
use neon::prelude::{Context, Finalize, NeonResult, ModuleContext};

#[derive(neon_class_macros::Class)]
pub struct TestStruct {
   fields: Vec<u32>,
}
impl Finalize for TestStruct {}
// ------------------ Step 2 ---------------
#[neon_class_macros::impl_block]
impl TestStruct {
   #[neon_class_macros::constructor]
   pub fn ctor() -> Result<Self, String> {
      Ok(Self{
         fields: Vec::new(),
      })
   }
// ------------------ Step 3 ---------------
   #[neon_class_macros::method]
   pub fn method(&self, num: u32, data: String) {
      println!("Do something {:?}-{}-{}", self.fields, num, data)
   }
}
// ------------------ Step 4 ---------------
#[neon::main]
fn node_entrypoint(mut cx: ModuleContext) -> NeonResult<()> {

    TestStruct::register_ctor(&mut cx)?;
    Ok(())
}
```

## System Dependencies

1. node/npm version 14 or newer
2. cargo version 1.56 or newer

## Install Instructions

1. npm install
2. npm run build:node_tests

## Running the tests

1. `cargo test --features -for-tests` or use the alias `cargo t`
