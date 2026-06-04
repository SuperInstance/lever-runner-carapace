//! Position-aware embedding: hash each word, XOR-fold into fixed-dim vector.
//! Pure math, no ML.

use crate::hash::hash_intent;

/// Default embedding dimensionality.
pub const EMBED_DIM: usize = 64;

/// A fixed-dimension embedding vector (f32).
#[derive(Clone, Debug, PartialEq)]
pub struct Embedding {
    pub dim: usize,
    pub data: Vec<f32>,
}

impl Embedding {
    pub fn zero(dim: usize) -> Self {
        Self {
            dim,
            data: vec![0.0; dim],
        }
    }

    /// L2 norm of the vector.
    pub fn norm(&self) -> f64 {
        let sum: f64 = self.data.iter().map(|&v| (v as f64) * (v as f64)).sum();
        sum.sqrt()
    }

    /// Normalize to unit length.
    pub fn normalize(&mut self) {
        let n = self.norm();
        if n > 0.0 {
            for v in &mut self.data {
                *v /= n as f32;
            }
        }
    }

    /// Cosine similarity between two embeddings.
    pub fn cosine_similarity(&self, other: &Embedding) -> f64 {
        let dot: f64 = self
            .data
            .iter()
            .zip(other.data.iter())
            .map(|(&a, &b)| (a as f64) * (b as f64))
            .sum();
        let n1 = self.norm();
        let n2 = other.norm();
        if n1 == 0.0 || n2 == 0.0 {
            return 0.0;
        }
        dot / (n1 * n2)
    }
}

/// Embed an intent string into a fixed-dim vector.
///
/// Strategy: split into words, hash each word with its position index,
/// use hash bytes to XOR-fold into the vector dimensions.
pub fn embed_intent(intent: &str) -> Embedding {
    embed_intent_with_dim(intent, EMBED_DIM)
}

/// Embed with custom dimensionality.
pub fn embed_intent_with_dim(intent: &str, dim: usize) -> Embedding {
    let mut emb = Embedding::zero(dim);
    let words: Vec<&str> = intent
        .split_whitespace()
        .filter(|w| !w.is_empty())
        .collect();

    if words.is_empty() {
        return emb;
    }

    for (pos, word) in words.iter().enumerate() {
        // Hash includes position for position-awareness
        let salted = format!("{}:{}", pos, word);
        let h = hash_intent(&salted);
        let hash_bytes = h.as_bytes();

        // XOR-fold hash bytes into the embedding dimensions
        for i in 0..dim {
            let hash_idx = (i * 4) % 32;
            let hash_val = u32::from_le_bytes([
                hash_bytes[hash_idx],
                hash_bytes[hash_idx + 1],
                hash_bytes[(hash_idx + 2) % 32],
                hash_bytes[(hash_idx + 3) % 32],
            ]);
            // Convert to f32 in [-1, 1] range
            // XOR-fold: mix with existing value
            emb.data[i] = f32::from_bits(emb.data[i].to_bits() ^ hash_val);
        }
    }

    emb.normalize();
    emb
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic() {
        let e1 = embed_intent("hello world");
        let e2 = embed_intent("hello world");
        assert_eq!(e1, e2);
    }

    #[test]
    fn test_position_matters() {
        let e1 = embed_intent("hello world");
        let e2 = embed_intent("world hello");
        assert_ne!(e1, e2);
    }

    #[test]
    fn test_unit_norm() {
        let e = embed_intent("test embedding here");
        let n = e.norm();
        assert!((n - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_empty_intent() {
        let e = embed_intent("");
        assert_eq!(e.norm(), 0.0);
    }

    #[test]
    fn test_similarity_same() {
        let e = embed_intent("test");
        let sim = e.cosine_similarity(&e);
        assert!((sim - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_similarity_different() {
        let e1 = embed_intent("hello world");
        let e2 = embed_intent("completely different topic xyz");
        let sim = e1.cosine_similarity(&e2);
        assert!(sim < 0.95); // different intents should differ
    }
}
