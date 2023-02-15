use syn::Result;

use crate::utils::Attribute;

#[derive(Default)]
pub struct StructAttr {
    pub transparent: bool,
}

impl_parse! {
    StructAttr(attr, out) {
        "transparent" => out.transparent = attr.parse_bool().unwrap_or(true)
    }
}

impl StructAttr {
    pub fn from_attrs(attrs: &mut Vec<Attribute>) -> Result<Self> {
        let mut result = Self::default();
        Self::try_from_attrs("specta", attrs, &mut result)?;
        #[cfg(feature = "serde")]
        Self::try_from_attrs("serde", attrs, &mut result)?;
        Self::try_from_attrs("repr", attrs, &mut result)?; // To handle `#[repr(transparent)]`
        Ok(result)
    }
}
