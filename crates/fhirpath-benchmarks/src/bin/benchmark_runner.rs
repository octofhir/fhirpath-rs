// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//\! Benchmark runner binary

use anyhow::Result;
use clap::{Arg, Command};
use fhirpath_benchmarks::BenchmarkSuite;

fn main() -> Result<()> {
    let matches = Command::new("benchmark-runner")
        .version("0.1.0")
        .about("Run performance benchmarks for FHIRPath implementation")
        .arg(
            Arg::new("output-dir")
                .long("output-dir")
                .value_name("DIR")
                .help("Output directory for benchmark results")
                .default_value("target/criterion"),
        )
        .get_matches();

    println!("Starting FHIRPath performance benchmarks...");

    let mut suite = BenchmarkSuite::new();
    suite.run_all();

    println!("Benchmarks completed!");

    Ok(())
}
