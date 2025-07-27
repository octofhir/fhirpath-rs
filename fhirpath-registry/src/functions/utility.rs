//! Utility functions

use crate::function::{FhirPathFunction, FunctionError, FunctionResult, EvaluationContext, LambdaEvaluationContext};
use crate::signature::{FunctionSignature, ParameterInfo};
use fhirpath_model::{FhirPathValue, TypeInfo};

/// iif() function - conditional expression (if-then-else)
pub struct IifFunction;

impl FhirPathFunction for IifFunction {
    fn name(&self) -> &str { "iif" }
    fn human_friendly_name(&self) -> &str { "If" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "iif",
                vec![
                    ParameterInfo::required("condition", TypeInfo::Boolean),
                    ParameterInfo::required("true_value", TypeInfo::Any),
                    ParameterInfo::optional("false_value", TypeInfo::Any),
                ],
                TypeInfo::Any,
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        
        let condition = match &args[0] {
            FhirPathValue::Boolean(b) => *b,
            _ => return Err(FunctionError::InvalidArgumentType {
                name: self.name().to_string(),
                index: 0,
                expected: "Boolean".to_string(),
                actual: format!("{:?}", args[0]),
            }),
        };
        
        if condition {
            Ok(args[1].clone())
        } else {
            Ok(args.get(2).cloned().unwrap_or(FhirPathValue::Empty))
        }
    }
}

/// trace() function - debugging function that logs and returns input
pub struct TraceFunction;

impl FhirPathFunction for TraceFunction {
    fn name(&self) -> &str { "trace" }
    fn human_friendly_name(&self) -> &str { "Trace" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "trace",
                vec![ParameterInfo::optional("name", TypeInfo::String)],
                TypeInfo::Any,
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        
        let name = if let Some(FhirPathValue::String(s)) = args.get(0) {
            s.clone()
        } else {
            "trace".to_string()
        };
        
        // In a real implementation, this would log to appropriate output
        eprintln!("{}: {:?}", name, context.input);
        
        Ok(context.input.clone())
    }
}

/// defineVariable() function - defines a variable in scope
pub struct DefineVariableFunction;

impl FhirPathFunction for DefineVariableFunction {
    fn name(&self) -> &str { "defineVariable" }
    fn human_friendly_name(&self) -> &str { "Define Variable" }
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
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        
        let _name = match &args[0] {
            FhirPathValue::String(s) => s,
            _ => return Err(FunctionError::InvalidArgumentType {
                name: self.name().to_string(),
                index: 0,
                expected: "String".to_string(),
                actual: format!("{:?}", args[0]),
            }),
        };
        
        let _value = args.get(1).cloned().unwrap_or_else(|| context.input.clone());
        
        // Variable definition would need to be handled at a higher level
        // For now, just return the input
        Ok(context.input.clone())
    }
}

/// repeat() function - repeats evaluation until no new results
pub struct RepeatFunction;

impl FhirPathFunction for RepeatFunction {
    fn name(&self) -> &str { "repeat" }
    fn human_friendly_name(&self) -> &str { "Repeat" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "repeat",
                vec![ParameterInfo::required("expression", TypeInfo::Any)],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        
        // Repeat would need lambda evaluation support
        Err(FunctionError::EvaluationError {
            name: self.name().to_string(),
            message: "repeat() requires lambda evaluation support".to_string(),
        })
    }
}

/// conformsTo() function - checks if resource conforms to profile
pub struct ConformsToFunction;

impl FhirPathFunction for ConformsToFunction {
    fn name(&self) -> &str { "conformsTo" }
    fn human_friendly_name(&self) -> &str { "Conforms To" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "conformsTo",
                vec![ParameterInfo::required("profile", TypeInfo::String)],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        
        let _profile = match &args[0] {
            FhirPathValue::String(s) => s,
            _ => return Err(FunctionError::InvalidArgumentType {
                name: self.name().to_string(),
                index: 0,
                expected: "String".to_string(),
                actual: format!("{:?}", args[0]),
            }),
        };
        
        // Profile conformance checking would need external validation
        // For now, return false
        Ok(FhirPathValue::Boolean(false))
    }
}