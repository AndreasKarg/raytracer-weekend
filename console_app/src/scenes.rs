use std::path::PathBuf;
use clap::Subcommand;
use rand::prelude::*;
use raytracer_weekend_lib::bvh::BvhNode;
use raytracer_weekend_lib::hittable::volumes::ConstantMedium;
use raytracer_weekend_lib::vec3::{Color, Point3, Vec3};
use raytracer_weekend_saveload::{CameraDescriptor, World};
use raytracer_weekend_saveload::hittable::{BvhNodeDescriptor, ConstantMediumDescriptor, CuboidDescriptor, HittableDescriptor, MovingSphereDescriptor, SphereDescriptor, TranslationDescriptor, WavefrontObjDescriptor, XYRectangleDescriptor, XZRectangleDescriptor, YRotationDescriptor};
use raytracer_weekend_saveload::material::{DielectricDescriptor, DiffuseLightDescriptor, LambertianDescriptor, MaterialDescriptor, MetalDescriptor};
use raytracer_weekend_saveload::texture::{CheckerDescriptor, ImageTextureDescriptor, NoiseDescriptor, SolidColorDescriptor};

#[derive(Debug, Clone, Subcommand)]
pub enum Scene {
    JumpyBalls,
    TwoSpheres,
    TwoPerlinSpheres,
    Earth,
    // SimpleLight,
    // CornellBox,
    // SmokeyCornellBox,
    Book2FinalScene,
    AnimatedBook2FinalScene,
    // SimpleTriangle,
    WavefrontCowObj,
    // WavefrontSuspensionObj,
    TexturedMonument,
}

impl Scene {
    pub fn generate(&self, aspect_ratio: f32, rng: &mut ThreadRng) -> World {
        let generator = match self {
            Scene::JumpyBalls => jumpy_balls,
            Scene::TwoSpheres => two_spheres,
            Scene::TwoPerlinSpheres => two_perlin_spheres,
            Scene::Earth => earth,
            // Scene::SimpleLight => simple_light,
            // Scene::CornellBox => cornell_box,
            // Scene::SmokeyCornellBox => smokey_cornell_box,
            Scene::Book2FinalScene => book2_final_scene,
            Scene::AnimatedBook2FinalScene => animated_book2_final,
            // Scene::SimpleTriangle => simple_triangle,
            Scene::WavefrontCowObj => wavefront_cow_obj,
            // Scene::WavefrontSuspensionObj => wavefront_suspension_obj,
            Scene::TexturedMonument => textured_monument,
        };

        generator(aspect_ratio, rng)
    }
}

pub fn jumpy_balls(aspect_ratio: f32, rng: &mut ThreadRng) -> World {
    let checker = Box::new(CheckerDescriptor::new(
        SolidColorDescriptor::new_rgb(0.2, 0.3, 0.1),
        SolidColorDescriptor::new_rgb(0.9, 0.9, 0.9),
        10.0,
    ));
    let material_ground = LambertianDescriptor::new(checker);
    let lambertian = LambertianDescriptor::new_solid_color(Color::new(0.4, 0.2, 0.1));
    let glass = DielectricDescriptor::new(1.5);
    let metal = MetalDescriptor::new(Color::new(0.7, 0.6, 0.5), 0.0);

    let mut world: Vec<Box<dyn HittableDescriptor>> = vec![
        Box::new(SphereDescriptor::new(
            Point3::new(0.0, -1000.0, 0.0),
            1000.0,
            Box::new(material_ground),
        )),
        Box::new(SphereDescriptor::new(
            Point3::new(-4.0, 0.2, 0.1),
            1.0,
            Box::new(lambertian),
        )),
        Box::new(SphereDescriptor::new(
            Point3::new(0.0, 1.0, 0.0),
            1.0,
            Box::new(glass.clone()),
        )),
        Box::new(SphereDescriptor::new(
            Point3::new(0.0, 1.0, 0.0),
            -0.95,
            Box::new(glass),
        )),
        Box::new(SphereDescriptor::new(
            Point3::new(4.0, 1.0, 0.0),
            1.0,
            Box::new(metal),
        )),
    ];

    for a in -11..11 {
        for b in -11..11 {
            let a = a as f32;
            let b = b as f32;

            let center = Point3::new(a + 0.9 * rng.gen::<f32>(), 0.2, b + 0.9 * rng.gen::<f32>());

            if (center - Point3::new(4.0, 0.2, 0.0)).length() <= 0.9 {
                continue;
            }

            let sphere_material: Box<dyn MaterialDescriptor>;

            let choose_mat: f64 = rng.gen();
            if choose_mat < 0.8 {
                let albedo = Color::random(rng) * Color::random(rng);
                sphere_material = Box::new(LambertianDescriptor::new_solid_color(albedo));
            } else if choose_mat < 0.95 {
                let albedo = Color::random_min_max(rng, 0.5..1.0);
                let fuzz = rng.gen_range(0.0..0.5);
                sphere_material = Box::new(MetalDescriptor::new(albedo, fuzz));
            } else {
                sphere_material = Box::new(DielectricDescriptor::new(1.5));
            }

            let center2 = center + Vec3::new(0.0, rng.gen_range(0.0..0.5), 0.0);

            let sphere = Box::new(MovingSphereDescriptor::new(
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

    let cam = CameraDescriptor::new(
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

    World { geometry: world, cameras: vec![cam], background: DEFAULT_BACKGROUND }
}

pub fn two_spheres(aspect_ratio: f32, _rng: &mut ThreadRng) -> World {
    // World
    let checker = Box::new(CheckerDescriptor::new(
        SolidColorDescriptor::new_rgb(0.2, 0.3, 0.1),
        SolidColorDescriptor::new_rgb(0.9, 0.9, 0.9),
        10.0,
    ));
    let material_ground = LambertianDescriptor::new(checker);

    let world: Vec<Box<dyn HittableDescriptor>> = vec![
        Box::new(SphereDescriptor::new(
            Point3::new(0.0, -10.0, 0.0),
            10.0,
            Box::new(material_ground.clone()),
        )),
        Box::new(SphereDescriptor::new(
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

    let cam = CameraDescriptor::new(
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

    World { geometry: world, cameras: vec![cam], background: DEFAULT_BACKGROUND }
}

pub fn two_perlin_spheres(aspect_ratio: f32, rng: &mut ThreadRng) -> World {
    // World
    let perlin_material = Box::new(NoiseDescriptor::new(4.0));
    let material_ground = LambertianDescriptor::new(perlin_material);

    let world: Vec<Box<dyn HittableDescriptor>> = vec![
        Box::new(SphereDescriptor::new(
            Point3::new(0.0, -1000.0, 0.0),
            1000.0,
            Box::new(material_ground.clone()),
        )),
        Box::new(SphereDescriptor::new(
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

    let cam = CameraDescriptor::new(
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

    World { geometry: world, cameras: vec![cam], background: DEFAULT_BACKGROUND }
}

pub fn earth(aspect_ratio: f32, _rng: &mut ThreadRng) -> World {
    // World
    let earth_texture = Box::new(ImageTextureDescriptor::new(PathBuf::from("models/earthmap.jpg")));
    let earth_surface = LambertianDescriptor::new(earth_texture);

    let world: Vec<Box<dyn HittableDescriptor>> = vec![Box::new(SphereDescriptor::new(
        Point3::new(0.0, 0.0, 0.0),
        2.0,
        Box::new(earth_surface),
    ))];

    // Camera
    let look_from = Point3::new(13.0, 2.0, 3.0);
    let look_at = Point3::new(0.0, 0.0, 0.0);
    let v_up = Vec3::new(0.0, 1.0, 0.0);
    let distance_to_focus = 10.0;
    let aperture = 0.0;
    let vfow = 20.0;
    let time0 = 0.0;
    let time1 = 1.0;

    let cam = CameraDescriptor::new(
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

    World { geometry: world, cameras: vec![cam], background: DEFAULT_BACKGROUND }
}
//
// pub fn simple_light(aspect_ratio: f32, rng: &mut ThreadRng) -> World {
//     // World
//     let earth_texture = ImageTextureDescriptor::open("models/earthmap.jpg").unwrap();
//     let earth_surface = DiffuseLightDescriptor::new(earth_texture);
//     // let earth_surface = DiffuseLightDescriptor::new(SolidColorDescriptor::new_rgb(4.0, 4.0, 4.0));
//
//     let perlin_material = NoiseDescriptor::new(Perlin::new(rng), 4.0);
//     let material_ground = LambertianDescriptor::new(perlin_material);
//
//     let world: Vec<Box<dyn HittableDescriptor>> = vec![
//         Box::new(SphereDescriptor::new(
//             Point3::new(0.0, -1000.0, 0.0),
//             1000.0,
//             Box::new(material_ground.clone()),
//         )),
//         Box::new(SphereDescriptor::new(
//             Point3::new(0.0, 2.0, 0.0),
//             2.0,
//             Box::new(material_ground),
//         )),
//         Box::new(XYRectangleDescriptor::new(
//             3.0,
//             5.0,
//             1.0,
//             3.0,
//             -2.0,
//             Box::new(earth_surface.clone()),
//         )),
//         Box::new(SphereDescriptor::new(
//             Point3::new(0.0, 6.0, 0.0),
//             2.0,
//             Box::new(earth_surface),
//         )),
//     ];
//
//     // Camera
//     let look_from = Point3::new(26.0, 3.0, 6.0);
//     let look_at = Point3::new(0.0, 2.0, 0.0);
//     let v_up = Vec3::new(0.0, 1.0, 0.0);
//     let distance_to_focus = 10.0;
//     let aperture = 0.0;
//     let vfow = 20.0;
//     let time0 = 0.0;
//     let time1 = 1.0;
//
//     let cam =CameraDescriptor::new(
//         look_from,
//         look_at,
//         v_up,
//         vfow,
//         aspect_ratio,
//         aperture,
//         distance_to_focus,
//         time0,
//         time1,
//     );
//
//     World { geometry: world, cameras: vec![cam], background: Color::new(0.0, 0.0, 0.0) }
// }
//
// pub fn cornell_box(aspect_ratio: f32, _rng: &mut ThreadRng) -> World {
//     // World
//     let red = Box::new(LambertianDescriptor::new_solid_color(Color::new(0.65, 0.05, 0.05)));
//     let white = Box::new(LambertianDescriptor::new_solid_color(Color::new(0.73, 0.73, 0.73)));
//     let green = Box::new(LambertianDescriptor::new_solid_color(Color::new(0.12, 0.45, 0.15)));
//     let light = Box::new(DiffuseLightDescriptor::new(SolidColorDescriptor::new_rgb(15.0, 15.0, 15.0)));
//
//     let box1 = CuboidDescriptor::new(
//         Point3::new(0.0, 0.0, 0.0),
//         Point3::new(165.0, 330.0, 165.0),
//         white.clone(),
//     )
//         .rotate_y(15.0)
//         .translate(Vec3::new(265.0, 0.0, 295.0));
//
//     let box2 = CuboidDescriptor::new(
//         Point3::new(0.0, 0.0, 0.0),
//         Point3::new(165.0, 165.0, 165.0),
//         white.clone(),
//     )
//         .rotate_y(-18.0)
//         .translate(Vec3::new(130.0, 0.0, 65.0));
//
//     let world: Vec<Box<dyn HittableDescriptor>> = vec![
//         Box::new(YZRectangleDescriptor::new(0.0, 555.0, 0.0, 555.0, 555.0, green)),
//         Box::new(YZRectangleDescriptor::new(0.0, 555.0, 0.0, 555.0, 0.0, red)),
//         Box::new(XZRectangleDescriptor::new(213.0, 343.0, 227.0, 332.0, 554.0, light)),
//         Box::new(XZRectangleDescriptor::new(0.0, 555.0, 0.0, 555.0, 0.0, white.clone())),
//         Box::new(XZRectangleDescriptor::new(
//             0.0,
//             555.0,
//             0.0,
//             555.0,
//             555.0,
//             white.clone(),
//         )),
//         Box::new(XYRectangleDescriptor::new(0.0, 555.0, 0.0, 555.0, 555.0, white)),
//         Box::new(box1),
//         Box::new(box2),
//     ];
//
//     // Camera
//     let look_from = Point3::new(278.0, 278.0, -800.0);
//     let look_at = Point3::new(278.0, 278.0, 0.0);
//     let v_up = Vec3::new(0.0, 1.0, 0.0);
//     let distance_to_focus = 10.0;
//     let aperture = 0.0;
//     let vfow = 40.0;
//     let time0 = 0.0;
//     let time1 = 1.0;
//
//     let cam =CameraDescriptor::new(
//         look_from,
//         look_at,
//         v_up,
//         vfow,
//         aspect_ratio,
//         aperture,
//         distance_to_focus,
//         time0,
//         time1,
//     );
//
//     World { geometry: world, cameras: vec![cam], background: Color::new(0.0, 0.0, 0.0) }
// }
//
// pub fn smokey_cornell_box(aspect_ratio: f32, _rng: &mut ThreadRng) -> World {
//     // World
//     let red = Box::new(LambertianDescriptor::new_solid_color(Color::new(0.65, 0.05, 0.05)));
//     let white = Box::new(LambertianDescriptor::new_solid_color(Color::new(0.73, 0.73, 0.73)));
//     let green = Box::new(LambertianDescriptor::new_solid_color(Color::new(0.12, 0.45, 0.15)));
//     let light = Box::new(DiffuseLightDescriptor::new(SolidColorDescriptor::new_rgb(7.0, 7.0, 7.0)));
//
//     let box1 = CuboidDescriptor::new(
//         Point3::new(0.0, 0.0, 0.0),
//         Point3::new(165.0, 330.0, 165.0),
//         white.clone(),
//     )
//         .rotate_y(15.0)
//         .translate(Vec3::new(265.0, 0.0, 295.0));
//
//     let box2 = CuboidDescriptor::new(
//         Point3::new(0.0, 0.0, 0.0),
//         Point3::new(165.0, 165.0, 165.0),
//         white.clone(),
//     )
//         .rotate_y(-18.0)
//         .translate(Vec3::new(130.0, 0.0, 65.0));
//
//     let box1 = ConstantMedium::new(box1, 0.005, SolidColorDescriptor::new_rgb(0.0, 0.0, 0.0));
//     let box2 = ConstantMedium::new(box2, 0.005, SolidColorDescriptor::new_rgb(1.0, 1.0, 1.0));
//
//     let world: Vec<Box<dyn HittableDescriptor>> = vec![
//         Box::new(YZRectangleDescriptor::new(0.0, 555.0, 0.0, 555.0, 555.0, green)),
//         Box::new(YZRectangleDescriptor::new(0.0, 555.0, 0.0, 555.0, 0.0, red)),
//         Box::new(XZRectangleDescriptor::new(113.0, 443.0, 127.0, 432.0, 554.0, light)),
//         Box::new(XZRectangleDescriptor::new(0.0, 555.0, 0.0, 555.0, 0.0, white.clone())),
//         Box::new(XZRectangleDescriptor::new(
//             0.0,
//             555.0,
//             0.0,
//             555.0,
//             555.0,
//             white.clone(),
//         )),
//         Box::new(XYRectangleDescriptor::new(0.0, 555.0, 0.0, 555.0, 555.0, white)),
//         Box::new(box1),
//         Box::new(box2),
//     ];
//
//     // Camera
//     let look_from = Point3::new(278.0, 278.0, -800.0);
//     let look_at = Point3::new(278.0, 278.0, 0.0);
//     let v_up = Vec3::new(0.0, 1.0, 0.0);
//     let distance_to_focus = 10.0;
//     let aperture = 0.0;
//     let vfow = 40.0;
//     let time0 = 0.0;
//     let time1 = 1.0;
//
//     let cam =CameraDescriptor::new(
//         look_from,
//         look_at,
//         v_up,
//         vfow,
//         aspect_ratio,
//         aperture,
//         distance_to_focus,
//         time0,
//         time1,
//     );
//
//     World { geometry: world, cameras: vec![cam], background: Color::new(0.0, 0.0, 0.0) }
// }
//
pub fn book2_final_scene(aspect_ratio: f32, rng: &mut ThreadRng) -> World {
    let mut boxes1: Vec<Box<dyn HittableDescriptor>> = Vec::new();
    let ground = Box::new(LambertianDescriptor::new_solid_color(Color::new(0.48, 0.83, 0.53)));

    let boxes_per_side = 20;
    for i in 0..boxes_per_side {
        for j in 0..boxes_per_side {
            let i = i as f32;
            let j = j as f32;

            let w = 100.0;
            let x0 = -1000.0 + i * w;
            let z0 = -1000.0 + j * w;
            let y0 = 0.0;
            let x1 = x0 + w;
            let y1 = rng.gen_range(1.0..101.0);
            let z1 = z0 + w;

            boxes1.push(Box::new(CuboidDescriptor::new(
                Point3::new(x0, y0, z0),
                Point3::new(x1, y1, z1),
                ground.clone(),
            )));
        }
    }

    let mut objects: Vec<Box<dyn HittableDescriptor>> = Vec::new();

    objects.push(Box::new(BvhNodeDescriptor::new(boxes1, 0.0, 1.0)));

    let light = Box::new(DiffuseLightDescriptor::new(SolidColorDescriptor::new_rgb(7.0, 7.0, 7.0)));
    objects.push(Box::new(XZRectangleDescriptor::new(
        123.0, 423.0, 147.0, 412.0, 554.0, light,
    )));

    let center1 = Point3::new(400.0, 400.0, 200.0);
    let center2 = center1 + Vec3::new(30.0, 0.0, 0.0);
    let moving_sphere_material = Box::new(LambertianDescriptor::new_solid_color(Color::new(0.7, 0.3, 0.1)));
    objects.push(Box::new(MovingSphereDescriptor::new(
        center1,
        0.0,
        center2,
        1.0,
        50.0,
        moving_sphere_material,
    )));

    objects.push(Box::new(SphereDescriptor::new(
        Point3::new(260.0, 150.0, 45.0),
        50.0,
        Box::new(DielectricDescriptor::new(1.5)),
    )));
    objects.push(Box::new(SphereDescriptor::new(
        Point3::new(0.0, 150.0, 145.0),
        50.0,
        Box::new(MetalDescriptor::new(Color::new(0.8, 0.8, 0.9), 1.0)),
    )));

    let boundary = Box::new(SphereDescriptor::new(
        Point3::new(360.0, 150.0, 145.0),
        70.0,
        Box::new(DielectricDescriptor::new(1.5)),
    ));
    objects.push(boundary.clone());
    objects.push(Box::new(ConstantMediumDescriptor::new(
        boundary,
        0.2,
        SolidColorDescriptor::new_rgb(0.2, 0.4, 0.9),
    )));

    let boundary = Box::new(SphereDescriptor::new(
        Point3::new(0.0, 0.0, 0.0),
        5000.0,
        Box::new(DielectricDescriptor::new(1.5)),
    ));
    objects.push(Box::new(ConstantMediumDescriptor::new(
        boundary,
        0.0001,
        SolidColorDescriptor::new_rgb(1.0, 1.0, 1.0),
    )));

    let emat = Box::new(LambertianDescriptor::new(
        Box::new(ImageTextureDescriptor::new(PathBuf::from("models/earthmap.jpg"))),
    ));
    objects.push(Box::new(SphereDescriptor::new(
        Point3::new(400.0, 200.0, 400.0),
        100.0,
        emat,
    )));
    let pertext = Box::new(NoiseDescriptor::new(0.1));
    objects.push(Box::new(SphereDescriptor::new(
        Point3::new(220.0, 280.0, 300.0),
        80.0,
        Box::new(LambertianDescriptor::new(pertext)),
    )));

    let mut boxes2: Vec<Box<dyn HittableDescriptor>> = Vec::new();
    let white = Box::new(LambertianDescriptor::new(SolidColorDescriptor::new_rgb(0.73, 0.73, 0.73)));
    let ns = 1000;
    for _ in 0..ns {
        boxes2.push(Box::new(SphereDescriptor::new(
            Point3::random_min_max(rng, 0.0..165.0),
            10.0,
            white.clone(),
        )));
    }

    objects.push(Box::new(TranslationDescriptor::new(Box::new(
        YRotationDescriptor::new(Box::new(BvhNodeDescriptor::new(boxes2, 0.0, 1.0)), 15.0)),
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

    let cam = CameraDescriptor::new(
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

    World { geometry: objects, cameras: vec![cam], background: Color::new(0.0, 0.0, 0.0) }
}

pub fn animated_book2_final(
    aspect_ratio: f32,
    rng: &mut ThreadRng,
) -> World {
    let base_scene = book2_final_scene(aspect_ratio, rng);

    // Camera
    let look_at = Point3::new(278.0, 278.0, 278.0);
    let v_up = Vec3::new(0.0, 1.0, 0.0);
    let aperture = 1.0;
    let vfow = 40.0;
    let time0 = 0.0;
    let time1 = 1.0;

    let len_s = 3.0;
    let fps = 10.0;
    let frames = fps * len_s;

    let cameras: Vec<_> = (0..(frames as usize))
        .into_iter()
        .map(|frame| {
            let from_x = 478.0 - frame as f32 * (2.0 * 478.0) / frames;
            let from_y = 278.0;
            let from_z = -600.0;

            let look_from = (from_x, from_y, from_z).into();
            let distance_to_focus = (look_at - look_from).length();

            CameraDescriptor::new(
                look_from,
                look_at,
                v_up,
                vfow,
                aspect_ratio,
                aperture,
                distance_to_focus,
                time0,
                time1,
            )
        })
        .collect();

    let world: Vec<Box<dyn HittableDescriptor>> = vec![Box::new(BvhNodeDescriptor::new(base_scene.geometry, 0.0, 1.0))];

    World { geometry: world, cameras, background: base_scene.background }
}

// pub fn simple_triangle(aspect_ratio: f32, _rng: &mut ThreadRng) -> World {
//     // World
//     let checker = CheckerDescriptor::new(
//         SolidColorDescriptor::new_rgb(0.2, 0.3, 0.1),
//         SolidColorDescriptor::new_rgb(0.9, 0.9, 0.9),
//         10.0,
//     );
//     let material_ground = LambertianDescriptor::new(checker);
//
//     let world: Vec<Box<dyn HittableDescriptor>> = vec![
//         Box::new(SphereDescriptor::new(
//             Point3::new(0.0, -10.0, 0.0),
//             10.0,
//             Box::new(material_ground),
//         )),
//         Box::new(Triangle::new_flat_shaded(
//             [
//                 Point3::new(-5.0, 0.0, 5.0),
//                 Point3::new(0.0, 7.0, 0.0),
//                 Point3::new(5.0, 0.0, -5.0),
//             ],
//             Arc::new(LambertianDescriptor::new(UVDebug::new())),
//         )),
//     ];
//
//     // Camera
//     let look_from = Point3::new(13.0, 2.0, 3.0);
//     let look_at = Point3::new(0.0, 2.5, 0.0);
//     let v_up = Vec3::new(0.0, 1.0, 0.0);
//     let distance_to_focus = 10.0;
//     let aperture = 0.0;
//     let vfow = 40.0;
//     let time0 = 0.0;
//     let time1 = 1.0;
//
//     let cam =CameraDescriptor::new(
//         look_from,
//         look_at,
//         v_up,
//         vfow,
//         aspect_ratio,
//         aperture,
//         distance_to_focus,
//         time0,
//         time1,
//     );
//
//     World { geometry: world, cameras: vec![cam], background: DEFAULT_BACKGROUND }
// }
//
pub fn wavefront_cow_obj(aspect_ratio: f32, rng: &mut ThreadRng) -> World {
    // World
    let checker = Box::new(CheckerDescriptor::new(
        SolidColorDescriptor::new_rgb(0.2, 0.3, 0.1),
        SolidColorDescriptor::new_rgb(0.9, 0.9, 0.9),
        10.0,
    ));
    let material_ground = LambertianDescriptor::new(checker);

    let cow = Box::new(WavefrontObjDescriptor::new(PathBuf::from("models/cow-nonormals.obj")));
    let cow = Box::new(TranslationDescriptor::new(cow, Vec3::new(0.0, 2.5, 0.0))) as Box<dyn HittableDescriptor>;

    let world: Vec<Box<dyn HittableDescriptor>> = vec![
        Box::new(SphereDescriptor::new(
            Point3::new(0.0, -10.6, 0.0),
            10.0,
            Box::new(material_ground),
        )),
        Box::new(XYRectangleDescriptor::new(
            1.0,
            5.0,
            1.0,
            7.0,
            5.0,
            Box::new(DiffuseLightDescriptor::new(SolidColorDescriptor::new_rgb(1.4, 1.3, 1.3))),
        )),
        cow,
    ];

    // Camera
    let look_from = Point3::new(13.0, 2.0, 3.0);
    let look_at = Point3::new(0.0, 2.5, 0.0);
    let v_up = Vec3::new(0.0, 1.0, 0.0);
    let distance_to_focus = 10.0;
    let aperture = 0.0;
    let vfow = 40.0;
    let time0 = 0.0;
    let time1 = 1.0;

    let cam = CameraDescriptor::new(
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

    World { geometry: world, cameras: vec![cam], background: Color::new_const(0.085, 0.1, 0.125) }
}
//
// pub fn wavefront_suspension_obj(aspect_ratio: f32, rng: &mut ThreadRng) -> World {
//     // World
//     let suspension = load_wavefront_obj("models/Normals_Try3.obj", rng).unwrap();
//     let suspension =
//         Box::new(Translation::new(suspension, Vec3::new(0.0, 2.5, 0.0))) as Box<dyn HittableDescriptor>;
//
//     let world: Vec<Box<dyn HittableDescriptor>> = vec![
//         Box::new(XYRectangleDescriptor::new(
//             -5.0,
//             5.0,
//             -7.0,
//             7.0,
//             1.0,
//             Box::new(DiffuseLightDescriptor::new(SolidColorDescriptor::new_rgb(1.2, 1.0, 1.0))),
//         )),
//         suspension,
//     ];
//
//     // Camera
//     let look_from = Point3::new(0.5, 2.5, 0.8);
//     let look_at = Point3::new(-0.1, 2.3, 0.15);
//     let v_up = Vec3::new(0.0, 1.0, 0.0);
//     let distance_to_focus = 10.0;
//     let aperture = 0.0;
//     let vfow = 40.0;
//     let time0 = 0.0;
//     let time1 = 1.0;
//
//     let cam =CameraDescriptor::new(
//         look_from,
//         look_at,
//         v_up,
//         vfow,
//         aspect_ratio,
//         aperture,
//         distance_to_focus,
//         time0,
//         time1,
//     );
//
//     World { geometry: world, cameras: vec![cam], background: Color::new_const(0.085, 0.1, 0.125) }
// }
//
pub fn textured_monument(aspect_ratio: f32, rng: &mut ThreadRng) -> World {
    // World
    let monument = Box::new(TranslationDescriptor::new(
        Box::new(WavefrontObjDescriptor::new(PathBuf::from("models/monument_downscaled_polygon_reduced.obj"))),
        Vec3::new(0.0, 0.0, -19.0),
    ));

    let world: Vec<Box<dyn HittableDescriptor>> = vec![
        Box::new(XYRectangleDescriptor::new(
            -15.0,
            15.0,
            -17.0,
            17.0,
            33.0,
            Box::new(DiffuseLightDescriptor::new(SolidColorDescriptor::new_rgb(1.2, 1.0, 1.0))),
        )),
        monument,
    ];

    // Camera
    let look_from = Point3::new(-5.0, -30.0, 25.0);
    let look_at = Point3::new(0.0, 0.0, 5.0);
    let v_up = Vec3::new(1.0, 0.0, 0.0);
    let distance_to_focus = 10.0;
    let aperture = 0.0;
    let vfow = 40.0;
    let time0 = 0.0;
    let time1 = 1.0;

    let cam = CameraDescriptor::new(
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

    World { geometry: world, cameras: vec![cam], background: Color::new_const(0.085, 0.1, 0.125) }
}

static DEFAULT_BACKGROUND: Color = Color::new_const(0.7, 0.8, 1.00);
