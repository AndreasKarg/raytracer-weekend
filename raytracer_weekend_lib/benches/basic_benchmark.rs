use criterion::{criterion_group, criterion_main, Criterion};
use rand::prelude::*;
use rayon::prelude::*;
use raytracer_weekend_lib::{
    bvh::BvhNode,
    camera::Camera,
    hittable::{
        rectangular::{Cuboid, XZRectangle},
        spherical::{MovingSphere, Sphere},
        transformations::{Translation, YRotation},
        volumes::ConstantMedium,
        Hittable,
    },
    image_texture::ImageTexture,
    light_source::DiffuseLight,
    material::{Dielectric, Lambertian, Metal},
    perlin::Perlin,
    texture::{Noise, SolidColor},
    vec3::{Color, Point3, Vec3},
    Raytracer,
};

pub fn book2_final_scene(
    aspect_ratio: f64,
    rng: &mut impl Rng,
) -> (Vec<Box<dyn Hittable>>, Camera, Color) {
    let mut boxes1: Vec<Box<dyn Hittable>> = Vec::new();
    let ground = Box::new(Lambertian::new_solid_color(Color::new(0.48, 0.83, 0.53)));

    let boxes_per_side = 20;
    for i in 0..boxes_per_side {
        for j in 0..boxes_per_side {
            let i = i as f64;
            let j = j as f64;

            let w = 100.0;
            let x0 = -1000.0 + i * w;
            let z0 = -1000.0 + j * w;
            let y0 = 0.0;
            let x1 = x0 + w;
            let y1 = rng.gen_range(1.0..101.0);
            let z1 = z0 + w;

            boxes1.push(Box::new(Cuboid::new(
                Point3::new(x0, y0, z0),
                Point3::new(x1, y1, z1),
                ground.clone(),
            )));
        }
    }

    let mut objects: Vec<Box<dyn Hittable>> = Vec::new();

    objects.push(Box::new(BvhNode::new(boxes1, 0.0, 1.0, rng)));

    let light = Box::new(DiffuseLight::new(SolidColor::new_rgb(7.0, 7.0, 7.0)));
    objects.push(Box::new(XZRectangle::new(
        123.0, 423.0, 147.0, 412.0, 554.0, light,
    )));

    let center1 = Point3::new(400.0, 400.0, 200.0);
    let center2 = center1 + Vec3::new(30.0, 0.0, 0.0);
    let moving_sphere_material = Box::new(Lambertian::new_solid_color(Color::new(0.7, 0.3, 0.1)));
    objects.push(Box::new(MovingSphere::new(
        center1,
        0.0,
        center2,
        1.0,
        50.0,
        moving_sphere_material,
    )));

    objects.push(Box::new(Sphere::new(
        Point3::new(260.0, 150.0, 45.0),
        50.0,
        Box::new(Dielectric::new(1.5)),
    )));
    objects.push(Box::new(Sphere::new(
        Point3::new(0.0, 150.0, 145.0),
        50.0,
        Box::new(Metal::new(Color::new(0.8, 0.8, 0.9), 1.0)),
    )));

    let boundary = Sphere::new(
        Point3::new(360.0, 150.0, 145.0),
        70.0,
        Box::new(Dielectric::new(1.5)),
    );
    objects.push(Box::new(boundary.clone()));
    objects.push(Box::new(ConstantMedium::new(
        boundary,
        0.2,
        SolidColor::new_rgb(0.2, 0.4, 0.9),
    )));

    let boundary = Sphere::new(
        Point3::new(0.0, 0.0, 0.0),
        5000.0,
        Box::new(Dielectric::new(1.5)),
    );
    objects.push(Box::new(ConstantMedium::new(
        boundary,
        0.0001,
        SolidColor::new_rgb(1.0, 1.0, 1.0),
    )));

    let emat = Box::new(Lambertian::new(ImageTexture::open("earthmap.jpg").unwrap()));
    objects.push(Box::new(Sphere::new(
        Point3::new(400.0, 200.0, 400.0),
        100.0,
        emat,
    )));
    let pertext = Noise::new(Perlin::new(rng), 0.1);
    objects.push(Box::new(Sphere::new(
        Point3::new(220.0, 280.0, 300.0),
        80.0,
        Box::new(Lambertian::new(pertext)),
    )));

    let mut boxes2: Vec<Box<dyn Hittable>> = Vec::new();
    let white = Box::new(Lambertian::new(SolidColor::new_rgb(0.73, 0.73, 0.73)));
    let ns = 1000;
    for _ in 0..ns {
        boxes2.push(Box::new(Sphere::new(
            Point3::random_min_max(rng, 0.0..165.0),
            10.0,
            white.clone(),
        )));
    }

    objects.push(Box::new(Translation::new(
        YRotation::new(BvhNode::new(boxes2, 0.0, 1.0, rng), 15.0),
        Vec3::new(-100.0, 270.0, 395.0),
    )));

    // Camera
    let look_from = Point3::new(478.0, 278.0, -600.0);
    let look_at = Point3::new(278.0, 278.0, 0.0);
    let v_up = Vec3::new(0.0, 1.0, 0.0);
    let distance_to_focus = (look_at - look_from).length();
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

    (objects, cam, Color::new(0.0, 0.0, 0.0))
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(1337);
    let (world, cam, background) = book2_final_scene(16.0 / 9.0, &mut rng);

    c.bench_function("book2_final_scene", |b| {
        b.iter(|| {
            let raytracer = Raytracer::new(&world, &cam, background, 40, 22, 100);
            raytracer.render().for_each(drop);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
