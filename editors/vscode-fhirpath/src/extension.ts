import * as path from 'path';
import { workspace, ExtensionContext, window } from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: ExtensionContext) {
  // Get configuration
  const config = workspace.getConfiguration('fhirpath');
  const serverExecutable = 'fhirpath-lsp'; // Assumes binary is in PATH

  // Check if server binary exists
  const serverOptions: ServerOptions = {
    command: serverExecutable,
    args: [],
    transport: TransportKind.stdio,
  };

  // Client options
  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: 'file', language: 'fhirpath' }],
    synchronize: {
      // Notify the server about file changes to .fhirpath-lsp.toml files
      fileEvents: workspace.createFileSystemWatcher('**/.fhirpath-lsp.toml'),
    },
    initializationOptions: {
      fhirVersion: config.get('fhirVersion', 'r5'),
      features: {
        diagnostics: config.get('features.diagnostics', true),
        completion: config.get('features.completion', true),
        hover: config.get('features.hover', true),
        inlayHints: config.get('features.inlayHints', true),
      },
    },
  };

  // Create the language client
  client = new LanguageClient(
    'fhirpath',
    'FHIRPath Language Server',
    serverOptions,
    clientOptions
  );

  // Start the client (this will also launch the server)
  client.start().catch((error) => {
    window.showErrorMessage(
      `Failed to start FHIRPath LSP: ${error.message}. Make sure 'fhirpath-lsp' is installed and in your PATH.`
    );
  });

  // Log activation
  console.log('FHIRPath extension activated');
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
