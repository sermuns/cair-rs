#![no_std]

use image::{Pixel, Rgb, RgbImage};

/// Compute vertical gradient (above minus below)
pub fn compute_gradient_y_of_image(img: &RgbImage, grad_x: &mut RgbImage) {
    for (x, y, Rgb([r, g, b])) in grad_x
        .enumerate_pixels_mut()
        .skip(img.width().try_into().unwrap())
    {
        let Rgb([above_r, above_g, above_b]) = img[(x, y - 1)];
        let Rgb([here_r, here_g, here_b]) = img[(x, y)];

        *r = above_r.saturating_sub(here_r);
        *g = above_g.saturating_sub(here_g);
        *b = above_b.saturating_sub(here_b);
    }
}

/// Compute horizontal gradient (left minus right)
pub fn compute_gradient_x_of_image(img: &RgbImage, grad_y: &mut RgbImage) {
    for (x, y, Rgb([r, g, b])) in grad_y.enumerate_pixels_mut() {
        if x == 0 {
            continue;
        }

        let Rgb([left_r, left_g, left_b]) = img[(x - 1, y)];
        let Rgb([here_r, here_g, here_b]) = img[(x, y)];

        *r = left_r.saturating_sub(here_r);
        *g = left_g.saturating_sub(here_g);
        *b = left_b.saturating_sub(here_b);
    }
}

/// Sum `grad_x` and `grad_y`
pub fn compute_gradient_magnitude(grad_x: &RgbImage, grad_y: &RgbImage, out: &mut RgbImage) {
    for ((grad_x_pixel, grad_y_pixel), out_pixel) in
        grad_x.pixels().zip(grad_y.pixels()).zip(out.pixels_mut())
    {
        for ((grad_x_c, grad_y_c), out_c) in grad_x_pixel
            .channels()
            .iter()
            .zip(grad_y_pixel.channels())
            .zip(out_pixel.channels_mut())
        {
            *out_c = grad_x_c.saturating_add(*grad_y_c);
        }
    }
}

pub fn establish_matching_relations() {
    todo!()
}

pub fn compute_energy_of_seam() {
    todo!()
}
