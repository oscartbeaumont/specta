use core::fmt;
use std::borrow::Cow;

use thiserror::Error;

use crate::{advanced::SerdeError, ImplLocation};

use super::ExportPath;

/// Describes where an error occurred.
#[derive(Error, Debug, PartialEq)]
pub enum NamedLocation {
    Type,
    Field,
    Variant,
}

impl fmt::Display for NamedLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Type => write!(f, "type"),
            Self::Field => write!(f, "field"),
            Self::Variant => write!(f, "variant"),
        }
    }
}

/// The error type for the TypeScript exporter.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum TsExportError {
    #[error("Attempted to export '{0}' but Specta configuration forbids exporting BigInt types (i64, u64, i128, u128) because we don't know if your se/deserializer supports it. You can change this behavior by editing your `ExportConfiguration`!")]
    BigIntForbidden(ExportPath),
    #[error("Serde error: {0}")]
    Serde(#[from] SerdeError),
    // #[error("Attempted to export '{0}' but was unable to export a tagged type which is unnamed")]
    // UnableToTagUnnamedType(ExportPath),
    #[error("Attempted to export '{1}' but was unable to due to {0} name '{2}' conflicting with a reserved keyword in Typescript. Try renaming it or using `#[specta(rename = \"new name\")]`")]
    ForbiddenName(NamedLocation, ExportPath, &'static str),
    #[error("Attempted to export '{0}' with tagging but the type is not tagged.")]
    InvalidTagging(ExportPath),
    #[error("Unable to export type named '{0}' from locations '{:?}' '{:?}'", .1.as_str(), .2.as_str())]
    DuplicateTypeName(Cow<'static, str>, ImplLocation, ImplLocation),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to export '{0}' due to error: {1}")]
    Other(ExportPath, String),
}

// TODO: This `impl` is cringe
impl PartialEq for TsExportError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::BigIntForbidden(l0), Self::BigIntForbidden(r0)) => l0 == r0,
            (Self::Serde(l0), Self::Serde(r0)) => l0 == r0,
            // (Self::UnableToTagUnnamedType(l0), Self::UnableToTagUnnamedType(r0)) => l0 == r0,
            (Self::ForbiddenName(l0, l1, l2), Self::ForbiddenName(r0, r1, r2)) => {
                l0 == r0 && l1 == r1 && l2 == r2
            }
            (Self::InvalidTagging(l0), Self::InvalidTagging(r0)) => l0 == r0,
            (Self::DuplicateTypeName(l0, l1, l2), Self::DuplicateTypeName(r0, r1, r2)) => {
                l0 == r0 && l1 == r1 && l2 == r2
            }
            (Self::Io(l0), Self::Io(r0)) => l0.to_string() == r0.to_string(), // This is a bit hacky but it will be fine for usage in unit tests!
            (Self::Other(l0, l1), Self::Other(r0, r1)) => l0 == r0 && l1 == r1,
            _ => false,
        }
    }
}
