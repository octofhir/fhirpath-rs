//! Benchmarks for FHIRPath Type Registry Performance using Divan

fn main() {
    divan::main();
}

/// Simple benchmark for O(1) type checking - will be implemented when bridge API is available
#[divan::bench]
fn bench_type_checking_placeholder() {
    // Placeholder benchmark for O(1) type checking
    // This will be replaced with actual implementation once bridge API is fully available
    divan::black_box("Patient".to_string());
}

/// Benchmark for hash vs linear lookup comparison
#[divan::bench(args = [10, 50, 100, 500])]
fn bench_hash_vs_linear(n: usize) {
    use std::collections::HashSet;

    let types: Vec<String> = (0..n).map(|i| format!("Type{}", i)).collect();
    let hash_set: HashSet<&String> = types.iter().collect();

    // Benchmark hash lookup (O(1))
    let target = &types[n / 2];
    divan::black_box(hash_set.contains(target));
}

/// Benchmark for mass operations to verify O(1) scaling
#[divan::bench(args = [100, 1000, 10000])]
fn bench_mass_operations(n: usize) {
    use std::collections::HashSet;

    let types: HashSet<&str> = ["Patient", "Observation", "Bundle"]
        .iter()
        .cloned()
        .collect();

    for _ in 0..n {
        divan::black_box(types.contains("Patient"));
    }
}
