mod ray;
mod vec3;

#[macro_use]
extern crate derive_more;

use std::io::BufWriter;
use {
    indicatif::ProgressIterator,
    ray::Ray,
    std::{
        fs::File,
        io::{self, Write},
    },
    vec3::{Color, Point3, Vec3},
};

#[derive(From, Into)]
struct Width(usize);

#[derive(From, Into)]
struct Height(usize);

fn main() {
    // Image
    const ASPECT_RATIO: f64 = 16.0 / 9.0;
    const IMAGE_WIDTH: Width = Width(400);
    const IMAGE_HEIGHT: Height = Height((IMAGE_WIDTH.0 as f64 / ASPECT_RATIO) as usize);

    // World
    let mut world: Vec<Box<dyn Hittable>> = vec![
        Box::new(Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5)),
        Box::new(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0)),
    ];

    // Camera

    let viewport_height = 2.0;
    let viewport_width = ASPECT_RATIO * viewport_height;
    let focal_length = 1.0;

    let origin = Point3::new(0.0, 0.0, 0.0);
    let horizontal = Vec3::new(viewport_width, 0.0, 0.0);
    let vertical = Vec3::new(0.0, viewport_height, 0.0);
    let lower_left_corner =
        origin - horizontal / 2.0 - vertical / 2.0 - Vec3::new(0.0, 0.0, focal_length);

    // Render

    let file = File::create("image.ppm").unwrap();
    let mut file = BufWriter::new(file);

    writeln!(&mut file, "P3\n{} {}\n255", IMAGE_WIDTH.0, IMAGE_HEIGHT.0).unwrap();

    for j in (0..IMAGE_HEIGHT.0).rev().progress() {
        for i in 0..IMAGE_WIDTH.0 {
            let u = (i as f64) / ((IMAGE_WIDTH.0 - 1) as f64);
            let v = (j as f64) / ((IMAGE_HEIGHT.0 - 1) as f64);
            let r = Ray::new(
                origin,
                lower_left_corner + u * horizontal + v * vertical - origin,
            );
            let pixel_color = ray_color(&r, &world);

            write_color(&mut file, pixel_color).unwrap();
        }
    }
}

fn write_color<F: Write>(f: &mut F, pixel_color: Vec3) -> io::Result<()> {
    let ir = (255.999 * pixel_color.x()) as u8;
    let ig = (255.999 * pixel_color.y()) as u8;
    let ib = (255.999 * pixel_color.z()) as u8;

    writeln!(f, "{} {} {}", ir, ig, ib)
}

fn ray_color(r: &Ray, world: &impl HittableVec) -> Color {
    if let Some(hit_record) = world.hit(r, 0.0, f64::INFINITY) {
        return 0.5 * (hit_record.normal + Color::new(1.0, 1.0, 1.0));
    }
    let unit_direction = r.direction().unit_vector();
    let t = 0.5 * (unit_direction.y() + 1.0);
    return (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * Color::new(0.5, 0.7, 1.0);
}

fn hit_sphere(center: &Point3, radius: f64, r: &Ray) -> Option<f64> {
    let origin_to_center = r.origin() - *center;
    let a = r.direction().length_squared();
    let half_b = origin_to_center.dot(&r.direction());
    let c = origin_to_center.length_squared() - radius * radius;
    let discriminant = half_b * half_b - a * c;
    if discriminant < 0.0 {
        None
    } else {
        Some((-half_b - discriminant.sqrt()) / a)
    }
}

#[derive(Debug, Constructor)]
struct HitRecord {
    pub p: Point3,
    pub normal: Vec3,
    pub t: f64,
    pub is_front_face: bool,
}

impl HitRecord {
    pub fn new_with_face_normal(p: Point3, t: f64, ray: &Ray, outward_normal: Vec3) -> Self {
        let is_front_face = ray.direction().dot(&outward_normal) < 0.0;

        let normal = match is_front_face {
            true => outward_normal,
            false => -outward_normal,
        };

        Self::new(p, normal, t, is_front_face)
    }
}

trait Hittable {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
}

#[derive(Constructor)]
struct Sphere {
    center: Point3,
    radius: f64,
}

impl Hittable for Sphere {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let center = self.center;
        let radius = self.radius;

        let origin_to_center = r.origin() - center;
        let a = r.direction().length_squared();
        let half_b = origin_to_center.dot(&r.direction());
        let c = origin_to_center.length_squared() - radius * radius;

        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();

        // Find the nearest root that lies in the acceptable range.
        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || t_max < root {
            root = (-half_b + sqrtd) / a;
            if root < t_min || t_max < root {
                return None;
            }
        }

        let t = root;
        let p = r.at(root);
        let outward_normal = (p - center) / radius;

        Some(HitRecord::new_with_face_normal(p, t, r, outward_normal))
    }
}

trait HittableVec {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
}

impl HittableVec for Vec<Box<dyn Hittable>> {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let mut hit_anything = false;
        let mut closest_so_far = t_max;
        let mut rec = None;

        for object in self.iter() {
            if let Some(temp_rec) = object.hit(r, t_min, closest_so_far) {
                hit_anything = true;
                closest_so_far = temp_rec.t;
                rec = Some(temp_rec);
            }
        }

        rec
    }
}
