#![no_std]

use image::{GenericImage, GenericImageView, RgbImage};

pub fn compute_gradient_x_of_image(img: &RgbImage, grad_x: &mut RgbImage) {
    for (_, row) in grad_x.enumerate_rows_mut() {
        for (x, y, pixel) in row {
            pixel.0 = [0xff, 0xff, 0x00];
        }
    }
}

pub fn compute_gradient_y_of_image(img: &RgbImage, grad_x: &mut RgbImage) {}

pub fn compute_energy_of_image(grad_x: &RgbImage, grad_y: &RgbImage, out: &mut RgbImage) {
    for (x, y, pixel) in out.enumerate_pixels_mut() {}
}

pub fn establish_matching_relations() {
    todo!()
}

pub fn compute_energy_of_seam() {
    todo!()
}
