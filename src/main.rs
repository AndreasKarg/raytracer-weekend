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
mod vec3;

use std::{
    fs::File,
    io::{self, BufWriter, Write},
};

use camera::Camera;
use derive_more::{From, Into};
use hittable::{Hittable, HittableVec};
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
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

const ASPECT_RATIO: f64 = 16.0 / 9.0;
const IMAGE_WIDTH: Width = Width(400);
const IMAGE_HEIGHT: Height = Height((IMAGE_WIDTH.0 as f64 / ASPECT_RATIO) as usize);
const SAMPLES_PER_PIXEL: usize = 100;
const MAX_DEPTH: usize = 50;
const BACKGROUND_COLOR: Color = Color::new(0.0, 0.0, 0.0);

fn main() {
    let mut rng = rand::thread_rng();

    // World
    // let (world, cam) = scenes::jumpy_balls(ASPECT_RATIO, &mut rng);
    // let (world, cam) = scenes::two_spheres(ASPECT_RATIO, &mut rng);
    // let (world, cam) = scenes::two_perlin_spheres(ASPECT_RATIO, &mut rng);
    // let (world, cam, background) = scenes::earth(ASPECT_RATIO, &mut rng);
    let (world, cam, background) = scenes::simple_light(ASPECT_RATIO, &mut rng);

    // Render
    let file = File::create("image.ppm").unwrap();
    let mut file = BufWriter::new(file);

    writeln!(&mut file, "P3\n{} {}\n255", IMAGE_WIDTH.0, IMAGE_HEIGHT.0).unwrap();

    let pixel_count = (IMAGE_HEIGHT.0 * IMAGE_WIDTH.0) as u64;
    let progress_bar = ProgressBar::new(pixel_count);
    progress_bar.set_style(
        ProgressStyle::default_bar().template(
            "[{elapsed_precise} / {eta_precise}] {wide_bar} {pos:>7}/{len:7} ({per_sec})",
        ),
    );

    progress_bar.set_draw_delta(pixel_count / 100);

    let pixel_range: Vec<_> = iproduct!((0..IMAGE_HEIGHT.0).rev(), 0..IMAGE_WIDTH.0).collect();
    let all_pixels = pixel_range
        .into_par_iter()
        .progress_with(progress_bar)
        .map(|(j, i)| evaluate_pixel(&world, &cam, background, j, i));

    let all_pixels: Vec<_> = all_pixels.collect();
    all_pixels
        .into_iter()
        .for_each(|pixel| write_color(&mut file, pixel, SAMPLES_PER_PIXEL).unwrap());
}

fn evaluate_pixel(
    world: &[Box<dyn Hittable>],
    cam: &Camera,
    background: Color,
    pixel_row: usize,
    pixel_column: usize,
) -> Vec3 {
    let mut rng = thread_rng();
    let mut pixel_color = Color::new(0.0, 0.0, 0.0);
    for _ in 0..SAMPLES_PER_PIXEL {
        let u = (pixel_column as f64 + rng.gen::<f64>()) / ((IMAGE_WIDTH.0 - 1) as f64);
        let v = (pixel_row as f64 + rng.gen::<f64>()) / ((IMAGE_HEIGHT.0 - 1) as f64);
        let r = cam.get_ray(u, v, &mut rng);
        pixel_color += ray_color(&r, world, &mut rng, MAX_DEPTH, background);
    }

    pixel_color
}

fn write_color<F: Write>(f: &mut F, pixel_color: Vec3, samples_per_pixel: usize) -> io::Result<()> {
    let r = pixel_color.x();
    let g = pixel_color.y();
    let b = pixel_color.z();

    // Divide the color by the number of samples and gamma-correct for gamma=2.0.
    let scale = 1.0 / samples_per_pixel as f64;
    let r = (scale * r).sqrt();
    let g = (scale * g).sqrt();
    let b = (scale * b).sqrt();

    let ir = (255.999 * clamp(r, 0.0, 0.999)) as u8;
    let ig = (255.999 * clamp(g, 0.0, 0.999)) as u8;
    let ib = (255.999 * clamp(b, 0.0, 0.999)) as u8;

    writeln!(f, "{} {} {}", ir, ig, ib)
}

fn ray_color(
    r: &Ray,
    world: &(impl HittableVec + ?Sized),
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

    return emitted
        + scatter.attenuation
            * ray_color(&scatter.scattered_ray, world, rng, depth - 1, background);
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
