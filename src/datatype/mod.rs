use std::{
    borrow::{Borrow, Cow},
    fmt::Display,
};

mod r#enum;
mod fields;
mod list;
mod literal;
mod named;
mod primitive;
pub mod reference;
mod r#struct;
mod tuple;

pub use fields::*;
pub use list::*;
pub use literal::*;
pub use named::*;
pub use primitive::*;
pub use r#enum::*;
pub use r#struct::*;
pub use tuple::*;

use crate::SpectaID;

/// Runtime type-erased representation of a Rust type.
///
/// A language exporter takes this general format and converts it into a language specific syntax.
#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Nullable(Box<DataType>),
    Primitive(PrimitiveType),
    Literal(LiteralType),
    List(List),
    Map(Box<(DataType, DataType)>),
    Enum(EnumType),
    Tuple(TupleType),
    Reference(DataTypeReference),
    Generic(GenericType),

    // TODO: Should we keep this - https://github.com/oscartbeaumont/specta/issues/192
    Result(Box<(DataType, DataType)>),

    // TODO: Remove these
    Any,
    Unknown,
    Struct(StructType),
    // Named(NamedDataType)
}

impl DataType {
    pub fn generics(&self) -> Option<&Vec<GenericType>> {
        match self {
            Self::Struct(s) => Some(s.generics()),
            Self::Enum(e) => Some(e.generics()),
            _ => None,
        }
    }

    /// convert a [`DataType`] to a named [`NamedDataType`].
    ///
    /// This is perfect if you want to export a type as a named type.
    pub fn to_named(self, name: impl Into<Cow<'static, str>>) -> NamedDataType {
        NamedDataType {
            name: name.into(),
            docs: Cow::Borrowed(""),
            deprecated: None,
            ext: None,
            inner: self,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DeprecatedType {
    /// A type that has been deprecated without a message.
    ///
    /// Eg. `#[deprecated]`
    Deprecated,
    /// A type that has been deprecated with a message and an optional `since` version.
    ///
    /// Eg. `#[deprecated = "Use something else"]` or `#[deprecated(since = "1.0.0", message = "Use something else")]`
    DeprecatedWithSince {
        since: Option<Cow<'static, str>>,
        note: Cow<'static, str>,
    },
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
pub struct DataTypeReference {
    pub(crate) name: Cow<'static, str>,
    pub(crate) sid: SpectaID,
    pub(crate) generics: Vec<(GenericType, DataType)>,
}

impl DataTypeReference {
    pub fn name(&self) -> &Cow<'static, str> {
        &self.name
    }

    pub fn sid(&self) -> SpectaID {
        self.sid
    }

    pub fn generics(&self) -> &Vec<(GenericType, DataType)> {
        &self.generics
    }
}

/// A generic ("placeholder") argument to a Specta-enabled type.
///
/// A generic does not hold a specific type instead it acts as a slot where a type can be provided when referencing this type.
///
/// A `GenericType` holds the identifier of the generic. Eg. Given a generic type `struct A<T>(T);` the generics will be represented as `vec![GenericType("A".into())]`
#[derive(Debug, Clone, PartialEq)]
pub struct GenericType(pub(crate) Cow<'static, str>);

impl Display for GenericType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Borrow<str> for GenericType {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl From<Cow<'static, str>> for GenericType {
    fn from(value: Cow<'static, str>) -> Self {
        Self(value)
    }
}

impl From<GenericType> for DataType {
    fn from(t: GenericType) -> Self {
        Self::Generic(t)
    }
}

impl<T: Into<DataType> + 'static> From<Vec<T>> for DataType {
    fn from(t: Vec<T>) -> Self {
        DataType::Enum(EnumType {
            name: "Vec".into(),
            sid: None,
            repr: EnumRepr::Untagged,
            skip_bigint_checks: false,
            variants: t
                .into_iter()
                .map(|t| {
                    let ty: DataType = t.into();
                    (
                        match &ty {
                            DataType::Struct(s) => s.name.clone(),
                            DataType::Enum(e) => e.name().clone(),
                            // TODO: This is probs gonna cause problems so we should try and remove the need for this entire impl block if we can.
                            _ => "".into(),
                        },
                        EnumVariant {
                            skip: false,
                            docs: Cow::Borrowed(""),
                            deprecated: None,
                            inner: EnumVariants::Unnamed(UnnamedFields {
                                fields: vec![Field {
                                    optional: false,
                                    flatten: false,
                                    deprecated: None,
                                    docs: Cow::Borrowed(""),
                                    ty: Some(ty),
                                }],
                            }),
                        },
                    )
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

impl<'a> From<Cow<'a, str>> for DataType {
    fn from(t: Cow<'a, str>) -> Self {
        LiteralType::String(t.to_string()).into()
    }
}
