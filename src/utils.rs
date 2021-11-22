use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::TypePath;

pub fn extract_from_native_type(arg_idx: usize, arg: &TypePath) -> (Ident, TokenStream) {
    let name = &arg.path.segments.last().unwrap().ident;

    let arg_name = format!("arg{}", arg_idx);
    let arg_ident = Ident::new(&arg_name, arg.span());
    let idx_literal = Literal::i32_unsuffixed(arg_idx as i32);
    let tok = if name == &Ident::new("String", Span::call_site()) {
        quote! {
            let #arg_ident = cx.argument::<neon::prelude::JsString>(#idx_literal)?.value(&mut cx);
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
