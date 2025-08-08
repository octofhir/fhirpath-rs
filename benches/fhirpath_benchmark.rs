//! FHIRPath Performance Benchmarks
//!
//! Comprehensive benchmark suite covering tokenizer, parser, and evaluator performance
//! across a wide range of expression complexity levels to demonstrate arena integration benefits.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use octofhir_fhirpath::engine::FhirPathEngine;
use octofhir_fhirpath::evaluator::{
    ContextInheritance, FunctionClosureOptimizer, SharedContextBuilder, SharedEvaluationContext,
};
use octofhir_fhirpath::model::{Collection, FhirPathValue, string_intern::StringInterner};
use octofhir_fhirpath::parser::{parse_expression_pratt, tokenizer::Tokenizer};
use octofhir_fhirpath::pipeline::{AsyncPool, FhirPathPools, PoolConfig, global_pools};
use octofhir_fhirpath::registry::{FunctionRegistry, OperatorRegistry};
use rustc_hash::FxHashMap;
use serde_json::Value;
use std::hint::black_box;
use std::sync::Arc;

/// Comprehensive test expressions ranging from very simple to very complex
/// to demonstrate arena integration performance benefits across complexity spectrum
const TEST_EXPRESSIONS: &[(&str, &str)] = &[
    // Very Simple - Basic literals and identifiers
    ("very_simple_literal", "42"),
    ("very_simple_identifier", "Patient"),
    ("very_simple_boolean", "true"),
    // Simple - Basic property access
    ("simple_property", "Patient.name"),
    ("simple_nested", "Patient.name.given"),
    ("simple_index", "Patient.name[0]"),
    // Medium - Single operations and functions
    ("medium_where", "Patient.name.where(use = 'official')"),
    ("medium_function", "Patient.name.first()"),
    ("medium_arithmetic", "Patient.age + 10"),
    ("medium_comparison", "Patient.age > 18"),
    // Complex - Multiple operations and nesting
    (
        "complex_chained",
        "Patient.name.where(use = 'official').given.first()",
    ),
    (
        "complex_filter",
        "Patient.telecom.where(system = 'phone' and use = 'home')",
    ),
    (
        "complex_nested_func",
        "Patient.name.where(family.exists()).given.select(substring(0, 1))",
    ),
    (
        "complex_arithmetic",
        "(Patient.age * 12) + Patient.birthDate.toString().length()",
    ),
    // Very Complex - Deep nesting, multiple filters, complex logic
    (
        "very_complex_multi_filter",
        "Patient.name.where(use = 'official' and family.exists()).given.where(length() > 2).select(upper())",
    ),
    (
        "very_complex_nested_logic",
        "Patient.telecom.where(system = 'phone').where(use = 'home' or use = 'work').value.select(substring(0, 3))",
    ),
    (
        "very_complex_deep_nesting",
        "Bundle.entry.resource.where($this is Patient).name.where(use = 'official').given.where(length() > 1).first().upper()",
    ),
    (
        "very_complex_lambda",
        "Patient.name.select(given.where(length() > 2).select($this + ' ' + %context.family.first()))",
    ),
    // Extremely Complex - Multiple lambdas, complex expressions
    (
        "extremely_complex_multi_lambda",
        "Bundle.entry.resource.where($this is Patient).select(name.where(use = 'official').select(given.where(length() > 1).select($this.upper() + ', ' + %context.family.first().lower())))",
    ),
    (
        "extremely_complex_nested_operations",
        "Patient.extension.where(url = 'http://example.com/race').extension.where(url = 'ombCategory').value.where($this is Coding).select(system + '|' + code + '|' + display.substring(0, 10))",
    ),
];

fn bench_tokenizer(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenizer");
    group.throughput(Throughput::Elements(1));

    for (complexity, expression) in TEST_EXPRESSIONS {
        group.bench_with_input(
            BenchmarkId::new("tokenize", complexity),
            expression,
            |b, expr| {
                b.iter(|| {
                    let mut tokenizer = Tokenizer::new(black_box(expr));
                    black_box(tokenizer.tokenize_all())
                })
            },
        );
    }

    group.finish();
}

fn bench_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser");
    group.throughput(Throughput::Elements(1));

    for (complexity, expression) in TEST_EXPRESSIONS {
        group.bench_with_input(
            BenchmarkId::new("parse", complexity),
            expression,
            |b, expr| b.iter(|| black_box(parse_expression_pratt(black_box(expr)))),
        );
    }

    group.finish();
}

fn bench_evaluator(c: &mut Criterion) {
    let mut group = c.benchmark_group("evaluator");
    group.throughput(Throughput::Elements(1));

    let input = Value::String("test".to_string());

    for (complexity, expression) in TEST_EXPRESSIONS {
        group.bench_with_input(
            BenchmarkId::new("evaluate", complexity),
            expression,
            |b, expr| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                b.iter(|| {
                    let mut engine = FhirPathEngine::new();
                    black_box(rt.block_on(engine.evaluate(black_box(expr), input.clone())))
                })
            },
        );
    }

    group.finish();
}

fn bench_throughput(c: &mut Criterion) {
    let expression = "Patient.name.where(use = 'official')";
    let input = Value::String("test".to_string());

    let mut group = c.benchmark_group("throughput");
    group.sample_size(1000);

    group.bench_function("tokenizer_throughput", |b| {
        b.iter(|| {
            let mut tokenizer = Tokenizer::new(black_box(expression));
            black_box(tokenizer.tokenize_all())
        })
    });

    group.bench_function("parser_throughput", |b| {
        b.iter(|| black_box(parse_expression_pratt(black_box(expression))))
    });

    group.bench_function("evaluator_throughput", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        b.iter(|| {
            let mut engine = FhirPathEngine::new();
            black_box(rt.block_on(engine.evaluate(black_box(expression), input.clone())))
        })
    });

    group.finish();
}

fn bench_string_interning_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_interning");
    group.sample_size(100);

    group.bench_function("string_interning_baseline", |b| {
        b.iter(|| {
            let interner = StringInterner::new();
            let mut interned_strings = Vec::new();

            // Create many strings to test interning performance
            for i in 0..100 {
                let s = format!("test_string_{i}");
                let interned = interner.intern(&s);
                interned_strings.push(interned);
            }

            // Test hit rate by re-interning the same strings
            for i in 0..100 {
                let s = format!("test_string_{i}");
                black_box(interner.intern(&s));
            }

            let stats = interner.stats();
            black_box(stats)
        })
    });

    group.bench_function("string_interning_hit_rate", |b| {
        b.iter(|| {
            let interner = StringInterner::new();

            // Create base set of strings
            let base_strings: Vec<String> = (0..100).map(|i| format!("base_string_{i}")).collect();

            // Intern base strings multiple times to test hit rate
            for _ in 0..10 {
                for s in &base_strings {
                    black_box(interner.intern(s));
                }
            }

            let stats = interner.stats();
            black_box(stats)
        })
    });

    group.finish();
}

/// Benchmark Arc-backed string interning in tokenizer
fn bench_tokenizer_interning(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenizer_interning");

    // Test with and without interning on common FHIR expressions
    let common_fhir_expressions = [
        "Patient.name.given.first()",
        "Patient.name.family.first()",
        "Bundle.entry.resource.name.given",
        "Patient.telecom.where(system = 'phone')",
        "Observation.value.where(code = 'vital-signs')",
        "Patient.identifier.where(system = 'official')",
        "Bundle.entry.resource.ofType(Patient).name",
        "Patient.address.line.first() + ', ' + Patient.address.city",
    ];

    // Benchmark regular tokenization
    group.bench_function("without_interning", |b| {
        b.iter(|| {
            for expr in &common_fhir_expressions {
                let mut tokenizer = Tokenizer::with_interning(black_box(expr), false);
                black_box(tokenizer.tokenize_all()).unwrap();
            }
        })
    });

    // Benchmark with interning enabled
    group.bench_function("with_interning", |b| {
        b.iter(|| {
            for expr in &common_fhir_expressions {
                let mut tokenizer = Tokenizer::with_interning(black_box(expr), true);
                black_box(tokenizer.tokenize_all()).unwrap();
            }
        })
    });

    // Benchmark interning statistics access
    group.bench_function("interner_stats", |b| {
        b.iter(|| {
            black_box(Tokenizer::interner_stats());
            black_box(Tokenizer::keyword_table_stats());
        })
    });

    group.finish();
}

/// Benchmark streaming tokenizer for large expressions
fn bench_tokenizer_streaming(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenizer_streaming");

    // Create large expressions of different sizes
    let small_expr = "Patient.name.given.first()".repeat(10);
    let medium_expr = "Patient.name.given.first()".repeat(100);
    let large_expr = "Patient.name.given.first()".repeat(1000);

    // Benchmark regular tokenization vs streaming for different sizes
    group.bench_with_input(
        BenchmarkId::new("regular_small", small_expr.len()),
        &small_expr,
        |b, expr| {
            b.iter(|| {
                let mut tokenizer = Tokenizer::new(black_box(expr));
                black_box(tokenizer.tokenize_all())
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("streaming_small", small_expr.len()),
        &small_expr,
        |b, expr| {
            b.iter(|| {
                let mut tokenizer = Tokenizer::with_streaming(black_box(expr), 256);
                let mut stream = tokenizer.tokenize_stream();
                black_box(stream.collect_all())
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("regular_medium", medium_expr.len()),
        &medium_expr,
        |b, expr| {
            b.iter(|| {
                let mut tokenizer = Tokenizer::new(black_box(expr));
                black_box(tokenizer.tokenize_all())
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("streaming_medium", medium_expr.len()),
        &medium_expr,
        |b, expr| {
            b.iter(|| {
                let mut tokenizer = Tokenizer::with_streaming(black_box(expr), 256);
                let mut stream = tokenizer.tokenize_stream();
                black_box(stream.collect_all())
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("regular_large", large_expr.len()),
        &large_expr,
        |b, expr| {
            b.iter(|| {
                let mut tokenizer = Tokenizer::new(black_box(expr));
                black_box(tokenizer.tokenize_all())
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("streaming_large", large_expr.len()),
        &large_expr,
        |b, expr| {
            b.iter(|| {
                let mut tokenizer = Tokenizer::with_streaming(black_box(expr), 256);
                let mut stream = tokenizer.tokenize_stream();
                black_box(stream.collect_all())
            })
        },
    );

    // Benchmark memory usage estimation
    group.bench_function("memory_estimation", |b| {
        b.iter(|| {
            let tokenizer = Tokenizer::new(black_box(&large_expr));
            black_box(tokenizer.estimate_memory_usage())
        })
    });

    group.finish();
}

/// Benchmark shared keyword lookup table performance
fn bench_tokenizer_keywords(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenizer_keywords");

    let keyword_expressions = [
        "true and false",
        "Patient where name exists",
        "Bundle select entry",
        "first() or last()",
        "count() > 0 and distinct().empty()",
        "Patient.name.where(use = 'official' and family.exists()).given.first()",
        "Bundle.entry.resource.ofType(Patient).name.family.first()",
        "true or false and not empty",
    ];

    // Benchmark keyword recognition performance
    group.bench_function("keyword_lookup", |b| {
        b.iter(|| {
            for expr in &keyword_expressions {
                let mut tokenizer = Tokenizer::new(black_box(expr));
                black_box(tokenizer.tokenize_all()).unwrap();
            }
        })
    });

    // Benchmark keyword table statistics
    group.bench_function("keyword_stats", |b| {
        b.iter(|| {
            black_box(Tokenizer::keyword_table_stats());
        })
    });

    // Benchmark is_keyword_str function
    let test_keywords = [
        "true", "false", "and", "or", "where", "select", "first", "last",
    ];
    group.bench_function("is_keyword_str", |b| {
        b.iter(|| {
            for keyword in &test_keywords {
                black_box(Tokenizer::is_keyword_str(black_box(keyword)));
            }
        })
    });

    group.finish();
}

/// Benchmark shared context creation performance
fn bench_shared_context_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("shared_context_creation");

    let functions = Arc::new(FunctionRegistry::new());
    let operators = Arc::new(OperatorRegistry::new());
    let input = FhirPathValue::Integer(42);

    // Benchmark basic context creation
    group.bench_function("basic_creation", |b| {
        b.iter(|| {
            black_box(SharedEvaluationContext::new(
                black_box(input.clone()),
                black_box(functions.clone()),
                black_box(operators.clone()),
            ))
        })
    });

    // Benchmark context creation with builder pattern
    group.bench_function("builder_creation", |b| {
        b.iter(|| {
            black_box(
                SharedContextBuilder::new()
                    .with_input(input.clone())
                    .with_functions(functions.clone())
                    .with_operators(operators.clone())
                    .build()
                    .unwrap(),
            )
        })
    });

    // Benchmark child context creation
    let parent_context =
        SharedEvaluationContext::new(input.clone(), functions.clone(), operators.clone());

    group.bench_function("child_with_input", |b| {
        b.iter(|| black_box(parent_context.with_input(black_box(input.clone()))))
    });

    group.bench_function("child_with_shared_variables", |b| {
        b.iter(|| black_box(parent_context.with_shared_variables(black_box(input.clone()))))
    });

    group.bench_function("child_with_inherited_variables", |b| {
        b.iter(|| black_box(parent_context.with_inherited_variables(black_box(input.clone()))))
    });

    group.finish();
}

/// Benchmark variable access performance  
fn bench_variable_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("variable_access");

    let functions = Arc::new(FunctionRegistry::new());
    let operators = Arc::new(OperatorRegistry::new());
    let input = FhirPathValue::Integer(42);

    // Create context with multiple variables
    let context = SharedEvaluationContext::new(input, functions, operators);
    for i in 0..100 {
        context.set_variable(format!("var{i}"), FhirPathValue::Integer(i));
    }

    // Benchmark variable access
    group.bench_function("single_variable_get", |b| {
        b.iter(|| black_box(context.get_variable(black_box("var50"))))
    });

    group.bench_function("single_variable_set", |b| {
        b.iter(|| {
            context.set_variable(
                black_box("temp_var".to_string()),
                black_box(FhirPathValue::Boolean(true)),
            )
        })
    });

    // Benchmark batch operations
    let batch_vars: FxHashMap<String, FhirPathValue> = (0..10)
        .map(|i| (format!("batch_var{i}"), FhirPathValue::Integer(i)))
        .collect();

    group.bench_function("batch_variable_set", |b| {
        b.iter(|| context.set_variables_batch(black_box(batch_vars.clone())))
    });

    // Benchmark memory stats
    group.bench_function("memory_stats", |b| {
        b.iter(|| black_box(context.memory_stats()))
    });

    group.bench_function("variable_sharing_info", |b| {
        b.iter(|| black_box(context.variable_sharing_info()))
    });

    group.finish();
}

/// Benchmark context inheritance performance
fn bench_context_inheritance(c: &mut Criterion) {
    let mut group = c.benchmark_group("context_inheritance");

    let functions = Arc::new(FunctionRegistry::new());
    let operators = Arc::new(OperatorRegistry::new());

    // Create parent contexts
    let parent_contexts: Vec<Arc<SharedEvaluationContext>> = (0..10)
        .map(|i| {
            let context = Arc::new(SharedEvaluationContext::new(
                FhirPathValue::Integer(i),
                functions.clone(),
                operators.clone(),
            ));
            for j in 0..10 {
                context.set_variable(format!("parent{i}_var{j}"), FhirPathValue::Integer(j));
            }
            context
        })
        .collect();

    // Benchmark inheritance chain creation
    group.bench_function("inheritance_chain_creation", |b| {
        b.iter(|| {
            black_box(ContextInheritance::compose(
                black_box(parent_contexts.clone()),
                10,
            ))
        })
    });

    let inheritance = ContextInheritance::compose(parent_contexts.clone(), 10);

    // Benchmark variable lookup in inheritance chain
    group.bench_function("inherited_variable_lookup", |b| {
        b.iter(|| black_box(inheritance.get_variable(black_box("parent5_var5"))))
    });

    group.bench_function("inherited_variable_shadowing_lookup", |b| {
        b.iter(|| black_box(inheritance.get_variable_with_shadowing(black_box("parent5_var5"))))
    });

    group.bench_function("collect_all_inherited_variables", |b| {
        b.iter(|| black_box(inheritance.collect_all_variables()))
    });

    group.finish();
}

/// Benchmark function closure optimizer performance
fn bench_function_closure_optimizer(c: &mut Criterion) {
    let mut group = c.benchmark_group("function_closure_optimizer");

    // Create optimizer with common patterns
    let optimizer = FunctionClosureOptimizer::with_common_patterns(100);

    let test_input = FhirPathValue::Collection(Collection::from_vec(vec![
        FhirPathValue::String("test1".to_string().into()),
        FhirPathValue::String("test2".to_string().into()),
    ]));

    // Benchmark closure execution
    group.bench_function("execute_cached_closure", |b| {
        b.iter(|| black_box(optimizer.execute_closure(black_box("count"), black_box(&test_input))))
    });

    // Benchmark batch execution
    let patterns = ["count", "first", "last", "is_empty", "not_empty"];
    group.bench_function("execute_closures_batch", |b| {
        b.iter(|| {
            black_box(
                optimizer.execute_closures_batch(black_box(&patterns), black_box(&test_input)),
            )
        })
    });

    // Benchmark cache management
    group.bench_function("cache_new_closure", |b| {
        b.iter(|| optimizer.cache_closure("test_pattern".to_string(), |input| input.clone()))
    });

    group.bench_function("optimizer_stats", |b| {
        b.iter(|| black_box(optimizer.stats()))
    });

    group.finish();
}

/// Benchmark context vs evaluationcontext performance comparison
fn bench_context_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("context_comparison");

    let functions = Arc::new(FunctionRegistry::new());
    let operators = Arc::new(OperatorRegistry::new());
    let input = FhirPathValue::Integer(42);

    // Setup contexts
    let shared_context =
        SharedEvaluationContext::new(input.clone(), functions.clone(), operators.clone());
    let evaluation_context = shared_context.to_evaluation_context();

    // Compare variable operations
    group.bench_function("shared_context_variable_set", |b| {
        b.iter(|| {
            shared_context.set_variable(
                black_box("test".to_string()),
                black_box(FhirPathValue::Boolean(true)),
            )
        })
    });

    group.bench_function("evaluation_context_variable_set", |b| {
        let mut ctx = evaluation_context.clone();
        b.iter(|| {
            ctx.set_variable(
                black_box("test".to_string()),
                black_box(FhirPathValue::Boolean(true)),
            )
        })
    });

    shared_context.set_variable("test".to_string(), FhirPathValue::Boolean(true));
    let mut eval_ctx = evaluation_context.clone();
    eval_ctx.set_variable("test".to_string(), FhirPathValue::Boolean(true));

    group.bench_function("shared_context_variable_get", |b| {
        b.iter(|| black_box(shared_context.get_variable(black_box("test"))))
    });

    group.bench_function("evaluation_context_variable_get", |b| {
        b.iter(|| black_box(eval_ctx.get_variable(black_box("test"))))
    });

    // Compare child context creation
    group.bench_function("shared_context_child_creation", |b| {
        b.iter(|| {
            black_box(
                shared_context
                    .with_input(black_box(FhirPathValue::String("child".to_string().into()))),
            )
        })
    });

    group.bench_function("evaluation_context_child_creation", |b| {
        b.iter(|| {
            black_box(
                eval_ctx.with_input(black_box(FhirPathValue::String("child".to_string().into()))),
            )
        })
    });

    group.finish();
}

/// Benchmark async memory pool performance and effectiveness
fn bench_memory_pool_performance(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("memory_pool");
    group.sample_size(100);

    // Test pool vs direct allocation for Vec<FhirPathValue>
    group.bench_function("pool_vs_direct_allocation", |b| {
        b.iter(|| {
            rt.block_on(async {
                let pool: AsyncPool<Vec<FhirPathValue>> = AsyncPool::new();

                // Pool-based allocation
                for _ in 0..100 {
                    let pooled_vec = pool.borrow().await;
                    black_box(&*pooled_vec);
                } // Objects automatically returned to pool

                // Direct allocation (baseline)
                for _ in 0..100 {
                    let direct_vec: Vec<FhirPathValue> = Vec::new();
                    black_box(direct_vec);
                }
            })
        })
    });

    // Test pool warming effectiveness
    group.bench_function("pool_warming_effectiveness", |b| {
        b.iter(|| {
            rt.block_on(async {
                let pool: AsyncPool<String> = AsyncPool::new();

                // Warm the pool
                pool.warm(50).await;

                // Now allocate - should mostly be hits
                for _ in 0..50 {
                    let pooled_string = pool.borrow().await;
                    black_box(&*pooled_string);
                }

                let stats = pool.stats().await;
                black_box(stats);
            })
        })
    });

    // Test async contention with multiple concurrent tasks
    group.bench_function("async_contention", |b| {
        b.iter(|| {
            rt.block_on(async {
                let pool = Arc::new(AsyncPool::<Vec<i32>>::new());
                pool.warm(20).await;

                let mut handles = Vec::new();

                // Spawn 10 concurrent tasks all accessing the same pool
                for _ in 0..10 {
                    let pool_clone = Arc::clone(&pool);
                    let handle = tokio::spawn(async move {
                        for _ in 0..10 {
                            let obj = pool_clone.borrow().await;
                            black_box(&*obj);
                            // Object returns to pool when dropped
                        }
                    });
                    handles.push(handle);
                }

                // Wait for all tasks to complete
                for handle in handles {
                    handle.await.unwrap();
                }

                let final_stats = pool.stats().await;
                black_box(final_stats);
            })
        })
    });

    // Test FhirPathPools comprehensive usage
    group.bench_function("fhirpath_pools_comprehensive", |b| {
        b.iter(|| {
            rt.block_on(async {
                let pools = FhirPathPools::new();

                // Warm all pools
                pools.warm_all().await;

                // Use values pool
                for _ in 0..20 {
                    let values = pools.values.borrow().await;
                    black_box(&*values);
                }

                // Use expressions pool
                for _ in 0..10 {
                    let expressions = pools.expressions.borrow().await;
                    black_box(&*expressions);
                }

                // Use strings pool
                for _ in 0..30 {
                    let strings = pools.strings.borrow().await;
                    black_box(&*strings);
                }

                // Get comprehensive stats
                let stats = pools.comprehensive_stats().await;
                black_box(stats);
            })
        })
    });

    // Test global pools access
    group.bench_function("global_pools_access", |b| {
        b.iter(|| {
            rt.block_on(async {
                let pools = global_pools();

                // Quick access to global pools
                for _ in 0..50 {
                    let value = pools.values.borrow().await;
                    black_box(&*value);
                }
            })
        })
    });

    // Test pool auto-adjustment under load
    group.bench_function("pool_auto_adjustment", |b| {
        b.iter(|| {
            rt.block_on(async {
                let pool: AsyncPool<Vec<u8>> = AsyncPool::with_config(PoolConfig {
                    initial_capacity: 10,
                    max_capacity: 100,
                    auto_adjust: true,
                    warm_threshold: 0.8,
                    cleanup_interval_secs: 1,
                });

                // Create high load to trigger auto-adjustment
                for _ in 0..200 {
                    let obj = pool.borrow().await;
                    black_box(&*obj);
                }

                // Trigger auto-adjustment
                pool.auto_adjust().await;

                let stats = pool.stats().await;
                black_box(stats);
            })
        })
    });

    group.finish();
}

criterion_group!(
    fhirpath_benchmarks,
    bench_tokenizer,
    bench_parser,
    bench_evaluator,
    bench_throughput,
    bench_string_interning_performance,
    bench_tokenizer_interning,
    bench_tokenizer_streaming,
    bench_tokenizer_keywords,
    bench_shared_context_creation,
    bench_variable_access,
    bench_context_inheritance,
    bench_function_closure_optimizer,
    bench_context_comparison,
    bench_memory_pool_performance
);

criterion_main!(fhirpath_benchmarks);
