use criterion::{Criterion, black_box, criterion_group, criterion_main};
use fhirpath_parser::tokenizer::Tokenizer;

fn benchmark_optimized_tokenizer_only(c: &mut Criterion) {
    let expression = "Patient.name.where(use = 'official').given";

    c.bench_function("optimized_tokenizer_only", |b| {
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

fn benchmark_optimized_tokenizer_complete(c: &mut Criterion) {
    let expression = "Patient.name.where(use = 'official').given";

    c.bench_function("optimized_tokenizer_complete", |b| {
        b.iter(|| {
            let mut tokenizer = Tokenizer::new(black_box(expression));
            black_box(tokenizer.tokenize_all())
        })
    });
}

fn benchmark_multiple_expressions(c: &mut Criterion) {
    let expressions = vec![
        "Patient.name",
        "Patient.name.given",
        "Patient.name.where(use = 'official')",
        "Patient.name.where(use = 'official').given",
        "Patient.telecom.where(system = 'phone').value",
        "Bundle.entry.resource.where($this is Patient).name",
        "Observation.value.where(code = 'weight').value",
        "Patient.address.where(use = 'home').line.first()",
    ];

    for (i, expression) in expressions.into_iter().enumerate() {
        c.bench_function(&format!("expr_{}_tokenizer", i), |b| {
            b.iter(|| {
                let mut tokenizer = Tokenizer::new(black_box(expression));
                black_box(tokenizer.tokenize_all())
            })
        });
    }
}

fn benchmark_operations_per_second(c: &mut Criterion) {
    let expression = "Patient.name.where(use = 'official').given";

    let mut group = c.benchmark_group("operations_per_second");
    group.sample_size(10000); // More samples for accurate measurement

    group.bench_function("tokenizer_ops_per_sec", |b| {
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
    benchmark_optimized_tokenizer_only,
    benchmark_optimized_tokenizer_complete,
    benchmark_multiple_expressions,
    benchmark_operations_per_second
);
criterion_main!(benches);
