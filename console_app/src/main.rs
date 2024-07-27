mod scenes;

use clap::Parser;
use image::{Rgb, RgbImage};
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressIterator, ProgressStyle};
use rand::thread_rng;
use rayon::prelude::*;
use raytracer_weekend_lib::Raytracer;
use scenes::Scene;

const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");
const CRATE_AUTHOR: &str = env!("CARGO_PKG_AUTHORS");

/// My raytracer, based on the book series on the interwebs.
#[derive(Parser)]
#[clap(version = CRATE_VERSION, author = CRATE_AUTHOR)]
struct Opts {
    #[clap(subcommand)]
    scene: Scene,
    #[clap(long, short, default_value = "400")]
    width: u32,
    #[clap(long, short, default_value = "1.7777778")]
    aspect_ratio: f64,
    #[clap(long, short, default_value = "100")]
    samples_per_pixel: u32,
}

fn main() {
    let opts: Opts = Opts::parse();

    let image_width = opts.width;
    let aspect_ratio = opts.aspect_ratio;
    let image_height = (image_width as f64 / aspect_ratio).round() as u32;
    let samples_per_pixel = opts.samples_per_pixel;

    let pixel_count = (image_width * image_height) as u64;

    let world = opts.scene.generate(
        (image_width as f32) / (image_height as f32),
        &mut thread_rng(),
    );

    let cameras = world.cameras;
    let overall_progress = ProgressBar::new(cameras.len() as u64)
        .with_style(ProgressStyle::default_bar().template(
            "[{elapsed_precise} / {eta_precise}] {wide_bar} {pos:>7}/{len:7} ({per_sec}",
        ));

    for (frame_no, cam) in cameras.iter().progress_with(overall_progress).enumerate() {
        let raytracer = Raytracer::new(
            &world.geometry,
            &cam,
            world.background,
            image_width,
            image_height,
            samples_per_pixel,
        );

        let frame_progress =
            ProgressBar::new(pixel_count).with_style(ProgressStyle::default_bar().template(
                "[{elapsed_precise} / {eta_precise}] {wide_bar} {pos:>7}/{len:7} ({per_sec})",
            ));
        frame_progress.set_draw_delta(pixel_count / 100);

        let all_pixels: Vec<_> = raytracer.render().progress_with(frame_progress).collect();

        let mut image = RgbImage::new(image_width as u32, image_height as u32);

        image
            .pixels_mut()
            .zip(all_pixels.iter())
            .for_each(|(img_pixel, render_pixel)| {
                {
                    let color = render_pixel.color;
                    let r = color.x();
                    let g = color.y();
                    let b = color.z();

                    // Divide the color by the number of samples and gamma-correct for gamma=2.0.
                    let scale = 1.0 / samples_per_pixel as f32;
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
