// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Lambda metadata support for FHIRPath lambda functions
//!
//! This module provides support for implicit variables available within lambda expressions:
//! - `$this` - Current item being processed
//! - `$index` - Current index in iteration (0-based)
//! - `$total` - Total count or accumulator value
//!
//! Lambda functions like `where`, `select`, `all`, `any` create contexts where these
//! implicit variables are automatically available for use within the lambda expression.

use octofhir_fhirpath_core::FhirPathValue;

/// Lambda-specific metadata for implicit variables
///
/// Lambda functions in FHIRPath (such as `where`, `select`, `all`, `any`) provide
/// special implicit variables that are automatically available within the lambda expression:
///
/// - `$this` - The current item being processed in the iteration
/// - `$index` - The zero-based index of the current item  
/// - `$total` - The total count or accumulator value (context-dependent)
///
/// # Examples
///
/// ```fhirpath
/// // $this is implicitly available in lambda
/// Patient.name.where($this.use = 'official')
///
/// // $index provides zero-based iteration index
/// Patient.name.select($this.given + ' (' + $index + ')')
///
/// // $total can represent count or accumulator
/// Patient.telecom.where($total.count() > 2)
/// ```
#[derive(Clone, Debug)]
pub struct LambdaMetadata {
    /// Current item being processed ($this)
    pub current_item: FhirPathValue,

    /// Current index in iteration ($index)  
    /// Always contains an Integer value representing the zero-based index
    pub current_index: FhirPathValue,

    /// Total count or accumulator ($total)
    /// Meaning depends on the specific lambda function context
    pub total_value: FhirPathValue,
}

impl LambdaMetadata {
    /// Create new lambda metadata with current item, index, and total
    ///
    /// # Arguments
    /// * `item` - The current item being processed (becomes `$this`)
    /// * `index` - Zero-based index of current item (becomes `$index`)
    /// * `total` - Total count or accumulator value (becomes `$total`)
    pub fn new(item: FhirPathValue, index: usize, total: FhirPathValue) -> Self {
        Self {
            current_item: item,
            current_index: FhirPathValue::Integer(index as i64),
            total_value: total,
        }
    }

    /// Get the value of an implicit variable by name
    ///
    /// Returns `Some(value)` if the variable name matches one of the implicit
    /// lambda variables, `None` otherwise.
    ///
    /// # Arguments
    /// * `name` - Variable name (without $ prefix)
    ///
    /// # Returns
    /// * `Some(&FhirPathValue)` - If name matches an implicit variable
    /// * `None` - If name is not an implicit lambda variable
    pub fn get_implicit_variable(&self, name: &str) -> Option<&FhirPathValue> {
        match name {
            "this" => Some(&self.current_item),
            "index" => Some(&self.current_index),
            "total" => Some(&self.total_value),
            _ => None,
        }
    }

    /// Check if a variable name is a lambda implicit variable
    ///
    /// # Arguments
    /// * `name` - Variable name (without $ prefix)
    ///
    /// # Returns
    /// * `true` - If the name is a recognized implicit lambda variable
    /// * `false` - Otherwise
    pub fn is_implicit_variable(name: &str) -> bool {
        matches!(name, "this" | "index" | "total")
    }

    /// Get the current item ($this)
    pub fn current_item(&self) -> &FhirPathValue {
        &self.current_item
    }

    /// Get the current index ($index) as an integer
    ///
    /// # Returns
    /// The zero-based index as i64, or 0 if somehow not an integer
    pub fn current_index_as_i64(&self) -> i64 {
        match &self.current_index {
            FhirPathValue::Integer(i) => *i,
            _ => 0, // Should never happen in practice
        }
    }

    /// Get the total value ($total)
    pub fn total_value(&self) -> &FhirPathValue {
        &self.total_value
    }

    /// Update the current item for a new iteration
    pub fn with_current_item(&self, new_item: FhirPathValue) -> Self {
        Self {
            current_item: new_item,
            current_index: self.current_index.clone(),
            total_value: self.total_value.clone(),
        }
    }

    /// Update the index for a new iteration
    pub fn with_index(&self, new_index: usize) -> Self {
        Self {
            current_item: self.current_item.clone(),
            current_index: FhirPathValue::Integer(new_index as i64),
            total_value: self.total_value.clone(),
        }
    }

    /// Update the total value
    pub fn with_total(&self, new_total: FhirPathValue) -> Self {
        Self {
            current_item: self.current_item.clone(),
            current_index: self.current_index.clone(),
            total_value: new_total,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lambda_metadata_creation() {
        let item = FhirPathValue::String("test".into());
        let total = FhirPathValue::Integer(10);
        let metadata = LambdaMetadata::new(item.clone(), 5, total.clone());

        assert_eq!(metadata.current_item, item);
        assert_eq!(metadata.current_index, FhirPathValue::Integer(5));
        assert_eq!(metadata.total_value, total);
    }

    #[test]
    fn test_get_implicit_variable() {
        let metadata = LambdaMetadata::new(
            FhirPathValue::String("test".into()),
            3,
            FhirPathValue::Integer(20),
        );

        assert_eq!(
            metadata.get_implicit_variable("this"),
            Some(&FhirPathValue::String("test".into()))
        );
        assert_eq!(
            metadata.get_implicit_variable("index"),
            Some(&FhirPathValue::Integer(3))
        );
        assert_eq!(
            metadata.get_implicit_variable("total"),
            Some(&FhirPathValue::Integer(20))
        );
        assert_eq!(metadata.get_implicit_variable("unknown"), None);
    }

    #[test]
    fn test_is_implicit_variable() {
        assert!(LambdaMetadata::is_implicit_variable("this"));
        assert!(LambdaMetadata::is_implicit_variable("index"));
        assert!(LambdaMetadata::is_implicit_variable("total"));
        assert!(!LambdaMetadata::is_implicit_variable("other"));
    }

    #[test]
    fn test_current_index_as_i64() {
        let metadata = LambdaMetadata::new(
            FhirPathValue::String("test".into()),
            42,
            FhirPathValue::Integer(100),
        );

        assert_eq!(metadata.current_index_as_i64(), 42);
    }

    #[test]
    fn test_with_methods() {
        let metadata = LambdaMetadata::new(
            FhirPathValue::String("original".into()),
            0,
            FhirPathValue::Integer(10),
        );

        let new_item = metadata.with_current_item(FhirPathValue::String("new".into()));
        assert_eq!(new_item.current_item, FhirPathValue::String("new".into()));
        assert_eq!(new_item.current_index, FhirPathValue::Integer(0));

        let new_index = metadata.with_index(5);
        assert_eq!(new_index.current_index, FhirPathValue::Integer(5));

        let new_total = metadata.with_total(FhirPathValue::Integer(50));
        assert_eq!(new_total.total_value, FhirPathValue::Integer(50));
    }
}
