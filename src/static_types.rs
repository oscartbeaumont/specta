use crate::{DataType, DefOpts, ExportError, LiteralType, Type};

/// A type that is typed as `any` in TypeScript.
///
/// This can be use like the following:
/// ```rust
/// use serde::Serialize;
/// use specta::{Type, Any};
///
/// #[derive(Serialize, Type)]
/// pub struct Demo {
///     #[specta(type = Any)]
///     pub field: String,
/// }
/// ```
pub struct Any {}

impl Type for Any {
    fn inline(_: DefOpts, _: &[DataType]) -> Result<DataType, ExportError> {
        Ok(DataType::Any)
    }
}

pub struct True;

impl Type for True {
    fn inline(_: DefOpts, _: &[DataType]) -> Result<DataType, ExportError> {
        Ok(DataType::Literal(LiteralType::bool(true)))
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for True {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        true.serialize(serializer)
    }
}

pub struct False;

impl Type for False {
    fn inline(_: DefOpts, _: &[DataType]) -> Result<DataType, ExportError> {
        Ok(DataType::Literal(LiteralType::bool(false)))
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for False {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        false.serialize(serializer)
    }
}
