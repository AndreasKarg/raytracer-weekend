use core::{
    fmt::Debug,
    ops::{Add, Mul},
};

use derive_more::Constructor;
use dyn_clone::{clone_trait_object, DynClone};
#[cfg(feature = "no_std")]
use micromath::F32Ext;

use super::vec3::{Color, Vec3};
use crate::perlin::Perlin;

#[derive(Debug, Copy, Clone)]
pub struct Point2d {
    pub u: f32,
    pub v: f32,
}

impl Mul<Point2d> for f32 {
    type Output = Point2d;

    fn mul(self, rhs: Point2d) -> Self::Output {
        Point2d {
            u: self * rhs.u,
            v: self * rhs.v,
        }
    }
}

impl Add for Point2d {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            u: self.u + rhs.u,
            v: self.v + rhs.v,
        }
    }
}

pub trait Texture: Debug + Send + Sync + DynClone {
    fn value(&self, uv: Point2d, p: &Vec3) -> Color;
}

clone_trait_object!(Texture);

impl Texture for Box<dyn Texture> {
    fn value(&self, uv: Point2d, p: &Vec3) -> Color {
        self.as_ref().value(uv, p)
    }
}

#[derive(Debug, Constructor, Clone)]
pub struct SolidColor {
    color_value: Color,
}

impl SolidColor {
    pub fn new_rgb(red: f32, green: f32, blue: f32) -> Self {
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
    frequency: f32,
}

impl<E: Texture + Clone, O: Texture + Clone> Texture for Checker<E, O> {
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
    scale: f32,
}

impl Texture for Noise {
    fn value(&self, _uv: Point2d, p: &Vec3) -> Color {
        Color::new(1.0, 1.0, 1.0)
            * 0.5
            * (1.0 + (self.scale * p.z() + 10.0 * self.noise.turbulence(&(*p), 7)).sin())
    }
}

#[derive(Debug, Clone, Constructor)]
pub struct UVDebug {}

impl Texture for UVDebug {
    fn value(&self, uv: Point2d, _p: &Vec3) -> Color {
        Color::new(uv.u, uv.v, 0.0)
    }
}
