use derive_more::Constructor;

use super::vec3::{Point3, Vec3};

#[derive(Constructor, Default, Debug)]
pub struct Ray {
    origin: Point3,
    direction: Vec3,
    time: f32,
}

impl Ray {
    pub fn origin(&self) -> Point3 {
        self.origin
    }

    pub fn direction(&self) -> Vec3 {
        self.direction
    }

    pub fn time(&self) -> f32 {
        self.time
    }

    pub fn at(&self, t: f32) -> Point3 {
        self.origin + t * self.direction
    }
}
