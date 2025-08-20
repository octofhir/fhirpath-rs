//! Caching infrastructure for the FHIRPath analyzer

use crate::types::{
    AnalysisResult, ExpressionHash, FunctionCallAnalysis, FunctionSignature, NodeId, SemanticInfo,
    UnionTypeInfo,
};
use dashmap::DashMap;
use octofhir_fhirpath_ast::ExpressionNode;
use octofhir_fhirpath_model::types::TypeInfo;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// High-performance analysis cache using DashMap for concurrent access
#[derive(Debug)]
pub struct AnalysisCache {
    /// Expression analysis results
    expression_cache: DashMap<String, AnalysisResult>,
    /// Type resolution cache
    type_cache: DashMap<(String, String), TypeInfo>,
    /// Function signature cache
    function_cache: DashMap<String, FunctionSignature>,
    /// Node-specific semantic info
    semantic_cache: DashMap<ExpressionHash, SemanticInfo>,
}

impl AnalysisCache {
    /// Create a new analysis cache
    pub fn new() -> Self {
        Self {
            expression_cache: DashMap::new(),
            type_cache: DashMap::new(),
            function_cache: DashMap::new(),
            semantic_cache: DashMap::new(),
        }
    }

    /// Create a new analysis cache with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            expression_cache: DashMap::with_capacity(capacity),
            type_cache: DashMap::with_capacity(capacity * 2),
            function_cache: DashMap::with_capacity(capacity / 10),
            semantic_cache: DashMap::with_capacity(capacity * 5),
        }
    }

    /// Get cached analysis result for expression
    pub fn get_analysis(&self, expression: &str) -> Option<AnalysisResult> {
        self.expression_cache.get(expression).map(|r| r.clone())
    }

    /// Cache analysis result for expression
    pub fn cache_analysis(&self, expression: String, result: AnalysisResult) {
        self.expression_cache.insert(expression, result);
    }

    /// Get cached semantic info for node hash
    pub fn get_semantic_info(&self, node_hash: ExpressionHash) -> Option<SemanticInfo> {
        self.semantic_cache.get(&node_hash).map(|s| s.clone())
    }

    /// Cache semantic info for node hash
    pub fn cache_semantic_info(&self, node_hash: ExpressionHash, info: SemanticInfo) {
        self.semantic_cache.insert(node_hash, info);
    }

    /// Get cached type info for context and type name
    pub fn get_type_info(&self, context: &str, type_name: &str) -> Option<TypeInfo> {
        self.type_cache
            .get(&(context.to_string(), type_name.to_string()))
            .map(|t| t.clone())
    }

    /// Cache type info for context and type name
    pub fn cache_type_info(&self, context: String, type_name: String, type_info: TypeInfo) {
        self.type_cache.insert((context, type_name), type_info);
    }

    /// Get cached function signature
    pub fn get_function_signature(&self, function_name: &str) -> Option<FunctionSignature> {
        self.function_cache.get(function_name).map(|f| f.clone())
    }

    /// Cache function signature
    pub fn cache_function_signature(&self, function_name: String, signature: FunctionSignature) {
        self.function_cache.insert(function_name, signature);
    }

    /// Clear all caches
    pub fn clear(&self) {
        self.expression_cache.clear();
        self.type_cache.clear();
        self.function_cache.clear();
        self.semantic_cache.clear();
    }
}

impl Default for AnalysisCache {
    fn default() -> Self {
        Self::new()
    }
}

/// External analysis mapping (no AST changes required)
#[derive(Debug)]
pub struct ExpressionAnalysisMap {
    /// Map AST nodes to analysis info using content-based hashing
    node_analysis: HashMap<ExpressionHash, SemanticInfo>,
    /// Function call analysis without AST modification
    function_analysis: HashMap<ExpressionHash, FunctionCallAnalysis>,
    /// Union type mapping for children() calls
    union_analysis: HashMap<ExpressionHash, UnionTypeInfo>,
    /// Node ID counter for tracking
    next_node_id: NodeId,
}

impl ExpressionAnalysisMap {
    /// Create a new expression analysis map
    pub fn new() -> Self {
        Self {
            node_analysis: HashMap::new(),
            function_analysis: HashMap::new(),
            union_analysis: HashMap::new(),
            next_node_id: 1,
        }
    }

    /// Get next available node ID
    pub fn get_next_node_id(&mut self) -> NodeId {
        let node_id = self.next_node_id;
        self.next_node_id += 1;
        node_id
    }

    /// Generate stable hash for any AST node without modification
    pub fn hash_node(node: &ExpressionNode) -> ExpressionHash {
        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::new();
        node.hash(&mut hasher);
        hasher.finish()
    }

    /// Attach analysis info to existing AST node without changing it
    pub fn attach_analysis(&mut self, node: &ExpressionNode, info: SemanticInfo) -> NodeId {
        let hash = Self::hash_node(node);
        let node_id = self.next_node_id;
        self.next_node_id += 1;

        self.node_analysis.insert(hash, info);
        node_id
    }

    /// Get analysis info for an AST node
    pub fn get_analysis(&self, node: &ExpressionNode) -> Option<&SemanticInfo> {
        let hash = Self::hash_node(node);
        self.node_analysis.get(&hash)
    }

    /// Attach function call analysis
    pub fn attach_function_analysis(
        &mut self,
        node: &ExpressionNode,
        analysis: FunctionCallAnalysis,
    ) {
        let hash = Self::hash_node(node);
        self.function_analysis.insert(hash, analysis);
    }

    /// Get function call analysis
    pub fn get_function_analysis(&self, node: &ExpressionNode) -> Option<&FunctionCallAnalysis> {
        let hash = Self::hash_node(node);
        self.function_analysis.get(&hash)
    }

    /// Attach union type analysis for children() calls
    pub fn attach_union_analysis(&mut self, node: &ExpressionNode, union_info: UnionTypeInfo) {
        let hash = Self::hash_node(node);
        self.union_analysis.insert(hash, union_info);
    }

    /// Get union type analysis
    pub fn get_union_analysis(&self, node: &ExpressionNode) -> Option<&UnionTypeInfo> {
        let hash = Self::hash_node(node);
        self.union_analysis.get(&hash)
    }

    /// Visit all nodes in AST and collect hashes (for bulk analysis)
    pub fn collect_node_hashes<'a>(
        &self,
        node: &'a ExpressionNode,
    ) -> Vec<(ExpressionHash, &'a ExpressionNode)> {
        let mut hashes = Vec::new();
        self.visit_nodes_recursive(node, &mut hashes);
        hashes
    }

    /// Recursively visit nodes to collect hashes and references
    fn visit_nodes_recursive<'a>(
        &self,
        node: &'a ExpressionNode,
        hashes: &mut Vec<(ExpressionHash, &'a ExpressionNode)>,
    ) {
        let hash = Self::hash_node(node);
        hashes.push((hash, node));

        // Visit child nodes based on AST structure
        match node {
            ExpressionNode::Path { base, .. } => {
                self.visit_nodes_recursive(base, hashes);
            }
            ExpressionNode::BinaryOp(data) => {
                self.visit_nodes_recursive(&data.left, hashes);
                self.visit_nodes_recursive(&data.right, hashes);
            }
            ExpressionNode::UnaryOp { operand, .. } => {
                self.visit_nodes_recursive(operand, hashes);
            }
            ExpressionNode::FunctionCall(data) => {
                for arg in &data.args {
                    self.visit_nodes_recursive(arg, hashes);
                }
            }
            ExpressionNode::MethodCall(data) => {
                self.visit_nodes_recursive(&data.base, hashes);
                for arg in &data.args {
                    self.visit_nodes_recursive(arg, hashes);
                }
            }
            ExpressionNode::Index { base, index } => {
                self.visit_nodes_recursive(base, hashes);
                self.visit_nodes_recursive(index, hashes);
            }
            ExpressionNode::Filter { base, condition } => {
                self.visit_nodes_recursive(base, hashes);
                self.visit_nodes_recursive(condition, hashes);
            }
            ExpressionNode::Union { left, right } => {
                self.visit_nodes_recursive(left, hashes);
                self.visit_nodes_recursive(right, hashes);
            }
            ExpressionNode::TypeCheck { expression, .. } => {
                self.visit_nodes_recursive(expression, hashes);
            }
            ExpressionNode::TypeCast { expression, .. } => {
                self.visit_nodes_recursive(expression, hashes);
            }
            ExpressionNode::Lambda(data) => {
                self.visit_nodes_recursive(&data.body, hashes);
            }
            ExpressionNode::Conditional(data) => {
                self.visit_nodes_recursive(&data.condition, hashes);
                self.visit_nodes_recursive(&data.then_expr, hashes);
                if let Some(else_expr) = &data.else_expr {
                    self.visit_nodes_recursive(else_expr, hashes);
                }
            }
            // Leaf nodes (Literal, Identifier, Variable) have no children
            ExpressionNode::Literal(_)
            | ExpressionNode::Identifier(_)
            | ExpressionNode::Variable(_) => {}
        }
    }

    /// Get all attached analyses for nodes
    pub fn get_all_analyses(&self) -> &HashMap<ExpressionHash, SemanticInfo> {
        &self.node_analysis
    }

    /// Get all function call analyses
    pub fn get_all_function_analyses(&self) -> &HashMap<ExpressionHash, FunctionCallAnalysis> {
        &self.function_analysis
    }

    /// Get all union type analyses
    pub fn get_all_union_analyses(&self) -> &HashMap<ExpressionHash, UnionTypeInfo> {
        &self.union_analysis
    }

    /// Clear all analysis mappings
    pub fn clear(&mut self) {
        self.node_analysis.clear();
        self.function_analysis.clear();
        self.union_analysis.clear();
        self.next_node_id = 1;
    }
}

impl Default for ExpressionAnalysisMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Cardinality, ConfidenceLevel};
    use octofhir_fhirpath_ast::{ExpressionNode, LiteralValue};

    #[test]
    fn test_stable_hashing() {
        let node1 = ExpressionNode::Literal(LiteralValue::String("test".into()));
        let node2 = ExpressionNode::Literal(LiteralValue::String("test".into()));

        assert_eq!(
            ExpressionAnalysisMap::hash_node(&node1),
            ExpressionAnalysisMap::hash_node(&node2)
        );
    }

    #[test]
    fn test_different_nodes_different_hashes() {
        let node1 = ExpressionNode::Literal(LiteralValue::String("test".into()));
        let node2 = ExpressionNode::Literal(LiteralValue::String("different".into()));

        assert_ne!(
            ExpressionAnalysisMap::hash_node(&node1),
            ExpressionAnalysisMap::hash_node(&node2)
        );
    }

    #[test]
    fn test_external_mapping() {
        let mut map = ExpressionAnalysisMap::new();
        let node = ExpressionNode::Identifier("Patient".to_string());
        let info = SemanticInfo {
            fhir_path_type: Some("String".to_string()),
            model_type: Some("Patient".to_string()),
            cardinality: Cardinality::ZeroToOne,
            confidence: ConfidenceLevel::High,
            scope_info: None,
            function_info: None,
        };

        let node_id = map.attach_analysis(&node, info.clone());
        assert!(node_id > 0);

        let retrieved = map.get_analysis(&node);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().fhir_path_type, info.fhir_path_type);
        assert_eq!(retrieved.unwrap().model_type, info.model_type);
        assert_eq!(retrieved.unwrap().cardinality, info.cardinality);
        assert_eq!(retrieved.unwrap().confidence, info.confidence);
    }

    #[test]
    fn test_function_analysis_mapping() {
        use crate::types::{FunctionCallAnalysis, FunctionSignature};
        use octofhir_fhirpath_model::types::TypeInfo;

        let mut map = ExpressionAnalysisMap::new();
        let node = ExpressionNode::function_call("count", vec![]);

        let analysis = FunctionCallAnalysis {
            node_id: 1,
            function_name: "count".to_string(),
            signature: FunctionSignature {
                name: "count".to_string(),
                parameters: vec![],
                return_type: TypeInfo::Integer,
                is_aggregate: true,
                description: "Count function".to_string(),
            },
            parameter_types: vec![],
            return_type: TypeInfo::Integer,
            validation_errors: vec![],
        };

        map.attach_function_analysis(&node, analysis.clone());

        let retrieved = map.get_function_analysis(&node);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().function_name, analysis.function_name);
        assert_eq!(retrieved.unwrap().return_type, analysis.return_type);
    }

    #[test]
    fn test_union_analysis_mapping() {
        use crate::types::UnionTypeInfo;
        use octofhir_fhirpath_model::types::TypeInfo;

        let mut map = ExpressionAnalysisMap::new();
        let node = ExpressionNode::function_call("children", vec![]);

        let union_info = UnionTypeInfo {
            constituent_types: vec![TypeInfo::String, TypeInfo::Integer],
            is_collection: true,
            model_context: [("Patient".to_string(), "name".to_string())]
                .iter()
                .cloned()
                .collect(),
        };

        map.attach_union_analysis(&node, union_info.clone());

        let retrieved = map.get_union_analysis(&node);
        assert!(retrieved.is_some());
        assert_eq!(
            retrieved.unwrap().constituent_types,
            union_info.constituent_types
        );
        assert_eq!(retrieved.unwrap().is_collection, union_info.is_collection);
    }

    #[test]
    fn test_node_traversal() {
        let map = ExpressionAnalysisMap::new();

        // Create a complex AST: Patient.name.where(use = 'official')
        let base = ExpressionNode::Identifier("Patient".to_string());
        let path_node = ExpressionNode::path(base, "name");
        let condition = ExpressionNode::binary_op(
            octofhir_fhirpath_ast::BinaryOperator::Equal,
            ExpressionNode::Identifier("use".to_string()),
            ExpressionNode::Literal(LiteralValue::String("official".to_string())),
        );
        let filter_node = ExpressionNode::filter(path_node, condition);

        let hashes = map.collect_node_hashes(&filter_node);

        // Should have at least 5 nodes: filter, path, Patient identifier, use identifier, and literal
        assert!(hashes.len() >= 5);

        // Verify all hashes are different
        let hash_values: Vec<u64> = hashes.iter().map(|(hash, _)| *hash).collect();
        let mut unique_hashes = hash_values.clone();
        unique_hashes.sort();
        unique_hashes.dedup();
        assert_eq!(hash_values.len(), unique_hashes.len());
    }

    #[test]
    fn test_cache_concurrent_access() {
        use std::sync::Arc;
        use std::thread;

        let cache = Arc::new(AnalysisCache::new());
        let mut handles = vec![];

        // Test concurrent writes
        for i in 0..10 {
            let cache_clone = Arc::clone(&cache);
            let handle = thread::spawn(move || {
                let key = format!("test_key_{i}");
                let result = AnalysisResult {
                    validation_errors: vec![],
                    type_annotations: HashMap::new(),
                    function_calls: vec![],
                    union_types: HashMap::new(),
                };
                cache_clone.cache_analysis(key.clone(), result);

                // Verify we can read back
                assert!(cache_clone.get_analysis(&key).is_some());
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify all entries are present
        for i in 0..10 {
            let key = format!("test_key_{i}");
            assert!(cache.get_analysis(&key).is_some());
        }
    }

    #[test]
    fn test_analysis_cache_functions() {
        let cache = AnalysisCache::new();

        // Test type info caching
        use octofhir_fhirpath_model::types::TypeInfo;
        cache.cache_type_info("Patient".to_string(), "name".to_string(), TypeInfo::String);
        assert_eq!(
            cache.get_type_info("Patient", "name"),
            Some(TypeInfo::String)
        );

        // Test function signature caching
        let signature = FunctionSignature {
            name: "length".to_string(),
            parameters: vec![],
            return_type: TypeInfo::Integer,
            is_aggregate: false,
            description: "Returns string length".to_string(),
        };
        cache.cache_function_signature("length".to_string(), signature.clone());
        let retrieved = cache.get_function_signature("length");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, signature.name);
    }

    #[test]
    fn test_clear_functionality() {
        let mut map = ExpressionAnalysisMap::new();
        let cache = AnalysisCache::new();

        // Add some data
        let node = ExpressionNode::Identifier("test".to_string());
        let info = SemanticInfo {
            fhir_path_type: Some("String".to_string()),
            model_type: None,
            cardinality: Cardinality::OneToOne,
            confidence: ConfidenceLevel::High,
            scope_info: None,
            function_info: None,
        };

        map.attach_analysis(&node, info);
        cache.cache_analysis(
            "test".to_string(),
            AnalysisResult {
                validation_errors: vec![],
                type_annotations: HashMap::new(),
                function_calls: vec![],
                union_types: HashMap::new(),
            },
        );

        // Verify data exists
        assert!(map.get_analysis(&node).is_some());
        assert!(cache.get_analysis("test").is_some());

        // Clear and verify empty
        map.clear();
        cache.clear();

        assert!(map.get_analysis(&node).is_none());
        assert!(cache.get_analysis("test").is_none());
        assert_eq!(map.next_node_id, 1); // Should reset to 1
    }
}
