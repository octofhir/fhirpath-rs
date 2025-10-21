# FHIRPath for VS Code

Professional FHIRPath language support for Visual Studio Code with full Language Server Protocol (LSP) integration.

## Features

### 🎨 Rich Syntax Highlighting
Semantic token-based highlighting for keywords, functions, operators, and more.

### 🔍 IntelliSense
Context-aware code completion for:
- FHIRPath functions (where, select, first, etc.)
- FHIR resource properties (Patient.name, Observation.value, etc.)
- Variables ($this, $index, $total)
- Keywords (and, or, implies, as, is)

### ⚡ Real-time Diagnostics
Instant detection of:
- Syntax errors
- Type mismatches
- Undefined functions
- Invalid property access

### 📖 Hover Documentation
Inline documentation with:
- Function signatures and descriptions
- Parameter information
- Usage examples
- Type information from FHIR schema

### 💡 Inlay Hints
Display inline:
- Parameter names
- Type annotations
- Implicit conversions

### 🔧 Code Actions
Quick fixes for:
- Typo corrections
- Expression simplifications
- Common refactorings

### 🎯 Go to Definition
Navigate to file references in `@input-file` directives.

### 📋 Document Outline
View all expressions and directives in the outline panel.

## Installation

### 1. Install the LSP Server

Download the `fhirpath-lsp` binary from [GitHub Releases](https://github.com/octofhir/fhirpath-rs/releases):

**macOS:**
```bash
curl -L https://github.com/octofhir/fhirpath-rs/releases/latest/download/fhirpath-lsp-macos-x86_64 -o fhirpath-lsp
chmod +x fhirpath-lsp
sudo mv fhirpath-lsp /usr/local/bin/
```

**Linux:**
```bash
curl -L https://github.com/octofhir/fhirpath-rs/releases/latest/download/fhirpath-lsp-linux-x86_64 -o fhirpath-lsp
chmod +x fhirpath-lsp
sudo mv fhirpath-lsp /usr/local/bin/
```

**Windows:**
```powershell
# Download from releases and add to PATH
```

Or build from source:
```bash
git clone https://github.com/octofhir/fhirpath-rs
cd fhirpath-rs
cargo build --release -p fhirpath-lsp
# Binary at target/release/fhirpath-lsp
```

### 2. Install the Extension

**From VS Code Marketplace** (when available):
1. Open VS Code
2. Press `Ctrl+Shift+X` (or `Cmd+Shift+X` on macOS)
3. Search for "FHIRPath"
4. Click Install

**Manual Installation:**
1. Download the `.vsix` file from [GitHub Releases](https://github.com/octofhir/fhirpath-rs/releases)
2. In VS Code, press `Ctrl+Shift+P` (or `Cmd+Shift+P` on macOS)
3. Type "Extensions: Install from VSIX"
4. Select the downloaded `.vsix` file

## Usage

### Creating a FHIRPath File

1. Create a file with `.fhirpath` extension
2. Start writing FHIRPath expressions:

```fhirpath
/**
 * @input {
 *   "resourceType": "Patient",
 *   "id": "example",
 *   "name": [{"family": "Smith", "given": ["John"]}],
 *   "birthDate": "1970-01-01"
 * }
 */

// Get patient's family name
Patient.name.family;

// Get first given name
Patient.name.given.first();

// Check if patient is adult
Patient.birthDate <= today() - 18 years
```

### Using Input Directives

**Inline Resource:**
```fhirpath
/** @input { "resourceType": "Patient", ... } */
```

**External File:**
```fhirpath
/** @input-file ./examples/patient.json */
```

The LSP will provide autocomplete for file paths!

## Configuration

### Workspace Configuration

Create `.fhirpath-lsp.toml` in your workspace root:

```toml
fhir_version = "r5"  # Options: "r4", "r4b", "r5"

[model_provider]
cache_schemas = true
max_cache_size = 100

[features]
diagnostics = true
semantic_tokens = true
completion = true
hover = true
inlay_hints = true
code_actions = true
goto_definition = true

[performance]
diagnostic_debounce_ms = 300
max_document_size_kb = 1024
```

### VS Code Settings

Configure via Settings UI or `settings.json`:

```json
{
  "fhirpath.fhirVersion": "r5",
  "fhirpath.trace.server": "off",
  "fhirpath.features.diagnostics": true,
  "fhirpath.features.completion": true,
  "fhirpath.features.hover": true,
  "fhirpath.features.inlayHints": true
}
```

## Keyboard Shortcuts

| Action | Shortcut |
|--------|----------|
| Trigger Completion | `Ctrl+Space` |
| Show Hover | Hover mouse or `Ctrl+K Ctrl+I` |
| Go to Definition | `F12` or `Ctrl+Click` |
| Show Code Actions | `Ctrl+.` or click lightbulb |
| Format Document | `Shift+Alt+F` |

## Troubleshooting

### Extension Not Working

1. **Check LSP server installation:**
   ```bash
   which fhirpath-lsp
   fhirpath-lsp --version
   ```

2. **Check VS Code Output:**
   - View → Output
   - Select "FHIRPath Language Server" from dropdown

3. **Enable verbose logging:**
   ```json
   {
     "fhirpath.trace.server": "verbose"
   }
   ```

### Common Issues

**"Command 'fhirpath-lsp' not found"**
- Ensure the binary is in your PATH
- Try absolute path in settings

**No syntax highlighting**
- Ensure file has `.fhirpath` extension
- Reload VS Code window

**Completion not working**
- Check if LSP server is running (see Output panel)
- Verify `.fhirpath-lsp.toml` syntax

## Examples

See the [examples](https://github.com/octofhir/fhirpath-rs/tree/main/examples) directory for sample `.fhirpath` files.

## Contributing

Report issues or contribute at: https://github.com/octofhir/fhirpath-rs

## License

MIT OR Apache-2.0
