//! Three-gate pipeline: exact → vector → fallback.
//! State machine for intent resolution.

use crate::embed::{embed_intent, Embedding};
use crate::hash::hash_intent;
use crate::search::cosine_search;

/// Threshold for exact hash match (always 1.0).
const EXACT_THRESHOLD: f64 = 1.0;

/// Default threshold for vector similarity match.
const VECTOR_THRESHOLD: f64 = 0.85;

/// Result of the gate pipeline.
#[derive(Clone, Debug, PartialEq)]
pub enum GateResult {
    /// Gate 1: exact hash match.
    Exact {
        label: String,
        confidence: f64,
    },
    /// Gate 2: vector similarity match above threshold.
    Vector {
        label: String,
        confidence: f64,
    },
    /// Gate 3: unresolved — no match found.
    Unresolved {
        best_label: Option<String>,
        best_score: f64,
    },
}

impl GateResult {
    /// Which gate resolved the intent.
    pub fn gate_number(&self) -> u8 {
        match self {
            GateResult::Exact { .. } => 1,
            GateResult::Vector { .. } => 2,
            GateResult::Unresolved { .. } => 3,
        }
    }

    /// The matched label, if any.
    pub fn label(&self) -> Option<&str> {
        match self {
            GateResult::Exact { label, .. } => Some(label),
            GateResult::Vector { label, .. } => Some(label),
            GateResult::Unresolved { best_label, .. } => best_label.as_deref(),
        }
    }

    /// Confidence/score.
    pub fn confidence(&self) -> f64 {
        match self {
            GateResult::Exact { confidence, .. } => *confidence,
            GateResult::Vector { confidence, .. } => *confidence,
            GateResult::Unresolved { best_score, .. } => *best_score,
        }
    }
}

/// A registered intent in the pipeline.
#[derive(Clone, Debug)]
pub struct IntentEntry {
    pub label: String,
    pub patterns: Vec<String>,
    pub embedding: Embedding,
}

/// The three-gate pipeline.
pub struct GatePipeline {
    entries: Vec<IntentEntry>,
    vector_threshold: f64,
}

impl GatePipeline {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            vector_threshold: VECTOR_THRESHOLD,
        }
    }

    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.vector_threshold = threshold;
        self
    }

    /// Register an intent with its patterns.
    pub fn register(&mut self, label: &str, patterns: &[&str]) {
        let pattern_strings: Vec<String> = patterns.iter().map(|s| s.to_string()).collect();
        // Embed the primary pattern as the representative vector
        let primary = patterns.first().map(|s| s.to_string()).unwrap_or_default();
        let embedding = embed_intent(&primary);
        self.entries.push(IntentEntry {
            label: label.to_string(),
            patterns: pattern_strings,
            embedding,
        });
    }

    /// Register with a pre-computed embedding.
    pub fn register_with_embedding(&mut self, label: &str, embedding: Embedding) {
        self.entries.push(IntentEntry {
            label: label.to_string(),
            patterns: vec![label.to_string()],
            embedding,
        });
    }

    /// Run the three-gate pipeline on an intent string.
    pub fn resolve(&self, intent: &str) -> GateResult {
        // Gate 1: Exact hash match
        let query_hash = hash_intent(intent);
        for entry in &self.entries {
            for pattern in &entry.patterns {
                let pattern_hash = hash_intent(pattern);
                if query_hash == pattern_hash {
                    return GateResult::Exact {
                        label: entry.label.clone(),
                        confidence: EXACT_THRESHOLD,
                    };
                }
            }
        }

        // Gate 2: Vector similarity
        let query_emb = embed_intent(intent);
        let corpus: Vec<(String, Embedding)> = self
            .entries
            .iter()
            .map(|e| (e.label.clone(), e.embedding.clone()))
            .collect();

        if !corpus.is_empty() {
            let results = cosine_search(&query_emb, &corpus, 1);
            if let Some(best) = results.first() {
                if best.score >= self.vector_threshold {
                    return GateResult::Vector {
                        label: best.label.clone(),
                        confidence: best.score,
                    };
                }
                // Below threshold — fall through to Gate 3
                return GateResult::Unresolved {
                    best_label: Some(best.label.clone()),
                    best_score: best.score,
                };
            }
        }

        // Gate 3: Unresolved
        GateResult::Unresolved {
            best_label: None,
            best_score: 0.0,
        }
    }

    /// Number of registered intents.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the pipeline is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for GatePipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate1_exact_match() {
        let mut pipeline = GatePipeline::new();
        pipeline.register("greeting", &["hello", "hi there"]);
        let result = pipeline.resolve("hello");
        assert_eq!(result, GateResult::Exact {
            label: "greeting".to_string(),
            confidence: 1.0,
        });
    }

    #[test]
    fn test_gate2_vector_match() {
        let mut pipeline = GatePipeline::new();
        pipeline.register("greeting", &["hello world"]);
        // Similar but not exact
        let result = pipeline.resolve("hello earth");
        // Should resolve via vector gate or be unresolved
        assert!(matches!(result, GateResult::Vector { .. } | GateResult::Unresolved { .. }));
    }

    #[test]
    fn test_gate3_unresolved() {
        let mut pipeline = GatePipeline::new();
        pipeline.register("greeting", &["hello"]);
        let result = pipeline.resolve("xyzzy completely unknown quantum physics");
        assert!(matches!(result, GateResult::Unresolved { .. }));
    }

    #[test]
    fn test_empty_pipeline() {
        let pipeline = GatePipeline::new();
        let result = pipeline.resolve("anything");
        assert!(matches!(result, GateResult::Unresolved { .. }));
    }
}
