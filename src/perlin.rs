use rand::prelude::*;

use crate::vec3::Point3;

const POINT_COUNT: usize = 256;

#[derive(Debug, Clone)]
pub struct Perlin {
    ranfloat: [f64; POINT_COUNT],
    perm_x: [usize; POINT_COUNT],
    perm_y: [usize; POINT_COUNT],
    perm_z: [usize; POINT_COUNT],
}

impl Perlin {
    pub fn new(rng: &mut ThreadRng) -> Self {
        let mut ranfloat = [0.0; POINT_COUNT];
        for item in &mut ranfloat[..] {
            *item = rng.gen();
        }

        let perm_x = Self::generate_perm(rng);
        let perm_y = Self::generate_perm(rng);
        let perm_z = Self::generate_perm(rng);

        Self {
            ranfloat,
            perm_x,
            perm_y,
            perm_z,
        }
    }

    fn generate_perm(rng: &mut ThreadRng) -> [usize; POINT_COUNT] {
        let mut p = [0; POINT_COUNT];

        for i in 0..POINT_COUNT {
            p[i] = i;
        }

        Self::permute(&mut p, POINT_COUNT, rng);

        p
    }

    fn permute(p: &mut [usize; POINT_COUNT], n: usize, rng: &mut ThreadRng) {
        for i in (1..n).rev() {
            let target = rng.gen_range(0..i);
            p.swap(i, target);
        }
    }

    pub fn noise(&self, p: &Point3) -> f64 {
        let i = (4.0 * p.x()) as i64 as usize & 255;
        let j = (4.0 * p.y()) as i64 as usize & 255;
        let k = (4.0 * p.z()) as i64 as usize & 255;

        self.ranfloat[self.perm_x[i] ^ self.perm_y[j] ^ self.perm_z[k]]
    }
}
