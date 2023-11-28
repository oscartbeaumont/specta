//! # Specta Implementations
//! A collection of Specta integrations for popular crates.
//!
//! TODO: List of supported features, why this exists, semver rules
#![forbid(unsafe_code)]
#![warn(clippy::all, clippy::unwrap_used, clippy::panic)] // TODO: missing_docs
#![allow(clippy::module_inception)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://github.com/oscartbeaumont/specta/raw/main/.github/logo-128.png",
    html_favicon_url = "https://github.com/oscartbeaumont/specta/raw/main/.github/logo-128.png"
)]

#[doc(hidden)]
mod internal;

#[cfg(feature = "testing")]
pub struct Testing();
