#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

use alloc::{boxed::Box, vec, vec::Vec};
use core::alloc::Layout;

use alloc_cortex_m::CortexMHeap;
use discovery_app as _;
use heapless;
use postcard::{to_vec, to_vec_cobs};
use rand::{prelude::SmallRng, Rng, SeedableRng};
use raytracer_weekend_lib::{
    bvh::BvhNode,
    camera::Camera,
    hittable::{
        rectangular::{Cuboid, XYRectangle, XZRectangle, YZRectangle},
        spherical::{MovingSphere, Sphere},
        transformations::{Transformable, Translation, YRotation},
        triangular::Triangle,
        volumes::ConstantMedium,
        Hittable,
    },
    light_source::DiffuseLight,
    material::{Dielectric, Lambertian, Material, Metal},
    perlin::Perlin,
    texture::{Checker, Noise, SolidColor, UVDebug},
    vec3::{Color, Point3, Vec3},
    ProgressMessage, Raytracer,
};
use stm32l4xx_hal::{
    pac::USART2,
    prelude::*,
    serial::{Config, Rx, Serial, Tx},
};

// global logger + panicking-behavior + memory layout

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("Hello, world!");

    let start = cortex_m_rt::heap_start() as usize;
    let size = 65536; // in bytes
    unsafe { ALLOCATOR.init(start, size) }

    defmt::info!("Allocator set up...");

    let (mut tx, _rx) = setup_usart2();

    let mut rng = SmallRng::seed_from_u64(1234);

    let image_width = 64;
    let image_height = 64;

    let aspect_ratio = image_width as f32 / image_height as f32;

    let samples_per_pixel = 200;

    let image_start_msg: heapless::Vec<u8, 256> = to_vec_cobs(&ProgressMessage::ImageStart {
        width: image_width,
        height: image_height,
        samples_per_pixel,
    })
    .unwrap();

    tx.bwrite_all(&[0x00, 0x00, 0x00, 0x00]).unwrap();
    tx.bwrite_all(&image_start_msg).unwrap();
    // tx.bflush().unwrap();

    defmt::info!("Creating world...");

    let (world, cams, background) = jumpy_balls(aspect_ratio, &mut rng);
    defmt::info!("World created.");

    for (frame_no, cam) in cams.iter().enumerate() {
        let raytracer = Raytracer::new(
            &world,
            &cam,
            background,
            image_width,
            image_height,
            samples_per_pixel,
        );

        let all_pixels = raytracer.render();

        for (idx, pixel) in all_pixels.enumerate() {
            /*if idx % 1 == 0*/
            {
                defmt::info!("{} pixels calculated!", idx);
                defmt::info!(
                    "row: {}, col: {}, color.r: {}, color.g: {}, color.b: {}",
                    pixel.row,
                    pixel.column,
                    pixel.color.x(),
                    pixel.color.y(),
                    pixel.color.z()
                );
            }

            let serialised: heapless::Vec<u8, 256> =
                to_vec_cobs(&ProgressMessage::Pixel(pixel)).unwrap();
            tx.bwrite_all(&serialised).unwrap();
            // tx.bflush();
        }
    }

    let serialised: heapless::Vec<u8, 256> = to_vec_cobs(&ProgressMessage::ImageEnd).unwrap();
    tx.bwrite_all(&serialised).unwrap();
    tx.bflush().unwrap();

    discovery_app::exit()
}

type World = (Vec<Box<dyn Hittable>>, Vec<Camera>, Color);
static DEFAULT_BACKGROUND: Color = Color::new_const(0.7, 0.8, 1.00);

pub fn jumpy_balls(aspect_ratio: f32, rng: &mut SmallRng) -> World {
    // World
    let red = Box::new(Lambertian::new_solid_color(Color::new(0.65, 0.05, 0.05)));
    let white = Box::new(Lambertian::new_solid_color(Color::new(0.73, 0.73, 0.73)));
    let green = Box::new(Lambertian::new_solid_color(Color::new(0.12, 0.45, 0.15)));
    let light = Box::new(DiffuseLight::new(SolidColor::new_rgb(7.0, 7.0, 7.0)));

    let box1 = Cuboid::new(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(165.0, 330.0, 165.0),
        white.clone(),
    )
    .rotate_y(15.0)
    .translate(Vec3::new(265.0, 0.0, 295.0));

    let box2 = Cuboid::new(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(165.0, 165.0, 165.0),
        white.clone(),
    )
    .rotate_y(-18.0)
    .translate(Vec3::new(130.0, 0.0, 65.0));

    let box1 = ConstantMedium::new(box1, 0.005, SolidColor::new_rgb(0.0, 0.0, 0.0));
    let box2 = ConstantMedium::new(box2, 0.005, SolidColor::new_rgb(1.0, 1.0, 1.0));

    let world: Vec<Box<dyn Hittable>> = vec![
        Box::new(YZRectangle::new(0.0, 555.0, 0.0, 555.0, 555.0, green)),
        Box::new(YZRectangle::new(0.0, 555.0, 0.0, 555.0, 0.0, red)),
        Box::new(XZRectangle::new(113.0, 443.0, 127.0, 432.0, 554.0, light)),
        Box::new(XZRectangle::new(0.0, 555.0, 0.0, 555.0, 0.0, white.clone())),
        Box::new(XZRectangle::new(
            0.0,
            555.0,
            0.0,
            555.0,
            555.0,
            white.clone(),
        )),
        Box::new(XYRectangle::new(0.0, 555.0, 0.0, 555.0, 555.0, white)),
        Box::new(box1),
        Box::new(box2),
    ];

    // Camera
    let look_from = Point3::new(278.0, 278.0, -800.0);
    let look_at = Point3::new(278.0, 278.0, 0.0);
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

    (world, vec![cam], Color::new(0.0, 0.0, 0.0))
}

fn setup_usart2() -> (Tx<USART2>, Rx<USART2>) {
    let p = stm32l4xx_hal::stm32::Peripherals::take().unwrap();

    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();
    let mut pwr = p.PWR.constrain(&mut rcc.apb1r1);

    let mut gpiod = p.GPIOD.split(&mut rcc.ahb2);

    // clock configuration using the default settings (all clocks run at 8 MHz)
    let clocks = rcc.cfgr.freeze(&mut flash.acr, &mut pwr);
    // TRY this alternate clock configuration (clocks run at nearly the maximum frequency)
    // let clocks = rcc
    //     .cfgr
    //     .sysclk(80.mhz())
    //     .pclk1(80.mhz())
    //     .pclk2(80.mhz())
    //     .freeze(&mut flash.acr, &mut pwr);

    let tx = gpiod.pd5.into_af7(&mut gpiod.moder, &mut gpiod.afrl);
    let rx = gpiod.pd6.into_af7(&mut gpiod.moder, &mut gpiod.afrl);

    let serial = Serial::usart2(
        p.USART2,
        (tx, rx),
        Config::default().baudrate(9_600.bps()),
        clocks,
        &mut rcc.apb1r1,
    );

    serial.split()
}

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[alloc_error_handler]
fn my_example_handler(layout: core::alloc::Layout) -> ! {
    panic!("memory allocation of {} bytes failed", layout.size())
}
