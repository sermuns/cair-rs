#![no_std]

use image::{Rgb, RgbImage};

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

pub fn compute_gradient_magnitude(img: &RgbImage, out: &mut RgbImage) {
    let width = img.width().try_into().unwrap();
    for (x, y, Rgb([r, g, b])) in out.enumerate_pixels_mut().skip(width) {
        if x == 0 {
            continue;
        }

        let Rgb([left_r, left_g, left_b]) = img[(x - 1, y)];
        let Rgb([above_r, above_g, above_b]) = img[(x, y - 1)];
        let Rgb([here_r, here_g, here_b]) = img[(x, y)];

        *r = (here_r)
            .saturating_mul(2)
            .saturating_sub(left_r)
            .saturating_sub(above_r);
        *g = (here_g)
            .saturating_mul(2)
            .saturating_sub(left_g)
            .saturating_sub(above_g);
        *b = (here_b)
            .saturating_mul(2)
            .saturating_sub(left_b)
            .saturating_sub(above_b);
    }
}

pub fn establish_matching_relations() {
    todo!()
}

pub fn compute_energy_of_seam() {
    todo!()
}
