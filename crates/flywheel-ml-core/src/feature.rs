use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::FeatureError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureVector {
    pub features: HashMap<String, FeatureValue>,
    pub timestamp: DateTime<Utc>,
    pub source_record_id: String,
    pub metadata: HashMap<String, String>,
}

impl FeatureVector {
    pub fn new(source_record_id: impl Into<String>) -> Self {
        Self {
            features: HashMap::new(),
            timestamp: Utc::now(),
            source_record_id: source_record_id.into(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_feature(mut self, name: impl Into<String>, value: FeatureValue) -> Self {
        self.features.insert(name.into(), value);
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn get(&self, name: &str) -> Option<&FeatureValue> {
        self.features.get(name)
    }

    pub fn get_float(&self, name: &str) -> Option<f64> {
        match self.features.get(name) {
            Some(FeatureValue::Float(v)) => Some(*v),
            Some(FeatureValue::Int(v)) => Some(*v as f64),
            _ => None,
        }
    }

    pub fn get_string(&self, name: &str) -> Option<&str> {
        match self.features.get(name) {
            Some(FeatureValue::String(s)) => Some(s),
            Some(FeatureValue::Categorical(s)) => Some(s),
            _ => None,
        }
    }

    pub fn feature_names(&self) -> Vec<&str> {
        self.features.keys().map(|k| k.as_str()).collect()
    }

    pub fn hash(&self) -> String {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();

        let mut keys: Vec<_> = self.features.keys().collect();
        keys.sort();

        for key in keys {
            key.hash(&mut hasher);
            if let Some(value) = self.features.get(key) {
                format!("{:?}", value).hash(&mut hasher);
            }
        }

        format!("{:x}", hasher.finish())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum FeatureValue {
    Float(f64),
    Int(i64),
    String(String),
    FloatArray(Vec<f64>),
    IntArray(Vec<i64>),
    Embedding(Vec<f32>),
    Categorical(String),
    Boolean(bool),
    Null,
}

impl FeatureValue {
    pub fn as_float(&self) -> Option<f64> {
        match self {
            FeatureValue::Float(v) => Some(*v),
            FeatureValue::Int(v) => Some(*v as f64),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            FeatureValue::Int(v) => Some(*v),
            FeatureValue::Float(v) => Some(*v as i64),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            FeatureValue::String(s) => Some(s),
            FeatureValue::Categorical(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            FeatureValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, FeatureValue::Null)
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            FeatureValue::Float(_) => "float",
            FeatureValue::Int(_) => "int",
            FeatureValue::String(_) => "string",
            FeatureValue::FloatArray(_) => "float_array",
            FeatureValue::IntArray(_) => "int_array",
            FeatureValue::Embedding(_) => "embedding",
            FeatureValue::Categorical(_) => "categorical",
            FeatureValue::Boolean(_) => "boolean",
            FeatureValue::Null => "null",
        }
    }
}

impl From<f64> for FeatureValue {
    fn from(v: f64) -> Self {
        FeatureValue::Float(v)
    }
}

impl From<i64> for FeatureValue {
    fn from(v: i64) -> Self {
        FeatureValue::Int(v)
    }
}

impl From<String> for FeatureValue {
    fn from(v: String) -> Self {
        FeatureValue::String(v)
    }
}

impl From<&str> for FeatureValue {
    fn from(v: &str) -> Self {
        FeatureValue::String(v.to_string())
    }
}

impl From<bool> for FeatureValue {
    fn from(v: bool) -> Self {
        FeatureValue::Boolean(v)
    }
}

impl From<Vec<f64>> for FeatureValue {
    fn from(v: Vec<f64>) -> Self {
        FeatureValue::FloatArray(v)
    }
}

impl From<Vec<f32>> for FeatureValue {
    fn from(v: Vec<f32>) -> Self {
        FeatureValue::Embedding(v)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureSchema {
    pub name: String,
    pub features: Vec<FeatureDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureDefinition {
    pub name: String,
    pub feature_type: FeatureType,
    pub nullable: bool,
    pub description: Option<String>,
    pub default_value: Option<FeatureValue>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FeatureType {
    Float,
    Int,
    String,
    FloatArray,
    IntArray,
    Embedding,
    Categorical,
    Boolean,
}

impl FeatureType {
    pub fn matches(&self, value: &FeatureValue) -> bool {
        match (self, value) {
            (FeatureType::Float, FeatureValue::Float(_)) => true,
            (FeatureType::Float, FeatureValue::Int(_)) => true, // Allow int -> float
            (FeatureType::Int, FeatureValue::Int(_)) => true,
            (FeatureType::String, FeatureValue::String(_)) => true,
            (FeatureType::FloatArray, FeatureValue::FloatArray(_)) => true,
            (FeatureType::IntArray, FeatureValue::IntArray(_)) => true,
            (FeatureType::Embedding, FeatureValue::Embedding(_)) => true,
            (FeatureType::Categorical, FeatureValue::Categorical(_)) => true,
            (FeatureType::Categorical, FeatureValue::String(_)) => true,
            (FeatureType::Boolean, FeatureValue::Boolean(_)) => true,
            (_, FeatureValue::Null) => true, // Null matches anything (checked separately for nullable)
            _ => false,
        }
    }
}

#[async_trait]
pub trait FeatureExtractor: Send + Sync {
    fn name(&self) -> &str;

    fn schema(&self) -> &FeatureSchema;

    async fn extract(&self, record: &serde_json::Value) -> Result<FeatureVector, FeatureError>;

    async fn extract_batch(
        &self,
        records: &[serde_json::Value],
    ) -> Result<Vec<FeatureVector>, FeatureError> {
        let mut results = Vec::with_capacity(records.len());
        for record in records {
            results.push(self.extract(record).await?);
        }
        Ok(results)
    }

    fn validate(&self, features: &FeatureVector) -> Result<(), FeatureError> {
        for def in &self.schema().features {
            match features.get(&def.name) {
                Some(value) => {
                    if !def.feature_type.matches(value) {
                        return Err(FeatureError::TypeMismatch {
                            feature: def.name.clone(),
                            expected: format!("{:?}", def.feature_type),
                            actual: value.type_name().to_string(),
                        });
                    }
                }
                None => {
                    if !def.nullable && def.default_value.is_none() {
                        return Err(FeatureError::MissingField(def.name.clone()));
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureTransform {
    Normalize { min: f64, max: f64 },
    Log1p,
    Clip { min: f64, max: f64 },
    Bucketize { boundaries: Vec<f64> },
    OneHot { categories: Vec<String> },
    StandardScale { mean: f64, std: f64 },
    MinMaxScale { min: f64, max: f64 },
}

impl FeatureTransform {
    pub fn apply(&self, value: &FeatureValue) -> Result<FeatureValue, FeatureError> {
        match (self, value) {
            (FeatureTransform::Normalize { min, max }, FeatureValue::Float(v)) => {
                let normalized = (v - min) / (max - min);
                Ok(FeatureValue::Float(normalized.clamp(0.0, 1.0)))
            }
            (FeatureTransform::Normalize { min, max }, FeatureValue::Int(v)) => {
                let v = *v as f64;
                let normalized = (v - min) / (max - min);
                Ok(FeatureValue::Float(normalized.clamp(0.0, 1.0)))
            }
            (FeatureTransform::Log1p, FeatureValue::Float(v)) => {
                Ok(FeatureValue::Float((v + 1.0).ln()))
            }
            (FeatureTransform::Log1p, FeatureValue::Int(v)) => {
                Ok(FeatureValue::Float((*v as f64 + 1.0).ln()))
            }
            (FeatureTransform::Clip { min, max }, FeatureValue::Float(v)) => {
                Ok(FeatureValue::Float(v.clamp(*min, *max)))
            }
            (FeatureTransform::Clip { min, max }, FeatureValue::Int(v)) => {
                let v = *v as f64;
                Ok(FeatureValue::Float(v.clamp(*min, *max)))
            }
            (FeatureTransform::Bucketize { boundaries }, FeatureValue::Float(v)) => {
                let bucket = boundaries.iter().filter(|b| v >= *b).count();
                Ok(FeatureValue::Int(bucket as i64))
            }
            (FeatureTransform::OneHot { categories }, FeatureValue::String(s))
            | (FeatureTransform::OneHot { categories }, FeatureValue::Categorical(s)) => {
                let one_hot: Vec<f64> = categories
                    .iter()
                    .map(|c| if c == s { 1.0 } else { 0.0 })
                    .collect();
                Ok(FeatureValue::FloatArray(one_hot))
            }
            (FeatureTransform::StandardScale { mean, std }, FeatureValue::Float(v)) => {
                Ok(FeatureValue::Float((v - mean) / std))
            }
            (FeatureTransform::MinMaxScale { min, max }, FeatureValue::Float(v)) => {
                let scaled = (v - min) / (max - min);
                Ok(FeatureValue::Float(scaled))
            }
            _ => Err(FeatureError::InvalidValue {
                feature: String::new(),
                reason: format!(
                    "Cannot apply {:?} transform to {:?}",
                    self,
                    value.type_name()
                ),
            }),
        }
    }
}
