//! Server-specific trace provider implementations for FHIRPath server API

use octofhir_fhirpath::core::{SharedTraceProvider, TraceProvider};
use serde_json::Value as JsonValue;
use std::sync::{Arc, Mutex};
use tracing::debug;

const MAX_TRACE_ENTRIES: usize = 2048;
const MAX_TRACE_MESSAGE_LENGTH: usize = 2048;
const TRUNCATION_NOTICE: &str = "TRACE[system]: trace output truncated";

#[derive(Debug, Default, Clone)]
struct TraceState {
    lines: Vec<String>,
    truncated: bool,
}

/// Trace entry with structured
#[derive(Debug, Clone)]
pub struct TraceEntry {
    pub name: String,
    pub values: Vec<JsonValue>,
}

/// Server trace provider that collects traces for API responses
/// This is optimized for server use where we need to collect all traces
/// and return them as part of the FHIRPath Lab API response
#[derive(Debug)]
pub struct ServerApiTraceProvider {
    traces: Arc<Mutex<TraceState>>,
    structured_traces: Arc<Mutex<Vec<TraceEntry>>>,
}

impl ServerApiTraceProvider {
    /// Create a new server API trace provider
    pub fn new() -> Self {
        Self {
            traces: Arc::new(Mutex::new(TraceState::default())),
            structured_traces: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a shared server API trace provider
    pub fn create_shared() -> SharedTraceProvider {
        Arc::new(Self::new())
    }

    /// Add a structured trace entry
    pub fn add_structured_trace(&self, name: &str, values: Vec<JsonValue>) {
        if let Ok(mut traces) = self.structured_traces.lock() {
            traces.push(TraceEntry {
                name: name.to_string(),
                values,
            });
        }
    }

    /// Get all structured traces
    pub fn collect_structured_traces(&self) -> Vec<TraceEntry> {
        if let Ok(traces) = self.structured_traces.lock() {
            traces.clone()
        } else {
            Vec::new()
        }
    }

    /// Clear structured traces
    pub fn clear_structured_traces(&self) {
        if let Ok(mut traces) = self.structured_traces.lock() {
            traces.clear();
        }
    }

    fn push_trace_line(&self, mut line: String) {
        if line.len() > MAX_TRACE_MESSAGE_LENGTH {
            line.truncate(MAX_TRACE_MESSAGE_LENGTH);
            line.push_str("...[truncated]");
            debug!(
                message_len = line.len(),
                "trace message truncated to max length"
            );
        }

        if let Ok(mut state) = self.traces.lock() {
            if state.lines.len() >= MAX_TRACE_ENTRIES {
                state.truncated = true;
                debug!(
                    trace_entries = state.lines.len(),
                    "trace buffer reached capacity; further messages discarded"
                );
                return;
            }
            state.lines.push(line);
            debug!(trace_entries = state.lines.len(), "trace line recorded");
        }
    }
}

impl Default for ServerApiTraceProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl TraceProvider for ServerApiTraceProvider {
    fn trace(&self, name: &str, index: usize, message: &str) {
        let trace_line = format!("TRACE[{name}][{index}]: {message}");
        self.push_trace_line(trace_line);
    }

    fn trace_simple(&self, name: &str, message: &str) {
        let trace_line = format!("TRACE[{name}]: {message}");
        self.push_trace_line(trace_line);
    }

    fn collect_traces(&self) -> Vec<String> {
        if let Ok(state) = self.traces.lock() {
            let mut lines = state.lines.clone();
            if state.truncated {
                lines.push(TRUNCATION_NOTICE.to_string());
            }
            lines
        } else {
            Vec::new()
        }
    }

    fn clear_traces(&self) {
        if let Ok(mut state) = self.traces.lock() {
            state.lines.clear();
            state.truncated = false;
        }
        self.clear_structured_traces();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_api_trace_provider() {
        let provider = ServerApiTraceProvider::new();

        provider.trace("test", 0, "indexed message");
        provider.trace("test", 1, "second indexed message");
        provider.trace_simple("simple", "simple message");

        let traces = provider.collect_traces();
        assert_eq!(traces.len(), 3);
        assert_eq!(traces[0], "TRACE[test][0]: indexed message");
        assert_eq!(traces[1], "TRACE[test][1]: second indexed message");
        assert_eq!(traces[2], "TRACE[simple]: simple message");

        provider.clear_traces();
        assert!(provider.collect_traces().is_empty());
    }

    #[test]
    fn test_shared_server_trace_provider() {
        let provider = ServerApiTraceProvider::create_shared();

        provider.trace_simple("shared", "shared test");

        let traces = provider.collect_traces();
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0], "TRACE[shared]: shared test");
    }

    #[test]
    fn test_trace_truncation() {
        let provider = ServerApiTraceProvider::new();

        for _ in 0..(MAX_TRACE_ENTRIES + 10) {
            provider.trace_simple("test", "x");
        }

        let traces = provider.collect_traces();
        assert_eq!(traces.len(), MAX_TRACE_ENTRIES + 1);
        assert_eq!(traces.last().unwrap(), TRUNCATION_NOTICE);
    }

    #[test]
    fn test_message_truncation() {
        let provider = ServerApiTraceProvider::new();
        let long_message = "a".repeat(MAX_TRACE_MESSAGE_LENGTH + 50);
        provider.trace_simple("long", &long_message);
        let traces = provider.collect_traces();
        assert_eq!(traces.len(), 1);
        assert!(traces[0].ends_with("...[truncated]"));
        assert!(traces[0].len() <= MAX_TRACE_MESSAGE_LENGTH + "...[truncated]".len());
    }
}
