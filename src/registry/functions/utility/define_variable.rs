//! defineVariable() function - defines a variable in scope

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{
    EvaluationContext, FhirPathFunction, FunctionError, FunctionResult,
};
use crate::registry::signature::{FunctionSignature, ParameterInfo};

/// defineVariable() function - defines a variable in scope
pub struct DefineVariableFunction;

impl FhirPathFunction for DefineVariableFunction {
    fn name(&self) -> &str {
        "defineVariable"
    }
    fn human_friendly_name(&self) -> &str {
        "Define Variable"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "defineVariable",
                vec![
                    ParameterInfo::required("name", TypeInfo::String),
                    ParameterInfo::optional("value", TypeInfo::Any),
                ],
                TypeInfo::Any,
            )
        });
        &SIG
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        // defineVariable is handled specially in the engine via MethodCall processing
        // This function implementation is a fallback that should not normally be called
        // when used as a method call (.defineVariable())

        if args.is_empty() || args.len() > 2 {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 1,
                max: Some(2),
                actual: args.len(),
            });
        }

        let var_name = match &args[0] {
            FhirPathValue::String(s) => s.clone(),
            _ => {
                return Err(FunctionError::InvalidArgumentType {
                    name: self.name().to_string(),
                    index: 0,
                    expected: "String".to_string(),
                    actual: format!("{:?}", args[0]),
                });
            }
        };

        let _var_value = if args.len() == 2 {
            args[1].clone()
        } else {
            // If no value provided, use current context
            context.input.clone()
        };

        // Since this is a standalone function call (not method call),
        // we cannot modify the context. Return the input unchanged.
        // The actual variable definition happens in the engine's MethodCall handling.

        // For validation, check if variable name is reserved
        if matches!(
            var_name.as_str(),
            "$this" | "$" | "$$" | "$resource" | "$total" | "context"
        ) {
            // Return empty for reserved variable names as per spec
            return Ok(FhirPathValue::Empty);
        }

        Ok(context.input.clone())
    }
}
