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
        let u = p.x() - p.x().floor();
        let v = p.y() - p.y().floor();
        let w = p.z() - p.z().floor();

        let i = p.x().floor() as i64 as usize;
        let j = p.y().floor() as i64 as usize;
        let k = p.z().floor() as i64 as usize;

        let mut c = [[[0.0; 2]; 2]; 2];

        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    c[di][dj][dk] = self.ranfloat[self.perm_x[(i + di) & 255]
                        ^ self.perm_y[(j + dj) & 255]
                        ^ self.perm_z[(k + dk) & 255]];
                }
            }
        }

        Self::trilinear_interp(c, u, v, w)
    }

    fn trilinear_interp(c: [[[f64; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
        let mut accum = 0.0;

        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let fi = i as f64;
                    let fj = j as f64;
                    let fk = k as f64;
                    accum += (fi * u + (1.0 - fi) * (1.0 - u))
                        * (fj * v + (1.0 - fj) * (1.0 - v))
                        * (fk * w + (1.0 - fk) * (1.0 - w))
                        * c[i][j][k];
                }
            }
        }

        accum
    }
}
