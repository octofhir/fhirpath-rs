//! LSP message handlers

use crate::analyzer::{AnalysisContext, StaticAnalyzer};
use crate::core::ModelProvider;
use crate::diagnostics::AriadneDiagnostic;
use crate::evaluator::FunctionRegistry;
use crate::lsp::code_actions::CodeActionProvider;
use crate::lsp::completion::{CompletionContext, CompletionProvider};
use crate::lsp::diagnostics::DiagnosticProvider;
use crate::lsp::document::DocumentManager;
use crate::lsp::document_symbols::DocumentSymbolProvider;
use crate::lsp::goto_definition::GotoDefinitionProvider;
use crate::lsp::hover::HoverProvider;
use crate::lsp::semantic_tokens::SemanticTokensProvider;
use crate::lsp::signature_help::SignatureHelpProvider;
use crate::lsp::{LspContext, SetContextParams};

use lsp_types::{
    CodeActionParams, CodeActionResponse, CompletionParams, CompletionResponse, Diagnostic,
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DocumentSymbolParams, DocumentSymbolResponse, GotoDefinitionParams, GotoDefinitionResponse,
    Hover, HoverParams, InitializeParams, InitializeResult, Position, SemanticTokensFullOptions,
    SemanticTokensOptions, SemanticTokensParams, SemanticTokensRangeParams, SemanticTokensResult,
    SemanticTokensServerCapabilities, ServerCapabilities, SignatureHelp, SignatureHelpOptions,
    SignatureHelpParams, TextDocumentSyncCapability, TextDocumentSyncKind,
};
use octofhir_fhir_model::TypeInfo;
use std::sync::Arc;

/// LSP handlers for server implementation
pub struct LspHandlers {
    documents: DocumentManager,
    completion_provider: CompletionProvider,
    hover_provider: HoverProvider,
    signature_help_provider: SignatureHelpProvider,
    goto_definition_provider: GotoDefinitionProvider,
    model_provider: Arc<dyn ModelProvider + Send + Sync>,
    context: LspContext,
}

impl LspHandlers {
    /// Create new LSP handlers
    pub fn new(
        completion_provider: CompletionProvider,
        model_provider: Arc<dyn ModelProvider + Send + Sync>,
        function_registry: Arc<FunctionRegistry>,
    ) -> Self {
        let hover_provider = HoverProvider::new(model_provider.clone(), function_registry.clone());
        let signature_help_provider = SignatureHelpProvider::new(function_registry);
        let goto_definition_provider = GotoDefinitionProvider::new(model_provider.clone());

        Self {
            documents: DocumentManager::new(),
            completion_provider,
            hover_provider,
            signature_help_provider,
            goto_definition_provider,
            model_provider,
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
            hover_provider: Some(lsp_types::HoverProviderCapability::Simple(true)),
            signature_help_provider: Some(SignatureHelpOptions {
                trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
                retrigger_characters: None,
                work_done_progress_options: Default::default(),
            }),
            semantic_tokens_provider: Some(
                SemanticTokensServerCapabilities::SemanticTokensOptions(SemanticTokensOptions {
                    legend: crate::lsp::semantic_tokens::build_legend(),
                    full: Some(SemanticTokensFullOptions::Bool(true)),
                    range: Some(true),
                    work_done_progress_options: Default::default(),
                }),
            ),
            code_action_provider: Some(lsp_types::CodeActionProviderCapability::Simple(true)),
            definition_provider: Some(lsp_types::OneOf::Left(true)),
            document_symbol_provider: Some(lsp_types::OneOf::Left(true)),
            ..Default::default()
        }
    }

    /// Handle textDocument/didOpen notification
    pub async fn did_open(&mut self, params: DidOpenTextDocumentParams) -> Vec<Diagnostic> {
        let uri = params.text_document.uri;
        let content = params.text_document.text;
        let version = params.text_document.version;

        self.documents.open(uri.clone(), content.clone(), version);

        // Analyze and return diagnostics
        let (lsp_diagnostics, ariadne_diagnostics) = self.analyze_document(&content, &uri).await;

        // Store diagnostics for code actions
        if let Some(doc) = self.documents.get_mut(&uri) {
            doc.last_diagnostics = ariadne_diagnostics;
        }

        lsp_diagnostics
    }

    /// Handle textDocument/didChange notification
    pub async fn did_change(&mut self, params: DidChangeTextDocumentParams) -> Vec<Diagnostic> {
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        // Full sync - take the last content change
        if let Some(change) = params.content_changes.into_iter().last() {
            self.documents.update(&uri, change.text.clone(), version);

            // Analyze and return diagnostics
            let (lsp_diagnostics, ariadne_diagnostics) =
                self.analyze_document(&change.text, &uri).await;

            // Store diagnostics for code actions
            if let Some(doc) = self.documents.get_mut(&uri) {
                doc.last_diagnostics = ariadne_diagnostics;
            }

            return lsp_diagnostics;
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

    /// Handle textDocument/hover request
    pub async fn hover(&self, params: HoverParams) -> Option<Hover> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let doc = self.documents.get(uri)?;
        let offset = Self::position_to_offset(&doc.content, position)?;

        self.hover_provider
            .provide(&doc.content, offset, &self.context)
            .await
    }

    /// Handle textDocument/signatureHelp request
    pub fn signature_help(&self, params: SignatureHelpParams) -> Option<SignatureHelp> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let doc = self.documents.get(uri)?;
        let offset = Self::position_to_offset(&doc.content, position)?;

        self.signature_help_provider.provide(&doc.content, offset)
    }

    /// Handle textDocument/semanticTokens/full request
    pub fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Option<SemanticTokensResult> {
        let uri = &params.text_document.uri;
        let doc = self.documents.get(uri)?;

        let tokens = SemanticTokensProvider::tokenize(&doc.content);

        Some(SemanticTokensResult::Tokens(lsp_types::SemanticTokens {
            result_id: None,
            data: tokens,
        }))
    }

    /// Handle textDocument/semanticTokens/range request
    pub fn semantic_tokens_range(
        &self,
        params: SemanticTokensRangeParams,
    ) -> Option<lsp_types::SemanticTokensRangeResult> {
        let uri = &params.text_document.uri;
        let doc = self.documents.get(uri)?;

        let start_offset = Self::position_to_offset(&doc.content, params.range.start).unwrap_or(0);
        let end_offset =
            Self::position_to_offset(&doc.content, params.range.end).unwrap_or(doc.content.len());

        let tokens = SemanticTokensProvider::tokenize_range(&doc.content, start_offset, end_offset);

        Some(lsp_types::SemanticTokensRangeResult::Tokens(
            lsp_types::SemanticTokens {
                result_id: None,
                data: tokens,
            },
        ))
    }

    /// Handle textDocument/definition request
    pub async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Option<GotoDefinitionResponse> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let doc = self.documents.get(uri)?;
        let offset = Self::position_to_offset(&doc.content, position)?;

        self.goto_definition_provider
            .provide(&doc.content, offset, &self.context)
            .await
    }

    /// Handle textDocument/documentSymbol request
    pub fn document_symbol(&self, params: DocumentSymbolParams) -> Option<DocumentSymbolResponse> {
        let uri = &params.text_document.uri;
        let doc = self.documents.get(uri)?;

        let symbols = DocumentSymbolProvider::provide(&doc.content);
        if symbols.is_empty() {
            None
        } else {
            Some(DocumentSymbolResponse::Nested(symbols))
        }
    }

    /// Handle textDocument/codeAction request
    pub fn code_action(&self, params: CodeActionParams) -> Option<CodeActionResponse> {
        let uri = &params.text_document.uri;
        let doc = self.documents.get(uri)?;

        let actions = CodeActionProvider::provide(
            &doc.last_diagnostics,
            &doc.content,
            uri,
            &params.context.diagnostics,
        );

        if actions.is_empty() {
            None
        } else {
            Some(
                actions
                    .into_iter()
                    .map(lsp_types::CodeActionOrCommand::CodeAction)
                    .collect(),
            )
        }
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

    /// Analyze a document and return both LSP diagnostics and raw AriadneDiagnostics
    async fn analyze_document(
        &self,
        content: &str,
        uri: &lsp_types::Url,
    ) -> (Vec<Diagnostic>, Vec<AriadneDiagnostic>) {
        // Determine root type for analysis
        let root_type = self
            .context
            .resource_type
            .as_ref()
            .map(|rt| TypeInfo {
                type_name: rt.clone(),
                singleton: Some(true),
                is_empty: Some(false),
                namespace: Some("FHIR".to_string()),
                name: Some(rt.clone()),
            })
            .unwrap_or_else(|| {
                // Try to infer from expression
                let inferred = CompletionContext::infer_resource_type(content);
                if let Some(rt) = inferred {
                    TypeInfo {
                        type_name: rt.clone(),
                        singleton: Some(true),
                        is_empty: Some(false),
                        namespace: Some("FHIR".to_string()),
                        name: Some(rt),
                    }
                } else {
                    TypeInfo::new_complex("Resource")
                }
            });

        // Use StaticAnalyzer for comprehensive analysis
        let mut analyzer = StaticAnalyzer::new(self.model_provider.clone());
        let analysis_context = AnalysisContext {
            root_type,
            deep_analysis: false,
            suggest_optimizations: false,
            max_suggestions: 0,
        };

        let result = analyzer.analyze_expression(content, analysis_context).await;

        // Convert AriadneDiagnostics to LSP Diagnostics using DiagnosticProvider
        let lsp_diagnostics: Vec<Diagnostic> = result
            .diagnostics
            .iter()
            .map(|d| DiagnosticProvider::convert_with_uri(d, content, uri))
            .collect();

        (lsp_diagnostics, result.diagnostics)
    }

    /// Convert LSP Position to byte offset
    pub(crate) fn position_to_offset(text: &str, position: Position) -> Option<usize> {
        let mut offset = 0;

        for (current_line, line) in text.lines().enumerate() {
            if current_line as u32 == position.line {
                // Found the line, add character offset
                let char_offset = line
                    .char_indices()
                    .nth(position.character as usize)
                    .map(|(i, _)| i)
                    .unwrap_or(line.len());
                return Some(offset + char_offset);
            }
            offset += line.len() + 1; // +1 for newline
        }

        // Position beyond end of text
        Some(text.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluator::create_function_registry;
    use octofhir_fhir_model::EmptyModelProvider;

    fn create_test_handlers() -> LspHandlers {
        let model_provider: Arc<dyn ModelProvider + Send + Sync> = Arc::new(EmptyModelProvider);
        let function_registry = Arc::new(create_function_registry());
        let completion_provider =
            CompletionProvider::new(model_provider.clone(), function_registry.clone());
        LspHandlers::new(completion_provider, model_provider, function_registry)
    }

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

    #[test]
    fn test_initialize_returns_all_capabilities() {
        let handlers = create_test_handlers();
        let result = handlers.initialize(InitializeParams::default());

        let caps = result.capabilities;
        assert!(caps.completion_provider.is_some());
        assert!(caps.hover_provider.is_some());
        assert!(caps.signature_help_provider.is_some());
        assert!(caps.semantic_tokens_provider.is_some());
        assert!(caps.code_action_provider.is_some());
        assert!(caps.definition_provider.is_some());
        assert!(caps.document_symbol_provider.is_some());
        assert!(caps.text_document_sync.is_some());
    }

    #[tokio::test]
    async fn test_did_open_returns_diagnostics() {
        let mut handlers = create_test_handlers();
        let uri = lsp_types::Url::parse("file:///test.fhirpath").unwrap();

        let params = DidOpenTextDocumentParams {
            text_document: lsp_types::TextDocumentItem {
                uri: uri.clone(),
                language_id: "fhirpath".to_string(),
                version: 1,
                text: "Patient.name".to_string(),
            },
        };

        let _diagnostics = handlers.did_open(params).await;
        // Valid expression should produce no errors (or just warnings)
        // With EmptyModelProvider, property validation may produce diagnostics
        assert!(handlers.documents().get(&uri).is_some());
    }

    #[tokio::test]
    async fn test_did_change_updates_document() {
        let mut handlers = create_test_handlers();
        let uri = lsp_types::Url::parse("file:///test.fhirpath").unwrap();

        // Open
        let open_params = DidOpenTextDocumentParams {
            text_document: lsp_types::TextDocumentItem {
                uri: uri.clone(),
                language_id: "fhirpath".to_string(),
                version: 1,
                text: "Patient.name".to_string(),
            },
        };
        handlers.did_open(open_params).await;

        // Change
        let change_params = DidChangeTextDocumentParams {
            text_document: lsp_types::VersionedTextDocumentIdentifier {
                uri: uri.clone(),
                version: 2,
            },
            content_changes: vec![lsp_types::TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: "Patient.name.given".to_string(),
            }],
        };
        handlers.did_change(change_params).await;

        let doc = handlers.documents().get(&uri).unwrap();
        assert_eq!(doc.content, "Patient.name.given");
        assert_eq!(doc.version, 2);
    }

    #[test]
    fn test_signature_help_where() {
        let handlers = create_test_handlers();

        // Test the provider directly
        let result = handlers
            .signature_help_provider
            .provide("Patient.name.where(", 19);
        assert!(result.is_some());
        let help = result.unwrap();
        assert_eq!(help.signatures.len(), 1);
        assert!(help.signatures[0].label.contains("where"));
    }

    #[test]
    fn test_semantic_tokens_simple_expression() {
        let tokens = SemanticTokensProvider::tokenize("1 + 2");
        // Parser now provides locations, so we should get tokens for the literals and operator
        assert!(
            !tokens.is_empty(),
            "Should produce semantic tokens now that parser tracks spans"
        );
    }

    #[test]
    fn test_set_and_clear_context() {
        let mut handlers = create_test_handlers();

        handlers.set_context(SetContextParams {
            resource_type: Some("Observation".to_string()),
            constants: None,
        });
        assert_eq!(
            handlers.context().resource_type,
            Some("Observation".to_string())
        );

        handlers.clear_context();
        assert!(handlers.context().resource_type.is_none());
    }
}
