//! Language Server Protocol support for FHIRPath expressions
//!
//! This module provides LSP capabilities for FHIRPath including:
//! - Autocompletion for properties, functions, keywords, and constants
//! - Diagnostics with error recovery
//! - Optional context API for enhanced features (ViewDefinition, SQL on FHIR)
//!
//! # Usage Modes
//!
//! ## Standalone Editor (VS Code, Neovim, etc.)
//! The LSP infers resource type from expression prefix:
//! ```text
//! Patient.name.given.first()  → Context: Patient
//! ```
//!
//! ## Enhanced Mode (ViewDefinition, SQL on FHIR)
//! Server can set context via `fhirpath/setContext` notification:
//! - Resource type from forEach expression
//! - External constants for completion

mod completion;
mod diagnostics;
mod document;
mod handlers;
mod server;

pub use completion::{CompletionContext, CompletionKind, CompletionProvider};
pub use diagnostics::DiagnosticProvider;
pub use document::{DocumentManager, DocumentState};
pub use handlers::LspHandlers;
pub use server::{
    ClearContext, FhirPathLspServer, ServerConfig, SetContext, Transport, run_stdio, run_websocket,
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Parameters for the `fhirpath/setContext` notification
///
/// Allows clients to provide explicit context for enhanced completion and diagnostics.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetContextParams {
    /// Resource type for the expression context (e.g., "Patient", "HumanName")
    /// When set, overrides context inference from expression prefix
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_type: Option<String>,

    /// External constants available in expressions
    /// These will appear in completion when user types `%`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constants: Option<HashMap<String, ConstantInfo>>,
}

/// Information about an external constant
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConstantInfo {
    /// Type name for the constant (e.g., "string", "Patient")
    pub type_name: String,

    /// Optional description shown in completion
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Current context for LSP operations
#[derive(Debug, Clone, Default)]
pub struct LspContext {
    /// Explicit resource type (from setContext)
    pub resource_type: Option<String>,

    /// External constants (from setContext)
    pub constants: HashMap<String, ConstantInfo>,
}

impl LspContext {
    /// Create a new empty context
    pub fn new() -> Self {
        Self::default()
    }

    /// Update context from SetContextParams
    pub fn update(&mut self, params: SetContextParams) {
        if let Some(rt) = params.resource_type {
            self.resource_type = Some(rt);
        }
        if let Some(constants) = params.constants {
            self.constants = constants;
        }
    }

    /// Clear the context
    pub fn clear(&mut self) {
        self.resource_type = None;
        self.constants.clear();
    }
}
