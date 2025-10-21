# FHIRPath for Zed

FHIRPath language support for Zed editor with full Language Server Protocol (LSP) integration.

## Features

- **Syntax Highlighting**: Rich semantic token-based highlighting
- **Real-time Diagnostics**: Instant syntax and semantic error detection
- **IntelliSense**: Context-aware code completion for functions, properties, and keywords
- **Hover Documentation**: Inline documentation with examples and type information
- **Inlay Hints**: Type annotations and parameter names
- **Code Actions**: Quick fixes and refactoring suggestions
- **Go to Definition**: Navigate to file references in `@input-file` directives
- **Document Symbols**: Outline view for easy navigation

## Installation

### Prerequisites

1. Install the `fhirpath-lsp` binary:
   ```bash
   # Download from GitHub releases
   curl -L https://github.com/octofhir/fhirpath-rs/releases/latest/download/fhirpath-lsp-$(uname -s)-$(uname -m) -o fhirpath-lsp
   chmod +x fhirpath-lsp
   sudo mv fhirpath-lsp /usr/local/bin/
   ```

   Or build from source:
   ```bash
   git clone https://github.com/octofhir/fhirpath-rs
   cd fhirpath-rs
   cargo build --release -p fhirpath-lsp
   sudo cp target/release/fhirpath-lsp /usr/local/bin/
   ```

### Extension Installation

1. Open Zed editor
2. Press `Cmd+Shift+P` (macOS) or `Ctrl+Shift+P` (Linux/Windows)
3. Type "zed: extensions"
4. Search for "FHIRPath"
5. Click Install

Or install manually:
```bash
# Copy extension to Zed extensions directory
mkdir -p ~/.config/zed/extensions
cd editors/zed-fhirpath
tar czf zed-fhirpath-0.1.0.tar.gz .
mv zed-fhirpath-0.1.0.tar.gz ~/.config/zed/extensions/
```

## Configuration

Create a `.fhirpath-lsp.toml` file in your workspace root:

```toml
# FHIR version to use for schema validation
fhir_version = "r5"  # Options: "r4", "r4b", "r5"

# Model provider settings
[model_provider]
cache_schemas = true
max_cache_size = 100

# LSP feature toggles
[features]
diagnostics = true
semantic_tokens = true
completion = true
hover = true
inlay_hints = true
code_actions = true
goto_definition = true

# Performance settings
[performance]
diagnostic_debounce_ms = 300
max_document_size_kb = 1024
```

## Usage

1. Create a `.fhirpath` file
2. Add FHIRPath expressions separated by semicolons
3. Optionally add input directives at the top:

```fhirpath
/**
 * @input {
 *   "resourceType": "Patient",
 *   "id": "example",
 *   "name": [{"family": "Smith", "given": ["John"]}]
 * }
 */

Patient.name.family;
Patient.name.given.first()
```

## Keyboard Shortcuts

- **Trigger Completion**: `Ctrl+Space`
- **Show Hover**: Hover with mouse or `Cmd+K Cmd+I`
- **Go to Definition**: `F12` or `Cmd+Click`
- **Show Code Actions**: `Cmd+.` or lightbulb icon

## Troubleshooting

### LSP server not starting

Check if `fhirpath-lsp` is in your PATH:
```bash
which fhirpath-lsp
fhirpath-lsp --version
```

### View LSP logs

Enable LSP logging in Zed:
1. Open Zed settings
2. Add: `"lsp": { "fhirpath-lsp": { "log_level": "debug" } }`
3. Check logs in Zed's log panel

### Extension not loading

Verify extension installation:
```bash
ls ~/.config/zed/extensions/
```

## Contributing

Report issues or contribute at: https://github.com/octofhir/fhirpath-rs

## License

MIT OR Apache-2.0
