mod scenes;

use std::iter::once;
use crate::scenes::{SceneGenerator, GENERATORS};
use clap::{Args, Parser, Subcommand};
use image::{Rgb, RgbImage};
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressIterator, ProgressStyle};
use rayon::prelude::*;
use raytracer_weekend_lib::Raytracer;
use raytracer_weekend_saveload::World;
use raytracer_weekend_saveload::hittable::HittableDescriptorList;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use clap::builder::{PossibleValue, ValueParser};
use phf::set::Iter;

const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");
const CRATE_AUTHOR: &str = env!("CARGO_PKG_AUTHORS");

pub static GENERATOR_NAMES: LazyLock<Vec<PossibleValue>> =
    LazyLock::new(|| {
        let all_option = [PossibleValue::new("all").help("Run all available scenes")];
        let generator_names = GENERATORS.keys().map(|k| PossibleValue::new(*k));

        let possible_values = all_option.into_iter().chain(generator_names);

        possible_values.collect()
    });

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
        #[arg(value_parser = GENERATOR_NAMES.clone())]
        scene: String,
        #[command(flatten)]
        render_args: RenderArgs,
    },
    RenderFile {
        #[command(flatten)]
        render_args: RenderArgs,
        scene_description: PathBuf,
    },
    ToJson {
        #[arg(value_parser = GENERATOR_NAMES.clone())]
        scene: String,
        output: PathBuf,
    },
    #[clap(alias = "to-yml")]
    ToYaml {
        #[arg(value_parser = GENERATOR_NAMES.clone())]
        scene: String,
        output: PathBuf,
    },
}

#[derive(Debug, Copy, Clone, Args)]
struct RenderArgs {
    #[clap(long, short, default_value = "400")]
    width: u32,
    #[clap(long, short = 'r', default_value = "1.7777778")]
    aspect_ratio: f64,
    #[clap(long, short, default_value = "100")]
    samples_per_pixel: u32,
}

fn run_render(world: World, args: RenderArgs, base_path: &Path) {
    let image_width = args.width;
    let aspect_ratio = args.aspect_ratio;
    let image_height = (image_width as f64 / aspect_ratio).round() as u32;
    let samples_per_pixel = args.samples_per_pixel;

    let pixel_count = (image_width * image_height) as u64;

    let out_dir = base_path.join("render/");
    std::fs::create_dir_all(&out_dir).unwrap();
    let filename_base = world.title.unwrap_or("image".to_string());

    let cameras = world.cameras;
    let overall_progress_style = ProgressStyle::default_bar()
        .template("[{elapsed_precise} / {eta_precise}] {wide_bar} {pos:>7}/{len:7} ({per_sec}")
        .unwrap();
    let overall_progress =
        ProgressBar::new(cameras.len() as u64).with_style(overall_progress_style);

    let geometry = world.geometry.to_hittables(&mut rand::rng(), base_path);

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

        let frame_progress_style = ProgressStyle::default_bar()
            .template("[{elapsed_precise} / {eta_precise}] {wide_bar} {pos:>7}/{len:7} ({per_sec})")
            .unwrap();
        let frame_progress = ProgressBar::new(pixel_count).with_style(frame_progress_style);

        let all_pixels: Vec<_> = raytracer
            .render(&mut rand::rng())
            .progress_with(frame_progress)
            .collect();

        let mut image = RgbImage::new(image_width, image_height);

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

        let frame_out_path = out_dir.join(format!("{}_{:04}.png", filename_base, frame_no));
        image.save(&frame_out_path).unwrap();
    }
}

fn main() {
    let args: MainArgs = MainArgs::parse();
    match args.command {
        Command::RenderCompiled {
            render_args, scene} => {
            let image_width = render_args.width;
            let aspect_ratio = render_args.aspect_ratio;
            let image_height = (image_width as f64 / aspect_ratio).round() as u32;

            let scenes = get_render_schedule(&scene);

            for scene_generator in scenes {
                let world = scene_generator(
                    (render_args.width as f32) / (image_height as f32),
                    &mut rand::rng(),
                );

                let base_path = std::env::current_dir().unwrap();

                run_render(world, render_args, &base_path)
            }
        }
        Command::RenderFile {
            render_args,
            scene_description,
        } => {
            let base_path = scene_description.parent().unwrap().to_path_buf();
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

            run_render(world, render_args, &base_path)
        }
        Command::ToJson { scene, output } => {
            let scene_generator = GENERATORS.get(&scene).unwrap();

            let world = scene_generator(16.0 / 9.0, &mut rand::rng());
            let json = serde_json::to_string_pretty(&world).unwrap();
            std::fs::write(output, json).unwrap();
        }
        Command::ToYaml { scene, output } => {
            let scene_generator = GENERATORS.get(&scene).unwrap();

            let world = scene_generator(16.0 / 9.0, &mut rand::rng());
            let yml = serde_yaml::to_string(&world).unwrap();
            std::fs::write(output, yml).unwrap();
        }
    }
}

fn get_render_schedule(scene_arg: &str) -> Box<dyn Iterator<Item = &'static SceneGenerator>> {
    match scene_arg {
        "all" => {
            Box::new(GENERATORS.values())
        },
        other => Box::new(once(GENERATORS.get(other).unwrap()))
    }
}
