use std::{
    collections::HashMap,
    fs,
    ops::{Add, Mul},
    sync::Arc,
};

use itertools::{Itertools, MinMaxResult};
use rand::prelude::ThreadRng;
use wavefront_obj::{
    mtl,
    mtl::{Illumination, MtlSet},
    obj,
    obj::{Geometry, Normal, Object, Primitive, TVertex, Vertex},
};

use crate::{
    aabb::Aabb,
    bvh::BvhNode,
    hittable::{HitRecord, Hittable},
    image_texture::ImageTexture,
    light_source::DiffuseLight,
    material::{Lambertian, Material},
    ray::Ray,
    texture::{Checker, Point2d, SolidColor},
    vec3::{Color, Point3, Vec3},
};

#[derive(Debug, Clone)]
pub struct Triangle {
    vertices: [Point3; 3],
    normals: [Vec3; 3],
    texture_uv: [Point2d; 3],
    material: Arc<dyn Material>,
}

impl Triangle {
    pub fn new(
        vertices: [Point3; 3],
        normals: [Option<Vec3>; 3],
        texture_uv: [Option<Point2d>; 3],
        material: Arc<dyn Material>,
    ) -> Self {
        let vertex_a = vertices[0];
        let vertex_b = vertices[1];
        let vertex_c = vertices[2];
        let a_to_b = vertex_b - vertex_a;
        let a_to_c = vertex_c - vertex_a;
        let triangle_normal = a_to_b.cross(&a_to_c);

        let normals = normals.map(|vertex_normal| vertex_normal.unwrap_or(triangle_normal));

        let default_uv = [
            Point2d { u: 0.0, v: 0.0 },
            Point2d { u: 1.0, v: 0.0 },
            Point2d { u: 0.0, v: 1.0 },
        ];
        let texture_uv = texture_uv
            .zip(default_uv)
            .map(|(param, default)| param.unwrap_or(default));

        Self {
            vertices,
            normals,
            texture_uv,
            material,
        }
    }

    pub fn new_flat_shaded(vertices: [Point3; 3], material: Arc<dyn Material>) -> Self {
        Self::new(vertices, [None, None, None], [None, None, None], material)
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

        let hit_normal = Self::interpolate_barycentric(u, v, &self.normals);
        let hit_uv = Self::interpolate_barycentric(u, v, &self.texture_uv);

        // TODO: Compute texture u/v properly
        Some(HitRecord::new_with_face_normal(
            p,
            t,
            hit_uv,
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

impl From<TVertex> for Point2d {
    fn from(v: TVertex) -> Self {
        Self { u: v.u, v: v.v }
    }
}

fn parse_geometry<'a>(
    geometry: &'a Geometry,
    vertices: &'a [Vertex],
    normals: &'a [Normal],
    texture_vertices: &'a [TVertex],
    materials: &Option<HashMap<String, Arc<dyn Material>>>,
) -> impl Iterator<Item = Box<dyn Hittable>> + 'a {
    let material = if let Some(mat_name) = geometry.material_name.as_ref() {
        let mat_lib = materials.as_ref().unwrap();
        mat_lib[mat_name].clone()
    } else {
        Arc::new(DiffuseLight::new(SolidColor::new_rgb(1.0, 0.0, 1.0)))
    };

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

                let texture_uv_1: Option<Point2d> =
                    vertex_1_idx.1.map(|idx| texture_vertices[idx].into());
                let texture_uv_2: Option<Point2d> =
                    vertex_2_idx.1.map(|idx| texture_vertices[idx].into());
                let texture_uv_3: Option<Point2d> =
                    vertex_3_idx.1.map(|idx| texture_vertices[idx].into());

                let normal_1: Option<Vec3> = vertex_1_idx.2.map(|idx| normals[idx].into());
                let normal_2: Option<Vec3> = vertex_2_idx.2.map(|idx| normals[idx].into());
                let normal_3: Option<Vec3> = vertex_3_idx.2.map(|idx| normals[idx].into());

                // TODO: Handle materials properly
                Box::new(Triangle::new(
                    [vertex_1.into(), vertex_2.into(), vertex_3.into()],
                    [normal_1, normal_2, normal_3],
                    [texture_uv_1, texture_uv_2, texture_uv_3],
                    material.clone(),
                )) as Box<dyn Hittable>
            }
        }
    })
}

fn parse_individual_object(
    object: &Object,
    materials: &Option<HashMap<String, Arc<dyn Material>>>,
) -> Vec<Box<dyn Hittable>> {
    object
        .geometry
        .iter()
        .flat_map(|geometry| {
            parse_geometry(
                geometry,
                &object.vertices,
                &object.normals,
                &object.tex_vertices,
                materials,
            )
        })
        .collect()
}

pub fn load_wavefront_obj(
    path: &str,
    rng: &mut ThreadRng,
) -> Result<Box<dyn Hittable>, Box<dyn std::error::Error>> {
    let obj_file = fs::read_to_string(path)?;
    let object_set = obj::parse(obj_file)?;
    let materials = object_set
        .material_library
        .as_ref()
        .map(|filename| path_to_file_in_same_folder(path, filename))
        .map(load_wavefront_mtl)
        .transpose()?;
    let triangles: Vec<Box<dyn Hittable>> = object_set
        .objects
        .iter()
        .flat_map(|obj| parse_individual_object(obj, &materials))
        .collect();
    // TODO: Sort out this time thing
    Ok(Box::new(BvhNode::new(triangles, 0.0, 1.0, rng)))
}

fn path_to_file_in_same_folder(path: &str, filename: &str) -> String {
    let mut base_path = fs::canonicalize(path).unwrap();
    println!("{}", base_path.display());
    base_path.pop();
    base_path.push(filename);

    let path = base_path.to_str().unwrap().to_owned();
    println!("{}", &path);

    path
}

fn load_wavefront_mtl(
    path: String,
) -> Result<HashMap<String, Arc<dyn Material>>, Box<dyn std::error::Error>> {
    let mtl_file = fs::read_to_string(path.clone())?;
    let material_set = mtl::parse(mtl_file)?;

    let materials: HashMap<_, _> = material_set
        .materials
        .iter()
        .map(|mtl| {
            let name = mtl.name.clone();
            let parsed_mtl = parse_material(mtl, path.as_ref());

            (name, parsed_mtl)
        })
        .collect();

    Ok(materials)
}

fn parse_material(obj_material: &mtl::Material, mtl_path: &str) -> Arc<dyn Material> {
    if obj_material.illumination != Illumination::AmbientDiffuse {
        panic!()
    }

    let texture = obj_material
        .diffuse_map
        .as_ref()
        .map(|filename| path_to_file_in_same_folder(mtl_path, filename))
        .map(|path| ImageTexture::open(&path).unwrap())
        .unwrap();

    Arc::new(Lambertian::new(texture))
}

impl Triangle {
    fn interpolate_barycentric<T>(u: f64, v: f64, interpolatee: &[T; 3]) -> T
    where
        f64: Mul<T, Output = T>,
        T: Add<Output = T> + Clone,
    {
        (1.0 - u - v) * interpolatee[0].clone()
            + u * interpolatee[1].clone()
            + v * interpolatee[2].clone()
    }
}
