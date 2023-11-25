use crate::*;

#[macro_use]
mod macros;
mod impls;
mod map;
mod post_process;
mod specta_id;

pub use map::*;
pub use post_process::*;
pub use specta_id::*;

use self::reference::Reference;

/// Provides runtime type information that can be fed into a language exporter to generate a type definition in another language.
/// Avoid implementing this trait yourself where possible and use the [`Type`](derive@crate::Type) macro instead.
pub trait Type {
    /// Returns the inline definition of a type with generics substituted for those provided.
    /// This function defines the base structure of every type, and is used in both
    /// [`definition`](crate::Type::definition) and [`reference`](crate::Type::definition)
    ///
    /// Implemented internally or via the [`Type`](derive@crate::Type) macro
    fn inline(type_map: &mut TypeMap, generics: &[DataType]) -> DataType;

    /// Returns the type parameter generics of a given type.
    /// Will usually be empty except for custom types.
    ///
    /// Implemented internally or via the [`Type`](derive@crate::Type) macro
    fn definition_generics() -> Vec<GenericType> {
        vec![]
    }

    /// Small wrapper around [`inline`](crate::Type::inline) that provides
    /// [`definition_generics`](crate::Type::definition_generics)
    /// as the value for the `generics` arg.
    ///
    /// Implemented internally
    fn definition(type_map: &mut TypeMap) -> DataType {
        Self::inline(
            type_map,
            &Self::definition_generics()
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>(),
        )
    }

    /// Generates a datatype corresponding to a reference to this type,
    /// as determined by its category. Getting a reference to a type implies that
    /// it should belong in the type map (since it has to be referenced from somewhere),
    /// so the output of [`definition`](crate::Type::definition) will be put into the type map.
    fn reference(type_map: &mut TypeMap, generics: &[DataType]) -> Reference {
        reference::inline::<Self>(type_map, generics)
    }
}

/// NamedType represents a type that can be converted into [NamedDataType].
/// This will be implemented for all types with the [Type] derive macro.
pub trait NamedType: Type {
    const SID: SpectaID;
    const IMPL_LOCATION: ImplLocation;

    /// this is equivalent to [Type::inline] but returns a [NamedDataType] instead.
    fn named_data_type(type_map: &mut TypeMap, generics: &[DataType]) -> NamedDataType;

    /// this is equivalent to [Type::definition] but returns a [NamedDataType] instead.
    fn definition_named_data_type(type_map: &mut TypeMap) -> NamedDataType {
        Self::named_data_type(
            type_map,
            &Self::definition_generics()
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>(),
        )
    }
}

/// Helpers for generating [Type::reference] implementations.
pub mod reference {
    use super::*;

    /// A reference datatype.
    ///
    // This type exists to force the user to use [reference::inline] or [reference::reference] which provides some extra safety.
    #[non_exhaustive]
    pub struct Reference {
        pub inner: DataType,
    }

    pub fn inline<T: Type + ?Sized>(type_map: &mut TypeMap, generics: &[DataType]) -> Reference {
        Reference {
            inner: T::inline(type_map, generics),
        }
    }

    pub fn reference<T: NamedType>(
        type_map: &mut TypeMap,
        reference: DataTypeReference,
    ) -> Reference {
        if type_map.map.get(&T::SID).is_none() {
            type_map.map.entry(T::SID).or_insert(None);

            let dt = T::definition_named_data_type(type_map);
            type_map.map.insert(T::SID, Some(dt));
        }

        Reference {
            inner: DataType::Reference(reference),
        }
    }
}

/// A marker trait for compile-time validation of which types can be flattened.
pub trait Flatten: Type {}
