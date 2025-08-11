//! Document management for the LSP server
//!
//! Handles tracking of open documents, incremental updates, and version management.

use lsp_types::{TextDocumentContentChangeEvent, Url};
use std::collections::HashMap;

/// Represents a document in the LSP server
#[derive(Debug, Clone)]
pub struct Document {
    /// Document URI
    pub uri: Url,
    /// Document text content
    pub text: String,
    /// Document version
    pub version: i32,
    /// Language identifier
    pub language_id: String,
}

impl Document {
    /// Create a new document
    pub fn new(uri: Url, text: String, version: i32, language_id: String) -> Self {
        Self {
            uri,
            text,
            version,
            language_id,
        }
    }

    /// Apply a content change to the document
    pub fn apply_change(&mut self, change: TextDocumentContentChangeEvent, new_version: i32) {
        self.version = new_version;

        if let Some(range) = change.range {
            // Incremental change
            let start_offset = self.position_to_offset(range.start);
            let end_offset = self.position_to_offset(range.end);
            
            if start_offset <= self.text.len() && end_offset <= self.text.len() && start_offset <= end_offset {
                self.text.replace_range(start_offset..end_offset, &change.text);
            } else {
                // Fallback to full text replacement on invalid range
                self.text = change.text;
            }
        } else {
            // Full document change
            self.text = change.text;
        }
    }

    /// Convert LSP position to byte offset
    fn position_to_offset(&self, position: lsp_types::Position) -> usize {
        let mut offset = 0;
        let mut line = 0u32;
        let mut character = 0u32;

        for ch in self.text.chars() {
            if line == position.line && character == position.character {
                break;
            }

            if ch == '\n' {
                line += 1;
                character = 0;
            } else {
                character += 1;
            }

            offset += ch.len_utf8();
        }

        // Ensure we don't go beyond the document length
        offset.min(self.text.len())
    }

    /// Get text at a specific range
    pub fn get_text_range(&self, range: lsp_types::Range) -> Option<String> {
        let start_offset = self.position_to_offset(range.start);
        let end_offset = self.position_to_offset(range.end);
        
        if start_offset <= self.text.len() && end_offset <= self.text.len() && start_offset <= end_offset {
            Some(self.text[start_offset..end_offset].to_string())
        } else {
            None
        }
    }

    /// Get line count
    pub fn line_count(&self) -> u32 {
        self.text.lines().count() as u32
    }

    /// Get text up to a position (useful for completion)
    pub fn get_text_up_to_position(&self, position: lsp_types::Position) -> String {
        let offset = self.position_to_offset(position);
        self.text[..offset].to_string()
    }
}

/// Manages documents for the LSP server
#[derive(Debug, Default)]
pub struct DocumentManager {
    /// Map of URI to document
    documents: HashMap<String, Document>,
}

impl DocumentManager {
    /// Create a new document manager
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
        }
    }

    /// Open a new document
    pub fn open_document(&mut self, uri: Url, text: String, version: i32) {
        let language_id = self.detect_language_id(&uri);
        let document = Document::new(uri.clone(), text, version, language_id);
        self.documents.insert(uri.to_string(), document);
    }

    /// Get a document by URI
    pub fn get_document(&self, uri: &Url) -> Option<&Document> {
        self.documents.get(&uri.to_string())
    }

    /// Get a mutable reference to a document
    pub fn get_document_mut(&mut self, uri: &Url) -> Option<&mut Document> {
        self.documents.get_mut(&uri.to_string())
    }

    /// Apply a change to a document
    pub fn apply_change(&mut self, uri: &Url, change: TextDocumentContentChangeEvent, version: i32) {
        if let Some(document) = self.get_document_mut(uri) {
            document.apply_change(change, version);
        }
    }

    /// Close a document
    pub fn close_document(&mut self, uri: &Url) {
        self.documents.remove(&uri.to_string());
    }

    /// Get all open documents
    pub fn get_all_documents(&self) -> Vec<&Document> {
        self.documents.values().collect()
    }

    /// Get document count
    pub fn document_count(&self) -> usize {
        self.documents.len()
    }

    /// Check if a document is open
    pub fn is_document_open(&self, uri: &Url) -> bool {
        self.documents.contains_key(&uri.to_string())
    }

    /// Detect language ID from URI
    fn detect_language_id(&self, uri: &Url) -> String {
        if let Some(path) = uri.path().split('/').last() {
            if path.ends_with(".fhirpath") || path.ends_with(".fp") {
                "fhirpath".to_string()
            } else if path.ends_with(".json") {
                "json".to_string()
            } else if path.ends_with(".xml") {
                "xml".to_string()
            } else {
                "plaintext".to_string()
            }
        } else {
            "fhirpath".to_string() // Default to FHIRPath
        }
    }

    /// Get document statistics
    pub fn get_stats(&self) -> DocumentManagerStats {
        let mut total_lines = 0;
        let mut total_characters = 0;
        
        for document in self.documents.values() {
            total_lines += document.line_count();
            total_characters += document.text.len() as u32;
        }

        DocumentManagerStats {
            document_count: self.documents.len(),
            total_lines,
            total_characters,
        }
    }
}

/// Statistics about the document manager
#[derive(Debug, Clone)]
pub struct DocumentManagerStats {
    /// Number of open documents
    pub document_count: usize,
    /// Total lines across all documents
    pub total_lines: u32,
    /// Total characters across all documents
    pub total_characters: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::{Position, Range};

    #[test]
    fn test_document_creation() {
        let uri = Url::parse("file:///test.fhirpath").unwrap();
        let text = "Patient.name.given".to_string();
        let document = Document::new(uri.clone(), text.clone(), 1, "fhirpath".to_string());

        assert_eq!(document.uri, uri);
        assert_eq!(document.text, text);
        assert_eq!(document.version, 1);
        assert_eq!(document.language_id, "fhirpath");
    }

    #[test]
    fn test_position_to_offset() {
        let uri = Url::parse("file:///test.fhirpath").unwrap();
        let text = "line1\nline2\nline3".to_string();
        let document = Document::new(uri, text, 1, "fhirpath".to_string());

        // Test various positions
        assert_eq!(document.position_to_offset(Position::new(0, 0)), 0);
        assert_eq!(document.position_to_offset(Position::new(0, 5)), 5);
        assert_eq!(document.position_to_offset(Position::new(1, 0)), 6);
        assert_eq!(document.position_to_offset(Position::new(1, 5)), 11);
        assert_eq!(document.position_to_offset(Position::new(2, 0)), 12);
    }

    #[test]
    fn test_document_changes() {
        let uri = Url::parse("file:///test.fhirpath").unwrap();
        let mut document = Document::new(uri, "Patient.name".to_string(), 1, "fhirpath".to_string());

        // Test full document change
        let full_change = TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: "Patient.name.given".to_string(),
        };
        document.apply_change(full_change, 2);
        assert_eq!(document.text, "Patient.name.given");
        assert_eq!(document.version, 2);

        // Test incremental change
        let incremental_change = TextDocumentContentChangeEvent {
            range: Some(Range::new(Position::new(0, 12), Position::new(0, 18))),
            range_length: Some(6),
            text: "family".to_string(),
        };
        document.apply_change(incremental_change, 3);
        assert_eq!(document.text, "Patient.name.family");
        assert_eq!(document.version, 3);
    }

    #[test]
    fn test_document_manager() {
        let mut manager = DocumentManager::new();
        let uri = Url::parse("file:///test.fhirpath").unwrap();

        // Test opening document
        manager.open_document(uri.clone(), "Patient.name".to_string(), 1);
        assert!(manager.is_document_open(&uri));
        assert_eq!(manager.document_count(), 1);

        // Test getting document
        let document = manager.get_document(&uri).unwrap();
        assert_eq!(document.text, "Patient.name");

        // Test closing document
        manager.close_document(&uri);
        assert!(!manager.is_document_open(&uri));
        assert_eq!(manager.document_count(), 0);
    }
}