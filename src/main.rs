use std::{
    fs::File,
    io,
    io::{BufWriter, Write},
};

use clap::Clap;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rand::thread_rng;
use rayon::prelude::*;
use raytracer_weekend_lib::{render, vec3::Vec3, Scene};

const ASPECT_RATIO: f64 = 1.0; // 16.0 / 9.0;
const IMAGE_WIDTH: usize = 800;
const IMAGE_HEIGHT: usize = (IMAGE_WIDTH as f64 / ASPECT_RATIO) as usize;
const SAMPLES_PER_PIXEL: usize = 100; //100;

const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");
const CRATE_AUTHOR: &str = env!("CARGO_PKG_AUTHORS");

/// My raytracer, based on the book series on the interwebs.
#[derive(Clap)]
#[clap(version = CRATE_VERSION, author = CRATE_AUTHOR)]
struct Opts {
    #[clap(default_value = "CornellBox")]
    scene: Scene,
}

fn main() {
    let opts: Opts = Opts::parse();

    let pixel_count = (IMAGE_WIDTH * IMAGE_HEIGHT) as u64;
    let progress_bar = ProgressBar::new(pixel_count);
    progress_bar.set_style(
        ProgressStyle::default_bar().template(
            "[{elapsed_precise} / {eta_precise}] {wide_bar} {pos:>7}/{len:7} ({per_sec})",
        ),
    );

    progress_bar.set_draw_delta(pixel_count / 100);

    let (world, cam, background) = opts.scene.generate(
        (IMAGE_WIDTH as f64) / (IMAGE_HEIGHT as f64),
        &mut thread_rng(),
    );

    let all_pixels: Vec<_> = render(
        world,
        cam,
        background,
        IMAGE_WIDTH,
        IMAGE_HEIGHT,
        SAMPLES_PER_PIXEL,
    )
    .progress_with(progress_bar)
    .collect();

    let file = File::create("image.ppm").unwrap();
    let mut file = BufWriter::new(file);

    writeln!(&mut file, "P3\n{} {}\n255", IMAGE_WIDTH, IMAGE_HEIGHT).unwrap();

    all_pixels
        .into_iter()
        .for_each(|pixel| write_color(&mut file, pixel, SAMPLES_PER_PIXEL).unwrap());
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

    let ir = (255.999 * r.clamp(0.0, 0.999)) as u8;
    let ig = (255.999 * g.clamp(0.0, 0.999)) as u8;
    let ib = (255.999 * b.clamp(0.0, 0.999)) as u8;

    writeln!(f, "{} {} {}", ir, ig, ib)
}
