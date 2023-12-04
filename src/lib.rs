#![doc = include_str!("./docs.md")]
#![forbid(unsafe_code)]
#![warn(clippy::all, clippy::unwrap_used, clippy::panic)] // TODO: missing_docs
#![allow(clippy::module_inception)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://github.com/oscartbeaumont/specta/raw/main/.github/logo-128.png",
    html_favicon_url = "https://github.com/oscartbeaumont/specta/raw/main/.github/logo-128.png"
)]

#[doc(hidden)]
pub mod internal;

/// Types related to working with [`DataType`](crate::DataType). Exposed for advanced users.
pub mod datatype;
/// Provides the global type store and a method to export them to other languages.
#[cfg(feature = "export")]
#[cfg_attr(docsrs, doc(cfg(feature = "export")))]
pub mod export;
/// Support for exporting Rust functions.
#[cfg(feature = "functions")]
#[cfg_attr(docsrs, doc(cfg(feature = "functions")))]
pub mod functions;
mod lang;
mod registry;
mod selection;
mod serde;
mod static_types;
/// Contains [`Type`] and everything related to it, including implementations and helper macros
pub mod r#type;

pub use crate::serde::*;
#[doc(hidden)]
pub use datatype::*;
pub use lang::*;
pub use r#type::*;
pub use registry::Registry;
pub use selection::*;
pub use static_types::*;

/// Implements [`Type`] for a given struct or enum.
///
/// ## Example
///
/// ```rust
/// use specta::Type;
///
/// // Use it on structs
/// #[derive(Type)]
/// pub struct MyCustomStruct {
///     pub name: String,
/// }
///
/// #[derive(Type)]
/// pub struct MyCustomStruct2(String, i32, bool);
///
/// // Use it on enums
/// #[derive(Type)]
/// pub enum MyCustomType {
///     VariantOne,
///     VariantTwo(String, i32),
///     VariantThree { name: String, age: i32 },
/// }
/// ```
pub use specta_macros::Type;

/// Generates an implementation to help converting a type into into [`DataType`].
///
/// **This is an advanced feature and should probably be limited to usage in libraries built on top of Specta.**
///
/// This differs from [`Type`] in that you can use other [`DataType`] values
/// at runtime inside the targeted type, providing an easy way to construct types at
/// runtime from other types which are known statically via [`Type`].
///
/// Along with inner data types such as [`StructType`] and [`EnumType`], some builtin types
/// can easily be convert to a [`DataType`]:
/// - [`Vec`] will become [`DataType::Enum`]
/// - [`Option`] will become the value it contains or [`LiteralType::None`] if it is [`None`]
/// - [`String`] and [`&str`] will become [`LiteralType::String`]
///
/// # Example
/// ```rust
/// use specta::{datatype::LiteralType, ts, DataType, DataTypeFrom, StructType, TupleType};
///
/// #[derive(Clone, DataTypeFrom)]
/// pub struct MyEnum(pub Vec<DataType>);
///
/// #[derive(Clone, DataTypeFrom)]
/// pub struct MyObject {
///     a: Vec<DataType>,
/// }
///
/// //
/// // Enum
/// //
///
/// let val: DataType = MyEnum(vec![
///     LiteralType::String("A".to_string()).into(),
///     LiteralType::String("B".to_string()).into(),
/// ]).into();
///
/// let anon = ts::datatype(&Default::default(), &val, &Default::default()).unwrap();
/// assert_eq!(anon, "\"A\" | \"B\"");
///
/// let named = val.to_named("MyEnum");
/// let named_export = ts::export_named_datatype(&Default::default(), &named, &Default::default()).unwrap();
/// assert_eq!(named_export, "export type MyEnum = \"A\" | \"B\"");
///
/// //
/// // Object
/// //
///
/// let val: StructType = MyObject {
///     a: vec![
///         LiteralType::String("A".to_string()).into(),
///         LiteralType::String("B".to_string()).into(),
///     ],
/// }
/// .into();
///
/// let anon = val.clone().to_anonymous();
/// let anon = ts::datatype(&Default::default(), &anon, &Default::default()).unwrap();
/// assert_eq!(anon, "{ a: \"A\" | \"B\" }");
///
/// let named = val.to_named("MyObject");
/// let named_export = ts::export_named_datatype(&Default::default(), &named, &Default::default()).unwrap();
/// assert_eq!(named_export, "export type MyObject = { a: \"A\" | \"B\" }");
/// ```
pub use specta_macros::DataTypeFrom;

/// Prepares a function to have its types extracted using [`fn_datatype`]
///
/// ## Example
///
/// ```rust
/// #[specta::specta]
/// fn my_function(arg1: i32, arg2: bool) -> &'static str {
///     "Hello World"
/// }
/// ```
#[cfg(feature = "functions")]
#[cfg_attr(docsrs, doc(cfg(feature = "functions")))]
pub use specta_macros::specta;

#[cfg(doctest)]
doc_comment::doctest!("../README.md");
