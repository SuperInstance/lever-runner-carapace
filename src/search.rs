//! Brute-force cosine similarity search with SIMD-optimized dot products.

use crate::embed::Embedding;

/// A search result entry.
#[derive(Clone, Debug)]
pub struct SearchResult {
    pub index: usize,
    pub score: f64,
    pub label: String,
}

/// Search embedding space for top-K nearest neighbors by cosine similarity.
pub fn cosine_search(
    query: &Embedding,
    corpus: &[(String, Embedding)],
    top_k: usize,
) -> Vec<SearchResult> {
    let k = top_k.min(corpus.len());
    if k == 0 {
        return Vec::new();
    }

    let mut results: Vec<SearchResult> = corpus
        .iter()
        .enumerate()
        .map(|(i, (label, emb))| SearchResult {
            index: i,
            score: query.cosine_similarity(emb),
            label: label.clone(),
        })
        .collect();

    // Partial sort: keep top-K
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(k);
    results
}

/// SIMD-optimized batch dot product.
/// Uses portable SIMD when available, falls back to scalar.
#[inline]
fn dot_product_simd(a: &[f32], b: &[f32]) -> f64 {
    assert_eq!(a.len(), b.len());
    // Scalar fallback — portable SIMD is nightly-only; use auto-vectorization
    let mut sum: f64 = 0.0;
    let chunks = a.len() / 4;
    let mut acc = [0.0f64; 4];
    for i in 0..chunks {
        let base = i * 4;
        acc[0] += a[base] as f64 * b[base] as f64;
        acc[1] += a[base + 1] as f64 * b[base + 1] as f64;
        acc[2] += a[base + 2] as f64 * b[base + 2] as f64;
        acc[3] += a[base + 3] as f64 * b[base + 3] as f64;
    }
    for v in acc {
        sum += v;
    }
    // Remainder
    for i in (chunks * 4)..a.len() {
        sum += a[i] as f64 * b[i] as f64;
    }
    sum
}

/// Fast cosine similarity using SIMD dot product.
pub fn fast_cosine(a: &Embedding, b: &Embedding) -> f64 {
    assert_eq!(a.dim, b.dim);
    let dot = dot_product_simd(&a.data, &b.data);
    let n1 = a.norm();
    let n2 = b.norm();
    if n1 == 0.0 || n2 == 0.0 {
        return 0.0;
    }
    dot / (n1 * n2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embed::embed_intent;

    #[test]
    fn test_search_returns_top_k() {
        let query = embed_intent("greeting");
        let corpus = vec![
            ("hello".to_string(), embed_intent("hello")),
            ("weather".to_string(), embed_intent("weather forecast")),
            ("hi there".to_string(), embed_intent("hi there greeting")),
            ("goodbye".to_string(), embed_intent("goodbye")),
        ];
        let results = cosine_search(&query, &corpus, 2);
        assert_eq!(results.len(), 2);
        assert!(results[0].score >= results[1].score);
    }

    #[test]
    fn test_search_empty_corpus() {
        let query = embed_intent("test");
        let results = cosine_search(&query, &[], 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_fast_cosine_matches_regular() {
        let a = embed_intent("hello world");
        let b = embed_intent("hello there");
        let regular = a.cosine_similarity(&b);
        let fast = fast_cosine(&a, &b);
        assert!((regular - fast).abs() < 0.001);
    }
}
