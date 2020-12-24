mod ray;
mod vec3;

#[macro_use]
extern crate derive_more;

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

    let mut file = File::create("image.ppm").unwrap();

    writeln!(&file, "P3\n{} {}\n255", IMAGE_WIDTH.0, IMAGE_HEIGHT.0).unwrap();

    for j in (0..IMAGE_HEIGHT.0).rev().progress() {
        for i in 0..IMAGE_WIDTH.0 {
            let u = (i as f64) / ((IMAGE_WIDTH.0 - 1) as f64);
            let v = (j as f64) / ((IMAGE_HEIGHT.0 - 1) as f64);
            let r = Ray::new(
                origin,
                lower_left_corner + u * horizontal + v * vertical - origin,
            );
            let pixel_color = ray_color(&r);

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

fn ray_color(r: &Ray) -> Color {
    if let Some(t) = hit_sphere(&Point3::new(0.0, 0.0, -1.0), 0.5, r) {
        let normal = Vec3::unit_vector(&(r.at(t) - Vec3::new(0.0, 0.0, -1.0)));
        return 0.5*Color::new(normal.x()+1.0, normal.y()+1.0, normal.z()+1.0);
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
    let discriminant = half_b*half_b - a*c;
    if discriminant < 0.0 {
        None
    } else {
        Some((-half_b - discriminant.sqrt()) / a)
    }
}
