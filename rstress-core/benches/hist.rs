use criterion::{criterion_group, criterion_main, Criterion};
use rstress_core::metrics::metrics::Metrics;

fn bench_hist(c: &mut Criterion) {
    c.bench_function("record 1k latencies", |b| {
        b.iter(|| {
            let mut m = Metrics::new();
            for _ in 0..1000 {
                m.record(true, 50_000, Some(200));
            }
        })
    });
}

criterion_group!(benches, bench_hist);
criterion_main!(benches);
