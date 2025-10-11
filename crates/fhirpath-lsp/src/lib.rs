//! FHIRPath Language Server Protocol library
//!
//! This crate provides LSP functionality for FHIRPath expressions.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod cache;
pub mod config;
pub mod directives;
pub mod document;
pub mod features;
pub mod server;
pub mod utils;
pub mod watcher;

// Re-exports
pub use config::Config;
pub use document::FhirPathDocument;
pub use server::FhirPathLanguageServer;
