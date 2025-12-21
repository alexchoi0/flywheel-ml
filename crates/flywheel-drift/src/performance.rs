use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceDriftResult {
    pub is_degraded: bool,
    pub current_accuracy: f64,
    pub baseline_accuracy: f64,
    pub accuracy_delta: f64,
    pub current_precision: f64,
    pub current_recall: f64,
    pub current_latency_p99_ms: u64,
    pub current_error_rate: f64,
}

pub struct PerformanceTracker {
    true_positives: u64,
    true_negatives: u64,
    false_positives: u64,
    false_negatives: u64,
    latencies: Vec<u64>,
    errors: u64,
    total: u64,
}

impl PerformanceTracker {
    pub fn new() -> Self {
        Self {
            true_positives: 0,
            true_negatives: 0,
            false_positives: 0,
            false_negatives: 0,
            latencies: Vec::new(),
            errors: 0,
            total: 0,
        }
    }

    pub fn record_prediction(
        &mut self,
        predicted: bool,
        actual: bool,
        latency_ms: u64,
        is_error: bool,
    ) {
        self.total += 1;

        if is_error {
            self.errors += 1;
        } else {
            match (predicted, actual) {
                (true, true) => self.true_positives += 1,
                (true, false) => self.false_positives += 1,
                (false, true) => self.false_negatives += 1,
                (false, false) => self.true_negatives += 1,
            }
        }

        self.latencies.push(latency_ms);
    }

    pub fn accuracy(&self) -> f64 {
        let correct = self.true_positives + self.true_negatives;
        let total = self.true_positives + self.true_negatives + self.false_positives + self.false_negatives;
        if total == 0 {
            return 0.0;
        }
        correct as f64 / total as f64
    }

    pub fn precision(&self) -> f64 {
        let denominator = self.true_positives + self.false_positives;
        if denominator == 0 {
            return 0.0;
        }
        self.true_positives as f64 / denominator as f64
    }

    pub fn recall(&self) -> f64 {
        let denominator = self.true_positives + self.false_negatives;
        if denominator == 0 {
            return 0.0;
        }
        self.true_positives as f64 / denominator as f64
    }

    pub fn f1_score(&self) -> f64 {
        let p = self.precision();
        let r = self.recall();
        if p + r == 0.0 {
            return 0.0;
        }
        2.0 * p * r / (p + r)
    }

    pub fn error_rate(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        self.errors as f64 / self.total as f64
    }

    pub fn latency_p99(&self) -> u64 {
        if self.latencies.is_empty() {
            return 0;
        }
        let mut sorted = self.latencies.clone();
        sorted.sort();
        let idx = (sorted.len() * 99 / 100).min(sorted.len() - 1);
        sorted[idx]
    }

    pub fn reset(&mut self) {
        self.true_positives = 0;
        self.true_negatives = 0;
        self.false_positives = 0;
        self.false_negatives = 0;
        self.latencies.clear();
        self.errors = 0;
        self.total = 0;
    }
}

impl Default for PerformanceTracker {
    fn default() -> Self {
        Self::new()
    }
}
