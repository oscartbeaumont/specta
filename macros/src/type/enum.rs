use super::{
    attr::*, generics::construct_datatype, named_data_type_wrapper, r#struct::decode_field_attrs,
};
use crate::utils::*;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DataEnum, Fields, GenericParam, Generics};

pub fn parse_enum(
    name: &TokenStream,
    enum_attrs: &EnumAttr,
    container_attrs: &ContainerAttr,
    generics: &Generics,
    crate_ref: &TokenStream,
    data: &DataEnum,
) -> syn::Result<(TokenStream, TokenStream, bool)> {
    let generic_idents = generics
        .params
        .iter()
        .enumerate()
        .filter_map(|(i, p)| match p {
            GenericParam::Type(t) => Some((i, &t.ident)),
            _ => None,
        });

    let definition_generics = generic_idents.clone().map(|(_, ident)| {
        let ident = &ident.clone();

        quote!(stringify!(#ident))
    });

    let parent_inline = container_attrs
        .inline
        .then(|| quote!(true))
        .unwrap_or(quote!(false));

    let reference_generics = generic_idents.clone().map(|(i, ident)| {
        let ident = &ident.clone();

        quote! {
            generics
                .get(#i)
                .cloned()
                .map_or_else(|| <#ident as #crate_ref::Type>::reference(
                    #crate_ref::DefOpts {
                        parent_inline: #parent_inline,
                        type_map: opts.type_map,
                    },
                    &[],
                ), Ok)?
        }
    });

    let repr = enum_attrs.tagged()?;
    let (variant_names, variant_types): (Vec<_>, Vec<_>) = data
        .variants
        .iter()
        .map(|v| {
            // We pass all the attributes at the start and when decoding them pop them off the list.
            // This means at the end we can check for any that weren't consumed and throw an error.
            let mut attrs = parse_attrs(&v.attrs)?;
            let variant_attrs = VariantAttr::from_attrs(&mut attrs)?;

            attrs
                .iter()
                .find(|attr| attr.root_ident == "specta")
                .map_or(Ok(()), |attr| {
                    Err(syn::Error::new(
                        attr.key.span(),
                        format!("specta: Found unsupported enum attribute '{}'", attr.key),
                    ))
                })?;

            Ok((v, variant_attrs))
        })
        .collect::<syn::Result<Vec<_>>>()?
        .into_iter()
        .filter(|(_, attrs)| !attrs.skip)
        .map(|(variant, attrs)| {
            let variant_ident_str = unraw_raw_ident(&variant.ident);

            let variant_name_str = match (attrs.rename, container_attrs.rename_all) {
                (Some(name), _) => name,
                (_, Some(inflection)) => inflection.apply(&variant_ident_str),
                (_, _) => variant_ident_str,
            };

            let generic_idents = generic_idents.clone().collect::<Vec<_>>();

            Ok((
                variant_name_str,
                match &variant.fields {
                    Fields::Unit => {
                        quote!(#crate_ref::EnumVariant::Unit)
                    }
                    Fields::Unnamed(fields) => {
                        let fields = fields
                            .unnamed
                            .iter()
                            .map(|field| {
                                let (field, field_attrs) = decode_field_attrs(field)?;
                                let field_ty = field_attrs.r#type.as_ref().unwrap_or(&field.ty);

                                let generic_vars = construct_datatype(
                                    format_ident!("gen"),
                                    field_ty,
                                    &generic_idents,
                                    crate_ref,
                                    attrs.inline,
                                )?;

                                Ok(quote!({
                                    #generic_vars

                                    gen
                                }))
                            })
                            .collect::<syn::Result<Vec<TokenStream>>>()?;

                        quote!(#crate_ref::EnumVariant::Unnamed(#crate_ref::TupleType {
                            fields: vec![#(#fields),*],
                            generics: vec![]
                        }))
                    }
                    Fields::Named(fields) => {
                        let fields = fields
                            .named
                            .iter()
                            .map(|field| {
                                let (field, field_attrs) = decode_field_attrs(field)?;

                                let field_ty = field_attrs.r#type.as_ref().unwrap_or(&field.ty);

                                let generic_vars = construct_datatype(
                                    format_ident!("gen"),
                                    field_ty,
                                    &generic_idents,
                                    crate_ref,
                                    attrs.inline,
                                )?;

                                let field_ident_str =
                                    unraw_raw_ident(field.ident.as_ref().unwrap());

                                let field_name = match (field_attrs.rename, attrs.rename_all) {
                                    (Some(name), _) => name,
                                    (_, Some(inflection)) => {
                                        let name = inflection.apply(&field_ident_str);
                                        quote::quote!(#name)
                                    }
                                    (_, _) => quote::quote!(#field_ident_str),
                                };

                                Ok(quote!(#crate_ref::ObjectField {
                                    key: #field_name,
                                    optional: false,
                                    flatten: false,
                                    ty: {
                                        #generic_vars

                                        gen
                                    },
                                }))
                            })
                            .collect::<syn::Result<Vec<TokenStream>>>()?;

                        quote!(#crate_ref::EnumVariant::Named(#crate_ref::ObjectType {
                            fields: vec![#(#fields),*],
                            generics: vec![],
                            tag: None,
                            module_path: Some(<Self as #crate_ref::Type>::MODULE_PATH)
                        }))
                    }
                },
            ))
        })
        .collect::<syn::Result<Vec<_>>>()?
        .into_iter()
        .unzip();

    let (enum_impl, can_flatten) = match repr {
        Tagged::Untagged => (
            quote! {
                #crate_ref::EnumType::Untagged {
                    generics: vec![#(#definition_generics),*],
                    variants: vec![#(#variant_types),*],
                }
            },
            data.variants
                .iter()
                .any(|v| matches!(&v.fields, Fields::Unit | Fields::Named(_))),
        ),
        Tagged::Externally => (
            quote! {
                #crate_ref::EnumType::Tagged {
                    generics: vec![#(#definition_generics),*],
                    variants: vec![#((#variant_names, #variant_types)),*],
                    repr: #crate_ref::EnumRepr::External,
                }
            },
            data.variants.iter().any(|v| match &v.fields {
                Fields::Unnamed(f) if f.unnamed.len() == 1 => true,
                Fields::Named(_) => true,
                _ => false,
            }),
        ),
        Tagged::Adjacently { tag, content } => (
            quote! {
                #crate_ref::EnumType::Tagged {
                    generics: vec![#(#definition_generics),*],
                    variants: vec![#((#variant_names, #variant_types)),*],
                    repr: #crate_ref::EnumRepr::Adjacent { tag: #tag, content: #content },
                }
            },
            true,
        ),
        Tagged::Internally { tag } => (
            quote! {
                #crate_ref::EnumType::Tagged {
                    generics: vec![#(#definition_generics),*],
                    variants: vec![#((#variant_names, #variant_types)),*],
                    repr: #crate_ref::EnumRepr::Internal { tag: #tag },
                }
            },
            data.variants
                .iter()
                .any(|v| matches!(&v.fields, Fields::Unit | Fields::Named(_))),
        ),
    };

    let body = named_data_type_wrapper(
        crate_ref,
        container_attrs,
        name,
        quote! {
            #crate_ref::NamedDataTypeItem::Enum(
                #enum_impl
            )
        },
    );

    Ok((
        body,
        quote! {
            #crate_ref::TypeCategory::Reference(#crate_ref::DataTypeReference {
                name: #name,
                sid: SID,
                generics: vec![#(#reference_generics),*],
                module_path: <Self as #crate_ref::Type>::MODULE_PATH
            })

        },
        can_flatten,
    ))
}
