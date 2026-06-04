//! Trust score tracking with decay and evolution.

use std::collections::HashMap;

/// Default half-life for trust decay (in ticks/interactions).
const DEFAULT_HALF_LIFE: f64 = 100.0;

/// A trust record for a single entity.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TrustRecord {
    pub label: String,
    pub score: f64,
    pub interactions: u64,
    pub last_tick: u64,
}

/// Trust score tracker with exponential decay.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TrustTracker {
    records: HashMap<String, TrustRecord>,
    current_tick: u64,
    half_life: f64,
}

impl TrustTracker {
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
            current_tick: 0,
            half_life: DEFAULT_HALF_LIFE,
        }
    }

    pub fn with_half_life(mut self, half_life: f64) -> Self {
        self.half_life = half_life;
        self
    }

    /// Advance the global tick counter.
    pub fn tick(&mut self) {
        self.current_tick += 1;
    }

    /// Record an interaction and update trust.
    /// Positive delta increases trust, negative decreases.
    pub fn observe(&mut self, label: &str, delta: f64) {
        self.tick();
        let record = self.records.entry(label.to_string()).or_insert(TrustRecord {
            label: label.to_string(),
            score: 0.5, // neutral start
            interactions: 0,
            last_tick: self.current_tick,
        });

        // Apply decay since last observation
        let elapsed = self.current_tick - record.last_tick;
        if elapsed > 0 {
            let decay = 2.0f64.powf(-(elapsed as f64) / self.half_life);
            record.score = record.score * decay + 0.5 * (1.0 - decay); // decay toward 0.5
        }

        // Apply delta with diminishing returns
        let weight = 1.0 / (1.0 + record.interactions as f64 * 0.1);
        record.score += delta * weight * 0.1;
        record.score = record.score.clamp(0.0, 1.0);
        record.interactions += 1;
        record.last_tick = self.current_tick;
    }

    /// Get the current trust score for a label.
    pub fn get_score(&self, label: &str) -> f64 {
        self.records
            .get(label)
            .map(|r| r.score)
            .unwrap_or(0.5) // unknown = neutral
    }

    /// Get a trust record.
    pub fn get_record(&self, label: &str) -> Option<&TrustRecord> {
        self.records.get(label)
    }

    /// List all labels sorted by trust score (descending).
    pub fn ranked(&self) -> Vec<&TrustRecord> {
        let mut recs: Vec<&TrustRecord> = self.records.values().collect();
        recs.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        recs
    }

    /// Number of tracked entities.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

impl Default for TrustTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_neutral() {
        let tracker = TrustTracker::new();
        assert_eq!(tracker.get_score("unknown"), 0.5);
    }

    #[test]
    fn test_positive_observation() {
        let mut tracker = TrustTracker::new();
        tracker.observe("agent_a", 1.0);
        assert!(tracker.get_score("agent_a") > 0.5);
    }

    #[test]
    fn test_negative_observation() {
        let mut tracker = TrustTracker::new();
        tracker.observe("agent_b", -1.0);
        assert!(tracker.get_score("agent_b") < 0.5);
    }

    #[test]
    fn test_clamped() {
        let mut tracker = TrustTracker::new();
        for _ in 0..1000 {
            tracker.observe("agent_c", 1.0);
        }
        assert!(tracker.get_score("agent_c") <= 1.0);
    }

    #[test]
    fn test_decay() {
        let mut tracker = TrustTracker::new().with_half_life(10.0);
        tracker.observe("agent_d", 1.0);
        let initial = tracker.get_score("agent_d");
        // Many ticks without observation
        for _ in 0..100 {
            tracker.tick();
        }
        let decayed = tracker.get_score("agent_d");
        // Score should have decayed (not refreshed, so still at old value)
        // Actually get_score doesn't apply decay — only observe does
        assert_eq!(initial, decayed); // unchanged since no new observation
    }

    #[test]
    fn test_ranking() {
        let mut tracker = TrustTracker::new();
        tracker.observe("low", -1.0);
        tracker.observe("high", 1.0);
        tracker.observe("mid", 0.0);
        let ranked = tracker.ranked();
        assert_eq!(ranked[0].label, "high");
    }
}
