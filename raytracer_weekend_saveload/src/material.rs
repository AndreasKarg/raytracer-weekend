use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use dyn_clone::{clone_trait_object, DynClone};
use raytracer_weekend_lib::material::{Dielectric, Lambertian, Material, Metal};
use raytracer_weekend_lib::texture::SolidColor;
use raytracer_weekend_lib::vec3::Color;
use crate::texture::{SolidColorDescriptor, TextureDescriptor};

#[typetag::serde]
pub trait MaterialDescriptor: Sync + Send + Debug + DynClone {
    fn to_material(&self) -> Box<dyn Material>;
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
    fn to_material(&self) -> Box<dyn Material> {
        Box::new(Lambertian::new(self.albedo.to_texture()))
    }
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct MetalDescriptor {
    albedo: Color,
    fuzz: f32,
}

#[typetag::serde(name = "Metal")]
impl MaterialDescriptor for MetalDescriptor {
    fn to_material(&self) -> Box<dyn Material> {
        Box::new(Metal::new(self.albedo, self.fuzz))
    }
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct DielectricDescriptor {
    ir: f32,
}

#[typetag::serde(name = "Dielectric")]
impl MaterialDescriptor for DielectricDescriptor {
    fn to_material(&self) -> Box<dyn Material> {
        Box::new(Dielectric::new(self.ir))
    }
}
