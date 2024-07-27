mod scenes;

use std::path::PathBuf;
use clap::{Args, Parser, Subcommand};
use image::{Rgb, RgbImage};
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressIterator, ProgressStyle};
use rand::thread_rng;
use rayon::prelude::*;
use raytracer_weekend_lib::Raytracer;
use raytracer_weekend_saveload::hittable::HittableDescriptorList;
use raytracer_weekend_saveload::World;
use scenes::Scene;

const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");
const CRATE_AUTHOR: &str = env!("CARGO_PKG_AUTHORS");

/// My raytracer, based on the book series on the interwebs.
#[derive(Parser)]
#[clap(version = CRATE_VERSION, author = CRATE_AUTHOR)]
struct MainArgs {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    RenderCompiled {
        #[command(subcommand)]
        scene: Scene,
        #[command(flatten)]
        render_args: RenderArgs,
    },
    RenderFile {
        #[command(flatten)]
        render_args: RenderArgs,
        scene_description: PathBuf,
    },
    ToJson {
        #[command(subcommand)]
        scene: Scene,
        output: PathBuf,
    },
    ToYml {
        #[command(subcommand)]
        scene: Scene,
        output: PathBuf,
    },
}

#[derive(Debug, Clone, Args)]
struct RenderArgs {
    #[clap(long, short, default_value = "400")]
    width: u32,
    #[clap(long, short, default_value = "1.7777778")]
    aspect_ratio: f64,
    #[clap(long, short, default_value = "100")]
    samples_per_pixel: u32,
}

fn run_render(world: World, args: RenderArgs) {
    let image_width = args.width;
    let aspect_ratio = args.aspect_ratio;
    let image_height = (image_width as f64 / aspect_ratio).round() as u32;
    let samples_per_pixel = args.samples_per_pixel;

    let pixel_count = (image_width * image_height) as u64;

    let cameras = world.cameras;
    let overall_progress = ProgressBar::new(cameras.len() as u64)
        .with_style(ProgressStyle::default_bar().template(
            "[{elapsed_precise} / {eta_precise}] {wide_bar} {pos:>7}/{len:7} ({per_sec}",
        ));

    let geometry = world.geometry.to_hittables();

    for (frame_no, cam) in cameras.iter().progress_with(overall_progress).enumerate() {
        let cam = cam.to_camera();
        let raytracer = Raytracer::new(
            &geometry,
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

fn main() {
    let args: MainArgs = MainArgs::parse();
    match args.command {
        Command::RenderCompiled { render_args, scene } => {
            let image_width = render_args.width;
            let aspect_ratio = render_args.aspect_ratio;
            let image_height = (image_width as f64 / aspect_ratio).round() as u32;

            let world = scene.generate(
                (render_args.width as f32) / (image_height as f32),
                &mut thread_rng(),
            );
            run_render(world, render_args)
        }
        Command::RenderFile { render_args, scene_description } => {
            let world = match scene_description.extension().unwrap().to_str().unwrap() {
                "json" => {
                    let json = std::fs::read_to_string(scene_description).unwrap();
                    serde_json::from_str(&json).unwrap()
                }
                "yml" | "yaml" => {
                    let yml = std::fs::read_to_string(scene_description).unwrap();
                    serde_yaml::from_str(&yml).unwrap()
                }
                _ => panic!("Unknown file type"),
            };
            run_render(world, render_args)
        }
        Command::ToJson {
            scene,
            output
        } => {
            let world = scene.generate(16.0 / 9.0, &mut thread_rng());
            let json = serde_json::to_string_pretty(&world).unwrap();
            std::fs::write(output, json).unwrap();
        }
        Command::ToYml {
            scene,
            output
        } => {
            let world = scene.generate(16.0 / 9.0, &mut thread_rng());
            let yml = serde_yaml::to_string(&world).unwrap();
            std::fs::write(output, yml).unwrap();
        }
    }
}
