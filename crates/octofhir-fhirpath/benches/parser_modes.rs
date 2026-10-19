//! Parser throughput measurements, kept separate from correctness and coverage tests.

use divan::{Bencher, black_box, counter::ItemsCount};
use octofhir_fhirpath::parser::{ParsingMode, parse_with_mode};

const EXPRESSIONS: &[&str] = &[
    "Patient.name.given.first()",
    "age > 18 and status = 'active'",
    "value * 2.5 + offset",
    "items.where(active = true).count() > 0",
    "Patient.name.first()",
    "true and false or true",
    "a = b",
    "Patient.extension.value.as(string)",
];

fn main() {
    divan::main();
}

#[divan::bench(args = EXPRESSIONS)]
fn fast(bencher: Bencher, expression: &str) {
    assert!(parse_with_mode(expression, ParsingMode::Fast).success);
    bencher
        .counter(ItemsCount::new(1_usize))
        .bench_local(|| black_box(parse_with_mode(black_box(expression), ParsingMode::Fast)));
}

#[divan::bench(args = EXPRESSIONS)]
fn analysis(bencher: Bencher, expression: &str) {
    assert!(parse_with_mode(expression, ParsingMode::Analysis).success);
    bencher.counter(ItemsCount::new(1_usize)).bench_local(|| {
        black_box(parse_with_mode(
            black_box(expression),
            ParsingMode::Analysis,
        ))
    });
}
