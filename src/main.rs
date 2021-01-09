mod scenes;

use clap::Clap;
use image::{Rgb, RgbImage};
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rand::thread_rng;
use rayon::prelude::*;
use raytracer_weekend_lib::Raytracer;
use scenes::Scene;

const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");
const CRATE_AUTHOR: &str = env!("CARGO_PKG_AUTHORS");

/// My raytracer, based on the book series on the interwebs.
#[derive(Clap)]
#[clap(version = CRATE_VERSION, author = CRATE_AUTHOR)]
struct Opts {
    #[clap(subcommand)]
    scene: Scene,
    #[clap(default_value = "400")]
    width: usize,
    #[clap(default_value = "1.7777778")]
    aspect_ratio: f64,
    #[clap(default_value = "100")]
    samples_per_pixel: usize,
}

fn main() {
    let opts: Opts = Opts::parse();

    let image_width = opts.width;
    let aspect_ratio = opts.aspect_ratio;
    let image_height = (image_width as f64 / aspect_ratio).round() as usize;
    let samples_per_pixel = opts.samples_per_pixel;

    let pixel_count = (image_width * image_height) as u64;

    let (world, cams, background) = opts.scene.generate(
        (image_width as f64) / (image_height as f64),
        &mut thread_rng(),
    );

    for (frame_no, cam) in cams.iter().enumerate() {
        let raytracer = Raytracer::new(
            &world,
            &cam,
            background,
            image_width,
            image_height,
            samples_per_pixel,
        );

        let progress_bar = ProgressBar::new(pixel_count);
        progress_bar.set_style(ProgressStyle::default_bar().template(
            "[[{msg}] {elapsed_precise} / {eta_precise}] {wide_bar} {pos:>7}/{len:7} ({per_sec})",
        ));
        progress_bar.set_draw_delta(pixel_count / 100);
        progress_bar.set_message(&format!("[{} / {}]", frame_no, cams.len()));
        let all_pixels: Vec<_> = raytracer.render().progress_with(progress_bar).collect();

        let mut image = RgbImage::new(image_width as u32, image_height as u32);

        image
            .pixels_mut()
            .zip(all_pixels.iter())
            .for_each(|(img_pixel, render_pixel)| {
                {
                    let r = render_pixel.x();
                    let g = render_pixel.y();
                    let b = render_pixel.z();

                    // Divide the color by the number of samples and gamma-correct for gamma=2.0.
                    let scale = 1.0 / samples_per_pixel as f64;
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
