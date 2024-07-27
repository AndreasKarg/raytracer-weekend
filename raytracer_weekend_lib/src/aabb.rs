//! An implementation of an Axis-Aligned Bounding Box (AABB)
use {
    super::{ray::Ray, vec3::Point3},
    core::mem::swap,
    derive_more::Constructor,
};

#[derive(Constructor, Debug, Clone)]
pub struct Aabb {
    minimum: Point3,
    maximum: Point3,
}

impl Aabb {
    pub fn min(&self) -> Point3 {
        self.minimum
    }

    pub fn max(&self) -> Point3 {
        self.maximum
    }

    pub fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> bool {
        let minimum = self.minimum;
        let maximum = self.maximum;
        let mut t_min = t_min;
        let mut t_max = t_max;
        for a in 0..3 {
            let inverted_denominator = 1.0 / ray.direction()[a];
            let mut t0 = (minimum[a] - ray.origin()[a]) * inverted_denominator;
            let mut t1 = (maximum[a] - ray.origin()[a]) * inverted_denominator;

            if inverted_denominator < 0.0 {
                swap(&mut t0, &mut t1);
            }

            t_min = t0.max(t_min);
            t_max = t1.min(t_max);

            // println!("min: {}\nmax: {}", t_min, t_max);

            if t_max <= t_min {
                return false;
            }
        }

        true
    }

    // pub fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> bool {
    //     let minimum = self.minimum;
    //     let maximum = self.maximum;
    //     let mut t_min = t_min;
    //     let mut t_max = t_max;
    //     for a in 0..3 {
    //         let t0 = ((minimum[a] - ray.origin()[a]) / ray.direction()[a])
    //             .min((maximum[a] - ray.origin()[a]) / ray.direction()[a]);
    //         let t1 = ((minimum[a] - ray.origin()[a]) / ray.direction()[a])
    //             .max((maximum[a] - ray.origin()[a]) / ray.direction()[a]);
    //
    //         t_min = t0.max(t_min);
    //         t_max = t1.min(t_max);
    //
    //         // println!("min: {}\nmax: {}", t_min, t_max);
    //
    //         if t_max <= t_min {
    //             return false;
    //         }
    //     }
    //
    //     true
    // }

    pub fn surrounding_box(box1: &Aabb, box2: &Aabb) -> Self {
        let small = Point3::new(
            box1.min().x().min(box2.min().x()),
            box1.min().y().min(box2.min().y()),
            box1.min().z().min(box2.min().z()),
        );

        let big = Point3::new(
            box1.max().x().max(box2.max().x()),
            box1.max().y().max(box2.max().y()),
            box1.max().z().max(box2.max().z()),
        );

        Aabb::new(small, big)
    }
}
