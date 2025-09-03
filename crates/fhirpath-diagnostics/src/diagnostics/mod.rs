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

//! Enhanced diagnostic system for FHIRPath analysis

pub mod diagnostic_engine;
pub mod error_codes;
pub mod formatters;
pub mod suggestions;

// Re-export main types from diagnostic_engine
pub use diagnostic_engine::{
    DiagnosticEngine, ErrorRecoveryEngine, FormatError, RecoveryResult, RecoveryStrategy,
    SuggestionEngine,
};

// Re-export error codes
pub use error_codes::{
    ErrorCategory, ErrorCodeRegistry, StructuredDiagnosticCode,
};

// Re-export formatters
pub use formatters::{
    RustLikeDiagnosticFormatter, RustLikeFormatterConfig,
};

// Re-export suggestion engine
pub use suggestions::{
    EnhancedSuggestionEngine, fuzzy_matching,
};