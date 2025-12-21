use flywheel_core::FeatureVector;

pub struct BatchProcessor {
    batch_size: usize,
    buffer: Vec<FeatureVector>,
}

impl BatchProcessor {
    pub fn new(batch_size: usize) -> Self {
        Self {
            batch_size,
            buffer: Vec::with_capacity(batch_size),
        }
    }

    pub fn add(&mut self, features: FeatureVector) -> Option<Vec<FeatureVector>> {
        self.buffer.push(features);
        if self.buffer.len() >= self.batch_size {
            Some(std::mem::take(&mut self.buffer))
        } else {
            None
        }
    }

    pub fn flush(&mut self) -> Vec<FeatureVector> {
        std::mem::take(&mut self.buffer)
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}
