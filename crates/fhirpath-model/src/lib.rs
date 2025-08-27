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

//! Value types and FHIR model support for FHIRPath implementation
//!
//! This crate provides the value types, FHIR resource handling, and model
//! provider abstractions used throughout the FHIRPath implementation.
//!
//! ## Performance Enhancement
//!
//! The [`FhirSchemaModelProvider`] now uses a precomputed type registry by default
//! for fast type operations. All default constructors (`new()`, `r4()`, `r5()`, etc.)
//! automatically build and use the [`PrecomputedTypeRegistry`] for optimal performance
//! in type reflection operations like `type()`, `is()`, and `as()` functions.

pub mod boxing;
pub mod cache;
pub mod choice_type_mapper;
pub mod coercion_utils;
pub mod error;
pub mod fhirschema_provider;
#[cfg(test)]
pub mod fhirschema_provider_test;
pub mod json_value;
pub mod legacy_cache;
pub mod mock_provider;
pub mod mock_type_definitions;
pub mod polymorphic_factory;
pub mod polymorphic_resolver;
pub mod precomputed_registry;
pub mod profile_resolver;
pub mod provider;
pub mod quantity;
pub mod resource;
pub mod smart_collection;
pub mod string_intern;
pub mod temporal;
pub mod type_analyzer;
pub mod type_coercion;
pub mod type_object;
pub mod types;
pub mod value;

// Re-export main types
pub use cache::{Cache, CacheConfig, CacheStats};
pub use choice_type_mapper::{ChoiceTypeMapper, ChoiceVariant, SharedChoiceTypeMapper};
pub use fhirschema_provider::FhirSchemaModelProvider;
pub use polymorphic_resolver::{PolymorphicPathResolver, PolymorphicResolverFactory, ResolvedPath};
// JsonParser functionality is integrated into JsonValue directly
pub use json_value::JsonValue;
pub use mock_provider::MockModelProvider;
pub use precomputed_registry::{
    ChoiceTypeInfo, FhirTypeInfo, PrecomputedTypeRegistry, PrimitiveTypeKind, PropertyInfo,
    RegistryStatistics, SystemTypeInfo,
};
pub use provider::ModelProvider;
pub use quantity::Quantity;
pub use smart_collection::{SmartCollection, SmartCollectionBuilder};
pub use temporal::{PrecisionDate, PrecisionDateTime, PrecisionTime, TemporalPrecision};
pub use type_object::{FhirPathTypeObject, TypeObjectMetadata, ValueTypeAnalyzer};
pub use value::{Collection, FhirPathValue};

// Re-export value pool functionality
pub use string_intern::{InternerStats, global_interner_stats};

// Re-export from workspace crates for convenience
pub use octofhir_fhirpath_core::{FhirPathError, Result};

// Re-export from external crates
pub use octofhir_fhir_model as fhir_model;
