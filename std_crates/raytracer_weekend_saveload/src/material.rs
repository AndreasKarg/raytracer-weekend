use crate::texture::{SolidColorDescriptor, TextureDescriptor};
use derive_more::Constructor;
use dyn_clone::{clone_trait_object, DynClone};
use eros::Result;
use rand::rngs::ThreadRng;
use raytracer_weekend_lib::light_source::DiffuseLight;
use raytracer_weekend_lib::material::{Dielectric, Lambertian, Material, Metal};
use raytracer_weekend_lib::vec3::Color;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::Arc;

#[typetag::serde]
pub trait MaterialDescriptor: Sync + Send + Debug + DynClone {
    fn to_material(&self, rng: &mut ThreadRng) -> Result<Arc<dyn Material<ThreadRng>>>;
}
clone_trait_object!(MaterialDescriptor);

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct LambertianDescriptor {
    albedo: Box<dyn TextureDescriptor>,
}

impl LambertianDescriptor {
    pub fn new_solid_color(color: Color) -> Self {
        Self::new(Box::new(SolidColorDescriptor::new(color)))
    }
}

#[typetag::serde(name = "Lambertian")]
impl MaterialDescriptor for LambertianDescriptor {
    fn to_material(&self, rng: &mut ThreadRng) -> Result<Arc<dyn Material<ThreadRng>>> {
        Ok(Arc::new(Lambertian::new(self.albedo.to_texture(rng)?)))
    }
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct MetalDescriptor {
    albedo: Color,
    fuzz: f32,
}

#[typetag::serde(name = "Metal")]
impl MaterialDescriptor for MetalDescriptor {
    fn to_material(&self, _: &mut ThreadRng) -> Result<Arc<dyn Material<ThreadRng>>> {
        Ok(Arc::new(Metal::new(self.albedo, self.fuzz)))
    }
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct DielectricDescriptor {
    ir: f32,
}

#[typetag::serde(name = "Dielectric")]
impl MaterialDescriptor for DielectricDescriptor {
    fn to_material(&self, _: &mut ThreadRng) -> Result<Arc<dyn Material<ThreadRng>>> {
        Ok(Arc::new(Dielectric::new(self.ir)))
    }
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct DiffuseLightDescriptor {
    emit: Box<dyn TextureDescriptor>,
}

#[typetag::serde(name = "DiffuseLight")]
impl MaterialDescriptor for DiffuseLightDescriptor {
    fn to_material(&self, rng: &mut ThreadRng) -> Result<Arc<dyn Material<ThreadRng>>> {
        Ok(Arc::new(DiffuseLight::new(self.emit.to_texture(rng)?)))
    }
}
