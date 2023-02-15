use syn::Result;

use crate::utils::Attribute;

#[derive(Default)]
pub struct FieldAttr {
    pub skip: bool,
}

impl_parse! {
    FieldAttr(attr, out) {
        "skip" => out.skip = true,
    }
}

impl FieldAttr {
    pub fn from_attrs(attrs: &mut Vec<Attribute>) -> Result<Self> {
        let mut result = Self::default();
        Self::try_from_attrs("specta", attrs, &mut result)?;
        Ok(result)
    }
}
