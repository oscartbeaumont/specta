use thiserror::Error;

use crate::*;

#[macro_use]
mod macros;
mod impls;
mod post_process;

pub use post_process::*;

/// The category a type falls under.
/// Determines how references are generated for a given type.
pub enum TypeCategory {
    /// No references should be created, instead just copies the inline representation of the type.
    Inline(DataType),
    /// The type should be properly referenced and stored in the type map to be defined outside of
    /// where it is referenced.
    Reference(DataTypeReference),
}

/// Type exporting errors.
#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[allow(missing_docs)]
pub enum ExportError {
    #[error("Atemmpted to export type defined at '{}' but encountered error: {1}", .0.as_str())]
    InvalidType(ImplLocation, &'static str),
}

/// Provides runtime type information that can be fed into a language exporter to generate a type definition in another language.
/// Avoid implementing this trait yourself where possible and use the [`Type`](derive@crate::Type) macro instead.
pub trait Type {
    const MODULE_PATH: &'static str;

    /// Returns the inline definition of a type with generics substituted for those provided.
    /// This function defines the base structure of every type, and is used in both
    /// [`definition`](crate::Type::definition) and [`reference`](crate::Type::definition)
    ///
    /// Implemented internally or via the [`Type`](derive@crate::Type) macro
    fn inline(opts: DefOpts, generics: &[DataType]) -> Result<DataType, ExportError>;

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
    fn definition(opts: DefOpts) -> Result<DataType, ExportError> {
        Self::inline(
            opts,
            &Self::definition_generics()
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>(),
        )
    }

    /// Defines which category this type falls into, determining how references to it are created.
    /// See [`TypeCategory`] for more info.
    ///
    /// Implemented internally or via the [`Type`](derive@crate::Type) macro
    fn category_impl(opts: DefOpts, generics: &[DataType]) -> Result<TypeCategory, ExportError> {
        Self::inline(opts, generics).map(TypeCategory::Inline)
    }

    /// Generates a datatype corresponding to a reference to this type,
    /// as determined by its category. Getting a reference to a type implies that
    /// it should belong in the type map (since it has to be referenced from somewhere),
    /// so the output of [`definition`](crate::Type::definition) will be put into the type map.
    ///
    /// Implemented internally
    fn reference(opts: DefOpts, generics: &[DataType]) -> Result<DataType, ExportError> {
        let category = Self::category_impl(
            DefOpts {
                parent_inline: opts.parent_inline,
                type_map: opts.type_map,
            },
            generics,
        )?;

        Ok(match category {
            TypeCategory::Inline(inline) => inline,
            TypeCategory::Reference(def) => {
                if opts.type_map.get(&def.sid).is_none() {
                    opts.type_map.entry(def.sid).or_default();

                    let definition = Self::definition(DefOpts {
                        parent_inline: opts.parent_inline,
                        type_map: opts.type_map,
                    })?;

                    // TODO: It would be nice if we removed the `TypeCategory` and used the `NamedType` trait or something so this unreachable isn't needed.
                    let definition = match definition {
                        DataType::Named(definition) => definition,
                        _ => unreachable!(),
                    };

                    opts.type_map.insert(def.sid, Some(definition));
                }

                DataType::Reference(def)
            }
        })
    }
}

/// NamedType represents a type that can be converted into [NamedDataType].
/// This will be all types created by the derive macro.
pub trait NamedType: Type {
    /// this is equivalent to [Type::inline] but returns a [NamedDataType] instead.
    /// This is a compile-time guaranteed alternative to extracting the `DataType::Named` variant.
    fn named_data_type(opts: DefOpts, generics: &[DataType]) -> Result<NamedDataType, ExportError>;

    /// this is equivalent to [Type::definition] but returns a [NamedDataType] instead.
    /// This is a compile-time guaranteed alternative to extracting the `DataType::Named` variant.
    fn definition_named_data_type(opts: DefOpts) -> Result<NamedDataType, ExportError> {
        Self::named_data_type(
            opts,
            &Self::definition_generics()
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>(),
        )
    }
}

/// A marker trait for compile-time validation of which types can be flattened.
pub trait Flatten: Type {}

/// The Specta ID for the type. Holds for the given properties `T::SID == T::SID`, `T::SID != S::SID` and `Type<T>::SID == Type<S>::SID` (unlike std::any::TypeId)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[doc(hidden)]
pub struct TypeSid(u64);

/// Compute an SID hash for a given type.
/// This hash function comes from https://stackoverflow.com/a/71464396
/// You should NOT use this directly. Rely on `sid!();` instead.
#[doc(hidden)]
pub const fn internal_sid_hash(
    module_path: &'static str,
    file: &'static str,
    // This is required for a unique hash because all impls generated by a `macro_rules!` will have an identical `module_path` and `file` value.
    type_name: &'static str,
) -> TypeSid {
    let mut hash = 0xcbf29ce484222325;
    let prime = 0x00000100000001B3;

    let mut bytes = module_path.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(prime);
        i += 1;
    }

    bytes = file.as_bytes();
    i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(prime);
        i += 1;
    }

    bytes = type_name.as_bytes();
    i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(prime);
        i += 1;
    }

    TypeSid(hash)
}

/// Compute an SID hash for a given type.
#[macro_export]
#[doc(hidden)]
macro_rules! sid {
    ($name:expr) => {
        $crate::sid!($name, $crate::impl_location!().as_str())
    };
     // Using `$crate_path:path` here does not work because: https://github.com/rust-lang/rust/issues/48067
    (@with_specta_path; $name:expr; $first:ident$(::$rest:ident)*) => {{
        use $first$(::$rest)*::{internal_sid_hash, impl_location};

        internal_sid_hash(
            module_path!(),
            impl_location!().as_str(),
            $name,
        )
    }};
}

/// The location of the impl block for a given type. This is used for error reporting.
/// The content of it is transparent and should be generated by the `impl_location!` macro.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[doc(hidden)]
pub struct ImplLocation(&'static str);

impl ImplLocation {
    #[doc(hidden)]
    pub const fn internal_new(s: &'static str) -> Self {
        Self(s)
    }

    /// Get the location as a string
    pub const fn as_str(&self) -> &'static str {
        self.0
    }
}

/// Compute the location for an impl block
#[macro_export]
#[doc(hidden)]
macro_rules! impl_location {
    () => {
        $crate::ImplLocation::internal_new(concat!(file!(), ":", line!(), ":", column!()))
    };
    // Using `$crate_path:path` here does not work because: https://github.com/rust-lang/rust/issues/48067
    (@with_specta_path; $first:ident$(::$rest:ident)*) => {
        $first$(::$rest)*::ImplLocation::internal_new(concat!(file!(), ":", line!(), ":", column!()))
    };
}
