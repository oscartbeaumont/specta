use crate::*;

#[macro_use]
mod macros;
mod impls;
mod specta_id;

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
    fn inline(opts: DefOpts, generics: &[DataType]) -> DataType;

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
    fn definition(opts: DefOpts) -> DataType {
        Self::inline(
            opts,
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
    fn reference(opts: DefOpts, generics: &[DataType]) -> Reference {
        reference::inline::<Self>(opts, generics)
    }
}

/// NamedType represents a type that can be converted into [NamedDataType].
/// This will be implemented for all types with the [Type] derive macro.
pub trait NamedType: Type {
    const SID: SpectaID;
    const IMPL_LOCATION: ImplLocation;

    /// this is equivalent to [Type::inline] but returns a [NamedDataType] instead.
    fn named_data_type(opts: DefOpts, generics: &[DataType]) -> NamedDataType;

    /// this is equivalent to [Type::definition] but returns a [NamedDataType] instead.
    fn definition_named_data_type(opts: DefOpts) -> NamedDataType {
        Self::named_data_type(
            opts,
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
    pub struct Reference {
        pub inner: DataType,
        pub(crate) _priv: (),
    }

    pub fn inline<T: Type + ?Sized>(opts: DefOpts, generics: &[DataType]) -> Reference {
        Reference {
            inner: T::inline(opts, generics),
            _priv: (),
        }
    }

    pub fn reference<T: NamedType>(
        opts: DefOpts,
        generics: &[DataType],
        reference: DataTypeReference,
    ) -> Reference {
        if opts.type_map.get(&T::SID).is_none() {
            // It's important we don't put `None` into the map here. By putting a *real* value we ensure that we don't stack overflow for recursive types when calling `named_data_type`.
            opts.type_map.entry(T::SID).or_insert(Some(NamedDataType {
                name: "placeholder".into(),
                comments: vec![],
                deprecated: None,
                ext: None,
                inner: DataType::Any,
            }));

            let dt = T::named_data_type(
                DefOpts {
                    parent_inline: true,
                    type_map: opts.type_map,
                },
                generics,
            );
            opts.type_map.insert(T::SID, Some(dt));
        }

        Reference {
            inner: DataType::Reference(reference),
            _priv: (),
        }
    }
}

/// A marker trait for compile-time validation of which types can be flattened.
pub trait Flatten: Type {}
