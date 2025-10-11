//! LSP performance benchmarks
//!
//! Benchmarks for various LSP operations to track performance improvements

use fhirpath_lsp::document::FhirPathDocument;
use fhirpath_lsp::features::completion::generate_completions;
use fhirpath_lsp::features::diagnostics::generate_diagnostics;
use fhirpath_lsp::features::semantic_tokens::generate_semantic_tokens;
use lsp_types::Position;
use octofhir_fhirpath::FhirPathEngine;
use std::sync::Arc;
use url::Url;

fn main() {
    divan::main();
}

/// Generate a sample FHIRPath document with N expressions
fn generate_document(num_lines: usize) -> FhirPathDocument {
    let mut expressions = Vec::new();

    for i in 0..num_lines {
        expressions.push(format!("Patient.name[{}].given.first()", i));
    }

    let text = expressions.join(";\n");
    let uri = Url::parse("file:///test.fhirpath").unwrap();

    FhirPathDocument::new(uri, text, 1)
}

/// Benchmark document parsing with varying sizes
#[divan::bench(args = [10, 50, 100, 500, 1000])]
fn document_parsing(bencher: divan::Bencher, size: usize) {
    bencher.bench_local(|| {
        let doc = generate_document(size);
        divan::black_box(doc);
    });
}

/// Benchmark completion generation
#[divan::bench]
fn completion_generation(bencher: divan::Bencher) {
    let doc = generate_document(10);
    let position = Position::new(0, 10);

    bencher.bench_local(|| {
        let result = generate_completions(&doc, position);
        divan::black_box(result);
    });
}

/// Benchmark semantic tokens generation
#[divan::bench(args = [10, 50, 100, 500, 1000])]
fn semantic_tokens_generation(bencher: divan::Bencher, size: usize) {
    let doc = generate_document(size);

    bencher.bench_local(|| {
        let result = generate_semantic_tokens(&doc);
        divan::black_box(result);
    });
}

/// Benchmark diagnostics generation (async)
#[divan::bench(args = [10, 50, 100])]
fn diagnostics_generation(bencher: divan::Bencher, size: usize) {
    let doc = generate_document(size);

    bencher
        .with_inputs(|| {
            // Setup: create engine with mock provider (fast)
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let model_provider = Arc::new(octofhir_fhirpath::mock::MockModelProvider::new());
            let registry = octofhir_fhirpath::create_function_registry();

            let engine = runtime.block_on(async {
                FhirPathEngine::new(
                    Arc::new(registry),
                    model_provider as Arc<dyn octofhir_fhirpath::ModelProvider + Send + Sync>,
                )
                .await
                .unwrap()
            });

            (runtime, engine)
        })
        .bench_values(|(runtime, engine)| {
            runtime.block_on(async {
                let diagnostics = generate_diagnostics(&doc, &engine).await;
                divan::black_box(diagnostics);
            });
        });
}

/// Benchmark incremental document updates
#[divan::bench(args = [10, 50, 100])]
fn incremental_updates(bencher: divan::Bencher, size: usize) {
    bencher
        .with_inputs(|| generate_document(size))
        .bench_values(|mut doc| {
            use lsp_types::{Range, TextDocumentContentChangeEvent};

            // Simulate typing a character
            let change = TextDocumentContentChangeEvent {
                range: Some(Range::new(Position::new(0, 0), Position::new(0, 0))),
                range_length: None,
                text: "x".to_string(),
            };

            doc.apply_change(change, 2);
            divan::black_box(doc);
        });
}

/// Benchmark position to offset conversion (common operation)
#[divan::bench]
fn position_to_offset(bencher: divan::Bencher) {
    let doc = generate_document(1000);

    bencher.bench_local(|| {
        let offset = doc.position_to_offset(Position::new(500, 10));
        divan::black_box(offset);
    });
}

/// Benchmark offset to position conversion
#[divan::bench]
fn offset_to_position(bencher: divan::Bencher) {
    let doc = generate_document(1000);
    let mid_offset = doc.text.len() / 2;

    bencher.bench_local(|| {
        let position = doc.offset_to_position(mid_offset);
        divan::black_box(position);
    });
}
