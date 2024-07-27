use std::fmt::Debug;
use derive_more::Constructor;
use dyn_clone::{clone_trait_object, DynClone};
use serde::{Deserialize, Serialize};
use raytracer_weekend_lib::camera::Camera;
use raytracer_weekend_lib::hittable::Hittable;
use raytracer_weekend_lib::hittable::spherical::Sphere;
use raytracer_weekend_lib::material::{Lambertian, Material};
use raytracer_weekend_lib::texture::Texture;
use raytracer_weekend_lib::vec3::{Color, Point3, Vec3};

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

#[typetag::serde]
pub trait MaterialDescriptor: Sync + Send + Debug + DynClone {
    fn to_material(&self) -> Box<dyn Material>;
}
clone_trait_object!(MaterialDescriptor);

#[typetag::serde]
pub trait TextureDescriptor: Sync + Send + Debug + DynClone {
    fn to_texture(&self) -> Box<dyn Texture>;
}
clone_trait_object!(TextureDescriptor);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World {
    pub geometry: Vec<Box<dyn HittableDescriptor>>,
    pub cameras: Vec<CameraDescriptor>,
    pub background: Color,
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct SphereDescriptor {
    center: Point3,
    radius: f32,
    material: Box<dyn MaterialDescriptor>,
}

#[typetag::serde]
impl HittableDescriptor for SphereDescriptor {
    fn to_hittable(&self) -> Box<dyn Hittable> {
        Box::new(Sphere::new(
            self.center,
            self.radius,
            self.material.to_material(),
        ))
    }
}

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

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct SolidColorDescriptor {
    color: Color,
}

impl SolidColorDescriptor {
    pub fn new_rgb(red: f32, green: f32, blue: f32) -> Box<Self> {
        Box::new(Self::new(Color::new(red, green, blue)))
    }
}

#[typetag::serde(name = "SolidColor")]
impl TextureDescriptor for SolidColorDescriptor {
    fn to_texture(&self) -> Box<dyn Texture> {
        Box::new(raytracer_weekend_lib::texture::SolidColor::new(self.color))
    }
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct CheckerDescriptor {
    even: Box<dyn TextureDescriptor>,
    odd: Box<dyn TextureDescriptor>,
    frequency: f32,
}

#[typetag::serde(name = "Checker")]
impl TextureDescriptor for CheckerDescriptor {
    fn to_texture(&self) -> Box<dyn Texture> {
        Box::new(raytracer_weekend_lib::texture::Checker::new(
            self.even.to_texture(),
            self.odd.to_texture(),
            self.frequency,
        ))
    }
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct CameraDescriptor {
    look_from: Point3,
    look_at: Point3,
    up_vector: Vec3,
    vertical_field_of_view: f32,
    aspect_ratio: f32,
    aperture: f32,
    focus_dist: f32,
    time0: f32,
    time1: f32,
}

impl CameraDescriptor {
    pub fn to_camera(&self) -> Camera {
        Camera::new(
            self.look_from,
            self.look_at,
            self.up_vector,
            self.vertical_field_of_view,
            self.aspect_ratio,
            self.aperture,
            self.focus_dist,
            self.time0,
            self.time1,
        )
    }
}
