use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lever_runner_carapace::{
    embed_intent, hash_intent, GatePipeline,
};
use lever_runner_carapace::embed::embed_intent_with_dim;
use lever_runner_carapace::search::cosine_search;

fn bench_hash(c: &mut Criterion) {
    let intent = "what is the weather in anchorage alaska today";
    c.bench_function("hash_intent", |b| {
        b.iter(|| hash_intent(black_box(intent)))
    });
}

fn bench_embed(c: &mut Criterion) {
    let intent = "what is the weather in anchorage alaska today";
    c.bench_function("embed_intent_64dim", |b| {
        b.iter(|| embed_intent(black_box(intent)))
    });

    c.bench_function("embed_intent_128dim", |b| {
        b.iter(|| embed_intent_with_dim(black_box(intent), 128))
    });
}

fn bench_search(c: &mut Criterion) {
    // Build a corpus of 10K embeddings
    let corpus: Vec<(String, _)> = (0..10_000)
        .map(|i| {
            let label = format!("intent_{}", i);
            let emb = embed_intent(&label);
            (label, emb)
        })
        .collect();

    let query = embed_intent("find something similar to intent_5000");

    c.bench_function("search_10k_top5", |b| {
        b.iter(|| cosine_search(black_box(&query), black_box(&corpus), 5))
    });
}

fn bench_gate_pipeline(c: &mut Criterion) {
    let mut pipeline = GatePipeline::new();
    for i in 0..1000 {
        let label = format!("intent_{}", i);
        let patterns = [format!("pattern_{}", i)];
        pipeline.register(&label, &[&patterns[0]]);
    }

    // Exact match
    c.bench_function("gate_exact_match", |b| {
        b.iter(|| pipeline.resolve(black_box("pattern_500")))
    });

    // Vector search (no exact match)
    c.bench_function("gate_vector_search", |b| {
        b.iter(|| pipeline.resolve(black_box("something completely different")))
    });
}

criterion_group!(benches, bench_hash, bench_embed, bench_search, bench_gate_pipeline);
criterion_main!(benches);
