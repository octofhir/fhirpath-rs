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
        self.register_iif_function()?; // Keep metadata for function discovery
        self.register_and_function()?;
        self.register_or_function()?;
        self.register_xor_function()?;
        self.register_implies_function()?;
        // defineVariable() function is now implemented through lambda functions module
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
            description: "Conditional function with lazy evaluation: evaluates condition first, then only the selected branch (then/else). Provides short-circuit evaluation.",
            parameters: [
                "condition": Some("boolean".to_string()) => "The condition to evaluate",
                "then": Some("any".to_string()) => "Expression to evaluate if condition is true",
                "else": Some("any".to_string()) => "Expression to evaluate if condition is false (optional)"
            ],
            return_type: "any",
            examples: [
                "iif(Patient.active, 'active', 'inactive')",
                "iif(true, 'yes', 'no')",
                "iif(false, 'yes')",
                "iif(Patient.gender = 'male', Patient.name.given[0], Patient.name.family)"
            ],
            implementation: |_context: &FunctionContext| -> Result<FhirPathValue> {
                // iif() is handled by the evaluator with lazy evaluation and short-circuiting
                // This implementation should not be called - the evaluator intercepts iif() calls
                Err(FhirPathError::evaluation_error(
                    FP0053,
                    "iif() function uses lazy evaluation - handled directly by evaluator".to_string()
                ))
            }
        )
    }

    fn register_and_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "and",
            category: FunctionCategory::Logic,
            description: "Logical AND operation. Returns true if both operands are true, false if either is false, empty if either is empty and the other is not false",
            parameters: ["right": Some("boolean".to_string()) => "Right operand"],
            return_type: "boolean",
            examples: [
                "true and true",
                "false and true",
                "Patient.active and Patient.name.exists()"
            ],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.len() != 1 || context.arguments.len() != 1 {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "and() requires exactly two boolean operands".to_string()
                    ));
                }

                let left = context.input.first().unwrap();
                let right = context.arguments.first().unwrap();

                match (left, right) {
                    // True if both are true
                    (FhirPathValue::Boolean(true), FhirPathValue::Boolean(true)) => Ok(FhirPathValue::Boolean(true)),
                    // False if either is explicitly false
                    (FhirPathValue::Boolean(false), _) => Ok(FhirPathValue::Boolean(false)),
                    (_, FhirPathValue::Boolean(false)) => Ok(FhirPathValue::Boolean(false)),
                    // Empty if either is empty and the other is not false
                    (FhirPathValue::Empty, FhirPathValue::Boolean(true)) => Ok(FhirPathValue::empty()),
                    (FhirPathValue::Boolean(true), FhirPathValue::Empty) => Ok(FhirPathValue::empty()),
                    (FhirPathValue::Empty, FhirPathValue::Empty) => Ok(FhirPathValue::empty()),
                    // Type error for non-boolean operands
                    _ => Err(FhirPathError::evaluation_error(
                        FP0053,
                        "and() can only be applied to boolean values".to_string()
                    ))
                }
            }
        )
    }

    fn register_or_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "or",
            category: FunctionCategory::Logic,
            description: "Logical OR operation. Returns true if either operand is true, false if both are false, empty if either is empty and the other is not true",
            parameters: ["right": Some("boolean".to_string()) => "Right operand"],
            return_type: "boolean",
            examples: [
                "true or false",
                "false or false",
                "Patient.active or Patient.deceased.exists()"
            ],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.len() != 1 || context.arguments.len() != 1 {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "or() requires exactly two boolean operands".to_string()
                    ));
                }

                let left = context.input.first().unwrap();
                let right = context.arguments.first().unwrap();

                match (left, right) {
                    // True if either is true
                    (FhirPathValue::Boolean(true), _) => Ok(FhirPathValue::Boolean(true)),
                    (_, FhirPathValue::Boolean(true)) => Ok(FhirPathValue::Boolean(true)),
                    // False if both are explicitly false
                    (FhirPathValue::Boolean(false), FhirPathValue::Boolean(false)) => Ok(FhirPathValue::Boolean(false)),
                    // Empty if either is empty and the other is not true
                    (FhirPathValue::Empty, FhirPathValue::Boolean(false)) => Ok(FhirPathValue::empty()),
                    (FhirPathValue::Boolean(false), FhirPathValue::Empty) => Ok(FhirPathValue::empty()),
                    (FhirPathValue::Empty, FhirPathValue::Empty) => Ok(FhirPathValue::empty()),
                    // Type error for non-boolean operands
                    _ => Err(FhirPathError::evaluation_error(
                        FP0053,
                        "or() can only be applied to boolean values".to_string()
                    ))
                }
            }
        )
    }

    fn register_xor_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "xor",
            category: FunctionCategory::Logic,
            description: "Logical XOR operation. Returns true if exactly one operand is true, false if both are the same, empty if either is empty",
            parameters: ["right": Some("boolean".to_string()) => "Right operand"],
            return_type: "boolean",
            examples: [
                "true xor false",
                "false xor false",
                "Patient.active xor Patient.deceased"
            ],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.len() != 1 || context.arguments.len() != 1 {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "xor() requires exactly two boolean operands".to_string()
                    ));
                }

                let left = context.input.first().unwrap();
                let right = context.arguments.first().unwrap();

                match (left, right) {
                    // XOR logic: true if exactly one is true
                    (FhirPathValue::Boolean(true), FhirPathValue::Boolean(false)) => Ok(FhirPathValue::Boolean(true)),
                    (FhirPathValue::Boolean(false), FhirPathValue::Boolean(true)) => Ok(FhirPathValue::Boolean(true)),
                    (FhirPathValue::Boolean(true), FhirPathValue::Boolean(true)) => Ok(FhirPathValue::Boolean(false)),
                    (FhirPathValue::Boolean(false), FhirPathValue::Boolean(false)) => Ok(FhirPathValue::Boolean(false)),
                    // Empty if either is empty
                    (FhirPathValue::Empty, _) => Ok(FhirPathValue::empty()),
                    (_, FhirPathValue::Empty) => Ok(FhirPathValue::empty()),
                    // Type error for non-boolean operands
                    _ => Err(FhirPathError::evaluation_error(
                        FP0053,
                        "xor() can only be applied to boolean values".to_string()
                    ))
                }
            }
        )
    }

    fn register_implies_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "implies",
            category: FunctionCategory::Logic,
            description: "Logical implication. Returns false only when left is true and right is false, true otherwise, empty if either is empty and result cannot be determined",
            parameters: ["right": Some("boolean".to_string()) => "Right operand (consequent)"],
            return_type: "boolean",
            examples: [
                "true implies false",
                "false implies true",
                "Patient.active implies Patient.name.exists()"
            ],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.len() != 1 || context.arguments.len() != 1 {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "implies() requires exactly two boolean operands".to_string()
                    ));
                }

                let left = context.input.first().unwrap();
                let right = context.arguments.first().unwrap();

                match (left, right) {
                    // Implication logic: false only when true implies false
                    (FhirPathValue::Boolean(true), FhirPathValue::Boolean(false)) => Ok(FhirPathValue::Boolean(false)),
                    (FhirPathValue::Boolean(true), FhirPathValue::Boolean(true)) => Ok(FhirPathValue::Boolean(true)),
                    (FhirPathValue::Boolean(false), _) => Ok(FhirPathValue::Boolean(true)),
                    // Handle empty values according to FHIRPath three-valued logic
                    (FhirPathValue::Boolean(true), FhirPathValue::Empty) => Ok(FhirPathValue::empty()),
                    (FhirPathValue::Boolean(false), FhirPathValue::Empty) => Ok(FhirPathValue::Boolean(true)),
                    (FhirPathValue::Empty, FhirPathValue::Boolean(true)) => Ok(FhirPathValue::Boolean(true)),
                    (FhirPathValue::Empty, FhirPathValue::Boolean(false)) => Ok(FhirPathValue::empty()),
                    (FhirPathValue::Empty, FhirPathValue::Empty) => Ok(FhirPathValue::empty()),
                    // Type error for non-boolean operands
                    _ => Err(FhirPathError::evaluation_error(
                        FP0053,
                        "implies() can only be applied to boolean values".to_string()
                    ))
                }
            }
        )
    }

}
