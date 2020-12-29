use std::fmt::Debug;

use derive_more::Constructor;

use super::vec3::{Color, Vec3};
use crate::perlin::Perlin;

#[derive(Debug, Copy, Clone)]
pub struct Point2d {
    pub u: f64,
    pub v: f64,
}

pub trait Texture: Debug + Send + Sync {
    fn value(&self, uv: Point2d, p: &Vec3) -> Color;
}

#[derive(Debug, Constructor, Clone)]
pub struct SolidColor {
    color_value: Color,
}

impl SolidColor {
    pub fn new_rgb(red: f64, green: f64, blue: f64) -> Self {
        Self::new(Color::new(red, green, blue))
    }
}

impl Texture for SolidColor {
    fn value(&self, _uv: Point2d, _p: &Vec3) -> Color {
        self.color_value
    }
}

#[derive(Debug, Constructor, Clone)]
pub struct Checker<E: Texture, O: Texture> {
    odd: O,
    even: E,
    frequency: f64,
}

impl<E: Texture, O: Texture> Texture for Checker<E, O> {
    fn value(&self, uv: Point2d, p: &Vec3) -> Color {
        let sines = (self.frequency * p.x()).sin()
            * (self.frequency * p.y()).sin()
            * (self.frequency * p.z()).sin();

        if sines < 0.0 {
            self.odd.value(uv, p)
        } else {
            self.even.value(uv, p)
        }
    }
}

#[derive(Debug, Constructor, Clone)]
pub struct Noise {
    noise: Perlin,
    scale: f64,
}

impl Texture for Noise {
    fn value(&self, _uv: Point2d, p: &Vec3) -> Color {
        Color::new(1.0, 1.0, 1.0) * 0.5 * (1.0 +self.noise.noise(&(*p * self.scale)))
    }
}
