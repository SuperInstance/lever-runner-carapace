use lever_runner_carapace::hash::hash_intent;

#[test]
fn test_deterministic() {
    let h1 = hash_intent("hello world");
    let h2 = hash_intent("hello world");
    assert_eq!(h1, h2);
}

#[test]
fn test_different_inputs() {
    let h1 = hash_intent("hello");
    let h2 = hash_intent("world");
    assert_ne!(h1, h2);
}

#[test]
fn test_hex_output() {
    let h = hash_intent("test");
    let hex = h.to_hex();
    assert_eq!(hex.len(), 64);
    assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_with_salt() {
    use lever_runner_carapace::hash::hash_intent_with_salt;
    let h1 = hash_intent_with_salt("test", b"salt1");
    let h2 = hash_intent_with_salt("test", b"salt2");
    assert_ne!(h1, h2);
}
