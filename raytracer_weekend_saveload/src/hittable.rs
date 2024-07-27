use raytracer_weekend_lib::hittable::Hittable;
use std::fmt::Debug;
use std::path::PathBuf;
use dyn_clone::{clone_trait_object, DynClone};
use derive_more::Constructor;
use raytracer_weekend_lib::hittable::spherical::Sphere;
use serde::{Deserialize, Serialize};
use raytracer_weekend_lib::ActiveRng;
use raytracer_weekend_lib::hittable::rectangular::{XYRectangle, XZRectangle, YZRectangle};
use raytracer_weekend_lib::hittable::transformations::Translation;
use raytracer_weekend_lib::hittable::triangular::load_wavefront_obj;
use raytracer_weekend_lib::material::Material;
use raytracer_weekend_lib::vec3::{Point3, Vec3};
use crate::material::MaterialDescriptor;

#[typetag::serde]
pub trait HittableDescriptor: Sync + Send + Debug + DynClone {
    fn to_hittable(&self, rng: &mut ActiveRng) -> Box<dyn Hittable>;
}
clone_trait_object!(HittableDescriptor);

pub trait HittableDescriptorList {
    fn to_hittables(&self, rng: &mut ActiveRng) -> Vec<Box<dyn Hittable>>;
}

impl HittableDescriptorList for Vec<Box<dyn HittableDescriptor>>
{
    fn to_hittables(&self, rng: &mut ActiveRng) -> Vec<Box<dyn Hittable>> {
        self.iter().map(|h| h.to_hittable(rng)).collect()
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
    fn to_hittable(&self, _: &mut ActiveRng) -> Box<dyn Hittable> {
        Box::new(Sphere::new(
            self.center,
            self.radius,
            self.material.to_material(),
        ))
    }
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct MovingSphereDescriptor {
    center0: Point3,
    time0: f32,
    center1: Point3,
    time1: f32,
    radius: f32,
    material: Box<dyn MaterialDescriptor>,
}

#[typetag::serde(name = "MovingSphere")]
impl HittableDescriptor for MovingSphereDescriptor {
    fn to_hittable(&self, _: &mut ActiveRng) -> Box<dyn Hittable> {
        Box::new(raytracer_weekend_lib::hittable::spherical::MovingSphere::new(
            self.center0,
            self.time0,
            self.center1,
            self.time1,
            self.radius,
            self.material.to_material(),
        ))
    }
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct WavefrontObjDescriptor {
    path: PathBuf,
}

#[typetag::serde(name = "WavefrontObj")]
impl HittableDescriptor for WavefrontObjDescriptor {
    fn to_hittable(&self, rng: &mut ActiveRng) -> Box<dyn Hittable> {
        Box::new(load_wavefront_obj(&self.path, rng).unwrap())
    }
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct TranslationDescriptor {
    inner: Box<dyn HittableDescriptor>,
    offset: Vec3,
}

#[typetag::serde(name = "Translation")]
impl HittableDescriptor for TranslationDescriptor {
    fn to_hittable(&self, rng: &mut ActiveRng) -> Box<dyn Hittable> {
        Box::new(Translation::new(
            self.inner.to_hittable(rng),
            self.offset,
        ))
    }
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct XYRectangleDescriptor {
    x0: f32,
    x1: f32,
    y0: f32,
    y1: f32,
    k: f32,
    material: Box<dyn MaterialDescriptor>,
}

#[typetag::serde(name = "XYRectangle")]
impl HittableDescriptor for XYRectangleDescriptor {
    fn to_hittable(&self, _: &mut ActiveRng) -> Box<dyn Hittable> {
        Box::new(XYRectangle::new(
            self.x0,
            self.x1,
            self.y0,
            self.y1,
            self.k,
            self.material.to_material(),
        ))
    }
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct XZRectangleDescriptor {
    x0: f32,
    x1: f32,
    z0: f32,
    z1: f32,
    k: f32,
    material: Box<dyn MaterialDescriptor>,
}

#[typetag::serde(name = "XZRectangle")]
impl HittableDescriptor for XZRectangleDescriptor {
    fn to_hittable(&self, _: &mut ActiveRng) -> Box<dyn Hittable> {
        Box::new(XZRectangle::new(
            self.x0,
            self.x1,
            self.z0,
            self.z1,
            self.k,
            self.material.to_material(),
        ))
    }
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct YZRectangleDescriptor {
    y0: f32,
    y1: f32,
    z0: f32,
    z1: f32,
    k: f32,
    material: Box<dyn MaterialDescriptor>,
}

#[typetag::serde(name = "YZRectangle")]
impl HittableDescriptor for YZRectangleDescriptor {
    fn to_hittable(&self, _: &mut ActiveRng) -> Box<dyn Hittable> {
        Box::new(YZRectangle::new(
            self.y0,
            self.y1,
            self.z0,
            self.z1,
            self.k,
            self.material.to_material(),
        ))
    }
}
