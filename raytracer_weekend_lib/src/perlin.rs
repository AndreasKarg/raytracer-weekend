#[cfg(feature = "no_std")]
use micromath::F32Ext;
use rand::prelude::*;

use crate::vec3::{CopyIndex, GenericVec3, Point3, Vec3};
const POINT_COUNT: usize = 256;

#[derive(Debug, Clone)]
pub struct Perlin {
    gradients: [Vec3; POINT_COUNT],
    permutations: [[usize; POINT_COUNT]; 3],
}

impl Perlin {
    pub fn new(rng: &mut impl Rng) -> Self {
        let mut gradients = [Vec3::new(0.0, 0.0, 0.0); POINT_COUNT];
        for item in &mut gradients[..] {
            *item = Vec3::random_min_max(rng, -1.0..1.0).unit_vector();
        }

        let x_permutations = Self::generate_perm(rng);
        let y_permutations = Self::generate_perm(rng);
        let z_permutations = Self::generate_perm(rng);

        Self {
            gradients,
            permutations: [x_permutations, y_permutations, z_permutations],
        }
    }

    fn generate_perm(rng: &mut impl Rng) -> [usize; POINT_COUNT] {
        let mut p = [0; POINT_COUNT];

        for (i, element) in p.iter_mut().enumerate() {
            *element = i;
        }

        Self::permute(&mut p, POINT_COUNT, rng);

        p
    }

    fn permute(p: &mut [usize; POINT_COUNT], n: usize, rng: &mut impl Rng) {
        for i in (1..n).rev() {
            let target = rng.gen_range(0..i);
            p.swap(i, target);
        }
    }

    pub fn noise(&self, p: &Point3) -> f32 {
        let p = *p;
        let base_point_on_lattice = p.floor().to_i64().to_usize();
        let point_within_lattice_cell = p - p.floor();

        let mut gradient_cube = [[[Vec3::new(0.0, 0.0, 0.0); 2]; 2]; 2];

        for (x_offset, aisle) in gradient_cube.iter_mut().enumerate() {
            for (y_offset, row) in aisle.iter_mut().enumerate() {
                for (z_offset, cell) in row.iter_mut().enumerate() {
                    let offset = GenericVec3::new(x_offset, y_offset, z_offset);
                    let current_lattice_point =
                        base_point_on_lattice.overflowing_add(offset).0 & 255;

                    let hash = self
                        .permutations
                        .get(&current_lattice_point)
                        .internal_bit_xor();

                    *cell = self.gradients[hash];
                }
            }
        }

        Self::perlin_interp(gradient_cube, point_within_lattice_cell)
    }

    pub fn turbulence(&self, p: &Point3, depth: usize) -> f32 {
        let mut accum = 0.0;
        let mut temp_p = *p;
        let mut weight = 1.0;

        for _ in 0..depth {
            accum += weight * self.noise(&temp_p);
            weight *= 0.5;
            temp_p *= 2.0;
        }

        accum.abs()
    }

    fn perlin_interp(gradient_cube: [[[Vec3; 2]; 2]; 2], point_within_lattice_cell: Vec3) -> f32 {
        let point_within_lattice_cell = Perlin::filter_hermit(point_within_lattice_cell);

        let mut accum = 0.0;

        // This performs trilinear interpolation.
        // TODO: Clarify code.
        for aisle in 0..2 {
            for row in 0..2 {
                for column in 0..2 {
                    let current_point_on_lattice: GenericVec3<usize> = (aisle, row, column).into();
                    let current_point_on_lattice = current_point_on_lattice.to_f32();
                    let weight_v = point_within_lattice_cell - current_point_on_lattice;

                    let unit_vector: Vec3 = (1.0, 1.0, 1.0).into();
                    let blend_factor = current_point_on_lattice * point_within_lattice_cell
                        + (unit_vector - current_point_on_lattice)
                            * (unit_vector - point_within_lattice_cell);
                    let blend_factor = blend_factor.internal_product();

                    accum += blend_factor * gradient_cube[aisle][row][column].dot(&weight_v);
                }
            }
        }

        accum
    }

    fn filter_hermit(p: Point3) -> Point3 {
        let offset = Vec3::new(3.0, 3.0, 3.0);
        p * p * (offset - 2.0 * p)
    }
}
