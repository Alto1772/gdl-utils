use image::imageops::FilterType;
use image::{DynamicImage, imageops};

use crate::combine::url_read::ImageReadError;
use crate::combine::{
    CombineLayout, ImageBuf, ImageCombiner, ImageCombinerInfo, ImagePixel, ImageResizeLimit,
    ImageResizeUnit, LineSizeUnit, StackOrientation, TileFillOrder, TileFitBy, TileGroupInfo,
    TileLastAlignment,
};
use crate::url::ImageUrl;

#[derive(Debug, Clone, Copy, Default)]
struct Size {
    w: f32,
    h: f32,
}

/// A destination rectangle in unscal
#[derive(Debug, Clone, Copy, Default)]
struct Rect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl StackOrientation {
    fn main(self, s: Size) -> f32 {
        match self {
            StackOrientation::Horizontal => s.w,
            StackOrientation::Vertical => s.h,
        }
    }

    fn cross(self, s: Size) -> f32 {
        match self {
            StackOrientation::Horizontal => s.h,
            StackOrientation::Vertical => s.w,
        }
    }

    fn compose(self, main: f32, cross: f32) -> (f32, f32) {
        match self {
            StackOrientation::Horizontal => (main, cross),
            StackOrientation::Vertical => (cross, main),
        }
    }
}

#[derive(Debug)]
pub struct ImageCanvas {
    width: f32,
    height: f32,
    background_color: ImagePixel,
    scale: f32,

    composite: Vec<CanvasCombineLayout>,
    combiner: ImageCombinerInfo,
}

#[derive(Debug)]
pub enum CanvasCombineLayout {
    Single(ImageLayer),
    Tiled(Vec<ImageLayer>, TileGroupInfo),
}

#[derive(Debug)]
pub struct ImageLayer {
    rect: Rect,
    image: DynamicImage,
}

impl ImageLayer {
    fn new(img: &ImageUrl) -> Result<Self, ImageReadError> {
        let image = img.read()?;
        Ok(ImageLayer {
            rect: Rect::default(),
            image,
        })
    }

    fn native_size(&self) -> Size {
        Size {
            w: self.image.width() as f32,
            h: self.image.height() as f32,
        }
    }
}

#[derive(Clone, Copy)]
enum Measured {
    Single(Size),
    Tiled {
        size: Size,
        tile: Size,
        columns: u32,
        gap: f32,
    },
}

impl Measured {
    fn size(&self) -> Size {
        match self {
            Measured::Single(s) => *s,
            Measured::Tiled { size, .. } => *size,
        }
    }
}

impl ImageCanvas {
    pub fn from_combiner(combiner: ImageCombiner) -> Result<Self, ImageReadError> {
        let layers_stack = combiner
            .layouts
            .into_iter()
            .map(|layout| {
                Ok(match layout {
                    CombineLayout::Single(img) => {
                        CanvasCombineLayout::Single(ImageLayer::new(&img)?)
                    }
                    CombineLayout::Tiled(tile, images) => CanvasCombineLayout::Tiled(
                        images
                            .iter()
                            .map(ImageLayer::new)
                            .collect::<Result<Vec<_>, _>>()?,
                        tile,
                    ),
                })
            })
            .collect::<Result<Vec<_>, ImageReadError>>()?;

        Ok(Self::from_layout(layers_stack, combiner.info))
    }

    pub fn from_layout(
        composite: Vec<CanvasCombineLayout>,
        combiner_info: ImageCombinerInfo,
    ) -> Self {
        #[cfg(feature = "hdri")]
        let background_color: ImagePixel = combiner_info.background_color.to_array().into();
        #[cfg(not(feature = "hdri"))]
        let background_color: ImagePixel = combiner_info.background_color.to_rgba8().into();

        let mut canvas = ImageCanvas {
            width: 0.,
            height: 0.,
            scale: 1.0,
            background_color,
            composite,
            combiner: combiner_info,
        };
        canvas.update_positions_and_sizes();
        canvas
    }

    fn measure_all(composite: &[CanvasCombineLayout]) -> Vec<Measured> {
        composite
            .iter()
            .map(|layer| match layer {
                CanvasCombineLayout::Single(img) => Measured::Single(img.native_size()),
                CanvasCombineLayout::Tiled(imgs, data) => Self::measure_tiled(imgs, data),
            })
            .collect()
    }

    fn compute_tile_sizes(sizes: &[(f32, f32)], fit_by: TileFitBy) -> Size {
        match fit_by {
            TileFitBy::Height => {
                let inv_aspect = sizes.iter().map(|&(w, h)| h / w).fold(0., f32::max);
                let w = sizes.iter().map(|&(w, _)| w).fold(0., f32::max);
                Size {
                    w,
                    h: inv_aspect * w,
                }
            }
            TileFitBy::Width => {
                let aspect = sizes.iter().map(|&(w, h)| w / h).fold(0., f32::max);
                let h = sizes.iter().map(|&(_, h)| h).fold(0., f32::max);
                Size { w: aspect * h, h }
            }
            TileFitBy::WidthHeight => Size {
                w: sizes.iter().map(|&(w, _)| w).fold(0., f32::max),
                h: sizes.iter().map(|&(_, h)| h).fold(0., f32::max),
            },
        }
    }

    fn measure_tiled(imgs: &[ImageLayer], data: &TileGroupInfo) -> Measured {
        let sizes: Vec<(f32, f32)> = imgs
            .iter()
            .map(|img| {
                let s = img.native_size();
                (s.w, s.h)
            })
            .collect();
        let tile = Self::compute_tile_sizes(&sizes, data.fit_by);
        let gap = Self::compute_line_size(&data.gap_size, tile.w.min(tile.h));

        let columns = match data.fill_order {
            TileFillOrder::ColumnByColumn => data.line_length,
            TileFillOrder::RowByRow => (imgs.len() as u32).div_ceil(data.line_length),
        };
        let rows = (imgs.len() as u32).div_ceil(columns);

        // saturating_sub: avoids a u32 underflow panic if `columns` or `rows`
        // is ever 0 (e.g. an empty tile group)
        let size = Size {
            w: tile.w * columns as f32 + gap * columns.saturating_sub(1) as f32,
            h: tile.h * rows as f32 + gap * rows.saturating_sub(1) as f32,
        };

        Measured::Tiled {
            size,
            tile,
            columns,
            gap,
        }
    }

    fn block_size(measured: &[Measured], orientation: StackOrientation) -> f32 {
        measured
            .iter()
            .map(|m| orientation.cross(m.size()))
            .fold(0., f32::max)
    }

    fn short_run_spacing(
        align: TileLastAlignment,
        tile: f32,
        gap: f32,
        columns: u32,
        n: u32,
    ) -> (f32, f32) {
        let free = (columns - n) as f32 * (tile + gap);
        let nf = n as f32;
        match align {
            TileLastAlignment::Left => (0., gap),
            TileLastAlignment::Center => (free / 2., gap),
            TileLastAlignment::Right => (free, gap),
            TileLastAlignment::SpaceBetween if n > 1 => (0., gap + free / (nf - 1.)),
            TileLastAlignment::SpaceBetween => (0., gap), // one tile: nothing to space between
            TileLastAlignment::SpaceAround => (free / nf / 2., gap + free / nf),
            TileLastAlignment::SpaceEvenly => (free / (nf + 1.), gap + free / (nf + 1.)),
        }
    }

    fn place_single(
        img: &mut ImageLayer,
        origin: (f32, f32),
        block_size: f32,
        orientation: StackOrientation,
    ) -> f32 {
        let native = img.native_size();
        let factor = block_size / orientation.cross(native);
        let main = orientation.main(native) * factor;
        let (w, h) = orientation.compose(main, block_size);

        img.rect = Rect {
            x: origin.0,
            y: origin.1,
            w,
            h,
        };
        main
    }

    fn place_tiled(
        imgs: &mut [ImageLayer],
        tile_data: &TileGroupInfo,
        measured: &Measured,
        origin: (f32, f32),
        block_size: f32,
        orientation: StackOrientation,
    ) -> f32 {
        let Measured::Tiled {
            size,
            tile,
            columns,
            gap,
        } = *measured
        else {
            unreachable!("place_tiled called with a non-Tiled Measured")
        };

        let factor = block_size / orientation.cross(size);
        let tile_w = tile.w * factor;
        let tile_h = tile.h * factor;
        let gap = gap * factor;

        let (ox, oy) = origin;
        let mut y = oy;

        for row in imgs.chunks_mut(columns as usize) {
            let n = row.len() as u32;
            let (lead, h_gap) = if n == columns {
                (0., gap)
            } else {
                Self::short_run_spacing(tile_data.last_alignment, tile_w, gap, columns, n)
            };

            let mut x = ox + lead;
            for img in row.iter_mut() {
                let native = img.native_size();
                let fit = (tile_w / native.w).min(tile_h / native.h);
                let w = native.w * fit;
                let h = native.h * fit;

                img.rect = Rect {
                    x: x + (tile_w - w) / 2.,
                    y: y + (tile_h - h) / 2.,
                    w,
                    h,
                };

                x += tile_w + h_gap;
            }

            y += tile_h + gap;
        }

        orientation.main(size) * factor
    }

    fn update_positions_and_sizes(&mut self) {
        let orientation = self.combiner.orientation;

        let measured = Self::measure_all(&self.composite);
        let block_size = Self::block_size(&measured, orientation);
        let border = Self::compute_line_size(&self.combiner.border_size, block_size);
        let layer_gap = Self::compute_line_size(&self.combiner.gap_size, block_size);

        let mut along = border; // position along the main (stacking) axis
        let cross = border; // fixed position along the cross axis (top/left border)

        for (layer, m) in self.composite.iter_mut().zip(&measured) {
            let origin = orientation.compose(along, cross);
            let advance = match layer {
                CanvasCombineLayout::Single(img) => {
                    Self::place_single(img, origin, block_size, orientation)
                }
                CanvasCombineLayout::Tiled(imgs, tile_data) => {
                    Self::place_tiled(imgs, tile_data, m, origin, block_size, orientation)
                }
            };
            along += advance + layer_gap;
        }
        along -= layer_gap; // no trailing gap after the last entry
        along += border; // bottom/right border

        let (canvas_w, canvas_h) = orientation.compose(along, block_size + 2. * border);

        self.scale = self.combiner.resize.as_ref().map_or(1.0, |resize| {
            Self::compute_scale(resize, canvas_w, canvas_h)
        });
        self.width = canvas_w;
        self.height = canvas_h;
    }

    fn compute_scale(resize: &ImageResizeUnit, base_w: f32, base_h: f32) -> f32 {
        let (scale, limit) = match resize {
            ImageResizeUnit::Percentage { p } => return p / 100.,

            ImageResizeUnit::Height { h, limit } => (*h as f32 / base_h, limit),
            ImageResizeUnit::Width { w, limit } => (*w as f32 / base_w, limit),
            ImageResizeUnit::WidthHeightFit { w, h, limit } => {
                ((*w as f32 / base_w).min(*h as f32 / base_h), limit)
            }
        };

        match limit {
            ImageResizeLimit::None => scale,
            ImageResizeLimit::Enlarge => scale.max(1.0),
            ImageResizeLimit::Shrink => scale.min(1.0),
        }
    }

    fn compute_line_size(lsize: &LineSizeUnit, base: f32) -> f32 {
        match lsize {
            LineSizeUnit::Exact(size) => *size as f32,
            LineSizeUnit::Percentage(percent) => base * (percent / 100.0),
        }
    }

    fn round_rect(r: Rect, scale: f32) -> (i64, i64, u32, u32) {
        let x0 = (r.x * scale).round();
        let y0 = (r.y * scale).round();
        let x1 = ((r.x + r.w) * scale).round();
        let y1 = ((r.y + r.h) * scale).round();
        (
            x0 as i64,
            y0 as i64,
            (x1 - x0).max(0.) as u32,
            (y1 - y0).max(0.) as u32,
        )
    }

    fn overlay_img(imgbuf: &mut ImageBuf, layer: &ImageLayer, scale: f32, filter: FilterType) {
        let (x, y, new_w, new_h) = Self::round_rect(layer.rect, scale);
        let native_w = layer.image.width();
        let native_h = layer.image.height();

        if native_w != new_w || native_h != new_h {
            let resized = layer.image.resize_exact(new_w, new_h, filter);
            imageops::overlay(imgbuf, &resized, x, y);
        } else {
            imageops::overlay(imgbuf, &layer.image, x, y);
        }
    }

    pub fn build(&self) -> ImageBuf {
        let canvas_w = (self.width * self.scale).round() as u32;
        let canvas_h = (self.height * self.scale).round() as u32;
        let mut imgbuf = ImageBuf::from_pixel(canvas_w, canvas_h, self.background_color);

        for layout in &self.composite {
            match layout {
                CanvasCombineLayout::Single(img) => {
                    Self::overlay_img(&mut imgbuf, img, self.scale, self.combiner.filter_type);
                }
                CanvasCombineLayout::Tiled(imgs, _tile) => {
                    for img in imgs {
                        Self::overlay_img(&mut imgbuf, img, self.scale, self.combiner.filter_type);
                    }
                }
            }
        }
        imgbuf
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width.round() as u32, self.height.round() as u32)
    }

    pub fn scale(&self) -> f32 {
        self.scale
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn compute_scale_respects_its_enlarge_and_shrink_limits(
            target in 1u32..100_000,
            base_w in 1.0f32..100_000.0,
            base_h in 1.0f32..100_000.0,
        ) {
            let raw_ratio = target as f32 / base_w;

            let none = ImageCanvas::compute_scale(
                &ImageResizeUnit::Width { w: target, limit: ImageResizeLimit::None },
                base_w,
                base_h,
            );
            let enlarge = ImageCanvas::compute_scale(
                &ImageResizeUnit::Width { w: target, limit: ImageResizeLimit::Enlarge },
                base_w,
                base_h,
            );
            let shrink = ImageCanvas::compute_scale(
                &ImageResizeUnit::Width { w: target, limit: ImageResizeLimit::Shrink },
                base_w,
                base_h,
            );

            prop_assert!((none - raw_ratio).abs() < 1e-3);
            prop_assert!(enlarge >= 1.0, "Enlarge limit produced a shrinking scale {enlarge}");
            prop_assert!(shrink <= 1.0, "Shrink limit produced an enlarging scale {shrink}");
        }
    }
}
