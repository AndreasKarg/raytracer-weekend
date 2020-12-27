use derive_more::Constructor;
use rand::prelude::Rng;

use super::{aabb::Aabb, hittable::Hittable};
use crate::{hittable::HitRecord, ray::Ray};

///! An implementation of a Boundary Volume Hierarchy thingamajig.

#[derive(Debug)]
pub struct BvhNode {
    left: Box<dyn Hittable>,
    right: Box<dyn Hittable>,
    bounding_box: Aabb,
}

impl BvhNode {
    fn box_x_compare(a: &dyn Hittable, b: &dyn Hittable) {
        unimplemented!()
    }

    fn box_y_compare(a: &dyn Hittable, b: &dyn Hittable) {
        unimplemented!()
    }

    fn box_z_compare(a: &dyn Hittable, b: &dyn Hittable) {
        unimplemented!()
    }

    pub fn new(
        src_objects: &[Box<dyn Hittable>],
        time0: f64,
        time1: f64,
        rng: &mut impl Rng,
    ) -> Self {
        let mut objects = Vec::from(src_objects);

        let axis = rng.gen_range(0..=2);

        let comparator = match axis {
            0 => Self::box_x_compare,
            1 => Self::box_y_compare,
            2 => Self::box_z_compare,
            _ => unreachable!(),
        };

        let mut left;
        let mut right;

        if objects.len() == 1 {
            left = objects[0].clone();
            right = left.clone()
        } else if objects.len() == 2 {
            left = objects[0].clone();
            right = objects[1].clone();
        } else {
            objects.sort_by(comparator);
            let mid = objects.len() / 2;
            left = Box::new(Self::new(&objects[..mid], time0, time1, rng));
            right = Box::new(Self::new(&objects[mid..], time0, time1, rng));
        }

        let box_left = left.bounding_box(time0, time1).unwrap();
        let box_right = right.bounding_box(time0, time1).unwrap();
        let surrounding_box = Aabb::surrounding_box(&box_left, &box_right);

        Self {
            left,
            right,
            bounding_box: surrounding_box,
        }
    }
}

impl Hittable for BvhNode {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord<'_>> {
        if !self.bounding_box.hit(r, t_min, t_max) {
            return None;
        };

        let hit_left = self.left.hit(r, t_min, t_max);
        let t_max = match &hit_left {
            None => t_max,
            Some(hit) => hit.t,
        };
        let hit_right = self.right.hit(r, t_min, t_max);

        match &hit_right {
            Some(hit) => hit_right,
            None => hit_left,
        }
    }

    fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb> {
        Some(self.bounding_box.clone())
    }
}
