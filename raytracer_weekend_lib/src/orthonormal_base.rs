use crate::vec3::Vec3;

pub struct OrthonormalBase {
    pub u: Vec3,
    pub v: Vec3,
    pub w: Vec3,
}

impl OrthonormalBase {
    pub fn from_w(w: Vec3) -> Self {
        let w = w.unit_vector();

        let a = if w.x().abs() > 0.9 {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        };

        let v = w.cross(&a).unit_vector();
        let u = w.cross(&v);

        Self { u, v, w }
    }
}

impl Vec3 {
    pub fn in_onb_coordinates(&self, onb: &OrthonormalBase) -> Vec3 {
        self.x() * onb.u + self.y() * onb.v + self.z() * onb.w
    }
}
