//! FHIRPath pipeline optimization components
//!
//! This module contains optimizations for the FHIRPath evaluation pipeline,
//! including memory pools, caching, and other performance enhancements.

pub mod memory_pool;

pub use memory_pool::{
    AsyncPool, FhirPathPools, PoolConfig, PoolMonitor, PoolStats, PooledObject, global_pools,
};
