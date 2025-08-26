//! Trace function implementation - sync version

use crate::signature::{FunctionSignature, ParameterType, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::Result;
use octofhir_fhirpath_model::FhirPathValue;

/// Trace function - debug output for expression evaluation
#[derive(Debug, Clone)]
pub struct TraceFunction;

impl TraceFunction {
    pub fn new() -> Self {
        Self
    }
}

impl SyncOperation for TraceFunction {
    fn name(&self) -> &'static str {
        "trace"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature {
                name: "trace",
                parameters: vec![ParameterType::Any], // Optional label parameter
                return_type: ValueType::Any,
                variadic: true, // Can take any number of arguments
            }
        });
        &SIGNATURE
    }

    fn execute(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        let label = if args.is_empty() {
            "trace".to_string()
        } else {
            args[0].as_string().unwrap_or("trace").to_string()
        };

        // Output trace information (configurable via environment)
        if std::env::var("FHIRPATH_TRACE").unwrap_or_default() == "true" {
            if args.len() <= 1 {
                // Simple trace with just the label
                eprintln!("TRACE [{}]: {:?}", label, context.input);
            } else {
                // Extended trace with additional arguments
                let additional_values: Vec<String> =
                    args[1..].iter().map(|arg| format!("{arg:?}")).collect();
                eprintln!(
                    "TRACE [{}]: {:?} | Additional: [{}]",
                    label,
                    context.input,
                    additional_values.join(", ")
                );
            }
        }

        // trace() returns its input unchanged
        Ok(context.input.clone())
    }

    fn validate_args(&self, _args: &[FhirPathValue]) -> Result<()> {
        // trace() can accept any number of arguments
        Ok(())
    }
}

impl Default for TraceFunction {
    fn default() -> Self {
        Self::new()
    }
}
