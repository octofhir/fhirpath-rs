//! Boolean logic functions for FHIRPath
//!
//! This module implements FHIRPath boolean logic functions according to the specification.
//! Reference: https://build.fhir.org/ig/HL7/FHIRPath/

use super::{FunctionCategory, FunctionContext, FunctionRegistry};
use crate::core::error_code::FP0053;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::register_function;

impl FunctionRegistry {
    pub fn register_logic_functions(&self) -> Result<()> {
        self.register_not_function()?;
        self.register_iif_function()?;
        self.register_define_variable_function()?;
        Ok(())
    }

    fn register_not_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "not",
            category: FunctionCategory::Logic,
            description: "Returns true if the input is false, false if the input is true, and empty if the input is empty",
            parameters: [],
            return_type: "boolean",
            examples: [
                "true.not()",
                "false.not()",
                "Patient.active.not()"
            ],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.len() == 0 {
                    return Ok(FhirPathValue::empty());
                }

                if context.input.len() != 1 {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "not() can only be called on a single value".to_string()
                    ));
                }

                match context.input.first() {
                    Some(FhirPathValue::Boolean(b)) => Ok(FhirPathValue::Boolean(!b)),
                    Some(FhirPathValue::Empty) => Ok(FhirPathValue::empty()),
                    // Treat any non-empty non-boolean as truthy per common FHIRPath semantics for truthiness
                    Some(_) => Ok(FhirPathValue::Boolean(false)),
                    None => Ok(FhirPathValue::empty())
                }
            }
        )
    }

    fn register_iif_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "iif",
            category: FunctionCategory::Logic,
            description: "Conditional function: returns the 'then' value if condition is true, 'else' value if false, empty if condition is empty",
            parameters: [
                "condition": Some("boolean".to_string()) => "The condition to evaluate",
                "then": Some("any".to_string()) => "Value to return if condition is true",
                "else": Some("any".to_string()) => "Value to return if condition is false (optional)"
            ],
            return_type: "any",
            examples: [
                "iif(Patient.active, 'active', 'inactive')",
                "iif(true, 'yes', 'no')",
                "iif(false, 'yes')"
            ],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.arguments.len() < 2 || context.arguments.len() > 3 {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "iif() requires 2 or 3 arguments".to_string()
                    ));
                }

                let args = context.arguments.cloned_collection();
                let condition = &args[0];
                let then_value = &args[1];
                let else_value = if args.len() > 2 {
                    Some(&args[2])
                } else {
                    None
                };

                match condition {
                    FhirPathValue::Boolean(true) => Ok(then_value.clone()),
                    FhirPathValue::Boolean(false) => {
                        if let Some(else_val) = else_value {
                            Ok(else_val.clone())
                        } else {
                            Ok(FhirPathValue::empty())
                        }
                    },
                    FhirPathValue::Empty => {
                        // According to FHIRPath spec, empty collections are falsy in boolean context
                        // so iif({}, then, else) should return else, not empty
                        if let Some(else_val) = else_value {
                            Ok(else_val.clone())
                        } else {
                            Ok(FhirPathValue::empty())
                        }
                    },
                    _ => Err(FhirPathError::evaluation_error(
                        FP0053,
                        "iif() first argument must be a boolean or empty".to_string()
                    ))
                }
            }
        )
    }

    fn register_define_variable_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "defineVariable",
            category: FunctionCategory::Logic,
            description: "Defines a variable in the current scope that can be accessed using %name syntax (placeholder implementation)",
            parameters: ["name": Some("string".to_string()) => "Variable name to define"],
            return_type: "any",
            examples: [
                "Patient.defineVariable('pat').name.family",
                "Bundle.entry.resource.defineVariable('res').id"
            ],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.arguments.len() < 1 || context.arguments.len() > 2 {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "defineVariable() requires 1 or 2 arguments (variable name and optional expression)".to_string()
                    ));
                }

                let _var_name = match context.arguments.first() {
                    Some(FhirPathValue::String(s)) => s,
                    Some(_) => {
                        return Err(FhirPathError::evaluation_error(
                            FP0053,
                            "defineVariable() variable name must be a string".to_string()
                        ));
                    }
                    None => {
                        return Err(FhirPathError::evaluation_error(
                            FP0053,
                            "defineVariable() requires exactly one argument (variable name)".to_string()
                        ));
                    }
                };

                // Placeholder implementation: defineVariable requires full variable scoping system
                // For now, just pass through the input unchanged
                // In full implementation, this would store the input value in a variable scope
                // that can be accessed later using %variableName

                match context.input.first() {
                    Some(value) => Ok(value.clone()),
                    None => Ok(FhirPathValue::empty())
                }
            }
        )
    }
}
