//! Performance benchmarks for FHIRPath analyzer using divan

use divan::Bencher;
use octofhir_fhirpath_analyzer::{AnalysisSettings, AnalyzerConfig, FhirPathAnalyzer};
use octofhir_fhirpath_model::mock_provider::MockModelProvider;
use octofhir_fhirpath_registry::create_standard_registry;
use std::sync::Arc;

fn main() {
    divan::main();
}

/// Test expressions categorized by complexity for analyzer benchmarks
#[derive(Debug, Clone)]
pub struct AnalyzerBenchmarkExpressions {
    pub simple: Vec<&'static str>,
    pub medium: Vec<&'static str>,
    pub complex: Vec<&'static str>,
}

impl Default for AnalyzerBenchmarkExpressions {
    fn default() -> Self {
        Self {
            simple: vec!["'hello world'", "42", "true", "Patient", "Patient.name"],
            medium: vec![
                "Patient.name.given",
                "count()",
                "exists()",
                "Patient.active",
                "Patient.name.family",
            ],
            complex: vec![
                "Patient.name.where(use = 'official').given.first()",
                "Bundle.entry.resource.ofType(Patient)",
                "Patient.children()",
                "Patient.telecom.where(system = 'phone' and use = 'home').value",
                "(Patient.name | Patient.contact.name).given",
            ],
        }
    }
}

/// Benchmark basic analysis operations
#[divan::bench(args = AnalyzerBenchmarkExpressions::default().simple)]
pub fn bench_analyze_simple(bencher: Bencher, expression: &str) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let provider = Arc::new(MockModelProvider::new());
    let analyzer = FhirPathAnalyzer::new(provider);

    bencher.bench_local(|| {
        rt.block_on(async {
            let _ = analyzer.analyze(expression).await.unwrap();
        })
    });
}

#[divan::bench(args = AnalyzerBenchmarkExpressions::default().medium)]
pub fn bench_analyze_medium(bencher: Bencher, expression: &str) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let provider = Arc::new(MockModelProvider::new());
    let analyzer = FhirPathAnalyzer::new(provider);

    bencher.bench_local(|| {
        rt.block_on(async {
            let _ = analyzer.analyze(expression).await.unwrap();
        })
    });
}

#[divan::bench(args = AnalyzerBenchmarkExpressions::default().complex)]
pub fn bench_analyze_complex(bencher: Bencher, expression: &str) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let provider = Arc::new(MockModelProvider::new());
    let analyzer = FhirPathAnalyzer::new(provider);

    bencher.bench_local(|| {
        rt.block_on(async {
            let _ = analyzer.analyze(expression).await.unwrap();
        })
    });
}

/// Benchmark analysis with function registry
#[divan::bench(args = AnalyzerBenchmarkExpressions::default().medium)]
pub fn bench_analyze_with_registry(bencher: Bencher, expression: &str) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Setup analyzer with function registry (expensive setup)
    let (analyzer, _registry) = rt.block_on(async {
        let provider = Arc::new(MockModelProvider::new());
        let registry = Arc::new(create_standard_registry().await.unwrap());
        let analyzer = FhirPathAnalyzer::with_function_registry(provider, registry.clone());
        (analyzer, registry)
    });

    bencher.bench_local(|| {
        rt.block_on(async {
            let _ = analyzer.analyze(expression).await.unwrap();
        })
    });
}

/// Benchmark children() function analysis specifically
#[divan::bench]
pub fn bench_children_function_analysis(bencher: Bencher) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (analyzer, _registry) = rt.block_on(async {
        let provider = Arc::new(MockModelProvider::new());
        let registry = Arc::new(create_standard_registry().await.unwrap());
        let analyzer = FhirPathAnalyzer::with_function_registry(provider, registry.clone());
        (analyzer, registry)
    });

    bencher.bench_local(|| {
        rt.block_on(async {
            let _ = analyzer.analyze("Patient.children()").await.unwrap();
        })
    });
}

/// Benchmark cache performance
#[divan::bench]
pub fn bench_analyze_cached(bencher: Bencher) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let provider = Arc::new(MockModelProvider::new());
    let analyzer = FhirPathAnalyzer::new(provider);
    let expression = "Patient.name.family";

    // Pre-warm the cache
    rt.block_on(async {
        let _ = analyzer.analyze(expression).await.unwrap();
    });

    bencher.bench_local(|| {
        rt.block_on(async {
            let _ = analyzer.analyze(expression).await.unwrap();
        })
    });
}

/// Benchmark different configuration impacts
#[divan::bench]
pub fn bench_analyze_minimal_config(bencher: Bencher) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let provider = Arc::new(MockModelProvider::new());

    let minimal_config = AnalyzerConfig {
        settings: AnalysisSettings {
            enable_type_inference: false,
            enable_function_validation: false,
            enable_union_analysis: false,
            max_analysis_depth: 10,
        },
        cache_size: 100,
        enable_profiling: false,
    };
    let analyzer = FhirPathAnalyzer::with_config(provider, minimal_config);

    bencher.bench_local(|| {
        rt.block_on(async {
            let _ = analyzer.analyze("Patient.name.given").await.unwrap();
        })
    });
}

#[divan::bench]
pub fn bench_analyze_full_config(bencher: Bencher) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let provider = Arc::new(MockModelProvider::new());

    let full_config = AnalyzerConfig {
        settings: AnalysisSettings {
            enable_type_inference: true,
            enable_function_validation: true,
            enable_union_analysis: true,
            max_analysis_depth: 100,
        },
        cache_size: 1000,
        enable_profiling: false,
    };
    let analyzer = FhirPathAnalyzer::with_config(provider, full_config);

    bencher.bench_local(|| {
        rt.block_on(async {
            let _ = analyzer.analyze("Patient.name.given").await.unwrap();
        })
    });
}

/// Benchmark memory-efficient analysis
#[divan::bench(args = [10, 100, 1000])]
pub fn bench_memory_efficient_analysis(bencher: Bencher, iterations: usize) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let provider = Arc::new(MockModelProvider::new());
    let analyzer = FhirPathAnalyzer::new(provider);

    let expressions = vec!["'literal'", "42", "Patient", "Patient.name", "count()"];

    bencher.bench_local(|| {
        rt.block_on(async {
            for _ in 0..iterations {
                for expr in &expressions {
                    let _ = analyzer.analyze(expr).await.unwrap();
                }
            }
        })
    });
}
