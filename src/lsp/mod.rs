//! Language Server Protocol (LSP) implementation for FHIRPath
//!
//! This module provides a high-performance, async-first LSP server for FHIRPath expressions,
//! featuring intelligent code completion, diagnostics, hover information, and navigation.
//! 
//! Built on top of the async analyzer framework with full integration to the ModelProvider
//! and FunctionRegistry for accurate type information and context-aware suggestions.

#[cfg(feature = "lsp")]
pub mod server;

#[cfg(feature = "lsp")]
pub mod completion;

#[cfg(feature = "lsp")]
pub mod diagnostics;

#[cfg(feature = "lsp")]
pub mod hover;

#[cfg(feature = "lsp")]
pub mod navigation;

#[cfg(feature = "lsp")]
pub mod document_manager;

// Re-export main types when LSP feature is enabled
#[cfg(feature = "lsp")]
pub use server::{FhirPathLanguageServer, LspConfig};

#[cfg(feature = "lsp")]
pub use completion::CompletionProvider;

#[cfg(feature = "lsp")]
pub use diagnostics::DiagnosticPublisher;

#[cfg(feature = "lsp")]
pub use hover::HoverProvider;

#[cfg(feature = "lsp")]
pub use navigation::NavigationProvider;

#[cfg(feature = "lsp")]
pub use document_manager::DocumentManager;