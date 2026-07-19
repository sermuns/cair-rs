use std::time::Instant;

use anyhow::bail;
use cair::{compute_gradient_magnitude, remove_seams};
use image::{DynamicImage, ImageReader, RgbImage};

fn main() -> anyhow::Result<()> {
    let Some(img_path) = std::env::args().nth(1) else {
        bail!("no image path provided!");
    };

    let img = ImageReader::open(img_path)?.decode()?.into_rgb8();
    let (width, height) = img.dimensions();

    let before = Instant::now();

    // let mut grad_x = RgbImage::new(width, height);
    // compute_gradient_x_of_image(&img, &mut grad_x);
    // let mut grad_y = RgbImage::new(width, height);
    // compute_gradient_y_of_image(&img, &mut grad_y);

    let mut grad_magnitude = RgbImage::new(width, height);
    compute_gradient_magnitude(&img, &mut grad_magnitude);

    println!("computed gradient magnitude in {:?}", before.elapsed());

    let before = Instant::now();

    let energy = DynamicImage::ImageRgb8(grad_magnitude).into_luma8();

    let carved = remove_seams(&img, &energy, 200);

    println!("removed seams in {:?}", before.elapsed());

    carved.save("out.png")?;

    Ok(())
}
