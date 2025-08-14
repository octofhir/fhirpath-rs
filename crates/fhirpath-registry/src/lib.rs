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

//! Function and operator registry for FHIRPath implementation
//!
//! This crate provides the comprehensive function registry with built-in functions,
//! operators, and extension system for FHIRPath expressions.

// Core registry modules
pub mod function;
pub mod signature;

// Lambda expression support 
pub mod expression_argument;
pub mod lambda_function;
// pub mod enhanced_aggregate; // Moved to unified_implementations/aggregates/
// pub mod enhanced_iif; // Moved to unified_implementations/utility/
pub mod engine_integration;
pub mod integration_test;
pub mod simple_lambda_test;
pub mod test_property_access;

// Caching and performance optimization
pub mod cache;
pub mod compiled_signatures;

// Unified system components
pub mod unified_function;
pub mod unified_implementations;
pub mod unified_operator;
pub mod unified_operator_registry;
pub mod unified_operators;
pub mod unified_registry;

// Enhanced metadata and LSP support
pub mod enhanced_metadata;
pub mod enhanced_operator_metadata;
pub mod metadata_builder;
pub mod operator_lsp;

// FHIRPath Registry V2 - Next generation async-first architecture
pub mod async_cache;
pub mod fhirpath_registry;
pub mod metadata;
pub mod migration_utils;
pub mod operation;
pub mod operations;

// Main registry types
pub use function::FunctionRegistry;
pub use signature::{FunctionSignature, ParameterInfo};

// Lambda expression exports
pub use expression_argument::{ExpressionArgument, VariableScope};
pub use lambda_function::{LambdaFhirPathFunction, LambdaEvaluationContext, EnhancedFunctionImpl, is_lambda_function, get_lambda_argument_indices};
// pub use enhanced_aggregate::EnhancedAggregateFunction; // Moved to unified_implementations/aggregates/
// pub use enhanced_iif::EnhancedIifFunction; // Moved to unified_implementations/utility/

// Unified system exports
pub use unified_function::{ExecutionMode, UnifiedFhirPathFunction};
pub use unified_operator::{
    ArithmeticOperator, Associativity, ComparisonOperator, LogicalOperator, OperatorError,
    OperatorResult, UnifiedFhirPathOperator,
};
pub use unified_operator_registry::{
    OperatorRegistryError, OperatorRegistryStats, UnifiedOperatorRegistry,
    create_unified_operator_registry,
};
pub use unified_registry::{RegistryError, RegistryStats, UnifiedFunctionRegistry};

// Enhanced metadata system
pub use enhanced_metadata::{
    EnhancedFunctionMetadata, PerformanceComplexity, PerformanceMetadata, UsagePattern,
};
pub use enhanced_operator_metadata::{
    EnhancedOperatorMetadata, OperatorComplexity, OperatorPerformanceMetadata,
};
pub use metadata_builder::MetadataBuilder;

// LSP and migration inventory support
pub use operator_lsp::{
    CompletionContext, OperatorCompletionItem, OperatorDiagnostic, OperatorLspProvider,
};
// pub use operator_migration_inventory::{ImplementationStatus, OperatorMigrationInventory};

// FHIRPath Registry V2 exports
pub use async_cache::{AsyncLruCache, CacheBuilder, CacheMetrics};
pub use fhirpath_registry::{FhirPathRegistry, DispatchKey};
pub use metadata::{
    OperationMetadata, OperationType as V2OperationType, TypeConstraint, FhirPathType,
    LspMetadata, OperationSpecificMetadata, FunctionMetadata, OperatorMetadata,
    PerformanceMetadata as V2PerformanceMetadata,
    MetadataBuilder as V2MetadataBuilder,
    Associativity as V2Associativity,
};
pub use migration_utils::{
    MigrationConfig, MigrationError, MigrationStats, RegistryMigrationHelper, ValidationReport,
};
pub use operation::{
    FhirPathOperation, OperationComplexity, CompilableOperation, CollectionOperation,
    ScalarOperation, CompiledOperation,
};
pub use operations::{
    arithmetic::{ArithmeticOperations, AdditionOperation, SubtractionOperation, MultiplicationOperation},
};

// Unified implementations (selected exports to avoid conflicts)
pub use unified_implementations::{
    aggregates, boolean, cda, collection, datetime, fhir, filtering, math, string, string_extended,
    tree_navigation, type_checking, type_conversion, utility,
};

/// Create a unified registry with all built-in functions
///
/// This is the new high-performance registry that will eventually replace
/// the legacy function registry after migration is complete.
pub fn create_unified_registry() -> UnifiedFunctionRegistry {
    let mut registry = UnifiedFunctionRegistry::new();

    // Register the unified collection functions (Phase 1 of migration)
    register_unified_collection_functions(&mut registry);

    // Register the unified string functions (Phase 2 of migration)
    register_unified_string_functions(&mut registry);

    // Register the unified math functions (Phase 3 of migration)
    register_unified_math_functions(&mut registry);

    // Register the unified type conversion functions (Phase 4 of migration)
    register_unified_type_conversion_functions(&mut registry);

    // Register the unified boolean functions (Phase 5 of migration)
    register_unified_boolean_functions(&mut registry);

    // Register the unified datetime functions (Phase 6 of migration)
    register_unified_datetime_functions(&mut registry);

    // Register the unified FHIR-specific functions (Phase 7 of migration)
    register_unified_fhir_functions(&mut registry);

    // Register the unified utility functions (Phase 8 of migration)
    register_unified_utility_functions(&mut registry);

    // Register the unified filtering functions (Phase 9 of migration)
    register_unified_filtering_functions(&mut registry);

    // Register the unified tree navigation functions (Phase 10 of migration)
    register_unified_tree_navigation_functions(&mut registry);

    // Register the unified aggregate functions (Phase 11 of migration)
    register_unified_aggregate_functions(&mut registry);

    // Register the unified string extended functions (Phase 12 of migration)
    register_unified_string_extended_functions(&mut registry);

    // Register the unified type checking functions (Phase 13 of migration)
    register_unified_type_checking_functions(&mut registry);

    // Register the unified CDA functions (Phase 14 of migration)
    register_unified_cda_functions(&mut registry);

    registry
}

/// Register unified collection functions
///
/// This is the first phase of the migration - collection functions are the highest priority
/// and most commonly used, making them safe to migrate first.
fn register_unified_collection_functions(registry: &mut UnifiedFunctionRegistry) {
    use crate::unified_implementations::collection::*;

    // Register basic collection functions
    let _ = registry.register(UnifiedCountFunction::new());
    let _ = registry.register(UnifiedEmptyFunction::new());
    let _ = registry.register(UnifiedExistsFunction::new());

    // Register element access functions
    let _ = registry.register(UnifiedFirstFunction::new());
    let _ = registry.register(UnifiedLastFunction::new());
    let _ = registry.register(UnifiedSingleFunction::new());

    // Register boolean aggregation functions
    let _ = registry.register(UnifiedAllFunction::new());
    let _ = registry.register(UnifiedAnyFunction::new());

    // Register collection manipulation functions
    let _ = registry.register(UnifiedDistinctFunction::new());
    let _ = registry.register(UnifiedIsDistinctFunction::new());
    let _ = registry.register(UnifiedTakeFunction::new());
    let _ = registry.register(UnifiedSkipFunction::new());
    let _ = registry.register(UnifiedTailFunction::new());
    // let _ = registry.register(UnifiedSortFunction::new()); // Replaced with enhanced version
    let _ = registry.register(EnhancedSortFunction::new()); // Enhanced sort with both simple and lambda support

    // Register collection search and analysis functions
    let _ = registry.register(UnifiedIndexOfFunction::new());
    let _ = registry.register(UnifiedIntersectFunction::new());
    let _ = registry.register(UnifiedFlattenFunction::new());
    let _ = registry.register(UnifiedExcludeFunction::new());
    let _ = registry.register(UnifiedCombineFunction::new());

    // Register collection set operations
    let _ = registry.register(UnifiedSubsetOfFunction::new());
    let _ = registry.register(UnifiedSupersetOfFunction::new());
    let _ = registry.register(UnifiedUnionFunction::new());
}

/// Register unified string functions
///
/// This is the second phase of the migration - string functions are commonly used
/// and have clear implementations, making them safe to migrate next.
fn register_unified_string_functions(registry: &mut UnifiedFunctionRegistry) {
    use crate::unified_implementations::string::*;

    // Register core string functions
    let _ = registry.register(UnifiedLengthFunction::new());
    let _ = registry.register(UnifiedSubstringFunction::new());
    let _ = registry.register(UnifiedContainsFunction::new());

    // Register string comparison functions
    let _ = registry.register(UnifiedStartsWithFunction::new());
    let _ = registry.register(UnifiedEndsWithFunction::new());

    // Register string transformation functions
    let _ = registry.register(UnifiedUpperFunction::new());
    let _ = registry.register(UnifiedLowerFunction::new());
    let _ = registry.register(UnifiedTrimFunction::new());
    let _ = registry.register(UnifiedToCharsFunction::new());
    let _ = registry.register(UnifiedEscapeFunction::new());
    let _ = registry.register(UnifiedUnescapeFunction::new());
}

/// Register unified math functions
///
/// This is the third phase of the migration - math functions are commonly used
/// and have straightforward implementations, making them safe to migrate.
fn register_unified_math_functions(registry: &mut UnifiedFunctionRegistry) {
    use crate::unified_implementations::math::*;

    // Register basic math functions
    let _ = registry.register(UnifiedAbsFunction::new());
    let _ = registry.register(UnifiedCeilingFunction::new());
    let _ = registry.register(UnifiedFloorFunction::new());
    let _ = registry.register(UnifiedRoundFunction::new());
    let _ = registry.register(UnifiedTruncateFunction::new());

    // Register advanced math functions
    let _ = registry.register(UnifiedSqrtFunction::new());
    let _ = registry.register(UnifiedExpFunction::new());
    let _ = registry.register(UnifiedLnFunction::new());
    let _ = registry.register(UnifiedLogFunction::new());
    let _ = registry.register(UnifiedPrecisionFunction::new());
    let _ = registry.register(UnifiedPowerFunction::new());
}

/// Register unified type conversion functions
///
/// This is the fourth phase of the migration - type conversion functions are commonly used
/// and essential for type safety and data validation in FHIRPath expressions.
fn register_unified_type_conversion_functions(registry: &mut UnifiedFunctionRegistry) {
    use crate::unified_implementations::type_conversion::*;

    // Register conversion functions
    let _ = registry.register(UnifiedToStringFunction::new());
    let _ = registry.register(UnifiedToIntegerFunction::new());
    let _ = registry.register(UnifiedToDecimalFunction::new());
    let _ = registry.register(UnifiedToBooleanFunction::new());
    let _ = registry.register(UnifiedToQuantityFunction::new());

    // Register validation functions
    let _ = registry.register(UnifiedConvertsToStringFunction::new());
    let _ = registry.register(UnifiedConvertsToIntegerFunction::new());
    let _ = registry.register(UnifiedConvertsToDecimalFunction::new());
    let _ = registry.register(UnifiedConvertsToBooleanFunction::new());
    let _ = registry.register(UnifiedConvertsToQuantityFunction::new());
}

/// Register unified boolean functions
///
/// This is the fifth phase of the migration - boolean functions are simple pure functions
/// with clear logic, making them safe to migrate after the core type and collection functions.
fn register_unified_boolean_functions(registry: &mut UnifiedFunctionRegistry) {
    use crate::unified_implementations::boolean::*;

    // Register boolean logic functions
    let _ = registry.register(UnifiedAllTrueFunction::new());
    let _ = registry.register(UnifiedAnyFalseFunction::new());
    let _ = registry.register(UnifiedAllFalseFunction::new());
    let _ = registry.register(UnifiedNotFunction::new());
    let _ = registry.register(UnifiedImpliesFunction::new());
}

/// Register unified datetime functions
///
/// This is the sixth phase of the migration - datetime functions provide temporal operations
/// with optimized sync execution for most cases.
fn register_unified_datetime_functions(registry: &mut UnifiedFunctionRegistry) {
    use crate::unified_implementations::datetime::*;

    // Register temporal functions
    let _ = registry.register(UnifiedNowFunction::new());
    let _ = registry.register(UnifiedTodayFunction::new());
    let _ = registry.register(UnifiedTimeOfDayFunction::new());

    // Register boundary calculation functions
    let _ = registry.register(UnifiedLowBoundaryFunction::new());
    let _ = registry.register(UnifiedHighBoundaryFunction::new());

    // Register type conversion functions
    let _ = registry.register(UnifiedConvertsToDateFunction::new());
    let _ = registry.register(UnifiedToDateFunction::new());
    let _ = registry.register(UnifiedConvertsToDateTimeFunction::new());
    let _ = registry.register(UnifiedToDateTimeFunction::new());
    let _ = registry.register(UnifiedConvertsToTimeFunction::new());
}

/// Register unified FHIR-specific functions
///
/// This is the seventh phase of the migration - FHIR-specific functions provide
/// specialized operations for FHIR resources and data types.
fn register_unified_fhir_functions(registry: &mut UnifiedFunctionRegistry) {
    use crate::unified_implementations::fhir::*;

    // Register resource reference resolution functions
    let _ = registry.register(UnifiedResolveFunction::new());

    // Register extension functions
    let _ = registry.register(UnifiedExtensionFunction::new());

    // Register validation functions
    let _ = registry.register(UnifiedConformsToFunction::new());

    // Register quantity comparison functions
    let _ = registry.register(UnifiedComparableFunction::new());
}

/// Register unified utility functions
///
/// This is the eighth phase of the migration - utility functions provide
/// conditional logic, debugging, and variable management capabilities.
fn register_unified_utility_functions(registry: &mut UnifiedFunctionRegistry) {
    use crate::unified_implementations::utility::*;

    // Register conditional functions
    let _ = registry.register(UnifiedIifFunction::new());

    // Register debugging functions
    let _ = registry.register(UnifiedTraceFunction::new());

    // Register value checking functions
    let _ = registry.register(UnifiedHasValueFunction::new());

    // Register variable management functions
    let _ = registry.register(UnifiedDefineVariableFunction::new());

    // Note: repeat() function is complex and requires full expression evaluation context
    // It will be implemented in a future phase when the evaluator integration is complete
    // let _ = registry.register(UnifiedRepeatFunction::new()); // Re-disabled - needs rewrite for current architecture
}

/// Register unified filtering functions
///
/// This is the ninth phase of the migration - filtering functions provide
/// essential collection filtering and projection capabilities.
fn register_unified_filtering_functions(registry: &mut UnifiedFunctionRegistry) {
    use crate::unified_implementations::filtering::*;

    // Register projection and filtering functions
    let _ = registry.register(UnifiedWhereFunction::new());
    let _ = registry.register(UnifiedSelectFunction::new());

    // Register type-based filtering function (async due to FHIR schema requirements)
    let _ = registry.register(UnifiedOfTypeFunction::new());
}

/// Register unified tree navigation functions
///
/// This is the tenth phase of the migration - tree navigation functions provide
/// hierarchical data traversal capabilities.
fn register_unified_tree_navigation_functions(registry: &mut UnifiedFunctionRegistry) {
    use crate::unified_implementations::tree_navigation::*;

    // Register tree traversal functions
    let _ = registry.register(UnifiedChildrenFunction::new());
    let _ = registry.register(UnifiedDescendantsFunction::new());
}

/// Register unified aggregate functions
///
/// This is the eleventh phase of the migration - aggregate functions provide
/// mathematical operations on collections.
fn register_unified_aggregate_functions(registry: &mut UnifiedFunctionRegistry) {
    use crate::unified_implementations::aggregates::*;

    // TODO: Replace with enhanced aggregate function with proper lambda support
    // For now, keep the old implementation until we can properly integrate lambda evaluation
    let _ = registry.register(UnifiedAggregateFunction::new());

    // Register mathematical aggregation functions
    let _ = registry.register(UnifiedSumFunction::new());
    let _ = registry.register(UnifiedAvgFunction::new());
    let _ = registry.register(UnifiedMinFunction::new());
    let _ = registry.register(UnifiedMaxFunction::new());
}

/// Register unified string extended functions
///
/// This is the twelfth phase of the migration - extended string functions provide
/// advanced text processing capabilities including regex operations.
fn register_unified_string_extended_functions(registry: &mut UnifiedFunctionRegistry) {
    use crate::unified_implementations::string_extended::*;

    // Register advanced string processing functions
    let _ = registry.register(UnifiedMatchesFunction::new());
    let _ = registry.register(UnifiedReplaceFunction::new());
    let _ = registry.register(UnifiedReplaceMatchesFunction::new());
    let _ = registry.register(UnifiedSplitFunction::new());
    let _ = registry.register(UnifiedJoinFunction::new());
    let _ = registry.register(UnifiedEncodeFunction::new());
    let _ = registry.register(UnifiedDecodeFunction::new());
}

/// Register unified type checking functions
///
/// This is the thirteenth phase of the migration - type checking functions provide
/// type inspection and casting capabilities for FHIRPath expressions.
fn register_unified_type_checking_functions(registry: &mut UnifiedFunctionRegistry) {
    use crate::unified_implementations::type_checking::*;

    // Register type checking and casting functions
    let _ = registry.register(UnifiedIsFunction::new());
    let _ = registry.register(UnifiedTypeFunction::new());
    let _ = registry.register(UnifiedAsFunction::new());
}

/// Register unified CDA functions
///
/// This registers CDA-specific extension functions for Clinical Document Architecture support.
fn register_unified_cda_functions(registry: &mut UnifiedFunctionRegistry) {
    use crate::unified_implementations::cda::*;

    // Register CDA-specific functions
    let _ = registry.register(UnifiedHasTemplateIdOfFunction::new());
}

// Re-export from workspace crates for convenience
pub use octofhir_fhirpath_ast::{BinaryOperator, ExpressionNode, UnaryOperator};
pub use octofhir_fhirpath_core::{FhirPathError, Result};
pub use octofhir_fhirpath_model::{FhirPathValue, ModelProvider};

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::function::EvaluationContext;

    #[test]
    fn test_unified_registry_integration() {
        let registry = create_unified_registry();

        // Verify all 21 collection functions are registered
        assert!(registry.contains("count"));
        assert!(registry.contains("empty"));
        assert!(registry.contains("exists"));
        assert!(registry.contains("first"));
        assert!(registry.contains("last"));
        assert!(registry.contains("single"));
        assert!(registry.contains("all"));
        assert!(registry.contains("any"));
        assert!(registry.contains("distinct"));
        assert!(registry.contains("take"));
        assert!(registry.contains("skip"));
        assert!(registry.contains("tail"));
        assert!(registry.contains("sort"));
        assert!(registry.contains("indexOf"));
        assert!(registry.contains("intersect"));
        assert!(registry.contains("flatten"));
        assert!(registry.contains("exclude"));
        assert!(registry.contains("combine"));
        assert!(registry.contains("subsetOf"));
        assert!(registry.contains("supersetOf"));
        assert!(registry.contains("union"));

        // Verify all 7 string functions are registered
        assert!(registry.contains("length"));
        assert!(registry.contains("substring"));
        assert!(registry.contains("contains"));
        assert!(registry.contains("startsWith"));
        assert!(registry.contains("endsWith"));
        assert!(registry.contains("upper"));
        assert!(registry.contains("lower"));

        // Verify all 8 math functions are registered
        assert!(registry.contains("abs"));
        assert!(registry.contains("ceiling"));
        assert!(registry.contains("floor"));
        assert!(registry.contains("round"));
        assert!(registry.contains("truncate"));
        assert!(registry.contains("sqrt"));
        assert!(registry.contains("exp"));
        assert!(registry.contains("ln"));

        // Verify all 6 type conversion functions are registered
        assert!(registry.contains("toString"));
        assert!(registry.contains("toInteger"));
        assert!(registry.contains("toDecimal"));
        assert!(registry.contains("toBoolean"));
        assert!(registry.contains("convertsToString"));
        assert!(registry.contains("convertsToInteger"));

        // Verify all 5 boolean functions are registered
        assert!(registry.contains("allTrue"));
        assert!(registry.contains("anyFalse"));
        assert!(registry.contains("allFalse"));
        assert!(registry.contains("not"));
        assert!(registry.contains("implies"));

        // Verify all 5 datetime functions are registered
        assert!(registry.contains("now"));
        assert!(registry.contains("today"));
        assert!(registry.contains("timeOfDay"));
        assert!(registry.contains("lowBoundary"));
        assert!(registry.contains("highBoundary"));

        // Verify all 4 FHIR-specific functions are registered
        assert!(registry.contains("resolve"));
        assert!(registry.contains("extension"));
        assert!(registry.contains("conformsTo"));
        assert!(registry.contains("comparable"));

        // Verify all 5 utility functions are registered
        assert!(registry.contains("iif"));
        assert!(registry.contains("trace"));
        assert!(registry.contains("hasValue"));
        assert!(registry.contains("defineVariable"));
        assert!(registry.contains("repeat"));

        // Verify all 3 filtering functions are registered
        assert!(registry.contains("where"));
        assert!(registry.contains("select"));
        assert!(registry.contains("ofType"));

        // Verify all 2 tree navigation functions are registered
        assert!(registry.contains("children"));
        assert!(registry.contains("descendants"));

        // Verify all 4 aggregate functions are registered
        assert!(registry.contains("sum"));
        assert!(registry.contains("avg"));
        assert!(registry.contains("min"));
        assert!(registry.contains("max"));

        // Verify all 7 string extended functions are registered
        assert!(registry.contains("matches"));
        assert!(registry.contains("replace"));
        assert!(registry.contains("replaceMatches"));
        assert!(registry.contains("split"));
        assert!(registry.contains("join"));
        assert!(registry.contains("encode"));
        assert!(registry.contains("decode"));

        // Verify all 3 type checking functions are registered
        assert!(registry.contains("is"));
        assert!(registry.contains("type"));
        assert!(registry.contains("as"));

        // Verify CDA functions are registered
        assert!(registry.contains("hasTemplateIdOf"));

        // Test count function
        let test_collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ]);

        let context = EvaluationContext::new(test_collection.clone());
        let result = registry
            .evaluate_function_sync("count", &[], &context)
            .unwrap();

        // Verify count returns collection with integer 3
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            if let Some(FhirPathValue::Integer(count)) = items.get(0) {
                assert_eq!(*count, 3);
            } else {
                panic!("Expected integer result from count function");
            }
        } else {
            panic!("Expected collection result from count function");
        }

        // Test first function
        let result = registry
            .evaluate_function_sync("first", &[], &context)
            .unwrap();
        assert_eq!(result, FhirPathValue::Integer(1));

        // Test take function with argument
        let args = vec![FhirPathValue::Integer(2)];
        let result = registry
            .evaluate_function_sync("take", &args, &context)
            .unwrap();
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 2);
            assert_eq!(items.get(0), Some(&FhirPathValue::Integer(1)));
            assert_eq!(items.get(1), Some(&FhirPathValue::Integer(2)));
        } else {
            panic!("Expected collection result from take function");
        }

        // Test string function
        let test_string = FhirPathValue::String("Hello World".into());
        let context = EvaluationContext::new(test_string.clone());
        let result = registry
            .evaluate_function_sync("length", &[], &context)
            .unwrap();
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Integer(11)));
        } else {
            panic!("Expected collection result from length function");
        }

        // Test contains function
        let args = vec![FhirPathValue::String("World".into())];
        let result = registry
            .evaluate_function_sync("contains", &args, &context)
            .unwrap();
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Boolean(true)));
        } else {
            panic!("Expected collection result from contains function");
        }

        // Test math function
        let test_number = FhirPathValue::Integer(-5);
        let context = EvaluationContext::new(test_number.clone());
        let result = registry
            .evaluate_function_sync("abs", &[], &context)
            .unwrap();
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Integer(5)));
        } else {
            panic!("Expected collection result from abs function");
        }

        // Test math function with decimal
        let test_decimal = FhirPathValue::Decimal({
            use rust_decimal::prelude::FromPrimitive;
            rust_decimal::Decimal::from_f64(3.7).unwrap()
        });
        let context = EvaluationContext::new(test_decimal.clone());
        let result = registry
            .evaluate_function_sync("floor", &[], &context)
            .unwrap();
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Integer(3)));
        } else {
            panic!("Expected collection result from floor function");
        }

        // Test type conversion function
        let test_number = FhirPathValue::Integer(42);
        let context = EvaluationContext::new(test_number.clone());
        let result = registry
            .evaluate_function_sync("toString", &[], &context)
            .unwrap();
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::String("42".into())));
        } else {
            panic!("Expected collection result from toString function");
        }

        // Test validation function
        let test_string = FhirPathValue::String("not-a-number".into());
        let context = EvaluationContext::new(test_string.clone());
        let result = registry
            .evaluate_function_sync("convertsToInteger", &[], &context)
            .unwrap();
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Boolean(false)));
        } else {
            panic!("Expected collection result from convertsToInteger function");
        }

        // Test boolean function - allTrue
        let test_booleans = FhirPathValue::collection(vec![
            FhirPathValue::Boolean(true),
            FhirPathValue::Boolean(true),
            FhirPathValue::Boolean(true),
        ]);
        let context = EvaluationContext::new(test_booleans.clone());
        let result = registry
            .evaluate_function_sync("allTrue", &[], &context)
            .unwrap();
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Boolean(true)));
        } else {
            panic!("Expected collection result from allTrue function");
        }

        // Test boolean function - not
        let test_boolean = FhirPathValue::Boolean(true);
        let context = EvaluationContext::new(test_boolean.clone());
        let result = registry
            .evaluate_function_sync("not", &[], &context)
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test datetime function - now
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let result = registry
            .evaluate_function_sync("now", &[], &context)
            .unwrap();
        match result {
            FhirPathValue::DateTime(_) => {
                // Success - got a datetime
            }
            _ => panic!("Expected DateTime result from now function"),
        }

        // Test datetime function - today
        let result = registry
            .evaluate_function_sync("today", &[], &context)
            .unwrap();
        match result {
            FhirPathValue::Date(_) => {
                // Success - got a date
            }
            _ => panic!("Expected Date result from today function"),
        }

        // Test boundary function - lowBoundary
        let test_decimal = FhirPathValue::Decimal({
            use rust_decimal::prelude::FromPrimitive;
            rust_decimal::Decimal::from_f64(2.58).unwrap()
        });
        let context = EvaluationContext::new(test_decimal.clone());
        let result = registry
            .evaluate_function_sync("lowBoundary", &[], &context)
            .unwrap();
        match result {
            FhirPathValue::Decimal(_) => {
                // Success - got a decimal
            }
            _ => panic!("Expected Decimal result from lowBoundary function"),
        }

        println!(
            "✅ Integration test passed: unified registry working correctly with {} functions",
            registry.get_stats().total_functions
        );
    }

    #[test]
    fn test_registry_stats() {
        let registry = create_unified_registry();
        let stats = registry.get_stats();

        // Verify we have 82 migrated functions (21 collection + 7 string + 9 math + 6 type conversion + 5 boolean + 5 datetime + 4 FHIR + 5 utility + 3 filtering + 2 tree navigation + 4 aggregate + 7 string extended + 3 type checking + 1 CDA)
        assert_eq!(stats.total_functions, 82);
        assert_eq!(stats.sync_functions, 73); // Most are sync (added 9 sync functions: subsetOf, supersetOf, union, sort, implies, timeOfDay, replaceMatches, encode, decode, hasTemplateIdOf, log)
        assert_eq!(stats.async_functions, 9); // resolve, conformsTo, repeat, where, select, ofType, is, type, as are async

        println!("✅ Registry stats test passed");
        println!("   Total functions: {}", stats.total_functions);
        println!("   Sync functions: {}", stats.sync_functions);
        println!("   Async functions: {}", stats.async_functions);
    }
}

/// Create standard registries for FHIRPath evaluation
///
/// Returns a tuple of (UnifiedFunctionRegistry, UnifiedOperatorRegistry) with all built-in
/// functions and operators. This function is used by the evaluator engine.
pub fn create_standard_registries() -> (UnifiedFunctionRegistry, UnifiedOperatorRegistry) {
    // Create unified function registry with all built-in functions (82+ functions)
    let function_registry = create_unified_registry();

    // Create unified operator registry with all built-in operators
    let operator_registry = create_unified_operator_registry();

    (function_registry, operator_registry)
}
