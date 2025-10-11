//! Main LSP server implementation

use dashmap::DashMap;
use std::sync::Arc;
use tokio::time::Duration;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use url::Url;

use crate::cache::{AnalysisCache, CompletionCache};
use crate::config::Config;
use crate::document::FhirPathDocument;
use crate::features::code_actions::generate_code_actions;
use crate::features::completion::generate_completions;
use crate::features::diagnostics::generate_diagnostics;
use crate::features::document_symbols::generate_document_symbols;
use crate::features::goto_definition::goto_definition;
use crate::features::hover::generate_hover;
use crate::features::inlay_hints::generate_inlay_hints;
use crate::features::semantic_tokens::generate_semantic_tokens;
use crate::metrics::{MetricOperation, MetricsTracker};
use octofhir_fhirpath::FhirPathEngine;

/// FHIRPath Language Server
pub struct FhirPathLanguageServer {
    /// LSP client handle
    client: Client,
    /// Server configuration
    config: Arc<parking_lot::RwLock<Config>>,
    /// Document cache (URI -> Document)
    documents: Arc<DashMap<Url, FhirPathDocument>>,
    /// Analysis result cache
    analysis_cache: Arc<AnalysisCache>,
    /// Completion cache
    completion_cache: Arc<CompletionCache>,
    /// FHIRPath engine (initialized lazily in initialized() callback)
    engine: Arc<tokio::sync::RwLock<Option<FhirPathEngine>>>,
    /// Workspace folders (URI -> WorkspaceFolder)
    workspace_folders: Arc<DashMap<Url, WorkspaceFolder>>,
    /// Performance metrics tracker
    metrics: MetricsTracker,
}

impl Clone for FhirPathLanguageServer {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            config: self.config.clone(),
            documents: self.documents.clone(),
            analysis_cache: self.analysis_cache.clone(),
            completion_cache: self.completion_cache.clone(),
            engine: self.engine.clone(),
            workspace_folders: self.workspace_folders.clone(),
            metrics: self.metrics.clone(),
        }
    }
}

impl FhirPathLanguageServer {
    /// Create a new language server instance (engine initialized in initialized() callback)
    pub fn new(client: Client) -> Self {
        Self::new_with_config(client, Config::default())
    }

    /// Create a new language server instance with specific config
    pub fn new_with_config(client: Client, config: Config) -> Self {
        Self {
            client,
            config: Arc::new(parking_lot::RwLock::new(config)),
            documents: Arc::new(DashMap::new()),
            analysis_cache: Arc::new(AnalysisCache::default()),
            completion_cache: Arc::new(CompletionCache::default()),
            engine: Arc::new(tokio::sync::RwLock::new(None)),
            workspace_folders: Arc::new(DashMap::new()),
            metrics: MetricsTracker::new(),
        }
    }

    /// Initialize FHIRPath engine asynchronously (called from initialized() callback)
    async fn initialize_engine(&self) {
        let config = self.config();
        let model_provider = Arc::new(config.fhir_version.create_embedded_provider());
        let registry = octofhir_fhirpath::create_function_registry();

        // Create engine asynchronously - no blocking!
        match FhirPathEngine::new(
            Arc::new(registry),
            model_provider as Arc<dyn octofhir_fhirpath::ModelProvider + Send + Sync>,
        )
        .await
        {
            Ok(engine) => {
                tracing::info!(
                    "Initialized FHIRPath engine with FHIR version: {:?}",
                    config.fhir_version
                );
                *self.engine.write().await = Some(engine);
            }
            Err(e) => {
                tracing::error!("Failed to initialize FHIRPath engine: {}", e);
            }
        }
    }

    /// Publish diagnostics for a document
    async fn publish_diagnostics(&self, uri: Url) {
        if let Some(doc) = self.get_document(&uri) {
            let start = std::time::Instant::now();

            let engine_guard = self.engine.read().await;
            let diagnostics = if let Some(ref engine) = *engine_guard {
                generate_diagnostics(&doc, engine).await
            } else {
                // Engine not initialized yet, skip diagnostics
                tracing::warn!("Engine not initialized, skipping diagnostics");
                Vec::new()
            };
            drop(engine_guard);

            // Record metrics
            let duration = start.elapsed();
            self.metrics.track(MetricOperation::Diagnostic, || duration);
            self.metrics.document_processed();

            self.client
                .publish_diagnostics(uri.clone(), diagnostics, Some(doc.version))
                .await;
        }
    }

    /// Get current configuration
    pub fn config(&self) -> Config {
        self.config.read().clone()
    }

    /// Update configuration
    pub fn update_config(&self, config: Config) {
        *self.config.write() = config;
    }

    /// Get document by URI
    pub fn get_document(&self, uri: &Url) -> Option<FhirPathDocument> {
        self.documents.get(uri).map(|doc| doc.clone())
    }

    /// Insert or update document
    pub fn insert_document(&self, uri: Url, document: FhirPathDocument) {
        self.documents.insert(uri, document);
    }

    /// Remove document
    pub fn remove_document(&self, uri: &Url) -> Option<FhirPathDocument> {
        self.documents.remove(uri).map(|(_, doc)| doc)
    }

    /// Get analysis cache
    pub fn analysis_cache(&self) -> &Arc<AnalysisCache> {
        &self.analysis_cache
    }

    /// Get completion cache
    pub fn completion_cache(&self) -> &Arc<CompletionCache> {
        &self.completion_cache
    }

    /// Get FHIRPath engine (async access)
    pub fn engine(&self) -> &Arc<tokio::sync::RwLock<Option<FhirPathEngine>>> {
        &self.engine
    }

    /// Reload configuration from file
    pub async fn reload_config(&self, config_path: &std::path::Path) {
        match Config::from_file(config_path) {
            Ok(config) => {
                tracing::info!("Configuration reloaded successfully");
                self.update_config(config);

                // Clear caches on config change (FHIR version might change)
                self.analysis_cache.clear();
                self.completion_cache.clear();

                self.client
                    .log_message(MessageType::INFO, "Configuration reloaded, caches cleared")
                    .await;
            }
            Err(e) => {
                tracing::error!("Failed to reload configuration: {}", e);
                self.client
                    .log_message(
                        MessageType::ERROR,
                        format!("Failed to reload configuration: {}", e),
                    )
                    .await;
            }
        }
    }

    /// Find workspace folder for a given document URI
    pub fn find_workspace_folder(&self, document_uri: &Url) -> Option<WorkspaceFolder> {
        // Find the workspace folder that contains this document
        for entry in self.workspace_folders.iter() {
            let workspace_uri = entry.key();
            let workspace_path = workspace_uri.path();
            let document_path = document_uri.path();

            // Check if document path starts with workspace path
            if document_path.starts_with(workspace_path) {
                return Some(entry.value().clone());
            }
        }
        None
    }

    /// Load configuration for a specific workspace folder
    pub fn load_workspace_config(&self, workspace_uri: &Url) -> Config {
        // Try to load .fhirpath-lsp.toml from workspace root
        if let Ok(workspace_path) = workspace_uri.to_file_path() {
            let config_path = workspace_path.join(".fhirpath-lsp.toml");

            if config_path.exists() {
                match Config::from_file(&config_path) {
                    Ok(config) => {
                        tracing::info!(
                            "Loaded workspace-specific config from: {}",
                            config_path.display()
                        );
                        return config;
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to load workspace config from {}: {}",
                            config_path.display(),
                            e
                        );
                    }
                }
            }
        }

        // Fall back to global config
        self.config()
    }

    /// Get configuration for a document (workspace-specific or global)
    pub fn get_document_config(&self, document_uri: &Url) -> Config {
        if let Some(workspace_folder) = self.find_workspace_folder(document_uri) {
            self.load_workspace_config(&workspace_folder.uri)
        } else {
            self.config()
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for FhirPathLanguageServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        tracing::info!("LSP initialize request received");

        // Store and log workspace folders
        if let Some(folders) = params.workspace_folders {
            for folder in folders {
                tracing::info!("Workspace folder: {}", folder.uri);
                self.workspace_folders.insert(folder.uri.clone(), folder);
            }
        }

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![
                        ".".to_string(),
                        "(".to_string(),
                        "$".to_string(),
                    ]),
                    ..Default::default()
                }),
                definition_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: SemanticTokensLegend {
                                token_types: vec![
                                    SemanticTokenType::KEYWORD,
                                    SemanticTokenType::FUNCTION,
                                    SemanticTokenType::OPERATOR,
                                    SemanticTokenType::VARIABLE,
                                    SemanticTokenType::PROPERTY,
                                    SemanticTokenType::NUMBER,
                                    SemanticTokenType::STRING,
                                    SemanticTokenType::COMMENT,
                                ],
                                token_modifiers: vec![
                                    SemanticTokenModifier::READONLY,
                                    SemanticTokenModifier::DEPRECATED,
                                    SemanticTokenModifier::DEFINITION,
                                ],
                            },
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            range: Some(true),
                            ..Default::default()
                        },
                    ),
                ),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                inlay_hint_provider: Some(OneOf::Left(true)),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "fhirpath-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        tracing::info!("LSP server initialized, starting engine initialization...");

        // Initialize engine asynchronously - no blocking!
        self.initialize_engine().await;

        self.client
            .log_message(MessageType::INFO, "FHIRPath LSP server ready")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        tracing::info!("LSP shutdown request received");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        let version = params.text_document.version;

        tracing::info!("Document opened: {}", uri);

        let document = FhirPathDocument::new(uri.clone(), text, version);
        self.insert_document(uri.clone(), document);

        // Publish diagnostics immediately
        self.publish_diagnostics(uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        tracing::debug!("Document changed: {}", uri);

        if let Some(mut doc) = self.get_document(&uri) {
            for change in params.content_changes {
                doc.apply_change(change, version);
            }

            // Invalidate cache for this document
            self.analysis_cache.invalidate_document(&uri);

            self.insert_document(uri.clone(), doc);

            // Debounced diagnostics update (300ms delay)
            let uri_clone = uri.clone();
            let server = self.clone();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(300)).await;
                server.publish_diagnostics(uri_clone).await;
            });
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        tracing::info!("Document saved: {}", params.text_document.uri);
        // Full reanalysis on save (immediate, no debounce)
        self.publish_diagnostics(params.text_document.uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        tracing::info!("Document closed: {}", uri);

        // Remove from cache
        self.analysis_cache.invalidate_document(&uri);
        self.remove_document(&uri);
    }

    async fn did_change_configuration(&self, _params: DidChangeConfigurationParams) {
        tracing::info!("Configuration changed");
        // TODO: Reload configuration
    }

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        for change in params.changes {
            tracing::info!("File changed: {} ({:?})", change.uri, change.typ);
            // TODO: Handle .fhirpath-lsp.toml changes
        }
    }

    async fn did_change_workspace_folders(&self, params: DidChangeWorkspaceFoldersParams) {
        tracing::info!("Workspace folders changed");

        // Add new workspace folders
        for folder in params.event.added {
            tracing::info!("Workspace folder added: {}", folder.uri);
            self.workspace_folders.insert(folder.uri.clone(), folder);
        }

        // Remove workspace folders
        for folder in params.event.removed {
            tracing::info!("Workspace folder removed: {}", folder.uri);
            self.workspace_folders.remove(&folder.uri);
        }

        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "Workspace folders updated: {} total",
                    self.workspace_folders.len()
                ),
            )
            .await;
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri;

        tracing::debug!("Semantic tokens requested for: {}", uri);

        if let Some(doc) = self.get_document(&uri) {
            let tokens = generate_semantic_tokens(&doc);
            Ok(Some(tokens))
        } else {
            tracing::warn!("Document not found for semantic tokens: {}", uri);
            Ok(None)
        }
    }

    async fn semantic_tokens_range(
        &self,
        params: SemanticTokensRangeParams,
    ) -> Result<Option<SemanticTokensRangeResult>> {
        let uri = params.text_document.uri;

        tracing::debug!("Semantic tokens range requested for: {}", uri);

        // For now, return full tokens (range filtering can be optimized later)
        if let Some(doc) = self.get_document(&uri) {
            let tokens = generate_semantic_tokens(&doc);
            match tokens {
                SemanticTokensResult::Tokens(tokens) => {
                    Ok(Some(SemanticTokensRangeResult::Tokens(tokens)))
                }
                SemanticTokensResult::Partial(_) => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        tracing::debug!("Completion requested for: {} at {:?}", uri, position);

        if let Some(doc) = self.get_document(&uri) {
            Ok(generate_completions(&doc, position))
        } else {
            tracing::warn!("Document not found for completion: {}", uri);
            Ok(None)
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        tracing::debug!("Hover requested for: {} at {:?}", uri, position);

        if let Some(doc) = self.get_document(&uri) {
            Ok(generate_hover(&doc, position))
        } else {
            tracing::warn!("Document not found for hover: {}", uri);
            Ok(None)
        }
    }

    async fn inlay_hint(&self, params: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
        let uri = params.text_document.uri;
        let range = params.range;

        tracing::debug!("Inlay hints requested for: {} in range {:?}", uri, range);

        if let Some(doc) = self.get_document(&uri) {
            let hints = generate_inlay_hints(&doc, range);
            Ok(Some(hints))
        } else {
            tracing::warn!("Document not found for inlay hints: {}", uri);
            Ok(None)
        }
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = params.text_document.uri;
        let range = params.range;
        let diagnostics = params.context.diagnostics;

        tracing::debug!("Code actions requested for: {} in range {:?}", uri, range);

        if let Some(doc) = self.get_document(&uri) {
            let actions = generate_code_actions(&doc, range, &diagnostics);
            Ok(Some(actions))
        } else {
            tracing::warn!("Document not found for code actions: {}", uri);
            Ok(None)
        }
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        tracing::debug!("Goto definition requested for: {} at {:?}", uri, position);

        if let Some(doc) = self.get_document(&uri) {
            Ok(goto_definition(&doc, position))
        } else {
            tracing::warn!("Document not found for goto definition: {}", uri);
            Ok(None)
        }
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri;

        tracing::debug!("Document symbols requested for: {}", uri);

        if let Some(doc) = self.get_document(&uri) {
            Ok(Some(generate_document_symbols(&doc)))
        } else {
            tracing::warn!("Document not found for document symbols: {}", uri);
            Ok(None)
        }
    }
}
