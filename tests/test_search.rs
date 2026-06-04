use lever_runner_carapace::embed::embed_intent;
use lever_runner_carapace::search::cosine_search;

#[test]
fn test_basic_search() {
    let query = embed_intent("greeting");
    let corpus = vec![
        ("hello".to_string(), embed_intent("hello")),
        ("weather".to_string(), embed_intent("weather")),
        ("goodbye".to_string(), embed_intent("goodbye")),
    ];
    let results = cosine_search(&query, &corpus, 2);
    assert_eq!(results.len(), 2);
    assert!(results[0].score >= results[1].score);
}

#[test]
fn test_top_1() {
    let query = embed_intent("hello");
    let corpus = vec![
        ("a".to_string(), embed_intent("hello")),
        ("b".to_string(), embed_intent("xyz")),
    ];
    let results = cosine_search(&query, &corpus, 1);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].label, "a");
}

#[test]
fn test_k_larger_than_corpus() {
    let query = embed_intent("test");
    let corpus = vec![("only".to_string(), embed_intent("one"))];
    let results = cosine_search(&query, &corpus, 10);
    assert_eq!(results.len(), 1);
}
