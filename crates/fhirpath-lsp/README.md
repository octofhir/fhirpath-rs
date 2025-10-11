# FHIRPath Language Server Protocol (LSP)

Language Server Protocol implementation for FHIRPath expressions.

## Features

- Real-time diagnostics
- Semantic token highlighting
- Context-aware completion
- Hover documentation
- Inlay hints
- Code actions
- Go to definition
- Document symbols

## Installation

Download the binary for your platform from [GitHub Releases](https://github.com/octofhir/fhirpath-rs/releases).

## Usage

```bash
fhirpath-lsp
```

The server communicates via stdio using the LSP protocol.

## Configuration

Create `.fhirpath-lsp.toml` in your workspace root. See [LSP_IMPLEMENTATION_PLAN.md](../../LSP_IMPLEMENTATION_PLAN.md) for details.

## Development

```bash
# Build
cargo build -p fhirpath-lsp

# Test
cargo test -p fhirpath-lsp

# Run
cargo run -p fhirpath-lsp
```

## License

MIT OR Apache-2.0
