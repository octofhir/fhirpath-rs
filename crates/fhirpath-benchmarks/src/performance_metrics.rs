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

//! Performance metrics collection and reporting

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Performance metrics collector
pub struct MetricsCollector {
    measurements: HashMap<String, Vec<Duration>>,
    start_times: HashMap<String, Instant>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            measurements: HashMap::new(),
            start_times: HashMap::new(),
        }
    }

    /// Start timing a metric
    pub fn start_timing(&mut self, metric_name: &str) {
        self.start_times
            .insert(metric_name.to_string(), Instant::now());
    }

    /// Stop timing a metric
    pub fn stop_timing(&mut self, metric_name: &str) {
        if let Some(start_time) = self.start_times.remove(metric_name) {
            let duration = start_time.elapsed();
            self.measurements
                .entry(metric_name.to_string())
                .or_default()
                .push(duration);
        }
    }

    /// Get performance report
    pub fn generate_report(&self) -> PerformanceReport {
        let mut metrics = HashMap::new();

        for (name, durations) in &self.measurements {
            if !durations.is_empty() {
                let total: Duration = durations.iter().sum();
                let avg = total / durations.len() as u32;
                let min = *durations.iter().min().unwrap();
                let max = *durations.iter().max().unwrap();

                metrics.insert(
                    name.clone(),
                    MetricSummary {
                        avg,
                        min,
                        max,
                        count: durations.len(),
                        total,
                    },
                );
            }
        }

        PerformanceReport { metrics }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance report
pub struct PerformanceReport {
    pub metrics: HashMap<String, MetricSummary>,
}

impl PerformanceReport {
    /// Generate markdown report
    pub fn to_markdown(&self) -> String {
        let mut report = String::new();
        report.push_str("# Performance Report\n\n");

        for (name, summary) in &self.metrics {
            report.push_str(&format!("## {}\n\n", name));
            report.push_str(&format!("- **Average**: {:?}\n", summary.avg));
            report.push_str(&format!("- **Min**: {:?}\n", summary.min));
            report.push_str(&format!("- **Max**: {:?}\n", summary.max));
            report.push_str(&format!("- **Count**: {}\n", summary.count));
            report.push_str(&format!("- **Total**: {:?}\n\n", summary.total));
        }

        report
    }
}

/// Summary of a performance metric
pub struct MetricSummary {
    pub avg: Duration,
    pub min: Duration,
    pub max: Duration,
    pub count: usize,
    pub total: Duration,
}
