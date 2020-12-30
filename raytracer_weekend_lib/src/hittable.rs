use std::{f64::consts::PI, fmt::Debug};

use derive_more::Constructor;

use super::{
    aabb::Aabb,
    material::Material,
    ray::Ray,
    texture::Point2d,
    vec3::{Point3, Vec3},
};

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
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
    fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb>;
}

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

#[derive(Constructor, Debug)]
pub struct Sphere {
    center: Point3,
    radius: f64,
    material: Box<dyn Material>,
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
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
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
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

pub trait HittableVec {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
    fn bounding_box(&self, t0: f64, t1: f64) -> Option<Aabb>;
}

impl HittableVec for [Box<dyn Hittable>] {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let mut closest_so_far = t_max;
        let mut rec = None;

        for object in self.iter() {
            if let Some(temp_rec) = object.hit(r, t_min, closest_so_far) {
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
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
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
        Some(Aabb::new(Point3::new(self.x0, self.y0, self.k-0.0001), Point3::new(self.x1, self.y1, self.k+0.0001)))
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
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
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
        Some(Aabb::new(Point3::new(self.x0, self.k-0.0001, self.z0), Point3::new(self.x1, self.k+0.0001, self.z1)))
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
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
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
        Some(Aabb::new(Point3::new(self.k-0.0001, self.y0, self.z0), Point3::new(self.k+0.0001, self.y1, self.z1)))
    }
}
