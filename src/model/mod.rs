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

//! Data model and value types for FHIRPath expressions
//!
//! This crate provides the core data types used in FHIRPath evaluation,
//! including the value model and FHIR resource wrappers.

#![warn(missing_docs)]

pub mod arc_pool;
pub mod boxing;
pub mod cache;
pub mod choice_types;
pub mod error;
pub mod fhirschema_provider;
pub mod json_arc;
pub mod lazy;
pub mod mock_provider;
pub mod profile_resolver;
pub mod provider;
pub mod quantity;
pub mod resource;
/// Smart collection types for efficient value storage and iteration
pub mod smart_collection;
pub mod string_intern;
pub mod type_mapper;
pub mod types;
pub mod value;
pub mod value_pool;

pub use arc_pool::{
    ArcPoolConfig, ArcPoolStats, CombinedArcPoolStats, FragmentationStats, GlobalArcPoolManager,
    TypedArcPool, get_pooled_collection, get_pooled_fhir_value, global_arc_pool,
};
pub use boxing::{BoxedValue, Boxing, Extension, PrimitiveElement, TypeInfo as BoxingTypeInfo};
pub use cache::{CacheConfig, CacheManager, CacheStats, ElementCache, TypeReflectionCache};
pub use choice_types::{ChoicePattern, ChoiceResolution, ChoiceTypeResolver};
pub use error::{ModelError, Result};
pub use fhirschema_provider::{FhirSchemaConfig, FhirSchemaModelProvider};
pub use json_arc::{ArcJsonValue, ArrayView};
pub use lazy::{LazyCollection, LazyIterator, ToLazy};
pub use mock_provider::MockModelProvider;
pub use octofhir_fhirschema::PackageSpec;
pub use profile_resolver::{ProfileResolver, ResolvedElement, ResolvedProfile, SlicingInfo};
pub use provider::{
    ElementInfo, FhirVersion, ModelProvider, SearchParameter, StructureDefinition,
    TypeReflectionInfo,
};
pub use quantity::Quantity;
pub use resource::FhirResource;
pub use smart_collection::{SmartCollection, SmartCollectionBuilder, SmartCollectionIter};
pub use string_intern::{
    InternerStats, clear_global_interner, global_interner_stats, global_interner_stats_compat,
    intern_string, is_interned,
};
pub use type_mapper::TypeMapper;
pub use types::TypeInfo;
pub use value::{Collection, FhirPathValue, ValueRef};
pub use value_pool::{
    CombinedValuePoolStats, PooledValue, ValuePoolConfig, ValuePoolStats, clear_global_pools,
    configure_global_pools, create_pooled_collection, get_pooled_collection_vec,
    get_pooled_json_value, get_pooled_string as get_pooled_string_vp, global_pool_stats,
    pooled_collection_vec, pooled_json_value, pooled_string, return_pooled_collection_vec,
    return_pooled_json_value, return_pooled_string,
};
