//! # Core Analysis Engine
//!
//! The main orchestration engine for FHIRPath static analysis. This engine coordinates
//! multiple analysis phases and produces comprehensive analysis results with rich diagnostics.

use std::sync::Arc;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use octofhir_fhirpath_ast::ExpressionNode;
use octofhir_fhirpath_parser::parse_expression;
use octofhir_fhirpath_diagnostics::{Diagnostic, Severity, SourceLocation};

use super::{
    error::AnalysisError,
    context::{AnalysisContext, AnalysisPhase},
    symbol_table::SymbolTable,
    type_system::{TypeSystem, TypeInformation},
};
use crate::providers::{
    fhir_provider::FhirProvider,
    function_provider::FunctionProvider,
};

/// Node identifier for AST nodes
pub type NodeId = usize;

/// Configuration for the analyzer
#[derive(Debug, Clone)]
pub struct AnalyzerConfig {
    /// Enable type checking phase
    pub enable_type_checking: bool,
    
    /// Enable property validation
    pub enable_property_validation: bool,
    
    /// Enable function validation
    pub enable_function_validation: bool,
    
    /// Enable performance analysis
    pub enable_performance_analysis: bool,
    
    /// Enable code completion suggestions
    pub enable_completions: bool,
    
    /// Cache size for analysis results
    pub cache_size: usize,
    
    /// Maximum analysis time
    pub max_analysis_time: Duration,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            enable_type_checking: true,
            enable_property_validation: true,
            enable_function_validation: true,
            enable_performance_analysis: true,
            enable_completions: false,
            cache_size: 1000,
            max_analysis_time: Duration::from_millis(5000),
        }
    }
}

/// Comprehensive analysis result
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    /// All diagnostics from analysis
    pub diagnostics: Vec<Diagnostic>,
    
    /// Type information for each AST node
    pub type_information: HashMap<NodeId, TypeInformation>,
    
    /// Symbol resolution information
    pub symbol_resolution: SymbolResolution,
    
    /// Performance optimization hints
    pub optimization_hints: Vec<OptimizationHint>,
    
    /// Code completion suggestions
    pub completions: Vec<CompletionItem>,
    
    /// Hover information for nodes
    pub hover_information: HashMap<NodeId, HoverInfo>,
    
    /// Analysis metadata
    pub metadata: AnalysisMetadata,
}

/// Symbol resolution information
#[derive(Debug, Clone)]
pub struct SymbolResolution {
    /// Resolved properties by node
    pub resolved_properties: HashMap<NodeId, PropertyResolution>,
    
    /// Resolved functions by node
    pub resolved_functions: HashMap<NodeId, FunctionResolution>,
    
    /// Resolved variables by node
    pub resolved_variables: HashMap<NodeId, VariableResolution>,
}

/// Property resolution result
#[derive(Debug, Clone)]
pub struct PropertyResolution {
    /// Property name
    pub property_name: String,
    
    /// Source type
    pub source_type: String,
    
    /// Target type
    pub target_type: String,
    
    /// Cardinality
    pub cardinality: (usize, Option<usize>),
    
    /// Whether property exists
    pub exists: bool,
}

/// Function resolution result
#[derive(Debug, Clone)]
pub struct FunctionResolution {
    /// Function name
    pub function_name: String,
    
    /// Parameter count
    pub parameter_count: usize,
    
    /// Return type
    pub return_type: String,
    
    /// Whether function exists
    pub exists: bool,
}

/// Variable resolution result
#[derive(Debug, Clone)]
pub struct VariableResolution {
    /// Variable name
    pub variable_name: String,
    
    /// Variable type
    pub variable_type: String,
    
    /// Scope where defined
    pub scope_id: usize,
    
    /// Whether variable exists
    pub exists: bool,
}

/// Performance optimization hint
#[derive(Debug, Clone)]
pub struct OptimizationHint {
    /// Type of optimization
    pub hint_type: OptimizationHintType,
    
    /// Description
    pub message: String,
    
    /// Location in source
    pub location: SourceLocation,
    
    /// Suggested improvement
    pub suggestion: Option<String>,
}

/// Types of optimization hints
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationHintType {
    /// Performance improvement
    Performance,
    
    /// Readability improvement  
    Readability,
    
    /// Best practice
    BestPractice,
    
    /// Memory usage
    Memory,
}

/// Code completion item
#[derive(Debug, Clone)]
pub struct CompletionItem {
    /// Label to display
    pub label: String,
    
    /// Kind of completion
    pub kind: CompletionKind,
    
    /// Detail text
    pub detail: Option<String>,
    
    /// Documentation
    pub documentation: Option<String>,
    
    /// Text to insert
    pub insert_text: String,
}

/// Completion item kinds
#[derive(Debug, Clone, PartialEq)]
pub enum CompletionKind {
    Property,
    Function,
    Variable,
    Constant,
    Keyword,
}

/// Hover information
#[derive(Debug, Clone)]
pub struct HoverInfo {
    /// Content to display
    pub content: String,
    
    /// Type information
    pub type_info: Option<String>,
    
    /// Documentation
    pub documentation: Option<String>,
}

/// Analysis execution metadata
#[derive(Debug, Clone)]
pub struct AnalysisMetadata {
    /// Total analysis duration
    pub duration: Duration,
    
    /// Individual phase durations
    pub phase_durations: HashMap<String, Duration>,
    
    /// Expression being analyzed
    pub expression: String,
    
    /// Whether analysis completed successfully
    pub completed: bool,
    
    /// Memory usage statistics
    pub memory_stats: MemoryStats,
}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    /// Peak memory during analysis
    pub peak_bytes: usize,
    
    /// Final memory usage
    pub final_bytes: usize,
}

/// Main FHIRPath analyzer engine
pub struct FhirPathAnalyzer<P> 
where 
    P: FhirProvider + Send + Sync + 'static 
{
    /// FHIR model provider
    provider: Arc<P>,
    
    /// Function provider (optional)
    function_provider: Option<Arc<dyn FunctionProvider>>,
    
    /// Type system
    type_system: TypeSystem,
    
    // Parser is now a function, not a struct
    
    /// Configuration
    config: AnalyzerConfig,
    
    /// Analysis cache
    cache: lru::LruCache<String, AnalysisResult>,
}

impl<P> FhirPathAnalyzer<P> 
where 
    P: FhirProvider + Send + Sync + 'static 
{
    /// Create a new analyzer with the given provider
    pub fn new(provider: Arc<P>) -> Self {
        let type_system = TypeSystem::new(Arc::clone(&provider) as Arc<dyn FhirProvider>);
        
        Self {
            provider: Arc::clone(&provider),
            function_provider: None,
            type_system,
            config: AnalyzerConfig::default(),
            cache: lru::LruCache::new(std::num::NonZeroUsize::new(1000).unwrap()),
        }
    }
    
    /// Create analyzer with custom configuration
    pub fn with_config(provider: Arc<P>, config: AnalyzerConfig) -> Self {
        let type_system = TypeSystem::new(Arc::clone(&provider) as Arc<dyn FhirProvider>);
        let cache_size = std::num::NonZeroUsize::new(config.cache_size).unwrap_or_else(|| std::num::NonZeroUsize::new(1000).unwrap());
        
        Self {
            provider: Arc::clone(&provider),
            function_provider: None,
            type_system,
            config,
            cache: lru::LruCache::new(cache_size),
        }
    }
    
    /// Create analyzer with function provider
    pub fn with_function_provider(provider: Arc<P>, function_provider: Arc<dyn FunctionProvider>) -> Self {
        let type_system = TypeSystem::new(Arc::clone(&provider) as Arc<dyn FhirProvider>);
        
        Self {
            provider: Arc::clone(&provider),
            function_provider: Some(function_provider),
            type_system,
            config: AnalyzerConfig::default(),
            cache: lru::LruCache::new(std::num::NonZeroUsize::new(1000).unwrap()),
        }
    }
    
    /// Analyze a FHIRPath expression
    pub async fn analyze(&mut self, expression: &str) -> Result<AnalysisResult, AnalysisError> {
        let start_time = Instant::now();
        
        // Check cache first
        if let Some(cached) = self.cache.get(expression) {
            return Ok(cached.clone());
        }
        
        // Parse expression to AST
        let ast = parse_expression(expression)
            .map_err(|e| AnalysisError::ParseError { 
                message: format!("Failed to parse expression: {}", e) 
            })?;
        
        // Run analysis on AST
        let result = self.analyze_ast(&ast, Some(expression.to_string())).await?;
        
        // Cache result
        self.cache.put(expression.to_string(), result.clone());
        
        Ok(result)
    }
    
    /// Analyze a pre-parsed AST
    pub async fn analyze_ast(&self, ast: &ExpressionNode, expression: Option<String>) -> Result<AnalysisResult, AnalysisError> {
        let start_time = Instant::now();
        let mut phase_durations = HashMap::new();
        
        // Initialize analysis context
        let mut context = AnalysisContext::new();
        let mut symbol_table = SymbolTable::new();
        let mut diagnostics = Vec::new();
        let mut type_information = HashMap::new();
        
        // Phase 1: Lexical Analysis
        if self.should_run_phase("lexical") {
            let phase_start = Instant::now();
            context.set_phase(AnalysisPhase::Lexical);
            
            // Basic lexical validation would go here
            // For now, just mark as completed
            
            phase_durations.insert("lexical".to_string(), phase_start.elapsed());
        }
        
        // Phase 2: Property Resolution
        if self.should_run_phase("property_resolution") {
            let phase_start = Instant::now();
            context.set_phase(AnalysisPhase::PropertyResolution);
            
            // Property resolution logic would go here
            // This would validate that properties exist on types
            
            phase_durations.insert("property_resolution".to_string(), phase_start.elapsed());
        }
        
        // Phase 3: Function Validation
        if self.should_run_phase("function_validation") {
            let phase_start = Instant::now();
            context.set_phase(AnalysisPhase::FunctionValidation);
            
            // Function validation logic would go here
            // This would check function signatures and parameters
            
            phase_durations.insert("function_validation".to_string(), phase_start.elapsed());
        }
        
        // Phase 4: Type Checking
        if self.should_run_phase("type_checking") {
            let phase_start = Instant::now();
            context.set_phase(AnalysisPhase::TypeChecking);
            
            // Type checking logic would go here
            // This would infer types and check compatibility
            
            phase_durations.insert("type_checking".to_string(), phase_start.elapsed());
        }
        
        // Phase 5: Lambda Validation
        if self.should_run_phase("lambda_validation") {
            let phase_start = Instant::now();
            context.set_phase(AnalysisPhase::LambdaValidation);
            
            // Lambda validation logic would go here
            // This would validate lambda expression scoping
            
            phase_durations.insert("lambda_validation".to_string(), phase_start.elapsed());
        }
        
        // Phase 6: Optimization Analysis
        if self.should_run_phase("optimization") {
            let phase_start = Instant::now();
            context.set_phase(AnalysisPhase::OptimizationAnalysis);
            
            // Optimization analysis would go here
            // This would generate performance hints
            
            phase_durations.insert("optimization".to_string(), phase_start.elapsed());
        }
        
        // Generate result
        let total_duration = start_time.elapsed();
        
        Ok(AnalysisResult {
            diagnostics,
            type_information,
            symbol_resolution: SymbolResolution {
                resolved_properties: HashMap::new(),
                resolved_functions: HashMap::new(),
                resolved_variables: HashMap::new(),
            },
            optimization_hints: Vec::new(),
            completions: Vec::new(),
            hover_information: HashMap::new(),
            metadata: AnalysisMetadata {
                duration: total_duration,
                phase_durations,
                expression: expression.unwrap_or_else(|| "<parsed AST>".to_string()),
                completed: true,
                memory_stats: MemoryStats {
                    peak_bytes: 0, // TODO: Implement memory tracking
                    final_bytes: 0,
                },
            },
        })
    }
    
    /// Analyze with a specific root resource type context
    pub async fn analyze_with_context(
        &mut self, 
        expression: &str, 
        root_resource_type: String
    ) -> Result<AnalysisResult, AnalysisError> {
        // Parse and analyze with context
        let ast = parse_expression(expression)
            .map_err(|e| AnalysisError::ParseError { 
                message: format!("Failed to parse expression: {}", e) 
            })?;
        
        // Create context with root type
        let mut context = AnalysisContext::with_root_type(root_resource_type);
        
        // Run analysis (simplified for now)
        self.analyze_ast(&ast, Some(expression.to_string())).await
    }
    
    /// Check if a phase should run based on configuration
    fn should_run_phase(&self, phase: &str) -> bool {
        match phase {
            "lexical" => true, // Always run lexical analysis
            "property_resolution" => self.config.enable_property_validation,
            "function_validation" => self.config.enable_function_validation,
            "type_checking" => self.config.enable_type_checking,
            "lambda_validation" => true, // Always validate lambdas
            "optimization" => self.config.enable_performance_analysis,
            _ => false,
        }
    }
    
    /// Get analyzer configuration
    pub fn config(&self) -> &AnalyzerConfig {
        &self.config
    }
    
    /// Update analyzer configuration
    pub fn set_config(&mut self, config: AnalyzerConfig) {
        self.config = config;
    }
    
    /// Clear analysis cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
    
    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        (self.cache.len(), self.cache.cap().get())
    }
}

impl AnalysisResult {
    /// Check if analysis found any errors
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    }
    
    /// Check if analysis found any warnings
    pub fn has_warnings(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Warning)
    }
    
    /// Get all diagnostics
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
    
    /// Get diagnostics by severity level
    pub fn diagnostics_by_severity(&self, severity: Severity) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == severity)
            .collect()
    }
    
    /// Get type information for a node
    pub fn get_type_info(&self, node_id: NodeId) -> Option<&TypeInformation> {
        self.type_information.get(&node_id)
    }
    
    /// Get optimization hints
    pub fn optimization_hints(&self) -> &[OptimizationHint] {
        &self.optimization_hints
    }
    
    /// Get completion items
    pub fn completions(&self) -> &[CompletionItem] {
        &self.completions
    }
    
    /// Get hover information for a node
    pub fn get_hover_info(&self, node_id: NodeId) -> Option<&HoverInfo> {
        self.hover_information.get(&node_id)
    }
    
    /// Get analysis metadata
    pub fn metadata(&self) -> &AnalysisMetadata {
        &self.metadata
    }
    
    /// Check if analysis completed successfully
    pub fn is_successful(&self) -> bool {
        self.metadata.completed && !self.has_errors()
    }
}