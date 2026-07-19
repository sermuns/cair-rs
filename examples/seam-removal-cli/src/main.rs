use std::{path::PathBuf, time::Instant};

use anyhow::ensure;
use cair::{compute_gradient_magnitude, remove_seams};
use clap::Parser;
use image::{DynamicImage, ImageReader, RgbImage};

#[derive(Parser)]
struct Args {
    img_path: PathBuf,

    output_path: PathBuf,

    #[arg(short = 'v', long, default_value_t = 0)]
    vertical_seams_to_remove: usize,

    #[arg(short = 'H', long, default_value_t = 0)]
    horizontal_seams_to_remove: usize,
}

fn main() -> anyhow::Result<()> {
    let Args {
        img_path,
        output_path,
        vertical_seams_to_remove,
        horizontal_seams_to_remove,
    } = Args::parse();

    ensure!(
        vertical_seams_to_remove > 0 || horizontal_seams_to_remove > 0,
        "You haven't requested to remove any seams!"
    );

    let img = ImageReader::open(img_path)?.decode()?.into_rgb8();
    let (width, height) = img.dimensions();

    let before = Instant::now();

    let mut grad_magnitude = RgbImage::new(width, height);
    compute_gradient_magnitude(&img, &mut grad_magnitude);

    println!("computed gradient magnitude in {:?}", before.elapsed());

    let before = Instant::now();

    let energy = DynamicImage::ImageRgb8(grad_magnitude).into_luma8();

    let carved = remove_seams(
        &img,
        &energy,
        horizontal_seams_to_remove,
        vertical_seams_to_remove,
    );

    println!("removed seams in {:?}", before.elapsed());

    carved.save(output_path)?;

    Ok(())
}
