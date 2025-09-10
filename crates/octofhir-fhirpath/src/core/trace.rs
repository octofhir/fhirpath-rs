//! Trace provider abstraction for FHIRPath expression evaluation
//!
//! This module provides a flexible trace system that can work in different contexts:
//! - CLI: Direct output via eprintln!()
//! - Server: Collect traces for API responses
//! - Production: Silent no-op for performance

use std::sync::{Arc, Mutex};

/// Trait for providing trace output in different contexts
pub trait TraceProvider: Send + Sync {
    /// Output a trace message for a specific item in a collection
    fn trace(&self, name: &str, index: usize, message: &str);
    
    /// Output a trace message without an index (for simple traces)
    fn trace_simple(&self, name: &str, message: &str);
    
    /// Collect all traces as a vector of strings (for server responses)
    fn collect_traces(&self) -> Vec<String>;
    
    /// Clear collected traces
    fn clear_traces(&self);
}

/// CLI trace provider that outputs directly to stderr via eprintln!
#[derive(Debug, Default)]
pub struct CliTraceProvider;

impl CliTraceProvider {
    pub fn new() -> Self {
        Self
    }
}

impl TraceProvider for CliTraceProvider {
    fn trace(&self, name: &str, index: usize, message: &str) {
        eprintln!("TRACE[{}][{}]: {}", name, index, message);
    }
    
    fn trace_simple(&self, name: &str, message: &str) {
        eprintln!("TRACE[{}]: {}", name, message);
    }
    
    fn collect_traces(&self) -> Vec<String> {
        // CLI provider doesn't collect traces, they're output immediately
        Vec::new()
    }
    
    fn clear_traces(&self) {
        // No-op for CLI provider
    }
}

/// Server trace provider that collects traces in memory for API responses
#[derive(Debug)]
pub struct ServerTraceProvider {
    traces: Arc<Mutex<Vec<String>>>,
}

impl ServerTraceProvider {
    pub fn new() -> Self {
        Self {
            traces: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl Default for ServerTraceProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl TraceProvider for ServerTraceProvider {
    fn trace(&self, name: &str, index: usize, message: &str) {
        let trace_line = format!("TRACE[{}][{}]: {}", name, index, message);
        if let Ok(mut traces) = self.traces.lock() {
            traces.push(trace_line);
        }
    }
    
    fn trace_simple(&self, name: &str, message: &str) {
        let trace_line = format!("TRACE[{}]: {}", name, message);
        if let Ok(mut traces) = self.traces.lock() {
            traces.push(trace_line);
        }
    }
    
    fn collect_traces(&self) -> Vec<String> {
        if let Ok(traces) = self.traces.lock() {
            traces.clone()
        } else {
            Vec::new()
        }
    }
    
    fn clear_traces(&self) {
        if let Ok(mut traces) = self.traces.lock() {
            traces.clear();
        }
    }
}

/// No-op trace provider for production environments where trace output is disabled
#[derive(Debug, Default)]
pub struct NoOpTraceProvider;

impl NoOpTraceProvider {
    pub fn new() -> Self {
        Self
    }
}

impl TraceProvider for NoOpTraceProvider {
    fn trace(&self, _name: &str, _index: usize, _message: &str) {
        // No-op
    }
    
    fn trace_simple(&self, _name: &str, _message: &str) {
        // No-op
    }
    
    fn collect_traces(&self) -> Vec<String> {
        Vec::new()
    }
    
    fn clear_traces(&self) {
        // No-op
    }
}

/// Convenience type for Arc<dyn TraceProvider>
pub type SharedTraceProvider = Arc<dyn TraceProvider>;

/// Create a CLI trace provider wrapped in Arc
pub fn create_cli_provider() -> SharedTraceProvider {
    Arc::new(CliTraceProvider::new())
}

/// Create a server trace provider wrapped in Arc
pub fn create_server_provider() -> SharedTraceProvider {
    Arc::new(ServerTraceProvider::new())
}

/// Create a no-op trace provider wrapped in Arc
pub fn create_noop_provider() -> SharedTraceProvider {
    Arc::new(NoOpTraceProvider::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_trace_provider() {
        let provider = CliTraceProvider::new();
        
        // These will output to stderr but won't be captured in tests
        provider.trace("test", 0, "test message");
        provider.trace_simple("test", "simple message");
        
        // CLI provider doesn't collect traces
        assert!(provider.collect_traces().is_empty());
    }

    #[test]
    fn test_server_trace_provider() {
        let provider = ServerTraceProvider::new();
        
        provider.trace("test", 0, "indexed message");
        provider.trace_simple("test", "simple message");
        
        let traces = provider.collect_traces();
        assert_eq!(traces.len(), 2);
        assert_eq!(traces[0], "TRACE[test][0]: indexed message");
        assert_eq!(traces[1], "TRACE[test]: simple message");
        
        provider.clear_traces();
        assert!(provider.collect_traces().is_empty());
    }

    #[test]
    fn test_noop_trace_provider() {
        let provider = NoOpTraceProvider::new();
        
        provider.trace("test", 0, "test message");
        provider.trace_simple("test", "simple message");
        
        // No-op provider doesn't collect traces
        assert!(provider.collect_traces().is_empty());
    }

    #[test]
    fn test_shared_providers() {
        let cli_provider = create_cli_provider();
        let server_provider = create_server_provider();
        let noop_provider = create_noop_provider();
        
        // Test that they implement the trait correctly
        cli_provider.trace_simple("test", "cli test");
        server_provider.trace_simple("test", "server test");
        noop_provider.trace_simple("test", "noop test");
        
        // Only server provider should collect traces
        assert!(cli_provider.collect_traces().is_empty());
        assert_eq!(server_provider.collect_traces().len(), 1);
        assert!(noop_provider.collect_traces().is_empty());
    }
}