//! Operand checks shared by the ordering operators (`<`, `<=`, `>`, `>=`).
//!
//! FHIRPath defines comparison for strings, integers, longs, decimals,
//! quantities, dates, datetimes and times, and requires both operands to be
//! of the same type (or implicitly convertible), throwing an error otherwise.

use crate::core::error_code::FP0051;
use crate::core::{FhirPathError, FhirPathValue, Result};

/// Fail when `left` and `right` cannot be ordered: a Boolean on either side
/// (booleans have no ordering), or a number against a string.
pub fn check_comparable(left: &FhirPathValue, right: &FhirPathValue, operator: &str) -> Result<()> {
    let is_boolean = |v: &FhirPathValue| matches!(v, FhirPathValue::Boolean(..));
    let is_numeric = |v: &FhirPathValue| {
        matches!(
            v,
            FhirPathValue::Integer(..)
                | FhirPathValue::Decimal(..)
                | FhirPathValue::Quantity { .. }
        )
    };
    let is_string = |v: &FhirPathValue| matches!(v, FhirPathValue::String(..));
    if is_boolean(left) || is_boolean(right) {
        return Err(FhirPathError::evaluation_error(
            FP0051,
            format!("Type mismatch: cannot compare Boolean values with '{operator}'"),
        ));
    }
    if (is_numeric(left) && is_string(right)) || (is_string(left) && is_numeric(right)) {
        return Err(FhirPathError::evaluation_error(
            FP0051,
            format!("Type mismatch: cannot compare numeric and string with '{operator}'"),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn booleans_and_number_string_pairs_cannot_be_ordered() {
        let (t, one, s) = (
            FhirPathValue::boolean(true),
            FhirPathValue::integer(1),
            FhirPathValue::string("a".to_string()),
        );
        assert!(check_comparable(&one, &t, ">").is_err());
        assert!(check_comparable(&t, &t, "<").is_err());
        assert!(check_comparable(&one, &s, "<=").is_err());
        assert!(check_comparable(&s, &one, ">=").is_err());
        assert!(check_comparable(&one, &one, ">").is_ok());
        assert!(check_comparable(&s, &s, ">").is_ok());
    }
}
