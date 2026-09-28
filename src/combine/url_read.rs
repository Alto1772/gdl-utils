use crate::url::ImageUrl;

use image::{DynamicImage, ImageReader};

#[derive(thiserror::Error, Debug)]
#[error("`{img}` read error")]
pub struct ImageReadError {
    img: String,

    #[source]
    kind: ImageReadErrorKind,
}

#[derive(thiserror::Error, Debug)]
enum ImageReadErrorKind {
    #[error("failed to open image")]
    Open(#[from] std::io::Error),
    #[error("failed to decode image")]
    Decode(#[from] image::error::ImageError),
}

impl ImageReadError {
    fn new(img: &str, err: impl Into<ImageReadErrorKind>) -> Self {
        Self {
            img: img.to_string(),
            kind: err.into(),
        }
    }
}

impl ImageUrl {
    pub fn read(&self) -> Result<DynamicImage, ImageReadError> {
        ImageReader::open(self.url())
            .map_err(|e| ImageReadError::new(self.url(), e))?
            .with_guessed_format()
            .map_err(|e| ImageReadError::new(self.url(), e))?
            .decode()
            .map_err(|e| ImageReadError::new(self.url(), e))
    }
}
