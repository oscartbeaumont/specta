//! This module contains functions that are public for the sole reason of the macros.
//!
//! They will not be documented and may go through breaking changes without a major version bump!
//!
//! DO NOT USE THEM! You have been warned!

use std::borrow::Cow;

#[cfg(feature = "export")]
pub use ctor;

#[cfg(feature = "functions")]
pub use specta_macros::fn_datatype;

mod intersperse;
pub use intersperse::*;

use crate::{DataType, Field, SpectaID, Type, TypeMap};

/// Functions used to construct `crate::datatype` types (they have private fields so can't be constructed directly).
/// We intentionally keep their fields private so we can modify them without a major version bump.
/// As this module is `#[doc(hidden)]` we allowed to make breaking changes within a minor version as it's only used by the macros.
pub mod construct {
    use std::borrow::Cow;

    use crate::{datatype::*, ImplLocation, ModulePath, SpectaID};

    pub const fn field(
        optional: bool,
        flatten: bool,
        deprecated: Option<DeprecatedType>,
        docs: Cow<'static, str>,
        ty: Option<DataType>,
    ) -> Field {
        Field {
            optional,
            flatten,
            deprecated,
            docs,
            ty,
        }
    }

    pub const fn r#struct(
        name: Cow<'static, str>,
        sid: Option<SpectaID>,
        generics: Vec<GenericType>,
        fields: StructFields,
    ) -> StructType {
        StructType {
            name,
            sid,
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
        name: Cow<'static, str>,
        sid: SpectaID,
        repr: EnumRepr,
        skip_bigint_checks: bool,
        generics: Vec<GenericType>,
        variants: Vec<(Cow<'static, str>, EnumVariant)>,
    ) -> EnumType {
        EnumType {
            name,
            sid: Some(sid),
            repr,
            skip_bigint_checks,
            generics,
            variants,
        }
    }

    pub const fn enum_variant(
        skip: bool,
        deprecated: Option<DeprecatedType>,
        docs: Cow<'static, str>,
        inner: EnumVariants,
    ) -> EnumVariant {
        EnumVariant {
            skip,
            docs,
            deprecated,
            inner,
        }
    }

    pub const fn enum_variant_unit() -> EnumVariants {
        EnumVariants::Unit
    }

    pub const fn enum_variant_unnamed(fields: Vec<Field>) -> EnumVariants {
        EnumVariants::Unnamed(UnnamedFields { fields })
    }

    pub const fn enum_variant_named(
        fields: Vec<(Cow<'static, str>, Field)>,
        tag: Option<Cow<'static, str>>,
    ) -> EnumVariants {
        EnumVariants::Named(NamedFields { fields, tag })
    }

    pub const fn named_data_type(
        name: Cow<'static, str>,
        docs: Cow<'static, str>,
        deprecated: Option<DeprecatedType>,
        sid: SpectaID,
        impl_location: ImplLocation,
        module_path: ModulePath,
        inner: DataType,
    ) -> NamedDataType {
        NamedDataType {
            name,
            docs,
            deprecated,
            ext: Some(NamedDataTypeExt {
                sid,
                impl_location,
                module_path,
            }),
            inner,
        }
    }

    pub const fn data_type_reference(
        name: Cow<'static, str>,
        sid: SpectaID,
        generics: Vec<(GenericType, DataType)>,
    ) -> DataTypeReference {
        DataTypeReference {
            name,
            sid,
            generics,
        }
    }

    pub const fn tuple(fields: Vec<DataType>) -> TupleType {
        TupleType { elements: fields }
    }

    pub const fn generic_data_type(name: &'static str) -> DataType {
        DataType::Generic(GenericType(Cow::Borrowed(name)))
    }

    pub const fn impl_location(loc: &'static str) -> ImplLocation {
        ImplLocation(loc)
    }

    pub const fn module_path(path: &'static str) -> ModulePath {
        ModulePath(path)
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
}

pub type NonSkipField<'a> = (&'a Field, &'a DataType);

pub fn skip_fields<'a>(
    fields: impl IntoIterator<Item = &'a Field>,
) -> impl Iterator<Item = NonSkipField<'a>> {
    fields
        .into_iter()
        .filter_map(|field| field.ty().map(|ty| (field, ty)))
}

pub fn skip_fields_named<'a>(
    fields: impl IntoIterator<Item = &'a (Cow<'static, str>, Field)>,
) -> impl Iterator<Item = (&'a Cow<'static, str>, NonSkipField<'a>)> {
    fields
        .into_iter()
        .filter_map(|(name, field)| field.ty().map(|ty| (name, (field, ty))))
}

#[track_caller]
pub fn flatten<T: Type>(sid: SpectaID, type_map: &mut TypeMap, generics: &[DataType]) -> DataType {
    type_map.flatten_stack.push(sid);

    #[allow(clippy::panic)]
    if type_map.flatten_stack.len() > 25 {
        // TODO: Handle this error without panicking
        panic!("Type recursion limit exceeded!");
    }

    let ty = T::inline(type_map, generics);

    type_map.flatten_stack.pop();

    ty
}
