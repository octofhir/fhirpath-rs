//! join() function - joins collection of strings

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{
    EvaluationContext, FhirPathFunction, FunctionError, FunctionResult,
};
use crate::registry::signature::{FunctionSignature, ParameterInfo};

/// join() function - joins collection of strings
pub struct JoinFunction;

impl FhirPathFunction for JoinFunction {
    fn name(&self) -> &str {
        "join"
    }
    fn human_friendly_name(&self) -> &str {
        "Join"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "join",
                vec![ParameterInfo::optional("separator", TypeInfo::String)],
                TypeInfo::String,
            )
        });
        &SIG
    }
    fn is_pure(&self) -> bool {
        true // join() is a pure string function
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let separator = match args.first() {
            Some(FhirPathValue::String(s)) => s.as_str(),
            Some(_) => {
                return Err(FunctionError::InvalidArgumentType {
                    name: self.name().to_string(),
                    index: 0,
                    expected: "String".to_string(),
                    actual: format!("{:?}", args[0]),
                });
            }
            None => "",
        };

        let items = context.input.clone().to_collection();
        let strings: Vec<String> = items
            .into_iter()
            .map(|item| match item {
                FhirPathValue::String(s) => s.clone(),
                FhirPathValue::Integer(i) => i.to_string(),
                FhirPathValue::Decimal(d) => d.to_string(),
                FhirPathValue::Boolean(b) => b.to_string(),
                FhirPathValue::Date(d) => d.to_string(),
                FhirPathValue::DateTime(dt) => dt.to_string(),
                FhirPathValue::Time(t) => t.to_string(),
                FhirPathValue::Quantity(q) => q.to_string(),
                FhirPathValue::Resource(r) => {
                    // Try to extract string value from FhirResource
                    match r.as_json() {
                        serde_json::Value::String(s) => s.clone(),
                        _ => format!("{r:?}"),
                    }
                }
                FhirPathValue::Empty => String::new(),
                FhirPathValue::Collection(_) => String::new(), // Empty collections become empty strings
                FhirPathValue::TypeInfoObject { namespace, name } => {
                    format!("{namespace}::{name}")
                }
            })
            .collect();

        Ok(FhirPathValue::String(strings.join(separator)))
    }
}
