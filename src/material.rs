use {
    super::{hittable::HitRecord, ray::Ray, vec3::Color, vec3::Vec3},
    rand::rngs::ThreadRng,
};

pub struct Scatter {
    pub attenuation: Color,
    pub scattered_ray: Ray,
}

pub trait Material: std::fmt::Debug {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, rng: &mut ThreadRng) -> Option<Scatter>;
}

#[derive(Debug, Constructor)]
pub struct Lambertian {
    albedo: Color,
}

impl Material for Lambertian {
    fn scatter(&self, _r_in: &Ray, rec: &HitRecord, rng: &mut ThreadRng) -> Option<Scatter> {
        let mut scatter_direction = rec.normal + Vec3::random_unit_vector(rng);

        if scatter_direction.is_near_zero() {
            scatter_direction = rec.normal;
        }

        let scattered_ray = Ray::new(rec.p, scatter_direction);
        let attenuation = self.albedo;

        Some(Scatter {
            attenuation,
            scattered_ray,
        })
    }
}

#[derive(Debug)]
pub struct Metal {
    albedo: Color,
    fuzz: f64,
}

impl Metal {
    pub fn new(albedo: Color, fuzz: f64) -> Self {
        assert!(fuzz <= 1.0);

        Self { albedo, fuzz }
    }
}

impl Material for Metal {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, rng: &mut ThreadRng) -> Option<Scatter> {
        let reflected = r_in.direction.unit_vector().reflect(&rec.normal);
        let scattered_ray = Ray::new(
            rec.p,
            reflected + self.fuzz * Vec3::random_in_unit_sphere(rng),
        );
        let attenuation = self.albedo;

        if scattered_ray.direction().dot(&rec.normal) > 0.0 {
            Some(Scatter {
                scattered_ray,
                attenuation,
            })
        } else {
            None
        }
    }
}
