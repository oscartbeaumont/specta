use crate::utils::{parse_attrs, unraw_raw_ident};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{spanned::Spanned, DataStruct, Field, Fields, GenericParam, Generics};

use super::{attr::*, generics::construct_datatype};

pub fn decode_field_attrs(field: &Field) -> syn::Result<(&Field, FieldAttr)> {
    // We pass all the attributes at the start and when decoding them pop them off the list.
    // This means at the end we can check for any that weren't consumed and throw an error.
    let mut attrs = parse_attrs(&field.attrs)?;
    let field_attrs = FieldAttr::from_attrs(&mut attrs)?;

    attrs
        .iter()
        .find(|attr| attr.root_ident == "specta")
        .map_or(Ok(()), |attr| {
            Err(syn::Error::new(
                attr.key.span(),
                format!("specta: Found unsupported field attribute '{}'", attr.key),
            ))
        })?;

    Ok((field, field_attrs))
}

pub fn parse_struct(
    name: &TokenStream,
    container_attrs: &ContainerAttr,
    generics: &Generics,
    crate_ref: &TokenStream,
    data: &DataStruct,
) -> syn::Result<(TokenStream, TokenStream, bool)> {
    let generic_idents = generics
        .params
        .iter()
        .enumerate()
        .filter_map(|(i, p)| match p {
            GenericParam::Type(t) => Some((i, &t.ident)),
            _ => None,
        })
        .collect::<Vec<_>>();

    let parent_inline = container_attrs
        .inline
        .then(|| quote!(true))
        .unwrap_or(quote!(false));

    let reference_generics = generic_idents.iter().map(|(i, ident)| {
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

    let definition_generics = generic_idents.iter().map(|(_, ident)| {
        let ident = ident.to_string();
        quote!(std::borrow::Cow::Borrowed(#ident).into())
    });

    let definition = if container_attrs.transparent {
        if let Fields::Unit = data.fields {
            return Err(syn::Error::new(
                data.fields.span(),
                "specta: unit structs cannot be transparent",
            ));
        } else if data.fields.len() != 1 {
            return Err(syn::Error::new(
                data.fields.span(),
                "specta: transparent structs must have exactly one field",
            ));
        }

        let (field, field_attrs) = decode_field_attrs(
            data.fields
                .iter()
                .next()
                .expect("unreachable: we just checked this!"),
        )?;

        let field_ty = field_attrs.r#type.as_ref().unwrap_or(&field.ty);

        let ty = construct_datatype(
            format_ident!("ty"),
            &field_ty,
            &generic_idents,
            crate_ref,
            field_attrs.inline,
        )?;

        quote!({
            #ty

            ty
        })
    } else {
        let fields = match &data.fields {
            Fields::Named(_) => {
                let fields = data.fields.iter().map(decode_field_attrs)
                .collect::<syn::Result<Vec<_>>>()?
                .iter()
                .filter_map(|(field, field_attrs)| {
    
                    if field_attrs.skip {
                        return None;
                    }
    
                    Some((field, field_attrs))
                }).map(|(field, field_attrs)| {
                    let field_ty = field_attrs.r#type.as_ref().unwrap_or(&field.ty);
    
                    let ty = construct_datatype(
                        format_ident!("ty"),
                        field_ty,
                        &generic_idents,
                        crate_ref,
                        field_attrs.inline,
                    )?;
    
                    let field_ident_str = unraw_raw_ident(field.ident.as_ref().unwrap());
    
                    let field_name = match (field_attrs.rename.clone(), container_attrs.rename_all) {
                        (Some(name), _) => name,
                        (_, Some(inflection)) => {
                            let name = inflection.apply(&field_ident_str);
                            quote::quote!(#name)
                        },
                        (_, _) => quote::quote!(#field_ident_str),
                    };
    
                    let optional = field_attrs.optional;
                    let flatten = field_attrs.flatten;
    
                    let parent_inline = container_attrs
                        .inline
                        .then(|| quote!(true))
                        .unwrap_or(parent_inline.clone());
    
                    let ty = if field_attrs.flatten {
                        quote! {
                            #[allow(warnings)]
                            {
                                #ty
                            }
    
                            fn validate_flatten<T: #crate_ref::Flatten>() {}
                            validate_flatten::<#field_ty>();
    
                            let mut ty = <#field_ty as #crate_ref::Type>::inline(#crate_ref::DefOpts {
                                parent_inline: #parent_inline,
                                type_map: opts.type_map
                            }, &generics);
    
                            ty
                        }
                    } else {
                        quote! {
                            #ty
    
                            ty
                        }
                    };
    
                    Ok(quote!((#field_name.into(), #crate_ref::internal::construct::field(
                        #optional,
                        #flatten,
                        {
                            #ty
                        }
                    ))))
                }).collect::<syn::Result<Vec<TokenStream>>>()?;
    
                let tag = container_attrs
                    .tag
                    .as_ref()
                    .map(|t| quote!(Some(#t.into())))
                    .unwrap_or(quote!(None));
    

                quote!(#crate_ref::internal::construct::struct_named(vec![#(#fields),*], #tag))
            }
            Fields::Unnamed(_) => {
                let fields = data
                    .fields
                    .iter()
                    .map(decode_field_attrs)
                    .collect::<syn::Result<Vec<_>>>()?
                    .iter()
                    .filter_map(|(field, field_attrs)| {
                        if field_attrs.skip {
                            return None;
                        }
    
                        Some((field, field_attrs))
                    })
                    .map(|(field, field_attrs)| {
                        let field_ty = field_attrs.r#type.as_ref().unwrap_or(&field.ty);
    
                        let generic_vars = construct_datatype(
                            format_ident!("gen"),
                            field_ty,
                            &generic_idents,
                            crate_ref,
                            field_attrs.inline,
                        )?;
    
                        let optional = field_attrs.optional;
                        let flatten = field_attrs.flatten;
                        Ok(quote!({
                            #generic_vars
    
                            #crate_ref::internal::construct::field(#optional, #flatten, gen)
                        }))
                    })
                    .collect::<syn::Result<Vec<TokenStream>>>()?;
    

                quote!(#crate_ref::internal::construct::struct_unnamed(vec![#(#fields),*]))
            }
            Fields::Unit => quote!(#crate_ref::internal::construct::struct_unit()),
        };

        quote!(#crate_ref::DataType::Struct(#crate_ref::internal::construct::r#struct(#name, vec![#(#definition_generics),*], #fields)))
    };

 

    let category = if container_attrs.inline {
        quote!({
            let generics = &[#(#reference_generics),*];
            #crate_ref::reference::inline::<Self>(opts, generics)
        })
    } else {
        quote!({
            let generics = vec![#(#reference_generics),*];
            #crate_ref::reference::reference::<Self>(opts, &generics, #crate_ref::internal::construct::data_type_reference(
                #name,
                SID,
                generics.clone() // TODO: This `clone` is cringe
            ))
        })
    };

    Ok((
        definition,
        category,
        true,
    ))
}
