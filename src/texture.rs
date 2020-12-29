use std::fmt::Debug;

use derive_more::Constructor;

use super::vec3::{Color, Vec3};

#[derive(Debug, Copy, Clone)]
pub struct Point2d {
    pub u: f64,
    pub v: f64,
}

pub trait Texture: Debug + Send + Sync {
    fn value(&self, uv: Point2d, p: &Vec3) -> Color;
}

#[derive(Debug, Constructor)]
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
