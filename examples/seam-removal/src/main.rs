use anyhow::bail;
use cair::compute_gradient_x_of_image;
use image::{ImageReader, RgbImage};

fn main() -> anyhow::Result<()> {
    let Some(img_path) = std::env::args().nth(1) else {
        bail!("no image path provided!");
    };

    let img = ImageReader::open(img_path)?.decode()?.into_rgb8();

    let mut grad_x = RgbImage::new(img.width(), img.height());

    compute_gradient_x_of_image(&img, &mut grad_x);

    grad_x.save("out.png")?;

    Ok(())
}
