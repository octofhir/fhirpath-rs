mod test_runner;

use test_runner::RustTestRunner;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("both");

    let runner = RustTestRunner::new()?;

    match command {
        "test" => {
            runner.run_tests()?;
        }
        "benchmark" => {
            runner.run_benchmarks()?;
        }
        "both" | _ => {
            runner.run_tests()?;
            runner.run_benchmarks()?;
        }
    }

    println!("✅ Rust test runner completed");
    Ok(())
}
