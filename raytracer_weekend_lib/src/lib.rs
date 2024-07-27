#![feature(trait_alias)]
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
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
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
    image_width: u32,
    image_height: u32,
    samples_per_pixel: u32,
}

#[cfg(feature = "rayon")]
pub trait RenderIterator = ParallelIterator<Item=Pixel>;

#[cfg(not(feature = "rayon"))]
pub trait RenderIterator = Iterator<Item=Pixel>;

impl<'a> Raytracer<'a> {
    pub fn render(&self) -> impl RenderIterator + '_ {
        let pixel_range = iproduct!((0..self.image_height).rev(), 0..self.image_width);

        #[cfg(feature = "rayon")]
        {
            let pixel_range: Vec<_> = pixel_range.collect();
            pixel_range.into_par_iter().map(move |(j, i)| {
                let mut rng = thread_rng();
                self.sample_pixel(j, i, &mut rng)
            })
        }

        #[cfg(not(feature = "rayon"))]
        {
            let mut rng = SmallRng::seed_from_u64(0xb234e6fea3886a1e);
            pixel_range
                .into_iter()
                .map(move |(j, i)| self.sample_pixel(j, i, &mut rng))
        }
    }

    fn sample_pixel(&self, pixel_row: u32, pixel_column: u32, rng: &mut ActiveRng) -> Pixel {
        let image_width = self.image_width;
        let image_height = self.image_height;

        let mut pixel_color = Color::new(0.0, 0.0, 0.0);
        for _ in 0..self.samples_per_pixel {
            let u = (pixel_column as f32 + rng.gen::<f32>()) / ((image_width - 1) as f32);
            let v = (pixel_row as f32 + rng.gen::<f32>()) / ((image_height - 1) as f32);
            let r = self.cam.get_ray(u, v, rng);
            pixel_color += self.sample_ray(&r, rng, MAX_DEPTH);
        }

        Pixel {
            row: pixel_row,
            column: pixel_column,
            color: pixel_color,
        }
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

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Pixel {
    pub row: u32,
    pub column: u32,
    pub color: Color,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ProgressMessage {
    ImageStart {
        width: u32,
        height: u32,
        samples_per_pixel: u32,
    },
    Pixel(Pixel),
    ImageEnd,
}
