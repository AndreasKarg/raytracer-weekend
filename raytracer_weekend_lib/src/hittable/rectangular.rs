use derive_more::Constructor;
use rand::prelude::ThreadRng;

use crate::{
    aabb::Aabb,
    hittable::{HitRecord, Hittable},
    material::Material,
    ray::Ray,
    texture::Point2d,
    vec3::{Point3, Vec3},
};

#[derive(Debug, Constructor)]
pub struct XYRectangle {
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    k: f64,
    material: Box<dyn Material>,
}

impl Hittable for XYRectangle {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64, _rng: &mut ThreadRng) -> Option<HitRecord> {
        let x0 = self.x0;
        let y0 = self.y0;
        let x1 = self.x1;
        let y1 = self.y1;
        let k = self.k;

        let t = (k - r.origin().z()) / r.direction().z();
        if t < t_min || t > t_max {
            return None;
        }
        let x = r.origin().x() + t * r.direction().x();
        let y = r.origin().y() + t * r.direction().y();
        if x < x0 || x > x1 || y < y0 || y > y1 {
            return None;
        }

        let u = (x - x0) / (x1 - x0);
        let v = (y - y0) / (y1 - y0);
        let t = t;
        let outward_normal = Vec3::new(0.0, 0.0, 1.0);
        let p = r.at(t);
        return Some(HitRecord::new_with_face_normal(
            p,
            t,
            Point2d { u, v },
            self.material.as_ref(),
            r,
            outward_normal,
        ));
    }

    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<Aabb> {
        Some(Aabb::new(
            Point3::new(self.x0, self.y0, self.k - 0.0001),
            Point3::new(self.x1, self.y1, self.k + 0.0001),
        ))
    }
}

#[derive(Debug, Constructor)]
pub struct XZRectangle {
    x0: f64,
    x1: f64,
    z0: f64,
    z1: f64,
    k: f64,
    material: Box<dyn Material>,
}

impl Hittable for XZRectangle {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64, _rng: &mut ThreadRng) -> Option<HitRecord> {
        let x0 = self.x0;
        let z0 = self.z0;
        let x1 = self.x1;
        let z1 = self.z1;
        let k = self.k;

        let t = (k - r.origin().y()) / r.direction().y();
        if t < t_min || t > t_max {
            return None;
        }
        let x = r.origin().x() + t * r.direction().x();
        let z = r.origin().z() + t * r.direction().z();
        if x < x0 || x > x1 || z < z0 || z > z1 {
            return None;
        }

        let u = (x - x0) / (x1 - x0);
        let v = (z - z0) / (z1 - z0);
        let t = t;
        let outward_normal = Vec3::new(0.0, 1.0, 0.0);
        let p = r.at(t);
        return Some(HitRecord::new_with_face_normal(
            p,
            t,
            Point2d { u, v },
            self.material.as_ref(),
            r,
            outward_normal,
        ));
    }

    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<Aabb> {
        Some(Aabb::new(
            Point3::new(self.x0, self.k - 0.0001, self.z0),
            Point3::new(self.x1, self.k + 0.0001, self.z1),
        ))
    }
}

#[derive(Debug, Constructor)]
pub struct YZRectangle {
    y0: f64,
    y1: f64,
    z0: f64,
    z1: f64,
    k: f64,
    material: Box<dyn Material>,
}

impl Hittable for YZRectangle {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64, _rng: &mut ThreadRng) -> Option<HitRecord> {
        let y0 = self.y0;
        let z0 = self.z0;
        let y1 = self.y1;
        let z1 = self.z1;
        let k = self.k;

        let t = (k - r.origin().x()) / r.direction().x();
        if t < t_min || t > t_max {
            return None;
        }
        let y = r.origin().y() + t * r.direction().y();
        let z = r.origin().z() + t * r.direction().z();
        if y < y0 || y > y1 || z < z0 || z > z1 {
            return None;
        }

        let u = (y - y0) / (y1 - y0);
        let v = (z - z0) / (z1 - z0);
        let t = t;
        let outward_normal = Vec3::new(1.0, 0.0, 0.0);
        let p = r.at(t);
        return Some(HitRecord::new_with_face_normal(
            p,
            t,
            Point2d { u, v },
            self.material.as_ref(),
            r,
            outward_normal,
        ));
    }

    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<Aabb> {
        Some(Aabb::new(
            Point3::new(self.k - 0.0001, self.y0, self.z0),
            Point3::new(self.k + 0.0001, self.y1, self.z1),
        ))
    }
}

#[derive(Debug)]
pub struct Cuboid {
    box_min: Point3,
    box_max: Point3,
    sides: [Box<dyn Hittable>; 6],
}

impl Cuboid {
    pub fn new(p0: Point3, p1: Point3, material: Box<dyn Material>) -> Self {
        let sides: [Box<dyn Hittable>; 6] = [
            Box::new(XYRectangle::new(
                p0.x(),
                p1.x(),
                p0.y(),
                p1.y(),
                p1.z(),
                material.clone(),
            )),
            Box::new(XYRectangle::new(
                p0.x(),
                p1.x(),
                p0.y(),
                p1.y(),
                p0.z(),
                material.clone(),
            )),
            Box::new(XZRectangle::new(
                p0.x(),
                p1.x(),
                p0.z(),
                p1.z(),
                p1.y(),
                material.clone(),
            )),
            Box::new(XZRectangle::new(
                p0.x(),
                p1.x(),
                p0.z(),
                p1.z(),
                p0.y(),
                material.clone(),
            )),
            Box::new(YZRectangle::new(
                p0.y(),
                p1.y(),
                p0.z(),
                p1.z(),
                p1.x(),
                material.clone(),
            )),
            Box::new(YZRectangle::new(
                p0.y(),
                p1.y(),
                p0.z(),
                p1.z(),
                p0.x(),
                material.clone(),
            )),
        ];

        Self {
            box_min: p0,
            box_max: p1,
            sides,
        }
    }
}

impl Hittable for Cuboid {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64, rng: &mut ThreadRng) -> Option<HitRecord> {
        self.sides.hit(r, t_min, t_max, rng)
    }

    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<Aabb> {
        Some(Aabb::new(self.box_min, self.box_max))
    }
}
