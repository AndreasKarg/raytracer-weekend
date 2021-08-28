use alloc::boxed::Box;
use core::f64::consts::PI;

use derive_more::Constructor;
use rand::prelude::Rng;

use crate::{
    aabb::Aabb,
    hittable::{HitRecord, Hittable},
    material::Material,
    ray::Ray,
    texture::Point2d,
    vec3::{Point3, Vec3},
    ActiveRng,
};

fn hit_sphere<'a>(
    ray: &Ray,
    t_min: f64,
    t_max: f64,
    center: Vec3,
    radius: f64,
    material: &'a (dyn Material + 'a),
) -> Option<HitRecord<'a>> {
    let origin_to_center = ray.origin() - center;
    let a = ray.direction().length_squared();
    let half_b = origin_to_center.dot(&ray.direction());
    let c = origin_to_center.length_squared() - radius * radius;

    let discriminant = half_b.powi(2) - a * c;
    if discriminant < 0.0 {
        return None;
    }

    let sqrtd = discriminant.sqrt();

    // Find the nearest root that lies in the acceptable range.
    let mut root = (-half_b - sqrtd) / a;
    if root < t_min || t_max < root {
        root = (-half_b + sqrtd) / a;
        if root < t_min || t_max < root {
            return None;
        }
    }

    let t = root;
    let hit_point = ray.at(root);
    let outward_normal = (hit_point - center) / radius;
    let texture_uv = get_sphere_uv(&outward_normal);

    Some(HitRecord::new_with_face_normal(
        hit_point,
        t,
        texture_uv,
        material,
        ray,
        outward_normal,
    ))
}

fn get_sphere_uv(p: &Point3) -> Point2d {
    // p: a given point on the sphere of radius one, centered at the origin.
    // u: returned value [0,1] of angle around the Y axis from X=-1.
    // v: returned value [0,1] of angle from Y=-1 to Y=+1.
    //     <1 0 0> yields <0.50 0.50>       <-1  0  0> yields <0.00 0.50>
    //     <0 1 0> yields <0.50 1.00>       < 0 -1  0> yields <0.50 0.00>
    //     <0 0 1> yields <0.25 0.50>       < 0  0 -1> yields <0.75 0.50>

    let theta = (-p.y()).acos();
    let phi = (-p.z()).atan2(p.x()) + PI;

    let u = phi / (2.0 * PI);
    let v = theta / PI;

    Point2d { u, v }
}

#[derive(Constructor, Debug, Clone)]
pub struct Sphere {
    center: Point3,
    radius: f64,
    material: Box<dyn Material>,
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64, _rng: &mut ActiveRng) -> Option<HitRecord> {
        hit_sphere(
            ray,
            t_min,
            t_max,
            self.center,
            self.radius,
            self.material.as_ref(),
        )
    }

    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<Aabb> {
        let center = self.center;
        let radius = self.radius;
        let radius_vector = Vec3::new(radius, radius, radius);
        Some(Aabb::new(center - radius_vector, center + radius_vector))
    }
}

#[derive(Constructor, Debug)]
pub struct MovingSphere {
    center0: Point3,
    time0: f64,
    center1: Point3,
    time1: f64,
    radius: f64,
    material: Box<dyn Material>,
}

impl MovingSphere {
    fn center_at_time(&self, time: f64) -> Point3 {
        let center0 = self.center0;
        let time0 = self.time0;
        let center1 = self.center1;
        let time1 = self.time1;
        center0 + ((time - time0) / (time1 - time0)) * (center1 - center0)
    }
}

impl Hittable for MovingSphere {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64, _rng: &mut ActiveRng) -> Option<HitRecord> {
        let center_at_time = self.center_at_time(ray.time());

        hit_sphere(
            ray,
            t_min,
            t_max,
            center_at_time,
            self.radius,
            self.material.as_ref(),
        )
    }

    fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb> {
        let start_center = self.center_at_time(time0);
        let end_center = self.center_at_time(time1);
        let radius = self.radius;
        let radius_vector = Vec3::new(radius, radius, radius);

        let start_box = Aabb::new(start_center - radius_vector, start_center + radius_vector);
        let end_box = Aabb::new(end_center - radius_vector, end_center + radius_vector);

        Some(Aabb::surrounding_box(&start_box, &end_box))
    }
}
