//! isDistinct() function - returns true if the collection contains no duplicates

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{AsyncFhirPathFunction, EvaluationContext, FunctionResult};
use crate::registry::signature::FunctionSignature;
use async_trait::async_trait;
use std::collections::HashMap;

/// isDistinct() function - returns true if the collection contains no duplicates
pub struct IsDistinctFunction;

/// Discriminant key for fast duplicate detection
#[derive(Hash, Eq, PartialEq)]
enum ValueDiscriminant {
    Empty,
    Boolean(bool),
    Integer(i64),
    Decimal(String), // Use string representation for decimal
    String(String),
    Date(String),
    DateTime(String),
    Time(String),
    Quantity(String, Option<String>), // value and unit
    Collection(usize),                // Collection with size
    Resource(String),                 // Resource type name
}

impl IsDistinctFunction {
    /// Create a discriminant for fast equality checking
    fn create_discriminant(value: &FhirPathValue) -> ValueDiscriminant {
        match value {
            FhirPathValue::Empty => ValueDiscriminant::Empty,
            FhirPathValue::Boolean(b) => ValueDiscriminant::Boolean(*b),
            FhirPathValue::Integer(i) => ValueDiscriminant::Integer(*i),
            FhirPathValue::Decimal(d) => ValueDiscriminant::Decimal(d.to_string()),
            FhirPathValue::String(s) => ValueDiscriminant::String(s.clone()),
            FhirPathValue::Date(d) => ValueDiscriminant::Date(d.to_string()),
            FhirPathValue::DateTime(dt) => ValueDiscriminant::DateTime(dt.to_string()),
            FhirPathValue::Time(t) => ValueDiscriminant::Time(t.to_string()),
            FhirPathValue::Quantity(q) => {
                ValueDiscriminant::Quantity(q.value.to_string(), q.unit.clone())
            }
            // FhirPathValue::Code variant doesn't exist yet
            // FhirPathValue::Code(c) => ValueDiscriminant::Code(
            //     c.code.clone(),
            //     c.system.clone(),
            // ),
            FhirPathValue::Collection(items) => ValueDiscriminant::Collection(items.len()),
            FhirPathValue::Resource(r) => {
                ValueDiscriminant::Resource(r.resource_type().unwrap_or("Unknown").to_string())
            }
            FhirPathValue::TypeInfoObject { namespace, name } => {
                ValueDiscriminant::String(format!("TypeInfo({namespace}.{name})"))
            } // FhirPathValue::Lambda variant doesn't exist yet
              // FhirPathValue::Lambda(_) => ValueDiscriminant::Lambda,
        }
    }

    /// Optimized duplicate detection using hash-like approach
    /// Time complexity: O(n) average case, O(n²) worst case
    /// Space complexity: O(n)
    #[inline]
    fn has_no_duplicates<'a, I>(items: I) -> bool
    where
        I: Iterator<Item = &'a FhirPathValue>,
    {
        // Use a HashMap with custom discriminant keys for fast duplicate detection
        let mut seen: HashMap<ValueDiscriminant, Vec<&'a FhirPathValue>> = HashMap::new();

        for item in items {
            let discriminant = Self::create_discriminant(item);

            // Check if we've seen this discriminant before
            if let Some(existing_items) = seen.get_mut(&discriminant) {
                // We have a potential match - need to check for actual equality
                // within items that have the same discriminant
                for existing_item in existing_items.iter() {
                    if item == *existing_item {
                        return false; // Duplicate found
                    }
                }
                existing_items.push(item);
            } else {
                seen.insert(discriminant, vec![item]);
            }
        }

        true // No duplicates found
    }
}

#[async_trait]
impl AsyncFhirPathFunction for IsDistinctFunction {
    fn name(&self) -> &str {
        "isDistinct"
    }
    fn human_friendly_name(&self) -> &str {
        "Is Distinct"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new("isDistinct", vec![], TypeInfo::Boolean)
        });
        &SIG
    }
    fn is_pure(&self) -> bool {
        true // isDistinct() is a pure boolean function
    }
    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let is_distinct = match &context.input {
            FhirPathValue::Empty => true, // Empty collection has no duplicates
            FhirPathValue::Collection(items) => Self::has_no_duplicates(items.iter()),
            _ => true, // Single value is always distinct
        };
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
            is_distinct,
        )]))
    }
}
