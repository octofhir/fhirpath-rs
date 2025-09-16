# FHIRPath for Rust

A focused, spec-first FHIRPath implementation in Rust. No fluff. The goal is correctness, clarity, and predictable behavior when evaluating FHIRPath over JSON FHIR resources.

**Docs**: https://docs.rs/octofhir-fhirpath

**Spec coverage**: see TEST_COVERAGE.md in this repo for the current report.

**CLI reference**: CLI.md in this repo.

**License**: MIT OR Apache-2.0

**Rust**: 1.87+ (edition 2024)

**Spec target**: FHIRPath 3.0.0

## Core Principles

- Spec-first: implement the FHIRPath 3.0.0 spec faithfully and transparently.
- Deterministic: evaluation is pure and predictable; no hidden state.
- Safety: no unsafe code; thread-safe and async-ready.
- Separation of concerns: engine, model provider, and terminology are decoupled.
- Minimal API: make the simple path easy; advanced use explicit.
- Transparent diagnostics: clear errors and optional detailed metadata.
- Performance matters, but never at the expense of correctness.

## Install

- Library: add `octofhir-fhirpath = "0.4"` to your Cargo.toml.
- CLI from source:
  - `cargo run -p fhirpath-cli -- --help` (from this repo), or
  - `cargo install --git https://github.com/octofhir/fhirpath-rs --bin octofhir-fhirpath`

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

## CLI

- From this repo:
  - `cargo run -p fhirpath-cli -- evaluate "Patient.name.given" --input patient.json`
  - `cargo run -p fhirpath-cli -- repl --input patient.json`
  - `cargo run -p fhirpath-cli -- server --port 8080`
- Installed binary:
  - `octofhir-fhirpath evaluate "Patient.active" --input patient.json`
  - See CLI.md for options and output formats.

## Workspace

- `crates/octofhir-fhirpath`: library crate (published)
- `crates/fhirpath-cli`: CLI crate (not published)
- `crates/fhirpath-dev-tools`: dev/test/bench tools (not published)

## Architecture

- Parser
  - `parse`, `parse_ast`, `validate` with configurable `ParserConfig`/`ParsingMode`.
  - Produces an AST (`ast::ExpressionNode`) used by the evaluator.
- Evaluator
  - `FhirPathEngine` orchestrates a `CompositeEvaluator` composed of Core/Function/Operator/Collection/Lambda evaluators.
  - AST cache avoids re-parsing hot expressions; auto-detects root FHIR context ("name.given" → "Patient.name.given").
  - Two paths: `evaluate` (plain `Collection`) and `evaluate_with_metadata` (preserves type/path metadata).
- Values and Types
  - `FhirPathValue` covers primitives, precise temporal types, UCUM-backed `Quantity`, `Collection`, and JSON-backed complex values via `Arc<serde_json::Value>`.
  - Helpers in `core::value::utils` convert JSON to `FhirPathValue` and back when needed.
- Context and Variables
  - `EvaluationContext` carries the start/root collections, user variables, and built-ins.
  - Built-ins include `%context`, `%resource`, `%rootResource`, `%terminologies` placeholder, and shortcuts like `%sct`, `%loinc`, `%ucum`.
- Model and Terminology
  - `ModelProvider` is a trait for type/structure info; you can start with `EmptyModelProvider` and plug richer providers externally.
  - Terminology is opt-in and provided through the context; the core library does not make hidden network calls.

## Design Trade-offs

- Arc<JsonValue> for complex values
  - Pro: eliminates cloning of large Bundles/resources; cheap sharing across contexts/threads.
  - Con: you must treat JSON as immutable; deep mutation is intentionally not supported.
- No implicit IO in the library
  - Pro: deterministic, testable, and predictable evaluation.
  - Con: external setup is required for terminology/server-backed behaviors.
- Auto root-context transformation
  - Pro: writing `name.given` against a Patient JSON “just works”.
  - Con: if the input is not a FHIR resource or already fully qualified, we keep it as-is; be explicit when needed.
- AST cache
  - Pro: significant speed-up for repeated expressions.
  - Con: memory usage proportional to unique expressions; you control lifecycle by engine lifetime.
- JSON as interchange
  - Pro: ubiquitous, simple integration; matches real-world FHIR payloads.
  - Con: no compile-time schema guarantees; use a ModelProvider when you need type checks.
- Quantities and temporals
  - UCUM-backed quantities and calendar units are supported; conversions follow UCUM where applicable.
  - Temporal types keep precision; operations may be more strict than ad-hoc implementations.
- Metadata-aware evaluation
  - Pro: preserves type/path/index info for tooling and rich outputs.
  - Con: slightly higher overhead than plain evaluation; use plain `evaluate` when you only need values.

## What This Library Provides

- Parser for FHIRPath 3.0.0 with analysis helpers.
- Evaluator with AST caching, metrics, and metadata-aware evaluation.
- Rich value system: precise temporals, UCUM quantities, collections, and JSON-backed resources.
- Pluggable ModelProvider and Terminology integration (e.g., tx.fhir.org via CLI tooling).
- Built-in variables and context: `%context`, `%resource`, `%rootResource`, `%terminologies`, and standard code system shortcuts.

## Non‑Goals / Clarifications

- No hidden network calls in the core library; external services are opt-in via context/CLI.
- JSON is the interchange type (`serde_json::Value`); no alternative JSON backends.
- Compliance numbers change; refer to TEST_COVERAGE.md rather than this file.

## References

- API docs: https://docs.rs/octofhir-fhirpath
- CLI usage: CLI.md
- Spec compliance report: TEST_COVERAGE.md
- Benchmarks and notes: benchmark.md

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

## 🛠️ Architecture

Modern, modular Rust architecture:

- **9 specialized crates** for flexibility and maintainability
- **Async-first design** for scalable healthcare applications  
- **Zero-copy parsing** with arena allocation for performance
- **Comprehensive error handling** with helpful diagnostic messages
- **Thread-safe by design** with full `Send + Sync` support

## 🔗 Resources

- **[FHIRPath Specification](http://hl7.org/fhirpath/)** - Official specification
- **[FHIR R5](https://hl7.org/fhir/)** - FHIR standard documentation  
- **[HL7 FHIR](https://www.hl7.org/fhir/)** - Healthcare interoperability standards

## 📞 Support & Community

- 🐛 **[Issues](https://github.com/octofhir/fhirpath-rs/issues)** - Bug reports and feature requests
- 💬 **[Discussions](https://github.com/octofhir/fhirpath-rs/discussions)** - Questions and community
- 📧 **Email**: funyloony@gmail.com
- 💝 **Sponsor**: [Boosty](https://boosty.to/octoshikari) - Support development

## 📄 License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.

---

**Built with ❤️ for healthcare interoperability**