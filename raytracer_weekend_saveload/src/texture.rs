use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use raytracer_weekend_lib::texture::Texture;
use raytracer_weekend_lib::vec3::Color;
use std::fmt::Debug;
use std::path::PathBuf;
use dyn_clone::{clone_trait_object, DynClone};
use raytracer_weekend_lib::image_texture::ImageTexture;
use raytracer_weekend_lib::light_source::DiffuseLight;

#[typetag::serde]
pub trait TextureDescriptor: Sync + Send + Debug + DynClone {
    fn to_texture(&self) -> Box<dyn Texture>;
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
    fn to_texture(&self) -> Box<dyn Texture> {
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
    fn to_texture(&self) -> Box<dyn Texture> {
        Box::new(raytracer_weekend_lib::texture::Checker::new(
            self.even.to_texture(),
            self.odd.to_texture(),
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
    fn to_texture(&self) -> Box<dyn Texture> {
        Box::new(ImageTexture::open(&self.path.to_str().unwrap()).unwrap())
    }
}
