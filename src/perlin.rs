use rand::prelude::*;

use crate::vec3::{Point3, Vec3};

const POINT_COUNT: usize = 256;

#[derive(Debug, Clone)]
pub struct Perlin {
    ranvec: [Vec3; POINT_COUNT],
    perm_x: [usize; POINT_COUNT],
    perm_y: [usize; POINT_COUNT],
    perm_z: [usize; POINT_COUNT],
}

impl Perlin {
    pub fn new(rng: &mut ThreadRng) -> Self {
        let mut ranvec = [Vec3::new(0.0,0.0,0.0); POINT_COUNT];
        for item in &mut ranvec[..] {
            *item = Vec3::random_min_max(rng, -1.0..1.0).unit_vector();
        }

        let perm_x = Self::generate_perm(rng);
        let perm_y = Self::generate_perm(rng);
        let perm_z = Self::generate_perm(rng);

        Self {
            ranvec,
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

        let mut c = [[[Vec3::new(0.0,0.0,0.0); 2]; 2]; 2];

        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    c[di][dj][dk] = self.ranvec[self.perm_x[(i + di) & 255]
                        ^ self.perm_y[(j + dj) & 255]
                        ^ self.perm_z[(k + dk) & 255]];
                }
            }
        }

        Self::perlin_interp(c, u, v, w)
    }

    fn perlin_interp(c: [[[Vec3; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
        let uu = u*u*(3.0-2.0*u);
        let vv = v*v*(3.0-2.0*v);
        let ww = w*w*(3.0-2.0*w);
        let mut accum = 0.0;

        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let fi = i as f64;
                    let fj = j as f64;
                    let fk = k as f64;
                    let weight_v = Vec3::new(u-fi, v-fj, w-fk);
                    accum += (fi * uu + (1.0 - fi) * (1.0 - uu))
                        * (fj * vv + (1.0 - fj) * (1.0 - vv))
                        * (fk * ww + (1.0 - fk) * (1.0 - ww))
                        * c[i][j][k].dot(&weight_v);
                }
            }
        }

        accum
    }
}
