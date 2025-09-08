//! TUI Utilities and Advanced Features
//! 
//! This module provides advanced functionality for the TUI including:
//! - Real-time syntax highlighting
//! - Context-aware auto-completion
//! - Performance optimizations
//! - Text manipulation utilities

pub mod completion_engine;
pub mod performance;
pub mod syntax_highlighter;
pub mod text_editor;

pub use completion_engine::{CompletionContext, CompletionEngine};
pub use performance::{PerformanceTracker, Timer};
pub use syntax_highlighter::{HighlightedSpan, SyntaxHighlighter};
pub use text_editor::TextEditor;

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[test]
    fn test_text_editor() {
        let mut editor = TextEditor::new();
        
        editor.insert_text("Patient.name");
        assert_eq!(editor.text(), "Patient.name");
        assert_eq!(editor.cursor_position(), 12);
        
        editor.cursor_left();
        editor.cursor_left();
        editor.cursor_left();
        editor.cursor_left();
        editor.insert_text("given.");
        assert_eq!(editor.text(), "Patient.given.name");
        
        editor.undo();
        assert_eq!(editor.text(), "Patient.name");
        
        editor.redo();
        assert_eq!(editor.text(), "Patient.given.name");
    }
    
    #[test]
    fn test_performance_tracker() {
        let mut tracker = PerformanceTracker::new(10);
        
        tracker.record("test_op", Duration::from_millis(10));
        tracker.record("test_op", Duration::from_millis(20));
        
        let summary = tracker.summary();
        assert!(!summary.is_empty());
    }
}