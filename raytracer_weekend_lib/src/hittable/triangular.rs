use std::fs;

use itertools::{Itertools, MinMaxResult};
use rand::prelude::ThreadRng;
use wavefront_obj::{
    obj,
    obj::{Geometry, Normal, Object, Primitive, Vertex},
};

use crate::{
    aabb::Aabb,
    bvh::BvhNode,
    hittable::{HitRecord, Hittable},
    material::{Lambertian, Material},
    ray::Ray,
    texture::Point2d,
    vec3::{Color, Point3, Vec3},
};

#[derive(Debug, Clone)]
pub struct Triangle {
    vertices: [Point3; 3],
    normals: [Vec3; 3],
    material: Box<dyn Material>,
}

impl Triangle {
    pub fn new(
        vertices: [Point3; 3],
        normals: [Option<Vec3>; 3],
        material: Box<dyn Material>,
    ) -> Self {
        let vertex_a = vertices[0];
        let vertex_b = vertices[1];
        let vertex_c = vertices[2];
        let a_to_b = vertex_b - vertex_a;
        let a_to_c = vertex_c - vertex_a;
        let triangle_normal = a_to_b.cross(&a_to_c);

        let normals = normals.map(|vertex_normal| vertex_normal.unwrap_or(triangle_normal));

        Self {
            vertices,
            normals,
            material,
        }
    }

    pub fn new_flat_shaded(vertices: [Point3; 3], material: Box<dyn Material>) -> Self {
        Self::new(vertices, [None, None, None], material)
    }

    fn min_max(nums: impl Iterator<Item = f64>) -> (f64, f64) {
        let mut min_max = match nums.minmax() {
            MinMaxResult::NoElements => {
                panic!()
            }
            MinMaxResult::OneElement(num) => (num, num),
            MinMaxResult::MinMax(min, max) => (min, max),
        };

        if (min_max.0 - min_max.1).abs() < 0.0002 {
            min_max = (min_max.0 - 0.0001, min_max.1 + 0.0001);
        }

        min_max
    }
}

impl Hittable for Triangle {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64, _rng: &mut ThreadRng) -> Option<HitRecord> {
        let vertex_a = self.vertices[0];
        let vertex_b = self.vertices[1];
        let vertex_c = self.vertices[2];
        let a_to_b = vertex_b - vertex_a;
        let a_to_c = vertex_c - vertex_a;
        let normal = a_to_b.cross(&a_to_c);
        let determinant = -ray.direction().dot(&normal);
        let inv_determinant = 1.0 / determinant;
        let a_to_ray_origin = ray.origin() - vertex_a;
        let a_to_ray_origin_cross_direction = a_to_ray_origin.cross(&ray.direction());

        let u = a_to_c.dot(&a_to_ray_origin_cross_direction) * inv_determinant;
        let v = -a_to_b.dot(&a_to_ray_origin_cross_direction) * inv_determinant;

        let t = a_to_ray_origin.dot(&normal) * inv_determinant;

        if t < t_min || t > t_max {
            return None;
        }

        let triangle_was_hit = t >= 0.0 && u >= 0.0 && v >= 0.0 && (u + v) <= 1.0;

        if !triangle_was_hit {
            return None;
        }

        let p = ray.at(t);

        let hit_normal = (1.0-u-v) * self.normals[0] + u*self.normals[1] + v*self.normals[2];

        // TODO: Compute texture u/v properly
        Some(HitRecord::new_with_face_normal(
            p,
            t,
            Point2d { u, v },
            self.material.as_ref(),
            ray,
            hit_normal,
        ))
    }

    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<Aabb> {
        let min_max_x = Self::min_max(self.vertices.iter().map(|v| v.x()));
        let min_max_y = Self::min_max(self.vertices.iter().map(|v| v.y()));
        let min_max_z = Self::min_max(self.vertices.iter().map(|v| v.z()));

        let min = Point3::new(min_max_x.0, min_max_y.0, min_max_z.0);
        let max = Point3::new(min_max_x.1, min_max_y.1, min_max_z.1);

        Some(Aabb::new(min, max))
    }
}

impl From<Vertex> for Point3 {
    fn from(v: Vertex) -> Self {
        Self::new(v.x, v.y, v.z)
    }
}

fn parse_geometry<'a>(
    geometry: &'a Geometry,
    vertices: &'a [Vertex],
    normals: &'a [Normal],
) -> impl Iterator<Item = Box<dyn Hittable>> + 'a {
    geometry.shapes.iter().map(move |shape| {
        match shape.primitive {
            Primitive::Point(_) => {
                panic!()
            }
            Primitive::Line(_, _) => {
                panic!()
            }
            Primitive::Triangle(vertex_1_idx, vertex_2_idx, vertex_3_idx) => {
                let vertex_1 = vertices[vertex_1_idx.0];
                let vertex_2 = vertices[vertex_2_idx.0];
                let vertex_3 = vertices[vertex_3_idx.0];

                let normal_1: Option<Vec3> = vertex_1_idx.2.map(|idx| normals[idx].into());
                let normal_2: Option<Vec3> = vertex_2_idx.2.map(|idx| normals[idx].into());
                let normal_3: Option<Vec3> = vertex_3_idx.2.map(|idx| normals[idx].into());

                // TODO: Handle materials properly
                Box::new(Triangle::new(
                    [vertex_1.into(), vertex_2.into(), vertex_3.into()],
                    [normal_1, normal_2, normal_3],
                    Box::new(Lambertian::new_solid_color(Color::new(0.9, 0.9, 0.9))),
                )) as Box<dyn Hittable>
            }
        }
    })
}

fn parse_individual_object(object: &Object) -> Vec<Box<dyn Hittable>> {
    object
        .geometry
        .iter()
        .flat_map(|geometry| parse_geometry(geometry, &object.vertices, &object.normals))
        .collect()
}

pub fn load_wavefront_obj(
    path: &str,
    rng: &mut ThreadRng,
) -> Result<Box<dyn Hittable>, Box<dyn std::error::Error>> {
    let file = fs::read_to_string(path)?;
    let object_set = obj::parse(file)?;
    let triangles: Vec<Box<dyn Hittable>> = object_set
        .objects
        .iter()
        .flat_map(parse_individual_object)
        .collect();
    // TODO: Sort out this time thing
    Ok(Box::new(BvhNode::new(triangles, 0.0, 1.0, rng)))
}
