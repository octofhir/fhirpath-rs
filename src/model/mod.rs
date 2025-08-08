//! Data model and value types for FHIRPath expressions
//!
//! This crate provides the core data types used in FHIRPath evaluation,
//! including the value model and FHIR resource wrappers.

#![warn(missing_docs)]

pub mod arc_pool;
pub mod error;
pub mod json_arc;
pub mod lazy;
pub mod provider;
pub mod quantity;
pub mod resource;
pub mod smart_collection;
pub mod string_intern;
pub mod types;
pub mod value;
pub mod value_pool;

pub use arc_pool::{
    ArcPoolConfig, ArcPoolStats, CombinedArcPoolStats, FragmentationStats, GlobalArcPoolManager,
    TypedArcPool, get_pooled_collection, get_pooled_fhir_value, global_arc_pool,
};
pub use error::{ModelError, Result};
pub use json_arc::{ArcJsonValue, ArrayView};
pub use lazy::{LazyCollection, LazyIterator, ToLazy};
pub use provider::{FhirVersion, ModelProvider};
pub use quantity::Quantity;
pub use resource::FhirResource;
pub use smart_collection::{SmartCollection, SmartCollectionBuilder, SmartCollectionIter};
pub use string_intern::{
    InternerStats, clear_global_interner, global_interner_stats, global_interner_stats_compat,
    intern_string, is_interned,
};
pub use types::TypeInfo;
pub use value::{Collection, FhirPathValue, ValueRef};
pub use value_pool::{
    CombinedValuePoolStats, PooledValue, ValuePoolConfig, ValuePoolStats, clear_global_pools,
    configure_global_pools, create_pooled_collection, get_pooled_collection_vec,
    get_pooled_json_value, get_pooled_string as get_pooled_string_vp, global_pool_stats,
    pooled_collection_vec, pooled_json_value, pooled_string, return_pooled_collection_vec,
    return_pooled_json_value, return_pooled_string,
};

// Re-export FHIR Schema types when async-schema feature is enabled
#[cfg(feature = "async-schema")]
pub mod schema;

#[cfg(feature = "async-schema")]
pub use schema::{FhirSchema, FhirSchemaProvider};
