use std::cmp::Ordering;

use derive_more::Constructor;
use rand::prelude::Rng;

use super::{aabb::Aabb, hittable::Hittable};
use crate::{hittable::HitRecord, ray::Ray};

///! An implementation of a Boundary Volume Hierarchy thingamajig.

#[derive(Debug)]
pub struct BvhNode {
    left: Box<dyn Hittable>,
    right: Option<Box<dyn Hittable>>,
    bounding_box: Aabb,
}

impl BvhNode {
    pub fn new(
        mut src_objects: Vec<Box<dyn Hittable>>,
        time0: f64,
        time1: f64,
        rng: &mut impl Rng,
    ) -> Self {
        let axis = rng.gen_range(0..=2);

        let comparator = match axis {
            0 => Self::box_x_compare,
            1 => Self::box_y_compare,
            2 => Self::box_z_compare,
            _ => unreachable!(),
        };

        let left;
        let right;

        if src_objects.len() == 1 {
            left = src_objects.pop().unwrap();
            right = None
        } else if src_objects.len() == 2 {
            left = src_objects.pop().unwrap();
            right = Some(src_objects.pop().unwrap());
        } else {
            src_objects.sort_by(comparator);
            let mid = src_objects.len() / 2;
            left = Box::new(Self::new(
                src_objects.drain(..mid).collect(),
                time0,
                time1,
                rng,
            ));
            right = Some(Box::new(Self::new(src_objects, time0, time1, rng)));
        }

        let box_left = left
            .bounding_box(time0, time1)
            .expect("No bounding box in bvh_node constructor.");

        let surrounding_box = match &right {
            None => box_left,
            Some(right) => {
                let box_right = right
                    .bounding_box(time0, time1)
                    .expect("No bounding box in bvh_node constructor.");
                Aabb::surrounding_box(&box_left, &box_right)
            }
        };

        Self {
            left,
            right,
            bounding_box: surrounding_box,
        }
    }

    fn box_x_compare(a: &Box<dyn Hittable>, b: &Box<dyn Hittable>) -> Ordering {
        Self::box_compare(a, b, 0)
    }

    fn box_y_compare(a: &Box<dyn Hittable>, b: &Box<dyn Hittable>) -> Ordering {
        Self::box_compare(a, b, 1)
    }

    fn box_z_compare(a: &Box<dyn Hittable>, b: &Box<dyn Hittable>) -> Ordering {
        Self::box_compare(a, b, 2)
    }

    fn box_compare(a: &Box<dyn Hittable>, b: &Box<dyn Hittable>, axis: usize) -> Ordering {
        let box_a = a
            .bounding_box(0.0, 0.0)
            .expect("No bounding box in bvh_node constructor.");
        let box_b = b
            .bounding_box(0.0, 0.0)
            .expect("No bounding box in bvh_node constructor.");

        box_a.min()[axis].partial_cmp(&box_b.min()[axis]).unwrap()
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
        let hit_right = self.right.as_ref().and_then(|h| h.hit(r, t_min, t_max));

        match &hit_right {
            Some(hit) => hit_right,
            None => hit_left,
        }
    }

    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<Aabb> {
        Some(self.bounding_box.clone())
    }
}
