use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::{FnArg, ImplItemMethod, Meta, NestedMeta, Type, TypePath};

pub fn parse_native_args(
    input_args: &Punctuated<FnArg, Comma>,
    is_method: bool,
) -> (Vec<Ident>, Vec<TokenStream>) {
    let parsed_args: Vec<(Ident, TokenStream)> = input_args
        .iter()
        .enumerate()
        .map(|(idx, f)| match f {
            FnArg::Typed(t) => match t.ty.as_ref() {
                Type::Path(d) => {
                    //TODO Fix this adjusted_idx kludge
                    if is_method && idx as i64 - 2 < 0 {
                        return None;
                    }
                    let adjusted_idx = if is_method { idx - 2 } else { idx };
                    Some(extract_from_native_type(adjusted_idx, d))
                }
                _ => None,
            },
            _ => None,
        })
        .flatten()
        .collect();

    parsed_args.iter().cloned().unzip()
}

pub fn extract_from_native_type(arg_idx: usize, arg: &TypePath) -> (Ident, TokenStream) {
    let name = &arg.path.segments.last().unwrap().ident;

    let arg_name = format!("arg{}", arg_idx);
    let arg_ident = Ident::new(&arg_name, arg.span());
    let idx_literal = Literal::i32_unsuffixed(arg_idx as i32);
    let tok = if name == &Ident::new("String", Span::call_site()) {
        quote! {
            let #arg_ident = cx.argument::<neon::prelude::JsString>(#idx_literal)?.value(&mut cx);
        }
    } else if name == &Ident::new("u32", Span::call_site()) {
        quote! {
            let #arg_ident = cx.argument::<neon::prelude::JsNumber>(#idx_literal)?.value(&mut cx) as #name;
        }
    } else {
        quote! {
            let #arg_ident = cx.argument::<neon::prelude::JsValue>(#idx_literal)?;
            let #arg_ident = neon_serde::from_value(&mut cx, #arg_ident).map_err(|e| {
                cx.throw_type_error::<_, ()>(format!("Failed deserialization. {}", e))
                .unwrap_err()
            })?;
        }
    };

    (arg_ident, tok)
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
