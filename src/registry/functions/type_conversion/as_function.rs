//! as() function - type casting function

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{
    EvaluationContext, FhirPathFunction, FunctionError, FunctionResult,
};
use crate::registry::signature::{FunctionSignature, ParameterInfo};
use chrono::TimeZone;
use rust_decimal::prelude::ToPrimitive;

/// as() function - performs type casting
pub struct AsFunction;

impl FhirPathFunction for AsFunction {
    fn name(&self) -> &str {
        "as"
    }

    fn human_friendly_name(&self) -> &str {
        "Type Cast"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "as",
                vec![ParameterInfo::required("type", TypeInfo::String)],
                TypeInfo::Any,
            )
        });
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // as() is a pure type conversion function
    }

    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        // Get the type name from the argument
        let type_name = match &args[0] {
            FhirPathValue::String(s) => s.as_str(),
            _ => {
                return Err(FunctionError::InvalidArgumentType {
                    name: self.name().to_string(),
                    index: 0,
                    expected: "String".to_string(),
                    actual: format!("{:?}", args[0]),
                });
            }
        };

        // Perform type casting based on the input value
        match (&context.input, type_name) {
            // Empty always returns empty
            (FhirPathValue::Empty, _) => Ok(FhirPathValue::Empty),

            // String casting - only if actually a string type
            (FhirPathValue::String(s), "string" | "String" | "System.String") => {
                Ok(FhirPathValue::String(s.clone()))
            }

            // Code type casting - in FHIRPath, code is a subtype of string
            // Since we store codes as strings, we accept string values for code type
            (FhirPathValue::String(s), "code" | "Code" | "FHIR.code") => {
                Ok(FhirPathValue::String(s.clone()))
            }

            // Integer casting
            (FhirPathValue::Integer(i), "integer" | "Integer" | "System.Integer") => {
                Ok(FhirPathValue::Integer(*i))
            }
            (FhirPathValue::Decimal(d), "integer" | "Integer" | "System.Integer") => {
                // Try to convert decimal to integer if it has no fractional part
                if d.fract().is_zero() {
                    if let Some(i) = d.to_i64() {
                        Ok(FhirPathValue::Integer(i))
                    } else {
                        Ok(FhirPathValue::Empty)
                    }
                } else {
                    Ok(FhirPathValue::Empty)
                }
            }
            (FhirPathValue::String(s), "integer" | "Integer" | "System.Integer") => {
                match s.parse::<i64>() {
                    Ok(i) => Ok(FhirPathValue::Integer(i)),
                    Err(_) => Ok(FhirPathValue::Empty),
                }
            }

            // Decimal casting
            (FhirPathValue::Decimal(d), "decimal" | "Decimal" | "System.Decimal") => {
                Ok(FhirPathValue::Decimal(*d))
            }
            (FhirPathValue::Integer(i), "decimal" | "Decimal" | "System.Decimal") => {
                Ok(FhirPathValue::Decimal(rust_decimal::Decimal::from(*i)))
            }
            (FhirPathValue::String(s), "decimal" | "Decimal" | "System.Decimal") => {
                match s.parse::<rust_decimal::Decimal>() {
                    Ok(d) => Ok(FhirPathValue::Decimal(d)),
                    Err(_) => Ok(FhirPathValue::Empty),
                }
            }

            // Boolean casting
            (FhirPathValue::Boolean(b), "boolean" | "Boolean" | "System.Boolean") => {
                Ok(FhirPathValue::Boolean(*b))
            }
            (FhirPathValue::String(s), "boolean" | "Boolean" | "System.Boolean") => {
                match s.as_str() {
                    "true" => Ok(FhirPathValue::Boolean(true)),
                    "false" => Ok(FhirPathValue::Boolean(false)),
                    _ => Ok(FhirPathValue::Empty),
                }
            }

            // Date casting
            (FhirPathValue::Date(d), "date" | "Date" | "System.Date") => {
                Ok(FhirPathValue::Date(*d))
            }
            (FhirPathValue::DateTime(dt), "date" | "Date" | "System.Date") => {
                Ok(FhirPathValue::Date(dt.date_naive()))
            }
            (FhirPathValue::String(s), "date" | "Date" | "System.Date") => {
                match chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                    Ok(d) => Ok(FhirPathValue::Date(d)),
                    Err(_) => Ok(FhirPathValue::Empty),
                }
            }

            // DateTime casting
            (FhirPathValue::DateTime(dt), "dateTime" | "DateTime" | "System.DateTime") => {
                Ok(FhirPathValue::DateTime(*dt))
            }
            (FhirPathValue::Date(d), "dateTime" | "DateTime" | "System.DateTime") => {
                let dt = d
                    .and_hms_opt(0, 0, 0)
                    .map(|naive| chrono::Utc.from_utc_datetime(&naive).fixed_offset());
                match dt {
                    Some(datetime) => Ok(FhirPathValue::DateTime(datetime)),
                    None => Ok(FhirPathValue::Empty),
                }
            }

            // Time casting
            (FhirPathValue::Time(t), "time" | "Time" | "System.Time") => {
                Ok(FhirPathValue::Time(*t))
            }
            (FhirPathValue::DateTime(dt), "time" | "Time" | "System.Time") => {
                Ok(FhirPathValue::Time(dt.time()))
            }

            // Quantity casting
            (FhirPathValue::Quantity(q), "Quantity" | "System.Quantity") => {
                Ok(FhirPathValue::Quantity(q.clone()))
            }

            // Collection handling - try to cast the collection
            (FhirPathValue::Collection(items), _) => {
                if items.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else if items.len() == 1 {
                    // Single item collection - cast the item
                    let item_context = EvaluationContext {
                        input: items.first().unwrap().clone(),
                        ..context.clone()
                    };
                    self.evaluate(args, &item_context)
                } else {
                    // Multiple items - return empty
                    Ok(FhirPathValue::Empty)
                }
            }

            // Default: if the type doesn't match, return empty
            _ => Ok(FhirPathValue::Empty),
        }
    }
}
