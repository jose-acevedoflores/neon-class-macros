use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;
use std::mem::MaybeUninit;
use std::ptr::addr_of_mut;
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
    pub args: Vec<String>,
}

impl NeonMacrosAttrs {
    pub fn new(method: ImplItemMethod) -> Self {
        let mut n = NeonMacrosAttrs {
            method,
            main: String::new(),
            args: Vec::new(),
        };

        n.method.attrs.iter().for_each(|attrs| {
            if let Some(attribute_pkg) = attrs.path.segments.first() {
                if attribute_pkg.ident != "neon_macros" {
                    return;
                }
            }

            n.main = format!("{}", attrs.path.segments.last().unwrap().ident);

            let m = attrs.parse_meta().unwrap();
            match &m {
                Meta::Path(p) => {
                    n.args.push(format!("{}", p.segments.last().unwrap().ident));
                }
                Meta::List(l) => {
                    let r = l.nested.first().unwrap();
                    match r {
                        NestedMeta::Meta(p) => {
                            if let Meta::Path(p) = p {
                                n.args.push(format!("{}", p.segments.last().unwrap().ident));
                            }
                        }
                        NestedMeta::Lit(_) => {}
                    }
                }
                Meta::NameValue(_) => {}
            }
        });
        n
    }

    pub fn is_constructor_exposed(&self) -> Option<bool> {
        if self.is_constructor() {
            Some(self.args.iter().any(|s| s == "expose"))
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

pub struct Constructor {
    pub method: ImplItemMethod,
    pub exposed: bool,
}

pub struct ImplTree {
    pub constructor: Constructor,
    pub methods: Vec<ImplItemMethod>,
}

impl ImplTree {
    pub fn new(methods: Vec<NeonMacrosAttrs>) -> Self {
        let mut s: MaybeUninit<ImplTree> = MaybeUninit::uninit();
        let mut v: Vec<ImplItemMethod> = Vec::with_capacity(methods.len() - 1);

        let ptr = s.as_mut_ptr();
        for method in methods {
            if method.is_constructor() {
                unsafe {
                    addr_of_mut!((*ptr).constructor).write(Constructor {
                        method: method.method.clone(),
                        exposed: method.is_constructor_exposed().unwrap_or(false),
                    });
                }
            } else if method.is_method() {
                v.push(method.method);
            }
        }

        unsafe {
            addr_of_mut!((*ptr).methods).write(v);
        }
        unsafe { s.assume_init() }
    }
}
