//! This module contains functions that are public for the sole reason of the macros.
//!
//! They will not be documented and may go through breaking changes without a major version bump!
//!
//! DO NOT USE THEM! You have been warned!

// Renaming the export so it's less likely to end up in LSP hints

#[cfg(feature = "export")]
pub use ctor::ctor as __specta_ctor;

pub use paste::paste as __specta_paste;

#[cfg(feature = "functions")]
mod function_args;
#[cfg(feature = "functions")]
mod function_result;

/// Traits used by the `specta` macro to infer type information from functions.
#[cfg(feature = "functions")]
pub mod functions {
    pub use super::function_args::FunctionArg;
    pub use super::function_result::FunctionOutput;
}

/// Functions used to construct `crate::datatype` types (they have private fields so can't be constructed directly).
/// We intentionally keep their fields private so we can modify them without a major version bump.
/// As this module is `#[doc(hidden)]` we allowed to make breaking changes within a minor version as it's only used by the macros.
pub mod construct {
    use std::borrow::Cow;

    use crate::{datatype::*, ImplLocation, SpectaID};

    pub const fn field(optional: bool, flatten: bool, ty: DataType) -> Field {
        Field {
            optional,
            flatten,
            ty,
        }
    }

    pub const fn r#struct(
        name: &'static str,
        generics: Vec<GenericType>,
        fields: StructFields,
    ) -> StructType {
        StructType {
            name: Cow::Borrowed(name),
            generics,
            fields,
        }
    }

    pub const fn struct_unit() -> StructFields {
        StructFields::Unit
    }

    pub const fn struct_unnamed(fields: Vec<Field>) -> StructFields {
        StructFields::Unnamed(UnnamedFields { fields })
    }

    pub const fn struct_named(
        fields: Vec<(Cow<'static, str>, Field)>,
        tag: Option<Cow<'static, str>>,
    ) -> StructFields {
        StructFields::Named(NamedFields { fields, tag })
    }

    pub const fn r#enum(
        name: &'static str,
        repr: EnumRepr,
        generics: Vec<GenericType>,
        variants: Vec<(Cow<'static, str>, EnumVariant)>,
    ) -> EnumType {
        EnumType {
            name: Cow::Borrowed(name),
            repr,
            generics,
            variants,
        }
    }

    pub const fn enum_variant_unit() -> EnumVariant {
        EnumVariant::Unit
    }

    pub const fn enum_variant_unnamed(fields: Vec<Field>) -> EnumVariant {
        EnumVariant::Unnamed(UnnamedFields { fields })
    }

    pub const fn enum_variant_named(
        fields: Vec<(Cow<'static, str>, Field)>,
        tag: Option<Cow<'static, str>>,
    ) -> EnumVariant {
        EnumVariant::Named(NamedFields { fields, tag })
    }

    pub const fn named_data_type(
        name: &'static str,
        comments: Vec<Cow<'static, str>>,
        deprecated: Option<&'static str>,
        sid: SpectaID,
        impl_location: ImplLocation,
        export: Option<bool>,
        inner: DataType,
    ) -> NamedDataType {
        NamedDataType {
            name: Cow::Borrowed(name),
            comments,
            deprecated: match deprecated {
                Some(msg) => Some(Cow::Borrowed(msg)),
                None => None,
            },
            ext: Some(NamedDataTypeExt {
                sid,
                impl_location,
                export,
            }),
            inner,
        }
    }

    pub const fn data_type_reference(
        name: &'static str,
        sid: SpectaID,
        generics: Vec<DataType>,
    ) -> DataTypeReference {
        DataTypeReference {
            name: Cow::Borrowed(name),
            sid,
            generics,
        }
    }

    pub const fn tuple(fields: Vec<DataType>) -> TupleType {
        TupleType { fields }
    }

    pub const fn impl_location(loc: &'static str) -> ImplLocation {
        ImplLocation(loc)
    }

    /// Compute an SID hash for a given type.
    /// This will produce a type hash from the arguments.
    /// This hashing function was derived from https://stackoverflow.com/a/71464396
    pub const fn sid(type_name: &'static str, type_identifier: &'static str) -> SpectaID {
        let mut hash = 0xcbf29ce484222325;
        let prime = 0x00000100000001B3;

        let mut bytes = type_name.as_bytes();
        let mut i = 0;

        while i < bytes.len() {
            hash ^= bytes[i] as u64;
            hash = hash.wrapping_mul(prime);
            i += 1;
        }

        bytes = type_identifier.as_bytes();
        i = 0;
        while i < bytes.len() {
            hash ^= bytes[i] as u64;
            hash = hash.wrapping_mul(prime);
            i += 1;
        }

        SpectaID { type_name, hash }
    }

    // TODO: Macros take in `&'static str` and then `Cow` inside here -> Do for all of them!

    #[cfg(feature = "functions")]
    pub fn function(export_fn: crate::ExportFn) -> crate::Function {
        crate::Function { export_fn }
    }

    #[cfg(feature = "functions")]
    pub fn function_type(
        asyncness: bool,
        name: &'static str,
        args: Vec<(&'static str, Option<DataType>)>,
        result: DataType,
        docs: Vec<Cow<'static, str>>,
    ) -> FunctionType {
        FunctionType {
            asyncness,
            name: Cow::Borrowed(name),
            args: args
                .into_iter()
                .filter_map(|(name, ty)| ty.map(|ty| (Cow::Borrowed(name), ty)))
                .collect(),
            result,
            docs,
        }
    }
}

/// Internal functions used by the macros for the export feature.
#[cfg(feature = "export")]
pub mod export {
    use std::sync::PoisonError;

    use crate::{export::TYPES, DefOpts, Type};

    // Called within ctor functions to register a type.
    pub fn register_ty<T: Type>() {
        let type_map = &mut *TYPES.write().unwrap_or_else(PoisonError::into_inner);

        // We call this for it's side effects on the `type_map`
        T::reference(
            DefOpts {
                parent_inline: false,
                type_map,
            },
            &[],
        );
    }
}
