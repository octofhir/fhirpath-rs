//! Analysis metadata for AST nodes
//!
//! This module provides optional analysis metadata that can be attached to AST nodes
//! during analysis parsing mode. This allows us to keep the core AST structures
//! lightweight for fast parsing while providing rich semantic information when needed.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use octofhir_fhir_model::TypeInfo;
use crate::diagnostics::Diagnostic;

/// Analysis metadata that can be attached to AST nodes during analysis parsing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisMetadata {
    /// Type information resolved during parsing
    pub type_info: Option<TypeInfo>,
    /// Full path in evaluation context (e.g., "Patient.name[0].given[1]")
    pub path: Option<String>,
    /// Diagnostics specific to this node
    pub diagnostics: Vec<Diagnostic>,
    /// Additional metadata for analysis
    pub properties: HashMap<String, AnalysisProperty>,
}

/// Property values for analysis metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnalysisProperty {
    /// String property
    String(String),
    /// Boolean property
    Boolean(bool),
    /// Integer property
    Integer(i64),
    /// Validation result
    ValidationResult(ValidationResult),
}

/// Result of semantic validation during analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether validation passed
    pub is_valid: bool,
    /// Validation message
    pub message: Option<String>,
    /// Suggested fixes
    pub suggestions: Vec<String>,
}

impl AnalysisMetadata {
    /// Create new empty analysis metadata
    pub fn new() -> Self {
        Self {
            type_info: None,
            path: None,
            diagnostics: Vec::new(),
            properties: HashMap::new(),
        }
    }

    /// Create analysis metadata with type info
    pub fn with_type_info(type_info: TypeInfo) -> Self {
        Self {
            type_info: Some(type_info),
            path: None,
            diagnostics: Vec::new(),
            properties: HashMap::new(),
        }
    }

    /// Add a diagnostic to this metadata
    pub fn add_diagnostic(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Set a property value
    pub fn set_property(&mut self, key: String, value: AnalysisProperty) {
        self.properties.insert(key, value);
    }

    /// Get a property value
    pub fn get_property(&self, key: &str) -> Option<&AnalysisProperty> {
        self.properties.get(key)
    }

    /// Check if this node has any errors
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| matches!(d.severity, crate::diagnostics::DiagnosticSeverity::Error))
    }

    /// Check if this node has any warnings
    pub fn has_warnings(&self) -> bool {
        self.diagnostics.iter().any(|d| matches!(d.severity, crate::diagnostics::DiagnosticSeverity::Warning))
    }
}

impl Default for AnalysisMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Enhanced AST node that includes optional analysis metadata
pub trait AnalysisEnhanced {
    /// Get analysis metadata if available
    fn analysis_metadata(&self) -> Option<&AnalysisMetadata>;

    /// Set analysis metadata
    fn set_analysis_metadata(&mut self, metadata: AnalysisMetadata);

    /// Get type info from analysis metadata
    fn resolved_type_info(&self) -> Option<&TypeInfo> {
        self.analysis_metadata()?.type_info.as_ref()
    }

    /// Get path from analysis metadata
    fn resolved_path(&self) -> Option<&str> {
        self.analysis_metadata()?.path.as_deref()
    }

    /// Check if node has analysis errors
    fn has_analysis_errors(&self) -> bool {
        self.analysis_metadata()
            .map(|m| m.has_errors())
            .unwrap_or(false)
    }
}

/// Container for AST nodes with optional analysis metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalyzedNode<T> {
    /// The core AST node
    pub node: T,
    /// Optional analysis metadata
    pub metadata: Option<AnalysisMetadata>,
}

impl<T> AnalyzedNode<T> {
    /// Create analyzed node without metadata
    pub fn new(node: T) -> Self {
        Self {
            node,
            metadata: None,
        }
    }

    /// Create analyzed node with metadata
    pub fn with_metadata(node: T, metadata: AnalysisMetadata) -> Self {
        Self {
            node,
            metadata: Some(metadata),
        }
    }
}

impl<T> AnalysisEnhanced for AnalyzedNode<T> {
    fn analysis_metadata(&self) -> Option<&AnalysisMetadata> {
        self.metadata.as_ref()
    }

    fn set_analysis_metadata(&mut self, metadata: AnalysisMetadata) {
        self.metadata = Some(metadata);
    }
}

/// Result of analyzing an expression
#[derive(Debug, Clone)]
pub struct ExpressionAnalysis {
    /// All diagnostics found during analysis
    pub diagnostics: Vec<Diagnostic>,
    /// Whether analysis succeeded
    pub success: bool,
    /// Type information for the root expression
    pub root_type: Option<TypeInfo>,
    /// All resolved paths in the expression
    pub paths: Vec<String>,
}

impl ExpressionAnalysis {
    /// Create successful analysis result
    pub fn success(root_type: Option<TypeInfo>) -> Self {
        Self {
            diagnostics: Vec::new(),
            success: true,
            root_type,
            paths: Vec::new(),
        }
    }

    /// Create failed analysis result
    pub fn failure(diagnostics: Vec<Diagnostic>) -> Self {
        Self {
            diagnostics,
            success: false,
            root_type: None,
            paths: Vec::new(),
        }
    }

    /// Add a diagnostic
    pub fn add_diagnostic(&mut self, diagnostic: Diagnostic) {
        let is_error = matches!(diagnostic.severity, crate::diagnostics::DiagnosticSeverity::Error);
        self.diagnostics.push(diagnostic);
        if is_error {
            self.success = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::{DiagnosticSeverity, DiagnosticCode};

    #[test]
    fn test_analysis_metadata_creation() {
        let metadata = AnalysisMetadata::new();
        assert!(metadata.type_info.is_none());
        assert!(metadata.path.is_none());
        assert!(metadata.diagnostics.is_empty());
        assert!(metadata.properties.is_empty());
    }

    #[test]
    fn test_metadata_with_type_info() {
        let type_info = TypeInfo::string();
        let metadata = AnalysisMetadata::with_type_info(type_info.clone());
        assert_eq!(metadata.type_info, Some(type_info));
    }

    #[test]
    fn test_diagnostic_methods() {
        let mut metadata = AnalysisMetadata::new();
        assert!(!metadata.has_errors());
        assert!(!metadata.has_warnings());

        let error_diagnostic = Diagnostic {
            severity: DiagnosticSeverity::Error,
            code: DiagnosticCode {
                code: "E001".to_string(),
                namespace: Some("test".to_string()),
            },
            message: "Test error".to_string(),
            location: None,
            related: vec![],
        };

        metadata.add_diagnostic(error_diagnostic);
        assert!(metadata.has_errors());
    }

    #[test]
    fn test_analyzed_node() {
        let node = "test_node";
        let analyzed = AnalyzedNode::new(node);
        assert!(analyzed.analysis_metadata().is_none());

        let metadata = AnalysisMetadata::new();
        let analyzed_with_meta = AnalyzedNode::with_metadata(node, metadata);
        assert!(analyzed_with_meta.analysis_metadata().is_some());
    }
}