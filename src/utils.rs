//! Utility functions to help deal with converting from [`neon::types`] to supported rust types and vice versa.
use proc_macro2::{Ident, Literal, TokenStream};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::{FnArg, ImplItemMethod, Meta, NestedMeta, ReturnType, Type, TypePath};

fn is_native_numeric(arg_type: &Ident) -> bool {
    arg_type == "u32" || arg_type == "f64" || arg_type == "i32"
}

pub type ContextIsArg = bool;

pub fn parse_native_args(
    input_args: &Punctuated<FnArg, Comma>,
) -> ((Vec<Ident>, Vec<TokenStream>), ContextIsArg) {
    // while parsing all the args we might encounter `self` and a `FunctionContext`. In those
    // cases we need to subtract that from the arg index in order to find the correct arg on the js side.
    // Example:
    //   For a function with the following signature
    //      fn start_camel<'ctx>(&self, mut cx: FunctionContext<'ctx>, num: u32) -> BLAH....
    //   `num` is really `idx` = 2 but with the `idx_adjuster` that becomes idx 0 so
    //   we can do `cx.argument.get(idx - idx_adjuster)`
    let mut idx_adjuster = 0;
    let mut context_is_arg = false;
    let parsed_args: Vec<(Ident, TokenStream)> = input_args
        .iter()
        .enumerate()
        .map(|(idx, f)| match f {
            FnArg::Typed(t) => match t.ty.as_ref() {
                Type::Path(d) => {
                    let arg_type = &d.path.segments.last().unwrap().ident;
                    if arg_type == "FunctionContext" {
                        // FunctionContext as second arg so skip this one
                        idx_adjuster += 1;
                        context_is_arg = true;
                        return None;
                    }
                    Some(extract_from_native_input_type(idx - idx_adjuster, d))
                }
                _ => None,
            },
            FnArg::Receiver(_) => {
                // self parameter, skip one in adjusted idx
                idx_adjuster += 1;
                None
            }
        })
        .flatten()
        .collect();

    (parsed_args.iter().cloned().unzip(), context_is_arg)
}

fn extract_from_native_input_type(arg_idx: usize, arg: &TypePath) -> (Ident, TokenStream) {
    let arg_type = &arg.path.segments.last().unwrap().ident;

    let arg_name = format!("arg_{}", arg_idx);
    let arg_ident = Ident::new(&arg_name, arg.span());
    let idx_literal = Literal::i32_unsuffixed(arg_idx as i32);
    let tok = if is_native_numeric(arg_type) {
        quote! {
            let #arg_ident = cx.argument::<neon::prelude::JsNumber>(#idx_literal)?.value(&mut cx) as #arg_type;
        }
    } else {
        quote! {
            let #arg_ident = cx.argument::<neon::prelude::JsValue>(#idx_literal)?;
            let #arg_ident = neon_serde::from_value(&mut cx, #arg_ident).map_err_into_throw(&mut cx)?;
        }
    };

    (arg_ident, tok)
}

type NativeResultParser = Option<fn(&Ident) -> proc_macro2::TokenStream>;

/// This functions is in charge of determining if the return type provided by the decorated method needs
/// to be modified or not.
///
/// Specifically it checks if the return type:
/// * Can be used as is, meaning the decorated method already returns a valid [`JsResult`](neon::prelude::JsResult)
/// * Needs to be converted. This applies to methods that return plain types like:
///    * [`String`]: which needs to be converted to a [`JsValue`](neon::prelude::JsValue)
///    * [`u32`]: which needs to be converted to a [`JsValue`](neon::prelude::JsValue)
///    * [`unit`]: which needs to be converted to [`JsUndefined`](neon::prelude::JsUndefined)
///    * etc ...
///
pub fn parse_return_type(output: &ReturnType) -> (proc_macro2::TokenStream, NativeResultParser) {
    match &output {
        ReturnType::Default => {
            let tok = quote! {
                -> neon::prelude::JsResult<neon::prelude::JsUndefined>
            };
            return (
                tok,
                Some(|_ident| {
                    quote! {
                        Ok(cx.undefined())
                    }
                }),
            );
        }
        ReturnType::Type(_, ty) => {
            if let Type::Path(path) = ty.as_ref() {
                let native_method_return_type = &path.path.segments.last().unwrap().ident;
                if native_method_return_type != "JsResult" {
                    let return_tok = quote! {
                        -> neon::prelude::JsResult<'ctx, neon::prelude::JsValue>
                    };

                    return (
                        return_tok,
                        Some(|ident| {
                            quote! {
                                let #ident = neon_serde::to_value(&mut cx, &#ident).map_err_into_throw(&mut cx)?;
                                Ok(#ident)
                            }
                        }),
                    );
                }
            }
        }
    }

    let tok = quote! {
        #output
    };
    (tok, None)
}

pub struct NeonMacrosAttrs {
    pub method: ImplItemMethod,
    pub main: String,
    /// List of args given to the macro
    ///
    /// For example, given `#[neon_macros::method(arg1, arg2)]` this `args` field would be:
    /// `["arg1", "arg2"]`
    pub args: Vec<String>,
}

impl NeonMacrosAttrs {
    pub fn new(method: ImplItemMethod) -> Self {
        let mut parsed_attrs = NeonMacrosAttrs {
            method,
            main: String::new(),
            args: Vec::new(),
        };

        parsed_attrs.method.attrs.iter().for_each(|attrs| {
            if let Some(attribute_pkg) = attrs.path.segments.first() {
                if attribute_pkg.ident != "neon_macros" {
                    return;
                }
            }

            parsed_attrs.main = format!("{}", attrs.path.segments.last().unwrap().ident);

            let m = attrs.parse_meta().unwrap();
            match &m {
                Meta::Path(path) => {
                    parsed_attrs
                        .args
                        .push(format!("{}", path.segments.last().unwrap().ident));
                }
                Meta::List(meta_ls) => {
                    let nested_meta = meta_ls.nested.first().unwrap();
                    match nested_meta {
                        NestedMeta::Meta(meta) => {
                            if let Meta::Path(path) = meta {
                                parsed_attrs
                                    .args
                                    .push(format!("{}", path.segments.last().unwrap().ident));
                            }
                        }
                        NestedMeta::Lit(_) => {}
                    }
                }
                Meta::NameValue(_) => {}
            }
        });
        parsed_attrs
    }

    pub fn is_constructor(&self) -> bool {
        &self.main == "constructor"
    }

    pub fn is_method(&self) -> bool {
        &self.main == "method"
    }
}

pub struct ImplTree {
    pub constructor: Option<ImplItemMethod>,
    pub methods: Vec<ImplItemMethod>,
}

impl ImplTree {
    pub fn new(methods: Vec<NeonMacrosAttrs>) -> Self {
        let mut s = ImplTree {
            constructor: None,
            methods: Vec::with_capacity(methods.len() - 1),
        };

        for method in methods {
            if method.is_constructor() {
                s.constructor = Some(method.method.clone());
            } else if method.is_method() {
                s.methods.push(method.method);
            }
        }

        s
    }
}
