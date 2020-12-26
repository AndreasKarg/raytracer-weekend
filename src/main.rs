mod hittable;
mod material;
mod ray;
mod vec3;

use itertools::all;
use {
    derive_more::{From, Into},
    hittable::{Hittable, HittableVec, Sphere},
    indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle},
    itertools::iproduct,
    material::{Dielectric, Lambertian, Material, Metal},
    rand::prelude::*,
    ray::Ray,
    rayon::prelude::*,
    std::{
        fs::File,
        io::{self, BufWriter, Write},
        sync::Arc,
    },
    vec3::{Color, Point3, Vec3},
};

#[derive(From, Into)]
struct Width(usize);

#[derive(From, Into)]
struct Height(usize);

const ASPECT_RATIO: f64 = 3.0 / 2.0;
const IMAGE_WIDTH: Width = Width(400);
const IMAGE_HEIGHT: Height = Height((IMAGE_WIDTH.0 as f64 / ASPECT_RATIO) as usize);
const SAMPLES_PER_PIXEL: usize = 100;
const MAX_DEPTH: usize = 50;

fn main() {
    let mut rng = rand::thread_rng();

    // World
    let material_ground = Arc::new(Lambertian::new(Color::new(0.5, 0.5, 0.5)));
    let lambertian = Arc::new(Lambertian::new(Color::new(0.4, 0.2, 0.1)));
    let glass = Arc::new(Dielectric::new(1.5));
    let metal = Arc::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.0));

    let mut world: Vec<Box<dyn Hittable>> = vec![
        Box::new(Sphere::new(
            Point3::new(0.0, -1000.0, 0.0),
            1000.0,
            material_ground,
        )),
        Box::new(Sphere::new(Point3::new(-4.0, 0.2, 0.1), 1.0, lambertian)),
        Box::new(Sphere::new(Point3::new(0.0, 1.0, 0.0), 1.0, glass.clone())),
        Box::new(Sphere::new(Point3::new(0.0, 1.0, 0.0), -0.95, glass)),
        Box::new(Sphere::new(Point3::new(4.0, 1.0, 0.0), 1.0, metal)),
    ];

    for a in -11..11 {
        for b in -11..11 {
            let a = a as f64;
            let b = b as f64;

            let center = Point3::new(a + 0.9 * rng.gen::<f64>(), 0.2, b + 0.9 * rng.gen::<f64>());

            if (center - Point3::new(4.0, 0.2, 0.0)).length() <= 0.9 {
                continue;
            }

            let sphere_material: Arc<dyn Material>;

            let choose_mat: f64 = rng.gen();
            if choose_mat < 0.8 {
                let albedo = Color::random(&mut rng) * Color::random(&mut rng);
                sphere_material = Arc::new(Lambertian::new(albedo));
            } else if choose_mat < 0.95 {
                let albedo = Color::random_min_max(&mut rng, 0.5..1.0);
                let fuzz = rng.gen_range(0.0..0.5);
                sphere_material = Arc::new(Metal::new(albedo, fuzz));
            } else {
                sphere_material = Arc::new(Dielectric::new(1.5));
            }

            let sphere = Box::new(Sphere::new(center, 0.2, sphere_material));
            world.push(sphere);
        }
    }

    // Camera
    let look_from = Point3::new(13.0, 2.0, 3.0);
    let look_at = Point3::new(0.0, 0.0, 0.0);
    let v_up = Vec3::new(0.0, 1.0, 0.0);
    let distance_to_focus = 10.0;
    let aperture = 0.1;

    let cam = Camera::new(
        look_from,
        look_at,
        v_up,
        20.0,
        ASPECT_RATIO,
        aperture,
        distance_to_focus,
    );

    // Render
    let file = File::create("image.ppm").unwrap();
    let mut file = BufWriter::new(file);

    writeln!(&mut file, "P3\n{} {}\n255", IMAGE_WIDTH.0, IMAGE_HEIGHT.0).unwrap();

    let progress_bar = ProgressBar::new((IMAGE_HEIGHT.0 * IMAGE_WIDTH.0) as u64);
    progress_bar.set_style(
        ProgressStyle::default_bar().template(
            "[{elapsed_precise} / {eta_precise}] {wide_bar} {pos:>7}/{len:7} ({per_sec})",
        ),
    );

    progress_bar.set_draw_delta(IMAGE_WIDTH.0 as u64);

    let pixel_range: Vec<_> = iproduct!((0..IMAGE_HEIGHT.0).rev(), 0..IMAGE_WIDTH.0).collect();
    let all_pixels = pixel_range
        .into_par_iter()
        .progress_with(progress_bar)
        .map(|(j, i)| evaluate_pixel(&world, &cam, j, i));

    let all_pixels: Vec<_> = all_pixels.collect();
    all_pixels
        .into_iter()
        .for_each(|pixel| write_color(&mut file, pixel, SAMPLES_PER_PIXEL).unwrap());
}

fn evaluate_pixel(
    world: &Vec<Box<dyn Hittable>>,
    cam: &Camera,
    pixel_row: usize,
    pixel_column: usize,
) -> Vec3 {
    let mut rng = thread_rng();
    let mut pixel_color = Color::new(0.0, 0.0, 0.0);
    for _ in 0..SAMPLES_PER_PIXEL {
        let u = (pixel_column as f64 + rng.gen::<f64>()) / ((IMAGE_WIDTH.0 - 1) as f64);
        let v = (pixel_row as f64 + rng.gen::<f64>()) / ((IMAGE_HEIGHT.0 - 1) as f64);
        let r = cam.get_ray(u, v, &mut rng);
        pixel_color += ray_color(&r, world, &mut rng, MAX_DEPTH);
    }

    pixel_color
}

fn write_color<F: Write>(f: &mut F, pixel_color: Vec3, samples_per_pixel: usize) -> io::Result<()> {
    let r = pixel_color.x();
    let g = pixel_color.y();
    let b = pixel_color.z();

    // Divide the color by the number of samples and gamma-correct for gamma=2.0.
    let scale = 1.0 / samples_per_pixel as f64;
    let r = (scale * r).sqrt();
    let g = (scale * g).sqrt();
    let b = (scale * b).sqrt();

    let ir = (255.999 * clamp(r, 0.0, 0.999)) as u8;
    let ig = (255.999 * clamp(g, 0.0, 0.999)) as u8;
    let ib = (255.999 * clamp(b, 0.0, 0.999)) as u8;

    writeln!(f, "{} {} {}", ir, ig, ib)
}

fn ray_color(r: &Ray, world: &impl HittableVec, rng: &mut ThreadRng, depth: usize) -> Color {
    if depth == 0 {
        return Color::new(0.0, 0.0, 0.0);
    }

    if let Some(hit_record) = world.hit(r, 0.001, f64::INFINITY) {
        if let Some(scatter) = hit_record.material.scatter(r, &hit_record, rng) {
            return scatter.attenuation * ray_color(&scatter.scattered_ray, world, rng, depth - 1);
        }
        return Color::new(0.0, 0.0, 0.0);
    }
    let unit_direction = r.direction().unit_vector();
    let t = 0.5 * (unit_direction.y() + 1.0);

    (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * Color::new(0.5, 0.7, 1.0)
}

struct Camera {
    origin: Point3,
    lower_left_corner: Point3,
    horizontal: Vec3,
    vertical: Vec3,
    u: Vec3,
    v: Vec3,
    _w: Vec3,
    lens_radius: f64,
}

impl Camera {
    pub fn new(
        lookfrom: Point3,
        lookat: Point3,
        vup: Vec3,
        vfow: f64,
        aspect_ratio: f64,
        aperture: f64,
        focus_dist: f64,
    ) -> Self {
        let theta = vfow.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = aspect_ratio * viewport_height;

        let w = (lookfrom - lookat).unit_vector();
        let u = vup.cross(&w).unit_vector();
        let v = w.cross(&u);

        let origin = lookfrom;
        let horizontal = focus_dist * viewport_width * u;
        let vertical = focus_dist * viewport_height * v;
        let lower_left_corner = origin - horizontal / 2.0 - vertical / 2.0 - focus_dist * w;

        let lens_radius = aperture / 2.0;

        Self {
            origin,
            lower_left_corner,
            horizontal,
            vertical,
            u,
            v,
            _w: w,
            lens_radius,
        }
    }

    pub fn get_ray(&self, s: f64, t: f64, rng: &mut ThreadRng) -> Ray {
        let rd = self.lens_radius * Vec3::random_in_unit_disk(rng);
        let offset = self.u * rd.x() + self.v * rd.y();
        Ray::new(
            self.origin + offset,
            self.lower_left_corner + s * self.horizontal + t * self.vertical - self.origin - offset,
        )
    }
}

fn clamp(x: f64, min: f64, max: f64) -> f64 {
    if x < min {
        min
    } else if x > max {
        max
    } else {
        x
    }
}
