# FHIRPath for Rust

A focused, spec-first FHIRPath implementation in Rust. No fluff. The goal is correctness, clarity, and predictable behavior when evaluating FHIRPath over JSON FHIR resources.

## 🔌 Pluggable Architecture

**This library is highly pluggable.** In production, you can use our ModelProvider and other ecosystem components, or create your own custom implementations. The architecture supports:

- Custom ModelProvider implementations for FHIR structure definitions
- Pluggable terminology providers (tx.fhir.org, local, custom)
- User-defined function registries and extensions
- Custom evaluation contexts and variables

**Production-ready CLI**: Our CLI already integrates all ecosystem providers and can be used on a daily basis for FHIRPath evaluation, REPL sessions, and server mode.

**Docs**: https://docs.rs/octofhir-fhirpath | **Spec coverage**: TEST_COVERAGE.md | **CLI**: CLI.md | **License**: MIT OR Apache-2.0

## Core Principles

- **Spec-first**: FHIRPath 3.0.0 spec faithfully implemented
- **Deterministic**: pure, predictable evaluation with no hidden state
- **Safe & Fast**: no unsafe code, thread-safe, async-ready
- **Pluggable**: decoupled engine, model provider, and terminology
- **Clear diagnostics**: transparent errors and detailed metadata

## Install

**Library**: `octofhir-fhirpath = "0.4"` in Cargo.toml
**CLI**: `cargo install --git https://github.com/octofhir/fhirpath-rs --bin octofhir-fhirpath`

## Quick Start

### Minimal evaluation (convenience API)

```rust
use octofhir_fhirpath::evaluate;
use octofhir_fhirpath::core::value::utils::json_to_fhirpath_value;
use serde_json::json;

#[tokio::main]
async fn main() -> octofhir_fhirpath::Result<()> {
    let patient = json!({
        "resourceType": "Patient",
        "name": [{"family": "Smith", "given": ["Alice", "A."]}],
        "active": true
    });

    // Convert JSON to FHIRPath value and evaluate
    let ctx = json_to_fhirpath_value(patient);
    let out = evaluate("Patient.name.given", &ctx).await?;
    println!("{:?}", out);
    Ok(())
}
```

### Full control (engine + context)

```rust
use octofhir_fhirpath::{FhirPathEngine, create_standard_registry};
use octofhir_fhirpath::evaluator::EvaluationContext;
use octofhir_fhirpath::core::value::utils::json_to_fhirpath_value;
use octofhir_fhir_model::EmptyModelProvider;
use serde_json::json;
use std::sync::Arc;

#[tokio::main]
async fn main() -> octofhir_fhirpath::Result<()> {
    // Engine is explicit about function registry and model provider
    let registry = Arc::new(create_standard_registry().await);
    let provider = Arc::new(EmptyModelProvider);
    let mut engine = FhirPathEngine::new(registry, provider).await?;

    // Context can be any FHIR JSON; engine auto-detects root resourceType
    let ctx_val = json_to_fhirpath_value(json!({
        "resourceType": "Patient",
        "name": [{"given": ["Alice"], "family": "Smith"}]
    }));
    let ctx = EvaluationContext::from_value(ctx_val);

    // "name.given" is automatically treated as "Patient.name.given"
    let result = engine.evaluate("name.given", &ctx).await?;
    println!("{:?}", result.value);
    Ok(())
}
```

## CLI Usage

```bash
octofhir-fhirpath evaluate "Patient.name.given" --input patient.json
octofhir-fhirpath repl --input patient.json
octofhir-fhirpath server --port 8080
```

See CLI.md for full options and output formats.

## Architecture

- **Parser**: Configurable AST generation with validation
- **Evaluator**: `FhirPathEngine` with AST cache and auto-context detection
- **Values**: Rich type system with UCUM quantities, temporal types, JSON-backed resources
- **Context**: Built-in variables (`%context`, `%resource`, `%terminologies`, `%sct`, `%loinc`, etc.)
- **Pluggable**: ModelProvider trait for FHIR structure definitions, opt-in terminology

## Key Features

- **Zero-copy evaluation**: `Arc<JsonValue>` for efficient resource sharing
- **Deterministic**: No implicit IO, all external services are explicit
- **Smart context**: Auto-detects FHIR root context (`name.given` → `Patient.name.given`)
- **Performance**: AST cache for repeated expressions
- **JSON-first**: Direct integration with real-world FHIR payloads
- **Precision**: UCUM quantities and precise temporal operations
- **Metadata-aware**: Optional type/path/index preservation for tooling

## Spec Compliance

As of 2025-09-23, this implementation passes 100% of the official FHIRPath test suite (114 suites, 1118 tests). See TEST_COVERAGE.md for the full report.

✅ **Fully Supported (100%)**
- Path navigation and filtering
- Collection operations (`where`, `select`, `first`, `last`, `count`, etc.)
- Mathematical operations and arithmetic
- String manipulation functions
- Boolean logic and comparisons
- Date/time operations
- Type operations (`is`, `as`, `ofType`)

🟡 **Well Supported (85%+)**
- FHIR-specific functions (`resolve`, `extension`, `children`)
- Advanced filtering with environment variables
- Lambda expressions and complex iterations
- Aggregate functions and advanced collections

See **[TEST_COVERAGE.md](TEST_COVERAGE.md)** for detailed compliance status.

## Resources & Support

- 📚 **[API Docs](https://docs.rs/octofhir-fhirpath)** - Complete API reference
- 🐛 **[Issues](https://github.com/octofhir/fhirpath-rs/issues)** - Bug reports and features
- 💬 **[Discussions](https://github.com/octofhir/fhirpath-rs/discussions)** - Community support
- 🌟 **[FHIRPath Spec](http://hl7.org/fhirpath/)** - Official specification
- 💝 **[Sponsor](https://boosty.to/octoshikari)** - Support development

Licensed under **MIT OR Apache-2.0**

---

**Built with ❤️ by the OctoFHIR team** 🦀