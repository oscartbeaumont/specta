use std::fmt::Debug;

use crate::{DataType, Type, TypeMap};

/// Easily convert a non-Specta type into a Specta compatible type.
/// This will be typed as `any` in Typescript.
///
/// WARNING: When used with `Option<Any<T>>`, Typescript will not prompt you about nullability checks as `any | null` is coalesced to `any` in Typescript.
///
/// # Examples
///
/// This can be used as a type override.
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
///
/// Or it can be used as a wrapper type.
/// ```rust
/// use serde::Serialize;
/// use specta::{Type, Any};
///
/// #[derive(Serialize, Type)]
/// pub struct Demo {
///     pub field: Any<String>,
/// }
/// ```
pub struct Any<T = ()>(T);

impl<T> Type for Any<T> {
    fn inline(_: &mut TypeMap) -> DataType {
        DataType::Any
    }
}

impl<T: Debug> Debug for Any<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Any").field(&self.0).finish()
    }
}

impl<T: Clone> Clone for Any<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[cfg(feature = "serde")]
impl<T: serde::Serialize> serde::Serialize for Any<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        T::serialize(&self.0, serializer)
    }
}

/// Easily convert a non-Specta type into a Specta compatible type.
/// This will be typed as `unknown` in Typescript.
///
/// # Examples
///
/// This can be used as a type override.
/// ```rust
/// use serde::Serialize;
/// use specta::{Type, Unknown};
///
/// #[derive(Serialize, Type)]
/// pub struct Demo {
///     #[specta(type = Unknown)]
///     pub field: String,
/// }
/// ```
///
/// Or it can be used as a wrapper type.
/// ```rust
/// use serde::Serialize;
/// use specta::{Type, Unknown};
///
/// #[derive(Serialize, Type)]
/// pub struct Demo {
///     pub field: Unknown<String>,
/// }
/// ```
pub struct Unknown<T = ()>(T);

impl<T> Type for Unknown<T> {
    fn inline(_: &mut TypeMap) -> DataType {
        DataType::Unknown
    }
}

impl<T: Debug> Debug for Unknown<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Any").field(&self.0).finish()
    }
}

impl<T: Clone> Clone for Unknown<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[cfg(feature = "serde")]
impl<T: serde::Serialize> serde::Serialize for Unknown<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        T::serialize(&self.0, serializer)
    }
}
