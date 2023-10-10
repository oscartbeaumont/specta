use super::{attr::*, generics::construct_datatype, r#struct::decode_field_attrs};
use crate::utils::*;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{spanned::Spanned, DataEnum, Fields, GenericParam, Generics};

pub fn parse_enum(
    name: &TokenStream,
    enum_attrs: &EnumAttr,
    container_attrs: &ContainerAttr,
    generics: &Generics,
    crate_ref: &TokenStream,
    data: &DataEnum,
) -> syn::Result<(TokenStream, TokenStream, bool)> {
    if container_attrs.transparent {
        return Err(syn::Error::new(
            data.enum_token.span(),
            "#[specta(transparent)] is not allowed on an enum",
        ));
    }

    let generic_idents = generics
        .params
        .iter()
        .enumerate()
        .filter_map(|(i, p)| match p {
            GenericParam::Type(t) => Some((i, &t.ident)),
            _ => None,
        });

    let definition_generics = generic_idents.clone().map(|(_, ident)| {
        let ident = ident.to_string();
        quote!(std::borrow::Cow::Borrowed(#ident).into())
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
                .unwrap_or_else(|| <#ident as #crate_ref::Type>::reference(
                    #crate_ref::DefOpts {
                        parent_inline: #parent_inline,
                        type_map: opts.type_map,
                    },
                    &[],
                ).inner)
        }
    });

    let repr = enum_attrs.tagged()?;
    let variant_types =
        data.variants
            .iter()
            .map(|v| {
                // We pass all the attributes at the start and when decoding them pop them off the list.
                // This means at the end we can check for any that weren't consumed and throw an error.
                let mut attrs = parse_attrs(&v.attrs)?;
                let variant_attrs = VariantAttr::from_attrs(&mut attrs)?;

                // The expectation is that when an attribute is processed it will be removed so if any are left over we know they are invalid
                // but we only throw errors for Specta-specific attributes so we don't continually break other attributes.
                if let Some(attrs) = attrs.iter().find(|attr| attr.key == "specta") {
                    match &attrs.value {
                        Some(AttributeValue::Attribute { attr, .. }) => {
                            if let Some(attr) = attr.first() {
                                return Err(syn::Error::new(
                                    attr.key.span(),
                                    format!(
                                        "specta: Found unsupported enum attribute '{}'",
                                        attr.key
                                    ),
                                ));
                            }
                        }
                        _ => todo!(),
                    }
                }

                Ok((v, variant_attrs))
            })
            .collect::<syn::Result<Vec<_>>>()?
            .into_iter()
            .map(|(variant, attrs)| {
                let variant_ident_str = unraw_raw_ident(&variant.ident);

                let variant_name_str = match (attrs.rename, container_attrs.rename_all) {
                    (Some(name), _) => name,
                    (_, Some(inflection)) => inflection.apply(&variant_ident_str).to_token_stream(),
                    (_, _) => variant_ident_str.to_token_stream(),
                };

                let generic_idents = generic_idents.clone().collect::<Vec<_>>();

                let inner = match &variant.fields {
                    Fields::Unit => quote!(#crate_ref::internal::construct::enum_variant_unit()),
                    Fields::Unnamed(fields) => {
                        let fields = fields
                            .unnamed
                            .iter()
                            .map(|field| {
                                let field_attrs = decode_field_attrs(field)?;
                                let deprecated = field_attrs.common.deprecated_as_tokens(crate_ref);
                                let field_ty = field_attrs.r#type.as_ref().unwrap_or(&field.ty);
                                let optional = field_attrs.optional;
                                let flatten = field_attrs.flatten;
                                let doc = field_attrs.common.doc;

                                let ty = field_attrs.skip.then(|| Ok(quote!(None)))
                                    .unwrap_or_else(|| {
	                                    construct_datatype(
	                                        format_ident!("gen"),
	                                        field_ty,
	                                        &generic_idents,
	                                        crate_ref,
	                                        attrs.inline,
	                                    ).map(|generic_vars| {
											quote! {{
			                                    #generic_vars

			                                    Some(gen)
			                                }}
										})
                                    })?;

                                Ok(quote!(#crate_ref::internal::construct::field(
                                    #optional,
                                    #flatten,
                                    #deprecated,
                                    #doc.into(),
                                    #ty
                                )))
                            })
                            .collect::<syn::Result<Vec<TokenStream>>>()?;

                        quote!(#crate_ref::internal::construct::enum_variant_unnamed(
                            vec![#(#fields),*],
                        ))
                    }
                    Fields::Named(fields) => {
                        let fields = fields
                        .named
                        .iter()
                        .map(|field| {
                            let field_attrs = decode_field_attrs(field)?;

                            let field_ty = field_attrs.r#type.as_ref().unwrap_or(&field.ty);

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
                            let deprecated = field_attrs.common.deprecated_as_tokens(crate_ref);
                            let optional = field_attrs.optional;
                            let flatten = field_attrs.flatten;
                            let doc = field_attrs.common.doc;

                            let ty = field_attrs.skip.then(|| Ok(quote!(None)))
                                .unwrap_or_else(|| {
	                                 construct_datatype(
			                            format_ident!("gen"),
			                            field_ty,
			                            &generic_idents,
			                            crate_ref,
			                            attrs.inline,
			                        ).map(|generic_vars|{
										quote! {
											Some({
												#generic_vars

												gen
											})
										}
									})
                                })?;

                            Ok(quote!((#field_name.into(), #crate_ref::internal::construct::field(
                                #optional,
                                #flatten,
                                #deprecated,
                                #doc.into(),
                                #ty
                            ))))
                        })
                        .collect::<syn::Result<Vec<TokenStream>>>()?;

                        quote!(#crate_ref::internal::construct::enum_variant_named(vec![#(#fields),*], None))
                    }
                };

                let deprecated = attrs.common.deprecated_as_tokens(crate_ref);
                let skip = attrs.skip;
                let doc = attrs.common.doc;
                Ok(quote!((#variant_name_str.into(), #crate_ref::internal::construct::enum_variant(#skip, #deprecated, #doc.into(), #inner))))
            })
            .collect::<syn::Result<Vec<_>>>()?;

    let (repr, can_flatten) = match repr {
        Tagged::Untagged => (
            quote!(#crate_ref::EnumRepr::Untagged),
            data.variants
                .iter()
                .any(|v| matches!(&v.fields, Fields::Unit | Fields::Named(_))),
        ),
        Tagged::Externally => (
            quote!(#crate_ref::EnumRepr::External),
            data.variants.iter().any(|v| match &v.fields {
                Fields::Unnamed(f) if f.unnamed.len() == 1 => true,
                Fields::Named(_) => true,
                _ => false,
            }),
        ),
        Tagged::Adjacently { tag, content } => (
            quote!(#crate_ref::EnumRepr::Adjacent { tag: #tag.into(), content: #content.into() }),
            true,
        ),
        Tagged::Internally { tag } => (
            quote!(#crate_ref::EnumRepr::Internal { tag: #tag.into() }),
            data.variants
                .iter()
                .any(|v| matches!(&v.fields, Fields::Unit | Fields::Named(_))),
        ),
    };

    Ok((
        quote!(#crate_ref::DataType::Enum(#crate_ref::internal::construct::r#enum(#name.into(), #repr, vec![#(#definition_generics),*], vec![#(#variant_types),*]))),
        quote!({
            let generics = vec![#(#reference_generics),*];
            #crate_ref::reference::reference::<Self>(opts, &generics, #crate_ref::internal::construct::data_type_reference(
                #name.into(),
                SID,
                generics.clone() // TODO: This `clone` is cringe
            ))
        }),
        can_flatten,
    ))
}
