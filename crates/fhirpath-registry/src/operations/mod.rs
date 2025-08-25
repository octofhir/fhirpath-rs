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

//! Simplified FHIRPath operations
//!
//! This module contains clean implementations of FHIRPath operations split into 
//! sync and async categories based on their actual needs.

// New simplified sync operations (80% of operations)
pub mod cda_sync;           // CDA document operations
pub mod collection_sync;    // Collection operations (count, first, etc.)
pub mod conversion_sync;    // Type conversion operations (toString, toInteger, etc.)
pub mod datetime_sync;      // DateTime extraction operations (dayOf, hourOf, etc.)
pub mod fhir_sync;         // FHIR data traversal operations (children, descendants)
pub mod logical_sync;      // Logical operations (not)
pub mod math_sync;         // Mathematical operations (abs, sqrt, etc.)
pub mod string_sync;       // String operations (length, contains, etc.)
pub mod utility_sync;      // Utility operations (encode, decode, etc.)

// New simplified async operations (20% of operations)
pub mod datetime_async;    // System datetime operations (now, today)
pub mod fhir_async;       // FHIR operations requiring ModelProvider (resolve, conforms_to)
pub mod types_async;      // Type operations requiring ModelProvider (as, is, type)

// Re-exports for convenience
pub use cda_sync::*;
pub use collection_sync::*;
pub use conversion_sync::*;
pub use datetime_sync::*;
pub use datetime_async::*;
pub use fhir_sync::*;
pub use fhir_async::*;
pub use logical_sync::*;
pub use math_sync::*;
pub use string_sync::*;
pub use types_async::*;
pub use utility_sync::*;