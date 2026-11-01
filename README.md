# FHIRPath for Rust

A focused, spec-first FHIRPath implementation in Rust. No fluff. The goal is correctness, clarity, and predictable behavior when evaluating FHIRPath over JSON FHIR resources.

## Supported Versions

| | Version |
| - | - |
| **FHIRPath** | [3.0.0 (STU3, Release 3)](https://hl7.org/fhirpath/STU3/), normative and STU parts, plus the additions in the [3.1 continuous build](https://build.fhir.org/ig/HL7/FHIRPath/) (snapshot of 2026-10-25 in [`spec/`](spec/)); checked against the official fhir-test-cases R5 and R6 FHIRPath suites |
| **FHIR models** | R4, R4B and R5 via [octofhir-fhirschema](https://crates.io/crates/octofhir-fhirschema) |
| **FHIR extensions** | FHIR-specific functions (`resolve()`, `extension()`, `memberOf()`, `conformsTo()`, ...), `%terminologies`, `%factory` and `%server` per the FHIR API Incubator |
| **SQL on FHIR** | `getResourceKey()` / `getReferenceKey()` behind the `sql-on-fhir` feature |
| **Rust** | 1.98 or newer (MSRV, `rust-version` in `Cargo.toml`), edition 2024 |

## 🔌 Pluggable Architecture

**This library is highly pluggable.** In production, you can use our ModelProvider and other ecosystem components, or create your own custom implementations. The architecture supports:

- Custom ModelProvider implementations for FHIR structure definitions
- Pluggable terminology providers (tx.fhir.org, local, custom)
- User-defined function registries and extensions
- Custom evaluation contexts and variables

**Production-ready CLI**: Our CLI already integrates all ecosystem providers and can be used on a daily basis for FHIRPath evaluation, REPL sessions, and server mode.

**Docs**: [API Documentation](https://docs.rs/octofhir-fhirpath) | **Spec coverage**: [TEST_COVERAGE.md](TEST_COVERAGE.md) | **Benchmarks**: [benchmark.md](benchmark.md) | **CLI**: [CLI.md](CLI.md) | **License**: MIT OR Apache-2.0

## Core Principles

- **Spec-first**: FHIRPath 3.0.0 (with 3.1 CI additions) implemented against the spec text and the official tests
- **Deterministic**: pure, predictable evaluation with no hidden state
- **Safe & Fast**: no unsafe code, thread-safe, async-ready
- **Pluggable**: decoupled engine, model provider, and terminology
- **Clear diagnostics**: transparent errors and detailed metadata
- **Zero warnings**: passes all clippy lints and maintains clean code standards

## Installation

### Library

Add to your `Cargo.toml`:

```toml
[dependencies]
octofhir-fhirpath = "0.4"
```

### CLI

```bash
cargo install --git https://github.com/octofhir/fhirpath-rs --bin octofhir-fhirpath
```

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

See [CLI.md](CLI.md) for full options and output formats.

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

## Code Quality

This codebase maintains the highest standards of Rust code quality:

- **Zero clippy warnings**: All 15 clippy warning categories addressed and resolved
- **100% test coverage**: 359+ passing tests across all components
- **Clean architecture**: No dead code, unused imports, or unnecessary complexity
- **Performance optimized**: Manual optimizations replaced with idiomatic Rust patterns
- **Memory safe**: No unsafe code blocks anywhere in the codebase
- **Modern Rust**: Leverages latest Rust idioms and best practices

Run `just qa` to verify all quality checks pass.

## Spec Compliance

The engine tracks FHIRPath 3.x (STU3, with the 3.1 CI build in [`spec/`](spec/)).

- **Curated gate**: `just test-coverage` runs 1291 tests (the R5 suite plus FHIRPath 3
  features) and every one must pass. See [TEST_COVERAGE.md](TEST_COVERAGE.md).
- **Official suites**: the current [fhir-test-cases](https://github.com/FHIR/fhir-test-cases)
  R5 and R6 suites live in [`test-cases/official`](test-cases/official/) and run with
  `just test-official-r5` / `just test-official-r6` (report only). Of the tests that run,
  98.7% (R5) and 98.8% (R6) pass; the Java RI `documentFunctions` group is skipped.

FHIRPath 3.x features include instance selectors (`Coding { system: 'x', code: 'y' }`,
`{ : }`), `sort()` with `asc`/`desc`, `pathname()`, the Long type with `toLong()`,
the date/time component functions, `duration()`/`difference()`, `toDate()`/`toDateTime()`
with format codes, regex flags, and quantities per the 3.1 unit rules: UCUM arithmetic
(`3 'm' + 3 'cm'` is `303 'cm'`, `12 'cm' * 3 'cm'` is `36 'cm2'`), `toQuantity(unit)`,
calendar/UCUM time units in equality and equivalence (`1 year = 12 months`), and
`%rootResource`.
The `%factory` API follows the FHIR API Incubator, and the SQL on FHIR functions
`getResourceKey()` / `getReferenceKey()` are available behind the `sql-on-fhir` feature.

Compile-time `is`-guard checks (§1.11.1) are available through
`parse_with_type_guard_analysis`.

Known gaps: the precision of decimals written with an exponent in JSON input (`9.5e2`) for
`lowBoundary`/`highBoundary`. Where the official tests and the specification text disagree,
the engine follows the specification: `distinct()` of incomparable dates,
calendar-duration comparisons (`1 year > 2 months` is `true`, since 1 year = 12 months),
and the result type of `@2014.highBoundary(6)` (a Date). `as()` on a multi-item collection
is applied per item, because FHIR's own invariants rely on it.

## Resources & Support

- 📚 **[API Docs](https://docs.rs/octofhir-fhirpath)** - Complete API reference
- 🐛 **[Issues](https://github.com/octofhir/fhirpath-rs/issues)** - Bug reports and features
- 💬 **[Discussions](https://github.com/octofhir/fhirpath-rs/discussions)** - Community support
- 🌟 **[FHIRPath Spec](http://hl7.org/fhirpath/)** - Official specification
- 💝 **[Sponsor](https://boosty.to/octoshikari)** - Support development

Licensed under **MIT OR Apache-2.0**

---

**Built with ❤️ by the OctoFHIR team** 🦀