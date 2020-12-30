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

use std::{
    fs::File,
    io::{self, BufWriter, Write},
};

use bvh::BvhNode;
use camera::Camera;
use derive_more::{From, Into};
use hittable::{Hittable, HittableVec};
use itertools::iproduct;
use material::Lambertian;
use rand::prelude::*;
use ray::Ray;
use rayon::prelude::*;
use vec3::{Color, Vec3};

#[derive(From, Into)]
struct Width(usize);

#[derive(From, Into)]
struct Height(usize);

const MAX_DEPTH: usize = 50;

pub fn render(
    image_width: usize,
    image_height: usize,
    samples_per_pixel: usize,
) -> impl ParallelIterator<Item = Color> {
    let mut rng = rand::thread_rng();
    let aspect_ratio = image_width as f64 / image_height as f64;

    // World
    // let (world, cam, background) = scenes::jumpy_balls(aspect_ratio, &mut rng);
    // let (world, cam, background) = scenes::two_spheres(aspect_ratio, &mut rng);
    // let (world, cam) = scenes::two_perlin_spheres(aspect_ratio, &mut rng);
    // let (world, cam, background) = scenes::earth(aspect_ratio, &mut rng);
    // let (world, cam, background) = scenes::simple_light(aspect_ratio, &mut rng);
    let (world, cam, background) = scenes::cornell_box(aspect_ratio, &mut rng);
    let world = bvh::BvhNode::new(world, 0.0, 1.0, &mut rng);

    // Render
    let pixel_range: Vec<_> = iproduct!((0..image_height).rev(), 0..image_width).collect();
    let all_pixels = pixel_range.into_par_iter().map(move |(j, i)| {
        evaluate_pixel(
            &world,
            &cam,
            background,
            j,
            i,
            image_width,
            image_height,
            samples_per_pixel,
        )
    });

    all_pixels
}

fn evaluate_pixel(
    world: &BvhNode,
    cam: &Camera,
    background: Color,
    pixel_row: usize,
    pixel_column: usize,
    image_width: usize,
    image_height: usize,
    samples_per_pixel: usize,
) -> Vec3 {
    let mut rng = thread_rng();
    let mut pixel_color = Color::new(0.0, 0.0, 0.0);
    for _ in 0..samples_per_pixel {
        let u = (pixel_column as f64 + rng.gen::<f64>()) / ((image_width - 1) as f64);
        let v = (pixel_row as f64 + rng.gen::<f64>()) / ((image_height - 1) as f64);
        let r = cam.get_ray(u, v, &mut rng);
        pixel_color += ray_color(&r, world, &mut rng, MAX_DEPTH, background);
    }

    pixel_color
}

fn ray_color(
    r: &Ray,
    world: &BvhNode,
    rng: &mut ThreadRng,
    depth: usize,
    background: Color,
) -> Color {
    if depth == 0 {
        return Color::new(0.0, 0.0, 0.0);
    }

    let hit_record = match world.hit(r, 0.001, f64::INFINITY) {
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
        + scatter.attenuation * ray_color(&scatter.scattered_ray, world, rng, depth - 1, background)
}

fn clamp(x: f64, min: f64, max: f64) -> f64 {
    if x < min {
        min
    } else if x > max {
        max
    } else {
        x
    }
}
