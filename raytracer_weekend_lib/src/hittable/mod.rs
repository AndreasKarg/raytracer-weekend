use std::fmt::Debug;

use derive_more::Constructor;
use rand::prelude::ThreadRng;

use super::{
    aabb::Aabb,
    material::Material,
    ray::Ray,
    texture::Point2d,
    vec3::{Point3, Vec3},
};

pub mod rectangular;
pub mod spherical;
pub mod transformations;
pub mod triangular;
pub mod volumes;

#[derive(Debug, Constructor)]
pub struct HitRecord<'a> {
    pub p: Point3,
    pub normal: Vec3,
    pub material: &'a (dyn Material + 'a),
    pub t: f64,
    pub texture_uv: Point2d,
    pub is_front_face: bool,
}

impl<'a> HitRecord<'a> {
    pub fn new_with_face_normal(
        p: Point3,
        t: f64,
        texture_uv: Point2d,
        material: &'a (dyn Material + 'a),
        ray: &Ray,
        outward_normal: Vec3,
    ) -> Self {
        let is_front_face = ray.direction().dot(&outward_normal) < 0.0;

        let normal = match is_front_face {
            true => outward_normal,
            false => -outward_normal,
        };

        Self::new(p, normal, material, t, texture_uv, is_front_face)
    }
}

pub trait Hittable: Sync + Send + Debug {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64, rng: &mut ThreadRng) -> Option<HitRecord>;
    fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb>;
}

impl Hittable for [Box<dyn Hittable>] {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64, rng: &mut ThreadRng) -> Option<HitRecord> {
        let mut closest_so_far = t_max;
        let mut rec = None;

        for object in self.iter() {
            if let Some(temp_rec) = object.hit(r, t_min, closest_so_far, rng) {
                closest_so_far = temp_rec.t;
                rec = Some(temp_rec);
            }
        }

        rec
    }

    fn bounding_box(&self, t0: f64, t1: f64) -> Option<Aabb> {
        if self.is_empty() {
            return None;
        }

        let mut output_box = None;

        for object in self.iter() {
            let temp_box = object.bounding_box(t0, t1)?;
            output_box = match output_box {
                None => Some(temp_box),
                Some(bounding_box) => Some(Aabb::surrounding_box(&bounding_box, &temp_box)),
            };
        }

        output_box
    }
}

impl Hittable for Vec<Box<dyn Hittable>> {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64, rng: &mut ThreadRng) -> Option<HitRecord> {
        self.as_slice().hit(r, t_min, t_max, rng)
    }

    fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb> {
        self.as_slice().bounding_box(time0, time1)
    }
}

impl Hittable for &[Box<dyn Hittable>] {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64, rng: &mut ThreadRng) -> Option<HitRecord> {
        (*self).hit(r, t_min, t_max, rng)
    }

    fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb> {
        (*self).bounding_box(time0, time1)
    }
}

impl Hittable for Box<dyn Hittable> {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64, rng: &mut ThreadRng) -> Option<HitRecord> {
        self.as_ref().hit(r, t_min, t_max, rng)
    }

    fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb> {
        self.as_ref().bounding_box(time0, time1)
    }
}
