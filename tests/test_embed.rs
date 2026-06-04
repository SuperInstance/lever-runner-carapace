use lever_runner_carapace::embed::{embed_intent, embed_intent_with_dim, EMBED_DIM};

#[test]
fn test_default_dim() {
    let e = embed_intent("hello");
    assert_eq!(e.dim, EMBED_DIM);
    assert_eq!(e.data.len(), EMBED_DIM);
}

#[test]
fn test_custom_dim() {
    let e = embed_intent_with_dim("hello", 128);
    assert_eq!(e.dim, 128);
}

#[test]
fn test_normalized() {
    let e = embed_intent("test string here");
    assert!((e.norm() - 1.0).abs() < 0.01);
}

#[test]
fn test_cosine_similarity_bounds() {
    let e1 = embed_intent("a");
    let e2 = embed_intent("b");
    let sim = e1.cosine_similarity(&e2);
    assert!(sim >= -1.0 && sim <= 1.0);
}
