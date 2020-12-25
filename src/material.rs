use super::{hittable::HitRecord, ray::Ray, vec3::Color};

pub trait Material {
    fn scatter(r_in: &Ray, rec: &HitRecord, attenuation: &Color) -> Option<Ray>;
}
