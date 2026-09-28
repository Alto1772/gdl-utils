pub mod cli;
pub mod composite;
mod url_read;

use crate::url::ImageUrl;
use image::{GenericImageView, imageops::FilterType};
use strum::{Display, EnumString, VariantNames};

#[cfg(feature = "hdri")]
pub type ImageBuf = image::Rgba32FImage;
#[cfg(not(feature = "hdri"))]
pub type ImageBuf = image::RgbaImage;
pub type ImagePixel = <ImageBuf as GenericImageView>::Pixel;

#[derive(Clone, Debug)]
pub struct ImageCombiner {
    info: ImageCombinerInfo,
    layouts: Vec<CombineLayout>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImageCombinerInfo {
    orientation: StackOrientation,
    border_size: LineSizeUnit,
    gap_size: LineSizeUnit,
    resize: Option<ImageResizeUnit>,
    background_color: csscolorparser::Color,
    filter_type: FilterType,
    #[allow(unused)]
    delete_sources: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LineSizeUnit {
    Exact(u32),
    Percentage(f32),
}

impl Default for LineSizeUnit {
    fn default() -> Self {
        Self::Exact(0)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ImageResizeLimit {
    Enlarge,
    Shrink,
    #[default]
    None,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ImageResizeUnit {
    Width {
        w: u32,
        limit: ImageResizeLimit,
    },
    Height {
        h: u32,
        limit: ImageResizeLimit,
    },
    WidthHeightFit {
        w: u32,
        h: u32,
        limit: ImageResizeLimit,
    },
    Percentage {
        p: f32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumString, Display, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum StackOrientation {
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug)]
pub enum CombineLayout {
    Single(ImageUrl),
    Tiled(TileGroupInfo, Vec<ImageUrl>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumString, Display, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum TileFillOrder {
    RowByRow,
    ColumnByColumn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumString, Display, Default, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum TileLastAlignment {
    #[default]
    Left,
    Right,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TileGroupInfo {
    line_length: u32,
    gap_size: LineSizeUnit,
    fill_order: TileFillOrder,
    last_alignment: TileLastAlignment,
    fit_by: TileFitBy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumString, Display, Default, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum TileFitBy {
    Width,
    Height,
    #[default]
    WidthHeight,
}

impl ImageCombiner {
    pub fn delete_sources(&mut self) {
        for layer in &mut self.layouts {
            match layer {
                CombineLayout::Single(img) => img.delete(),
                CombineLayout::Tiled(_tile, images) => {
                    images.iter_mut().for_each(ImageUrl::delete);
                }
            }
        }
    }

    pub fn resolve_urls(&mut self) {
        todo!()
    }
}
