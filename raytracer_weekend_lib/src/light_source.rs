use derive_more::Constructor;

use crate::{
    hittable::HitRecord,
    material::{Material, Scatter},
    ray::Ray,
    texture::{Point2d, Texture},
    vec3::{Color, Point3},
    ActiveRng,
};

#[derive(Constructor, Debug, Clone)]
pub struct DiffuseLight<T: Texture> {
    emit: T,
}

impl<T: Texture> Material for DiffuseLight<T> {
    fn scatter(&self, _r_in: &Ray, _rec: &HitRecord, _rng: &mut ActiveRng) -> Option<Scatter> {
        None
    }

    fn emitted(&self, uv: Point2d, p: &Point3) -> Color {
        self.emit.value(uv, p)
    }
}
