use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fhirpath_core::evaluator::{
    evaluate_expression, evaluate_expression_optimized, evaluate_expression_with_visitor,
    LoggingVisitor, NoopVisitor,
};
use fhirpath_core::lexer::tokenize;
use fhirpath_core::parser::parse;
use serde_json::json;

fn bench_lexer(c: &mut Criterion) {
    let mut group = c.benchmark_group("Lexer");

    // Simple expression
    group.bench_function("simple_expression", |b| {
        b.iter(|| {
            let expr = "Patient.name.given";
            tokenize(black_box(expr)).unwrap()
        })
    });

    // Complex expression
    group.bench_function("complex_expression", |b| {
        b.iter(|| {
            let expr = "Patient.name[0].given[0] = 'John' and Patient.gender = 'male'";
            tokenize(black_box(expr)).unwrap()
        })
    });

    // Expression with functions
    group.bench_function("expression_with_functions", |b| {
        b.iter(|| {
            let expr = "Patient.name.where(given.startsWith('J')).count() > 0";
            tokenize(black_box(expr)).unwrap()
        })
    });

    group.finish();
}

fn bench_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("Parser");

    // Simple expression
    group.bench_function("simple_expression", |b| {
        b.iter(|| {
            let expr = "Patient.name.given";
            let tokens = tokenize(expr).unwrap();
            parse(black_box(&tokens)).unwrap()
        })
    });

    // Complex expression
    group.bench_function("complex_expression", |b| {
        b.iter(|| {
            let expr = "Patient.name[0].given[0] = 'John' and Patient.gender = 'male'";
            let tokens = tokenize(expr).unwrap();
            parse(black_box(&tokens)).unwrap()
        })
    });

    // Expression with functions
    group.bench_function("expression_with_functions", |b| {
        b.iter(|| {
            let expr = "Patient.name.where(given.startsWith('J')).count() > 0";
            let tokens = tokenize(expr).unwrap();
            parse(black_box(&tokens)).unwrap()
        })
    });

    group.finish();
}

fn bench_evaluator(c: &mut Criterion) {
    let mut group = c.benchmark_group("Evaluator");

    // Sample patient resource
    let patient = json!({
        "resourceType": "Patient",
        "id": "example",
        "name": [
            {
                "use": "official",
                "family": "Smith",
                "given": ["John", "Adam"]
            }
        ],
        "gender": "male",
        "birthDate": "1974-12-25"
    });

    // Simple expression
    group.bench_function("simple_expression", |b| {
        b.iter(|| {
            let expr = "Patient.name.given";
            evaluate_expression(black_box(expr), black_box(patient.clone())).unwrap()
        })
    });

    // Complex expression
    group.bench_function("complex_expression", |b| {
        b.iter(|| {
            let expr = "Patient.name[0].given[0] = 'John' and Patient.gender = 'male'";
            evaluate_expression(black_box(expr), black_box(patient.clone())).unwrap()
        })
    });

    // Expression with functions
    group.bench_function("expression_with_functions", |b| {
        b.iter(|| {
            let expr = "Patient.name.where(given[0] = 'John').exists()";
            evaluate_expression(black_box(expr), black_box(patient.clone())).unwrap()
        })
    });

    group.finish();
}

fn bench_evaluator_with_visitor(c: &mut Criterion) {
    let mut group = c.benchmark_group("Evaluator with Visitor");

    // Sample patient resource
    let patient = json!({
        "resourceType": "Patient",
        "id": "example",
        "name": [
            {
                "use": "official",
                "family": "Smith",
                "given": ["John", "Adam"]
            }
        ],
        "gender": "male",
        "birthDate": "1974-12-25"
    });

    // With NoopVisitor
    group.bench_function("with_noop_visitor", |b| {
        b.iter(|| {
            let expr = "Patient.name[0].given[0] = 'John' and Patient.gender = 'male'";
            let visitor = NoopVisitor::new();
            evaluate_expression_with_visitor(black_box(expr), black_box(patient.clone()), &visitor)
                .unwrap()
        })
    });

    // With LoggingVisitor (only when trace feature is enabled)
    #[cfg(feature = "trace")]
    group.bench_function("with_logging_visitor", |b| {
        b.iter(|| {
            let expr = "Patient.name[0].given[0] = 'John' and Patient.gender = 'male'";
            let visitor = LoggingVisitor::new();
            evaluate_expression_with_visitor(black_box(expr), black_box(patient.clone()), &visitor)
                .unwrap()
        })
    });

    group.finish();
}

fn bench_optimization(c: &mut Criterion) {
    let mut group = c.benchmark_group("Optimization");

    // Sample patient resource
    let patient = json!({
        "resourceType": "Patient",
        "id": "example",
        "name": [
            {
                "use": "official",
                "family": "Smith",
                "given": ["John", "Adam"]
            }
        ],
        "gender": "male",
        "birthDate": "1974-12-25"
    });

    // Test caching performance - repeated evaluation of the same expression
    group.bench_function("without_caching_repeated", |b| {
        b.iter(|| {
            let expr = "Patient.name[0].given[0] = 'John' and Patient.gender = 'male'";
            // Evaluate the same expression multiple times without caching
            for _ in 0..10 {
                evaluate_expression(black_box(expr), black_box(patient.clone())).unwrap();
            }
        })
    });

    group.bench_function("with_caching_repeated", |b| {
        b.iter(|| {
            let expr = "Patient.name[0].given[0] = 'John' and Patient.gender = 'male'";
            // Evaluate the same expression multiple times with caching
            for _ in 0..10 {
                evaluate_expression_optimized(black_box(expr), black_box(patient.clone())).unwrap();
            }
        })
    });

    // Test caching with more complex repeated subexpressions
    group.bench_function("complex_caching_benefit", |b| {
        b.iter(|| {
            let expr = "Patient.name.where(given[0] = 'John').exists() and Patient.name.where(given[0] = 'John').family = 'Smith'";
            // This expression has repeated subexpressions that should benefit from caching
            for _ in 0..5 {
                evaluate_expression_optimized(black_box(expr), black_box(patient.clone())).unwrap();
            }
        })
    });

    group.bench_function("complex_without_caching_benefit", |b| {
        b.iter(|| {
            let expr = "Patient.name.where(given[0] = 'John').exists() and Patient.name.where(given[0] = 'John').family = 'Smith'";
            // Same expression without caching
            for _ in 0..5 {
                evaluate_expression(black_box(expr), black_box(patient.clone())).unwrap();
            }
        })
    });

    // Test constant folding optimization
    group.bench_function("constant_folding", |b| {
        b.iter(|| {
            let expr = "true and false or (1 + 2 = 3) and 'hello' + 'world' = 'helloworld'";
            evaluate_expression_optimized(black_box(expr), black_box(patient.clone())).unwrap()
        })
    });

    group.bench_function("without_constant_folding", |b| {
        b.iter(|| {
            let expr = "true and false or (1 + 2 = 3) and 'hello' + 'world' = 'helloworld'";
            evaluate_expression(black_box(expr), black_box(patient.clone())).unwrap()
        })
    });

    // Test complex expression optimization
    group.bench_function("complex_without_optimization", |b| {
        b.iter(|| {
            let expr = "Patient.name.where(given[0] = 'John').exists() and Patient.gender = 'male' and Patient.birthDate.exists()";
            evaluate_expression(black_box(expr), black_box(patient.clone())).unwrap()
        })
    });

    group.bench_function("complex_with_optimization", |b| {
        b.iter(|| {
            let expr = "Patient.name.where(given[0] = 'John').exists() and Patient.gender = 'male' and Patient.birthDate.exists()";
            evaluate_expression_optimized(black_box(expr), black_box(patient.clone())).unwrap()
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_lexer,
    bench_parser,
    bench_evaluator,
    bench_evaluator_with_visitor,
    bench_optimization
);
criterion_main!(benches);
