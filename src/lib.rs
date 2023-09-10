//! Easily export your Rust types to other languages
//!
//! Specta provides a system for type introspection and a set of language exporters which allow you to export your Rust types to other languages!
//!
//! **Currently we only support exporting to [TypeScript](https://www.typescriptlang.org) but work has begun on other languages.**
//!
//! ## Features
//!  - Export structs and enums to [Typescript](https://www.typescriptlang.org)
//!  - Get function types to use in libraries like [tauri-specta](https://github.com/oscartbeaumont/tauri-specta)
//!  - Supports wide range of common crates in Rust ecosystem
//!  - Supports type inference - can determine type of `fn demo() -> impl Type`.
//!
//! ## Ecosystem
//!
//! Specta can be used in your application either directly or through a library which simplifies the process of using it.
//!
//! - [rspc](https://github.com/oscartbeaumont/rspc) for easily building end-to-end typesafe APIs
//! - [tauri-specta](https://github.com/oscartbeaumont/tauri-specta) for typesafe Tauri commands
//!
//! ## Example
//! ```rust
//! use specta::{*, ts::*};
//!
//! #[derive(Type)]
//! pub struct MyCustomType {
//!    pub my_field: String,
//! }
//!
//! fn main() {
//!     assert_eq!(
//!         ts::export::<MyCustomType>(&ExportConfig::default()).unwrap(),
//!         "export type MyCustomType = { my_field: string }".to_string()
//!     );
//! }
//! ```
//!
//! ## Supported Libraries
//!
//! If you are using [Prisma Client Rust](https://prisma.brendonovich.dev) you can enable the `rspc` feature on it to allow for Specta support on types coming directly from your database. This includes support for the types created via a selection.
//!
//! ## Feature flags
#![doc = document_features::document_features!()]
//! ## Alternatives
//!
//! #### Why not ts-rs?
//!
//! [ts-rs](https://github.com/Aleph-Alpha/ts-rs) is a great library,
//! but it has a few limitations which became a problem when I was building [rspc](https://github.com/oscartbeaumont/rspc).
//! Namely it deals with types individually which means it is not possible to export a type and all of the other types it depends on.
//!
//! #### Why not Typeshare?
//! [Typeshare](https://github.com/1Password/typeshare) is also great, but its approach is fundamentally different.
//! While Specta uses traits and runtime information, Typeshare statically analyzes your Rust
//! files.
//! This results in a loss of information and lack of compatability with types from other crates.
//!
#![forbid(unsafe_code)]
#![warn(clippy::all, clippy::unwrap_used, clippy::panic)] // TODO: missing_docs
#![allow(clippy::module_inception)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[doc(hidden)]
pub mod internal;

pub mod advanced;
/// Types related to working with [`DataType`](crate::DataType). Exposed for advanced users.
pub mod datatype;
/// Provides the global type store and a method to export them to other languages.
#[cfg(feature = "export")]
#[cfg_attr(docsrs, doc(cfg(feature = "export")))]
pub mod export;
#[cfg(feature = "functions")]
mod function;
mod lang;
mod selection;
mod static_types;
/// Contains [`Type`] and everything related to it, including implementations and helper macros
pub mod r#type;

pub use datatype::*;
#[cfg(feature = "functions")]
#[cfg_attr(docsrs, doc(cfg(feature = "functions")))]
pub use function::*;
pub use lang::*;
pub use r#type::*;
pub use selection::*;
pub use serde::*;
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
/// let val: TupleType = MyEnum(vec![
///     LiteralType::String("A".to_string()).into(),
///     LiteralType::String("B".to_string()).into(),
/// ])
/// .into();
///
/// let anon = val.clone().to_anonymous();
/// let anon = ts::datatype(&Default::default(), &anon, &Default::default()).unwrap();
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
