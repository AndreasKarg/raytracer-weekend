use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("basic_main", |b| b.iter(raytracer_weekend_lib::render));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
