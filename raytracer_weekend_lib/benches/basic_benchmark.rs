use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::thread_rng;
use rayon::iter::ParallelIterator;
use raytracer_weekend_lib::Scene;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("book2_final_scene", |b| {
        b.iter(|| {
            let (world, cam, background) =
                Scene::Book2FinalScene.generate(16.0 / 9.0, &mut thread_rng());
            raytracer_weekend_lib::render(world, cam, background, 400, 225, 100)
                .collect::<Vec<_>>();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
