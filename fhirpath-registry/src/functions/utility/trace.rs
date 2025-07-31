//! trace() function - debugging function that logs and returns input

use crate::function::{
    EvaluationContext, FhirPathFunction, FunctionResult,
};
use crate::signature::{FunctionSignature, ParameterInfo};
use fhirpath_model::{FhirPathValue, TypeInfo};

/// trace() function - debugging function that logs and returns input
pub struct TraceFunction;

impl FhirPathFunction for TraceFunction {
    fn name(&self) -> &str {
        "trace"
    }
    fn human_friendly_name(&self) -> &str {
        "Trace"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "trace",
                vec![
                    ParameterInfo::required("name", TypeInfo::String),
                    ParameterInfo::optional("selector", TypeInfo::Any),
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
        self.validate_args(args)?;

        let _name = match &args[0] {
            FhirPathValue::String(s) => s.clone(),
            _ => "trace".to_string(),
        };

        // Check if there's a second argument (selector)
        let _value_to_trace = if args.len() > 1 {
            &args[1]
        } else {
            &context.input
        };

        // In a real implementation, this would log to appropriate output
        // For debugging: eprintln!("{}: {:?}", name, value_to_trace);

        // trace() function always returns the original input (context), not the traced value
        Ok(context.input.clone())
    }
}