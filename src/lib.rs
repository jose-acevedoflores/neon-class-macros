use proc_macro::TokenStream;
use std::str::FromStr;

use quote::quote;
use syn::{parse_macro_input, DeriveInput, ImplItem, ImplItemConst, ImplItemMethod, ItemImpl};

#[proc_macro_attribute]
pub fn my_attribute(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let tokens = quote! {
        #input

        struct Hello;
    };

    tokens.into()
}

#[proc_macro_derive(Class)]
pub fn derive_class(input: TokenStream) -> TokenStream {
    let _input = parse_macro_input!(input as DeriveInput);

    let tokens = quote! {};

    tokens.into()
}

pub(crate) fn get_gen_constructor_name(orig_name: &proc_macro2::Ident) -> proc_macro2::Ident {
    let gen_constructor_name = format!("__neon_gen_{}", orig_name);
    syn::Ident::new(&gen_constructor_name, orig_name.span())
}

///
/// ## Attribute Args:
/// * `expose` - expose this constructor to the JS side.
#[proc_macro_attribute]
pub fn constructor(_args: TokenStream, input: TokenStream) -> TokenStream {
    let method_ast = parse_macro_input!(input as ImplItemMethod);
    let gen_constructor_ident = get_gen_constructor_name(&method_ast.sig.ident);

    let tokens = quote! {
        #method_ast

        pub fn #gen_constructor_ident(mut cx: FunctionContext)  -> JsResult<JsUndefined>  {



            Ok(cx.undefined())
        }
    };

    tokens.into()
}

#[proc_macro_attribute]
pub fn method(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(input);

    let tokens = quote! {
        #input

    };

    tokens.into()
}

/// Adds a `const THIS: &str ...` variable to reference the `this` object.
/// 1. User can provide their own and not use the macro. As long as there is a THIS const present
/// in the `impl` block the other macros should use that.
#[proc_macro_attribute]
pub fn impl_block(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut impl_ast = parse_macro_input!(input as ItemImpl);

    let this =
        proc_macro2::TokenStream::from_str("const THIS: &'static str =\"__this_obj\";").unwrap();
    let this: proc_macro::TokenStream = this.into();
    let cst = parse_macro_input!(this as ImplItemConst);
    impl_ast.items.push(ImplItem::Const(cst));

    let tokens = quote! {
        #impl_ast
    };

    tokens.into()
}
