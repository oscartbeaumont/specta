use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::Result;

use crate::utils::{Attribute, Inflection};

#[derive(Default, Clone)]
pub struct ContainerAttr {
    pub rename_all: Option<Inflection>,
    pub rename: Option<TokenStream>,
    pub tag: Option<String>,
    pub crate_name: Option<String>,
    pub inline: bool,
    pub remote: Option<String>,
    pub export: Option<bool>, // Option is used because if not explicitly set, we enable it
    pub doc: Vec<String>,
    pub deprecated: Option<String>,
    pub module_path: Option<&'static str>,
}

impl_parse! {
    ContainerAttr(attr, out) {
        "rename_all" => out.rename_all = out.rename_all.take().or(Some(attr.parse_inflection()?)),
        "rename" => {
            let attr = attr.parse_string()?;
            out.rename = out.rename.take().or_else(|| Some({
                let name = crate::r#type::unraw_raw_ident(&quote::format_ident!("{}", attr));
                quote::quote!( #name )
            }))
        },
        // TODO: This is deprecated and will be removed in a future version. Use `rename_from_path` method instead.
        "rename_to_value" => {
            let attr = attr.parse_path()?;
            out.rename = out.rename.take().or_else(|| Some({
                let expr = attr.to_token_stream();
                quote::quote!( #expr )
            }))
        },
        "rename_from_path" => {
            let attr = attr.parse_path()?;
            out.rename = out.rename.take().or_else(|| Some({
                let expr = attr.to_token_stream();
                quote::quote!( #expr )
            }))
        },
        "tag" => out.tag = out.tag.take().or(Some(attr.parse_string()?)),
        "crate" => {
            if attr.root_ident == "specta" {
                out.crate_name = out.crate_name.take().or(Some(attr.parse_string()?));
            }
        },
        "inline" => out.inline = attr.parse_bool().unwrap_or(true),
        "remote" => out.remote = out.remote.take().or(Some(attr.parse_string()?)),
        "export" => out.export = out.export.take().or(Some(attr.parse_bool().unwrap_or(true))),
        "doc" => {
            if attr.key == "doc" {
                out.doc.push(attr.parse_string()?);
            }
        },
        // TODO: Finish implementing by supporting the official `#[deprecated]` attribute: https://github.com/oscartbeaumont/specta/issues/32
        "deprecated" => {
            if attr.key == "specta" {
                out.deprecated = out.deprecated.take().or(Some(attr.parse_string()?));
            }
        },
    }
}

impl ContainerAttr {
    pub fn from_attrs(attrs: &mut Vec<Attribute>) -> Result<Self> {
        let mut result = Self::default();
        Self::try_from_attrs("specta", attrs, &mut result)?;
        #[cfg(feature = "serde")]
        Self::try_from_attrs("serde", attrs, &mut result)?;
        Self::try_from_attrs("doc", attrs, &mut result)?;
        Ok(result)
    }
}
