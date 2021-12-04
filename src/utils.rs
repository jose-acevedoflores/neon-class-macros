//! Utility functions to help deal with converting from [`neon::types`] to supported rust types and vice versa.
use proc_macro2::{Ident, Literal, TokenStream};
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{
    FnArg, GenericArgument, ImplItemMethod, Lifetime, Meta, NestedMeta, Pat, PathArguments,
    ReturnType, Type, TypePath,
};

fn is_native_numeric(arg_type: &Ident) -> bool {
    arg_type == "u32" || arg_type == "f64" || arg_type == "i32"
}

pub type CxIsArg = bool;

pub fn parse_rust_fn_args(
    input_args: &Punctuated<FnArg, Comma>,
) -> ((Vec<Ident>, Vec<TokenStream>), CxIsArg) {
    // while parsing all the args we might encounter `self` and a `FunctionContext`. In those
    // cases we need to subtract that from the arg index in order to find the correct arg on the js side.
    // Example:
    //   For a function with the following signature
    //      fn start_camel<'ctx>(&self, mut cx: FunctionContext<'ctx>, num: u32) -> BLAH....
    //   `num` is really `idx` = 2 but with the `idx_adjuster` that becomes idx 0 so
    //   we can do `cx.argument.get(idx - idx_adjuster)`
    let mut idx_adjuster = 0;
    let mut cx_is_arg = false;
    let parsed_args: Vec<(Ident, TokenStream)> = input_args
        .iter()
        .enumerate()
        .map(|(idx, fn_arg)| match fn_arg {
            FnArg::Typed(fn_arg) => {
                if let Pat::Ident(p_ident) = fn_arg.pat.as_ref() {
                    let arg_name = &p_ident.ident;
                    if arg_name == "cx" || arg_name == "_cx" {
                        idx_adjuster += 1;
                        cx_is_arg = true;
                        return None;
                    }
                }
                match fn_arg.ty.as_ref() {
                    Type::Path(tp) => Some(extract_from_native_input_type(idx - idx_adjuster, tp)),
                    _ => None,
                }
            }
            FnArg::Receiver(_) => {
                // '&self' parameter, skip one in adjusted idx
                idx_adjuster += 1;
                None
            }
        })
        .flatten()
        .collect();

    (parsed_args.iter().cloned().unzip(), cx_is_arg)
}

fn extract_from_native_input_type(arg_idx: usize, arg: &TypePath) -> (Ident, TokenStream) {
    let arg_ident = format_ident!("arg_{}", arg_idx);
    let idx_literal = Literal::i32_unsuffixed(arg_idx as i32);

    let arg_type = arg.path.get_ident().filter(|i| is_native_numeric(i));
    let tok = if let Some(arg_type) = arg_type {
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
/// * Needs to be converted. This applies to methods that don't return a [`JsResult`](neon::prelude::JsResult)\
/// To convert the return types we use `neon_serde` so whatever is valid there should apply here.
///
pub fn parse_return_type(
    output: &ReturnType,
    lifetime: &Lifetime,
    throws_on_err: bool,
) -> (proc_macro2::TokenStream, NativeResultParser) {
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
                        -> neon::prelude::JsResult<#lifetime, neon::prelude::JsValue>
                    };

                    let parse_tok: NativeResultParser = if throws_on_err {
                        Some(|ident| {
                            quote! {
                                let #ident = #ident.map_err(|e| {
                                    cx.throw_error::<_, ()>(format!("{}", e)).unwrap_err()
                                })?;
                                let #ident = neon_serde::to_value(&mut cx, &#ident).map_err_into_throw(&mut cx)?;
                                Ok(#ident)
                            }
                        })
                    } else {
                        Some(|ident| {
                            quote! {
                                let #ident = neon_serde::to_value(&mut cx, &#ident).map_err_into_throw(&mut cx)?;
                                Ok(#ident)
                            }
                        })
                    };

                    return (return_tok, parse_tok);
                }
            }
        }
    }

    let tok = quote! {
        #output
    };
    // If we reach this point this means the user was already returning a `JsResult` so no need
    // to parse the result of the original decorated method.
    (tok, None)
}

pub struct NeonMacrosAttrs {
    /// The full method AST.
    pub method: ImplItemMethod,
    /// From a list of attributes, the first one
    ///
    /// For example, given `#[neon_class(method, throw_on_err)]` this `main` field would be:
    /// `"method"`
    pub main: String,
    /// List of args given to the macro, excluding the main one.
    ///
    /// For example, given `#[neon_class(method, throw_on_err)]` this `args` field would be:
    /// `["throw_on_err"]`
    pub args: Vec<String>,
}

impl NeonMacrosAttrs {
    const VALID_ARGS: [&'static str; 1] = ["throw_on_err"];

    pub fn new(method: ImplItemMethod) -> Option<Self> {
        let mut parsed_attrs = NeonMacrosAttrs {
            method,
            main: String::new(),
            args: Vec::new(),
        };

        let mut neon_class_attribute_found = false;

        parsed_attrs.method.attrs.iter().for_each(|attrs| {
            if let Some(attribute_pkg) = attrs.path.segments.first() {
                // TODO fix this for renames. currently, not sure how to get if the macro was renamed
                // at import. We use the macro name here to find the methods in the ast that were marked as
                // 'constructor' or 'method'. See the rename_macro_error.rs test.
                if attribute_pkg.ident != "neon_class" {
                    return;
                } else {
                    neon_class_attribute_found = true;
                }
            }

            let m = attrs.parse_meta().unwrap();
            match &m {
                Meta::Path(path) => {
                    parsed_attrs
                        .args
                        .push(format!("{}", path.segments.last().unwrap().ident));
                }
                Meta::List(meta_ls) => {
                    let main_arg = meta_ls.nested.first().unwrap();
                    match main_arg {
                        NestedMeta::Meta(meta) => {
                            if let Meta::Path(path) = meta {
                                parsed_attrs.main =
                                    format!("{}", path.segments.last().unwrap().ident);
                            }
                        }
                        NestedMeta::Lit(_) => {}
                    }
                    // Skip 1 here since the first one is saved as the main
                    meta_ls.nested.iter().skip(1).for_each(|nm| {
                        let id = get_nested_meta_ident(nm).unwrap();
                        if Self::VALID_ARGS.iter().any(|s| id == s) {
                            parsed_attrs.args.push(format!("{}", id));
                        } else {
                            panic!("Invalid arg {}", id);
                        }
                    });
                }
                Meta::NameValue(_) => {}
            }
        });

        if neon_class_attribute_found {
            Some(parsed_attrs)
        } else {
            None
        }
    }

    pub fn is_constructor(&self) -> bool {
        &self.main == "constructor"
    }

    pub fn is_method(&self) -> bool {
        &self.main == "method"
    }
}

pub struct ImplTree {
    /// Only allow one constructor since only one value can be exported with a given struct's name.
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
                if s.constructor.is_none() {
                    s.constructor = Some(method.method);
                } else {
                    panic!(
                        "There is already a method annotated as constructor with the name '{}'.\n\
                    With neon we can only export one value so the method '{}' is not allowed.\n\
                    To fix it, choose one of the two.",
                        s.constructor.unwrap().sig.ident,
                        method.method.sig.ident
                    )
                }
            } else if method.is_method() {
                s.methods.push(method.method);
            }
        }

        s
    }
}

pub fn get_lifetime_from_return_type(output: &ReturnType) -> Option<Lifetime> {
    if let ReturnType::Type(_, ty) = output {
        if let Type::Path(t_obj) = ty.as_ref() {
            if let PathArguments::AngleBracketed(ab) =
                &t_obj.path.segments.last().unwrap().arguments
            {
                if let GenericArgument::Lifetime(lf) = ab.args.first().unwrap() {
                    return Some(lf.clone());
                }
            }
        }
    }
    None
}

fn get_nested_meta_ident(nm: &NestedMeta) -> Option<&Ident> {
    if let NestedMeta::Meta(Meta::Path(arg)) = nm {
        arg.get_ident()
    } else {
        None
    }
}

pub fn throws_on_err(attrs: &[NestedMeta]) -> bool {
    attrs.iter().any(|attr| {
        let id = get_nested_meta_ident(attr).unwrap();
        id == NeonMacrosAttrs::VALID_ARGS[0]
    })
}
