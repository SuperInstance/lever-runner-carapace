use lever_runner_carapace::gate::{GatePipeline, GateResult};

#[test]
fn test_exact_match() {
    let mut p = GatePipeline::new();
    p.register("greet", &["hello", "hi"]);
    let r = p.resolve("hello");
    assert!(matches!(r, GateResult::Exact { .. }));
    assert_eq!(r.label(), Some("greet"));
}

#[test]
fn test_unresolved() {
    let mut p = GatePipeline::new();
    p.register("greet", &["hello"]);
    let r = p.resolve("completely unrelated quantum physics equation");
    assert!(matches!(r, GateResult::Unresolved { .. }));
}

#[test]
fn test_gate_numbers() {
    let mut p = GatePipeline::new();
    p.register("test", &["exact_pattern"]);
    let r1 = p.resolve("exact_pattern");
    assert_eq!(r1.gate_number(), 1);

    let r2 = p.resolve("something else entirely");
    assert!(r2.gate_number() >= 2);
}

#[test]
fn test_multiple_entries() {
    let mut p = GatePipeline::new();
    p.register("greet", &["hello"]);
    p.register("weather", &["what's the weather"]);
    p.register("time", &["what time is it"]);
    assert_eq!(p.len(), 3);

    assert_eq!(p.resolve("hello").label(), Some("greet"));
    assert_eq!(p.resolve("what's the weather").label(), Some("weather"));
}
