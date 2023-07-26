use std::{borrow::Cow, collections::BTreeMap};

mod r#enum;
mod literal;
mod object;
mod primitive;
mod tuple;

pub use literal::*;
pub use object::*;
pub use primitive::*;
pub use r#enum::*;
pub use tuple::*;

use crate::{ImplLocation, TypeSid};

/// A map used to store the types "discovered" while exporting a type.
/// You can iterate over this to export all types which the type/s you exported references on.
///
/// [`None`] indicates that the entry is a placeholder.
pub type TypeDefs = BTreeMap<TypeSid, Option<NamedDataType>>;

/// Arguments for [`Type::inline`](crate::Type::inline), [`Type::reference`](crate::Type::reference) and [`Type::definition`](crate::Type::definition).
pub struct DefOpts<'a> {
    /// is the parent type inlined?
    pub parent_inline: bool,
    /// a map of types which have been visited. This prevents stack overflows when a type references itself and also allows the caller to get a list of all types in the "schema".
    pub type_map: &'a mut TypeDefs,
}

/// Runtime type-erased representation of a Rust type.
///
/// A language exporter takes this general format and converts it into a language specific syntax.
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub enum DataType {
    // Always inlined
    Any,
    Primitive(PrimitiveType),
    Literal(LiteralType),
    List(Box<DataType>),
    Nullable(Box<DataType>),
    Record(Box<(DataType, DataType)>),
    // Named reference types
    Named(NamedDataType),
    // Anonymous Reference types
    Object(ObjectType),
    Enum(EnumType),
    Tuple(TupleType),
    // A reference type that has already been defined
    Reference(DataTypeReference),
    Generic(GenericType),
}

/// A named type represents a non-primitive type capable of being exported as it's own named entity.
#[derive(Debug, Clone, PartialEq)]
pub struct NamedDataType {
    /// The name of the type
    pub name: &'static str,
    /// The Specta ID for the type. The value for this should come from the `sid!();` macro.
    pub sid: Option<TypeSid>,
    /// The code location where this type is implemented. Used for error reporting.
    pub impl_location: Option<ImplLocation>,
    /// Rust documentation comments on the type
    pub comments: &'static [&'static str],
    /// DEPRECATED. This is not used and shouldn't be. Will be removed in Specta v2!
    pub export: Option<bool>,
    /// The Rust deprecated comment if the type is deprecated.
    pub deprecated: Option<&'static str>,
    /// the actual type definition.
    pub item: NamedDataTypeItem,
    /// A string containing the module path for the given struct. Used for namespace-based exporting
    pub module_path: Option<&'static str>,
}

impl From<NamedDataType> for DataType {
    fn from(t: NamedDataType) -> Self {
        Self::Named(t)
    }
}

/// The possible types for a [`NamedDataType`].
#[derive(Debug, Clone, PartialEq)]
pub enum NamedDataTypeItem {
    /// Represents an Rust struct with named fields
    Object(ObjectType),
    /// Represents an Rust enum
    Enum(EnumType),
    /// Represents an Rust struct with unnamed fields
    Tuple(TupleType),
}

/// A reference to a [`DataType`] that can be used before a type is resolved in order to
/// support recursive types without causing an infinite loop.
///
/// This works since a child type that references a parent type does not care about the
/// parent's fields, only really its name. Once all of the parent's fields have been
/// resolved will the parent's definition be placed in the type map.
///
// This doesn't account for flattening and inlining recursive types, however, which will
// require a more complex solution since it will require multiple processing stages.
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub struct DataTypeReference {
    pub name: &'static str,
    pub sid: TypeSid,
    pub generics: Vec<DataType>,
    pub module_path: &'static str,
}

/// A generic parameter to another type.
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub struct GenericType(pub &'static str);

impl From<GenericType> for DataType {
    fn from(t: GenericType) -> Self {
        Self::Generic(t)
    }
}

impl<T: Into<DataType> + 'static> From<Vec<T>> for DataType {
    fn from(t: Vec<T>) -> Self {
        DataType::Enum(EnumType::Untagged {
            variants: t
                .into_iter()
                .map(|t| {
                    EnumVariant::Unnamed(TupleType {
                        fields: vec![t.into()],
                        generics: vec![],
                    })
                })
                .collect(),
            generics: vec![],
        })
    }
}

impl<T: Into<DataType> + 'static> From<Option<T>> for DataType {
    fn from(t: Option<T>) -> Self {
        t.map(Into::into)
            .unwrap_or_else(|| LiteralType::None.into())
    }
}

impl<'a> From<&'a str> for DataType {
    fn from(t: &'a str) -> Self {
        LiteralType::String(t.to_string()).into()
    }
}

impl From<String> for DataType {
    fn from(t: String) -> Self {
        LiteralType::String(t).into()
    }
}

impl From<Cow<'static, str>> for DataType {
    fn from(t: Cow<'static, str>) -> Self {
        LiteralType::String(t.to_string()).into()
    }
}
