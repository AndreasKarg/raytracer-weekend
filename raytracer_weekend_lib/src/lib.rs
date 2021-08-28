#![feature(array_map, array_zip, trait_alias)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod aabb;
pub mod bvh;
pub mod camera;
pub mod hittable;
pub mod image_texture;
pub mod light_source;
pub mod material;
pub mod perlin;
mod ray;
pub mod texture;
pub mod vec3;

use alloc::{boxed::Box, vec::Vec};

use camera::Camera;
use derive_more::Constructor;
use hittable::Hittable;
use itertools::iproduct;
use rand::prelude::*;
use ray::Ray;
#[cfg(feature = "rayon")]
use rayon::prelude::*;
use vec3::Color;

const MAX_DEPTH: usize = 50;

#[cfg(feature = "std")]
type ActiveRng = ThreadRng;

#[cfg(not(feature = "std"))]
type ActiveRng = SmallRng;

#[derive(Constructor)]
pub struct Raytracer<'a> {
    world: &'a [Box<dyn Hittable>],
    cam: &'a Camera,
    background: Color,
    image_width: usize,
    image_height: usize,
    samples_per_pixel: usize,
}

#[cfg(feature = "rayon")]
trait RenderIterator = ParallelIterator<Item = Color>;

#[cfg(not(feature = "rayon"))]
trait RenderIterator = Iterator<Item = Color>;

impl<'a> Raytracer<'a> {
    pub fn render(&self) -> impl RenderIterator + '_ {
        let pixel_range: Vec<_> =
            iproduct!((0..self.image_height).rev(), 0..self.image_width).collect();

        let mut rng;

        #[cfg(feature = "std")]
        {
            rng = thread_rng()
        };

        #[cfg(not(feature = "std"))]
        {
            rng = SmallRng::seed_from_u64(0xb234e6fea3886a1e);
        }

        #[cfg(feature = "rayon")]
        {
            pixel_range
                .into_par_iter()
                .map(move |(j, i)| self.sample_pixel(j, i, &mut rng))
        }

        #[cfg(not(feature = "rayon"))]
        {
            pixel_range
                .into_iter()
                .map(move |(j, i)| self.sample_pixel(j, i, &mut rng))
        }
    }

    fn sample_pixel(&self, pixel_row: usize, pixel_column: usize, rng: &mut ActiveRng) -> Color {
        let image_width = self.image_width;
        let image_height = self.image_height;

        let mut pixel_color = Color::new(0.0, 0.0, 0.0);
        for _ in 0..self.samples_per_pixel {
            let u = (pixel_column as f32 + rng.gen::<f32>()) / ((image_width - 1) as f32);
            let v = (pixel_row as f32 + rng.gen::<f32>()) / ((image_height - 1) as f32);
            let r = self.cam.get_ray(u, v, rng);
            pixel_color += self.sample_ray(&r, rng, MAX_DEPTH);
        }

        pixel_color
    }

    fn sample_ray(&self, r: &Ray, rng: &mut ActiveRng, depth: usize) -> Color {
        if depth == 0 {
            return Color::new(0.0, 0.0, 0.0);
        }

        let hit_record = match self.world.hit(r, 0.001, f32::INFINITY, rng) {
            Some(hit) => hit,
            _ => return self.background,
        };

        let emitted = hit_record
            .material
            .emitted(hit_record.texture_uv, &hit_record.p);

        let scatter = match hit_record.material.scatter(r, &hit_record, rng) {
            Some(scatter) => scatter,
            _ => return emitted,
        };

        emitted + scatter.attenuation * self.sample_ray(&scatter.scattered_ray, rng, depth - 1)
    }
}
