use flywheel_ml_drift::{DriftConfig, DriftDetector, DriftResult};

pub struct DriftDetectionTransform {
    detector: DriftDetector,
    shadow_mode: bool,
}

impl DriftDetectionTransform {
    pub fn new(config: DriftConfig, shadow_mode: bool) -> Self {
        Self {
            detector: DriftDetector::new(config),
            shadow_mode,
        }
    }

    pub fn set_reference(&mut self, values: Vec<f64>) {
        self.detector.set_reference(values);
    }

    pub fn add_value(&mut self, value: f64) {
        self.detector.add_value(value);
    }

    pub fn check_drift(&self) -> DriftResult {
        self.detector.check_drift()
    }

    pub fn is_shadow_mode(&self) -> bool {
        self.shadow_mode
    }
}
