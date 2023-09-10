// inspired by https://github.com/tauri-apps/tauri/blob/2901145c497299f033ba7120af5f2e7ead16c75a/core/tauri-macros/src/command/handler.rs

use quote::{quote, ToTokens};
use syn::{parse_macro_input, FnArg, ItemFn, Pat, ReturnType, Visibility};

use crate::utils::format_fn_wrapper;

pub fn attribute(item: proc_macro::TokenStream) -> syn::Result<proc_macro::TokenStream> {
    let function = parse_macro_input::parse::<ItemFn>(item)?;
    let wrapper = format_fn_wrapper(&function.sig.ident);
    let crate_ref = quote!(specta);

    let visibility = &function.vis;
    let maybe_macro_export = match &visibility {
        Visibility::Public(_) => quote!(#[macro_export]),
        _ => Default::default(),
    };

    let ident = &function.sig.ident;
    let name = ident.to_string();
    let asyncness = function.sig.asyncness.is_some();

    let args = function.sig.inputs.iter().map(|input| match input {
        FnArg::Receiver(_) => return Err(syn::Error::new_spanned(
            input,
            "functions with `#[specta]` cannot take 'self'",
        )),
        FnArg::Typed(arg) => {
            let name = &arg.pat.to_token_stream().to_string();
            let ty = &arg.ty;
            Ok(quote!(
                (
                    #name.into(),
                    <#ty as #crate_ref::internal::functions::FunctionArg<_>>::to_datatype(#crate_ref::DefOpts {
                        parent_inline: false,
                        type_map,
                    })
                )
            ))
        }
    }).collect::<syn::Result<Vec<_>>>()?;

    let output = match &function.sig.output {
        ReturnType::Default => quote!(()),
        ReturnType::Type(_, ty) => quote!(#ty),
    };

    let docs = function
        .attrs
        .iter()
        .filter(|attr| attr.path.is_ident("doc"))
        .filter_map(|attr| match attr.parse_meta() {
            Ok(syn::Meta::NameValue(v)) => Some(v.lit),
            _ => None,
        });

    Ok(quote! {
        #function

        #maybe_macro_export
        #[doc(hidden)]
        macro_rules! #wrapper {
            (@internal) => {{
                // fn export_self(type_map: &mut #crate_ref::TypeMap) -> #crate_ref::FunctionType {
                //     #crate_ref::internal::construct::function_type(
                //         #asyncness,
                //         #name,
                //         vec![#(#args),*],
                //         <#output as #crate_ref::internal::functions::FunctionOutput<_>>::to_datatype(#crate_ref::DefOpts {
                //             parent_inline: false,
                //             type_map,
                //         }),
                //         vec![#(#docs.into()),*]
                //     )
                // }
                // export_self as fn(&mut _) -> _

                let f: fn(&mut _) -> _ = |type_map| {
                    // todo!();
                    #crate_ref::internal::construct::function_type(
                        #asyncness,
                        #name,
                        vec![#(#args),*],
                        <#output as #crate_ref::internal::functions::FunctionOutput<_>>::to_datatype(#crate_ref::DefOpts {
                            parent_inline: false,
                            type_map,
                        }),
                        vec![#(#docs.into()),*]
                    )
                };

                #crate_ref::internal::construct::function(&f)
            }}
        }

        // allow the macro to be resolved with the same path as the function
        #[allow(unused_imports)]
        #visibility use #wrapper;
    }
    .into())
}
