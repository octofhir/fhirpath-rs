# FHIRPath for IntelliJ IDEA

Professional FHIRPath language support for IntelliJ IDEA with Language Server Protocol (LSP) integration.

## Features

### 🎨 Rich Syntax Highlighting
Semantic token-based highlighting powered by LSP.

### 🔍 IntelliSense
Context-aware code completion for:
- FHIRPath functions (where, select, first, etc.)
- FHIR resource properties (Patient.name, Observation.value, etc.)
- Variables ($this, $index, $total)
- Keywords (and, or, implies, as, is)

### ⚡ Real-time Diagnostics
Instant detection of syntax errors, type mismatches, and undefined functions.

### 📖 Hover Documentation
Inline documentation with function signatures, examples, and type information.

### 💡 Inlay Hints
Display inline parameter names, type annotations, and implicit conversions.

### 🔧 Code Actions
Quick fixes for common errors and expression simplifications.

### 🎯 Go to Definition
Navigate to file references in `@input-file` directives.

### 📋 Document Structure
View all expressions and directives in the structure panel.

## Requirements

- **IntelliJ IDEA Ultimate** (Community Edition does not support LSP)
- **fhirpath-lsp** binary installed in PATH

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

### 2. Install the Plugin

**From JetBrains Marketplace** (when available):
1. Open IntelliJ IDEA
2. Go to Settings/Preferences → Plugins
3. Search for "FHIRPath"
4. Click Install

**Manual Installation:**
1. Download the plugin ZIP from [GitHub Releases](https://github.com/octofhir/fhirpath-rs/releases)
2. Go to Settings/Preferences → Plugins
3. Click ⚙️ (gear icon) → Install Plugin from Disk
4. Select the downloaded ZIP file
5. Restart IntelliJ IDEA

### 3. Verify Installation

1. Check that `fhirpath-lsp` is in PATH:
   ```bash
   which fhirpath-lsp
   fhirpath-lsp --version
   ```

2. Create a test file:
   - File → New → File
   - Name it `test.fhirpath`
   - IntelliJ should recognize it as a FHIRPath file

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

Create `.fhirpath-lsp.toml` in your project root:

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

### IntelliJ Settings

Configure LSP behavior:
1. Settings → Languages & Frameworks → Language Server Protocol
2. Find "FHIRPath" in the list
3. Configure server options if needed

## Keyboard Shortcuts

| Action | Shortcut |
|--------|----------|
| Trigger Completion | `Ctrl+Space` |
| Show Hover | `Ctrl+Q` (Windows/Linux) or `F1` (macOS) |
| Go to Definition | `Ctrl+B` or `Ctrl+Click` |
| Show Code Actions | `Alt+Enter` |
| Reformat Code | `Ctrl+Alt+L` (Windows/Linux) or `Cmd+Option+L` (macOS) |

## Troubleshooting

### Plugin Not Loading

1. **Verify IntelliJ version:**
   - Requires IntelliJ IDEA Ultimate 2023.2 or later
   - Community Edition is NOT supported (no LSP)

2. **Check plugin installation:**
   - Settings → Plugins → Installed
   - Look for "FHIRPath"

### LSP Server Not Starting

1. **Check server installation:**
   ```bash
   which fhirpath-lsp
   fhirpath-lsp --version
   ```

2. **View LSP logs:**
   - Help → Show Log in Finder/Explorer
   - Look for LSP-related errors

3. **Test server manually:**
   ```bash
   fhirpath-lsp
   # Should start and wait for LSP messages
   ```

### No Syntax Highlighting

1. Ensure file has `.fhirpath` extension
2. Invalidate caches: File → Invalidate Caches → Invalidate and Restart

### Common Errors

**"Cannot find 'fhirpath-lsp' command"**
- Add the binary to your PATH
- Or configure absolute path in LSP settings

**"LSP server crashed"**
- Check LSP logs for errors
- Verify server binary is compatible with your OS

## Development

To build the plugin from source:

```bash
cd editors/idea-fhirpath
./gradlew buildPlugin
# Plugin at build/distributions/fhirpath-idea-plugin-0.1.0.zip
```

## Examples

See the [examples](https://github.com/octofhir/fhirpath-rs/tree/main/examples) directory for sample `.fhirpath` files.

## Contributing

Report issues or contribute at: https://github.com/octofhir/fhirpath-rs

## License

MIT OR Apache-2.0
