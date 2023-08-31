//! This module contains functions that are public for the sole reason of the macros.
//!
//! They will not be documented and may go through breaking changes without a major version bump!
//!
//! DO NOT USE THEM! You have been warned!

#[cfg(feature = "export")]
pub use ctor;

#[cfg(feature = "functions")]
pub use specta_macros::fn_datatype;

/// Functions used to construct `crate::datatype` types (they have private fields so can't be constructed directly).
/// We intentionally keep their fields private so we can modify them without a major version bump.
pub mod construct {
    use std::borrow::Cow;

    use crate::{datatype::*, ImplLocation, SpectaID};

    pub const fn unit_struct() -> StructType {
        StructType::Unit
    }

    // TODO: By taking in `DataType` how does `flatten` and `inline` work
    pub const fn unnamed_struct(generics: Vec<GenericType>, fields: Vec<DataType>) -> StructType {
        StructType::Unnamed(TupleType { generics, fields })
    }

    pub const fn named_struct(
        generics: Vec<GenericType>,
        fields: Vec<StructField>,
        tag: Option<Cow<'static, str>>,
    ) -> StructType {
        StructType::Named(StructNamedFields {
            generics,
            fields,
            tag,
        })
    }

    pub const fn struct_field(
        key: Cow<'static, str>,
        optional: bool,
        flatten: bool,
        ty: DataType,
    ) -> StructField {
        StructField {
            key,
            optional,
            flatten,
            ty,
        }
    }

    pub const fn named_data_type(
        name: Cow<'static, str>,
        comments: Vec<Cow<'static, str>>,
        deprecated: Option<Cow<'static, str>>,
        sid: SpectaID,
        impl_location: ImplLocation,
        export: Option<bool>,
        item: NamedDataTypeItem,
    ) -> NamedDataType {
        NamedDataType {
            name,
            comments,
            deprecated,
            ext: Some(NamedDataTypeExt {
                sid,
                impl_location,
                export,
            }),
            item,
        }
    }

    pub const fn data_type_reference(
        name: Cow<'static, str>,
        sid: SpectaID,
        generics: Vec<DataType>,
    ) -> DataTypeReference {
        DataTypeReference {
            name,
            sid,
            generics,
        }
    }

    pub const fn untagged_enum(variants: Vec<EnumVariant>, generics: Vec<GenericType>) -> EnumType {
        EnumType::Untagged(UntaggedEnum { variants, generics })
    }

    pub const fn tagged_enum(
        variants: Vec<(Cow<'static, str>, EnumVariant)>,
        generics: Vec<GenericType>,
        repr: EnumRepr,
    ) -> EnumType {
        EnumType::Tagged(TaggedEnum {
            variants,
            generics,
            repr,
        })
    }

    pub const fn tuple_type(generics: Vec<GenericType>, fields: Vec<DataType>) -> TupleType {
        TupleType { fields, generics }
    }
}
