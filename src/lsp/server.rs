//! Async-first FHIRPath Language Server implementation
//!
//! This module implements the core LSP server using tower-lsp with full async integration
//! to the ModelProvider and analyzer systems.

use crate::analyzer::{FhirPathAnalyzer, AnalyzerConfig};
use crate::model::provider::ModelProvider;
use crate::registry::FunctionRegistry;
use lsp_types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::{Client, LanguageServer, LspService, Server};

/// Configuration for the LSP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspConfig {
    /// Enable detailed logging
    pub enable_logging: bool,
    /// Maximum analysis depth
    pub max_analysis_depth: u32,
    /// Enable completion suggestions
    pub enable_completions: bool,
    /// Enable diagnostics
    pub enable_diagnostics: bool,
    /// Enable hover information
    pub enable_hover: bool,
    /// Enable go-to-definition
    pub enable_navigation: bool,
    /// Completion trigger characters
    pub completion_triggers: Vec<String>,
    /// Diagnostic update delay in milliseconds
    pub diagnostic_delay_ms: u64,
}

impl Default for LspConfig {
    fn default() -> Self {
        Self {
            enable_logging: true,
            max_analysis_depth: 50,
            enable_completions: true,
            enable_diagnostics: true,
            enable_hover: true,
            enable_navigation: true,
            completion_triggers: vec![".".to_string(), "(".to_string(), " ".to_string()],
            diagnostic_delay_ms: 300,
        }
    }
}

/// The main FHIRPath Language Server
pub struct FhirPathLanguageServer<P: ModelProvider> {
    /// LSP client connection
    client: Client,
    /// Document manager for tracking open documents
    document_manager: Arc<RwLock<super::document_manager::DocumentManager>>,
    /// Analyzer for static analysis
    analyzer: Arc<FhirPathAnalyzer<P>>,
    /// Function registry
    function_registry: Arc<FunctionRegistry>,
    /// Server configuration
    config: LspConfig,
    /// Client capabilities
    client_capabilities: RwLock<Option<ClientCapabilities>>,
}

impl<P: ModelProvider + 'static> FhirPathLanguageServer<P> {
    /// Create a new language server
    pub fn new(
        client: Client,
        model_provider: Arc<P>,
        function_registry: Arc<FunctionRegistry>,
        config: LspConfig,
    ) -> Self {
        let analyzer_config = AnalyzerConfig {
            detailed_type_inference: true,
            enable_completions: config.enable_completions,
            enable_diagnostics: config.enable_diagnostics,
            max_analysis_depth: config.max_analysis_depth,
            enable_symbol_tracking: config.enable_navigation,
        };

        let analyzer = Arc::new(FhirPathAnalyzer::with_config(model_provider, analyzer_config));
        let document_manager = Arc::new(RwLock::new(
            super::document_manager::DocumentManager::new(),
        ));

        Self {
            client,
            document_manager,
            analyzer,
            function_registry,
            config,
            client_capabilities: RwLock::new(None),
        }
    }

    /// Get server capabilities
    fn server_capabilities(&self) -> ServerCapabilities {
        let completion_provider = if self.config.enable_completions {
            Some(CompletionOptions {
                resolve_provider: Some(true),
                trigger_characters: Some(self.config.completion_triggers.clone()),
                all_commit_characters: None,
                work_done_progress_options: WorkDoneProgressOptions::default(),
                completion_item: Some(CompletionOptionsCompletionItem {
                    label_details_support: Some(true),
                }),
            })
        } else {
            None
        };

        let hover_provider = if self.config.enable_hover {
            Some(HoverProviderCapability::Simple(true))
        } else {
            None
        };

        let definition_provider = if self.config.enable_navigation {
            Some(OneOf::Left(true))
        } else {
            None
        };

        ServerCapabilities {
            position_encoding: Some(PositionEncodingKind::UTF16),
            text_document_sync: Some(TextDocumentSyncCapability::Kind(
                TextDocumentSyncKind::INCREMENTAL,
            )),
            completion_provider,
            hover_provider,
            definition_provider,
            diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                DiagnosticOptions {
                    identifier: Some("fhirpath".to_string()),
                    inter_file_dependencies: false,
                    workspace_diagnostics: false,
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                },
            )),
            ..Default::default()
        }
    }

    /// Log message to client
    async fn log_message(&self, typ: MessageType, message: impl AsRef<str>) {
        if self.config.enable_logging {
            self.client
                .log_message(typ, message.as_ref())
                .await;
        }
    }

    /// Show message to user
    async fn show_message(&self, typ: MessageType, message: impl AsRef<str>) {
        self.client
            .show_message(typ, message.as_ref())
            .await;
    }

    /// Analyze document and publish diagnostics
    async fn analyze_and_publish_diagnostics(&self, uri: &Url) {
        if !self.config.enable_diagnostics {
            return;
        }

        // Add delay to avoid excessive analysis during fast typing
        tokio::time::sleep(tokio::time::Duration::from_millis(self.config.diagnostic_delay_ms)).await;

        let document_manager = self.document_manager.read().await;
        if let Some(document) = document_manager.get_document(uri) {
            let text = document.text.clone();
            drop(document_manager); // Release the read lock

            // Parse and analyze the document
            match crate::parser::parse(&text) {
                Ok(expression) => {
                    match self
                        .analyzer
                        .analyze(&expression, Some("Resource")) // Default to Resource context
                        .await
                    {
                        Ok(analysis_result) => {
                            // Convert diagnostics to LSP format
                            let diagnostics: Vec<Diagnostic> = analysis_result
                                .diagnostics
                                .into_iter()
                                .map(|diag| self.convert_diagnostic(diag, &text))
                                .collect();

                            // Publish diagnostics
                            self.client
                                .publish_diagnostics(uri.clone(), diagnostics, None)
                                .await;

                            self.log_message(
                                MessageType::INFO,
                                format!("Analysis complete for {}", uri),
                            )
                            .await;
                        }
                        Err(err) => {
                            self.log_message(
                                MessageType::ERROR,
                                format!("Analysis error for {}: {}", uri, err),
                            )
                            .await;
                        }
                    }
                }
                Err(parse_error) => {
                    // Create diagnostic for parse error
                    let diagnostic = Diagnostic {
                        range: Range::new(Position::new(0, 0), Position::new(0, text.len() as u32)),
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: Some(NumberOrString::String("parse-error".to_string())),
                        code_description: None,
                        source: Some("fhirpath".to_string()),
                        message: format!("Parse error: {}", parse_error),
                        related_information: None,
                        tags: None,
                        data: None,
                    };

                    self.client
                        .publish_diagnostics(uri.clone(), vec![diagnostic], None)
                        .await;
                }
            }
        }
    }

    /// Convert analyzer diagnostic to LSP diagnostic
    fn convert_diagnostic(&self, diagnostic: crate::analyzer::diagnostics::Diagnostic, text: &str) -> Diagnostic {
        // Convert span to LSP range
        let range = self.span_to_range(&diagnostic.span, text);

        Diagnostic {
            range,
            severity: Some(match diagnostic.severity {
                crate::analyzer::diagnostics::DiagnosticSeverity::Error => DiagnosticSeverity::ERROR,
                crate::analyzer::diagnostics::DiagnosticSeverity::Warning => DiagnosticSeverity::WARNING,
                crate::analyzer::diagnostics::DiagnosticSeverity::Information => DiagnosticSeverity::INFORMATION,
                crate::analyzer::diagnostics::DiagnosticSeverity::Hint => DiagnosticSeverity::HINT,
            }),
            code: diagnostic.code.map(NumberOrString::String),
            source: Some("fhirpath".to_string()),
            message: diagnostic.message,
            related_information: None,
            tags: None,
            data: None,
            code_description: None,
        }
    }

    /// Convert span to LSP range
    fn span_to_range(&self, span: &Option<crate::parser::span::Span>, text: &str) -> Range {
        if let Some(span) = span {
            let start_pos = self.offset_to_position(span.start, text);
            let end_pos = self.offset_to_position(span.end, text);
            Range::new(start_pos, end_pos)
        } else {
            // Default to start of document if no span
            Range::new(Position::new(0, 0), Position::new(0, 0))
        }
    }

    /// Convert byte offset to LSP position
    fn offset_to_position(&self, offset: usize, text: &str) -> Position {
        let mut line = 0u32;
        let mut character = 0u32;
        
        for (i, ch) in text.char_indices() {
            if i >= offset {
                break;
            }
            
            if ch == '\n' {
                line += 1;
                character = 0;
            } else {
                character += 1;
            }
        }
        
        Position::new(line, character)
    }
}

#[tower_lsp::async_trait]
impl<P: ModelProvider + 'static> LanguageServer for FhirPathLanguageServer<P> {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        // Store client capabilities
        *self.client_capabilities.write().await = Some(params.capabilities.clone());

        self.log_message(MessageType::INFO, "FHIRPath Language Server initializing").await;

        Ok(InitializeResult {
            capabilities: self.server_capabilities(),
            server_info: Some(ServerInfo {
                name: "fhirpath-language-server".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.show_message(MessageType::INFO, "FHIRPath Language Server initialized").await;
        self.log_message(MessageType::INFO, "Server ready for requests").await;
    }

    async fn shutdown(&self) -> Result<()> {
        self.log_message(MessageType::INFO, "Server shutting down").await;
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        let version = params.text_document.version;

        // Store document
        {
            let mut document_manager = self.document_manager.write().await;
            document_manager.open_document(uri.clone(), text, version);
        }

        // Trigger analysis
        self.analyze_and_publish_diagnostics(&uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        // Update document
        {
            let mut document_manager = self.document_manager.write().await;
            for change in params.content_changes {
                document_manager.apply_change(&uri, change, version);
            }
        }

        // Trigger analysis
        self.analyze_and_publish_diagnostics(&uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;

        // Remove document
        {
            let mut document_manager = self.document_manager.write().await;
            document_manager.close_document(&uri);
        }

        // Clear diagnostics
        self.client.publish_diagnostics(uri, vec![], None).await;
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        if !self.config.enable_completions {
            return Ok(None);
        }

        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let document_manager = self.document_manager.read().await;
        let document = match document_manager.get_document(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let text = document.text.clone();
        drop(document_manager);

        // Get completion items using the analyzer
        match super::completion::get_completions(
            &text,
            position,
            &self.analyzer,
            &self.function_registry,
        )
        .await
        {
            Ok(completions) => {
                self.log_message(
                    MessageType::INFO,
                    format!("Providing {} completions", completions.len()),
                )
                .await;
                
                Ok(Some(CompletionResponse::Array(completions)))
            }
            Err(err) => {
                self.log_message(MessageType::ERROR, format!("Completion error: {}", err))
                    .await;
                Ok(None)
            }
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        if !self.config.enable_hover {
            return Ok(None);
        }

        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let document_manager = self.document_manager.read().await;
        let document = match document_manager.get_document(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let text = document.text.clone();
        drop(document_manager);

        // Get hover information
        match super::hover::get_hover(&text, position, &self.analyzer).await {
            Ok(Some(hover)) => Ok(Some(hover)),
            Ok(None) => Ok(None),
            Err(err) => {
                self.log_message(MessageType::ERROR, format!("Hover error: {}", err))
                    .await;
                Ok(None)
            }
        }
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        if !self.config.enable_navigation {
            return Ok(None);
        }

        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let document_manager = self.document_manager.read().await;
        let document = match document_manager.get_document(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let text = document.text.clone();
        drop(document_manager);

        // Get definition location
        match super::navigation::get_definition(&text, position, &self.analyzer).await {
            Ok(Some(location)) => Ok(Some(GotoDefinitionResponse::Scalar(location))),
            Ok(None) => Ok(None),
            Err(err) => {
                self.log_message(MessageType::ERROR, format!("Navigation error: {}", err))
                    .await;
                Ok(None)
            }
        }
    }
}

/// Create and start the LSP server
pub async fn start_server<P: ModelProvider + 'static>(
    model_provider: Arc<P>,
    function_registry: Arc<FunctionRegistry>,
    config: LspConfig,
) -> Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| {
        FhirPathLanguageServer::new(client, model_provider, function_registry, config)
    })
    .finish();

    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}