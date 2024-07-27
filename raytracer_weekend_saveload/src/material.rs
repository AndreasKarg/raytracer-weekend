use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use dyn_clone::{clone_trait_object, DynClone};
use raytracer_weekend_lib::material::{Lambertian, Material};
use crate::texture::TextureDescriptor;

#[typetag::serde]
pub trait MaterialDescriptor: Sync + Send + Debug + DynClone {
    fn to_material(&self) -> Box<dyn Material>;
}
clone_trait_object!(MaterialDescriptor);

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct LambertianDescriptor {
    albedo: Box<dyn TextureDescriptor>,
}

#[typetag::serde(name = "Lambertian")]
impl MaterialDescriptor for LambertianDescriptor {
    fn to_material(&self) -> Box<dyn Material> {
        Box::new(Lambertian::new(self.albedo.to_texture()))
    }
}