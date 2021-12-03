//!
#![doc = include_str!("../README.md")]
//!
//! ## Example
//!
//! This example focuses on exporting a `TestStruct` as a JS class. It shows some of the method
//! signatures that are possible by using this crate.
//!
//! #### Rust side
//!
//! ```rust
#![doc = include_str!("../node_tests/src/derived_class.rs")]
//! ```
//!
//! To register the struct as a class check the following example:
//!
//! ```ignore
#![doc = include_str!("../node_tests/src/lib.rs")]
//! ```
//!
//! #### JavaScript Side
//!
//! ```javascript
#![doc = include_str!("../node_tests/derivedClass.test.js")]
//! ```
//!
use crate::utils::{ImplTree, NeonMacrosAttrs};
use heck::MixedCase;
use proc_macro::TokenStream;
use proc_macro2::Literal;
use quote::{format_ident, quote};
use std::str::FromStr;
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, AttributeArgs, DeriveInput, ImplItem, ImplItemConst, ImplItemMethod,
    ItemImpl, Lifetime, Meta, NestedMeta, Type,
};

mod utils;

const GENERATED_METHOD_PREFIX: &str = "__neon_gen_";

/// Not doing anything currently.
#[proc_macro_derive(Class)]
pub fn derive_class(input: TokenStream) -> TokenStream {
    let _input = parse_macro_input!(input as DeriveInput);

    let tokens = quote! {};

    tokens.into()
}

pub(crate) fn get_gen_method_name(orig_name: &proc_macro2::Ident) -> proc_macro2::Ident {
    let gen_constructor_name = format!("{}{}", GENERATED_METHOD_PREFIX, orig_name);
    syn::Ident::new(&gen_constructor_name, orig_name.span())
}

#[proc_macro_attribute]
pub fn neon_class(args: TokenStream, input: TokenStream) -> TokenStream {
    let args_cl = args.clone();
    let parsed_args = parse_macro_input!(args_cl as AttributeArgs);
    if let Some(nested_meta) = parsed_args.first() {
        match &nested_meta {
            NestedMeta::Meta(meta) => match meta {
                Meta::List(_) => {}

                Meta::Path(path) => {
                    let id = &path.segments.last().unwrap().ident;
                    match id.to_string().as_ref() {
                        "method" => {
                            return method(args, input);
                        }
                        "impl_block" => {
                            return impl_block(args, input);
                        }
                        "constructor" => {
                            return constructor(args, input);
                        }
                        _ => {}
                    }
                }
                Meta::NameValue(_) => {}
            },
            NestedMeta::Lit(_) => {}
        }
    }
    let tokens = quote! {};

    tokens.into()
}

/// Generates a constructor for the JS side based on the annotated method.
fn constructor(_args: TokenStream, input: TokenStream) -> TokenStream {
    let orig_ctor_ast = parse_macro_input!(input as ImplItemMethod);
    let orig_ctor_name = &orig_ctor_ast.sig.ident;
    let gen_ctor_name = get_gen_method_name(orig_ctor_name);

    let ((arg_idents, arg_parsing), cx_is_arg) =
        utils::parse_native_args(&orig_ctor_ast.sig.inputs);

    let native_method_call = if cx_is_arg {
        quote! {
            Self::#orig_ctor_name(&mut cx, #(#arg_idents,)*).map_err(|e| {
                cx.throw_type_error::<_, ()>(format!("Failed to construct {}", e))
                    .unwrap_err()
            })?;
        }
    } else {
        quote! {
            Self::#orig_ctor_name(#(#arg_idents,)*).map_err(|e| {
                cx.throw_type_error::<_, ()>(format!("Failed to construct {}", e))
                    .unwrap_err()
            })?;
        }
    };

    let tokens = quote! {
        #orig_ctor_ast

        /// Generated constructor for the JS side.
        ///
        /// This method is what will be called when the JS side performs a `new` like:
        /// ```js
        /// const jsObj = new RustExportedValue();
        /// ```
        pub fn #gen_ctor_name(mut cx: neon::prelude::FunctionContext) -> neon::prelude::JsResult<neon::prelude::JsUndefined> {
            // Need this in scope for cx.this().set to work
            use neon::prelude::Object;
            // required by the expansion of `arg_parsing`
            use neon_serde::errors::MapErrIntoThrow;

            #(#arg_parsing)*

            let res = #native_method_call

            let this = cx.boxed(res);
            cx.this().set(&mut cx, Self::THIS, this)?;
            Ok(cx.undefined())
        }
    };

    tokens.into()
}

/// Macro that decorates the methods that should be included in the JS prototype.
///
#[doc = include_str!("../docs/method_macro.md")]
///
fn method(args: TokenStream, input: TokenStream) -> TokenStream {
    let parsed_args = parse_macro_input!(args as AttributeArgs);

    let orig_method_ast = parse_macro_input!(input as ImplItemMethod);
    let orig_method_name = &orig_method_ast.sig.ident;
    let gen_method_name = get_gen_method_name(orig_method_name);
    let output = &orig_method_ast.sig.output;

    // methods that return a JsResult directly must provide a lifetime so if they do, we use that for the
    // generated method. If they don't (like when they return native rust types) then we default to 'ctx.
    let output_lifetime = utils::get_lifetime_from_return_type(output)
        .unwrap_or_else(|| Lifetime::new("'ctx", output.span()));

    let gen_doc = proc_macro2::TokenStream::from_str(&format!(
        "/// Generated method for [`{0}`](#method.{0}). See [`method`](neon_class_macros::method) macro for details.",
        orig_method_name
    ))
    .unwrap();

    let ((arg_idents, arg_parsing), cx_is_arg) =
        utils::parse_native_args(&orig_method_ast.sig.inputs);

    let throws_on_err = utils::throws_on_err(&parsed_args);
    let (output, native_method_result_parser) =
        utils::parse_return_type(output, &output_lifetime, throws_on_err);

    let native_method_call = if cx_is_arg {
        quote! {
            this.#orig_method_name(&mut cx, #(#arg_idents,)*)
        }
    } else {
        quote! {
            this.#orig_method_name(#(#arg_idents,)*)
        }
    };

    let return_call = if let Some(fnct) = native_method_result_parser {
        let result_ident = format_ident!("res");
        let real_result = fnct(&result_ident);
        quote! {
            let #result_ident = #native_method_call;
            #real_result
        }
    } else {
        native_method_call
    };

    let tokens = quote! {
        #orig_method_ast

        ///
        #gen_doc
        pub fn #gen_method_name<#output_lifetime>(mut cx: neon::prelude::FunctionContext<#output_lifetime>) #output {
            use neon::prelude::Object;
            // required by the expansion of `arg_parsing`
            use neon_serde::errors::MapErrIntoThrow;

            #(#arg_parsing)*

            let this = cx.this();
            let this = this.get(&mut cx, Self::THIS)?
                .downcast_or_throw::<neon::prelude::JsBox<Self>, _>(&mut cx)?;

            #return_call
        }
    };

    tokens.into()
}

/// This macro is used to decorate impl blocks.
///
/// ## Examples
/// The following are examples of how to use some of the methods generated by this macro.
#[doc = include_str!("../docs/to_js_obj.md")]
///
fn impl_block(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut impl_ast = parse_macro_input!(input as ItemImpl);

    let this_token = {
        let this = quote! {
            const THIS: &'static str ="__this_obj";
        };
        let this: proc_macro::TokenStream = this.into();
        parse_macro_input!(this as ImplItemConst)
    };

    impl_ast.items.push(ImplItem::Const(this_token));

    let struct_name = if let Type::Path(arg) = impl_ast.self_ty.as_ref() {
        let name = &arg.path.segments.last().unwrap().ident;
        Literal::string(&name.to_string())
    } else {
        panic!("No struct_name for impl block")
    };

    let attrs_for_each_decorated_method = impl_ast
        .items
        .iter()
        .map(|item| {
            if let ImplItem::Method(method) = item {
                NeonMacrosAttrs::new(method.clone())
            } else {
                None
            }
        })
        .flatten()
        .collect::<Vec<NeonMacrosAttrs>>();

    if attrs_for_each_decorated_method.is_empty() {
        panic!(
            "Found an 'impl_block' argument for struct {} but no constructor or methods were found.\n\
        This could be because:\n  \
          1. none are implemented, in which case the 'impl_block' decoration can be removed.\n  \
          2. the 'neon_class' macro was renamed to something else. This is currently not supported.",
               struct_name.to_string()
        );
    }

    let impl_tree = ImplTree::new(attrs_for_each_decorated_method);
    let gen_method_names: Vec<proc_macro2::Ident> = impl_tree
        .methods
        .iter()
        .map(|e| get_gen_method_name(&e.sig.ident))
        .collect();
    let js_names: Vec<Literal> = impl_tree
        .methods
        .iter()
        .map(|e| {
            let js_name = format!("{}", &e.sig.ident).to_mixed_case();
            Literal::string(&js_name)
        })
        .collect();

    let prototype_setup_tok = quote! {
        use neon::prelude::Object;

        let prototype = constructor
            .get(cx, "prototype")?
            .downcast_or_throw::<neon::prelude::JsObject, _>(cx)?;

        #(
            let f = neon::prelude::JsFunction::new(cx, Self::#gen_method_names)?;
            prototype.set(cx, #js_names, f)?;
        )*
    };

    if let Some(constructor) = &impl_tree.constructor {
        let orig_ctor_name = &constructor.sig.ident;
        let gen_ctor_name = get_gen_method_name(orig_ctor_name);
        let register_fn_name = format_ident!("register_{}", orig_ctor_name);

        let register_fn = {
            let fnct = quote! {
                /// Expose the constructor for this object to the JS side.
                pub fn #register_fn_name(cx: &mut neon::prelude::ModuleContext) -> neon::prelude::NeonResult<()> {
                    let constructor = neon::prelude::JsFunction::new(cx, Self::#gen_ctor_name)?;

                    #prototype_setup_tok

                    cx.export_value(#struct_name, constructor)?;
                    Ok(())
                }
            };
            let fnct: proc_macro::TokenStream = fnct.into();
            parse_macro_input!(fnct as ImplItemMethod)
        };
        impl_ast.items.push(ImplItem::Method(register_fn));
    }

    let to_js_obj_fn = {
        let fnct = quote! {
            /// Turn an object of `Self` into a JS object.
            ///
            /// See example usage in [impl_block](macro@neon_class_macros::impl_block#to_js_obj).
            pub fn to_js_obj<'a, 'b>(cx: &'b mut impl neon::prelude::Context<'a>, obj: Self) -> neon::prelude::JsResult<'a, neon::prelude::JsObject> {
                let constructor = neon::prelude::JsFunction::new(cx, |mut cx| {
                    let this = cx.argument::<neon::prelude::JsBox<Self>>(0)?;
                    cx.this().set(&mut cx, Self::THIS, this)?;
                    Ok(cx.undefined())
                })?;

                #prototype_setup_tok

                let handle = cx.boxed(obj);
                let c = constructor.construct(cx, [handle])?;
                Ok(c)
            }
        };
        let fnct: proc_macro::TokenStream = fnct.into();
        parse_macro_input!(fnct as ImplItemMethod)
    };
    impl_ast.items.push(ImplItem::Method(to_js_obj_fn));

    let tokens = quote! {
        #impl_ast
    };

    tokens.into()
}
