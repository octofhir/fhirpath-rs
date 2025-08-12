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

//\! Performance profiler binary

use anyhow::Result;
use clap::{Arg, Command};
use octofhir_fhirpath_benchmarks::profiling::ProfilerContext;

fn main() -> Result<()> {
    let matches = Command::new("performance-profiler")
        .version("0.1.0")
        .about("Profile FHIRPath performance and generate flamegraphs")
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Output file for flamegraph")
                .default_value("flamegraph.svg"),
        )
        .get_matches();

    println!("Starting performance profiling...");

    let profiler = ProfilerContext::start();

    // TODO: Run actual profiling workload
    std::thread::sleep(std::time::Duration::from_millis(100));

    if let Some(flamegraph) = profiler.flamegraph() {
        let output_file = matches.get_one::<String>("output").unwrap();
        std::fs::write(output_file, flamegraph)?;
        println!("Flamegraph written to: {}", output_file);
    } else {
        println!("Profiling feature not available");
    }

    Ok(())
}
