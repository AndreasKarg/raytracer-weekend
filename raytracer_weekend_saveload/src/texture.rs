use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use raytracer_weekend_lib::texture::{Noise, Texture};
use raytracer_weekend_lib::vec3::Color;
use std::fmt::Debug;
use std::path::PathBuf;
use dyn_clone::{clone_trait_object, DynClone};
use raytracer_weekend_lib::ActiveRng;
use raytracer_weekend_lib::image_texture::ImageTexture;
use raytracer_weekend_lib::light_source::DiffuseLight;
use raytracer_weekend_lib::perlin::Perlin;

#[typetag::serde]
pub trait TextureDescriptor: Sync + Send + Debug + DynClone {
    fn to_texture(&self, rng: &mut ActiveRng) -> Box<dyn Texture>;
}
clone_trait_object!(TextureDescriptor);

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct SolidColorDescriptor {
    color: Color,
}

impl SolidColorDescriptor {
    pub fn new_rgb(red: f32, green: f32, blue: f32) -> Box<Self> {
        Box::new(Self::new(Color::new(red, green, blue)))
    }
}

#[typetag::serde(name = "SolidColor")]
impl TextureDescriptor for SolidColorDescriptor {
    fn to_texture(&self, _: &mut ActiveRng) -> Box<dyn Texture> {
        Box::new(raytracer_weekend_lib::texture::SolidColor::new(self.color))
    }
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct CheckerDescriptor {
    even: Box<dyn TextureDescriptor>,
    odd: Box<dyn TextureDescriptor>,
    frequency: f32,
}

#[typetag::serde(name = "Checker")]
impl TextureDescriptor for CheckerDescriptor {
    fn to_texture(&self, rng: &mut ActiveRng) -> Box<dyn Texture> {
        Box::new(raytracer_weekend_lib::texture::Checker::new(
            self.even.to_texture(rng),
            self.odd.to_texture(rng),
            self.frequency,
        ))
    }
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct ImageTextureDescriptor {
    path: PathBuf,
}

#[typetag::serde(name = "ImageTexture")]
impl TextureDescriptor for ImageTextureDescriptor {
    fn to_texture(&self, _: &mut ActiveRng) -> Box<dyn Texture> {
        Box::new(ImageTexture::open(&self.path.to_str().unwrap()).unwrap())
    }
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct NoiseDescriptor {
    scale: f32,
}

#[typetag::serde(name = "Noise")]
impl TextureDescriptor for NoiseDescriptor {
    fn to_texture(&self, rng: &mut ActiveRng) -> Box<dyn Texture> {
        Box::new(Noise::new(Perlin::new(rng), self.scale))
    }
}
