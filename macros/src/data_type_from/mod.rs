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

    let crate_name = format_ident!(
        "{}",
        container_attrs
            .crate_name
            .unwrap_or_else(|| "specta".into())
    );

    let body = match data {
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
                        #crate_name::ObjectField {
                            key: stringify!(#ident),
                            optional: false,
                            flatten: false
                            ty: t.#ident, // .into(), // DataTypeItem -> DataType // TODO: Fix this
                        }
                    })
                });

                quote! {
                    #crate_name::ObjectType {
                        name: stringify!(#ident),
                        generics: vec![],
                        fields: vec![#(#fields),*],
                        tag: None,
                    }.into()
                }
            }
            Fields::Unnamed(_) => {
                let fields = data.fields.iter().enumerate().map(|(i, _)| {
                    let i = proc_macro2::Literal::usize_unsuffixed(i);
                    quote!(t.#i.into()) // TODO: Maybe need to fix this `into`?
                });

                quote! {
                    #crate_name::TupleType {
                        generics: vec![],
                        fields: vec![#(#fields),*]
                    }.into() // TODO: Maybe need to fix this `into`?
                }
            }
            _ => todo!("ToDataType only supports named structs"),
        },
        _ => todo!("ToDataType only supports named structs"),
    };

    Ok(quote! {
        #[automatically_derived]
        impl From<#ident> for #crate_name::DataType {
            fn from(t: #ident) -> Self {
                // This impl is created as a unique type.
                #crate_name::DataType {
                    name: stringify!(#ident),
                    sid: #crate_name::sid!(stringify!(#ident), #crate_name::impl_location!().as_str()),
                    impl_location: #crate_name::impl_location!(),
                    item: #body,
                }
            }
        }
    }
    .into())
}
