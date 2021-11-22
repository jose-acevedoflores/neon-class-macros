use proc_macro::TokenStream;

use quote::quote;
use syn::{
    parse_macro_input, DeriveInput, FnArg, ImplItem, ImplItemConst, ImplItemMethod, ItemImpl, Type,
};

mod utils;

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

/// Generates a constructor for the JS side based on the annotated method.
/// ## Attribute Args:
/// * `expose` - expose this constructor to the JS side.
#[proc_macro_attribute]
pub fn constructor(_args: TokenStream, input: TokenStream) -> TokenStream {
    let method_ast = parse_macro_input!(input as ImplItemMethod);
    let method_name = &method_ast.sig.ident;
    let gen_constructor_ident = get_gen_constructor_name(method_name);

    let parsed_args: Vec<(proc_macro2::Ident, proc_macro2::TokenStream)> = method_ast
        .sig
        .inputs
        .iter()
        .enumerate()
        .map(|(idx, f)| match f {
            FnArg::Typed(t) => match t.ty.as_ref() {
                Type::Path(d) => Some(utils::extract_from_native_type(idx, d)),
                _ => None,
            },
            _ => None,
        })
        .flatten()
        .collect();

    let (arg_idents, arg_parsing): (Vec<_>, Vec<_>) = parsed_args.iter().cloned().unzip();

    let tokens = quote! {
        #method_ast

        /// Generated constructor.
        pub fn #gen_constructor_ident(mut cx: neon::prelude::FunctionContext) -> neon::prelude::JsResult<neon::prelude::JsUndefined>  {
            // Need this in scope for cx.this().set to work
            use neon::prelude::Object;

            #(#arg_parsing)*

            let res = Self::#method_name(#(#arg_idents,)*).map_err(|e| {
                cx.throw_type_error::<_, ()>(format!("Failed to construct {}", e))
                    .unwrap_err()
            })?;

            let this = cx.boxed(res);
            cx.this().set(&mut cx, Self::THIS, this)?;
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
/// 1. <TODO:"User can provide their own"> As long as there is a THIS const present
/// in the `impl` block the other macros should use that.
#[proc_macro_attribute]
pub fn impl_block(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut impl_ast = parse_macro_input!(input as ItemImpl);

    let this_token = {
        let this = quote! {
            const THIS: &'static str ="__this_obj";
        };
        let this: proc_macro::TokenStream = this.into();
        parse_macro_input!(this as ImplItemConst)
    };

    impl_ast.items.push(ImplItem::Const(this_token));

    let gen_register_fn = {
        let gen_register_fn = quote! {
            pub fn __neon_gen_expose_register(cx: &mut neon::prelude::ModuleContext) -> neon::prelude::NeonResult<()> {
                Ok(())
            }
        };
        let gen_register_fn: proc_macro::TokenStream = gen_register_fn.into();
        parse_macro_input!(gen_register_fn as ImplItemMethod)
    };
    impl_ast.items.push(ImplItem::Method(gen_register_fn));

    let tokens = quote! {
        #impl_ast
    };

    tokens.into()
}
