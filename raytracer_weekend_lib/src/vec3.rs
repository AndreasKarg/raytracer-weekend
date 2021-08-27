use core::{
    fmt::{Debug, Display, Formatter},
    ops::{
        Add, AddAssign, BitAnd, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Range, Sub,
    },
};

use num_traits::Num;
use rand::Rng;

#[derive(Copy, Clone, Debug)]
pub struct GenericVec3<T>
where
    T: Num + Copy,
{
    e: [T; 3],
}

impl<T: Num + Copy> GenericVec3<T> {
    pub fn new(e0: T, e1: T, e2: T) -> Self {
        Self { e: [e0, e1, e2] }
    }

    pub fn x(&self) -> T {
        self.e[0]
    }

    pub fn y(&self) -> T {
        self.e[1]
    }

    pub fn z(&self) -> T {
        self.e[2]
    }

    pub fn length_squared(&self) -> T {
        let e = self.e;
        e[0] * e[0] + e[1] * e[1] + e[2] * e[2]
    }

    pub fn dot(&self, rhs: &Self) -> T {
        self.e[0] * rhs.e[0] + self.e[1] * rhs.e[1] + self.e[2] * rhs.e[2]
    }

    pub fn cross(&self, rhs: &Self) -> Self {
        Self::new(
            self.e[1] * rhs.e[2] - self.e[2] * rhs.e[1],
            self.e[2] * rhs.e[0] - self.e[0] * rhs.e[2],
            self.e[0] * rhs.e[1] - self.e[1] * rhs.e[0],
        )
    }

    pub fn internal_product(&self) -> T {
        let e = self.e;

        e[0] * e[1] * e[2]
    }

    pub fn as_tuple(&self) -> (T, T, T) {
        let e = self.e;
        (e[0], e[1], e[2])
    }
}

impl<T: Num + Copy> From<(T, T, T)> for GenericVec3<T> {
    fn from(tuple: (T, T, T)) -> Self {
        Self::new(tuple.0, tuple.1, tuple.2)
    }
}

impl GenericVec3<f64> {
    pub const fn new_const(e0: f64, e1: f64, e2: f64) -> Self {
        Self { e: [e0, e1, e2] }
    }

    pub fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }

    pub fn unit_vector(&self) -> Self {
        *self / self.length()
    }

    pub fn random(rng: &mut impl Rng) -> Self {
        Self::random_min_max(rng, 0.0..1.0)
    }

    pub fn random_min_max(rng: &mut impl Rng, range: Range<f64>) -> Self {
        Self::new(
            rng.gen_range(range.clone()),
            rng.gen_range(range.clone()),
            rng.gen_range(range),
        )
    }

    pub fn random_in_unit_sphere(rng: &mut impl Rng) -> Self {
        loop {
            let p = Self::random_min_max(rng, -1.0..1.0);
            if p.length_squared() < 1.0 {
                return p;
            }
        }
    }

    pub fn random_unit_vector(rng: &mut impl Rng) -> Self {
        Self::random_in_unit_sphere(rng).unit_vector()
    }

    pub fn random_in_hemisphere(normal: &Vec3, rng: &mut impl Rng) -> Self {
        let in_unit_sphere = Self::random_in_unit_sphere(rng);
        if in_unit_sphere.dot(normal) > 0.0 {
            // In the same hemisphere as the normal
            in_unit_sphere
        } else {
            -in_unit_sphere
        }
    }

    pub fn random_in_unit_disk(rng: &mut impl Rng) -> Self {
        loop {
            let p = Self::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0), 0.0);
            if p.length_squared() < 1.0 {
                return p;
            }
        }
    }

    pub fn is_near_zero(&self) -> bool {
        // Return true if the vector is close to zero in all dimensions.
        const S: f64 = 1e-8;

        (self.e[0].abs() < S) && (self.e[1].abs() < S) && (self.e[2].abs() < S)
    }

    pub fn reflect(&self, normal: &Vec3) -> Vec3 {
        *self - 2.0 * self.dot(normal) * *normal
    }

    pub fn refract(&self, n: &Vec3, eta_i_over_eta_t: f64) -> Self {
        let uv = *self;
        let n = *n;
        let cos_theta = (-uv).dot(&n).min(1.0);
        let r_out_perpendicular = eta_i_over_eta_t * (uv + cos_theta * n);
        let r_out_parallel = -(1.0 - r_out_perpendicular.length_squared()).abs().sqrt() * n;
        r_out_perpendicular + r_out_parallel
    }

    pub fn floor(self) -> Vec3 {
        let e = self.e;
        Vec3::new(e[0].floor(), e[1].floor(), e[2].floor())
    }

    pub fn to_i64(&self) -> GenericVec3<i64> {
        let e0 = self.e[0] as i64;
        let e1 = self.e[1] as i64;
        let e2 = self.e[2] as i64;

        GenericVec3 { e: [e0, e1, e2] }
    }
}

impl GenericVec3<i64> {
    pub fn to_usize(&self) -> GenericVec3<usize> {
        let e0 = self.e[0] as usize;
        let e1 = self.e[1] as usize;
        let e2 = self.e[2] as usize;

        GenericVec3 { e: [e0, e1, e2] }
    }
}

impl GenericVec3<usize> {
    pub fn to_f64(&self) -> GenericVec3<f64> {
        let e0 = self.e[0] as f64;
        let e1 = self.e[1] as f64;
        let e2 = self.e[2] as f64;

        GenericVec3 { e: [e0, e1, e2] }
    }

    pub fn overflowing_add(&self, rhs: Self) -> (Self, bool) {
        let (x, x_overflow) = self.e[0].overflowing_add(rhs.e[0]);
        let (y, y_overflow) = self.e[1].overflowing_add(rhs.e[1]);
        let (z, z_overflow) = self.e[2].overflowing_add(rhs.e[2]);

        (Self::new(x, y, z), x_overflow || y_overflow || z_overflow)
    }

    pub fn internal_bit_xor(&self) -> usize {
        self.e[0] ^ self.e[1] ^ self.e[2]
    }
}

impl<T: Default + Num + Copy> Default for GenericVec3<T> {
    fn default() -> Self {
        Self::new(T::default(), T::default(), T::default())
    }
}

impl<T: Num + Copy> Sub for GenericVec3<T> {
    type Output = GenericVec3<<T as Sub>::Output>;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(
            self.e[0] - rhs.e[0],
            self.e[1] - rhs.e[1],
            self.e[2] - rhs.e[2],
        )
    }
}

impl<T: Num + Copy> Add for GenericVec3<T> {
    type Output = GenericVec3<<T as Add>::Output>;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(
            self.e[0] + rhs.e[0],
            self.e[1] + rhs.e[1],
            self.e[2] + rhs.e[2],
        )
    }
}

impl<T: Num + Copy + AddAssign> AddAssign for GenericVec3<T> {
    fn add_assign(&mut self, rhs: Self) {
        self.e[0] += rhs.e[0];
        self.e[1] += rhs.e[1];
        self.e[2] += rhs.e[2];
    }
}

impl<T, U> Mul<U> for GenericVec3<T>
where
    T: Num + Copy + Mul<U, Output = T>,
    U: Num + Copy,
    <T as Mul<U>>::Output: Num + Copy,
{
    type Output = GenericVec3<<T as Mul<U>>::Output>;

    fn mul(self, rhs: U) -> Self::Output {
        Self::new(self.e[0] * rhs, self.e[1] * rhs, self.e[2] * rhs)
    }
}

impl Mul<GenericVec3<f64>> for f64 {
    type Output = GenericVec3<Self>;

    fn mul(self, rhs: Self::Output) -> Self::Output {
        rhs * self
    }
}

impl<T: Num + Copy> Mul for GenericVec3<T> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(
            self.e[0] * rhs.e[0],
            self.e[1] * rhs.e[1],
            self.e[2] * rhs.e[2],
        )
    }
}

impl<T: MulAssign + Num + Copy> MulAssign<T> for GenericVec3<T> {
    fn mul_assign(&mut self, rhs: T) {
        self.e[0] *= rhs;
        self.e[1] *= rhs;
        self.e[2] *= rhs;
    }
}

impl<T: Num + Copy> Div<T> for GenericVec3<T> {
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Self::new(self.e[0] / rhs, self.e[1] / rhs, self.e[2] / rhs)
    }
}

impl<T: Num + Copy> Div for GenericVec3<T> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self::new(
            self.e[0] / rhs.e[0],
            self.e[1] / rhs.e[1],
            self.e[2] / rhs.e[2],
        )
    }
}

impl<T: DivAssign + Num + Copy> DivAssign<T> for GenericVec3<T> {
    fn div_assign(&mut self, rhs: T) {
        self.e[0] /= rhs;
        self.e[1] /= rhs;
        self.e[2] /= rhs;
    }
}

impl<T: Neg<Output = T> + Num + Copy> Neg for GenericVec3<T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.e[0], -self.e[1], -self.e[2])
    }
}

impl<T: BitAnd<U, Output = T> + Num + Copy, U: Copy> BitAnd<U> for GenericVec3<T> {
    type Output = Self;

    fn bitand(self, rhs: U) -> Self::Output {
        Self::new(self.e[0] & rhs, self.e[1] & rhs, self.e[2] & rhs)
    }
}

impl<T: Num + Copy> Index<usize> for GenericVec3<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.e[index]
    }
}

impl<T: Num + Copy> IndexMut<usize> for GenericVec3<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.e[index]
    }
}

pub trait CopyIndex<T> {
    type Output;

    fn get(&self, index: &T) -> Self::Output;
}

impl<T: Num + Copy, const N: usize> CopyIndex<GenericVec3<usize>> for [[T; N]; 3] {
    type Output = GenericVec3<T>;

    fn get(&self, index: &GenericVec3<usize>) -> Self::Output {
        Self::Output::new(
            self[0][index.e[0]],
            self[1][index.e[1]],
            self[2][index.e[2]],
        )
    }
}

pub type Vec3 = GenericVec3<f64>;
pub type Point3 = Vec3;
pub type Color = Vec3;

impl Display for Vec3 {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} {} {}", self.e[0], self.e[1], self.e[2])
    }
}
