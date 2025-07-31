//! Type coercion and conversion utilities for FHIRPath

use super::value::FhirPathValue;
use super::types::TypeInfo;
use rust_decimal::Decimal;
use std::str::FromStr;

/// Result type for type coercion operations
pub type CoercionResult<T> = Result<T, CoercionError>;

/// Errors that can occur during type coercion
#[derive(Debug, Clone, PartialEq)]
pub enum CoercionError {
    /// Cannot coerce between the specified types
    IncompatibleTypes { from: String, to: String },
    /// The value format is invalid for the target type
    InvalidFormat { value: String, target_type: String },
    /// Multiple items in collection when single item expected
    MultipleItems,
    /// Empty collection when value expected
    EmptyCollection,
}

impl std::fmt::Display for CoercionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoercionError::IncompatibleTypes { from, to } => {
                write!(f, "Cannot coerce from {} to {}", from, to)
            }
            CoercionError::InvalidFormat { value, target_type } => {
                write!(f, "Invalid format '{}' for type {}", value, target_type)
            }
            CoercionError::MultipleItems => {
                write!(f, "Cannot coerce collection with multiple items to single value")
            }
            CoercionError::EmptyCollection => {
                write!(f, "Cannot coerce empty collection to value")
            }
        }
    }
}

impl std::error::Error for CoercionError {}

/// Type coercion utility for FHIRPath values
pub struct TypeCoercion;

impl TypeCoercion {
    /// Attempt to coerce a value to the specified type
    pub fn coerce_to_type(
        value: &FhirPathValue,
        target_type: &TypeInfo,
    ) -> CoercionResult<FhirPathValue> {
        match target_type {
            TypeInfo::Boolean => Self::coerce_to_boolean(value),
            TypeInfo::Integer => Self::coerce_to_integer(value),
            TypeInfo::Decimal => Self::coerce_to_decimal(value),
            TypeInfo::String => Self::coerce_to_string(value),
            TypeInfo::Date => Self::coerce_to_date(value),
            TypeInfo::DateTime => Self::coerce_to_datetime(value),
            TypeInfo::Time => Self::coerce_to_time(value),
            TypeInfo::Collection(elem_type) => Self::coerce_to_collection(value, elem_type),
            TypeInfo::Optional(inner_type) => {
                // For optional types, accept empty values
                if value.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else {
                    Self::coerce_to_type(value, inner_type)
                }
            }
            TypeInfo::Union(types) => Self::coerce_to_union(value, types),
            TypeInfo::Any => Ok(value.clone()),
            _ => Err(CoercionError::IncompatibleTypes {
                from: value.type_name().to_string(),
                to: target_type.type_name(),
            }),
        }
    }

    /// Coerce value to boolean
    pub fn coerce_to_boolean(value: &FhirPathValue) -> CoercionResult<FhirPathValue> {
        match value {
            FhirPathValue::Boolean(b) => Ok(FhirPathValue::Boolean(*b)),
            FhirPathValue::Integer(i) => Ok(FhirPathValue::Boolean(*i != 0)),
            FhirPathValue::Decimal(d) => Ok(FhirPathValue::Boolean(!d.is_zero())),
            FhirPathValue::String(s) => {
                let lower = s.to_lowercase();
                match lower.as_str() {
                    "true" | "t" | "yes" | "y" | "1" | "1.0" => Ok(FhirPathValue::Boolean(true)),
                    "false" | "f" | "no" | "n" | "0" | "0.0" => Ok(FhirPathValue::Boolean(false)),
                    _ => Err(CoercionError::InvalidFormat {
                        value: s.clone(),
                        target_type: "Boolean".to_string(),
                    }),
                }
            }
            FhirPathValue::Collection(items) => {
                if items.len() > 1 {
                    Err(CoercionError::MultipleItems)
                } else if items.is_empty() {
                    Err(CoercionError::EmptyCollection)
                } else {
                    Self::coerce_to_boolean(items.first().unwrap())
                }
            }
            FhirPathValue::Empty => Err(CoercionError::EmptyCollection),
            _ => Err(CoercionError::IncompatibleTypes {
                from: value.type_name().to_string(),
                to: "Boolean".to_string(),
            }),
        }
    }

    /// Coerce value to integer
    pub fn coerce_to_integer(value: &FhirPathValue) -> CoercionResult<FhirPathValue> {
        match value {
            FhirPathValue::Integer(i) => Ok(FhirPathValue::Integer(*i)),
            FhirPathValue::Decimal(d) => {
                if d.fract().is_zero() {
                    if let Some(i) = d.to_i64() {
                        Ok(FhirPathValue::Integer(i))
                    } else {
                        Err(CoercionError::InvalidFormat {
                            value: d.to_string(),
                            target_type: "Integer".to_string(),
                        })
                    }
                } else {
                    Err(CoercionError::InvalidFormat {
                        value: d.to_string(),
                        target_type: "Integer".to_string(),
                    })
                }
            }
            FhirPathValue::Boolean(b) => Ok(FhirPathValue::Integer(if *b { 1 } else { 0 })),
            FhirPathValue::String(s) => {
                if s.contains('.') {
                    Err(CoercionError::InvalidFormat {
                        value: s.clone(),
                        target_type: "Integer".to_string(),
                    })
                } else {
                    match s.trim().parse::<i64>() {
                        Ok(i) => Ok(FhirPathValue::Integer(i)),
                        Err(_) => Err(CoercionError::InvalidFormat {
                            value: s.clone(),
                            target_type: "Integer".to_string(),
                        }),
                    }
                }
            }
            FhirPathValue::Collection(items) => {
                if items.len() > 1 {
                    Err(CoercionError::MultipleItems)
                } else if items.is_empty() {
                    Err(CoercionError::EmptyCollection)
                } else {
                    Self::coerce_to_integer(items.first().unwrap())
                }
            }
            FhirPathValue::Empty => Err(CoercionError::EmptyCollection),
            _ => Err(CoercionError::IncompatibleTypes {
                from: value.type_name().to_string(),
                to: "Integer".to_string(),
            }),
        }
    }

    /// Coerce value to decimal
    pub fn coerce_to_decimal(value: &FhirPathValue) -> CoercionResult<FhirPathValue> {
        match value {
            FhirPathValue::Decimal(d) => Ok(FhirPathValue::Decimal(*d)),
            FhirPathValue::Integer(i) => Ok(FhirPathValue::Decimal(Decimal::from(*i))),
            FhirPathValue::Boolean(b) => Ok(FhirPathValue::Decimal(
                if *b { Decimal::ONE } else { Decimal::ZERO }
            )),
            FhirPathValue::String(s) => {
                match Decimal::from_str(s.trim()) {
                    Ok(d) => Ok(FhirPathValue::Decimal(d)),
                    Err(_) => Err(CoercionError::InvalidFormat {
                        value: s.clone(),
                        target_type: "Decimal".to_string(),
                    }),
                }
            }
            FhirPathValue::Collection(items) => {
                if items.len() > 1 {
                    Err(CoercionError::MultipleItems)
                } else if items.is_empty() {
                    Err(CoercionError::EmptyCollection)
                } else {
                    Self::coerce_to_decimal(items.first().unwrap())
                }
            }
            FhirPathValue::Empty => Err(CoercionError::EmptyCollection),
            _ => Err(CoercionError::IncompatibleTypes {
                from: value.type_name().to_string(),
                to: "Decimal".to_string(),
            }),
        }
    }

    /// Coerce value to string
    pub fn coerce_to_string(value: &FhirPathValue) -> CoercionResult<FhirPathValue> {
        if let Some(string_val) = value.to_string_value() {
            Ok(FhirPathValue::String(string_val))
        } else {
            match value {
                FhirPathValue::Collection(items) => {
                    if items.len() > 1 {
                        Err(CoercionError::MultipleItems)
                    } else if items.is_empty() {
                        Err(CoercionError::EmptyCollection)
                    } else {
                        Self::coerce_to_string(items.first().unwrap())
                    }
                }
                FhirPathValue::Empty => Err(CoercionError::EmptyCollection),
                _ => Err(CoercionError::IncompatibleTypes {
                    from: value.type_name().to_string(),
                    to: "String".to_string(),
                }),
            }
        }
    }

    /// Coerce value to date
    pub fn coerce_to_date(value: &FhirPathValue) -> CoercionResult<FhirPathValue> {
        match value {
            FhirPathValue::Date(d) => Ok(FhirPathValue::Date(*d)),
            FhirPathValue::String(s) => {
                match chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                    Ok(date) => Ok(FhirPathValue::Date(date)),
                    Err(_) => Err(CoercionError::InvalidFormat {
                        value: s.clone(),
                        target_type: "Date".to_string(),
                    }),
                }
            }
            FhirPathValue::Collection(items) => {
                if items.len() > 1 {
                    Err(CoercionError::MultipleItems)
                } else if items.is_empty() {
                    Err(CoercionError::EmptyCollection)
                } else {
                    Self::coerce_to_date(items.first().unwrap())
                }
            }
            FhirPathValue::Empty => Err(CoercionError::EmptyCollection),
            _ => Err(CoercionError::IncompatibleTypes {
                from: value.type_name().to_string(),
                to: "Date".to_string(),
            }),
        }
    }

    /// Coerce value to datetime
    pub fn coerce_to_datetime(value: &FhirPathValue) -> CoercionResult<FhirPathValue> {
        match value {
            FhirPathValue::DateTime(dt) => Ok(FhirPathValue::DateTime(*dt)),
            FhirPathValue::Date(d) => {
                // Convert date to datetime at midnight
                let dt = d.and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_local_timezone(chrono::FixedOffset::east_opt(0).unwrap())
                    .unwrap();
                Ok(FhirPathValue::DateTime(dt))
            }
            FhirPathValue::String(s) => {
                match chrono::DateTime::parse_from_rfc3339(s) {
                    Ok(dt) => Ok(FhirPathValue::DateTime(dt.fixed_offset())),
                    Err(_) => Err(CoercionError::InvalidFormat {
                        value: s.clone(),
                        target_type: "DateTime".to_string(),
                    }),
                }
            }
            FhirPathValue::Collection(items) => {
                if items.len() > 1 {
                    Err(CoercionError::MultipleItems)
                } else if items.is_empty() {
                    Err(CoercionError::EmptyCollection)
                } else {
                    Self::coerce_to_datetime(items.first().unwrap())
                }
            }
            FhirPathValue::Empty => Err(CoercionError::EmptyCollection),
            _ => Err(CoercionError::IncompatibleTypes {
                from: value.type_name().to_string(),
                to: "DateTime".to_string(),
            }),
        }
    }

    /// Coerce value to time
    pub fn coerce_to_time(value: &FhirPathValue) -> CoercionResult<FhirPathValue> {
        match value {
            FhirPathValue::Time(t) => Ok(FhirPathValue::Time(*t)),
            FhirPathValue::String(s) => {
                match chrono::NaiveTime::parse_from_str(s, "%H:%M:%S") {
                    Ok(time) => Ok(FhirPathValue::Time(time)),
                    Err(_) => match chrono::NaiveTime::parse_from_str(s, "%H:%M:%S%.f") {
                        Ok(time) => Ok(FhirPathValue::Time(time)),
                        Err(_) => Err(CoercionError::InvalidFormat {
                            value: s.clone(),
                            target_type: "Time".to_string(),
                        }),
                    },
                }
            }
            FhirPathValue::Collection(items) => {
                if items.len() > 1 {
                    Err(CoercionError::MultipleItems)
                } else if items.is_empty() {
                    Err(CoercionError::EmptyCollection)
                } else {
                    Self::coerce_to_time(items.first().unwrap())
                }
            }
            FhirPathValue::Empty => Err(CoercionError::EmptyCollection),
            _ => Err(CoercionError::IncompatibleTypes {
                from: value.type_name().to_string(),
                to: "Time".to_string(),
            }),
        }
    }

    /// Coerce value to collection
    pub fn coerce_to_collection(
        value: &FhirPathValue,
        element_type: &TypeInfo,
    ) -> CoercionResult<FhirPathValue> {
        match value {
            FhirPathValue::Collection(items) => {
                let mut coerced_items = Vec::new();
                for item in items.iter() {
                    coerced_items.push(Self::coerce_to_type(item, element_type)?);
                }
                Ok(FhirPathValue::collection(coerced_items))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::collection(vec![])),
            single => {
                let coerced = Self::coerce_to_type(single, element_type)?;
                Ok(FhirPathValue::collection(vec![coerced]))
            }
        }
    }

    /// Coerce value to union type
    pub fn coerce_to_union(
        value: &FhirPathValue,
        types: &[TypeInfo],
    ) -> CoercionResult<FhirPathValue> {
        // Try each type in the union until one succeeds
        for target_type in types {
            if let Ok(result) = Self::coerce_to_type(value, target_type) {
                return Ok(result);
            }
        }

        // If no type worked, return error
        let type_names: Vec<String> = types.iter().map(|t| t.type_name()).collect();
        Err(CoercionError::IncompatibleTypes {
            from: value.type_name().to_string(),
            to: format!("Union<{}>", type_names.join(", ")),
        })
    }

    /// Check if a value can be coerced to a specific type
    pub fn can_coerce_to_type(value: &FhirPathValue, target_type: &TypeInfo) -> bool {
        Self::coerce_to_type(value, target_type).is_ok()
    }

    /// Find the best common type for a collection of values
    pub fn find_common_type(values: &[FhirPathValue]) -> TypeInfo {
        if values.is_empty() {
            return TypeInfo::Any;
        }

        let first_type = values[0].to_type_info();
        
        // If all values have the same type, return that type
        if values.iter().all(|v| v.to_type_info() == first_type) {
            return first_type;
        }

        // Check for numeric type compatibility
        if values.iter().all(|v| matches!(v, FhirPathValue::Integer(_) | FhirPathValue::Decimal(_))) {
            return TypeInfo::Decimal; // Promote to decimal
        }

        // Check if all can be converted to string
        if values.iter().all(|v| v.to_string_value().is_some()) {
            return TypeInfo::String;
        }

        // Otherwise, use union type
        let unique_types: std::collections::HashSet<_> = values.iter()
            .map(|v| v.to_type_info())
            .collect();
        
        TypeInfo::union(unique_types.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn test_boolean_coercion() {
        // Boolean to boolean
        assert_eq!(
            TypeCoercion::coerce_to_boolean(&FhirPathValue::Boolean(true)).unwrap(),
            FhirPathValue::Boolean(true)
        );

        // Integer to boolean
        assert_eq!(
            TypeCoercion::coerce_to_boolean(&FhirPathValue::Integer(1)).unwrap(),
            FhirPathValue::Boolean(true)
        );
        assert_eq!(
            TypeCoercion::coerce_to_boolean(&FhirPathValue::Integer(0)).unwrap(),
            FhirPathValue::Boolean(false)
        );

        // String to boolean
        assert_eq!(
            TypeCoercion::coerce_to_boolean(&FhirPathValue::String("true".to_string())).unwrap(),
            FhirPathValue::Boolean(true)
        );
        assert_eq!(
            TypeCoercion::coerce_to_boolean(&FhirPathValue::String("false".to_string())).unwrap(),
            FhirPathValue::Boolean(false)
        );

        // Invalid string to boolean should fail
        assert!(TypeCoercion::coerce_to_boolean(&FhirPathValue::String("invalid".to_string())).is_err());
    }

    #[test]
    fn test_integer_coercion() {
        // Integer to integer
        assert_eq!(
            TypeCoercion::coerce_to_integer(&FhirPathValue::Integer(42)).unwrap(),
            FhirPathValue::Integer(42)
        );

        // Decimal to integer (whole number)
        assert_eq!(
            TypeCoercion::coerce_to_integer(&FhirPathValue::Decimal(Decimal::from(42))).unwrap(),
            FhirPathValue::Integer(42)
        );

        // Decimal to integer (fractional should fail)
        assert!(TypeCoercion::coerce_to_integer(&FhirPathValue::Decimal(Decimal::new(425, 1))).is_err());

        // String to integer
        assert_eq!(
            TypeCoercion::coerce_to_integer(&FhirPathValue::String("42".to_string())).unwrap(),
            FhirPathValue::Integer(42)
        );

        // String with decimal point should fail
        assert!(TypeCoercion::coerce_to_integer(&FhirPathValue::String("42.5".to_string())).is_err());
    }

    #[test]
    fn test_common_type_detection() {
        let values = vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ];
        assert_eq!(TypeCoercion::find_common_type(&values), TypeInfo::Integer);

        let mixed_numeric = vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Decimal(Decimal::new(25, 1)),
        ];
        assert_eq!(TypeCoercion::find_common_type(&mixed_numeric), TypeInfo::Decimal);
    }
}