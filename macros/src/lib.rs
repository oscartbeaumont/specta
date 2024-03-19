//! Easily export your Rust types to other languages
//!
//! This crate contains the macro which are reexported by the `specta` crate.
//! You shouldn't need to use this crate directly.
//! Checkout [Specta](https://docs.rs/specta).
#![doc(
    html_logo_url = "https://github.com/oscartbeaumont/specta/raw/main/.github/logo-128.png",
    html_favicon_url = "https://github.com/oscartbeaumont/specta/raw/main/.github/logo-128.png"
)]

#[macro_use]
mod utils;
mod data_type_from;
#[cfg(feature = "functions")]
mod fn_datatype;
#[cfg(feature = "functions")]
mod specta;
mod r#type;

#[proc_macro_derive(Type, attributes(specta, serde))]
pub fn derive_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    r#type::derive(input).unwrap_or_else(|err| err.into_compile_error().into())
}

#[proc_macro_derive(DataTypeFrom, attributes(specta))]
pub fn derive_data_type_from(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    data_type_from::derive(input).unwrap_or_else(|err| err.into_compile_error().into())
}

#[proc_macro_attribute]
#[cfg(feature = "functions")]
pub fn specta(
    _: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    specta::attribute(item).unwrap_or_else(|err| err.into_compile_error().into())
}

#[proc_macro]
#[cfg(feature = "functions")]
pub fn fn_datatype(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    use syn::parse_macro_input;

    fn_datatype::proc_macro(parse_macro_input!(input as fn_datatype::FnDatatypeInput))
        .unwrap_or_else(|err| err.into_compile_error())
        .into()
}

#[proc_macro_attribute]
// #[cfg(test)] // TODO: Require special internal feature so it's not for user's tests
pub fn stest(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    use quote::quote;
    use syn::{parse_macro_input, ItemFn};

    let args: proc_macro2::TokenStream = args.into();

    let function = match parse_macro_input::parse::<ItemFn>(item) {
        Ok(function) => function,
        Err(err) => return err.into_compile_error().into(),
    };
    let function_ident = function.sig.ident.clone();

    quote! {
        #[test]
        #[ignore]
        #[cfg(not(#args))]
        fn #function_ident() {}

        #[test]
        #[cfg(#args)]
        #function
    }
    .into()
}
