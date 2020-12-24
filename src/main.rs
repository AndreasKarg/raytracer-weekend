use {
    indicatif::ProgressIterator,
    std::{fs::File, io::Write},
};

mod vec3;

fn main() {
    const IMAGE_WIDTH: i32 = 256;
    const IMAGE_HEIGHT: i32 = 256;

    let file = File::create("image.ppm").unwrap();

    writeln!(&file, "P3\n{} {}\n255", IMAGE_WIDTH, IMAGE_HEIGHT).unwrap();

    for j in (0..IMAGE_HEIGHT).rev().progress() {
        for i in 0..IMAGE_WIDTH {
            let r = (i as f64) / ((IMAGE_WIDTH - 1) as f64);
            let g = (j as f64) / ((IMAGE_HEIGHT - 1) as f64);
            let b = 0.25;

            let ir = (255.999 * r) as u8;
            let ig = (255.999 * g) as u8;
            let ib = (255.999 * b) as u8;

            writeln!(&file, "{} {} {}", ir, ig, ib).unwrap();
        }
    }
}
