use criterion::{Criterion, black_box, criterion_group, criterion_main};
use fhirpath_parser::{parse, tokenizer::Tokenizer};

fn benchmark_parser(c: &mut Criterion) {
    let expression = "Patient.name.where(use = 'official').given";

    c.bench_function("parser", |b| {
        b.iter(|| black_box(parse(black_box(expression))))
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

fn benchmark_tokenizer_complete(c: &mut Criterion) {
    let expression = "Patient.name.where(use = 'official').given";

    c.bench_function("tokenizer_complete", |b| {
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
        "Bundle.entry.resource.name",
        "Observation.value",
        "Patient.address.line",
    ];

    for (i, expression) in expressions.into_iter().enumerate() {
        c.bench_function(&format!("expr_{}_tokenizer", i), |b| {
            b.iter(|| {
                let mut tokenizer = Tokenizer::new(black_box(expression));
                black_box(tokenizer.tokenize_all())
            })
        });

        c.bench_function(&format!("expr_{}_parser", i), |b| {
            b.iter(|| black_box(parse(black_box(expression))))
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

    group.bench_function("parser_ops_per_sec", |b| {
        b.iter(|| black_box(parse(black_box(expression))))
    });

    group.finish();
}

fn benchmark_parser_vs_tokenizer_comparison(c: &mut Criterion) {
    let expressions = vec![
        ("simple", "Patient.name"),
        ("medium", "Patient.name.given"),
        ("complex", "Patient.name.where(use = 'official').given"),
    ];

    for (name, expression) in expressions {
        let mut group = c.benchmark_group(&format!("comparison_{}", name));

        group.bench_function("tokenizer", |b| {
            b.iter(|| {
                let mut tokenizer = Tokenizer::new(black_box(expression));
                let mut token_count = 0;
                while let Ok(Some(_)) = tokenizer.next_token() {
                    token_count += 1;
                }
                black_box(token_count)
            })
        });

        group.bench_function("parser", |b| {
            b.iter(|| black_box(parse(black_box(expression))))
        });

        group.finish();
    }
}

criterion_group!(
    benches,
    benchmark_parser,
    benchmark_tokenizer_only,
    benchmark_tokenizer_complete,
    benchmark_multiple_expressions,
    benchmark_operations_per_second,
    benchmark_parser_vs_tokenizer_comparison
);
criterion_main!(benches);
