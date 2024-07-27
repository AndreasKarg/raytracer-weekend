use core::fmt::Debug;

#[cfg(feature = "no_std")]
use micromath::F32Ext;
use rand::Rng;

use crate::{
    aabb::Aabb,
    hittable::{HitRecord, Hittable},
    material::Isotropic,
    ray::Ray,
    texture::{Point2d, Texture},
    vec3::Vec3,
    ActiveRng,
};

#[derive(Debug)]
pub struct ConstantMedium<H: Hittable, T: Texture + Clone> {
    boundary: H,
    phase_function: Isotropic<T>,
    neg_inv_density: f32,
}

impl<H: Hittable, T: Texture + Clone> ConstantMedium<H, T> {
    pub fn new(boundary: H, density: f32, texture: T) -> Self {
        let neg_inv_density = -1.0 / density;
        let phase_function = Isotropic::new(texture);

        Self {
            boundary,
            phase_function,
            neg_inv_density,
        }
    }
}

impl<H: Hittable, T: Texture + Clone> Hittable for ConstantMedium<H, T> {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32, rng: &mut ActiveRng) -> Option<HitRecord> {
        let rec1 = self
            .boundary
            .hit(r, f32::NEG_INFINITY, f32::INFINITY, rng)?;
        let rec2 = self.boundary.hit(r, rec1.t + 0.0001, f32::INFINITY, rng)?;

        let mut rec1_t = rec1.t;
        let mut rec2_t = rec2.t;

        rec1_t = rec1_t.max(t_min);
        rec2_t = rec2_t.min(t_max);

        if rec1_t >= rec2_t {
            return None;
        }

        rec1_t = rec1_t.max(0.0);

        let ray_length = r.direction().length();
        let distance_inside_boundary = (rec2_t - rec1_t) * ray_length;
        let hit_distance = self.neg_inv_density * rng.gen::<f32>().log10();

        if hit_distance > distance_inside_boundary {
            return None;
        }

        let t = rec1_t + hit_distance / ray_length;
        let p = r.at(t);
        let normal = Vec3::new(1.0, 0.0, 0.0); // arbitrary
        let front_face = true;
        let dummy_texture_uv = Point2d { u: 0.0, v: 0.0 };

        Some(HitRecord::new(
            p,
            normal,
            &self.phase_function,
            t,
            dummy_texture_uv,
            front_face,
        ))
    }

    fn bounding_box(&self, time0: f32, time1: f32) -> Option<Aabb> {
        self.boundary.bounding_box(time0, time1)
    }
}
