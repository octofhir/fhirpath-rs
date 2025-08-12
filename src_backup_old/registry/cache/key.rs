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

//! Cache key implementations for function resolution

use crate::model::TypeInfo;
use std::hash::{Hash, Hasher};

/// Cache key for function resolution by name and argument types
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionCacheKey {
    /// Function name
    pub name: String,
    /// Argument types for the function call
    pub arg_types: Vec<TypeInfo>,
}

impl FunctionCacheKey {
    /// Create a new function cache key
    pub fn new(name: impl Into<String>, arg_types: Vec<TypeInfo>) -> Self {
        Self {
            name: name.into(),
            arg_types,
        }
    }
}

impl Hash for FunctionCacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);

        // Hash the number of arguments first for better distribution
        self.arg_types.len().hash(state);

        // Hash each argument type
        for arg_type in &self.arg_types {
            // Use discriminant for efficient hashing
            std::mem::discriminant(arg_type).hash(state);

            // Hash additional data for complex types
            match arg_type {
                TypeInfo::Named { name, .. } => name.hash(state),
                TypeInfo::Collection(inner) => inner.hash(state),
                TypeInfo::Function {
                    parameters,
                    return_type,
                } => {
                    parameters.hash(state);
                    return_type.hash(state);
                }
                TypeInfo::Union(types) => {
                    for t in types {
                        t.hash(state);
                    }
                }
                _ => {} // Simple types are covered by discriminant
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;

    #[test]
    fn test_cache_key_equality() {
        let key1 = FunctionCacheKey::new("test", vec![TypeInfo::String, TypeInfo::Integer]);
        let key2 = FunctionCacheKey::new("test", vec![TypeInfo::String, TypeInfo::Integer]);
        let key3 = FunctionCacheKey::new("test", vec![TypeInfo::Integer, TypeInfo::String]);
        let key4 = FunctionCacheKey::new("other", vec![TypeInfo::String, TypeInfo::Integer]);

        assert_eq!(key1, key2);
        assert_ne!(key1, key3); // Different order
        assert_ne!(key1, key4); // Different name
    }

    #[test]
    fn test_cache_key_hash() {
        let key1 = FunctionCacheKey::new("test", vec![TypeInfo::String]);
        let key2 = FunctionCacheKey::new("test", vec![TypeInfo::String]);
        let key3 = FunctionCacheKey::new("test", vec![TypeInfo::Integer]);

        let hash1 = calculate_hash(&key1);
        let hash2 = calculate_hash(&key2);
        let hash3 = calculate_hash(&key3);

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_cache_key_with_complex_types() {
        let key1 = FunctionCacheKey::new(
            "test",
            vec![TypeInfo::Collection(Box::new(TypeInfo::String))],
        );
        let key2 = FunctionCacheKey::new(
            "test",
            vec![TypeInfo::Collection(Box::new(TypeInfo::Integer))],
        );

        assert_ne!(key1, key2);
        assert_ne!(calculate_hash(&key1), calculate_hash(&key2));
    }

    fn calculate_hash<T: Hash>(t: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        t.hash(&mut hasher);
        hasher.finish()
    }
}
