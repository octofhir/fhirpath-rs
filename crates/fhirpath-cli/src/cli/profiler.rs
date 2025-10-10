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

//! Performance profiling for FHIRPath evaluation

use comfy_table::{Table, presets::UTF8_FULL};
use std::time::{Duration, Instant};

/// Performance profiler for tracking operation timings
#[derive(Debug, Clone)]
pub struct PerformanceProfiler {
    start_time: Instant,
    phases: Vec<ProfilePhase>,
    current_phase: Option<String>,
    phase_start: Option<Instant>,
}

/// A profiled phase of execution
#[derive(Debug, Clone)]
pub struct ProfilePhase {
    pub name: String,
    pub duration: Duration,
    pub percentage: f64,
}

impl PerformanceProfiler {
    /// Create a new profiler
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            phases: Vec::new(),
            current_phase: None,
            phase_start: None,
        }
    }

    /// Start a new phase
    pub fn start_phase(&mut self, name: impl Into<String>) {
        // End previous phase if any
        if self.current_phase.is_some() {
            self.end_phase();
        }

        self.current_phase = Some(name.into());
        self.phase_start = Some(Instant::now());
    }

    /// End the current phase
    pub fn end_phase(&mut self) {
        if let (Some(name), Some(start)) = (self.current_phase.take(), self.phase_start.take()) {
            let duration = start.elapsed();
            self.phases.push(ProfilePhase {
                name,
                duration,
                percentage: 0.0, // Will be calculated in finalize()
            });
        }
    }

    /// Finalize profiling and calculate percentages
    pub fn finalize(&mut self) -> PerformanceReport {
        // End current phase if any
        if self.current_phase.is_some() {
            self.end_phase();
        }

        let total_duration = self.start_time.elapsed();

        // Calculate percentages
        for phase in &mut self.phases {
            phase.percentage =
                (phase.duration.as_secs_f64() / total_duration.as_secs_f64()) * 100.0;
        }

        PerformanceReport {
            total_duration,
            phases: self.phases.clone(),
        }
    }

    /// Get total elapsed time since profiler creation
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

impl Default for PerformanceProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance report with timing breakdown
#[derive(Debug, Clone)]
pub struct PerformanceReport {
    pub total_duration: Duration,
    pub phases: Vec<ProfilePhase>,
}

impl PerformanceReport {
    /// Format the report as a table
    pub fn format_table(&self) -> String {
        let mut table = Table::new();
        table.load_preset(UTF8_FULL);
        table.set_header(vec!["Phase", "Duration", "Percentage"]);

        for phase in &self.phases {
            table.add_row(vec![
                &phase.name,
                &format!("{:.2}ms", phase.duration.as_secs_f64() * 1000.0),
                &format!("{:.1}%", phase.percentage),
            ]);
        }

        // Add total row
        table.add_row(vec![
            "TOTAL",
            &format!("{:.2}ms", self.total_duration.as_secs_f64() * 1000.0),
            "100.0%",
        ]);

        format!("\n{}\n", table)
    }

    /// Format the report as simple text
    pub fn format_text(&self) -> String {
        let mut output = String::new();
        output.push_str("\n⏱️  Performance Profile:\n\n");

        for phase in &self.phases {
            output.push_str(&format!(
                "  {} {:.2}ms ({:.1}%)\n",
                phase.name,
                phase.duration.as_secs_f64() * 1000.0,
                phase.percentage
            ));
        }

        output.push_str(&format!(
            "\n  TOTAL: {:.2}ms\n",
            self.total_duration.as_secs_f64() * 1000.0
        ));

        output
    }

    /// Format as JSON
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "total_ms": self.total_duration.as_secs_f64() * 1000.0,
            "phases": self.phases.iter().map(|p| {
                serde_json::json!({
                    "name": p.name,
                    "duration_ms": p.duration.as_secs_f64() * 1000.0,
                    "percentage": p.percentage
                })
            }).collect::<Vec<_>>()
        })
    }
}

/// Helper macro to time a block of code
#[macro_export]
macro_rules! profile_phase {
    ($profiler:expr, $name:expr, $block:block) => {{
        $profiler.start_phase($name);
        let result = $block;
        $profiler.end_phase();
        result
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_profiler() {
        let mut profiler = PerformanceProfiler::new();

        profiler.start_phase("Phase 1");
        sleep(Duration::from_millis(10));
        profiler.end_phase();

        profiler.start_phase("Phase 2");
        sleep(Duration::from_millis(20));
        profiler.end_phase();

        let report = profiler.finalize();
        assert_eq!(report.phases.len(), 2);
        assert!(report.total_duration.as_millis() >= 30);
    }

    #[test]
    fn test_auto_end_phase() {
        let mut profiler = PerformanceProfiler::new();

        profiler.start_phase("Phase 1");
        profiler.start_phase("Phase 2"); // Should auto-end Phase 1

        let report = profiler.finalize();
        assert_eq!(report.phases.len(), 2);
    }
}
