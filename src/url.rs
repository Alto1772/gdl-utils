#![allow(unused)]
// TODO do something here...

#[derive(Debug, Clone, Copy)]
pub enum ImageUrlType {
    Local,
    Online,
}

#[derive(Debug, Clone)]
pub struct ImageUrl {
    // url: String,
    source: String,
    url_type: ImageUrlType,
    deleted: bool,
}

impl ImageUrl {
    pub fn new(source: &str) -> Self {
        Self {
            // url: "[WIP]".to_string(),
            source: source.to_string(),
            url_type: ImageUrlType::Local,
            deleted: false,
        }
    }

    pub fn url(&self) -> &str {
        &self.source
    }

    pub fn delete(&mut self) -> Result<(), (&str, std::io::Error)> {
        if !self.deleted {
            if let Err(e) = std::fs::remove_file(&self.source) {
                return Err((&self.source, e));
            }
            self.deleted = true;
        }
        Ok(())
    }
}

pub fn parse_image_url(url: &str) -> ImageUrl {
    ImageUrl::new(url)
}
