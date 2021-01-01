use derive_more::Constructor;
use rand::prelude::ThreadRng;

use crate::{
    aabb::Aabb,
    hittable::{HitRecord, Hittable},
    ray::Ray,
    vec3::{Point3, Vec3},
};

#[derive(Debug, Constructor)]
pub struct Translation<T: Hittable> {
    inner: T,
    offset: Vec3,
}

impl<T: Hittable> Hittable for Translation<T> {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64, rng: &mut ThreadRng) -> Option<HitRecord> {
        let translated_ray = Ray::new(r.origin() - self.offset, r.direction(), r.time());

        let hit = self.inner.hit(&translated_ray, t_min, t_max, rng)?;

        let translated_hitpoint = hit.p + self.offset;

        Some(HitRecord::new_with_face_normal(
            translated_hitpoint,
            hit.t,
            hit.texture_uv,
            hit.material,
            &translated_ray,
            hit.normal,
        ))
    }

    fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb> {
        let bounding_box = self.inner.bounding_box(time0, time1)?;

        Some(Aabb::new(
            bounding_box.min() + self.offset,
            bounding_box.max() + self.offset,
        ))
    }
}

#[derive(Debug)]
pub struct YRotation<T: Hittable> {
    inner: T,
    sin_theta: f64,
    cos_theta: f64,
    bounding_box: Option<Aabb>,
}

impl<T: Hittable> YRotation<T> {
    pub fn new(inner: T, angle_degrees: f64) -> Self {
        let angle_radians = angle_degrees.to_radians();

        let sin_theta = angle_radians.sin();
        let cos_theta = angle_radians.cos();

        let bounding_box = inner
            .bounding_box(0.0, 1.0)
            .map(|b| Self::rotate_bounding_box(b, sin_theta, cos_theta));

        Self {
            inner,
            sin_theta,
            cos_theta,
            bounding_box,
        }
    }

    fn rotate_bounding_box(bbox: Aabb, sin_theta: f64, cos_theta: f64) -> Aabb {
        let mut min = Point3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
        let mut max = Point3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);

        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let i = i as f64;
                    let j = j as f64;
                    let k = k as f64;

                    let x = i * bbox.max().x() + (1.0 - i) * bbox.min().x();
                    let y = j * bbox.max().y() + (1.0 - j) * bbox.min().y();
                    let z = k * bbox.max().z() + (1.0 - k) * bbox.min().z();

                    let new_x = cos_theta * x + sin_theta * z;
                    let new_z = -sin_theta * x + cos_theta * z;

                    let tester = Vec3::new(new_x, y, new_z);

                    for axis in 0..3 {
                        min[axis] = min[axis].min(tester[axis]);
                        max[axis] = max[axis].max(tester[axis]);
                    }
                }
            }
        }

        Aabb::new(min, max)
    }
}

impl<T: Hittable> Hittable for YRotation<T> {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64, rng: &mut ThreadRng) -> Option<HitRecord> {
        let sin_theta = self.sin_theta;
        let cos_theta = self.cos_theta;

        let mut origin = r.origin();
        let mut direction = r.direction();

        origin[0] = cos_theta * r.origin()[0] - sin_theta * r.origin()[2];
        origin[2] = sin_theta * r.origin()[0] + cos_theta * r.origin()[2];

        direction[0] = cos_theta * r.direction()[0] - sin_theta * r.direction()[2];
        direction[2] = sin_theta * r.direction()[0] + cos_theta * r.direction()[2];

        let rotated_r = Ray::new(origin, direction, r.time());
        let rec = self.inner.hit(&rotated_r, t_min, t_max, rng)?;

        let mut p = rec.p;
        let mut normal = rec.normal;

        p[0] = cos_theta * rec.p[0] + sin_theta * rec.p[2];
        p[2] = -sin_theta * rec.p[0] + cos_theta * rec.p[2];

        normal[0] = cos_theta * rec.normal[0] + sin_theta * rec.normal[2];
        normal[2] = -sin_theta * rec.normal[0] + cos_theta * rec.normal[2];

        Some(HitRecord::new_with_face_normal(
            p,
            rec.t,
            rec.texture_uv,
            rec.material,
            &rotated_r,
            normal,
        ))
    }

    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<Aabb> {
        self.bounding_box.clone()
    }
}

pub trait Transformable {
    type Inner: Hittable;

    fn rotate_y(self, angle_degrees: f64) -> YRotation<Self::Inner>;
    fn translate(self, offset: Vec3) -> Translation<Self::Inner>;
}

impl<T: Hittable> Transformable for T {
    type Inner = T;

    fn rotate_y(self, angle_degrees: f64) -> YRotation<Self::Inner> {
        YRotation::new(self, angle_degrees)
    }

    fn translate(self, offset: Vec3) -> Translation<Self::Inner> {
        Translation::new(self, offset)
    }
}
