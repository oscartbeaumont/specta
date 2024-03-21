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

use crate::{DataType, DeprecatedType, Field, Generics, SpectaID, Type, TypeMap};

/// Functions used to construct `crate::datatype` types (they have private fields so can't be constructed directly).
/// We intentionally keep their fields private so we can modify them without a major version bump.
/// As this module is `#[doc(hidden)]` we allowed to make breaking changes within a minor version as it's only used by the macros.
pub mod construct {
    use std::borrow::Cow;

    use crate::{datatype::*, ImplLocation, SpectaID};

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
        inner: DataType,
    ) -> NamedDataType {
        NamedDataType {
            name,
            docs,
            deprecated,
            ext: Some(NamedDataTypeExt { sid, impl_location }),
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

    let ty = T::inline(type_map, Generics::Provided(generics));

    type_map.flatten_stack.pop();

    ty
}

#[cfg(feature = "functions")]
mod functions {
    use super::*;
    use crate::functions::{FunctionDataType, SpectaFunction};

    #[doc(hidden)]
    /// A helper for exporting a command to a [`CommandDataType`].
    /// You shouldn't use this directly and instead should use [`fn_datatype!`](crate::fn_datatype).
    pub fn get_fn_datatype<TMarker, T: SpectaFunction<TMarker>>(
        _: T,
        asyncness: bool,
        name: Cow<'static, str>,
        type_map: &mut TypeMap,
        fields: &[Cow<'static, str>],
        docs: Cow<'static, str>,
        deprecated: Option<DeprecatedType>,
        no_return_type: bool,
    ) -> FunctionDataType {
        T::to_datatype(
            asyncness,
            name,
            type_map,
            fields,
            docs,
            deprecated,
            no_return_type,
        )
    }
}
#[cfg(feature = "functions")]
pub use functions::*;

// This code is taken from `erased-serde` - https://github.com/dtolnay/erased-serde/blob/master/src/any.rs
#[allow(unsafe_code)]
pub(crate) mod type_id {
    use std::{any::TypeId, marker::PhantomData};

    trait NonStaticAny {
        fn get_type_id(&self) -> TypeId
        where
            Self: 'static;
    }

    impl<T: ?Sized> NonStaticAny for PhantomData<T> {
        fn get_type_id(&self) -> TypeId
        where
            Self: 'static,
        {
            TypeId::of::<T>()
        }
    }

    pub fn non_static_type_id<T: ?Sized>() -> TypeId {
        let non_static_thing = PhantomData::<T>;
        let thing = unsafe {
            std::mem::transmute::<&dyn NonStaticAny, &(dyn NonStaticAny + 'static)>(
                &non_static_thing,
            )
        };
        NonStaticAny::get_type_id(thing)
    }
}
