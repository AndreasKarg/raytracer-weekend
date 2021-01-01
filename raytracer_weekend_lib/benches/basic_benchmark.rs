use criterion::{criterion_group, criterion_main, Criterion};
use rand::thread_rng;
use rayon::iter::ParallelIterator;
use raytracer_weekend_lib::{Raytracer, Scene};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("book2_final_scene", |b| {
        b.iter(|| {
            let (world, cam, background) =
                Scene::Book2FinalScene.generate(16.0 / 9.0, &mut thread_rng());
            let raytracer = Raytracer::new(world, cam, background, 400, 225, 100);
            raytracer.render().collect::<Vec<_>>();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
