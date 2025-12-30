//! LSP message handlers

use crate::lsp::completion::{CompletionContext, CompletionProvider};
use crate::lsp::document::DocumentManager;
use crate::lsp::{LspContext, SetContextParams};
use crate::parser::parse_with_analysis;

use lsp_types::{
    CompletionParams, CompletionResponse, Diagnostic, DidChangeTextDocumentParams,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, InitializeParams, InitializeResult,
    Position, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind,
};

/// LSP handlers for server implementation
pub struct LspHandlers {
    documents: DocumentManager,
    completion_provider: CompletionProvider,
    context: LspContext,
}

impl LspHandlers {
    /// Create new LSP handlers
    pub fn new(completion_provider: CompletionProvider) -> Self {
        Self {
            documents: DocumentManager::new(),
            completion_provider,
            context: LspContext::new(),
        }
    }

    /// Handle initialize request
    pub fn initialize(&self, _params: InitializeParams) -> InitializeResult {
        InitializeResult {
            capabilities: Self::server_capabilities(),
            server_info: Some(lsp_types::ServerInfo {
                name: "fhirpath-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        }
    }

    /// Get server capabilities
    pub fn server_capabilities() -> ServerCapabilities {
        ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
            completion_provider: Some(lsp_types::CompletionOptions {
                trigger_characters: Some(vec![
                    ".".to_string(),
                    " ".to_string(),
                    "(".to_string(),
                    "%".to_string(),
                    "$".to_string(),
                ]),
                resolve_provider: Some(false),
                work_done_progress_options: Default::default(),
                all_commit_characters: None,
                completion_item: None,
            }),
            // No hover - not needed per requirements
            hover_provider: None,
            ..Default::default()
        }
    }

    /// Handle textDocument/didOpen notification
    pub fn did_open(&mut self, params: DidOpenTextDocumentParams) -> Vec<Diagnostic> {
        let uri = params.text_document.uri;
        let content = params.text_document.text;
        let version = params.text_document.version as i32;

        self.documents.open(uri.clone(), content.clone(), version);

        // Analyze and return diagnostics
        self.analyze_document(&content)
    }

    /// Handle textDocument/didChange notification
    pub fn did_change(&mut self, params: DidChangeTextDocumentParams) -> Vec<Diagnostic> {
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        // Full sync - take the last content change
        if let Some(change) = params.content_changes.into_iter().last() {
            self.documents.update(&uri, change.text.clone(), version);

            // Analyze and return diagnostics
            return self.analyze_document(&change.text);
        }

        Vec::new()
    }

    /// Handle textDocument/didClose notification
    pub fn did_close(&mut self, params: DidCloseTextDocumentParams) {
        self.documents.close(&params.text_document.uri);
    }

    /// Handle textDocument/completion request
    pub async fn completion(&self, params: CompletionParams) -> Option<CompletionResponse> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let doc = self.documents.get(uri)?;

        // Convert LSP position to byte offset
        let offset = Self::position_to_offset(&doc.content, position)?;

        // Get trigger character
        let trigger_char = params
            .context
            .as_ref()
            .and_then(|ctx| ctx.trigger_character.as_ref())
            .and_then(|s| s.chars().next());

        // Analyze completion context
        let context = CompletionContext::analyze(&doc.content, offset, trigger_char);

        // Get completions
        let items = self
            .completion_provider
            .provide(&context, &self.context)
            .await;

        Some(CompletionResponse::Array(items))
    }

    /// Handle fhirpath/setContext notification
    pub fn set_context(&mut self, params: SetContextParams) {
        self.context.update(params);
    }

    /// Handle fhirpath/clearContext notification
    pub fn clear_context(&mut self) {
        self.context.clear();
    }

    /// Get context reference
    pub fn context(&self) -> &LspContext {
        &self.context
    }

    /// Get documents reference
    pub fn documents(&self) -> &DocumentManager {
        &self.documents
    }

    /// Get mutable documents reference
    pub fn documents_mut(&mut self) -> &mut DocumentManager {
        &mut self.documents
    }

    /// Analyze a document and return diagnostics
    fn analyze_document(&self, content: &str) -> Vec<Diagnostic> {
        let result = parse_with_analysis(content);

        result
            .diagnostics
            .iter()
            .filter_map(|d| {
                let location = d.location.as_ref()?;

                Some(Diagnostic {
                    range: lsp_types::Range {
                        start: lsp_types::Position {
                            line: location.line as u32,
                            character: location.column as u32,
                        },
                        end: lsp_types::Position {
                            line: location.line as u32,
                            character: (location.column + location.length) as u32,
                        },
                    },
                    severity: Some(match d.severity {
                        crate::diagnostics::DiagnosticSeverity::Error => {
                            lsp_types::DiagnosticSeverity::ERROR
                        }
                        crate::diagnostics::DiagnosticSeverity::Warning => {
                            lsp_types::DiagnosticSeverity::WARNING
                        }
                        crate::diagnostics::DiagnosticSeverity::Info => {
                            lsp_types::DiagnosticSeverity::INFORMATION
                        }
                        crate::diagnostics::DiagnosticSeverity::Hint => {
                            lsp_types::DiagnosticSeverity::HINT
                        }
                    }),
                    code: Some(lsp_types::NumberOrString::String(d.code.code.clone())),
                    code_description: None,
                    source: Some("fhirpath".to_string()),
                    message: d.message.clone(),
                    related_information: None,
                    tags: None,
                    data: None,
                })
            })
            .collect()
    }

    /// Convert LSP Position to byte offset
    fn position_to_offset(text: &str, position: Position) -> Option<usize> {
        let mut offset = 0;
        let mut current_line = 0u32;

        for line in text.lines() {
            if current_line == position.line {
                // Found the line, add character offset
                let char_offset = line
                    .char_indices()
                    .nth(position.character as usize)
                    .map(|(i, _)| i)
                    .unwrap_or(line.len());
                return Some(offset + char_offset);
            }
            offset += line.len() + 1; // +1 for newline
            current_line += 1;
        }

        // Position beyond end of text
        Some(text.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_to_offset_first_line() {
        let text = "Patient.name";
        let pos = Position {
            line: 0,
            character: 8,
        };
        assert_eq!(LspHandlers::position_to_offset(text, pos), Some(8));
    }

    #[test]
    fn test_position_to_offset_second_line() {
        let text = "Patient\n.name";
        let pos = Position {
            line: 1,
            character: 1,
        };
        assert_eq!(LspHandlers::position_to_offset(text, pos), Some(9));
    }
}
