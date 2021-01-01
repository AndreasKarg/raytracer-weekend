mod aabb;
mod bvh;
mod camera;
mod hittable;
mod image_texture;
mod light_source;
mod material;
mod perlin;
mod ray;
mod scenes;
mod texture;
pub mod vec3;

use bvh::BvhNode;
use camera::Camera;
use derive_more::{From, Into};
use hittable::Hittable;
use itertools::iproduct;
use material::Lambertian;
use rand::prelude::*;
use ray::Ray;
use rayon::prelude::*;
pub use scenes::Scene;
use vec3::Color;

#[derive(From, Into)]
struct Width(usize);

#[derive(From, Into)]
struct Height(usize);

const MAX_DEPTH: usize = 50;

pub fn render(
    world: Vec<Box<dyn Hittable>>,
    cam: Camera,
    background: Color,
    image_width: usize,
    image_height: usize,
    samples_per_pixel: usize,
) -> impl ParallelIterator<Item = Color> {
    let mut rng = rand::thread_rng();

    // World
    let world = BvhNode::new(world, 0.0, 1.0, &mut rng);

    // Render
    let pixel_range: Vec<_> = iproduct!((0..image_height).rev(), 0..image_width).collect();

    pixel_range.into_par_iter().map(move |(j, i)| {
        sample_pixel(
            &world,
            &cam,
            background,
            j,
            i,
            image_width,
            image_height,
            samples_per_pixel,
        )
    })
}

fn sample_pixel(
    world: &dyn Hittable,
    cam: &Camera,
    background: Color,
    pixel_row: usize,
    pixel_column: usize,
    image_width: usize,
    image_height: usize,
    samples_per_pixel: usize,
) -> Color {
    let mut rng = thread_rng();
    let mut pixel_color = Color::new(0.0, 0.0, 0.0);
    for _ in 0..samples_per_pixel {
        let u = (pixel_column as f64 + rng.gen::<f64>()) / ((image_width - 1) as f64);
        let v = (pixel_row as f64 + rng.gen::<f64>()) / ((image_height - 1) as f64);
        let r = cam.get_ray(u, v, &mut rng);
        pixel_color += sample_ray(&r, world, &mut rng, MAX_DEPTH, background);
    }

    pixel_color
}

fn sample_ray(
    r: &Ray,
    world: &dyn Hittable,
    rng: &mut ThreadRng,
    depth: usize,
    background: Color,
) -> Color {
    if depth == 0 {
        return Color::new(0.0, 0.0, 0.0);
    }

    let hit_record = match world.hit(r, 0.001, f64::INFINITY, rng) {
        Some(hit) => hit,
        _ => return background,
    };

    let emitted = hit_record
        .material
        .emitted(hit_record.texture_uv, &hit_record.p);

    let scatter = match hit_record.material.scatter(r, &hit_record, rng) {
        Some(scatter) => scatter,
        _ => return emitted,
    };

    emitted
        + scatter.attenuation
            * sample_ray(&scatter.scattered_ray, world, rng, depth - 1, background)
}
