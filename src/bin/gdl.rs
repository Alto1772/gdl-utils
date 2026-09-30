use std::time::Instant;

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
    /// [TODO] gallery-dl configuration options
    #[arg(short, long, value_name = "KEY=VALUE")]
    options: Vec<String>,

    /// Enable debugging information
    #[arg(short, long)]
    verbose: bool,

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
        Commands::Combine(args) => do_combine(args, matches.verbose)?,
        cmd => unimplemented!("{cmd:?}"),
    }

    Ok(())
}

fn do_combine(args: CombineArgs, verbose: bool) -> Result<()> {
    macro_rules! v_eprintln {
        ($($arg:tt)*) => {
            if verbose {
                eprintln!($($arg)*);
            }
        };
    }

    let path = args.get_output();
    let combiner = args.to_combiner()?;

    let time_read_start = Instant::now();
    let mut canvas = ImageCanvas::from_combiner(combiner)?;
    let (canvas_w, canvas_h) = canvas.dimensions();
    let scale = canvas.scale();
    v_eprintln!("Time taken to read images: {:?}", time_read_start.elapsed());
    v_eprintln!("Canvas size: {canvas_w} x {canvas_h}");
    v_eprintln!("Scale: {scale:.2}");
    v_eprintln!();

    let time_build_start = Instant::now();
    let image = DynamicImage::from(canvas.build());
    v_eprintln!(
        "Time taken to generate image: {:?}",
        time_build_start.elapsed()
    );
    if scale != 1.0 {
        let (final_w, final_h) = image.dimensions();
        v_eprintln!("Size after scale: {final_w} x {final_h}");
    }

    let time_save_start = Instant::now();
    let res = image
        .save(path)
        .with_context(|| format!("Cannot save to `{path}`"));
    v_eprintln!("Time taken to save image: {:?}", time_save_start.elapsed());
    if let Err(e) = res {
        let _ = std::fs::remove_file(path);
        return Err(e);
    }
    v_eprintln!("Image creation success.");

    if args.should_delete_sources()
        && let Err(errs) = canvas.delete_sources()
    {
        v_eprintln!("Cannot delete {} of the following images:", errs.len());
        for (path, err) in errs {
            v_eprintln!("  {path}: {err}");
        }
    }

    Ok(())
}
