//! Lambda function implementations with proper variable scoping
//! 
//! This module implements FHIRPath lambda functions (`where()`, `aggregate()`, `select()`, etc.)
//! with proper variable scoping and FHIRPath specification compliance.

use super::{FunctionCategory, FunctionContext, FunctionRegistry};
use crate::core::{FhirPathError, FhirPathValue, Result};

impl FunctionRegistry {
    /// Register all lambda functions with proper variable scoping
    pub fn register_lambda_functions(&self) -> Result<()> {
        self.register_where_function()?;
        self.register_select_function()?;
        self.register_aggregate_function()?;
        self.register_define_variable_function()?;
        self.register_all_function()?;
        self.register_exists_function()?;
        Ok(())
    }

    /// Register where() function with proper variable scoping
    /// where() function filters collection based on lambda expression
    fn register_where_function(&self) -> Result<()> {
        use crate::registry::builder::FunctionBuilder;
        
        let builder = FunctionBuilder::new("where", FunctionCategory::Collection)
            .description("Returns items from the collection where the given expression evaluates to true")
            .parameter("criteria", Some("expression".to_string()), false, "Boolean expression to filter by")
            .return_type("collection")
            .example("Patient.name.where(use = 'official')")
            .example("Bundle.entry.where(resource.active = true)")
            .example("Bundle.entry.where(resource.resourceType = 'Patient')")
            .example("(1 | 2 | 3).where($this > 1)");
        
        // Note: where() function is implemented directly in the evaluator
        // This registry entry is for metadata/documentation purposes only
        let metadata_only_impl = std::sync::Arc::new(|_context: &FunctionContext| -> Result<FhirPathValue> {
            // This should never be called since evaluator handles lambda functions directly
            unreachable!("where() function should be handled by evaluator, not registry")
        });

        builder.register_sync(self, metadata_only_impl)
    }

    /// Register select() function with type preservation
    /// select() function transforms each item in collection
    fn register_select_function(&self) -> Result<()> {
        use crate::registry::builder::FunctionBuilder;
        
        let builder = FunctionBuilder::new("select", FunctionCategory::Collection)
            .description("Projects each item in the collection through the given expression")
            .parameter("projection", Some("expression".to_string()), false, "Expression to apply to each item")
            .return_type("collection")
            .example("Patient.name.select(family + ', ' + given.first())")
            .example("Bundle.entry.select(resource.id)")
            .example("Bundle.entry.select(resource)")
            .example("(1 | 2 | 3).select($this * 2)");
        
        // Note: select() function is implemented directly in the evaluator
        let metadata_only_impl = std::sync::Arc::new(|_context: &FunctionContext| -> Result<FhirPathValue> {
            unreachable!("select() function should be handled by evaluator, not registry")
        });

        builder.register_sync(self, metadata_only_impl)
    }

    /// Register aggregate() function with type inference from init parameter
    /// aggregate() function performs aggregation over collection
    fn register_aggregate_function(&self) -> Result<()> {
        use crate::registry::builder::FunctionBuilder;
        
        let builder = FunctionBuilder::new("aggregate", FunctionCategory::Collection)
            .description("Perform aggregation over collection with type inference from init parameter")
            .parameter("aggregator", Some("expression".to_string()), false, "Expression to evaluate for each item")
            .parameter("init", Some("any".to_string()), true, "Initial value for aggregation (optional)")
            .return_type("any")
            .does_not_propagate_empty() // aggregate() returns result even with empty collection
            .example("(1 | 2 | 3).aggregate($total + $this, 0)")
            .example("Patient.name.aggregate($total + given.first(), '')")
            .example("Bundle.entry.aggregate($total.count() + 1, {})");
        
        // Note: aggregate() function is implemented directly in the evaluator
        let metadata_only_impl = std::sync::Arc::new(|_context: &FunctionContext| -> Result<FhirPathValue> {
            unreachable!("aggregate() function should be handled by evaluator, not registry")
        });

        builder.register_sync(self, metadata_only_impl)
    }

    /// Register defineVariable() function with redefinition protection
    /// defineVariable() function defines variables with FHIRPath spec compliance
    fn register_define_variable_function(&self) -> Result<()> {
        use crate::registry::builder::FunctionBuilder;
        
        let builder = FunctionBuilder::new("defineVariable", FunctionCategory::Utility)
            .description("Define variable in evaluation context with redefinition protection per FHIRPath §1.5.10.3")
            .parameter("name", Some("string".to_string()), false, "Variable name (can be literal or expression)")
            .parameter("value", Some("any".to_string()), true, "Variable value (optional, defaults to input)")
            .return_type("any")
            .does_not_propagate_empty() // defineVariable() returns input value, not empty
            .example("Patient.defineVariable('name', name.first().given.first())")
            .example("(1 | 2 | 3).defineVariable('sum').select($this + %sum)")
            .example("Bundle.entry.defineVariable('total', count()).resource");
        
        let implementation = std::sync::Arc::new(|_context: &FunctionContext| -> Result<FhirPathValue> {
            // For now, this is a placeholder that shows the signature
            // The actual implementation requires AST evaluation and context modification
            // which is handled by the composite evaluator
            Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "defineVariable() function requires context modification - use composite evaluator".to_string()
            ))
        });
        
        builder.register_sync(self, implementation)
    }

    /// Register all() function with proper lambda context
    /// all() function checks if all items satisfy condition
    fn register_all_function(&self) -> Result<()> {
        use crate::registry::builder::FunctionBuilder;
        
        let builder = FunctionBuilder::new("all", FunctionCategory::Collection)
            .description("Returns true if the given expression evaluates to true for all items in the collection")
            .parameter("criteria", Some("expression".to_string()), false, "Boolean expression to evaluate for each item")
            .return_type("boolean")
            .does_not_propagate_empty() // all() on empty collection returns true per FHIRPath spec
            .example("Patient.name.all(use = 'official')")
            .example("Bundle.entry.all(resource.exists())")
            .example("Patient.name.all(family.exists())")
            .example("(1 | 2 | 3).all($this > 0)");
        
        let implementation = std::sync::Arc::new(|_context: &FunctionContext| -> Result<FhirPathValue> {
            // This is a placeholder - all() requires AST evaluation for lambda expressions
            Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "all() function requires lambda expression evaluation - use composite evaluator".to_string()
            ))
        });
        
        builder.register_sync(self, implementation)
    }

    /// Register exists() function with proper lambda context
    /// exists() function checks if any items satisfy condition
    fn register_exists_function(&self) -> Result<()> {
        use crate::registry::builder::FunctionBuilder;
        
        let builder = FunctionBuilder::new("exists", FunctionCategory::Logic)
            .description("Check if any items in collection satisfy the condition")
            .parameter("condition", Some("expression".to_string()), true, "Boolean expression to evaluate for each item (optional)")
            .return_type("boolean")
            .does_not_propagate_empty() // exists() always returns boolean, never propagates empty
            .example("Patient.name.exists()")
            .example("Patient.name.exists(use = 'official')")
            .example("Bundle.entry.exists(resource.resourceType = 'Patient')")
            .example("(1 | 2 | 3).exists($this = 2)");
        
        let implementation = std::sync::Arc::new(|context: &FunctionContext| -> Result<FhirPathValue> {
            // Handle simple exists() without condition (non-lambda version)
            if context.arguments.is_empty() {
                let has_items = !context.input.is_empty();
                return Ok(FhirPathValue::Boolean(has_items));
            }
            
            // Lambda version with condition is handled by composite evaluator
            // This should not be reached in normal operation as the composite evaluator
            // intercepts lambda functions before they reach the registry
            Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "exists() with condition requires lambda expression evaluation - use composite evaluator".to_string()
            ))
        });
        
        builder.register_sync(self, implementation)
    }
}