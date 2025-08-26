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

//! Diagnostic and error reporting for FHIRPath implementation
//!
//! This crate provides comprehensive error handling, diagnostic reporting,
//! and LSP support for FHIRPath expressions.

pub mod builder;
pub mod diagnostic;
pub mod diagnostic_reporter;
pub mod formatter;
pub mod location;
pub mod lsp;

// Re-export main types (using streamlined modules)
pub use builder::DiagnosticBuilder;
pub use diagnostic::{Diagnostic, DiagnosticCode, Severity, Severity as DiagnosticSeverity};
pub use diagnostic_reporter::DiagnosticReporter;
pub use formatter::DiagnosticFormatter;
pub use location::{Position, SourceLocation, Span};
