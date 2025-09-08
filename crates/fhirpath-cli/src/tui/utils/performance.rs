//! Performance monitoring and optimization for TUI operations

use std::time::{Duration, Instant};

/// Performance tracking for TUI operations
pub struct PerformanceTracker {
    metrics: std::collections::HashMap<String, PerformanceMetric>,
    render_times: std::collections::VecDeque<Duration>,
    max_samples: usize,
}

/// Individual performance metric
#[derive(Debug, Clone)]
pub struct PerformanceMetric {
    pub name: String,
    pub total_time: Duration,
    pub count: u64,
    pub average_time: Duration,
    pub min_time: Duration,
    pub max_time: Duration,
    pub last_time: Duration,
}

impl PerformanceTracker {
    /// Create a new performance tracker
    pub fn new(max_samples: usize) -> Self {
        Self {
            metrics: std::collections::HashMap::new(),
            render_times: std::collections::VecDeque::new(),
            max_samples,
        }
    }
    
    /// Start timing an operation
    pub fn start_timer(&self, _name: &str) -> Timer {
        Timer::new()
    }
    
    /// Record operation completion
    pub fn record(&mut self, name: &str, duration: Duration) {
        let metric = self.metrics.entry(name.to_string()).or_insert_with(|| {
            PerformanceMetric {
                name: name.to_string(),
                total_time: Duration::ZERO,
                count: 0,
                average_time: Duration::ZERO,
                min_time: Duration::MAX,
                max_time: Duration::ZERO,
                last_time: Duration::ZERO,
            }
        });
        
        metric.total_time += duration;
        metric.count += 1;
        metric.average_time = metric.total_time / metric.count as u32;
        metric.min_time = metric.min_time.min(duration);
        metric.max_time = metric.max_time.max(duration);
        metric.last_time = duration;
    }
    
    /// Record render time
    pub fn record_render_time(&mut self, duration: Duration) {
        self.render_times.push_back(duration);
        if self.render_times.len() > self.max_samples {
            self.render_times.pop_front();
        }
    }
    
    /// Get average render time
    pub fn average_render_time(&self) -> Duration {
        if self.render_times.is_empty() {
            Duration::ZERO
        } else {
            let total: Duration = self.render_times.iter().sum();
            total / self.render_times.len() as u32
        }
    }
    
    /// Get current FPS estimate
    pub fn current_fps(&self) -> f64 {
        let avg_render_time = self.average_render_time();
        if avg_render_time.is_zero() {
            0.0
        } else {
            1000.0 / avg_render_time.as_millis() as f64
        }
    }
    
    /// Get performance summary
    pub fn summary(&self) -> Vec<String> {
        let mut summary = Vec::new();
        
        summary.push(format!("Average FPS: {:.1}", self.current_fps()));
        summary.push(format!("Average render time: {:?}", self.average_render_time()));
        
        for metric in self.metrics.values() {
            summary.push(format!(
                "{}: avg={:?}, count={}, min={:?}, max={:?}",
                metric.name,
                metric.average_time,
                metric.count,
                metric.min_time,
                metric.max_time
            ));
        }
        
        summary
    }
    
    /// Check if performance is acceptable
    pub fn is_performance_acceptable(&self) -> bool {
        self.current_fps() >= 30.0 // Target 30 FPS minimum
    }
}

/// Timer for measuring operation duration
pub struct Timer {
    start_time: Instant,
}

impl Timer {
    fn new() -> Self {
        Self {
            start_time: Instant::now(),
        }
    }
    
    /// Stop timer and get elapsed duration
    pub fn elapsed(self) -> Duration {
        self.start_time.elapsed()
    }
}