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

pub fn derive(
    input: proc_macro::TokenStream,
    default_crate_name: String,
) -> syn::Result<proc_macro::TokenStream> {
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

    let ident = container_attrs
        .remote
        .as_ref()
        .map(|i| format_ident!("{}", i))
        .unwrap_or_else(|| ident.clone());

    let crate_name: TokenStream = container_attrs
        .crate_name
        .clone()
        .unwrap_or(default_crate_name)
        .parse()
        .unwrap();

    let name = container_attrs.rename.clone().unwrap_or_else(|| {
        unraw_raw_ident(&format_ident!("{}", ident.to_string())).to_token_stream()
    });

    let (inlines, category, can_flatten) = match data {
        Data::Struct(data) => parse_struct(
            &name,
            (&container_attrs, StructAttr::from_attrs(&mut attrs)?),
            generics,
            &crate_name,
            data,
        ),
        Data::Enum(data) => parse_enum(
            &name,
            &EnumAttr::from_attrs(&container_attrs, &mut attrs)?,
            &container_attrs,
            generics,
            &crate_name,
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
        let ident = &param.ident;

        quote!(#crate_name::GenericType(stringify!(#ident)))
    });

    let bounds = generics_with_ident_and_bounds_only(generics);
    let type_args = generics_with_ident_only(generics);
    let where_bound = add_type_to_where_clause(&quote!(#crate_name::Type), generics);

    let flatten_impl = can_flatten.then(|| {
        quote! {
            #[automatically_derived]
            impl #bounds #crate_name::Flatten for #ident #type_args #where_bound {}
        }
    });

    let type_impl_heading = impl_heading(quote!(#crate_name::Type), &ident, generics);

    let export = (cfg!(feature = "export") && container_attrs.export.unwrap_or(true)).then(|| {
        let export_fn_name = format_ident!("__push_specta_type_{}", ident);

        let generic_params = generics
            .params
            .iter()
            .filter(|param| matches!(param, syn::GenericParam::Type(_)))
            .map(|_| quote! { () });
        let ty = quote!(<#ident<#(#generic_params),*> as #crate_name::Type>);

        quote! {
            #[#crate_name::internal::ctor::ctor]
            #[allow(non_snake_case)]
            fn #export_fn_name() {
                let (type_map, errors) = &mut *#crate_name::export::TYPES.lock().unwrap();

                if let Err(err) = #ty::reference(
                    #crate_name::DefOpts {
                        parent_inline: false,
                        type_map
                    },
                    &[]
                ) {
                    errors.insert(err);
                }
            }
        }
    });

    Ok(quote! {
        const _: () = {
            // We do this so `sid!()` is only called once, preventing the type ended up with multiple ids
            const SID: #crate_name::TypeSid = #crate_name::sid!(@with_specta_path; #name; #crate_name);
            const IMPL_LOCATION: #crate_name::ImplLocation = #crate_name::impl_location!(@with_specta_path; #crate_name);

            const PATH_MACRO: &str = module_path!();

            #[automatically_derived]
            #type_impl_heading {
                const MODULE_PATH: &'static str = module_path!();
                
                fn inline(opts: #crate_name::DefOpts, generics: &[#crate_name::DataType]) -> std::result::Result<#crate_name::DataType, #crate_name::ExportError> {
                    Ok(#crate_name::DataType::Named(<Self as #crate_name::NamedType>::named_data_type(opts, generics)?))
                }

                fn category_impl(opts: #crate_name::DefOpts, generics: &[#crate_name::DataType]) -> std::result::Result<#crate_name::TypeCategory, #crate_name::ExportError> {
                    Ok(#category)
                }

                fn definition_generics() -> Vec<#crate_name::GenericType> {
                    vec![#(#definition_generics),*]
                }
            }

            #[automatically_derived]
            impl #bounds #crate_name::NamedType for #ident #type_args #where_bound {
                fn named_data_type(opts: #crate_name::DefOpts, generics: &[#crate_name::DataType]) ->  std::result::Result<#crate_name::NamedDataType, #crate_name::ExportError> {
                    Ok(#inlines)
                }
            }

            #flatten_impl

            #export
        };

    }.into())
}

pub fn named_data_type_wrapper(
    crate_ref: &TokenStream,
    container_attrs: &ContainerAttr,
    name: &TokenStream,
    t: TokenStream,
) -> TokenStream {
    let comments = {
        let comments = &container_attrs.doc;
        quote!(&[#(#comments),*])
    };
    let should_export = match container_attrs.export {
        Some(export) => quote!(Some(#export)),
        None => quote!(None),
    };
    let deprecated = match &container_attrs.deprecated {
        Some(msg) => quote!(Some(#msg)),
        None => quote!(None),
    };

    let module_path = match &container_attrs.module_path {
        Some(path) => quote!(Some(#path)),
        None => quote!(Some(PATH_MACRO)),
    };

    quote! {
        #crate_ref::NamedDataType {
            name: #name,
            sid: Some(SID),
            impl_location: Some(IMPL_LOCATION),
            comments: #comments,
            export: #should_export,
            deprecated: #deprecated,
            module_path: #module_path,
            item: #t
        }
    }
}
