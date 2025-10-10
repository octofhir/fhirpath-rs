# FHIRPath CLI

[![Crate](https://img.shields.io/crates/v/fhirpath-cli.svg)](https://crates.io/crates/fhirpath-cli)
[![Documentation](https://docs.rs/fhirpath-cli/badge.svg)](https://docs.rs/fhirpath-cli)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

Command-line tools for FHIRPath expression evaluation and analysis against FHIR resources.

## Features

- 🔍 **Expression Evaluation** - Evaluate FHIRPath expressions against FHIR resources
- 🎨 **Interactive REPL** - Command-line interface with history, completion, and syntax highlighting
- 📺 **Terminal UI (TUI)** - Advanced multi-panel interface with real-time diagnostics
- 🌐 **HTTP Server** - REST API for remote FHIRPath evaluation
- 📊 **Multiple Output Formats** - JSON, Pretty (Ariadne), Raw text
- 🔄 **Watch Mode** - Auto-reload and re-evaluate on file changes
- 📦 **Batch Processing** - Process multiple files with glob patterns
- ⚡ **Performance Profiling** - Detailed timing breakdowns
- 🔧 **Static Analysis** - Validate and analyze expressions
- 💾 **Configuration** - File-based config with favorites and defaults
- 🐚 **Shell Completion** - Generate completions for Bash, Zsh, Fish, PowerShell

## Installation

### From Source

```bash
cd crates/fhirpath-cli
cargo install --path .
```

### From Crates.io (when published)

```bash
# Install with default features (CLI + REPL)
cargo install fhirpath-cli

# Install with all features
cargo install fhirpath-cli --all-features

# Install with specific features
cargo install fhirpath-cli --no-default-features --features "cli,tui,server"
```

### Feature Flags

The CLI can be built with different feature combinations to optimize binary size and compilation time:

| Feature | Description | Default | Dependencies |
|---------|-------------|---------|--------------|
| `cli` | Core CLI functionality | ✅ Yes | Minimal |
| `repl` | Interactive REPL | ✅ Yes | reedline, fuzzy-matcher |
| `tui` | Terminal User Interface | ❌ No | ratatui, crossterm |
| `server` | HTTP API server | ❌ No | axum, tower-http |
| `watch` | File watching | ❌ No | notify |
| `all` | All features | ❌ No | All of the above |

**Examples:**

```bash
# Minimal build (CLI only)
cargo build --no-default-features --features cli

# With REPL and TUI
cargo build --features "tui"

# Everything
cargo build --all-features
```

**Binary Size Impact:**

- Minimal (`cli` only): ~8-10 MB
- Default (`cli + repl`): ~12-15 MB  
- All features: ~18-20 MB

Commands available per feature:
- `cli`: evaluate, validate, analyze, docs, registry, completions, config
- `repl`: + repl command
- `tui`: + tui command
- `server`: + server command
- `watch`: + --watch flag for evaluate command

## Quick Start

### Basic Evaluation

Evaluate a FHIRPath expression against a FHIR resource:

```bash
# From file
octofhir-fhirpath evaluate "Patient.name.family" --input patient.json

# From stdin
cat patient.json | octofhir-fhirpath evaluate "Patient.name.family"

# With variables
octofhir-fhirpath evaluate "Patient.name.where(use = %givenUse)" \
  --input patient.json \
  --var "givenUse=official"

# Pretty output (default)
octofhir-fhirpath evaluate "Patient.name" --input patient.json

# JSON output
octofhir-fhirpath evaluate "Patient.name" --input patient.json --output-format json

# With analysis
octofhir-fhirpath evaluate "Patient.name.family" --input patient.json --analyze
```

### Interactive REPL

Start an interactive session:

```bash
# Basic REPL
octofhir-fhirpath repl

# With initial resource
octofhir-fhirpath repl --input patient.json

# With variables
octofhir-fhirpath repl --input patient.json --variable "env=production"
```

REPL commands:
- `:help` - Show help
- `:load <file>` - Load FHIR resource
- `:set <var>=<value>` - Set variable
- `:vars` - List variables
- `:clear` - Clear screen
- `:quit` or `Ctrl+D` - Exit

### Terminal UI (TUI)

Advanced multi-panel interface with syntax highlighting:

```bash
# Start TUI
octofhir-fhirpath tui

# With resource
octofhir-fhirpath tui --input patient.json

# With theme
octofhir-fhirpath tui --theme light

# Check terminal compatibility
octofhir-fhirpath tui --check-terminal
```

Features:
- Multi-panel layout (input, output, diagnostics, variables, history)
- Real-time syntax highlighting and validation
- Interactive auto-completion
- Mouse and keyboard navigation
- Configurable themes (dark, light, high_contrast)

### Static Analysis

Validate and analyze expressions:

```bash
# Validate syntax
octofhir-fhirpath validate "Patient.name.family"

# Full analysis with type checking
octofhir-fhirpath analyze "Patient.name.family"

# Verbose analysis with suggestions
octofhir-fhirpath analyze "Patient.name.where(use='official')" --verbose
```

### Watch Mode

Auto-reload and re-evaluate on file changes:

```bash
octofhir-fhirpath evaluate "Patient.name" --input patient.json --watch
```

### Batch Processing

Process multiple files with glob patterns:

```bash
# Process all JSON files
octofhir-fhirpath evaluate "Patient.active" --batch "patients/*.json"

# Continue on errors
octofhir-fhirpath evaluate "Patient.name" \
  --batch "data/**/*.json" \
  --continue-on-error
```

### HTTP Server

Start REST API server for remote evaluation:

```bash
# Default port 8084
octofhir-fhirpath server

# Custom port
octofhir-fhirpath server --port 8080

# Development mode (CORS enabled)
octofhir-fhirpath server --cors-all

# Custom host
octofhir-fhirpath server --host 0.0.0.0 --port 8080
```

API endpoints:
- `GET /health` - Health check
- `GET /version` - Version info
- `POST /r4` - Evaluate with FHIR R4
- `POST /r4b` - Evaluate with FHIR R4B
- `POST /r5` - Evaluate with FHIR R5
- `POST /r6` - Evaluate with FHIR R6

Example request:
```bash
curl -X POST http://localhost:8084/r4 \
  -H "Content-Type: application/json" \
  -d '{
    "resourceType": "Parameters",
    "parameter": [
      {"name": "expression", "valueString": "Patient.name.family"},
      {"name": "resource", "resource": {"resourceType": "Patient", "name": [{"family": "Doe"}]}}
    ]
  }'
```

### Function Registry

Explore available FHIRPath functions and operators:

```bash
# List all functions
octofhir-fhirpath registry list functions

# List by category
octofhir-fhirpath registry list functions --category string

# Search functions
octofhir-fhirpath registry list functions --search "where"

# Show function details
octofhir-fhirpath registry show where

# List operators
octofhir-fhirpath registry list operators

# Show operator details
octofhir-fhirpath registry show "+" --target operator
```

### Configuration

Create a configuration file at `~/.fhirpathrc` or `.fhirpathrc`:

```toml
# Default FHIR version
fhir_version = "r4"

# Default output format
output_format = "pretty"

# Disable colors
no_color = false

# Quiet mode
quiet = false

# Favorite expressions
[[favorites]]
alias = "patient-name"
expression = "Patient.name.given.first() + ' ' + Patient.name.family"
description = "Get patient's full name"

[[favorites]]
alias = "obs-value"
expression = "Observation.value.ofType(Quantity).value"
description = "Extract observation quantity value"
```

Manage configuration:
```bash
# Show current config
octofhir-fhirpath config show

# Show config file path
octofhir-fhirpath config path

# Initialize new config
octofhir-fhirpath config init

# Edit config in default editor
octofhir-fhirpath config edit

# Add favorite expression
octofhir-fhirpath config add-favorite patient-active "Patient.active" \
  --description "Check if patient is active"
```

### Shell Completion

Generate shell completions:

```bash
# Bash
octofhir-fhirpath completions bash > /usr/local/share/bash-completion/completions/octofhir-fhirpath

# Zsh
octofhir-fhirpath completions zsh > /usr/local/share/zsh/site-functions/_octofhir-fhirpath

# Fish
octofhir-fhirpath completions fish > ~/.config/fish/completions/octofhir-fhirpath.fish

# PowerShell
octofhir-fhirpath completions powershell > octofhir-fhirpath.ps1
```

### Performance Profiling

Get detailed performance breakdowns:

```bash
# Basic profiling
octofhir-fhirpath evaluate "Patient.name" --input patient.json --profile

# With analysis
octofhir-fhirpath evaluate "Bundle.entry.resource.count()" \
  --input bundle.json \
  --profile \
  --analyze
```

## Output Formats

### Pretty (Default)

Beautiful Ariadne-style diagnostics with colors and source highlighting:

```bash
octofhir-fhirpath evaluate "Patient.invalid" --input patient.json
```

Output:
```
Error: FP0054
  ┌─ expression:1:9
  │
1 │ Patient.invalid
  │         ^^^^^^^ Unknown field 'invalid' on type 'Patient'
  │
  = help: Did you mean 'active'?
```

### JSON

Structured output for tooling integration:

```bash
octofhir-fhirpath evaluate "Patient.name" --input patient.json --output-format json
```

Output:
```json
{
  "success": true,
  "result": [{"family": "Doe", "given": ["John"]}],
  "expression": "Patient.name",
  "execution_time_ms": 2.5
}
```

### Raw

Plain text output:

```bash
octofhir-fhirpath evaluate "Patient.active" --input patient.json --output-format raw
```

Output:
```
true
```

## Common Use Cases

### Data Extraction

```bash
# Extract patient names
octofhir-fhirpath evaluate "Patient.name.given.first() + ' ' + Patient.name.family" \
  --input patient.json

# Extract observation values
octofhir-fhirpath evaluate "Observation.value.ofType(Quantity).value" \
  --input observation.json
```

### Data Validation

```bash
# Check if patient has name
octofhir-fhirpath evaluate "Patient.name.exists()" --input patient.json

# Validate required fields
octofhir-fhirpath evaluate "Patient.name.exists() and Patient.birthDate.exists()" \
  --input patient.json
```

### Batch Processing

```bash
# Validate all patient files
octofhir-fhirpath evaluate "Patient.name.exists()" \
  --batch "patients/*.json" \
  --output-format json

# Extract data from multiple resources
octofhir-fhirpath evaluate "Patient.active" \
  --batch "data/**/*.json" \
  --continue-on-error
```

### Integration with Other Tools

```bash
# Pipe to jq for further processing
octofhir-fhirpath evaluate "Patient.name" \
  --input patient.json \
  --output-format json | jq '.result[0].family'

# Use in scripts
#!/bin/bash
for file in patients/*.json; do
  echo "Processing $file"
  octofhir-fhirpath evaluate "Patient.active" --input "$file" --quiet
done

# NDJSON streaming
cat patients.ndjson | octofhir-fhirpath evaluate "Patient.name.family" --pipe
```

## Environment Variables

- `FHIRPATH_NO_COLOR` or `NO_COLOR` - Disable colored output
- `FHIRPATH_FHIR_VERSION` - Default FHIR version (r4, r4b, r5, r6)

## Error Codes

FHIRPath uses structured error codes (FP0001-FP9999) for diagnostics. Get help on specific errors:

```bash
octofhir-fhirpath docs FP0054
```

Common error codes:
- `FP0001` - Parse error
- `FP0054` - Unknown field on type
- `FP0055` - Type mismatch
- `FP0056` - Function not found
- `FP0057` - Invalid argument count

## Development

### Building

```bash
# Debug build
cargo build --package fhirpath-cli

# Release build
cargo build --package fhirpath-cli --release

# With all features
cargo build --package fhirpath-cli --all-features
```

### Testing

```bash
# Run tests
cargo test --package fhirpath-cli

# With coverage
just coverage

# Integration tests
cargo test --package fhirpath-cli --test '*'
```

### Using with Just

The project includes a comprehensive `justfile`:

```bash
# Show available commands
just

# Start REPL
just repl

# Start TUI
just tui

# Start server
just server

# Run specific test
just test-case patient-example

# Registry operations
just registry-functions
just registry-operators
just registry-show where
```

## Examples

See the [examples/](examples/) directory for more detailed usage examples:

- `basic_evaluation.rs` - Simple expression evaluation
- `batch_processing.rs` - Process multiple files
- `custom_formatter.rs` - Custom output formatting
- `programmatic_repl.rs` - Embed REPL in application
- `server_integration.rs` - Use server programmatically

## Architecture

```
fhirpath-cli/
├── src/
│   ├── main.rs              # Entry point
│   ├── lib.rs               # Public API
│   ├── cli/                 # CLI framework
│   │   ├── handlers/        # Command handlers
│   │   ├── output/          # Output formatters
│   │   ├── repl/            # REPL implementation
│   │   └── server/          # HTTP server
│   └── tui/                 # Terminal UI
├── tests/                   # Integration tests
└── examples/                # Usage examples
```

## Troubleshooting

### TUI Not Working

Check terminal compatibility:
```bash
octofhir-fhirpath tui --check-terminal
```

Minimum requirements:
- Terminal size: 80x24
- Color support: 256 colors recommended

### Performance Issues

Enable profiling to identify bottlenecks:
```bash
octofhir-fhirpath evaluate "complex.expression" \
  --input large-file.json \
  --profile \
  --verbose
```

### Server Not Starting

Check if port is already in use:
```bash
# Check port 8084
lsof -i :8084

# Use different port
octofhir-fhirpath server --port 9000
```

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

## License

Licensed under Apache License 2.0. See [LICENSE](../../LICENSE) for details.

## Links

- [Main Documentation](../../README.md)
- [FHIRPath Specification](http://hl7.org/fhirpath/)
- [FHIR Specification](https://hl7.org/fhir/)
- [API Documentation](https://docs.rs/octofhir-fhirpath)

## Support

- GitHub Issues: [Report bugs or request features](https://github.com/octofhir/fhirpath-rs/issues)
- GitHub Discussions: [Ask questions and discuss](https://github.com/octofhir/fhirpath-rs/discussions)

---

**Version:** 0.4.x  
**FHIR Versions Supported:** R4, R4B, R5, R6  
**Rust Version:** 1.75+
