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

//! Enhanced metadata system for FHIRPath operators
//!
//! This module provides a comprehensive metadata system for operators that includes
//! performance characteristics, LSP support, usage patterns, and rich documentation.

use crate::unified_operator::Associativity;
use serde::{Deserialize, Serialize};

/// Enhanced metadata for FHIRPath operators
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnhancedOperatorMetadata {
    /// Basic operator information
    pub basic: BasicOperatorMetadata,
    
    /// Performance characteristics
    pub performance: OperatorPerformanceMetadata,
    
    /// Language Server Protocol support
    pub lsp: OperatorLspMetadata,
    
    /// Usage patterns and examples
    pub usage: OperatorUsageMetadata,
    
    /// Type system information
    pub types: OperatorTypeMetadata,
    
    /// FHIRPath specification compliance
    pub compliance: OperatorComplianceMetadata,
}

/// Basic operator metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BasicOperatorMetadata {
    /// Operator symbol (e.g., "+", "=", "and")
    pub symbol: String,
    
    /// Human-readable display name
    pub display_name: String,
    
    /// Detailed description of the operator
    pub description: String,
    
    /// Operator category for grouping
    pub category: OperatorCategory,
    
    /// Operator precedence (higher values bind tighter)
    pub precedence: u8,
    
    /// Operator associativity
    pub associativity: Associativity,
    
    /// Whether the operator is pure (deterministic with no side effects)
    pub is_pure: bool,
    
    /// Whether the operator is commutative (a op b = b op a)
    pub is_commutative: bool,
    
    /// Whether the operator supports both binary and unary forms
    pub supports_unary: bool,
    
    /// Whether the operator supports binary form
    pub supports_binary: bool,
}

/// Performance characteristics of the operator
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperatorPerformanceMetadata {
    /// Computational complexity
    pub complexity: OperatorComplexity,
    
    /// Memory usage pattern
    pub memory_usage: OperatorMemoryUsage,
    
    /// Whether the operator can be optimized
    pub optimizable: bool,
    
    /// Whether the operator benefits from operand ordering
    pub order_sensitive: bool,
    
    /// Whether the operator can short-circuit evaluation
    pub short_circuits: bool,
}

/// LSP (Language Server Protocol) support metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperatorLspMetadata {
    /// Completion snippet for the operator
    pub snippet: String,
    
    /// When to show this operator in completions
    pub completion_visibility: OperatorCompletionVisibility,
    
    /// Keywords for search and filtering
    pub keywords: Vec<String>,
    
    /// Hover documentation
    pub hover_documentation: String,
    
    /// Symbol kind for LSP
    pub symbol_kind: OperatorSymbolKind,
}

/// Usage patterns and examples
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperatorUsageMetadata {
    /// Usage examples
    pub examples: Vec<OperatorExample>,
    
    /// Common patterns where this operator is used
    pub patterns: Vec<OperatorPattern>,
    
    /// Related operators
    pub related_operators: Vec<String>,
    
    /// Common mistakes and how to avoid them
    pub common_mistakes: Vec<String>,
}

/// Type system information for the operator
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperatorTypeMetadata {
    /// Supported type combinations
    pub type_signatures: Vec<OperatorTypeSignature>,
    
    /// Whether the operator performs type coercion
    pub performs_coercion: bool,
    
    /// Whether the operator requires exact type matches
    pub requires_exact_types: bool,
    
    /// Default result type when types are ambiguous
    pub default_result_type: Option<String>,
}

/// FHIRPath specification compliance information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperatorComplianceMetadata {
    /// FHIRPath specification version
    pub spec_version: String,
    
    /// Whether this is a standard FHIRPath operator
    pub is_standard: bool,
    
    /// Extension information if not standard
    pub extension_info: Option<OperatorExtensionInfo>,
    
    /// Known compatibility issues
    pub compatibility_notes: Vec<String>,
}

/// Operator categories for grouping and organization
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OperatorCategory {
    /// Arithmetic operations (+, -, *, /, etc.)
    Arithmetic,
    /// Comparison operations (=, !=, <, >, etc.)
    Comparison,
    /// Logical operations (and, or, not, xor)
    Logical,
    /// String operations (&, concatenation)
    String,
    /// Collection operations (in, contains)
    Collection,
    /// Type operations (is, as)
    Type,
    /// Membership operations (in, contains)
    Membership,
    /// Custom/extension operators
    Extension,
}

/// Computational complexity patterns for operators
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OperatorComplexity {
    /// Constant time O(1)
    Constant,
    /// Linear time O(n)
    Linear,
    /// Quadratic time O(n²)
    Quadratic,
    /// Logarithmic time O(log n)
    Logarithmic,
    /// Depends on operand types
    TypeDependent,
}

/// Memory usage patterns for operators
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OperatorMemoryUsage {
    /// Minimal memory allocation
    Minimal,
    /// Memory proportional to input size
    Linear,
    /// Potential for large memory allocations
    High,
    /// Streaming/constant memory usage
    Streaming,
}

/// When to show operators in completions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OperatorCompletionVisibility {
    /// Always show in completions
    Always,
    /// Show only in expression contexts
    ExpressionOnly,
    /// Show only when operands are present
    WithOperands,
    /// Show only in advanced mode
    Advanced,
    /// Never show (deprecated/internal)
    Never,
}

/// LSP symbol kinds for operators
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OperatorSymbolKind {
    /// Standard operator symbol
    Operator,
    /// Keyword operator (and, or, not)
    Keyword,
    /// Function-like operator
    Function,
}

/// Usage example for an operator
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperatorExample {
    /// Example expression
    pub expression: String,
    /// Description of what the example demonstrates
    pub description: String,
    /// Expected result (if applicable)
    pub expected_result: Option<String>,
    /// Context where this example is useful
    pub context: String,
}

/// Common usage pattern for an operator
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperatorPattern {
    /// Pattern name
    pub name: String,
    /// Pattern template
    pub template: String,
    /// Description of when to use this pattern
    pub description: String,
}

/// Type signature for an operator
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperatorTypeSignature {
    /// Left operand type (None for unary operators)
    pub left_type: Option<String>,
    /// Right operand type
    pub right_type: String,
    /// Result type
    pub result_type: String,
    /// Whether this signature is preferred
    pub is_preferred: bool,
}

/// Extension information for non-standard operators
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperatorExtensionInfo {
    /// Extension namespace
    pub namespace: String,
    /// Extension version
    pub version: String,
    /// Extension author/organization
    pub author: String,
    /// Extension documentation URL
    pub documentation_url: Option<String>,
}

impl EnhancedOperatorMetadata {
    /// Create basic metadata with minimal information
    pub fn basic(symbol: &str, category: OperatorCategory, precedence: u8, associativity: Associativity) -> Self {
        Self {
            basic: BasicOperatorMetadata {
                symbol: symbol.to_string(),
                display_name: symbol.to_string(),
                description: format!("The {} operator", symbol),
                category,
                precedence,
                associativity,
                is_pure: true,
                is_commutative: false,
                supports_unary: false,
                supports_binary: true,
            },
            performance: OperatorPerformanceMetadata {
                complexity: OperatorComplexity::Constant,
                memory_usage: OperatorMemoryUsage::Minimal,
                optimizable: true,
                order_sensitive: true,
                short_circuits: false,
            },
            lsp: OperatorLspMetadata {
                snippet: format!(" {} ", symbol),
                completion_visibility: OperatorCompletionVisibility::Always,
                keywords: vec![symbol.to_string()],
                hover_documentation: format!("The {} operator", symbol),
                symbol_kind: if symbol.chars().all(|c| c.is_alphabetic()) {
                    OperatorSymbolKind::Keyword
                } else {
                    OperatorSymbolKind::Operator
                },
            },
            usage: OperatorUsageMetadata {
                examples: vec![],
                patterns: vec![],
                related_operators: vec![],
                common_mistakes: vec![],
            },
            types: OperatorTypeMetadata {
                type_signatures: vec![],
                performs_coercion: false,
                requires_exact_types: false,
                default_result_type: None,
            },
            compliance: OperatorComplianceMetadata {
                spec_version: "R4".to_string(),
                is_standard: true,
                extension_info: None,
                compatibility_notes: vec![],
            },
        }
    }
    
    /// Get all keywords for search functionality
    pub fn all_keywords(&self) -> Vec<&str> {
        let mut keywords = vec![
            self.basic.symbol.as_str(),
            self.basic.display_name.as_str(),
        ];
        keywords.extend(self.lsp.keywords.iter().map(|s| s.as_str()));
        keywords
    }
    
    /// Check if this operator matches a search query
    pub fn matches_search(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.all_keywords().iter().any(|keyword| {
            keyword.to_lowercase().contains(&query_lower)
        })
    }
    
    /// Get completion snippet with placeholders
    pub fn completion_snippet(&self) -> String {
        if self.basic.supports_binary {
            format!("${{1:operand}} {} ${{2:operand}}", self.basic.symbol)
        } else {
            format!("{} ${{1:operand}}", self.basic.symbol)
        }
    }
}

impl std::fmt::Display for OperatorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperatorCategory::Arithmetic => write!(f, "Arithmetic"),
            OperatorCategory::Comparison => write!(f, "Comparison"),
            OperatorCategory::Logical => write!(f, "Logical"),
            OperatorCategory::String => write!(f, "String"),
            OperatorCategory::Collection => write!(f, "Collection"),
            OperatorCategory::Type => write!(f, "Type"),
            OperatorCategory::Membership => write!(f, "Membership"),
            OperatorCategory::Extension => write!(f, "Extension"),
        }
    }
}

impl std::fmt::Display for OperatorComplexity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperatorComplexity::Constant => write!(f, "O(1)"),
            OperatorComplexity::Linear => write!(f, "O(n)"),
            OperatorComplexity::Quadratic => write!(f, "O(n²)"),
            OperatorComplexity::Logarithmic => write!(f, "O(log n)"),
            OperatorComplexity::TypeDependent => write!(f, "Type Dependent"),
        }
    }
}

impl std::fmt::Display for OperatorMemoryUsage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperatorMemoryUsage::Minimal => write!(f, "Minimal"),
            OperatorMemoryUsage::Linear => write!(f, "Linear"),
            OperatorMemoryUsage::High => write!(f, "High"),
            OperatorMemoryUsage::Streaming => write!(f, "Streaming"),
        }
    }
}

/// Builder for creating enhanced operator metadata
pub struct OperatorMetadataBuilder {
    metadata: EnhancedOperatorMetadata,
}

impl OperatorMetadataBuilder {
    /// Create a new builder with basic information
    pub fn new(symbol: &str, category: OperatorCategory, precedence: u8, associativity: Associativity) -> Self {
        Self {
            metadata: EnhancedOperatorMetadata::basic(symbol, category, precedence, associativity),
        }
    }
    
    /// Set the display name
    pub fn display_name(mut self, name: &str) -> Self {
        self.metadata.basic.display_name = name.to_string();
        self
    }
    
    /// Set the description
    pub fn description(mut self, description: &str) -> Self {
        self.metadata.basic.description = description.to_string();
        self
    }
    
    /// Mark as commutative
    pub fn commutative(mut self, is_commutative: bool) -> Self {
        self.metadata.basic.is_commutative = is_commutative;
        self
    }
    
    /// Set unary support
    pub fn supports_unary(mut self, supports: bool) -> Self {
        self.metadata.basic.supports_unary = supports;
        self
    }
    
    /// Set complexity
    pub fn complexity(mut self, complexity: OperatorComplexity) -> Self {
        self.metadata.performance.complexity = complexity;
        self
    }
    
    /// Set memory usage
    pub fn memory_usage(mut self, usage: OperatorMemoryUsage) -> Self {
        self.metadata.performance.memory_usage = usage;
        self
    }
    
    /// Mark as short-circuiting
    pub fn short_circuits(mut self, short_circuits: bool) -> Self {
        self.metadata.performance.short_circuits = short_circuits;
        self
    }
    
    /// Add an example
    pub fn example(mut self, expression: &str, description: &str) -> Self {
        self.metadata.usage.examples.push(OperatorExample {
            expression: expression.to_string(),
            description: description.to_string(),
            expected_result: None,
            context: "General usage".to_string(),
        });
        self
    }
    
    /// Add keywords
    pub fn keywords(mut self, keywords: Vec<&str>) -> Self {
        self.metadata.lsp.keywords = keywords.iter().map(|s| s.to_string()).collect();
        self
    }
    
    /// Set completion visibility
    pub fn completion_visibility(mut self, visibility: OperatorCompletionVisibility) -> Self {
        self.metadata.lsp.completion_visibility = visibility;
        self
    }
    
    /// Build the metadata
    pub fn build(self) -> EnhancedOperatorMetadata {
        self.metadata
    }
}