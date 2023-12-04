use std::borrow::Cow;

use crate::{DataType, DeprecatedType, ImplLocation, ModulePath, SpectaID};

/// A NamedDataTypeImpl includes extra information which is only available for [NamedDataType]'s that come from a real Rust type.
#[derive(Debug, Clone, PartialEq)]
pub struct NamedDataTypeExt {
    /// The Specta ID for the type. The value for this should come from the `sid!();` macro.
    pub(crate) sid: SpectaID,
    /// The code location where this type is implemented. Used for error reporting.
    pub(crate) impl_location: ImplLocation,
    /// The Rust module where the type was defined.
    pub(crate) module_path: ModulePath,
}

impl NamedDataTypeExt {
    pub fn sid(&self) -> &SpectaID {
        &self.sid
    }

    pub fn impl_location(&self) -> &ImplLocation {
        &self.impl_location
    }
}

/// A named type represents a non-primitive type capable of being exported as it's own named entity.
#[derive(Debug, Clone, PartialEq)]
pub struct NamedDataType {
    /// The name of the type
    pub(crate) name: Cow<'static, str>,
    /// Rust documentation comments on the type
    pub(crate) docs: Cow<'static, str>,
    /// The Rust deprecated comment if the type is deprecated.
    pub(crate) deprecated: Option<DeprecatedType>,
    /// Extra information that comes from a real Rust type (using the `Type` macro).
    /// This will be `None` when constructing [NamedDataType] using `StructType::to_named` or `TupleType::to_named` since those types do not correspond to actual Rust types.
    pub(crate) ext: Option<NamedDataTypeExt>,
    /// the actual type definition.
    // This field is public because we match on it in flattening code. // TODO: Review if this can be made private when reviewing the flattening logic/error handling
    pub inner: DataType,
}

impl NamedDataType {
    pub fn name(&self) -> &Cow<'static, str> {
        &self.name
    }

    pub fn docs(&self) -> &Cow<'static, str> {
        &self.docs
    }

    pub fn deprecated(&self) -> Option<&DeprecatedType> {
        self.deprecated.as_ref()
    }

    pub fn ext(&self) -> Option<&NamedDataTypeExt> {
        self.ext.as_ref()
    }
}
