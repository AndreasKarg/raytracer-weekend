use {
    super::{
        material::Material,
        ray::Ray,
        vec3::{Point3, Vec3},
    },
    derive_more::Constructor,
    std::sync::Arc,
};

#[derive(Debug, Constructor)]
pub struct HitRecord<'a> {
    pub p: Point3,
    pub normal: Vec3,
    pub material: &'a (dyn Material + 'a),
    pub t: f64,
    pub is_front_face: bool,
}

impl<'a> HitRecord<'a> {
    pub fn new_with_face_normal(
        p: Point3,
        t: f64,
        material: &'a (dyn Material + 'a),
        ray: &Ray,
        outward_normal: Vec3,
    ) -> Self {
        let is_front_face = ray.direction().dot(&outward_normal) < 0.0;

        let normal = match is_front_face {
            true => outward_normal,
            false => -outward_normal,
        };

        Self::new(p, normal, material, t, is_front_face)
    }
}

pub trait Hittable: Sync + Send {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
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

    let discriminant = half_b * half_b - a * c;
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

    Some(HitRecord::new_with_face_normal(
        hit_point,
        t,
        material,
        ray,
        outward_normal,
    ))
}

#[derive(Constructor)]
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
}

#[derive(Constructor)]
pub struct MovingSphere {
    center0: Point3,
    time0: f64,
    center1: Point3,
    time1: f64,
    radius: f64,
    material: Box<dyn Material>,
}

impl Hittable for MovingSphere {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let center0 = self.center0;
        let time0 = self.time0;
        let center1 = self.center1;
        let time1 = self.time1;

        let center_at_time =
            center0 + ((ray.time() - time0) / (time1 - time0)) * (center1 - center0);

        hit_sphere(
            ray,
            t_min,
            t_max,
            center_at_time,
            self.radius,
            self.material.as_ref(),
        )
    }
}

pub trait HittableVec {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
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
}
