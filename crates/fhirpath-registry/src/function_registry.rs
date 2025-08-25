//! Function Registry for FHIRPath Operations
//!
//! This module provides a function registry that can handle both synchronous and asynchronous
//! FHIRPath operations, automatically dispatching to the appropriate implementation for
//! optimal performance.

use crate::traits::{SyncOperation, AsyncOperation, EvaluationContext};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use std::collections::HashMap;
use std::sync::Arc;

/// Function registry that manages both sync and async FHIRPath operations
/// 
/// The registry automatically dispatches to sync operations when possible for better
/// performance, falling back to async operations when necessary.
pub struct FunctionRegistry {
    sync_operations: HashMap<String, Arc<dyn SyncOperation>>,
    async_operations: HashMap<String, Arc<dyn AsyncOperation>>,
}

impl FunctionRegistry {
    /// Create a new empty function registry
    pub fn new() -> Self {
        Self {
            sync_operations: HashMap::new(),
            async_operations: HashMap::new(),
        }
    }

    /// Register a synchronous operation
    pub fn register_sync<T>(&mut self, operation: T) -> &mut Self
    where
        T: SyncOperation + 'static,
    {
        let name = operation.name().to_string();
        self.sync_operations.insert(name, Arc::new(operation));
        self
    }

    /// Register an asynchronous operation
    pub fn register_async<T>(&mut self, operation: T) -> &mut Self
    where
        T: AsyncOperation + 'static,
    {
        let name = operation.name().to_string();
        self.async_operations.insert(name, Arc::new(operation));
        self
    }

    /// Register multiple synchronous operations at once
    pub fn register_sync_many(&mut self, operations: Vec<Box<dyn SyncOperation>>) -> &mut Self {
        for operation in operations {
            let name = operation.name().to_string();
            self.sync_operations.insert(name, operation.into());
        }
        self
    }

    /// Register multiple asynchronous operations at once  
    pub fn register_async_many(&mut self, operations: Vec<Box<dyn AsyncOperation>>) -> &mut Self {
        for operation in operations {
            let name = operation.name().to_string();
            self.async_operations.insert(name, operation.into());
        }
        self
    }

    /// Evaluate a function by name with smart dispatch
    /// 
    /// This method tries sync operations first for better performance,
    /// then falls back to async operations if needed.
    pub async fn evaluate(
        &self,
        name: &str,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Try sync first for performance
        if let Some(sync_op) = self.sync_operations.get(name) {
            return sync_op.execute(args, context);
        }

        // Fall back to async if needed
        if let Some(async_op) = self.async_operations.get(name) {
            return async_op.execute(args, context).await;
        }

        // Function not found
        Err(FhirPathError::UnknownFunction {
            function_name: name.to_string(),
        })
    }

    /// Try to evaluate synchronously only
    /// 
    /// Returns None if the operation requires async execution
    pub fn try_evaluate_sync(
        &self,
        name: &str,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        self.sync_operations
            .get(name)
            .map(|sync_op| sync_op.execute(args, context))
    }

    /// Check if a function exists (sync or async)
    pub fn has_function(&self, name: &str) -> bool {
        self.sync_operations.contains_key(name) || self.async_operations.contains_key(name)
    }

    /// Check if a function supports synchronous execution
    pub fn supports_sync(&self, name: &str) -> bool {
        self.sync_operations.contains_key(name)
    }

    /// Get list of all function names
    pub fn function_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        names.extend(self.sync_operations.keys().cloned());
        names.extend(self.async_operations.keys().cloned());
        names.sort();
        names.dedup();
        names
    }

    /// Get statistics about the registry
    pub fn stats(&self) -> RegistryStats {
        RegistryStats {
            sync_operations: self.sync_operations.len(),
            async_operations: self.async_operations.len(),
            total_operations: self.sync_operations.len() + self.async_operations.len(),
        }
    }
}

impl Default for FunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about operations in the registry
#[derive(Debug, Clone)]
pub struct RegistryStats {
    pub sync_operations: usize,
    pub async_operations: usize,
    pub total_operations: usize,
}

impl RegistryStats {
    pub fn sync_percentage(&self) -> f64 {
        if self.total_operations == 0 {
            0.0
        } else {
            (self.sync_operations as f64 / self.total_operations as f64) * 100.0
        }
    }
}

/// Create a registry with all standard FHIRPath operations
pub fn create_standard_registry() -> FunctionRegistry {
    let mut registry = FunctionRegistry::new();

    // Register sync string operations using batch registration
    registry.register_sync_many(vec![
        Box::new(crate::operations::string_sync::SimpleLengthFunction::default()),
        Box::new(crate::operations::string_sync::SimpleUpperFunction::default()),
        Box::new(crate::operations::string_sync::SimpleLowerFunction::default()),
        Box::new(crate::operations::string_sync::SimpleContainsFunction::default()),
        Box::new(crate::operations::string_sync::SimpleStartsWithFunction::default()),
        Box::new(crate::operations::string_sync::SimpleEndsWithFunction::default()),
        Box::new(crate::operations::string_sync::SimpleIndexOfFunction::default()),
        Box::new(crate::operations::string_sync::SimpleLastIndexOfFunction::default()),
        Box::new(crate::operations::string_sync::SimpleSubstringFunction::default()),
        Box::new(crate::operations::string_sync::SimpleReplaceFunction::default()),
        Box::new(crate::operations::string_sync::SimpleSplitFunction::default()),
        Box::new(crate::operations::string_sync::SimpleJoinFunction::default()),
        Box::new(crate::operations::string_sync::SimpleTrimFunction::default()),
        Box::new(crate::operations::string_sync::SimpleToCharsFunction::default()),
        Box::new(crate::operations::string_sync::SimpleMatchesFunction::default()),
        Box::new(crate::operations::string_sync::SimpleMatchesFullFunction::default()),
        Box::new(crate::operations::string_sync::SimpleReplaceMatchesFunction::default()),
    ]);

    // Register sync math operations using batch registration
    registry.register_sync_many(vec![
        Box::new(crate::operations::math_sync::SimpleAbsFunction::default()),
        Box::new(crate::operations::math_sync::SimpleCeilingFunction::default()),
        Box::new(crate::operations::math_sync::SimpleFloorFunction::default()),
        Box::new(crate::operations::math_sync::SimpleRoundFunction::default()),
        Box::new(crate::operations::math_sync::SimpleTruncateFunction::default()),
        Box::new(crate::operations::math_sync::SimpleSqrtFunction::default()),
        Box::new(crate::operations::math_sync::SimplePowerFunction::default()),
        Box::new(crate::operations::math_sync::SimpleLnFunction::default()),
        Box::new(crate::operations::math_sync::SimpleLogFunction::default()),
        Box::new(crate::operations::math_sync::SimpleExpFunction::default()),
        Box::new(crate::operations::math_sync::SimplePrecisionFunction::default()),
        Box::new(crate::operations::math_sync::SimpleAddFunction::default()),
        Box::new(crate::operations::math_sync::SimpleSubtractFunction::default()),
        Box::new(crate::operations::math_sync::SimpleMultiplyFunction::default()),
        Box::new(crate::operations::math_sync::SimpleDivideFunction::default()),
        Box::new(crate::operations::math_sync::SimpleModuloFunction::default()),
    ]);

    // Register sync collection operations using batch registration
    registry.register_sync_many(vec![
        Box::new(crate::operations::collection_sync::SimpleCountFunction::default()),
        Box::new(crate::operations::collection_sync::SimpleEmptyFunction::default()),
        Box::new(crate::operations::collection_sync::SimpleExistsFunction::default()),
        Box::new(crate::operations::collection_sync::SimpleFirstFunction::default()),
        Box::new(crate::operations::collection_sync::SimpleLastFunction::default()),
        Box::new(crate::operations::collection_sync::SimpleTailFunction::default()),
        Box::new(crate::operations::collection_sync::SimpleSkipFunction::default()),
        Box::new(crate::operations::collection_sync::SimpleTakeFunction::default()),
        Box::new(crate::operations::collection_sync::SimpleSingleFunction::default()),
        Box::new(crate::operations::collection_sync::SimpleDistinctFunction::default()),
        Box::new(crate::operations::collection_sync::SimpleIsDistinctFunction::default()),
        Box::new(crate::operations::collection_sync::SimpleUnionFunction::default()),
        Box::new(crate::operations::collection_sync::SimpleIntersectFunction::default()),
        Box::new(crate::operations::collection_sync::SimpleExcludeFunction::default()),
        Box::new(crate::operations::collection_sync::SimpleSubsetOfFunction::default()),
        Box::new(crate::operations::collection_sync::SimpleSupersetOfFunction::default()),
        Box::new(crate::operations::collection_sync::SimpleAllTrueFunction::default()),
        Box::new(crate::operations::collection_sync::SimpleAnyTrueFunction::default()),
        Box::new(crate::operations::collection_sync::SimpleAllFalseFunction::default()),
        Box::new(crate::operations::collection_sync::SimpleAnyFalseFunction::default()),
        Box::new(crate::operations::collection_sync::SimpleCombineFunction::default()),
    ]);

    // Register sync datetime extraction operations (from Task 24) 
    registry.register_sync_many(vec![
        Box::new(crate::operations::datetime_sync::DayOfFunction::default()),
        Box::new(crate::operations::datetime_sync::HourOfFunction::default()),
        Box::new(crate::operations::datetime_sync::MinuteOfFunction::default()),
        Box::new(crate::operations::datetime_sync::SecondOfFunction::default()),
        Box::new(crate::operations::datetime_sync::MillisecondOfFunction::default()),
        Box::new(crate::operations::datetime_sync::MonthOfFunction::default()),
        Box::new(crate::operations::datetime_sync::YearOfFunction::default()),
        Box::new(crate::operations::datetime_sync::TimezoneOffsetOfFunction::default()),
        Box::new(crate::operations::datetime_sync::TimeOfDayFunction::default()),
        Box::new(crate::operations::datetime_sync::HighBoundaryFunction::default()),
        Box::new(crate::operations::datetime_sync::LowBoundaryFunction::default()),
    ]);

    // Register sync FHIR data traversal operations (from Task 16)
    registry.register_sync_many(vec![
        Box::new(crate::operations::fhir_sync::ChildrenFunction::default()),
        Box::new(crate::operations::fhir_sync::DescendantsFunction::default()),
    ]);

    // Register sync utility operations (from Task 23)
    registry.register_sync_many(vec![
        Box::new(crate::operations::utility_sync::HasValueFunction::default()),
        Box::new(crate::operations::utility_sync::ComparableFunction::default()),
        Box::new(crate::operations::utility_sync::EncodeFunction::default()),
        Box::new(crate::operations::utility_sync::DecodeFunction::default()),
        Box::new(crate::operations::utility_sync::EscapeFunction::default()),
        Box::new(crate::operations::utility_sync::UnescapeFunction::default()),
        Box::new(crate::operations::utility_sync::TraceFunction::default()),
        Box::new(crate::operations::utility_sync::DefineVariableFunction::default()),
    ]);

    // Register sync logical operations (from Task 23)
    registry.register_sync_many(vec![
        Box::new(crate::operations::logical_sync::NotOperation::default()),
    ]);

    // Register async datetime system call operations (from Task 24) using batch registration
    registry.register_async_many(vec![
        Box::new(crate::operations::datetime_async::NowFunction::default()),
        Box::new(crate::operations::datetime_async::TodayFunction::default()),
    ]);

    // Register async FHIR ModelProvider operations (from Task 16) using batch registration
    registry.register_async_many(vec![
        Box::new(crate::operations::fhir_async::ResolveFunction::default()),
        Box::new(crate::operations::fhir_async::ConformsToFunction::default()),
        Box::new(crate::operations::fhir_async::ExtensionFunction::default()),
    ]);

    // Register async type operations using batch registration
    registry.register_async_many(vec![
        Box::new(crate::operations::types_async::TypeFunction::default()),
        Box::new(crate::operations::types_async::IsOperation::default()),
        Box::new(crate::operations::types_async::OfTypeFunction::default()),
        Box::new(crate::operations::types_async::AsOperation::default()),
    ]);

    // Register sync CDA operations
    registry.register_sync_many(vec![
        Box::new(crate::operations::cda_sync::HasTemplateIdOfFunction::default()),
    ]);

    // Register sync conversion operations using batch registration
    registry.register_sync_many(vec![
        // Type checking operations (converts_to_*)
        Box::new(crate::operations::conversion_sync::ConvertsToBooleanFunction),
        Box::new(crate::operations::conversion_sync::ConvertsToDateFunction),
        Box::new(crate::operations::conversion_sync::ConvertsToDateTimeFunction),
        Box::new(crate::operations::conversion_sync::ConvertsToDecimalFunction),
        Box::new(crate::operations::conversion_sync::ConvertsToIntegerFunction),
        Box::new(crate::operations::conversion_sync::ConvertsToLongFunction),
        Box::new(crate::operations::conversion_sync::ConvertsToQuantityFunction),
        Box::new(crate::operations::conversion_sync::ConvertsToStringFunction),
        Box::new(crate::operations::conversion_sync::ConvertsToTimeFunction),
        
        // Type conversion operations (to_*)
        Box::new(crate::operations::conversion_sync::ToBooleanFunction),
        Box::new(crate::operations::conversion_sync::ToDateFunction),
        Box::new(crate::operations::conversion_sync::ToDateTimeFunction),
        Box::new(crate::operations::conversion_sync::ToDecimalFunction),
        Box::new(crate::operations::conversion_sync::ToIntegerFunction),
        Box::new(crate::operations::conversion_sync::ToLongFunction),
        Box::new(crate::operations::conversion_sync::ToQuantityFunction),
        Box::new(crate::operations::conversion_sync::ToStringFunction),
        Box::new(crate::operations::conversion_sync::ToTimeFunction),
    ]);

    registry
}