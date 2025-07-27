use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fhirpath_parser::parse_expression;
use fhirpath_parser::tokenizer::Tokenizer;

fn benchmark_parser_complete(c: &mut Criterion) {
    let expression = "Patient.name.where(use = 'official').given";

    c.bench_function("parser_complete", |b| {
        b.iter(|| {
            black_box(parse_expression(black_box(expression)))
        })
    });
}

fn benchmark_tokenizer_only(c: &mut Criterion) {
    let expression = "Patient.name.where(use = 'official').given";

    c.bench_function("tokenizer_only", |b| {
        b.iter(|| {
            let mut tokenizer = Tokenizer::new(black_box(expression));
            let mut token_count = 0;
            while let Ok(Some(_)) = tokenizer.next_token() {
                token_count += 1;
            }
            black_box(token_count)
        })
    });
}

fn benchmark_performance_target(c: &mut Criterion) {
    let expression = "Patient.name.where(use = 'official').given";

    let mut group = c.benchmark_group("performance_validation");
    group.sample_size(10000);

    group.bench_function("parser_1M_ops_target", |b| {
        b.iter(|| {
            black_box(parse_expression(black_box(expression)))
        })
    });

    group.bench_function("tokenizer_11M_ops_target", |b| {
        b.iter(|| {
            let mut tokenizer = Tokenizer::new(black_box(expression));
            let mut token_count = 0;
            while let Ok(Some(_)) = tokenizer.next_token() {
                token_count += 1;
            }
            black_box(token_count)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_parser_complete,
    benchmark_tokenizer_only,
    benchmark_performance_target
);
criterion_main!(benches);
