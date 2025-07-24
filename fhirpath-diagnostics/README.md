# fhirpath-diagnostics

Comprehensive diagnostic system for FHIRPath parsing and evaluation errors.

## Overview

This crate provides a rich diagnostic system that can produce both human-friendly error messages and machine-readable diagnostics suitable for IDE integration. It supports:

- Detailed error messages with source locations
- Suggestions for fixing common mistakes
- Multiple output formats (text, JSON, compact)
- LSP (Language Server Protocol) integration
- Colored terminal output

## Features

- `serde`: Enable serialization support
- `lsp`: Enable LSP integration
- `terminal`: Enable colored terminal output

## Usage

### Basic Diagnostics

```rust
use fhirpath_diagnostics::{DiagnosticBuilder, DiagnosticCode, Position, Span};

// Create a diagnostic using the builder
let diagnostic = DiagnosticBuilder::error(DiagnosticCode::UnknownFunction)
    .with_message("Unknown function 'whre'")
    .with_span(Span::new(Position::new(0, 10), Position::new(0, 14)))
    .with_source_text("Patient.whre(active = true)")
    .suggest("Did you mean 'where'?", Some("where".to_string()))
    .build();

// Format for display
println!("{}", diagnostic.to_text());
```

Output:
```
error: Unknown function 'whre' [E007]
 --> 1:11-15
   1 | Patient.whre(active = true)
     |          ^^^^

suggestions:
  - Did you mean 'where'? (replace with 'where')
```

### Convenience Builders

```rust
use fhirpath_diagnostics::DiagnosticBuilder;

// Common diagnostics have convenience methods
let diag = DiagnosticBuilder::unknown_function("foo")
    .with_location(0, 10, 0, 13)
    .build();

let diag = DiagnosticBuilder::type_mismatch("String", "Integer")
    .with_location(2, 5, 2, 10)
    .build();

let diag = DiagnosticBuilder::property_not_found("nam", "Patient")
    .suggest("Did you mean 'name'?", Some("name".to_string()))
    .build();
```

### Multiple Output Formats

```rust
use fhirpath_diagnostics::{DiagnosticFormatter, Format};

let formatter = DiagnosticFormatter::new(Format::Text)
    .with_code(true)
    .with_suggestions(true);

// Text format (human-readable)
println!("{}", formatter.format(&diagnostic));

// JSON format (for tooling)
let json_formatter = DiagnosticFormatter::new(Format::Json);
println!("{}", json_formatter.format(&diagnostic));

// Compact format (single line)
let compact_formatter = DiagnosticFormatter::new(Format::Compact);
println!("{}", compact_formatter.format(&diagnostic));
```

### LSP Integration

When the `lsp` feature is enabled:

```rust
use fhirpath_diagnostics::to_lsp_diagnostic;

let lsp_diagnostic = to_lsp_diagnostic(&diagnostic);
// Use with language server implementation
```

### Source Location Tracking

```rust
use fhirpath_diagnostics::{SourceLocation, Position, Span};

// Create from positions
let location = SourceLocation::new(
    Span::new(Position::new(5, 10), Position::new(5, 15))
);

// Create from byte offsets
let span = Span::from_offsets(source_text, 45, 50);

// Add context
let location = SourceLocation::complete(
    span,
    "Patient.name.given".to_string(),
    "/path/to/file.fhirpath".to_string()
);
```

### Related Information

```rust
let diagnostic = DiagnosticBuilder::error(DiagnosticCode::TypeMismatch {
    expected: "String".to_string(),
    actual: "Integer".to_string(),
})
.with_message("Type mismatch in function argument")
.with_location(2, 15, 2, 20)
.related(
    SourceLocation::new(Span::new(Position::new(1, 0), Position::new(1, 10))),
    "Function defined here"
)
.build();
```

## Diagnostic Codes

The crate defines standard diagnostic codes:

- **E001-E099**: Parsing errors
- **E100-E199**: Type errors  
- **E200-E299**: Semantic errors
- **E300-E399**: Runtime errors

Each diagnostic has a unique code that can be used for:
- Suppressing specific warnings
- Documentation references
- Automated error handling

## Integration with fhirpath-parser

The diagnostics crate is designed to work seamlessly with the FHIRPath parser:

```rust
use fhirpath_parser::parse_with_diagnostics;
use fhirpath_diagnostics::DiagnosticFormatter;

let (ast, diagnostics) = parse_with_diagnostics("Patient.nam");

if !diagnostics.is_empty() {
    let formatter = DiagnosticFormatter::default();
    for diagnostic in &diagnostics {
        eprintln!("{}", formatter.format(diagnostic));
    }
}
```

## License

This project is licensed under the Apache-2.0 license.