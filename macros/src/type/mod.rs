use attr::*;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use r#enum::parse_enum;
use r#struct::parse_struct;
use syn::{parse_macro_input, Data, DeriveInput};

use generics::impl_heading;

use crate::utils::{parse_attrs, unraw_raw_ident};

use self::generics::{
    add_type_to_where_clause, generics_with_ident_and_bounds_only, generics_with_ident_only,
};

mod attr;
mod r#enum;
mod generics;
mod r#struct;

pub fn derive(input: proc_macro::TokenStream) -> syn::Result<proc_macro::TokenStream> {
    let DeriveInput {
        ident,
        generics,
        data,
        attrs,
        ..
    } = &parse_macro_input::parse::<DeriveInput>(input)?;

    // We pass all the attributes at the start and when decoding them pop them off the list.
    // This means at the end we can check for any that weren't consumed and throw an error.
    let mut attrs = parse_attrs(attrs)?;
    let container_attrs = ContainerAttr::from_attrs(&mut attrs)?;

    let raw_ident = ident;
    let ident = container_attrs
        .remote
        .clone()
        .unwrap_or_else(|| ident.to_token_stream());

    let crate_ref: TokenStream = container_attrs.crate_name.clone().unwrap_or(quote!(specta));

    let name = container_attrs.rename.clone().unwrap_or_else(|| {
        unraw_raw_ident(&format_ident!("{}", raw_ident.to_string())).to_token_stream()
    });

    let (inlines, reference, can_flatten) = match data {
        Data::Struct(data) => parse_struct(&name, &container_attrs, generics, &crate_ref, data),
        Data::Enum(data) => parse_enum(
            &name,
            &EnumAttr::from_attrs(&container_attrs, &mut attrs)?,
            &container_attrs,
            generics,
            &crate_ref,
            data,
        ),
        Data::Union(data) => Err(syn::Error::new_spanned(
            data.union_token,
            "specta: Union types are not supported by Specta yet!",
        )),
    }?;

    attrs
        .iter()
        .find(|attr| attr.root_ident == "specta")
        .map_or(Ok(()), |attr| {
            Err(syn::Error::new(
                attr.key.span(),
                format!(
                    "specta: Found unsupported container attribute '{}'",
                    attr.key
                ),
            ))
        })?;

    let definition_generics = generics.type_params().map(|param| {
        let ident = param.ident.to_string();
        quote!(std::borrow::Cow::Borrowed(#ident).into())
    });

    let bounds = generics_with_ident_and_bounds_only(generics);
    let type_args = generics_with_ident_only(generics);
    let where_bound = add_type_to_where_clause(&quote!(#crate_ref::Type), generics);

    let flatten_impl = can_flatten.then(|| {
        quote! {
            #[automatically_derived]
            impl #bounds #crate_ref::Flatten for #ident #type_args #where_bound {}
        }
    });

    let type_impl_heading = impl_heading(quote!(#crate_ref::Type), &ident, generics);

    let export = (cfg!(feature = "export") && container_attrs.export.unwrap_or(true)).then(|| {
        let export_fn_name = format_ident!("__push_specta_type_{}", raw_ident);

        let generic_params = generics
            .params
            .iter()
            .filter(|param| matches!(param, syn::GenericParam::Type(_)))
            .map(|_| quote! { () });

        quote! {
            #[allow(non_snake_case)]
            #[#crate_ref::internal::__specta_ctor]
            fn #export_fn_name() {
                #crate_ref::internal::export::register_ty::<#ident<#(#generic_params),*>>();
            }
        }
    });

    let comments = {
        let comments = &container_attrs.doc;
        quote!(vec![#(#comments.into()),*])
    };
    let should_export = match container_attrs.export {
        Some(export) => quote!(Some(#export)),
        None => quote!(None),
    };
    let deprecated = match &container_attrs.deprecated {
        Some(msg) => quote!(Some(#msg.into())),
        None => quote!(None),
    };

    let sid = quote!(#crate_ref::internal::construct::sid(#name, concat!("::", module_path!(), ":", line!(), ":", column!())));
    let impl_location = quote!(#crate_ref::internal::construct::impl_location(concat!(file!(), ":", line!(), ":", column!())));

    Ok(quote! {
        const _: () = {
        	const SID: #crate_ref::SpectaID = #sid;
	        const IMPL_LOCATION: #crate_ref::ImplLocation = #impl_location;

            #[automatically_derived]
            #type_impl_heading {
                fn inline(opts: #crate_ref::DefOpts, generics: &[#crate_ref::DataType]) -> #crate_ref::DataType {
                    #inlines
                }

                fn definition_generics() -> Vec<#crate_ref::GenericType> {
                    vec![#(#definition_generics),*]
                }

                fn reference(opts: #crate_ref::DefOpts, generics: &[#crate_ref::DataType]) -> #crate_ref::reference::Reference {
                    #reference
                }
            }

            #[automatically_derived]
            impl #bounds #crate_ref::NamedType for #ident #type_args #where_bound {
	            const SID: #crate_ref::SpectaID = SID;
	            const IMPL_LOCATION: #crate_ref::ImplLocation = IMPL_LOCATION;

                fn named_data_type(opts: #crate_ref::DefOpts, generics: &[#crate_ref::DataType]) -> #crate_ref::NamedDataType {
                    #crate_ref::internal::construct::named_data_type(
                        #name,
                        #comments,
                        #deprecated,
                        SID,
                        IMPL_LOCATION,
                        #should_export,
                        <Self as #crate_ref::Type>::inline(opts, generics)
                    )
                }
            }

            #flatten_impl

            #export
        };

    }.into())
}
