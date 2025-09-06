//! FHIRPath static analysis and type checking

pub mod analyzer;
pub mod context;
pub mod optimization_detector_simple;
pub mod property_validator;
pub mod type_checker;
pub mod validation;
pub mod visitor;
pub use optimization_detector_simple as optimization_detector;
pub mod optimization_report;

pub use analyzer::*;
pub use context::{AnalysisContext, ScopeInfo, ScopeType};
pub use optimization_detector_simple::{
    ComplexityIssue, ComplexityIssueType, DepthAnalysis, FunctionCallStats, IssueSeverity,
    OptimizationAnalysisResult, OptimizationDetector, PatternMatch, PatternType,
};
pub use optimization_report::{OptimizationReporter, ReportConfig};
pub use property_validator::{
    Cardinality, PropertyInfo, PropertySuggestion, PropertyValidationResult, PropertyValidator,
};
pub use type_checker::{NodeId, TypeAnalysisResult, TypeChecker, TypeInfo, TypeWarning};
pub use visitor::{CollectingVisitor, DefaultExpressionVisitor, ExpressionVisitor};
