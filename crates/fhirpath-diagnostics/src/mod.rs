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

//! Diagnostic system for FHIRPath parsing and evaluation errors
//!
//! This crate provides a comprehensive diagnostic system that can produce
//! both human-friendly error messages and machine-readable diagnostics
//! suitable for IDE integration.

#![warn(missing_docs)]

pub mod builder;
pub mod diagnostic;
pub mod diagnostic_reporter;
pub mod enhanced_diagnostic;
pub mod formatter;
pub mod location;

pub use builder::DiagnosticBuilder;
pub use diagnostic::{Diagnostic, DiagnosticCode, RelatedInformation, Severity, Suggestion};
pub use diagnostic_reporter::{
    DiagnosticAnalysis, DiagnosticReport, DiagnosticReporter, DiagnosticSummary, ErrorPattern,
    GroupedDiagnostics, ReporterConfig, RootCause, WorkflowStep,
};
pub use enhanced_diagnostic::{
    DocumentationLink, EnhancedDiagnostic, QuickFix, SmartSuggestion, SuggestionCategory,
    SuggestionGenerator,
};
pub use formatter::{DiagnosticFormatter, Format};
pub use location::{Position, SourceLocation, Span};

// LSP integration moved to separate crate
