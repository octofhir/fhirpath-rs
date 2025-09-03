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

//! Suggestion engine for generating helpful diagnostic suggestions

use crate::diagnostic::{Suggestion, SuggestionType, TextEdit};
use crate::location::SourceLocation;
use std::collections::HashMap;

/// Fuzzy matching utilities for finding similar strings
pub mod fuzzy_matching {
    /// Calculate similarity between two strings using Levenshtein distance
    /// Returns a value between 0.0 (no similarity) and 1.0 (identical)
    pub fn calculate_similarity(input: &str, candidate: &str) -> f32 {
        if input.is_empty() && candidate.is_empty() {
            return 1.0;
        }
        if input.is_empty() || candidate.is_empty() {
            return 0.0;
        }
        
        let distance = levenshtein_distance(input, candidate);
        let max_len = input.len().max(candidate.len());
        1.0 - (distance as f32 / max_len as f32)
    }
    
    /// Find the best matches for an input string from a list of candidates
    /// Returns tuples of (candidate, similarity_score) sorted by score (descending)
    pub fn find_best_matches(
        input: &str,
        candidates: &[String], 
        max_results: usize
    ) -> Vec<(String, f32)> {
        let mut matches: Vec<(String, f32)> = candidates
            .iter()
            .map(|candidate| {
                let similarity = calculate_similarity(input, candidate);
                (candidate.clone(), similarity)
            })
            .collect();
        
        // Sort by similarity score (descending)
        matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Take only the best matches above a threshold
        matches
            .into_iter()
            .filter(|(_, score)| *score > 0.3) // Minimum similarity threshold
            .take(max_results)
            .collect()
    }
    
    /// Calculate Levenshtein distance between two strings
    fn levenshtein_distance(s1: &str, s2: &str) -> usize {
        let len1 = s1.chars().count();
        let len2 = s2.chars().count();
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];
        
        // Initialize first row and column
        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }
        
        let chars1: Vec<char> = s1.chars().collect();
        let chars2: Vec<char> = s2.chars().collect();
        
        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if chars1[i - 1] == chars2[j - 1] { 0 } else { 1 };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }
        
        matrix[len1][len2]
    }
    
    #[cfg(test)]
    mod tests {
        use super::*;
        
        #[test]
        fn test_levenshtein_distance() {
            assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
            assert_eq!(levenshtein_distance("hello", "hello"), 0);
            assert_eq!(levenshtein_distance("", ""), 0);
            assert_eq!(levenshtein_distance("a", ""), 1);
            assert_eq!(levenshtein_distance("", "a"), 1);
        }
        
        #[test]
        fn test_calculate_similarity() {
            assert_eq!(calculate_similarity("hello", "hello"), 1.0);
            assert!(calculate_similarity("hello", "helo") > 0.7);
            assert!(calculate_similarity("test", "best") > 0.6);
            assert!(calculate_similarity("abc", "xyz") < 0.5);
        }
        
        #[test]
        fn test_find_best_matches() {
            let candidates = vec![
                "identifier".to_string(),
                "active".to_string(),
                "name".to_string(),
                "telecom".to_string(),
                "gender".to_string(),
            ];
            
            let matches = find_best_matches("identifer", &candidates, 3);
            
            assert!(!matches.is_empty());
            assert_eq!(matches[0].0, "identifier");
            assert!(matches[0].1 > 0.8);
        }
    }
}

/// Enhanced suggestion engine that provides context-aware suggestions
pub struct EnhancedSuggestionEngine {
    /// Cache for suggestion results
    suggestion_cache: HashMap<String, Vec<Suggestion>>,
    /// Known FHIR properties for different resource types
    fhir_properties: HashMap<String, Vec<String>>,
    /// Known FHIRPath functions
    fhirpath_functions: Vec<String>,
}

impl EnhancedSuggestionEngine {
    /// Create a new enhanced suggestion engine
    pub fn new() -> Self {
        let mut engine = Self {
            suggestion_cache: HashMap::new(),
            fhir_properties: HashMap::new(),
            fhirpath_functions: Vec::new(),
        };
        
        engine.initialize_fhir_properties();
        engine.initialize_fhirpath_functions();
        engine
    }
    
    /// Initialize known FHIR properties for common resource types
    fn initialize_fhir_properties(&mut self) {
        // Patient resource properties
        self.fhir_properties.insert("Patient".to_string(), vec![
            "id".to_string(),
            "meta".to_string(),
            "implicitRules".to_string(),
            "language".to_string(),
            "text".to_string(),
            "contained".to_string(),
            "extension".to_string(),
            "modifierExtension".to_string(),
            "identifier".to_string(),
            "active".to_string(),
            "name".to_string(),
            "telecom".to_string(),
            "gender".to_string(),
            "birthDate".to_string(),
            "deceased".to_string(),
            "address".to_string(),
            "maritalStatus".to_string(),
            "multipleBirth".to_string(),
            "photo".to_string(),
            "contact".to_string(),
            "communication".to_string(),
            "generalPractitioner".to_string(),
            "managingOrganization".to_string(),
            "link".to_string(),
        ]);
        
        // Observation resource properties
        self.fhir_properties.insert("Observation".to_string(), vec![
            "id".to_string(),
            "meta".to_string(),
            "implicitRules".to_string(),
            "language".to_string(),
            "text".to_string(),
            "contained".to_string(),
            "extension".to_string(),
            "modifierExtension".to_string(),
            "identifier".to_string(),
            "basedOn".to_string(),
            "partOf".to_string(),
            "status".to_string(),
            "category".to_string(),
            "code".to_string(),
            "subject".to_string(),
            "focus".to_string(),
            "encounter".to_string(),
            "effective".to_string(),
            "issued".to_string(),
            "performer".to_string(),
            "value".to_string(),
            "dataAbsentReason".to_string(),
            "interpretation".to_string(),
            "note".to_string(),
            "bodySite".to_string(),
            "method".to_string(),
            "specimen".to_string(),
            "device".to_string(),
            "referenceRange".to_string(),
            "hasMember".to_string(),
            "derivedFrom".to_string(),
            "component".to_string(),
        ]);
    }
    
    /// Initialize known FHIRPath functions
    fn initialize_fhirpath_functions(&mut self) {
        self.fhirpath_functions = vec![
            // Collection functions
            "empty".to_string(),
            "exists".to_string(),
            "all".to_string(),
            "allTrue".to_string(),
            "anyTrue".to_string(),
            "allFalse".to_string(),
            "anyFalse".to_string(),
            "count".to_string(),
            "distinct".to_string(),
            "isDistinct".to_string(),
            "subsetOf".to_string(),
            "supersetOf".to_string(),
            "intersect".to_string(),
            "exclude".to_string(),
            "union".to_string(),
            "combine".to_string(),
            "first".to_string(),
            "last".to_string(),
            "tail".to_string(),
            "skip".to_string(),
            "take".to_string(),
            "single".to_string(),
            "select".to_string(),
            "repeat".to_string(),
            "ofType".to_string(),
            
            // String functions
            "indexOf".to_string(),
            "substring".to_string(),
            "startsWith".to_string(),
            "endsWith".to_string(),
            "contains".to_string(),
            "upper".to_string(),
            "lower".to_string(),
            "replace".to_string(),
            "matches".to_string(),
            "replaceMatches".to_string(),
            "length".to_string(),
            "toChars".to_string(),
            
            // Math functions
            "abs".to_string(),
            "ceiling".to_string(),
            "exp".to_string(),
            "floor".to_string(),
            "ln".to_string(),
            "log".to_string(),
            "power".to_string(),
            "round".to_string(),
            "sqrt".to_string(),
            "truncate".to_string(),
            
            // Type functions
            "is".to_string(),
            "as".to_string(),
            "type".to_string(),
            
            // Utility functions
            "iif".to_string(),
            "trace".to_string(),
            "today".to_string(),
            "now".to_string(),
            
            // FHIR functions
            "extension".to_string(),
            "hasValue".to_string(),
            "resolve".to_string(),
            "conformsTo".to_string(),
        ];
    }
    
    /// Generate suggestions for unknown properties
    pub fn suggest_property_fixes(
        &mut self,
        invalid_property: &str,
        base_type: &str,
        location: SourceLocation,
    ) -> Vec<Suggestion> {
        let cache_key = format!("prop:{}:{}", invalid_property, base_type);
        
        if let Some(cached) = self.suggestion_cache.get(&cache_key) {
            return cached.clone();
        }
        
        let mut suggestions = Vec::new();
        
        // Look up properties for the base type
        if let Some(properties) = self.fhir_properties.get(base_type) {
            let matches = fuzzy_matching::find_best_matches(
                invalid_property,
                properties,
                3
            );
            
            for (property_name, confidence) in matches {
                suggestions.push(Suggestion::with_replacement(
                    format!("Did you mean '{}'?", property_name),
                    TextEdit::new(location.clone(), property_name),
                    SuggestionType::TypoFix,
                    confidence,
                ));
            }
        }
        
        // Add general suggestions if no specific matches
        if suggestions.is_empty() {
            suggestions.push(Suggestion::new(
                format!("Property '{}' is not defined on type '{}'", invalid_property, base_type),
                SuggestionType::AlternativeProperty,
                0.5,
            ));
            
            if base_type == "Patient" {
                suggestions.push(Suggestion::new(
                    "Common Patient properties: identifier, name, telecom, gender, birthDate".to_string(),
                    SuggestionType::AlternativeProperty,
                    0.7,
                ));
            }
        }
        
        self.suggestion_cache.insert(cache_key, suggestions.clone());
        suggestions
    }
    
    /// Generate suggestions for unknown functions
    pub fn suggest_function_fixes(
        &mut self,
        invalid_function: &str,
        location: SourceLocation,
    ) -> Vec<Suggestion> {
        let cache_key = format!("func:{}", invalid_function);
        
        if let Some(cached) = self.suggestion_cache.get(&cache_key) {
            return cached.clone();
        }
        
        let mut suggestions = Vec::new();
        
        let matches = fuzzy_matching::find_best_matches(
            invalid_function,
            &self.fhirpath_functions,
            3
        );
        
        for (function_name, confidence) in matches {
            suggestions.push(Suggestion::with_replacement(
                format!("Did you mean '{}()'?", function_name),
                TextEdit::new(location.clone(), function_name),
                SuggestionType::TypoFix,
                confidence,
            ));
        }
        
        // Add category-based suggestions
        if suggestions.is_empty() {
            if invalid_function.contains("count") || invalid_function.contains("length") {
                suggestions.push(Suggestion::new(
                    "For collection size, use 'count()' function".to_string(),
                    SuggestionType::AlternativeFunction,
                    0.8,
                ));
            } else if invalid_function.contains("first") || invalid_function.contains("head") {
                suggestions.push(Suggestion::new(
                    "To get first element, use 'first()' function".to_string(),
                    SuggestionType::AlternativeFunction,
                    0.8,
                ));
            } else if invalid_function.contains("last") || invalid_function.contains("tail") {
                suggestions.push(Suggestion::new(
                    "To get last element, use 'last()' function".to_string(),
                    SuggestionType::AlternativeFunction,
                    0.8,
                ));
            }
        }
        
        self.suggestion_cache.insert(cache_key, suggestions.clone());
        suggestions
    }
    
    /// Generate suggestions for type conversion errors
    pub fn suggest_type_fixes(
        &mut self,
        expected_type: &str,
        actual_type: &str,
        location: SourceLocation,
    ) -> Vec<Suggestion> {
        let cache_key = format!("type:{}:{}", expected_type, actual_type);
        
        if let Some(cached) = self.suggestion_cache.get(&cache_key) {
            return cached.clone();
        }
        
        let mut suggestions = Vec::new();
        
        match (expected_type, actual_type) {
            ("Boolean", "String") => {
                suggestions.push(Suggestion::with_replacement(
                    "Convert string to boolean using comparison".to_string(),
                    TextEdit::new(location.clone(), " = 'true'".to_string()),
                    SuggestionType::TypeConversion,
                    0.9,
                ));
            },
            ("String", "Boolean") => {
                suggestions.push(Suggestion::with_replacement(
                    "Convert boolean to string using toString()".to_string(),
                    TextEdit::new(location.clone(), ".toString()".to_string()),
                    SuggestionType::TypeConversion,
                    0.9,
                ));
            },
            ("Integer", "String") => {
                suggestions.push(Suggestion::with_replacement(
                    "Convert string to integer using toInteger()".to_string(),
                    TextEdit::new(location.clone(), ".toInteger()".to_string()),
                    SuggestionType::TypeConversion,
                    0.9,
                ));
            },
            _ => {
                suggestions.push(Suggestion::new(
                    format!("Expected {} but got {}", expected_type, actual_type),
                    SuggestionType::TypeConversion,
                    0.6,
                ));
            },
        }
        
        self.suggestion_cache.insert(cache_key, suggestions.clone());
        suggestions
    }
    
    /// Generate performance optimization suggestions
    pub fn suggest_performance_optimizations(
        &mut self,
        expression: &str,
    ) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();
        
        // Detect expensive patterns
        if expression.contains("descendants()") {
            suggestions.push(Suggestion::new(
                "descendants() can be expensive on large resources. Consider using specific paths.".to_string(),
                SuggestionType::PerformanceOptimization,
                0.8,
            ));
        }
        
        if expression.contains("where(") && expression.contains(".count() >") {
            suggestions.push(Suggestion::new(
                "Instead of 'where(...).count() > 0', use 'where(...).exists()'".to_string(),
                SuggestionType::PerformanceOptimization,
                0.9,
            ));
        }
        
        suggestions
    }
    
    /// Clear the suggestion cache
    pub fn clear_cache(&mut self) {
        self.suggestion_cache.clear();
    }
    
    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        (self.suggestion_cache.len(), self.suggestion_cache.capacity())
    }
}

impl Default for EnhancedSuggestionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::location::{Position, Span};
    
    #[test]
    fn test_property_suggestions() {
        let mut engine = EnhancedSuggestionEngine::new();
        let location = SourceLocation {
            span: Span::new(Position::new(0, 0), Position::new(0, 5)),
            source_text: Some("test".to_string()),
            file_path: None,
        };
        
        let suggestions = engine.suggest_property_fixes("identifer", "Patient", location);
        
        assert!(!suggestions.is_empty());
        assert_eq!(suggestions[0].suggestion_type, SuggestionType::TypoFix);
        assert!(suggestions[0].confidence > 0.8);
    }
    
    #[test]
    fn test_function_suggestions() {
        let mut engine = EnhancedSuggestionEngine::new();
        let location = SourceLocation {
            span: Span::new(Position::new(0, 0), Position::new(0, 5)),
            source_text: Some("test".to_string()),
            file_path: None,
        };
        
        let suggestions = engine.suggest_function_fixes("frist", location);
        
        assert!(!suggestions.is_empty());
        assert_eq!(suggestions[0].suggestion_type, SuggestionType::TypoFix);
    }
    
    #[test]
    fn test_type_suggestions() {
        let mut engine = EnhancedSuggestionEngine::new();
        let location = SourceLocation {
            span: Span::new(Position::new(0, 0), Position::new(0, 5)),
            source_text: Some("test".to_string()),
            file_path: None,
        };
        
        let suggestions = engine.suggest_type_fixes("Boolean", "String", location);
        
        assert!(!suggestions.is_empty());
        assert_eq!(suggestions[0].suggestion_type, SuggestionType::TypeConversion);
        assert!(suggestions[0].confidence > 0.8);
    }
    
    #[test]
    fn test_performance_suggestions() {
        let mut engine = EnhancedSuggestionEngine::new();
        
        let suggestions = engine.suggest_performance_optimizations("Patient.descendants().count() > 0");
        
        assert!(!suggestions.is_empty());
        assert_eq!(suggestions[0].suggestion_type, SuggestionType::PerformanceOptimization);
    }
    
    #[test]
    fn test_cache_functionality() {
        let mut engine = EnhancedSuggestionEngine::new();
        let location = SourceLocation {
            span: Span::new(Position::new(0, 0), Position::new(0, 5)),
            source_text: Some("test".to_string()),
            file_path: None,
        };
        
        // First call should populate cache
        let suggestions1 = engine.suggest_property_fixes("identifer", "Patient", location.clone());
        let (cache_size1, _) = engine.cache_stats();
        
        // Second call should use cache
        let suggestions2 = engine.suggest_property_fixes("identifer", "Patient", location);
        let (cache_size2, _) = engine.cache_stats();
        
        assert_eq!(suggestions1.len(), suggestions2.len());
        assert_eq!(cache_size1, cache_size2); // Cache size shouldn't change on second call
        assert!(cache_size1 > 0);
    }
}