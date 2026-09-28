use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use gdl_utils::combine::{cli::CombineArgs, composite::ImageCanvas};
use image::{DynamicImage, GenericImageView};

/// An intricate wrapper around gallery-dl with extra functionality.
///
/// Download supports ID-to-URL resolution with link expansion and metadata embedding.
#[derive(Parser)]
#[command(about)]
struct Cli {
    #[arg(short, long, value_name = "KEY=VALUE")]
    options: Vec<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Utility for retrieving and downloading metadata from specified URLs or local JSON files.
    Metadata,
    /// Performs image compositing by tiling multiple images and stacking the resulting tiles.
    Combine(CombineArgs),
    /// Downloads images using URLs.
    Download,
    /// Converts an identifier (e.g., imgco:42) into an expanded URL (e.g., https://image.co/img/42).
    ExpandId,
    /// Initiates a graphical user interface for image combining operations.
    Gui,
}

fn main() -> Result<()> {
    let matches = Cli::parse();

    match matches.command {
        Commands::Combine(args) => do_combine(args)?,
        cmd => unimplemented!("{cmd:?}"),
    }

    Ok(())
}

fn do_combine(args: CombineArgs) -> Result<()> {
    let path = args.get_output();
    let mut combiner = args.to_combiner()?;

    let canvas = ImageCanvas::from_combiner(combiner.clone())?;
    let image = DynamicImage::from(canvas.build());
    let (canvas_w, canvas_h) = canvas.dimensions();
    let scale = canvas.scale();
    let (final_w, final_h) = image.dimensions();

    println!("Canvas size: {canvas_w} x {canvas_h}");
    println!("Scale: {scale:.2}");
    if scale != 1.0 {
        println!("Size after scale: {final_w} x {final_h}");
    }

    let res = image
        .save(path)
        .with_context(|| format!("Cannot save to `{path}`"));
    if res.is_err() {
        let _ = std::fs::remove_file(path);
    }
    res?;

    if args.request_delete()
        && let Err(errs) = combiner.delete_sources()
    {
        println!("Cannot delete {} of the following images:", errs.len());
        for (path, err) in errs {
            println!("  {path}: {err}");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use glob::glob;

    #[derive(Parser)]
    struct TestCombine {
        #[command(flatten)]
        args: CombineArgs,
    }

    fn listglob(path: &str) -> Vec<String> {
        glob(path)
            .unwrap()
            .map(|p| p.unwrap().into_string().unwrap())
            .collect()
    }

    fn repeat_path(path: &str, n: usize) -> Vec<String> {
        std::iter::repeat_n(path.to_string(), n).collect()
    }

    fn map_path(paths: &[&str], parent: &str) -> Vec<String> {
        paths
            .iter()
            .map(|p| {
                if p.starts_with('+') {
                    p.to_string()
                } else {
                    parent.to_string() + p
                }
            })
            .collect()
    }

    fn tmp_output(path: &str) -> String {
        let dir = std::env::temp_dir().join("gdl-combine-outputs");
        std::fs::create_dir_all(&dir).unwrap();

        dir.join(path).to_string_lossy().to_string()
    }

    macro_rules! test_image_generate {
        ($out:ident, $ext:expr, $imgs:expr $(, $params:expr)? $(,)?) => {
            paste::paste! {
                #[test]
                fn [<test_ $out>]() {
                    let images: Vec<String> = $imgs;
                    let output: String = tmp_output(concat!(stringify!([<output_ $out>]), $ext));

                    let mut args = vec!["", "-o", &output];
                    $(
                        args.extend($params);
                    )?
                    args.extend(images.iter().map(String::as_str));

                    let combine_args = TestCombine::parse_from(args).args;
                    do_combine(combine_args).unwrap()
                }
            }
        };
    }

    test_image_generate!(
        defaults,
        ".jpg",
        listglob("tests/inputs/placeholder/1200x800-*.jpg")
    );

    test_image_generate!(
        horizontal,
        ".jpg",
        listglob("tests/inputs/placeholder/1200x800-*.jpg"),
        ["-O", "horizontal"]
    );

    test_image_generate!(
        horizontal_stack_plus_gap,
        ".jpg",
        listglob("tests/inputs/placeholder/1200x800-*.jpg"),
        ["-O", "horizontal", "--gap", "10"]
    );

    test_image_generate!(
        horizontal_stack_plus_gap_percent,
        ".jpg",
        listglob("tests/inputs/placeholder/1200x800-*.jpg"),
        ["-O", "horizontal", "--gap", "10%"]
    );

    test_image_generate!(
        horizontal_stack_plus_border,
        ".jpg",
        listglob("tests/inputs/placeholder/1200x800-*.jpg"),
        ["-O", "horizontal", "--border-size", "50"]
    );

    test_image_generate!(
        horizontal_stack_plus_border_percent,
        ".jpg",
        listglob("tests/inputs/placeholder/1200x800-*.jpg"),
        ["-O", "horizontal", "--border-size", "12.5%"]
    );

    test_image_generate!(
        background_rgb_hex,
        ".jpg",
        listglob("tests/inputs/placeholder/1200x800-*.jpg"),
        [
            "--border-size",
            "12.5%",
            "--gap",
            "10%",
            "--background",
            "#7c2"
        ]
    );

    test_image_generate!(
        background_rgb_hex_2,
        ".jpg",
        listglob("tests/inputs/placeholder/1200x800-*.jpg"),
        [
            "--border-size",
            "12.5%",
            "--gap",
            "10%",
            "--background",
            "#a040cc"
        ]
    );

    test_image_generate!(
        background_rgb,
        ".jpg",
        listglob("tests/inputs/placeholder/1200x800-*.jpg"),
        [
            "--border-size",
            "12.5%",
            "--gap",
            "10%",
            "--background",
            "rgb(16,114,109)"
        ]
    );

    test_image_generate!(
        background_rgba,
        ".png",
        listglob("tests/inputs/placeholder/1200x800-*.jpg"),
        [
            "--border-size",
            "12.5%",
            "--gap",
            "10%",
            "--background",
            "rgba(16,114,109,0.52)"
        ]
    );

    test_image_generate!(
        background_named_color,
        ".jpg",
        listglob("tests/inputs/placeholder/1200x800-*.jpg"),
        [
            "--border-size",
            "12.5%",
            "--gap",
            "10%",
            "--background",
            "green"
        ]
    );

    test_image_generate!(
        background_named_color_2,
        ".jpg",
        listglob("tests/inputs/placeholder/1200x800-*.jpg"),
        [
            "--border-size",
            "12.5%",
            "--gap",
            "10%",
            "--background",
            "firebrick"
        ]
    );

    test_image_generate!(
        background_rgb_from,
        ".jpg",
        listglob("tests/inputs/placeholder/1200x800-*.jpg"),
        [
            "--border-size",
            "12.5%",
            "--gap",
            "10%",
            "--background",
            "rgb(from gold calc(r+10) calc(g-10) calc(b+20))"
        ]
    );

    test_image_generate!(
        background_resize_width_1000,
        ".jpg",
        listglob("tests/inputs/placeholder/1200x800-*.jpg"),
        ["-O", "horizontal", "--resize", "1000"]
    );

    test_image_generate!(
        background_resize_height_1000,
        ".jpg",
        listglob("tests/inputs/placeholder/1200x800-*.jpg"),
        ["-O", "horizontal", "--resize", "x1000"]
    );

    test_image_generate!(
        tiling,
        ".png",
        repeat_path("tests/inputs/placeholder/256.png", 12),
        ["-O", "horizontal", "+rows:3"]
    );

    test_image_generate!(
        tiling_gap_30,
        ".png",
        repeat_path("tests/inputs/placeholder/256.png", 12),
        ["-O", "horizontal", "+rows:3+30"]
    );

    test_image_generate!(
        tiling_gap_30_percent,
        ".png",
        repeat_path("tests/inputs/placeholder/256.png", 12),
        ["-O", "horizontal", "+rows:3+30%"]
    );

    test_image_generate!(
        heavy_tiling,
        ".jpg",
        listglob("tests/inputs/picsum/*-3000x5000.jpg"),
        ["+rows:3"]
    );

    test_image_generate!(
        heavy_tiling_resize_width_1000,
        ".jpg",
        listglob("tests/inputs/picsum/*-3000x5000.jpg"),
        ["--resize", "1000", "+rows:3"]
    );

    test_image_generate!(
        heavy_tiling_resize_1000x1000,
        ".jpg",
        listglob("tests/inputs/picsum/*-3000x5000.jpg"),
        ["--resize", "1000x1000", "+rows:3"]
    );

    test_image_generate!(
        hstack_various_sizes,
        ".png",
        map_path(
            &[
                "64.png",
                "128.png",
                "64.png",
                "128x64.png",
                "64.png",
                "64x128.png",
            ],
            "tests/inputs/placeholder/"
        ),
        ["-O", "horizontal"]
    );

    test_image_generate!(
        vstack_various_sizes,
        ".png",
        map_path(
            &[
                "200x100.png",
                "300x100.png",
                "400x100.png",
                "400x200.png",
                "800x500.png",
                "256x128.png",
            ],
            "tests/inputs/placeholder/"
        ),
        ["-O", "vertical"]
    );

    test_image_generate!(
        vstack_various_sizes_resize_2000,
        ".png",
        map_path(
            &[
                "200x100.png",
                "300x100.png",
                "400x100.png",
                "400x200.png",
                "800x500.png",
                "256x128.png",
            ],
            "tests/inputs/placeholder/"
        ),
        ["-O", "vertical", "--resize", "2000"]
    );

    test_image_generate!(
        tiling_various_sizes,
        ".png",
        map_path(
            &[
                "64.png",
                "64x128.png",
                "128.png",
                "400x200.png",
                "128x64.png",
                "800.png",
                "256x128.png",
                "400x100.png",
                "100.png",
                "128x256.png",
                "256.png",
                "300x100.png",
            ],
            "tests/inputs/placeholder/"
        ),
        ["+rows:3"]
    );

    test_image_generate!(
        tiling_various_sizes_fit_width,
        ".png",
        map_path(
            &[
                "64.png",
                "64x128.png",
                "128.png",
                "400x200.png",
                "128x64.png",
                "800.png",
                "256x128.png",
                "400x100.png",
                "100.png",
                "128x256.png",
                "256.png",
                "300x100.png",
            ],
            "tests/inputs/placeholder/"
        ),
        ["+rows:3,fit=w"]
    );

    test_image_generate!(
        tiling_various_sizes_fit_height,
        ".png",
        map_path(
            &[
                "64.png",
                "64x128.png",
                "128.png",
                "400x200.png",
                "128x64.png",
                "800.png",
                "256x128.png",
                "400x100.png",
                "100.png",
                "128x256.png",
                "256.png",
                "300x100.png",
            ],
            "tests/inputs/placeholder/"
        ),
        ["+rows:3,fit=h"]
    );

    test_image_generate!(
        tiling_various_sizes_fit_height_2,
        ".png",
        map_path(
            &[
                "200x100.png",
                "300x100.png",
                "400x100.png",
                "400x200.png",
                "800x500.png",
                "256x128.png",
            ],
            "tests/inputs/placeholder/"
        ),
        ["+rows:3,fit=h"]
    );

    test_image_generate!(
        tiling_various_sizes_rows_single,
        ".png",
        map_path(
            &[
                "200x100.png",
                "300x100.png",
                "400x100.png",
                "400x200.png",
                "800x500.png",
                "256x128.png",
            ],
            "tests/inputs/placeholder/"
        ),
        ["+rows:1"]
    );

    test_image_generate!(
        tiling_various_sizes_cols_single,
        ".png",
        map_path(
            &[
                "200x100.png",
                "300x100.png",
                "400x100.png",
                "400x200.png",
                "800x500.png",
                "256x128.png",
            ],
            "tests/inputs/placeholder/"
        ),
        ["+cols:1"]
    );

    test_image_generate!(
        tiling_various_sizes_tile_gap_5percent_border_30percent,
        ".png",
        map_path(
            &[
                "64.png",
                "64x128.png",
                "128.png",
                "128x64.png",
                "800.png",
                "256x128.png",
                "100.png",
                "128x256.png",
                "256.png",
            ],
            "tests/inputs/placeholder/"
        ),
        ["--border-size", "5%", "+rows:3+30%"]
    );

    test_image_generate!(
        tiling_rem_align_left,
        ".png",
        repeat_path("tests/inputs/placeholder/256.png", 13),
        ["+cols:5,align=left"]
    );

    test_image_generate!(
        tiling_rem_align_right,
        ".png",
        repeat_path("tests/inputs/placeholder/256.png", 13),
        ["+cols:5,align=right"]
    );

    test_image_generate!(
        tiling_rem_align_center,
        ".png",
        repeat_path("tests/inputs/placeholder/256.png", 13),
        ["+cols:5,align=center"]
    );

    test_image_generate!(
        tiling_rem_align_space_between,
        ".png",
        repeat_path("tests/inputs/placeholder/256.png", 13),
        ["+cols:5,align=space-between"]
    );

    test_image_generate!(
        tiling_rem_align_space_around,
        ".png",
        repeat_path("tests/inputs/placeholder/256.png", 13),
        ["+cols:5,align=space-around"]
    );

    test_image_generate!(
        tiling_rem_align_space_evenly,
        ".png",
        repeat_path("tests/inputs/placeholder/256.png", 13),
        ["+cols:5,align=space-evenly"]
    );

    test_image_generate!(
        tiling_rem_rows,
        ".png",
        repeat_path("tests/inputs/placeholder/256.png", 12),
        ["+rows:5"]
    );

    test_image_generate!(
        tiling_rem_cols_align_space_between,
        ".png",
        repeat_path("tests/inputs/placeholder/256.png", 12),
        ["+cols:18,align=space-between"]
    );

    test_image_generate!(
        tiling_resize_bound_1000x1000,
        ".png",
        repeat_path("tests/inputs/placeholder/256.png", 12),
        ["--resize", "1000x1000", "+rows:3+30"]
    );

    test_image_generate!(
        tiling_resize_shrink_width_1000,
        ".png",
        repeat_path("tests/inputs/placeholder/256.png", 12),
        ["--resize", "1000>", "+rows:3+30"]
    );

    test_image_generate!(
        tiling_resize_enlarge_height_2000,
        ".png",
        repeat_path("tests/inputs/placeholder/256.png", 12),
        ["--resize", "x2000<", "+rows:3+30"]
    );

    test_image_generate!(
        tiling_resize_enlarge_height_800_no_effect,
        ".png",
        repeat_path("tests/inputs/placeholder/256.png", 12),
        ["--resize", "x800<", "+rows:3+30"]
    );
}
