use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rayon::iter::ParallelIterator;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("basic_main", |b| {
        b.iter(|| {
            raytracer_weekend_lib::render(400, 225, 100).collect::<Vec<_>>();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
