mod attr;

use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

use attr::*;

use crate::utils::parse_attrs;

pub fn derive(input: proc_macro::TokenStream) -> syn::Result<proc_macro::TokenStream> {
    let DeriveInput {
        ident, data, attrs, ..
    } = &parse_macro_input::parse::<DeriveInput>(input)?;

    let mut attrs = parse_attrs(attrs)?;
    let container_attrs = ContainerAttr::from_attrs(&mut attrs)?;

    let crate_ref = format_ident!(
        "{}",
        container_attrs
            .crate_name
            .unwrap_or_else(|| "specta".into())
    );

    Ok(match data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(_) => {
                let fields = data
                    .fields
                    .iter()
                    .map(|field| {
                        let mut attrs = parse_attrs(&field.attrs)?;
                        let field_attrs = FieldAttr::from_attrs(&mut attrs)?;

                        Ok((field, field_attrs))
                    })
                    .collect::<syn::Result<Vec<_>>>()?;
                let fields = fields.iter().filter_map(|(field, attrs)| {
                    if attrs.skip {
                        return None;
                    }

                    let ident = &field.ident;

                    Some(quote! {
                        #crate_ref::ObjectField {
                            key: stringify!(#ident),
                            optional: false,
                            flatten: false,
                            ty: t.#ident.into(),
                        }
                    })
                });

                quote! {
                    #[automatically_derived]
                    impl From<#ident> for #crate_ref::ObjectType {
                        fn from(t: #ident) -> #crate_ref::ObjectType {
                            #crate_ref::ObjectType {
                                generics: vec![],
                                fields: vec![#(#fields),*],
                                tag: None,
                                module_path: None,
                        }
                        }
                    }

                    #[automatically_derived]
                    impl From<#ident> for #crate_ref::DataType {
                        fn from(t: #ident) -> #crate_ref::DataType {
                            #crate_ref::DataType::Object(t.into())
                        }
                    }
                }
            }
            Fields::Unnamed(_) => {
                let fields = data.fields.iter().enumerate().map(|(i, _)| {
                    let i = proc_macro2::Literal::usize_unsuffixed(i);
                    quote!(t.#i.into())
                });

                quote! {
                    #[automatically_derived]
                    impl From<#ident> for #crate_ref::TupleType {
                        fn from(t: #ident) -> #crate_ref::TupleType {
                            #crate_ref::TupleType {
                                generics: vec![],
                                fields: vec![#(#fields),*]
                            }
                        }
                    }

                    #[automatically_derived]
                    impl From<#ident> for #crate_ref::DataType {
                        fn from(t: #ident) -> #crate_ref::DataType {
                            #crate_ref::DataType::Tuple(t.into())
                        }
                    }
                }
            }
            Fields::Unit => {
                syn::Error::new_spanned(ident, "DataTypeFrom does not support unit structs")
                    .into_compile_error()
            }
        },
        _ => syn::Error::new_spanned(ident, "DataTypeFrom only supports structs")
            .into_compile_error(),
    }
    .into())
}
