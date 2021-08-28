#![cfg(feature = "image")]

use alloc::string::String;
use core::{
    any::type_name,
    fmt::{Debug, Formatter},
};

use image::{io::Reader as ImageReader, DynamicImage, GenericImageView};

use crate::{
    texture::{Point2d, Texture},
    vec3::{Color, Vec3},
};

#[derive(Clone)]
pub struct ImageTexture {
    image: DynamicImage,
    path: String,
}

impl ImageTexture {
    pub fn open(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let image = ImageReader::open(path)?.decode()?;

        Ok(Self {
            image,
            path: path.to_string(),
        })
    }
}

impl Texture for ImageTexture {
    fn value(&self, uv: Point2d, _p: &Vec3) -> Color {
        let image = &self.image;

        let u = uv.u.clamp(0.0, 1.0);
        let v = 1.0 - uv.v.clamp(0.0, 1.0);

        let i = ((u * image.width() as f32) as u32).clamp(0, image.width() - 1);
        let j = ((v * image.height() as f32) as u32).clamp(0, image.height() - 1);

        let color_scale = 1.0 / 255.0;
        let pixel = image.get_pixel(i, j);

        Color::new(
            pixel[0] as f32 * color_scale,
            pixel[1] as f32 * color_scale,
            pixel[2] as f32 * color_scale,
        )
    }
}

impl Debug for ImageTexture {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct(type_name::<Self>())
            .field("image", &self.path)
            .finish()
    }
}
