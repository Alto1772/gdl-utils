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

#[derive(Debug, Eq, PartialEq, Default)]
pub enum DeleteStatus {
    AlreadyGone,
    #[default]
    Deleted,
    Remote,
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

    pub fn delete(&mut self) -> Result<DeleteStatus, (&str, std::io::Error)> {
        if !self.deleted {
            let source: &str = &self.source;
            if let Err(e) = std::fs::remove_file(source) {
                if e.kind() == std::io::ErrorKind::NotFound {
                    self.deleted = true;
                    return Ok(DeleteStatus::AlreadyGone);
                } else {
                    return Err((source, e));
                }
            }

            self.deleted = true;
            Ok(DeleteStatus::Deleted)
        } else {
            Ok(DeleteStatus::AlreadyGone)
        }
    }
}

pub fn parse_image_url(url: &str) -> ImageUrl {
    ImageUrl::new(url)
}
