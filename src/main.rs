mod hittable;
mod material;
mod ray;
mod vec3;

#[macro_use]
extern crate derive_more;

use std::io::BufWriter;
use {
    hittable::{Hittable, HittableVec, Sphere},
    indicatif::ProgressIterator,
    rand::prelude::*,
    ray::Ray,
    std::{
        fs::File,
        io::{self, Write},
    },
    vec3::{Color, Point3, Vec3},
};

#[derive(From, Into)]
struct Width(usize);

#[derive(From, Into)]
struct Height(usize);

fn main() {
    // Image
    const ASPECT_RATIO: f64 = 16.0 / 9.0;
    const IMAGE_WIDTH: Width = Width(400);
    const IMAGE_HEIGHT: Height = Height((IMAGE_WIDTH.0 as f64 / ASPECT_RATIO) as usize);
    const SAMPLES_PER_PIXEL: usize = 100;
    const MAX_DEPTH: usize = 50;

    // World
    let world: Vec<Box<dyn Hittable>> = vec![
        Box::new(Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5)),
        Box::new(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0)),
    ];

    // Camera
    let cam = Camera::new();

    // Render
    let file = File::create("image.ppm").unwrap();
    let mut file = BufWriter::new(file);

    let mut rng = rand::thread_rng();

    writeln!(&mut file, "P3\n{} {}\n255", IMAGE_WIDTH.0, IMAGE_HEIGHT.0).unwrap();

    for j in (0..IMAGE_HEIGHT.0).rev().progress() {
        for i in 0..IMAGE_WIDTH.0 {
            let mut pixel_color = Color::new(0.0, 0.0, 0.0);
            for _ in 0..SAMPLES_PER_PIXEL {
                let u = (i as f64 + rng.gen::<f64>()) / ((IMAGE_WIDTH.0 - 1) as f64);
                let v = (j as f64 + rng.gen::<f64>()) / ((IMAGE_HEIGHT.0 - 1) as f64);
                let r = cam.get_ray(u, v);
                pixel_color += ray_color(&r, &world, &mut rng, MAX_DEPTH);
            }

            write_color(&mut file, pixel_color, SAMPLES_PER_PIXEL).unwrap();
        }
    }
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

fn ray_color(r: &Ray, world: &impl HittableVec, rng: &mut impl Rng, depth: usize) -> Color {
    if depth == 0 {
        return Color::new(0.0, 0.0, 0.0);
    }

    if let Some(hit_record) = world.hit(r, 0.001, f64::INFINITY) {
        let target = hit_record.p + Vec3::random_in_hemisphere(&hit_record.normal, rng); //+ Vec3::random_unit_vector(rng);
        return 0.5
            * ray_color(
                &Ray::new(hit_record.p, target - hit_record.p),
                world,
                rng,
                depth - 1,
            );
    }
    let unit_direction = r.direction().unit_vector();
    let t = 0.5 * (unit_direction.y() + 1.0);
    return (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * Color::new(0.5, 0.7, 1.0);
}

struct Camera {
    origin: Point3,
    lower_left_corner: Point3,
    horizontal: Vec3,
    vertical: Vec3,
}

impl Camera {
    pub fn new() -> Self {
        let aspect_ratio = 16.0 / 9.0;
        let viewport_height = 2.0;
        let viewport_width = aspect_ratio * viewport_height;
        let focal_length = 1.0;

        let origin = Point3::new(0.0, 0.0, 0.0);
        let horizontal = Vec3::new(viewport_width, 0.0, 0.0);
        let vertical = Vec3::new(0.0, viewport_height, 0.0);
        let lower_left_corner =
            origin - horizontal / 2.0 - vertical / 2.0 - Vec3::new(0.0, 0.0, focal_length);

        Self {
            origin,
            lower_left_corner,
            horizontal,
            vertical,
        }
    }

    pub fn get_ray(&self, u: f64, v: f64) -> Ray {
        Ray::new(
            self.origin,
            self.lower_left_corner + u * self.horizontal + v * self.vertical - self.origin,
        )
    }
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
