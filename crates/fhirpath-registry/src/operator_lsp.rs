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

//! Language Server Protocol (LSP) support for FHIRPath operators

use crate::enhanced_operator_metadata::{
    EnhancedOperatorMetadata, OperatorCategory, OperatorCompletionVisibility, 
    OperatorSymbolKind,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// LSP completion item for an operator
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperatorCompletionItem {
    /// The operator symbol
    pub symbol: String,
    
    /// Display name for the completion item
    pub display_name: String,
    
    /// Detailed description
    pub description: String,
    
    /// Completion snippet with placeholders
    pub snippet: String,
    
    /// Operator category
    pub category: OperatorCategory,
    
    /// When to show this completion
    pub visibility: OperatorCompletionVisibility,
    
    /// LSP symbol kind
    pub symbol_kind: OperatorSymbolKind,
    
    /// Keywords for filtering
    pub keywords: Vec<String>,
    
    /// Precedence for sorting
    pub precedence: u8,
    
    /// Whether operator is commutative
    pub is_commutative: bool,
    
    /// Usage examples
    pub examples: Vec<OperatorExample>,
}

/// Example usage of an operator for LSP
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperatorExample {
    /// Example expression
    pub expression: String,
    
    /// Description of the example
    pub description: String,
    
    /// Expected result (if applicable)
    pub expected_result: Option<String>,
}

/// Hover information for an operator
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperatorHoverInfo {
    /// The operator symbol
    pub symbol: String,
    
    /// Display name
    pub display_name: String,
    
    /// Detailed description
    pub description: String,
    
    /// Type signatures
    pub signatures: Vec<String>,
    
    /// Usage examples
    pub examples: Vec<OperatorExample>,
    
    /// Performance information
    pub performance_info: Option<String>,
    
    /// Related operators
    pub related_operators: Vec<String>,
    
    /// Common mistakes to avoid
    pub common_mistakes: Vec<String>,
}

/// Diagnostic information for operator usage
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperatorDiagnostic {
    /// Severity level
    pub severity: DiagnosticSeverity,
    
    /// Diagnostic message
    pub message: String,
    
    /// Suggested fix (if applicable)
    pub suggested_fix: Option<String>,
    
    /// Related operator symbol
    pub operator_symbol: String,
}

/// Diagnostic severity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    /// Error that prevents compilation/evaluation
    Error,
    /// Warning about potential issues
    Warning,
    /// Informational message
    Info,
    /// Hint for improvement
    Hint,
}

/// LSP support provider for operators
#[derive(Clone)]
pub struct OperatorLspProvider {
    /// Cached completion items by category
    completions_by_category: HashMap<OperatorCategory, Vec<OperatorCompletionItem>>,
    
    /// Cached completion items by symbol
    completions_by_symbol: HashMap<String, OperatorCompletionItem>,
    
    /// Hover information cache
    hover_cache: HashMap<String, OperatorHoverInfo>,
    
    /// All available operators
    operators: Vec<EnhancedOperatorMetadata>,
}

impl OperatorLspProvider {
    /// Create a new LSP provider
    pub fn new() -> Self {
        Self {
            completions_by_category: HashMap::new(),
            completions_by_symbol: HashMap::new(),
            hover_cache: HashMap::new(),
            operators: Vec::new(),
        }
    }

    /// Register an operator with the LSP provider
    pub fn register_operator(&mut self, metadata: EnhancedOperatorMetadata) {
        let completion_item = self.create_completion_item(&metadata);
        let hover_info = self.create_hover_info(&metadata);
        
        // Cache by category
        self.completions_by_category
            .entry(metadata.basic.category)
            .or_insert_with(Vec::new)
            .push(completion_item.clone());
        
        // Cache by symbol
        self.completions_by_symbol
            .insert(metadata.basic.symbol.clone(), completion_item);
        
        // Cache hover info
        self.hover_cache
            .insert(metadata.basic.symbol.clone(), hover_info);
        
        // Store metadata
        self.operators.push(metadata);
    }

    /// Get completion items for a given context
    pub fn get_completions(&self, context: &CompletionContext) -> Vec<OperatorCompletionItem> {
        let mut completions = Vec::new();
        
        // Filter by category if specified
        if let Some(category) = context.preferred_category {
            if let Some(category_completions) = self.completions_by_category.get(&category) {
                completions.extend(category_completions.iter().cloned());
            }
        } else {
            // Return all completions
            completions.extend(self.completions_by_symbol.values().cloned());
        }
        
        // Filter by visibility
        completions.retain(|item| {
            self.should_show_completion(item, context)
        });
        
        // Filter by query
        if let Some(query) = &context.query {
            completions.retain(|item| {
                self.matches_query(item, query)
            });
        }
        
        // Sort by relevance
        completions.sort_by(|a, b| {
            // First by category relevance
            let category_cmp = self.category_relevance(a.category, context)
                .cmp(&self.category_relevance(b.category, context))
                .reverse();
            
            if category_cmp != std::cmp::Ordering::Equal {
                return category_cmp;
            }
            
            // Then by precedence (higher precedence first for operators)
            let precedence_cmp = a.precedence.cmp(&b.precedence).reverse();
            
            if precedence_cmp != std::cmp::Ordering::Equal {
                return precedence_cmp;
            }
            
            // Finally by symbol alphabetically
            a.symbol.cmp(&b.symbol)
        });
        
        completions
    }

    /// Get hover information for an operator
    pub fn get_hover_info(&self, symbol: &str) -> Option<&OperatorHoverInfo> {
        self.hover_cache.get(symbol)
    }

    /// Get diagnostics for operator usage
    pub fn get_diagnostics(&self, expression: &str, _cursor_position: usize) -> Vec<OperatorDiagnostic> {
        let mut diagnostics = Vec::new();
        
        // Simple analysis - look for common operator mistakes
        for operator in &self.operators {
            let symbol = &operator.basic.symbol;
            
            // Check for common mistakes mentioned in metadata
            for mistake in &operator.usage.common_mistakes {
                if expression.contains(symbol) && self.matches_common_mistake(expression, mistake) {
                    diagnostics.push(OperatorDiagnostic {
                        severity: DiagnosticSeverity::Warning,
                        message: mistake.clone(),
                        suggested_fix: None,
                        operator_symbol: symbol.clone(),
                    });
                }
            }
        }
        
        diagnostics
    }

    /// Search operators by query
    pub fn search_operators(&self, query: &str) -> Vec<&EnhancedOperatorMetadata> {
        let query_lower = query.to_lowercase();
        
        self.operators
            .iter()
            .filter(|op| op.matches_search(&query_lower))
            .collect()
    }

    /// Get operators by category
    pub fn get_operators_by_category(&self, category: OperatorCategory) -> Vec<&EnhancedOperatorMetadata> {
        self.operators
            .iter()
            .filter(|op| op.basic.category == category)
            .collect()
    }

    /// Create completion item from metadata
    fn create_completion_item(&self, metadata: &EnhancedOperatorMetadata) -> OperatorCompletionItem {
        OperatorCompletionItem {
            symbol: metadata.basic.symbol.clone(),
            display_name: metadata.basic.display_name.clone(),
            description: metadata.basic.description.clone(),
            snippet: metadata.completion_snippet(),
            category: metadata.basic.category,
            visibility: metadata.lsp.completion_visibility,
            symbol_kind: metadata.lsp.symbol_kind,
            keywords: metadata.lsp.keywords.clone(),
            precedence: metadata.basic.precedence,
            is_commutative: metadata.basic.is_commutative,
            examples: metadata.usage.examples.iter().map(|ex| OperatorExample {
                expression: ex.expression.clone(),
                description: ex.description.clone(),
                expected_result: ex.expected_result.clone(),
            }).collect(),
        }
    }

    /// Create hover info from metadata
    fn create_hover_info(&self, metadata: &EnhancedOperatorMetadata) -> OperatorHoverInfo {
        let signatures = metadata.types.type_signatures.iter().map(|sig| {
            match &sig.left_type {
                Some(left) => format!("{} {} {} -> {}", left, metadata.basic.symbol, sig.right_type, sig.result_type),
                None => format!("{} {} -> {}", metadata.basic.symbol, sig.right_type, sig.result_type),
            }
        }).collect();

        let performance_info = if metadata.performance.optimizable || metadata.performance.short_circuits {
            let mut info = Vec::new();
            if metadata.performance.optimizable {
                info.push("Optimizable".to_string());
            }
            if metadata.performance.short_circuits {
                info.push("Short-circuits evaluation".to_string());
            }
            Some(format!("Performance: {}", info.join(", ")))
        } else {
            None
        };

        OperatorHoverInfo {
            symbol: metadata.basic.symbol.clone(),
            display_name: metadata.basic.display_name.clone(),
            description: metadata.lsp.hover_documentation.clone(),
            signatures,
            examples: metadata.usage.examples.iter().map(|ex| OperatorExample {
                expression: ex.expression.clone(),
                description: ex.description.clone(),
                expected_result: ex.expected_result.clone(),
            }).collect(),
            performance_info,
            related_operators: metadata.usage.related_operators.clone(),
            common_mistakes: metadata.usage.common_mistakes.clone(),
        }
    }

    /// Check if completion should be shown in context
    fn should_show_completion(&self, item: &OperatorCompletionItem, context: &CompletionContext) -> bool {
        match item.visibility {
            OperatorCompletionVisibility::Always => true,
            OperatorCompletionVisibility::ExpressionOnly => context.in_expression,
            OperatorCompletionVisibility::WithOperands => context.has_operands,
            OperatorCompletionVisibility::Advanced => context.show_advanced,
            OperatorCompletionVisibility::Never => false,
        }
    }

    /// Check if item matches query
    fn matches_query(&self, item: &OperatorCompletionItem, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        
        // Check symbol
        if item.symbol.to_lowercase().contains(&query_lower) {
            return true;
        }
        
        // Check display name
        if item.display_name.to_lowercase().contains(&query_lower) {
            return true;
        }
        
        // Check keywords
        item.keywords.iter().any(|keyword| {
            keyword.to_lowercase().contains(&query_lower)
        })
    }

    /// Get category relevance score for context
    fn category_relevance(&self, category: OperatorCategory, context: &CompletionContext) -> u8 {
        match (category, context.preferred_category) {
            (cat1, Some(cat2)) if cat1 == cat2 => 10, // Perfect match
            (OperatorCategory::Comparison, _) => 8, // Comparisons are very common
            (OperatorCategory::Arithmetic, _) => 7, // Arithmetic is common
            (OperatorCategory::Logical, _) => 6, // Logical operations are common
            (OperatorCategory::String, _) => 5, // String operations
            (OperatorCategory::Collection, _) => 4, // Collection operations
            (OperatorCategory::Type, _) => 3, // Type operations
            (OperatorCategory::Membership, _) => 2, // Membership operations
            (OperatorCategory::Extension, _) => 1, // Extensions are less common
        }
    }

    /// Check if expression matches a common mistake pattern
    fn matches_common_mistake(&self, expression: &str, mistake: &str) -> bool {
        // Simple pattern matching - can be enhanced with regex
        mistake.to_lowercase().split_whitespace().any(|word| {
            expression.to_lowercase().contains(word)
        })
    }
}

impl Default for OperatorLspProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// Context for operator completions
#[derive(Debug, Clone)]
pub struct CompletionContext {
    /// Query string for filtering
    pub query: Option<String>,
    
    /// Preferred operator category
    pub preferred_category: Option<OperatorCategory>,
    
    /// Whether we're in an expression context
    pub in_expression: bool,
    
    /// Whether operands are available
    pub has_operands: bool,
    
    /// Whether to show advanced operators
    pub show_advanced: bool,
    
    /// Current cursor position in the expression
    pub _cursor_position: usize,
}

impl Default for CompletionContext {
    fn default() -> Self {
        Self {
            query: None,
            preferred_category: None,
            in_expression: true,
            has_operands: false,
            show_advanced: false,
            _cursor_position: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enhanced_operator_metadata::*;
    use crate::unified_operator::Associativity;

    fn create_test_operator() -> EnhancedOperatorMetadata {
        OperatorMetadataBuilder::new(
            "+",
            OperatorCategory::Arithmetic,
            12,
            Associativity::Left,
        )
        .display_name("Addition")
        .description("Adds two numbers together")
        .example("3 + 2", "Basic addition")
        .keywords(vec!["add", "plus", "arithmetic"])
        .build()
    }

    #[test]
    fn test_lsp_provider_registration() {
        let mut provider = OperatorLspProvider::new();
        let metadata = create_test_operator();
        
        provider.register_operator(metadata);
        
        assert_eq!(provider.operators.len(), 1);
        assert!(provider.completions_by_symbol.contains_key("+"));
        assert!(provider.hover_cache.contains_key("+"));
    }

    #[test]
    fn test_completions_filtering() {
        let mut provider = OperatorLspProvider::new();
        provider.register_operator(create_test_operator());
        
        let context = CompletionContext {
            query: Some("add".to_string()),
            ..Default::default()
        };
        
        let completions = provider.get_completions(&context);
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].symbol, "+");
    }

    #[test]
    fn test_hover_info() {
        let mut provider = OperatorLspProvider::new();
        provider.register_operator(create_test_operator());
        
        let hover = provider.get_hover_info("+");
        assert!(hover.is_some());
        
        let hover = hover.unwrap();
        assert_eq!(hover.symbol, "+");
        assert_eq!(hover.display_name, "Addition");
    }

    #[test]
    fn test_search_operators() {
        let mut provider = OperatorLspProvider::new();
        provider.register_operator(create_test_operator());
        
        let results = provider.search_operators("arithmetic");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].basic.symbol, "+");
    }

    #[test]
    fn test_category_filtering() {
        let mut provider = OperatorLspProvider::new();
        provider.register_operator(create_test_operator());
        
        let arithmetic_ops = provider.get_operators_by_category(OperatorCategory::Arithmetic);
        assert_eq!(arithmetic_ops.len(), 1);
        
        let logical_ops = provider.get_operators_by_category(OperatorCategory::Logical);
        assert_eq!(logical_ops.len(), 0);
    }
}