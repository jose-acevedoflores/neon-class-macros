use crate::utils::{ImplTree, NeonMacrosAttrs};
use heck::MixedCase;
use proc_macro::TokenStream;
use proc_macro2::Literal;
use quote::quote;
use syn::{
    parse_macro_input, DeriveInput, ImplItem, ImplItemConst, ImplItemMethod, ItemImpl, Type,
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

pub(crate) fn get_gen_method_name(orig_name: &proc_macro2::Ident) -> proc_macro2::Ident {
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
    let gen_constructor_ident = get_gen_method_name(method_name);

    let (arg_idents, arg_parsing) = utils::parse_native_args(&method_ast.sig.inputs, false);

    let tokens = quote! {
        #method_ast

        /// Generated constructor.
        pub fn #gen_constructor_ident(mut cx: neon::prelude::FunctionContext) -> neon::prelude::JsResult<neon::prelude::JsUndefined> {
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
    let method_ast = parse_macro_input!(input as ImplItemMethod);
    let method_name = &method_ast.sig.ident;
    let gen_method_name = get_gen_method_name(method_name);
    let output = &method_ast.sig.output;

    let (arg_idents, arg_parsing) = utils::parse_native_args(&method_ast.sig.inputs, true);

    let tokens = quote! {
        #method_ast

        // TODO DERIVE LIFETIME FROM OUTPUT
        pub fn #gen_method_name<'ctx>(mut cx: neon::prelude::FunctionContext<'ctx>) #output {
            use neon::prelude::Object;

            #(#arg_parsing)*

            let this = cx.this();
            let this = this.get(&mut cx, Self::THIS)?
                .downcast_or_throw::<neon::prelude::JsBox<Self>, _>(&mut cx)?;

            this.#method_name(cx, #(#arg_idents,)*)
        }
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
                Some(NeonMacrosAttrs::new(method.clone()))
            } else {
                None
            }
        })
        .flatten()
        .collect::<Vec<NeonMacrosAttrs>>();

    let impl_tree = ImplTree::new(attrs_for_each_decorated_method);
    if impl_tree.constructor.exposed {
        let gen_ctor_name = get_gen_method_name(&impl_tree.constructor.method.sig.ident);
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
        let gen_register_fn = {
            let gen_register_fn = quote! {
                pub fn __neon_gen_expose_register(cx: &mut neon::prelude::ModuleContext) -> neon::prelude::NeonResult<()> {
                    use neon::prelude::Object;
                    let constructor = neon::prelude::JsFunction::new(cx, Self::#gen_ctor_name)?;

                    let prototype = constructor
                        .get(cx, "prototype")?
                        .downcast_or_throw::<neon::prelude::JsObject, _>(cx)?;

                    #(
                        let f = neon::prelude::JsFunction::new(cx, Self::#gen_method_names)?;
                        prototype.set(cx, #js_names, f)?;
                    )*

                    cx.export_value(#struct_name, constructor)?;
                    Ok(())
                }
            };
            let gen_register_fn: proc_macro::TokenStream = gen_register_fn.into();
            parse_macro_input!(gen_register_fn as ImplItemMethod)
        };
        impl_ast.items.push(ImplItem::Method(gen_register_fn));
    }

    let tokens = quote! {
        #impl_ast
    };

    tokens.into()
}
