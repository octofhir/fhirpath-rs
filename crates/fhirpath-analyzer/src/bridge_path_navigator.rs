//! Bridge API-enabled path navigation with intelligent suggestions
//!
//! This module provides enhanced path navigation that leverages the Bridge Support Architecture
//! for generating contextual path suggestions with relevance ranking.

use octofhir_fhirpath_model::provider::ModelProvider;
use octofhir_fhirschema::FhirSchemaPackageManager;
use std::sync::Arc;

use crate::bridge_field_validator::{AnalyzerError, SimilarityMatcher};

/// Path suggestion with enhanced metadata
#[derive(Debug, Clone)]
pub struct PathSuggestion {
    /// The suggested property name
    pub property_name: String,
    /// Full path for this suggestion
    pub full_path: String,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Brief description of the property
    pub description: Option<String>,
    /// Property type information
    pub property_type: String,
    /// Whether this is a collection property
    pub is_collection: bool,
}

/// Enhanced path suggestion with analyzer-specific information
#[derive(Debug, Clone)]
pub struct EnhancedPathSuggestion {
    /// Base suggestion information
    pub base_suggestion: PathSuggestion,
    /// Detailed property information
    pub property_info: super::bridge_field_validator::PropertyInfo,
    /// Usage statistics for this path
    pub usage_stats: UsageStatistics,
    /// Complexity score for evaluation
    pub complexity_score: f64,
    /// Optimization hints for this path
    pub optimization_hints: Vec<String>,
}

/// Usage statistics for path suggestions
#[derive(Debug, Clone)]
pub struct UsageStatistics {
    /// How frequently this path is used in typical queries
    pub frequency_score: f64,
    /// Performance characteristics of this path
    pub performance_score: f64,
    /// Whether this path is commonly used with other paths
    pub correlation_score: f64,
}

/// Bridge-enabled path navigator with suggestion generation
pub struct AnalyzerPathNavigator {
    /// Schema manager for bridge API operations
    schema_manager: Arc<FhirSchemaPackageManager>,
    /// Model provider for type information
    model_provider: Arc<dyn ModelProvider>,
    /// Similarity matcher for suggestions
    similarity_matcher: SimilarityMatcher,
    /// Cache for path suggestions
    suggestion_cache: dashmap::DashMap<String, Vec<EnhancedPathSuggestion>>,
}

impl AnalyzerPathNavigator {
    /// Create new analyzer path navigator with bridge support
    pub async fn new(schema_manager: Arc<FhirSchemaPackageManager>) -> Result<Self, AnalyzerError> {
        let model_provider: Arc<dyn ModelProvider> = Arc::new(
            octofhir_fhirpath_model::FhirSchemaModelProvider::new()
                .await
                .map_err(|e| AnalyzerError::InitializationError {
                    message: format!("Failed to create model provider: {}", e),
                })?,
        );

        let similarity_matcher = SimilarityMatcher::new();
        let suggestion_cache = dashmap::DashMap::new();

        Ok(Self {
            schema_manager,
            model_provider,
            similarity_matcher,
            suggestion_cache,
        })
    }

    /// Generate path suggestions for a partial path
    pub async fn generate_path_suggestions(
        &self,
        resource_type: &str,
        partial_path: &str,
    ) -> Result<Vec<PathSuggestion>, AnalyzerError> {
        // Create cache key
        let cache_key = format!("{}#{}", resource_type, partial_path);

        // Check cache first
        if let Some(cached_suggestions) = self.suggestion_cache.get(&cache_key) {
            return Ok(cached_suggestions
                .iter()
                .map(|es| es.base_suggestion.clone())
                .collect());
        }

        // Generate new suggestions
        let base_suggestions = self
            .generate_base_suggestions(resource_type, partial_path)
            .await?;

        // Enhance suggestions with analyzer-specific information
        let mut enhanced_suggestions = Vec::new();
        for suggestion in base_suggestions {
            if let Ok(enhanced) = self.enhance_suggestion(suggestion, resource_type).await {
                enhanced_suggestions.push(enhanced);
            }
        }

        // Sort by relevance and confidence
        enhanced_suggestions.sort_by(|a, b| {
            // Primary sort: confidence score
            let conf_cmp = b
                .base_suggestion
                .confidence
                .partial_cmp(&a.base_suggestion.confidence)
                .unwrap_or(std::cmp::Ordering::Equal);

            if conf_cmp != std::cmp::Ordering::Equal {
                return conf_cmp;
            }

            // Secondary sort: usage frequency
            b.usage_stats
                .frequency_score
                .partial_cmp(&a.usage_stats.frequency_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Cache the results
        self.suggestion_cache
            .insert(cache_key, enhanced_suggestions.clone());

        // Return base suggestions
        Ok(enhanced_suggestions
            .iter()
            .map(|es| es.base_suggestion.clone())
            .collect())
    }

    /// Generate base suggestions using similarity matching
    async fn generate_base_suggestions(
        &self,
        resource_type: &str,
        partial_path: &str,
    ) -> Result<Vec<PathSuggestion>, AnalyzerError> {
        let mut suggestions = Vec::new();

        // Get type information for the resource
        if let Some(type_info) = self.model_provider.get_type_reflection(resource_type).await {
            if let octofhir_fhirpath_model::provider::TypeReflectionInfo::ClassInfo {
                elements,
                ..
            } = type_info
            {
                for element in &elements {
                    let similarity = self
                        .similarity_matcher
                        .calculate_similarity(partial_path, &element.name);

                    if similarity > 0.3 {
                        // Lower threshold for broader suggestions
                        let suggestion = PathSuggestion {
                            property_name: element.name.clone(),
                            full_path: format!("{}.{}", resource_type, element.name),
                            confidence: similarity,
                            description: element.documentation.clone(),
                            property_type: self.extract_type_name(&element.type_info),
                            is_collection: element.max_cardinality.map_or(false, |max| max > 1),
                        };

                        suggestions.push(suggestion);
                    }
                }

                // Also check for exact prefix matches (higher confidence)
                for element in elements.iter() {
                    if element.name.starts_with(partial_path) && !partial_path.is_empty() {
                        // Check if we already have this suggestion
                        let exists = suggestions.iter().any(|s| s.property_name == element.name);

                        if !exists {
                            let suggestion = PathSuggestion {
                                property_name: element.name.clone(),
                                full_path: format!("{}.{}", resource_type, element.name),
                                confidence: 0.9, // High confidence for prefix matches
                                description: element.documentation.clone(),
                                property_type: self.extract_type_name(&element.type_info),
                                is_collection: element.max_cardinality.map_or(false, |max| max > 1),
                            };

                            suggestions.push(suggestion);
                        } else {
                            // Update confidence for existing suggestion
                            if let Some(existing) = suggestions
                                .iter_mut()
                                .find(|s| s.property_name == element.name)
                            {
                                existing.confidence = existing.confidence.max(0.9);
                            }
                        }
                    }
                }
            }
        }

        // Sort by confidence
        suggestions.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit to reasonable number
        suggestions.truncate(10);

        Ok(suggestions)
    }

    /// Enhance a suggestion with analyzer-specific information
    async fn enhance_suggestion(
        &self,
        suggestion: PathSuggestion,
        resource_type: &str,
    ) -> Result<EnhancedPathSuggestion, AnalyzerError> {
        // Get property information
        let property_info = self
            .get_property_info_for_suggestion(resource_type, &suggestion.property_name)
            .await?;

        // Calculate usage statistics
        let usage_stats = self
            .calculate_usage_statistics(&suggestion.full_path)
            .await?;

        // Calculate complexity score
        let complexity_score = self.calculate_complexity_score(&property_info).await?;

        // Generate optimization hints
        let optimization_hints = self.generate_path_optimization_hints(&property_info);

        Ok(EnhancedPathSuggestion {
            base_suggestion: suggestion,
            property_info,
            usage_stats,
            complexity_score,
            optimization_hints,
        })
    }

    /// Get property information for a suggestion
    async fn get_property_info_for_suggestion(
        &self,
        resource_type: &str,
        property_name: &str,
    ) -> Result<super::bridge_field_validator::PropertyInfo, AnalyzerError> {
        if let Some(type_info) = self.model_provider.get_type_reflection(resource_type).await {
            if let octofhir_fhirpath_model::provider::TypeReflectionInfo::ClassInfo {
                elements,
                ..
            } = type_info
            {
                for element in elements {
                    if element.name == property_name {
                        use super::bridge_field_validator::{BridgeCardinality, PropertyInfo};

                        let cardinality = BridgeCardinality {
                            min: if element.is_required() { 1 } else { 0 },
                            max: element.max_cardinality,
                            cardinality_string: format!(
                                "{}..{}",
                                if element.is_required() { 1 } else { 0 },
                                element
                                    .max_cardinality
                                    .map_or("*".to_string(), |m| m.to_string())
                            ),
                        };

                        return Ok(PropertyInfo {
                            name: element.name.clone(),
                            property_type: self.extract_type_name(&element.type_info),
                            cardinality,
                            is_choice_type: element.name.ends_with("[x]"),
                            choice_alternatives: Vec::new(), // TODO: Implement choice alternatives
                        });
                    }
                }
            }
        }

        Err(AnalyzerError::BridgeApiError {
            message: format!(
                "Property '{}' not found in resource type '{}'",
                property_name, resource_type
            ),
        })
    }

    /// Calculate usage statistics for a path
    async fn calculate_usage_statistics(
        &self,
        full_path: &str,
    ) -> Result<UsageStatistics, AnalyzerError> {
        // TODO: Implement real usage statistics based on query patterns
        // For now, return mock statistics based on common FHIR patterns

        let frequency_score = if full_path.contains(".id") || full_path.contains(".resourceType") {
            0.9 // Very common paths
        } else if full_path.contains(".name") || full_path.contains(".identifier") {
            0.8 // Common paths
        } else if full_path.contains(".extension") || full_path.contains(".meta") {
            0.4 // Less common paths
        } else {
            0.6 // Default frequency
        };

        let performance_score = if full_path.contains("[") || full_path.contains("where") {
            0.5 // Complex navigation
        } else if full_path.split('.').count() > 3 {
            0.6 // Deep navigation
        } else {
            0.8 // Simple navigation
        };

        let correlation_score = if full_path.contains(".name") || full_path.contains(".identifier")
        {
            0.7 // Often used with other identifiers
        } else {
            0.5 // Default correlation
        };

        Ok(UsageStatistics {
            frequency_score,
            performance_score,
            correlation_score,
        })
    }

    /// Calculate complexity score for a property
    async fn calculate_complexity_score(
        &self,
        property_info: &super::bridge_field_validator::PropertyInfo,
    ) -> Result<f64, AnalyzerError> {
        let mut complexity: f64 = 0.0;

        // Base complexity
        complexity += 0.1;

        // Collection complexity
        if property_info.cardinality.max.is_none() || property_info.cardinality.max.unwrap_or(0) > 1
        {
            complexity += 0.2;
        }

        // Choice type complexity
        if property_info.is_choice_type {
            complexity += 0.3;
        }

        // Type complexity
        if property_info.property_type.contains("Complex")
            || property_info.property_type.contains("Reference")
        {
            complexity += 0.2;
        }

        Ok(complexity.min(1.0))
    }

    /// Generate optimization hints for a property
    fn generate_path_optimization_hints(
        &self,
        property_info: &super::bridge_field_validator::PropertyInfo,
    ) -> Vec<String> {
        let mut hints = Vec::new();

        if property_info.cardinality.max.is_none() || property_info.cardinality.max.unwrap_or(0) > 1
        {
            hints.push("This is a collection property - consider using .first() or .exists() for single value access".to_string());
        }

        if property_info.is_choice_type {
            hints.push("This is a choice type - use specific concrete types like valueQuantity or valueString".to_string());
        }

        if property_info.property_type.contains("Reference") {
            hints.push(
                "This is a reference property - use resolve() to navigate to referenced resources"
                    .to_string(),
            );
        }

        if hints.is_empty() {
            hints.push("Direct property access with good performance".to_string());
        }

        hints
    }

    /// Extract type name from type reflection info
    fn extract_type_name(
        &self,
        type_info: &octofhir_fhirpath_model::provider::TypeReflectionInfo,
    ) -> String {
        use octofhir_fhirpath_model::provider::TypeReflectionInfo;

        match type_info {
            TypeReflectionInfo::SimpleType { name, .. } => name.clone(),
            TypeReflectionInfo::ClassInfo { name, .. } => name.clone(),
            TypeReflectionInfo::ListType { element_type } => {
                format!("List<{}>", self.extract_type_name(element_type))
            }
            TypeReflectionInfo::TupleType { .. } => "Tuple".to_string(),
        }
    }

    /// Clear suggestion cache
    pub fn clear_cache(&self) {
        self.suggestion_cache.clear();
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> NavigationCacheStats {
        NavigationCacheStats {
            total_entries: self.suggestion_cache.len(),
            memory_usage_estimate: self.suggestion_cache.len()
                * std::mem::size_of::<EnhancedPathSuggestion>(),
        }
    }

    /// Get schema manager reference
    pub fn schema_manager(&self) -> &Arc<FhirSchemaPackageManager> {
        &self.schema_manager
    }
}

/// Navigation cache statistics
#[derive(Debug, Clone)]
pub struct NavigationCacheStats {
    /// Total number of cached entries
    pub total_entries: usize,
    /// Estimated memory usage in bytes
    pub memory_usage_estimate: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_canonical_manager::FcmConfig;
    use octofhir_fhirschema::PackageManagerConfig;

    async fn create_test_navigator() -> Result<AnalyzerPathNavigator, AnalyzerError> {
        let fcm_config = FcmConfig::default();
        let config = PackageManagerConfig::default();
        let schema_manager = Arc::new(
            FhirSchemaPackageManager::new(fcm_config, config)
                .await
                .map_err(|e| AnalyzerError::InitializationError {
                    message: format!("Failed to create schema manager: {}", e),
                })?,
        );

        AnalyzerPathNavigator::new(schema_manager).await
    }

    #[tokio::test]
    async fn test_path_suggestions() -> Result<(), Box<dyn std::error::Error>> {
        let navigator = create_test_navigator().await?;

        let suggestions = navigator
            .generate_path_suggestions("Patient", "nam")
            .await?;

        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.property_name.contains("name")));

        Ok(())
    }

    #[tokio::test]
    async fn test_usage_statistics() -> Result<(), Box<dyn std::error::Error>> {
        let navigator = create_test_navigator().await?;

        let stats = navigator
            .calculate_usage_statistics("Patient.name.given")
            .await?;

        assert!(stats.frequency_score > 0.0);
        assert!(stats.performance_score > 0.0);
        assert!(stats.correlation_score > 0.0);

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_functionality() -> Result<(), Box<dyn std::error::Error>> {
        let navigator = create_test_navigator().await?;

        // First call should populate cache
        let _suggestions1 = navigator
            .generate_path_suggestions("Patient", "nam")
            .await?;

        let stats_before = navigator.get_cache_stats();
        assert_eq!(stats_before.total_entries, 1);

        // Second call should use cache
        let _suggestions2 = navigator
            .generate_path_suggestions("Patient", "nam")
            .await?;

        let stats_after = navigator.get_cache_stats();
        assert_eq!(stats_after.total_entries, 1); // Same entry

        Ok(())
    }
}
