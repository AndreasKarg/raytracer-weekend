use std::fmt::Debug;

use derive_more::Constructor;
use serde::{Deserialize, Serialize};

use raytracer_weekend_lib::camera::Camera;
use raytracer_weekend_lib::vec3::{Color, Point3, Vec3};

use crate::hittable::HittableDescriptor;

pub mod material;
pub mod hittable;
pub mod texture;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World {
    pub geometry: Vec<Box<dyn HittableDescriptor>>,
    pub cameras: Vec<CameraDescriptor>,
    pub background: Color,
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
