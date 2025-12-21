use flywheel_ml_core::{FeedbackRecord, StoredPrediction, LabeledExample};
use std::collections::HashMap;

pub struct Labeler {
    implicit_rules: Vec<ImplicitLabelRule>,
    label_mapping: HashMap<String, String>,
}

pub struct ImplicitLabelRule {
    pub event_type: String,
    pub label: String,
    pub confidence: f64,
}

impl Labeler {
    pub fn new() -> Self {
        Self {
            implicit_rules: Vec::new(),
            label_mapping: HashMap::new(),
        }
    }

    pub fn add_implicit_rule(&mut self, event_type: impl Into<String>, label: impl Into<String>, confidence: f64) {
        self.implicit_rules.push(ImplicitLabelRule {
            event_type: event_type.into(),
            label: label.into(),
            confidence,
        });
    }

    pub fn add_label_mapping(&mut self, from: impl Into<String>, to: impl Into<String>) {
        self.label_mapping.insert(from.into(), to.into());
    }

    pub fn apply_implicit_rule(&self, event_type: &str) -> Option<(String, f64)> {
        for rule in &self.implicit_rules {
            if rule.event_type == event_type {
                return Some((rule.label.clone(), rule.confidence));
            }
        }
        None
    }

    pub fn map_label(&self, label: &str) -> String {
        self.label_mapping
            .get(label)
            .cloned()
            .unwrap_or_else(|| label.to_string())
    }

    pub fn create_labeled_example(
        &self,
        prediction: &StoredPrediction,
        feedback: &FeedbackRecord,
    ) -> LabeledExample {
        LabeledExample::from_prediction_and_feedback(prediction, feedback)
    }
}

impl Default for Labeler {
    fn default() -> Self {
        Self::new()
    }
}
