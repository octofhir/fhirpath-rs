//! FHIRPath LSP Server implementation

use crate::core::ModelProvider;
use crate::evaluator::create_function_registry;
use crate::lsp::SetContextParams;
use crate::lsp::completion::CompletionProvider;
use crate::lsp::handlers::LspHandlers;

use async_lsp::ClientSocket;
use async_lsp::concurrency::ConcurrencyLayer;
use async_lsp::panic::CatchUnwindLayer;
use async_lsp::router::Router;
use async_lsp::tracing::TracingLayer;
use lsp_types::notification::{
    DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, Initialized, Notification,
    PublishDiagnostics,
};
use lsp_types::request::{Completion, Initialize, Shutdown};
use lsp_types::{
    CompletionParams, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, InitializeParams, PublishDiagnosticsParams,
};
use std::ops::ControlFlow;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use tower::ServiceBuilder;

/// Transport type for the LSP server
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Transport {
    /// Standard I/O (default for editors)
    #[default]
    Stdio,
    /// WebSocket (for web integration)
    WebSocket,
}

/// Configuration for the LSP server
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Transport type
    pub transport: Transport,
    /// WebSocket port (only used when transport is WebSocket)
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            transport: Transport::Stdio,
            port: 8081,
        }
    }
}

/// Custom notification for setting FHIRPath context
pub struct SetContext;

impl Notification for SetContext {
    type Params = SetContextParams;
    const METHOD: &'static str = "fhirpath/setContext";
}

/// Custom notification for clearing FHIRPath context
pub struct ClearContext;

impl Notification for ClearContext {
    type Params = ();
    const METHOD: &'static str = "fhirpath/clearContext";
}

/// FHIRPath Language Server state
pub struct FhirPathLspServer {
    handlers: Arc<Mutex<LspHandlers>>,
    client: ClientSocket,
}

impl FhirPathLspServer {
    /// Create a new LSP server
    pub fn new(model_provider: Arc<dyn ModelProvider + Send + Sync>, client: ClientSocket) -> Self {
        let function_registry = Arc::new(create_function_registry());
        let completion_provider = CompletionProvider::new(model_provider, function_registry);
        let handlers = LspHandlers::new(completion_provider);

        Self {
            handlers: Arc::new(Mutex::new(handlers)),
            client,
        }
    }

    /// Publish diagnostics to the client
    fn publish_diagnostics(&self, uri: lsp_types::Url, diagnostics: Vec<lsp_types::Diagnostic>) {
        let params = PublishDiagnosticsParams {
            uri,
            diagnostics,
            version: None,
        };

        if let Err(e) = self.client.notify::<PublishDiagnostics>(params) {
            tracing::error!("Failed to publish diagnostics: {}", e);
        }
    }
}

/// Run the LSP server with stdio transport
pub async fn run_stdio(
    model_provider: Arc<dyn ModelProvider + Send + Sync>,
) -> Result<(), async_lsp::Error> {
    let (mainloop, _) = async_lsp::MainLoop::new_server(|client| {
        let function_registry = Arc::new(create_function_registry());
        let completion_provider =
            CompletionProvider::new(model_provider.clone(), function_registry);
        let handlers = LspHandlers::new(completion_provider);

        let server = FhirPathLspServer {
            handlers: Arc::new(Mutex::new(handlers)),
            client: client.clone(),
        };

        let mut router = Router::new(server);

        // Initialize request
        router.request::<Initialize, _>(|server, params: InitializeParams| {
            let handlers = server.handlers.clone();
            async move {
                let handlers = handlers.lock().await;
                Ok(handlers.initialize(params))
            }
        });

        // Shutdown request
        router.request::<Shutdown, _>(|_server, _params: ()| async move { Ok(()) });

        // Completion request
        router.request::<Completion, _>(|server, params: CompletionParams| {
            let handlers = server.handlers.clone();
            async move {
                let handlers = handlers.lock().await;
                Ok(handlers.completion(params).await)
            }
        });

        // Initialized notification
        router.notification::<Initialized>(|_server, _params| {
            tracing::info!("FHIRPath LSP server initialized");
            ControlFlow::Continue(())
        });

        // didOpen notification
        router.notification::<DidOpenTextDocument>(|server, params: DidOpenTextDocumentParams| {
            let uri = params.text_document.uri.clone();

            let diagnostics = {
                let handlers = server.handlers.clone();
                let rt = tokio::runtime::Handle::current();
                rt.block_on(async {
                    let mut handlers = handlers.lock().await;
                    handlers.did_open(params)
                })
            };

            server.publish_diagnostics(uri, diagnostics);
            ControlFlow::Continue(())
        });

        // didChange notification
        router.notification::<DidChangeTextDocument>(
            |server, params: DidChangeTextDocumentParams| {
                let uri = params.text_document.uri.clone();

                let diagnostics = {
                    let handlers = server.handlers.clone();
                    let rt = tokio::runtime::Handle::current();
                    rt.block_on(async {
                        let mut handlers = handlers.lock().await;
                        handlers.did_change(params)
                    })
                };

                server.publish_diagnostics(uri, diagnostics);
                ControlFlow::Continue(())
            },
        );

        // didClose notification
        router.notification::<DidCloseTextDocument>(
            |server, params: DidCloseTextDocumentParams| {
                let handlers = server.handlers.clone();
                let rt = tokio::runtime::Handle::current();
                rt.block_on(async {
                    let mut handlers = handlers.lock().await;
                    handlers.did_close(params);
                });
                ControlFlow::Continue(())
            },
        );

        // setContext notification
        router.notification::<SetContext>(|server, params: SetContextParams| {
            let handlers = server.handlers.clone();
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let mut handlers = handlers.lock().await;
                handlers.set_context(params);
            });
            tracing::debug!("FHIRPath context updated");
            ControlFlow::Continue(())
        });

        // clearContext notification
        router.notification::<ClearContext>(|server, _params: ()| {
            let handlers = server.handlers.clone();
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let mut handlers = handlers.lock().await;
                handlers.clear_context();
            });
            tracing::debug!("FHIRPath context cleared");
            ControlFlow::Continue(())
        });

        ServiceBuilder::new()
            .layer(TracingLayer::default())
            .layer(CatchUnwindLayer::default())
            .layer(ConcurrencyLayer::default())
            .service(router)
    });

    let stdin = tokio::io::stdin().compat();
    let stdout = tokio::io::stdout().compat_write();
    mainloop.run_buffered(stdin, stdout).await
}

/// Run the LSP server with WebSocket transport
pub async fn run_websocket(
    model_provider: Arc<dyn ModelProvider + Send + Sync>,
    port: u16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use axum::{
        Router,
        extract::State,
        extract::ws::{Message, WebSocket, WebSocketUpgrade},
        response::IntoResponse,
        routing::get,
    };
    use futures::{SinkExt, StreamExt};
    use tokio::sync::mpsc;
    use tower_http::cors::{Any, CorsLayer};

    /// Application state shared across WebSocket connections
    #[derive(Clone)]
    struct AppState {
        model_provider: Arc<dyn ModelProvider + Send + Sync>,
    }

    /// Handle WebSocket upgrade request
    async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
        ws.on_upgrade(move |socket| handle_socket(socket, state))
    }

    /// Handle a single WebSocket connection
    async fn handle_socket(socket: WebSocket, state: AppState) {
        let (mut ws_sender, mut ws_receiver) = socket.split();

        // Create channels for LSP communication
        let (lsp_tx, mut lsp_rx) = mpsc::unbounded_channel::<String>();
        let (response_tx, mut response_rx) = mpsc::unbounded_channel::<String>();

        // Create LSP handlers
        let function_registry = Arc::new(create_function_registry());
        let completion_provider =
            CompletionProvider::new(state.model_provider.clone(), function_registry);
        let handlers = Arc::new(Mutex::new(LspHandlers::new(completion_provider)));

        // Spawn task to forward responses to WebSocket
        let send_task = tokio::spawn(async move {
            while let Some(msg) = response_rx.recv().await {
                if ws_sender.send(Message::Text(msg.into())).await.is_err() {
                    break;
                }
            }
        });

        // Spawn task to handle incoming LSP messages
        let handlers_clone = handlers.clone();
        let response_tx_clone = response_tx.clone();
        let handle_task = tokio::spawn(async move {
            while let Some(msg) = lsp_rx.recv().await {
                if let Some(response) =
                    process_lsp_message(&msg, &handlers_clone, &response_tx_clone).await
                {
                    if response_tx_clone.send(response).is_err() {
                        break;
                    }
                }
            }
        });

        // Process incoming WebSocket messages
        while let Some(Ok(msg)) = ws_receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if lsp_tx.send(text.to_string()).is_err() {
                        break;
                    }
                }
                Message::Binary(data) => {
                    if let Ok(text) = String::from_utf8(data.to_vec()) {
                        if lsp_tx.send(text).is_err() {
                            break;
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }

        // Clean up
        drop(lsp_tx);
        let _ = handle_task.await;
        let _ = send_task.await;
    }

    /// Process a single LSP message and return response if any
    async fn process_lsp_message(
        message: &str,
        handlers: &Arc<Mutex<LspHandlers>>,
        response_tx: &mpsc::UnboundedSender<String>,
    ) -> Option<String> {
        // Parse the LSP message (JSON-RPC)
        let json: serde_json::Value = match serde_json::from_str(message) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("Failed to parse LSP message: {}", e);
                return None;
            }
        };

        let method = json.get("method")?.as_str()?;
        let id = json.get("id");
        let params = json.get("params");

        match method {
            "initialize" => {
                let params: InitializeParams = serde_json::from_value(params?.clone()).ok()?;
                let handlers = handlers.lock().await;
                let result = handlers.initialize(params);
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id?,
                    "result": result
                });
                Some(serde_json::to_string(&response).ok()?)
            }
            "shutdown" => {
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id?,
                    "result": null
                });
                Some(serde_json::to_string(&response).ok()?)
            }
            "textDocument/completion" => {
                let params: CompletionParams = serde_json::from_value(params?.clone()).ok()?;
                let handlers = handlers.lock().await;
                let result = handlers.completion(params).await;
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id?,
                    "result": result
                });
                Some(serde_json::to_string(&response).ok()?)
            }
            "textDocument/didOpen" => {
                let params: DidOpenTextDocumentParams =
                    serde_json::from_value(params?.clone()).ok()?;
                let uri = params.text_document.uri.clone();
                let mut handlers = handlers.lock().await;
                let diagnostics = handlers.did_open(params);
                // Send diagnostics notification
                let notification = serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "textDocument/publishDiagnostics",
                    "params": {
                        "uri": uri.to_string(),
                        "diagnostics": diagnostics
                    }
                });
                let _ = response_tx.send(serde_json::to_string(&notification).ok()?);
                None
            }
            "textDocument/didChange" => {
                let params: DidChangeTextDocumentParams =
                    serde_json::from_value(params?.clone()).ok()?;
                let uri = params.text_document.uri.clone();
                let mut handlers = handlers.lock().await;
                let diagnostics = handlers.did_change(params);
                // Send diagnostics notification
                let notification = serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "textDocument/publishDiagnostics",
                    "params": {
                        "uri": uri.to_string(),
                        "diagnostics": diagnostics
                    }
                });
                let _ = response_tx.send(serde_json::to_string(&notification).ok()?);
                None
            }
            "textDocument/didClose" => {
                let params: DidCloseTextDocumentParams =
                    serde_json::from_value(params?.clone()).ok()?;
                let mut handlers = handlers.lock().await;
                handlers.did_close(params);
                None
            }
            "fhirpath/setContext" => {
                let params: SetContextParams = serde_json::from_value(params?.clone()).ok()?;
                let mut handlers = handlers.lock().await;
                handlers.set_context(params);
                tracing::debug!("FHIRPath context updated via WebSocket");
                None
            }
            "fhirpath/clearContext" => {
                let mut handlers = handlers.lock().await;
                handlers.clear_context();
                tracing::debug!("FHIRPath context cleared via WebSocket");
                None
            }
            "initialized" => {
                tracing::info!("FHIRPath LSP server initialized via WebSocket");
                None
            }
            _ => {
                tracing::warn!("Unknown LSP method: {}", method);
                if id.is_some() {
                    // Send error response for unknown requests
                    let response = serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id?,
                        "error": {
                            "code": -32601,
                            "message": format!("Method not found: {}", method)
                        }
                    });
                    Some(serde_json::to_string(&response).ok()?)
                } else {
                    None
                }
            }
        }
    }

    // Set up CORS for browser access
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let state = AppState { model_provider };

    let app = Router::new()
        .route("/", get(ws_handler))
        .layer(cors)
        .with_state(state);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    tracing::info!("FHIRPath LSP WebSocket server listening on ws://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
