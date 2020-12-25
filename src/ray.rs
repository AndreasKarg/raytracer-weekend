use super::vec3::{Point3, Vec3};

#[derive(Constructor, Default)]
pub(crate) struct Ray {
    pub origin: Point3,
    pub direction: Vec3,
}

impl Ray {
    pub fn origin(&self) -> Point3 {
        self.origin
    }
    pub fn direction(&self) -> Vec3 {
        self.direction
    }

    pub fn at(&self, t: f64) -> Point3 {
        self.origin + t * self.direction
    }
}
