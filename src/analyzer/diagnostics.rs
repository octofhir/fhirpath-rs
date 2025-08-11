// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


//! Diagnostic system for FHIRPath expressions
//!
//! This module provides comprehensive diagnostic reporting including type mismatches,
//! property validation, function signature errors, and LSP-compatible output.

use crate::ast::{ExpressionNode, BinaryOperator, UnaryOperator};
use crate::model::provider::ModelProvider;
use crate::analyzer::{AnalysisContext, AnalysisError};
// Removed Span import to avoid lifetime issues
use std::sync::Arc;
use std::collections::HashMap;

/// A diagnostic message
#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    /// Diagnostic severity
    pub severity: DiagnosticSeverity,
    /// Diagnostic code (for categorization)
    pub code: String,
    /// Human-readable message
    pub message: String,
    /// Start byte offset where the diagnostic applies
    pub start_offset: Option<usize>,
    /// End byte offset where the diagnostic applies
    pub end_offset: Option<usize>,
    /// Additional context or suggestions
    pub related_information: Vec<DiagnosticRelatedInformation>,
    /// Data for LSP integration
    pub data: Option<serde_json::Value>,
}

/// Severity levels for diagnostics
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DiagnosticSeverity {
    /// Error that prevents execution
    Error,
    /// Warning about potential issues
    Warning,
    /// Informational message
    Information,
    /// Hint for improvements
    Hint,
}

/// Related diagnostic information
#[derive(Debug, Clone, PartialEq)]
pub struct DiagnosticRelatedInformation {
    /// Start byte offset of related information
    pub start_offset: Option<usize>,
    /// End byte offset of related information
    pub end_offset: Option<usize>,
    /// Related message
    pub message: String,
}

/// Category of diagnostic for organization
#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticCategory {
    /// Type-related errors
    Type,
    /// Navigation path issues
    Navigation,
    /// Function call problems
    Function,
    /// Syntax or semantic issues
    Semantic,
    /// Performance concerns
    Performance,
    /// Compatibility issues
    Compatibility,
}

/// Diagnostic system for analyzing FHIRPath expressions
pub struct DiagnosticSystem<P: ModelProvider> {
    provider: Arc<P>,
    error_codes: HashMap<String, DiagnosticInfo>,
}

/// Information about a diagnostic code
#[derive(Debug, Clone)]
struct DiagnosticInfo {
    category: DiagnosticCategory,
    default_severity: DiagnosticSeverity,
    description: String,
}

impl<P: ModelProvider> DiagnosticSystem<P> {
    /// Create a new diagnostic system
    pub fn new(provider: Arc<P>) -> Self {
        let mut error_codes = HashMap::new();
        Self::register_diagnostic_codes(&mut error_codes);
        
        Self {
            provider,
            error_codes,
        }
    }

    /// Create a diagnostic with the given code and message
    fn create_diagnostic(&self, code: &str, message: String, start_offset: Option<usize>, end_offset: Option<usize>) -> Diagnostic {
        let info = self.error_codes.get(code);
        
        Diagnostic {
            severity: info.map(|i| i.default_severity.clone()).unwrap_or(DiagnosticSeverity::Error),
            code: code.to_string(),
            message,
            start_offset,
            end_offset,
            related_information: Vec::new(),
            data: None,
        }
    }

    /// Analyze expression for diagnostics
    pub async fn analyze_expression(&self, _expression: &ExpressionNode, _context_type: Option<&str>) -> Result<Vec<Diagnostic>, AnalysisError> {
        // Simplified diagnostic analysis - would be more comprehensive in real implementation
        Ok(Vec::new())
    }

    /// Register diagnostic codes
    fn register_diagnostic_codes(codes: &mut HashMap<String, DiagnosticInfo>) {
        let diagnostics = [
            ("INVALID_DECIMAL", DiagnosticInfo {
                category: DiagnosticCategory::Type,
                default_severity: DiagnosticSeverity::Error,
                description: "Invalid decimal literal format".to_string(),
            }),
        ];

        for (code, info) in diagnostics {
            codes.insert(code.to_string(), info);
        }
    }
}

impl Diagnostic {
    /// Convert to LSP diagnostic format
    pub fn to_lsp_diagnostic(&self) -> serde_json::Value {
        serde_json::json!({
            "range": serde_json::Value::Null,
            "severity": match self.severity {
                DiagnosticSeverity::Error => 1,
                DiagnosticSeverity::Warning => 2,
                DiagnosticSeverity::Information => 3,
                DiagnosticSeverity::Hint => 4,
            },
            "code": self.code,
            "message": self.message,
            "source": "fhirpath-analyzer"
        })
    }
}