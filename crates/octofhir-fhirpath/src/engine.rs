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

//! FHIRPath engine - the main entry point for FHIRPath evaluation
//!
//! This module provides type aliases and re-exports for the main FHIRPath engines.
//! The core implementation is in FhirPathEngine from the evaluator crate.

use octofhir_fhirpath_evaluator::FhirPathEngine;

/// Type alias for the main FHIRPath engine
/// This is now the unified FhirPathEngine 
pub type FhirPathEngineWithCache = FhirPathEngine;

/// Type alias for compatibility - all engines are now Send-safe by default
pub type IntegratedFhirPathEngine = FhirPathEngine;