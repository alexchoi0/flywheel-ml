use flywheel_ml_core::{LabeledExample, SamplingConfig};
use rand::Rng;

pub struct Sampler {
    config: SamplingConfig,
    reservoir: Option<ReservoirSampler>,
}

impl Sampler {
    pub fn new(config: SamplingConfig) -> Self {
        let reservoir = match &config {
            SamplingConfig::ReservoirSampling { size } => Some(ReservoirSampler::new(*size)),
            _ => None,
        };
        Self { config, reservoir }
    }

    pub fn sample(&mut self, examples: Vec<LabeledExample>) -> Vec<LabeledExample> {
        match &mut self.reservoir {
            Some(reservoir) => {
                for example in examples {
                    reservoir.add(example);
                }
                reservoir.get_sample()
            }
            None => examples
                .into_iter()
                .filter(|e| self.should_sample(e))
                .collect(),
        }
    }

    pub fn sample_one(&mut self, example: LabeledExample) -> Option<LabeledExample> {
        match &mut self.reservoir {
            Some(reservoir) => {
                reservoir.add(example);
                None
            }
            None => {
                if self.should_sample(&example) {
                    Some(example)
                } else {
                    None
                }
            }
        }
    }

    fn should_sample(&self, example: &LabeledExample) -> bool {
        match &self.config {
            SamplingConfig::All => true,
            SamplingConfig::Random { rate } => rand_rate(*rate),
            SamplingConfig::Stratified {
                positive_rate,
                negative_rate,
            } => {
                if example.is_positive() {
                    rand_rate(*positive_rate)
                } else {
                    rand_rate(*negative_rate)
                }
            }
            SamplingConfig::HardNegative { threshold } => {
                if example.is_false_positive() {
                    if let Some(confidence) = example
                        .prediction
                        .get("confidence")
                        .and_then(|v| v.as_f64())
                    {
                        return confidence > *threshold;
                    }
                }
                example.is_positive()
            }
            SamplingConfig::ReservoirSampling { .. } => true,
        }
    }

    pub fn drain_reservoir(&mut self) -> Vec<LabeledExample> {
        if let Some(reservoir) = self.reservoir.take() {
            let samples = reservoir.get_sample();
            self.reservoir = Some(ReservoirSampler::new(samples.capacity()));
            samples
        } else {
            Vec::new()
        }
    }
}

struct ReservoirSampler {
    size: usize,
    reservoir: Vec<LabeledExample>,
    count: usize,
}

impl ReservoirSampler {
    fn new(size: usize) -> Self {
        Self {
            size,
            reservoir: Vec::with_capacity(size),
            count: 0,
        }
    }

    fn add(&mut self, example: LabeledExample) {
        self.count += 1;
        if self.reservoir.len() < self.size {
            self.reservoir.push(example);
        } else {
            let mut rng = rand::thread_rng();
            let j = rng.gen_range(0..self.count);
            if j < self.size {
                self.reservoir[j] = example;
            }
        }
    }

    fn get_sample(&self) -> Vec<LabeledExample> {
        self.reservoir.clone()
    }
}

fn rand_rate(rate: f64) -> bool {
    let mut rng = rand::thread_rng();
    rng.gen::<f64>() < rate
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use flywheel_ml_core::GroundTruth;
    use std::collections::HashMap;

    fn make_test_example(is_positive: bool) -> LabeledExample {
        LabeledExample {
            example_id: uuid::Uuid::new_v4().to_string(),
            prediction_id: "pred-1".to_string(),
            model_id: "model-1".to_string(),
            model_version: "v1".to_string(),
            features: serde_json::json!({"cpu": 0.85}),
            prediction: serde_json::json!({"score": 0.9, "confidence": 0.95}),
            ground_truth: GroundTruth::Binary(is_positive),
            prediction_timestamp: Utc::now(),
            feedback_timestamp: Utc::now(),
            delay_ms: 1000,
            feedback_confidence: 0.95,
            is_correct: Some(true),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_sample_all() {
        let mut sampler = Sampler::new(SamplingConfig::All);
        let examples = vec![make_test_example(true), make_test_example(false)];
        let sampled = sampler.sample(examples);
        assert_eq!(sampled.len(), 2);
    }

    #[test]
    fn test_reservoir_sampling() {
        let mut sampler = Sampler::new(SamplingConfig::ReservoirSampling { size: 5 });

        for _ in 0..100 {
            sampler.sample_one(make_test_example(true));
        }

        let sampled = sampler.drain_reservoir();
        assert_eq!(sampled.len(), 5);
    }

    #[test]
    fn test_random_sampling() {
        let mut sampler = Sampler::new(SamplingConfig::Random { rate: 0.5 });
        let examples: Vec<_> = (0..1000).map(|_| make_test_example(true)).collect();
        let sampled = sampler.sample(examples);
        assert!(sampled.len() > 300 && sampled.len() < 700);
    }
}
