use rand::prelude::*;

use crate::{
    camera::Camera,
    hittable::{Hittable, MovingSphere, Sphere},
    material::{Dielectric, Material, Metal},
    perlin::Perlin,
    texture::{Checker, Noise, SolidColor},
    vec3::{Color, Point3, Vec3},
    Lambertian,
};

pub fn jumpy_balls(aspect_ratio: f64, rng: &mut ThreadRng) -> (Vec<Box<dyn Hittable>>, Camera) {
    let checker = Checker::new(
        SolidColor::new_rgb(0.2, 0.3, 0.1),
        SolidColor::new_rgb(0.9, 0.9, 0.9),
        10.0,
    );
    let material_ground = Lambertian::new(checker);
    let lambertian = Lambertian::new_solid_color(Color::new(0.4, 0.2, 0.1));
    let glass = Dielectric::new(1.5);
    let metal = Metal::new(Color::new(0.7, 0.6, 0.5), 0.0);

    let mut world: Vec<Box<dyn Hittable>> = vec![
        Box::new(Sphere::new(
            Point3::new(0.0, -1000.0, 0.0),
            1000.0,
            Box::new(material_ground),
        )),
        Box::new(Sphere::new(
            Point3::new(-4.0, 0.2, 0.1),
            1.0,
            Box::new(lambertian),
        )),
        Box::new(Sphere::new(
            Point3::new(0.0, 1.0, 0.0),
            1.0,
            Box::new(glass.clone()),
        )),
        Box::new(Sphere::new(
            Point3::new(0.0, 1.0, 0.0),
            -0.95,
            Box::new(glass),
        )),
        Box::new(Sphere::new(
            Point3::new(4.0, 1.0, 0.0),
            1.0,
            Box::new(metal),
        )),
    ];

    for a in -11..11 {
        for b in -11..11 {
            let a = a as f64;
            let b = b as f64;

            let center = Point3::new(a + 0.9 * rng.gen::<f64>(), 0.2, b + 0.9 * rng.gen::<f64>());

            if (center - Point3::new(4.0, 0.2, 0.0)).length() <= 0.9 {
                continue;
            }

            let sphere_material: Box<dyn Material>;

            let choose_mat: f64 = rng.gen();
            if choose_mat < 0.8 {
                let albedo = Color::random(rng) * Color::random(rng);
                sphere_material = Box::new(Lambertian::new_solid_color(albedo));
            } else if choose_mat < 0.95 {
                let albedo = Color::random_min_max(rng, 0.5..1.0);
                let fuzz = rng.gen_range(0.0..0.5);
                sphere_material = Box::new(Metal::new(albedo, fuzz));
            } else {
                sphere_material = Box::new(Dielectric::new(1.5));
            }

            let center2 = center + Vec3::new(0.0, rng.gen_range(0.0..0.5), 0.0);

            let sphere = Box::new(MovingSphere::new(
                center,
                0.0,
                center2,
                1.0,
                0.2,
                sphere_material,
            ));

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
        aspect_ratio,
        aperture,
        distance_to_focus,
        0.0,
        1.0,
    );

    (world, cam)
}

pub fn two_spheres(aspect_ratio: f64, _rng: &mut ThreadRng) -> (Vec<Box<dyn Hittable>>, Camera) {
    // World
    let checker = Checker::new(
        SolidColor::new_rgb(0.2, 0.3, 0.1),
        SolidColor::new_rgb(0.9, 0.9, 0.9),
        10.0,
    );
    let material_ground = Lambertian::new(checker);

    let world: Vec<Box<dyn Hittable>> = vec![
        Box::new(Sphere::new(
            Point3::new(0.0, -10.0, 0.0),
            10.0,
            Box::new(material_ground.clone()),
        )),
        Box::new(Sphere::new(
            Point3::new(0.0, 10.0, 0.0),
            10.0,
            Box::new(material_ground),
        )),
    ];

    // Camera
    let look_from = Point3::new(13.0, 2.0, 3.0);
    let look_at = Point3::new(0.0, 0.0, 0.0);
    let v_up = Vec3::new(0.0, 1.0, 0.0);
    let distance_to_focus = 10.0;
    let aperture = 0.0;
    let vfow = 40.0;
    let time0 = 0.0;
    let time1 = 1.0;

    let cam = Camera::new(
        look_from,
        look_at,
        v_up,
        vfow,
        aspect_ratio,
        aperture,
        distance_to_focus,
        time0,
        time1,
    );

    (world, cam)
}

pub fn two_perlin_spheres(
    aspect_ratio: f64,
    rng: &mut ThreadRng,
) -> (Vec<Box<dyn Hittable>>, Camera) {
    // World
    let perlin_material = Noise::new(Perlin::new(rng), 4.0);
    let material_ground = Lambertian::new(perlin_material);

    let world: Vec<Box<dyn Hittable>> = vec![
        Box::new(Sphere::new(
            Point3::new(0.0, -1000.0, 0.0),
            1000.0,
            Box::new(material_ground.clone()),
        )),
        Box::new(Sphere::new(
            Point3::new(0.0, 2.0, 0.0),
            2.0,
            Box::new(material_ground),
        )),
    ];

    // Camera
    let look_from = Point3::new(13.0, 2.0, 3.0);
    let look_at = Point3::new(0.0, 0.0, 0.0);
    let v_up = Vec3::new(0.0, 1.0, 0.0);
    let distance_to_focus = 10.0;
    let aperture = 0.0;
    let vfow = 40.0;
    let time0 = 0.0;
    let time1 = 1.0;

    let cam = Camera::new(
        look_from,
        look_at,
        v_up,
        vfow,
        aspect_ratio,
        aperture,
        distance_to_focus,
        time0,
        time1,
    );

    (world, cam)
}
