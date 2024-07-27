use raytracer_weekend_lib::hittable::Hittable;
use std::fmt::Debug;
use dyn_clone::{clone_trait_object, DynClone};
use derive_more::Constructor;
use raytracer_weekend_lib::hittable::spherical::Sphere;
use serde::{Deserialize, Serialize};
use raytracer_weekend_lib::vec3::Point3;
use crate::material::MaterialDescriptor;

#[typetag::serde]
pub trait HittableDescriptor: Sync + Send + Debug + DynClone {
    fn to_hittable(&self) -> Box<dyn Hittable>;
}
clone_trait_object!(HittableDescriptor);

pub trait HittableDescriptorList {
    fn to_hittables(&self) -> Vec<Box<dyn Hittable>>;
}

impl HittableDescriptorList for Vec<Box<dyn HittableDescriptor>>
{
    fn to_hittables(&self) -> Vec<Box<dyn Hittable>> {
        self.iter().map(|h| h.to_hittable()).collect()
    }
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct SphereDescriptor {
    center: Point3,
    radius: f32,
    material: Box<dyn MaterialDescriptor>,
}

#[typetag::serde(name = "Sphere")]
impl HittableDescriptor for SphereDescriptor {
    fn to_hittable(&self) -> Box<dyn Hittable> {
        Box::new(Sphere::new(
            self.center,
            self.radius,
            self.material.to_material(),
        ))
    }
}