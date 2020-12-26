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
pub struct HitRecord {
    pub p: Point3,
    pub normal: Vec3,
    pub material: Arc<dyn Material>,
    pub t: f64,
    pub is_front_face: bool,
}

impl HitRecord {
    pub fn new_with_face_normal(
        p: Point3,
        t: f64,
        material: Arc<dyn Material>,
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

#[derive(Constructor)]
pub struct Sphere {
    center: Point3,
    radius: f64,
    material: Arc<dyn Material>,
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let center = self.center;
        let radius = self.radius;

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
            self.material.clone(),
            ray,
            outward_normal,
        ))
    }
}

pub trait HittableVec {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
}

impl HittableVec for Vec<Box<dyn Hittable>> {
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
