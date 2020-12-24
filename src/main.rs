mod ray;
mod vec3;

#[macro_use]
extern crate derive_more;

use {
    indicatif::ProgressIterator,
    std::{
        fs::File,
        io::{self, Write},
    },
    vec3::{Color, Vec3},
};

fn main() {
    const IMAGE_WIDTH: i32 = 256;
    const IMAGE_HEIGHT: i32 = 256;

    let mut file = File::create("image.ppm").unwrap();

    writeln!(&file, "P3\n{} {}\n255", IMAGE_WIDTH, IMAGE_HEIGHT).unwrap();

    for j in (0..IMAGE_HEIGHT).rev().progress() {
        for i in 0..IMAGE_WIDTH {
            let pixel_color = Color::new(
                (i as f64) / ((IMAGE_WIDTH - 1) as f64),
                (j as f64) / ((IMAGE_HEIGHT - 1) as f64),
                0.25,
            );

            write_color(&mut file, pixel_color);
        }
    }
}

fn write_color<F: Write>(f: &mut F, pixel_color: Vec3) -> io::Result<()> {
    let ir = (255.999 * pixel_color.x()) as u8;
    let ig = (255.999 * pixel_color.y()) as u8;
    let ib = (255.999 * pixel_color.z()) as u8;

    writeln!(f, "{} {} {}", ir, ig, ib)
}
