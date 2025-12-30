//! Document state management for the LSP server

use lsp_types::Url;
use std::collections::HashMap;

/// State of an open document
#[derive(Debug, Clone)]
pub struct DocumentState {
    /// Document content
    pub content: String,

    /// Document version (incremented on each change)
    pub version: i32,

    /// Document URI
    pub uri: Url,
}

impl DocumentState {
    /// Create a new document state
    pub fn new(uri: Url, content: String, version: i32) -> Self {
        Self {
            content,
            version,
            uri,
        }
    }

    /// Update the document content
    pub fn update(&mut self, content: String, version: i32) {
        self.content = content;
        self.version = version;
    }
}

/// Manager for open documents
#[derive(Debug, Default)]
pub struct DocumentManager {
    documents: HashMap<Url, DocumentState>,
}

impl DocumentManager {
    /// Create a new document manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Open a document
    pub fn open(&mut self, uri: Url, content: String, version: i32) {
        self.documents
            .insert(uri.clone(), DocumentState::new(uri, content, version));
    }

    /// Update a document
    pub fn update(&mut self, uri: &Url, content: String, version: i32) {
        if let Some(doc) = self.documents.get_mut(uri) {
            doc.update(content, version);
        }
    }

    /// Close a document
    pub fn close(&mut self, uri: &Url) {
        self.documents.remove(uri);
    }

    /// Get a document by URI
    pub fn get(&self, uri: &Url) -> Option<&DocumentState> {
        self.documents.get(uri)
    }

    /// Get all open documents
    pub fn all(&self) -> impl Iterator<Item = &DocumentState> {
        self.documents.values()
    }
}
