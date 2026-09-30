use std::error::Error;
use std::path::PathBuf;

use crate::combine::{
    CombineLayout, ImageCombiner, ImageCombinerInfo, ImageResizeLimit, ImageResizeUnit,
    LineSizeUnit, StackOrientation, TileFillOrder, TileFitBy, TileGroupInfo, TileLastAlignment,
};
use crate::url::{ImageUrl, parse_image_url};
use clap::Args;
use strum::VariantNames;

const IMAGES_HELP: &str = "\
Files or URLs to be combined, in placement order.
TODO: Add helpful message here
";

const TILESPEC_FMT: &str = "+<rows|cols>:<COUNT>[+GAP][,align=ALIGN][,fit=FIT]";

#[derive(Args, Debug)]
pub struct CombineArgs {
    /// Adds border size (padding) around the resultant image
    #[arg(long, value_name = "SIZE", value_parser = parse_line_size)]
    border_size: Option<LineSizeUnit>,

    /// Adds gap size between rows or columns.
    #[arg(long, value_name = "SIZE", value_parser = parse_line_size)]
    gap: Option<LineSizeUnit>,

    /// Resizes resultant image
    #[arg(long, value_name = "WxH|W|xH|S%", value_parser = parse_resize)]
    resize: Option<ImageResizeUnit>,

    /// Adds background color around the image
    #[arg(short, long, value_name = "COLOR", default_value = "transparent")]
    background: csscolorparser::Color,

    /// Deletes original images after successful combination
    #[arg(long)]
    delete: bool,

    /// Direction in which images are stacked.
    #[arg(
        short = 'O',
        long = "stack-orientation",
        value_name = "DIRECTION",
        default_value = "vertical"
    )]
    orientation: StackOrientation,

    #[arg(short, long, required = true)]
    output: String,

    /// Files or URLs to be combined, in placement order.
    #[arg(required = true, value_name = "+TILED | URL", long_help = IMAGES_HELP, value_parser = parse_input)]
    images: Vec<ParsedInput>,
}

#[allow(unused)]
#[derive(Clone, Debug)]
enum ParsedInput {
    TileSpec(TileGroupInfo),
    SingleSeparator,
    LocalPath(PathBuf),
    Url(String),
    Identifier {
        category: Option<String>,
        subcategory: Option<String>,
        id: String,
    },
}

fn parse_identifier(s: &str) -> Option<(Option<String>, Option<String>, String)> {
    let splits = s.splitn(4, ':').collect::<Vec<_>>();

    match splits.as_slice() {
        ["", id] => Some((None, None, (*id).into())),
        [cat, id] => Some((Some((*cat).into()), None, (*id).into())),
        [cat, subcat, id] => Some((Some((*cat).into()), Some((*subcat).into()), (*id).into())),
        _ => None,
    }
}

fn parse_input(s: &str) -> Result<ParsedInput, ParseLayoutErrorKind> {
    if let Some(spec) = s.strip_prefix('+') {
        if spec.is_empty() {
            Ok(ParsedInput::SingleSeparator)
        } else {
            Ok(ParsedInput::TileSpec(parse_tile_spec(spec)?))
        }
    } else if s.starts_with("http://") || s.starts_with("https://") {
        Ok(ParsedInput::Url(s.into()))
    } else if let Some((category, subcategory, id)) = parse_identifier(s) {
        Ok(ParsedInput::Identifier {
            category,
            subcategory,
            id,
        })
    } else {
        Ok(ParsedInput::LocalPath(s.into()))
    }
}

fn parse_line_size(s: &str) -> Result<LineSizeUnit, Box<dyn Error + Send + Sync + 'static>> {
    if let Some(percent) = s.strip_suffix('%') {
        Ok(LineSizeUnit::Percentage(percent.parse()?))
    } else {
        Ok(LineSizeUnit::Exact(s.parse()?))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ParseResizeError {
    #[error("Invalid width number")]
    InvalidWidth,
    #[error("Invalid height number")]
    InvalidHeight,
    #[error("Invalid percent number")]
    InvalidPercentage,
    #[error("Invalid value")]
    ExpectedValue,
}

fn parse_resize(s: &str) -> Result<ImageResizeUnit, ParseResizeError> {
    if let Some(percent) = s.strip_suffix('%') {
        Ok(ImageResizeUnit::Percentage {
            p: percent
                .parse()
                .map_err(|_| ParseResizeError::InvalidPercentage)?,
        })
    } else {
        let (wh_s, limit) = if let Some(wh_s) = s.strip_suffix('>') {
            (wh_s, ImageResizeLimit::Shrink)
        } else if let Some(wh_s) = s.strip_suffix('<') {
            (wh_s, ImageResizeLimit::Enlarge)
        } else {
            (s, ImageResizeLimit::None)
        };

        let mut tok = wh_s.splitn(2, 'x');
        let width_s = tok.next();
        let height_s = tok.next();

        match (width_s, height_s) {
            (Some(""), Some(height_s)) => Ok(ImageResizeUnit::Height {
                h: height_s
                    .parse()
                    .map_err(|_| ParseResizeError::InvalidHeight)?,
                limit,
            }),
            (Some(width_s), Some(height_s)) => Ok(ImageResizeUnit::WidthHeightFit {
                w: width_s
                    .parse()
                    .map_err(|_| ParseResizeError::InvalidWidth)?,
                h: height_s
                    .parse()
                    .map_err(|_| ParseResizeError::InvalidHeight)?,
                limit,
            }),
            (Some(width_s), None) => Ok(ImageResizeUnit::Width {
                w: width_s
                    .parse()
                    .map_err(|_| ParseResizeError::InvalidWidth)?,
                limit,
            }),
            _ => Err(ParseResizeError::ExpectedValue),
        }
    }
}

fn parse_tile_fill_order(s: &str) -> Option<TileFillOrder> {
    match s {
        "rows" | "r" => Some(TileFillOrder::RowByRow),
        "cols" | "c" | "columns" => Some(TileFillOrder::ColumnByColumn),
        _ => None,
    }
}

fn parse_tile_fit_by(s: &str) -> Option<TileFitBy> {
    match s {
        "w" => Some(TileFitBy::Width),
        "h" => Some(TileFitBy::Height),
        "wh" => Some(TileFitBy::WidthHeight),
        _ => None,
    }
}

fn available_variants(variants: &[&str]) -> String {
    variants.join(", ")
}

#[derive(thiserror::Error, Debug)]
#[error("failed to parse layout spec at arg {pos}: {kind}", pos = position + 1)]
pub struct ParseLayoutError {
    spec: TileGroupInfo,
    position: usize,

    #[source]
    kind: ParseLayoutErrorKind,
}

impl ParseLayoutError {
    fn new(spec: &TileGroupInfo, position: usize, kind: ParseLayoutErrorKind) -> Self {
        Self {
            spec: spec.clone(),
            position,
            kind,
        }
    }
}

#[derive(thiserror::Error, Debug)]
enum ParseLayoutErrorKind {
    #[error("Too many parameters (expected {ts})", ts = TILESPEC_FMT)]
    TooManyParams,

    #[error("Not enough parameters (expected {ts})", ts = TILESPEC_FMT)]
    NotEnoughParams,

    #[error("Expected `=` for optional parameters")]
    InvalidOptParam,

    #[error("Invalid parameter `{0}`")]
    InvalidOptParamName(String),

    #[error("invalid fill order `{0}` (expected `row` or `col`)")]
    InvalidFillOrder(String),

    #[error("invalid line length `{0}` (expected a positive integer)")]
    InvalidLineLength(String),

    #[error("invalid gap size `{0}` (expected a positive integer or percentage)")]
    InvalidGap(String),

    #[error("invalid alignment `{0}` (expected {variants})", variants = available_variants(TileLastAlignment::VARIANTS))]
    InvalidAlignment(String),

    #[error("invalid tile fit by `{0}` (expected `w`, `h`, `wh`)")]
    InvalidTileFitBy(String),

    #[error("no images assigned for tile specifier")]
    NoImages,
}

use ParseLayoutErrorKind::*;

fn parse_tile_spec(spec: &str) -> Result<TileGroupInfo, ParseLayoutErrorKind> {
    let (fill_order_s, next1) = spec.split_once(':').ok_or(NotEnoughParams)?;
    let fill_order = parse_tile_fill_order(fill_order_s)
        .ok_or_else(|| InvalidFillOrder(fill_order_s.to_string()))?;

    let (length_gap, opt_args) = next1
        .split_once(',')
        .map_or((next1, None), |(lg, oa)| (lg, Some(oa)));

    if length_gap.is_empty() {
        return Err(NotEnoughParams);
    }
    let (line_length_s, gap_size_s) = length_gap.split_once('+').unwrap_or((length_gap, ""));

    if line_length_s.is_empty() {
        return Err(NotEnoughParams);
    }
    let line_length: u32 = line_length_s
        .parse()
        .map_err(|_| InvalidLineLength(line_length_s.to_string()))?;
    if line_length == 0 {
        return Err(InvalidLineLength(line_length_s.to_string()));
    }

    let gap_size = if gap_size_s.is_empty() {
        LineSizeUnit::default()
    } else {
        parse_line_size(gap_size_s).map_err(|_| InvalidGap(gap_size_s.to_string()))?
    };

    let mut last_alignment: Option<TileLastAlignment> = None;
    let mut fit_by: Option<TileFitBy> = None;

    if let Some(opt_args) = opt_args {
        for (i, optarg) in opt_args.split(',').enumerate() {
            if i >= 2 {
                return Err(TooManyParams);
            }
            if let Some((param, value)) = optarg.split_once('=') {
                match param {
                    "" => return Err(InvalidOptParam),
                    "align" => {
                        if last_alignment.is_some() {
                            return Err(TooManyParams);
                        }
                        last_alignment = Some(
                            value
                                .parse()
                                .map_err(|_| InvalidAlignment(value.to_string()))?,
                        );
                    }
                    "fit" => {
                        if fit_by.is_some() {
                            return Err(TooManyParams);
                        }
                        fit_by = Some(
                            parse_tile_fit_by(value)
                                .ok_or_else(|| InvalidTileFitBy(value.to_string()))?,
                        );
                    }
                    _ => return Err(InvalidOptParamName(param.to_string())),
                }
            } else {
                return Err(InvalidOptParam);
            }
        }
    }

    Ok(TileGroupInfo {
        fill_order,
        line_length,
        gap_size,
        last_alignment: last_alignment.unwrap_or_default(),
        fit_by: fit_by.unwrap_or_default(),
    })
}

impl CombineArgs {
    pub fn get_output(&self) -> &str {
        &self.output
    }

    pub fn to_combiner(&self) -> Result<ImageCombiner, ParseLayoutError> {
        let border_size = self.border_size.clone().unwrap_or_default();
        let gap_size = self.gap.clone().unwrap_or_default();
        let resize = self.resize.clone();

        let layouts = Self::parse_layout(&self.images)?;

        Ok(ImageCombiner {
            info: ImageCombinerInfo {
                border_size,
                gap_size,
                resize,
                background_color: self.background.clone(),
                orientation: self.orientation,
                delete_sources: self.delete,
                filter_type: image::imageops::FilterType::Gaussian, // TODO replace default
            },
            layouts,
        })
    }

    fn parse_layout(images: &[ParsedInput]) -> Result<Vec<CombineLayout>, ParseLayoutError> {
        let mut layout = vec![];
        let mut urls: Vec<ImageUrl> = vec![];
        let mut cur_spec: Option<(usize, &TileGroupInfo)> = None;

        for (i, input) in images.iter().enumerate() {
            match input {
                ParsedInput::LocalPath(path) => {
                    let image_url = parse_image_url(&path.to_string_lossy());
                    if cur_spec.is_some() {
                        urls.push(image_url);
                    } else {
                        layout.push(CombineLayout::Single(image_url));
                    }
                }
                ParsedInput::TileSpec(spec) => {
                    flush_tile_group(cur_spec, &mut urls, &mut layout)?;
                    cur_spec = Some((i, spec));
                }
                ParsedInput::SingleSeparator => {
                    flush_tile_group(cur_spec, &mut urls, &mut layout)?;
                    cur_spec = None;
                }
                _ => unimplemented!(),
            }
        }
        flush_tile_group(cur_spec, &mut urls, &mut layout)?;

        Ok(layout)
    }

    pub fn should_delete_sources(&self) -> bool {
        self.delete
    }
}

fn flush_tile_group(
    cur_spec: Option<(usize, &TileGroupInfo)>,
    urls: &mut Vec<ImageUrl>,
    layout: &mut Vec<CombineLayout>,
) -> Result<(), ParseLayoutError> {
    if let Some((si, spec)) = cur_spec {
        if urls.is_empty() {
            return Err(ParseLayoutError::new(spec, si, NoImages));
        }
        layout.push(CombineLayout::Tiled(spec.clone(), std::mem::take(urls)));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use super::*;

    #[test]
    fn parse_resize_recognizes_every_supported_form() {
        const CASES: &[(&str, ImageResizeUnit)] = &[
            (
                "800",
                ImageResizeUnit::Width {
                    w: 800,
                    limit: ImageResizeLimit::None,
                },
            ),
            (
                "800<",
                ImageResizeUnit::Width {
                    w: 800,
                    limit: ImageResizeLimit::Enlarge,
                },
            ),
            (
                "800>",
                ImageResizeUnit::Width {
                    w: 800,
                    limit: ImageResizeLimit::Shrink,
                },
            ),
            (
                "x600",
                ImageResizeUnit::Height {
                    h: 600,
                    limit: ImageResizeLimit::None,
                },
            ),
            (
                "x600>",
                ImageResizeUnit::Height {
                    h: 600,
                    limit: ImageResizeLimit::Shrink,
                },
            ),
            (
                "800x600",
                ImageResizeUnit::WidthHeightFit {
                    w: 800,
                    h: 600,
                    limit: ImageResizeLimit::None,
                },
            ),
            (
                "800x600<",
                ImageResizeUnit::WidthHeightFit {
                    w: 800,
                    h: 600,
                    limit: ImageResizeLimit::Enlarge,
                },
            ),
            ("50%", ImageResizeUnit::Percentage { p: 50.0 }),
        ];

        for (input, expected) in CASES {
            assert_eq!(
                parse_resize(input).unwrap(),
                *expected,
                "unexpected result parsing {input:?}"
            );
        }
    }

    #[test]
    fn parse_resize_reports_which_component_was_invalid() {
        assert_matches!(parse_resize("abcx600"), Err(ParseResizeError::InvalidWidth));
        assert_matches!(
            parse_resize("800xabc"),
            Err(ParseResizeError::InvalidHeight)
        );
        assert_matches!(
            parse_resize("800x600xabc"),
            Err(ParseResizeError::InvalidHeight)
        );
        assert_matches!(parse_resize("xxabc"), Err(ParseResizeError::InvalidHeight));
        assert_matches!(
            parse_resize("abc%"),
            Err(ParseResizeError::InvalidPercentage)
        );
        assert_matches!(parse_resize(""), Err(ParseResizeError::InvalidWidth));
    }

    #[test]
    fn parse_tile_spec_recognizes_every_supported_form() {
        const CASES: &[(&str, TileGroupInfo)] = &[
            (
                "rows:3",
                TileGroupInfo {
                    line_length: 3,
                    fill_order: TileFillOrder::RowByRow,
                    fit_by: TileFitBy::WidthHeight,
                    gap_size: LineSizeUnit::Exact(0),
                    last_alignment: TileLastAlignment::Left,
                },
            ),
            (
                "cols:5",
                TileGroupInfo {
                    line_length: 5,
                    fill_order: TileFillOrder::ColumnByColumn,
                    fit_by: TileFitBy::WidthHeight,
                    gap_size: LineSizeUnit::Exact(0),
                    last_alignment: TileLastAlignment::Left,
                },
            ),
            (
                "cols:7,align=center,fit=w",
                TileGroupInfo {
                    line_length: 7,
                    fill_order: TileFillOrder::ColumnByColumn,
                    fit_by: TileFitBy::Width,
                    gap_size: LineSizeUnit::Exact(0),
                    last_alignment: TileLastAlignment::Center,
                },
            ),
            (
                "rows:100,fit=h,align=space-between",
                TileGroupInfo {
                    line_length: 100,
                    fill_order: TileFillOrder::RowByRow,
                    fit_by: TileFitBy::Height,
                    gap_size: LineSizeUnit::Exact(0),
                    last_alignment: TileLastAlignment::SpaceBetween,
                },
            ),
            (
                "rows:89+20,fit=wh",
                TileGroupInfo {
                    line_length: 89,
                    fill_order: TileFillOrder::RowByRow,
                    fit_by: TileFitBy::WidthHeight,
                    gap_size: LineSizeUnit::Exact(20),
                    last_alignment: TileLastAlignment::Left,
                },
            ),
            (
                "rows:89+25.125%,align=right",
                TileGroupInfo {
                    line_length: 89,
                    fill_order: TileFillOrder::RowByRow,
                    fit_by: TileFitBy::WidthHeight,
                    gap_size: LineSizeUnit::Percentage(25.125),
                    last_alignment: TileLastAlignment::Right,
                },
            ),
        ];

        for (input, expected) in CASES {
            assert_eq!(
                parse_tile_spec(input).unwrap(),
                *expected,
                "unexpected result parsing {input:?}"
            );
        }
    }

    #[test]
    fn parse_tile_spec_reports_which_component_was_invalid() {
        use ParseLayoutErrorKind::*;

        assert_matches!(parse_tile_spec(""), Err(NotEnoughParams));
        assert_matches!(parse_tile_spec("rows"), Err(NotEnoughParams));
        assert_matches!(parse_tile_spec("rows:"), Err(NotEnoughParams));
        assert_matches!(parse_tile_spec("rows:+"), Err(NotEnoughParams));
        assert_matches!(parse_tile_spec("rows:+10"), Err(NotEnoughParams));
        assert_matches!(parse_tile_spec("diagonal:5"), Err(InvalidFillOrder(_)));
        assert_matches!(parse_tile_spec("r:5abc"), Err(InvalidLineLength(_)));
        assert_matches!(parse_tile_spec("rows:10+10%!!!"), Err(InvalidGap(_)));
        assert_matches!(
            parse_tile_spec("rows:10,xxxx=42"),
            Err(InvalidOptParamName(_))
        );
        assert_matches!(
            parse_tile_spec("rows:10,xxxx="),
            Err(InvalidOptParamName(_))
        );
        assert_matches!(parse_tile_spec("rows:10,xxxx"), Err(InvalidOptParam));
        assert_matches!(parse_tile_spec("rows:10,="), Err(InvalidOptParam));
        assert_matches!(parse_tile_spec("rows:10,"), Err(InvalidOptParam));
    }
}
