use derive_more::Constructor;
use dyn_clone::{clone_trait_object, DynClone};
#[cfg(feature = "no_std")]
use micromath::F32Ext;
use rand::Rng;

use super::{
    hittable::HitRecord,
    ray::Ray,
    vec3::{Color, Vec3},
};
use crate::{
    orthonormal_base::OrthonormalBase,
    texture::{Point2d, SolidColor, Texture},
    vec3::Point3,
    ActiveRng,
};

pub struct Scatter {
    pub attenuation: Color,
    pub scattered_ray: Ray,
    pub pdf: f32,
}

pub trait Material: core::fmt::Debug + Sync + Send + DynClone {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, rng: &mut ActiveRng) -> Option<Scatter>;
    fn scattering_pdf(&self, r_in: &Ray, rec: &HitRecord, scattered_ray: &Ray) -> f32;
    fn emitted(&self, uv: Point2d, p: &Point3) -> Color;
}

clone_trait_object!(Material);

#[derive(Debug, Constructor, Clone)]
pub struct Lambertian<T: Texture> {
    albedo: T,
}

impl Lambertian<SolidColor> {
    pub fn new_solid_color(color: Color) -> Self {
        Self::new(SolidColor::new(color))
    }
}

impl<T: Texture> Material for Lambertian<T> {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, rng: &mut ActiveRng) -> Option<Scatter> {
        // let mut scatter_direction = rec.normal + Vec3::random_unit_vector(rng);

        // if scatter_direction.is_near_zero() {
        //     scatter_direction = rec.normal;
        // }

        let uvw = OrthonormalBase::from_w(rec.normal);
        let scatter_direction = Vec3::random_cosine_direction(rng).in_onb_coordinates(&uvw);

        let scattered_ray = Ray::new(rec.p, scatter_direction.unit_vector(), r_in.time());
        let attenuation = self.albedo.value(rec.texture_uv, &rec.p);
        let pdf = uvw.w.dot(&scattered_ray.direction()) / std::f32::consts::PI;

        Some(Scatter {
            attenuation,
            scattered_ray,
            pdf,
        })
    }

    fn scattering_pdf(&self, r_in: &Ray, rec: &HitRecord, scattered_ray: &Ray) -> f32 {
        let cosine = rec.normal.dot(&scattered_ray.direction().unit_vector());

        if cosine < 0.0f32 {
            0.0f32
        } else {
            cosine / std::f32::consts::PI
        }
    }

    fn emitted(&self, _uv: Point2d, _p: &Point3) -> Color {
        emit_black()
    }
}

#[derive(Debug, Clone)]
pub struct Metal {
    albedo: Color,
    fuzz: f32,
}

impl Metal {
    pub fn new(albedo: Color, fuzz: f32) -> Self {
        assert!(fuzz <= 1.0);

        Self { albedo, fuzz }
    }
}

impl Material for Metal {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, rng: &mut ActiveRng) -> Option<Scatter> {
        let reflected = r_in.direction().unit_vector().reflect(&rec.normal);
        let scattered_ray = Ray::new(
            rec.p,
            reflected + self.fuzz * Vec3::random_in_unit_sphere(rng),
            r_in.time(),
        );
        let attenuation = self.albedo;

        todo!("pdf");

        if scattered_ray.direction().dot(&rec.normal) > 0.0 {
            Some(Scatter {
                scattered_ray,
                attenuation,
                pdf: 0.0,
            })
        } else {
            None
        }
    }

    fn scattering_pdf(&self, r_in: &Ray, rec: &HitRecord, scattered_ray: &Ray) -> f32 {
        todo!()
    }

    fn emitted(&self, _uv: Point2d, _p: &Point3) -> Color {
        emit_black()
    }
}

#[derive(Debug, Constructor, Clone)]
pub struct Dielectric {
    ir: f32,
}

impl Dielectric {
    fn reflectance(cosine: f32, ref_idx: f32) -> f32 {
        let r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
        let r0 = r0 * r0;
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}

impl Material for Dielectric {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, rng: &mut ActiveRng) -> Option<Scatter> {
        let ir = self.ir;

        let attenuation = Color::new(1.0, 1.0, 1.0);
        let refraction_ratio = if rec.is_front_face { 1.0 / ir } else { ir };

        let unit_direction = r_in.direction().unit_vector();
        let cos_theta = (-unit_direction).dot(&rec.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = (refraction_ratio * sin_theta) > 1.0;

        let direction = if cannot_refract
            || Self::reflectance(cos_theta, refraction_ratio) > rng.gen::<f32>()
        {
            unit_direction.reflect(&rec.normal)
        } else {
            unit_direction.refract(&rec.normal, refraction_ratio)
        };

        let scattered_ray = Ray::new(rec.p, direction, r_in.time());

        todo!("pdf");

        Some(Scatter {
            attenuation,
            scattered_ray,
            pdf: 0.0,
        })
    }

    fn scattering_pdf(&self, r_in: &Ray, rec: &HitRecord, scattered_ray: &Ray) -> f32 {
        todo!()
    }

    fn emitted(&self, _uv: Point2d, _p: &Point3) -> Color {
        emit_black()
    }
}

#[derive(Debug, Clone, Constructor)]
pub struct Isotropic<T: Texture> {
    albedo: T,
}

impl<T: Texture> Material for Isotropic<T> {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, rng: &mut ActiveRng) -> Option<Scatter> {
        let attenuation = self.albedo.value(rec.texture_uv, &rec.p);
        let scattered_ray = Ray::new(rec.p, Vec3::random_in_unit_sphere(rng), r_in.time());

        todo!("pdf");

        Some(Scatter {
            attenuation,
            scattered_ray,
            pdf: 0.0,
        })
    }

    fn scattering_pdf(&self, r_in: &Ray, rec: &HitRecord, scattered_ray: &Ray) -> f32 {
        todo!()
    }

    fn emitted(&self, _uv: Point2d, _p: &Point3) -> Color {
        emit_black()
    }
}

fn emit_black() -> Color {
    Color::new(0.0, 0.0, 0.0)
}
