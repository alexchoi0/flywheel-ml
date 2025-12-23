use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalDriftResult {
    pub is_drifted: bool,
    pub psi_score: f64,
    pub kl_divergence: f64,
    pub feature_drifts: HashMap<String, FeatureDriftResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureDriftResult {
    pub feature_name: String,
    pub psi_score: f64,
    pub kl_divergence: f64,
    pub is_drifted: bool,
}

pub fn compute_psi(reference: &[f64], current: &[f64], bins: usize) -> f64 {
    let ref_hist = histogram(reference, bins);
    let cur_hist = histogram(current, bins);

    let mut psi = 0.0;
    for (ref_pct, cur_pct) in ref_hist.iter().zip(cur_hist.iter()) {
        let ref_pct = ref_pct.max(0.0001);
        let cur_pct = cur_pct.max(0.0001);
        psi += (cur_pct - ref_pct) * (cur_pct / ref_pct).ln();
    }
    psi
}

pub fn compute_kl_divergence(p: &[f64], q: &[f64]) -> f64 {
    p.iter()
        .zip(q.iter())
        .map(|(p_i, q_i)| {
            let p_i = p_i.max(1e-10);
            let q_i = q_i.max(1e-10);
            p_i * (p_i / q_i).ln()
        })
        .sum()
}

fn histogram(values: &[f64], bins: usize) -> Vec<f64> {
    if values.is_empty() {
        return vec![0.0; bins];
    }

    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    if (max - min).abs() < f64::EPSILON {
        let mut hist = vec![0.0; bins];
        hist[0] = 1.0;
        return hist;
    }

    let bin_width = (max - min) / bins as f64;
    let mut counts = vec![0usize; bins];

    for &v in values {
        let bin = ((v - min) / bin_width).floor() as usize;
        let bin = bin.min(bins - 1);
        counts[bin] += 1;
    }

    let total = values.len() as f64;
    counts.iter().map(|&c| c as f64 / total).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_psi_no_drift() {
        let reference: Vec<f64> = (0..1000).map(|i| i as f64 / 1000.0).collect();
        let current: Vec<f64> = (0..1000).map(|i| i as f64 / 1000.0).collect();

        let psi = compute_psi(&reference, &current, 10);
        assert!(psi < 0.1, "PSI should be low for identical distributions");
    }

    #[test]
    fn test_psi_with_drift() {
        let reference: Vec<f64> = (0..1000).map(|i| i as f64 / 1000.0).collect();
        let current: Vec<f64> = (0..1000).map(|i| (i as f64 / 1000.0) + 0.5).collect();

        let psi = compute_psi(&reference, &current, 10);
        assert!(psi > 0.1, "PSI should be high for shifted distributions");
    }
}
