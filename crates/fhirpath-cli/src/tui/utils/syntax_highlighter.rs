//! Real-time syntax highlighting for FHIRPath expressions

use std::ops::Range;
use std::sync::Arc;

use ratatui::style::Style;
use ratatui::text::{Line, Span};

use octofhir_fhirpath::diagnostics::AriadneDiagnostic;

use crate::tui::themes::TuiTheme;

/// Real-time syntax highlighter for FHIRPath expressions
pub struct SyntaxHighlighter {
    theme: Arc<TuiTheme>,
    cache: HighlightCache,
}

/// Cache for syntax highlighting results
#[derive(Default)]
struct HighlightCache {
    last_expression: String,
    last_result: Vec<HighlightedSpan>,
}

/// A highlighted span of text
#[derive(Debug, Clone)]
pub struct HighlightedSpan {
    pub range: Range<usize>,
    pub style: Style,
    pub token_type: String,
}

impl SyntaxHighlighter {
    /// Create a new syntax highlighter
    pub fn new(theme: Arc<TuiTheme>) -> Self {
        Self {
            theme,
            cache: HighlightCache::default(),
        }
    }
    
    /// Highlight a FHIRPath expression
    pub fn highlight(&mut self, expression: &str) -> Vec<HighlightedSpan> {
        // Check cache first
        if expression == self.cache.last_expression {
            return self.cache.last_result.clone();
        }
        
        // Simple pattern-based highlighting for now
        let mut spans = Vec::new();
        
        // Basic FHIRPath keyword highlighting (will use registry later)
        let _keywords = ["and", "or", "xor", "implies", "is", "as", "div", "mod", "in", "contains"];
        let _functions = ["first", "last", "count", "exists", "empty", "where", "select"];
        
        // For now, just highlight the whole expression as normal text
        // This is a simplified version until we have proper tokenization
        spans.push(HighlightedSpan {
            range: 0..expression.len(),
            style: self.theme.get_syntax_style("identifier"),
            token_type: "identifier".to_string(),
        });
        
        // Update cache
        self.cache.last_expression = expression.to_string();
        self.cache.last_result = spans.clone();
        
        spans
    }
    
    /// Highlight expression with diagnostic overlays
    pub fn highlight_with_diagnostics(
        &mut self,
        expression: &str,
        diagnostics: &[AriadneDiagnostic],
    ) -> Vec<HighlightedSpan> {
        let mut spans = self.highlight(expression);
        
        // Overlay diagnostic highlighting
        for diagnostic in diagnostics {
            let range = &diagnostic.span;
            let severity_str = format!("{:?}", diagnostic.severity).to_lowercase();
            let error_style = self.theme.get_diagnostic_style(&severity_str);
            
            // Find spans that overlap with the diagnostic range
            for span in &mut spans {
                if spans_overlap(&span.range, range) {
                    span.style = error_style;
                    span.token_type = format!("{:?}_{}", diagnostic.severity, span.token_type);
                }
            }
        }
        
        spans
    }
    
    /// Convert highlighted spans to ratatui Line
    pub fn spans_to_line<'a>(&self, text: &'a str, spans: &[HighlightedSpan]) -> Line<'a> {
        if spans.is_empty() {
            return Line::from(text);
        }
        
        let mut ratatui_spans = Vec::new();
        let mut last_end = 0;
        
        for span in spans {
            // Add any unhighlighted text before this span
            if span.range.start > last_end {
                let unhighlighted = &text[last_end..span.range.start];
                if !unhighlighted.is_empty() {
                    ratatui_spans.push(Span::raw(unhighlighted));
                }
            }
            
            // Add the highlighted span
            let highlighted_text = &text[span.range.clone()];
            ratatui_spans.push(Span::styled(highlighted_text, span.style));
            
            last_end = span.range.end;
        }
        
        // Add any remaining unhighlighted text
        if last_end < text.len() {
            let remaining = &text[last_end..];
            if !remaining.is_empty() {
                ratatui_spans.push(Span::raw(remaining));
            }
        }
        
        Line::from(ratatui_spans)
    }
    
    /// Update theme
    pub fn set_theme(&mut self, theme: Arc<TuiTheme>) {
        self.theme = theme;
        // Clear cache to force re-highlighting with new theme
        self.cache.last_expression.clear();
    }
}

/// Check if two ranges overlap
fn spans_overlap(range1: &Range<usize>, range2: &Range<usize>) -> bool {
    range1.start < range2.end && range2.start < range1.end
}