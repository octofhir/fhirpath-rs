//! # Analysis Errors
//!
//! Error types for the analysis engine.

/// Analysis engine errors
#[derive(Debug, thiserror::Error)]
pub enum AnalysisError {
    #[error("Analysis failed: {message}")]
    AnalysisFailed { message: String },
    
    #[error("Parse error: {message}")]
    ParseError { message: String },
    
    #[error("Type system error: {message}")]
    TypeSystemError { message: String },
    
    #[error("Provider error: {source}")]
    ProviderError { 
        #[from]
        source: Box<dyn std::error::Error + Send + Sync> 
    },
}