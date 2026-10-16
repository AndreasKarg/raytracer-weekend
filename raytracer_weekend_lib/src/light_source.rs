use core::fmt::Debug;
use derive_new::new;
use rand::Rng;

use crate::{
    hittable::HitRecord,
    material::{Material, Scatter},
    ray::Ray,
    texture::{Point2d, Texture},
    vec3::{Color, Point3},
};

#[derive(Debug, Clone, new)]
pub struct DiffuseLight<T: Texture> {
    emit: T,
}

impl<R: Rng + Debug + ?Sized, T: Texture> Material<R> for DiffuseLight<T> {
    fn scatter(&self, _r_in: &Ray, _rec: &HitRecord<R>, _rng: &mut R) -> Option<Scatter> {
        None
    }

    fn emitted(&self, uv: Point2d, p: &Point3) -> Color {
        self.emit.value(uv, p)
    }
}
