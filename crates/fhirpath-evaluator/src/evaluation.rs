//! Core FHIRPath expression evaluation logic
//!
//! This module contains the primary expression evaluation engine that dispatches
//! to specialized evaluators based on expression node types. It handles recursion
//! depth checking, performance monitoring, and delegates to appropriate evaluators.

// This module is now consolidated into engine.rs
// All evaluation logic has been moved there to avoid method duplication