//! # Function Provider

use async_trait::async_trait;

#[async_trait]
pub trait FunctionProvider: Send + Sync {
    async fn get_function_signature(&self, name: &str) -> Result<Option<FunctionSignature>, FunctionProviderError>;
}

#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: String,
    pub parameter_count: usize,
    pub return_type: String,
}

#[derive(Debug, thiserror::Error)]
pub enum FunctionProviderError {
    #[error("Function provider error: {message}")]
    ProviderError { message: String },
}