use clap::Clap;
use image::{Rgb, RgbImage};
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rand::thread_rng;
use rayon::prelude::*;
use raytracer_weekend_lib::{Raytracer, Scene};

const ASPECT_RATIO: f64 = 4.0 / 3.0;
const IMAGE_WIDTH: usize = 400;
const IMAGE_HEIGHT: usize = (IMAGE_WIDTH as f64 / ASPECT_RATIO) as usize;
const SAMPLES_PER_PIXEL: usize = 2000; //100;

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

    let (world, cams, background) = opts.scene.generate(
        (IMAGE_WIDTH as f64) / (IMAGE_HEIGHT as f64),
        &mut thread_rng(),
    );

    for (frame_no, cam) in cams.iter().enumerate() {
        let raytracer = Raytracer::new(
            &world,
            &cam,
            background,
            IMAGE_WIDTH,
            IMAGE_HEIGHT,
            SAMPLES_PER_PIXEL,
        );

        let progress_bar = ProgressBar::new(pixel_count);
        progress_bar.set_style(ProgressStyle::default_bar().template(
            "[[{msg}] {elapsed_precise} / {eta_precise}] {wide_bar} {pos:>7}/{len:7} ({per_sec})",
        ));
        progress_bar.set_draw_delta(pixel_count / 100);
        progress_bar.set_message(&format!("[{} / {}]", frame_no, cams.len()));
        let all_pixels: Vec<_> = raytracer.render().progress_with(progress_bar).collect();

        let mut image = RgbImage::new(IMAGE_WIDTH as u32, IMAGE_HEIGHT as u32);

        image
            .pixels_mut()
            .zip(all_pixels.iter())
            .for_each(|(img_pixel, render_pixel)| {
                {
                    let r = render_pixel.x();
                    let g = render_pixel.y();
                    let b = render_pixel.z();

                    // Divide the color by the number of samples and gamma-correct for gamma=2.0.
                    let scale = 1.0 / SAMPLES_PER_PIXEL as f64;
                    let r = (scale * r).sqrt();
                    let g = (scale * g).sqrt();
                    let b = (scale * b).sqrt();

                    let ir = (255.999 * r.clamp(0.0, 0.999)) as u8;
                    let ig = (255.999 * g.clamp(0.0, 0.999)) as u8;
                    let ib = (255.999 * b.clamp(0.0, 0.999)) as u8;

                    *img_pixel = Rgb([ir, ig, ib]);
                }
            });

        image
            .save(&format!("render/image_{:04}.png", frame_no))
            .unwrap();
    }
}
