use proc_macro::TokenStream;

use quote::quote;
use syn::{parse_macro_input, DeriveInput};

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

///
/// ## Attribute Args:
/// * `expose` - expose this constructor to the JS side.
#[proc_macro_attribute]
pub fn constructor(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(input);

    let tokens = quote! {
        #input

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
