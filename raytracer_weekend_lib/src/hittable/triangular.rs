use derive_more::Constructor;
use itertools::{Itertools, MinMaxResult};
use rand::prelude::ThreadRng;

use crate::{
    aabb::Aabb,
    hittable::{HitRecord, Hittable},
    material::Material,
    ray::Ray,
    texture::Point2d,
    vec3::Point3,
};

#[derive(Debug, Constructor)]
pub struct Triangle<M: Material> {
    vertices: [Point3; 3],
    material: M,
}

impl<M: Material> Triangle<M> {
    fn min_max(nums: impl Iterator<Item = f64>) -> (f64, f64) {
        match nums.minmax() {
            MinMaxResult::NoElements => {
                panic!()
            }
            MinMaxResult::OneElement(num) => (num, num),
            MinMaxResult::MinMax(min, max) => (min, max),
        }
    }
}

impl<M: Material> Hittable for Triangle<M> {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64, rng: &mut ThreadRng) -> Option<HitRecord> {
        let vertex_a = self.vertices[0];
        let vertex_b = self.vertices[1];
        let vertex_c = self.vertices[2];
        let a_to_b = vertex_b - vertex_a;
        let a_to_c = vertex_c - vertex_a;
        let normal = a_to_b.cross(&a_to_c);
        let determinant = -r.direction().dot(&normal);
        let inv_determinant = 1.0 / determinant;
        let a_to_ray_origin = r.origin() - vertex_a;
        let a_to_ray_origin_cross_direction = a_to_ray_origin.cross(&r.direction());
        let u = a_to_c.dot(&a_to_ray_origin_cross_direction) * inv_determinant;
        let v = -a_to_b.dot(&a_to_ray_origin_cross_direction) * inv_determinant;
        let t = a_to_ray_origin.dot(&normal) * inv_determinant;

        if t < t_min || t > t_max {
            return None;
        }

        let triangle_was_hit =
            determinant >= 1e-6 && t >= 0.0 && u >= 0.0 && v >= 0.0 && (u + v) <= 1.0;

        if !triangle_was_hit {
            return None;
        }

        let p = r.at(t);

        // TODO: Compute texture u/v properly
        Some(HitRecord::new_with_face_normal(
            p,
            t,
            Point2d { u: 0.0, v: 0.0 },
            &self.material,
            r,
            normal,
        ))
    }

    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<Aabb> {
        let min_max_x = Self::min_max(self.vertices.iter().map(|v| v.x()));
        let min_max_y = Self::min_max(self.vertices.iter().map(|v| v.y()));
        let min_max_z = Self::min_max(self.vertices.iter().map(|v| v.z()));

        let min = Point3::new(min_max_x.0, min_max_y.0, min_max_z.0);
        let max = Point3::new(min_max_x.1, min_max_y.1, min_max_z.1);

        Some(Aabb::new(min, max))
    }
}
