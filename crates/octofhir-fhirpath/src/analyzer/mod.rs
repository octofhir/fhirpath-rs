//! FHIRPath static analysis and type checking

pub mod analyzer;
pub mod type_checker;
pub mod validation;
pub mod visitor;
pub mod context;
pub mod property_validator;
pub mod optimization_detector_simple;
pub use optimization_detector_simple as optimization_detector;
pub mod optimization_report;

pub use analyzer::*;
pub use type_checker::{TypeInfo, TypeChecker, TypeAnalysisResult, TypeWarning, NodeId};
pub use context::{AnalysisContext, ScopeInfo, ScopeType};
pub use visitor::{ExpressionVisitor, DefaultExpressionVisitor, CollectingVisitor};
pub use property_validator::{PropertyValidator, PropertyValidationResult, PropertySuggestion, PropertyInfo, Cardinality};
pub use optimization_detector_simple::{
    OptimizationDetector, OptimizationAnalysisResult, ComplexityIssue, PatternMatch, 
    ComplexityIssueType, IssueSeverity, PatternType, FunctionCallStats, DepthAnalysis
};
pub use optimization_report::{OptimizationReporter, ReportConfig};