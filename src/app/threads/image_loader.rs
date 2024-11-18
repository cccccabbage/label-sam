use std::path::PathBuf;

use image::DynamicImage;

#[derive(Clone)]
pub struct Image {
    pub data: DynamicImage,
    pub path: PathBuf,
    pub size: [f32; 2],
    pub file_size: f32,
}

impl Image {
    pub fn load(path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let data = image::ImageReader::open(&path)?.decode()?;
        let size = [data.width() as f32, data.height() as f32];
        let file_size = std::fs::metadata(&path)?.len() as f32;

        Ok(Image {
            data,
            path,
            size,
            file_size,
        })
    }
}
