use std::{
    borrow::{Borrow, Cow},
    collections::BTreeMap,
};

mod r#enum;
mod fields;
#[cfg(feature = "functions")]
mod function;
mod literal;
mod named;
mod primitive;
mod r#struct;
mod tuple;

pub use fields::*;
#[cfg(feature = "functions")]
#[cfg_attr(docsrs, doc(cfg(feature = "functions")))]
pub use function::*;
pub use literal::*;
pub use named::*;
pub use primitive::*;
pub use r#enum::*;
pub use r#struct::*;
pub use tuple::*;

use crate::SpectaID;

/// A map used to store the types "discovered" while exporting a type.
/// You can iterate over this to export all types which the type/s you exported references on.
///
/// [`None`] indicates that the entry is a placeholder. It was reference but we haven't reached it's definition yet.
pub type TypeMap = BTreeMap<SpectaID, Option<NamedDataType>>;

/// Arguments for [`Type::inline`](crate::Type::inline), [`Type::reference`](crate::Type::reference) and [`Type::definition`](crate::Type::definition).
pub struct DefOpts<'a> {
    /// is the parent type inlined?
    pub parent_inline: bool,
    /// a map of types which have been visited. This prevents stack overflows when a type references itself and also allows the caller to get a list of all types in the "schema".
    pub type_map: &'a mut TypeMap,
}

/// Runtime type-erased representation of a Rust type.
///
/// A language exporter takes this general format and converts it into a language specific syntax.
#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    // Always inlined
    Any,
    Primitive(PrimitiveType),
    Literal(LiteralType),
    /// Either a `Set` or a `Vec`
    List(Box<DataType>),
    Nullable(Box<DataType>),
    Map(Box<(DataType, DataType)>),
    // Anonymous Reference types
    Struct(StructType),
    Enum(EnumType),
    Tuple(TupleType),
    // Result
    Result(Box<(DataType, DataType)>),
    // A reference type that has already been defined
    Reference(DataTypeReference),
    Generic(GenericType),
}

impl DataType {
    pub fn generics(&self) -> Option<&Vec<GenericType>> {
        match self {
            Self::Struct(s) => Some(s.generics()),
            Self::Enum(e) => Some(e.generics()),
            _ => None,
        }
    }
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
    pub(crate) generics: Vec<DataType>,
}

impl DataTypeReference {
    pub fn name(&self) -> &Cow<'static, str> {
        &self.name
    }

    pub fn sid(&self) -> SpectaID {
        self.sid
    }

    pub fn generics(&self) -> &Vec<DataType> {
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
            repr: EnumRepr::Untagged,
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
                        EnumVariant::Unnamed(UnnamedFields {
                            fields: vec![Field {
                                optional: false,
                                flatten: false,
                                ty,
                            }],
                        }),
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
