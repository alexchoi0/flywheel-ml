use crate::performance::PerformanceTracker;
use crate::statistical::compute_psi;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftConfig {
    pub psi_threshold: f64,
    pub kl_threshold: f64,
    pub accuracy_threshold: f64,
    pub window_size: usize,
    pub check_interval_secs: u64,
}

impl Default for DriftConfig {
    fn default() -> Self {
        Self {
            psi_threshold: 0.25,
            kl_threshold: 0.1,
            accuracy_threshold: 0.85,
            window_size: 10000,
            check_interval_secs: 300,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftResult {
    pub is_drifted: bool,
    pub drift_type: Option<DriftType>,
    pub severity: DriftSeverity,
    pub psi_score: Option<f64>,
    pub kl_divergence: Option<f64>,
    pub accuracy_delta: Option<f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DriftType {
    Statistical,
    Performance,
    Both,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DriftSeverity {
    None,
    Low,
    Medium,
    High,
    Critical,
}

impl DriftSeverity {
    pub fn from_psi(psi: f64) -> Self {
        if psi < 0.1 {
            DriftSeverity::None
        } else if psi < 0.25 {
            DriftSeverity::Low
        } else if psi < 0.5 {
            DriftSeverity::Medium
        } else if psi < 1.0 {
            DriftSeverity::High
        } else {
            DriftSeverity::Critical
        }
    }
}

pub struct DriftDetector {
    config: DriftConfig,
    reference_values: Vec<f64>,
    current_window: Vec<f64>,
    performance_tracker: PerformanceTracker,
    baseline_accuracy: f64,
}

impl DriftDetector {
    pub fn new(config: DriftConfig) -> Self {
        Self {
            config,
            reference_values: Vec::new(),
            current_window: Vec::new(),
            performance_tracker: PerformanceTracker::new(),
            baseline_accuracy: 0.9,
        }
    }

    pub fn set_reference(&mut self, values: Vec<f64>) {
        self.reference_values = values;
    }

    pub fn set_baseline_accuracy(&mut self, accuracy: f64) {
        self.baseline_accuracy = accuracy;
    }

    pub fn add_value(&mut self, value: f64) {
        self.current_window.push(value);
        if self.current_window.len() > self.config.window_size {
            self.current_window.remove(0);
        }
    }

    pub fn check_drift(&self) -> DriftResult {
        if self.reference_values.is_empty() || self.current_window.len() < 100 {
            return DriftResult {
                is_drifted: false,
                drift_type: None,
                severity: DriftSeverity::None,
                psi_score: None,
                kl_divergence: None,
                accuracy_delta: None,
            };
        }

        let psi = compute_psi(&self.reference_values, &self.current_window, 10);
        let statistical_drifted = psi > self.config.psi_threshold;

        let current_accuracy = self.performance_tracker.accuracy();
        let accuracy_delta = self.baseline_accuracy - current_accuracy;
        let performance_drifted = current_accuracy < self.config.accuracy_threshold;

        let (is_drifted, drift_type) = match (statistical_drifted, performance_drifted) {
            (true, true) => (true, Some(DriftType::Both)),
            (true, false) => (true, Some(DriftType::Statistical)),
            (false, true) => (true, Some(DriftType::Performance)),
            (false, false) => (false, None),
        };

        let severity = DriftSeverity::from_psi(psi);

        DriftResult {
            is_drifted,
            drift_type,
            severity,
            psi_score: Some(psi),
            kl_divergence: None,
            accuracy_delta: Some(accuracy_delta),
        }
    }
}
