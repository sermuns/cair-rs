#![no_std]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;
use image::{GrayImage, ImageBuffer, Luma, Rgb, RgbImage};

/*
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
*/

pub fn compute_gradient_magnitude(img: &RgbImage, out: &mut RgbImage) {
    for (x, y, Rgb(out_rgb)) in out.enumerate_pixels_mut() {
        if x == 0 || y == 0 {
            continue;
        }

        let Rgb([left_r, left_g, left_b]) = img[(x - 1, y)];
        let Rgb([above_r, above_g, above_b]) = img[(x, y - 1)];
        let Rgb([here_r, here_g, here_b]) = img[(x, y)];

        *out_rgb = [
            (here_r.abs_diff(left_r)).saturating_add(here_r.abs_diff(above_r)),
            (here_g.abs_diff(left_g)).saturating_add(here_g.abs_diff(above_g)),
            (here_b.abs_diff(left_b)).saturating_add(here_b.abs_diff(above_b)),
        ];
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum MatchDirection {
    Straight,
    Cross,
}

// TODO: Stop allocating. Take mut ref to slice(s) instead!
/// Works for both vertical/horizontal
fn match_adjacent(row_or_column_k_a: &[f32], row_or_column_kp1_m: &[f32]) -> Vec<usize> {
    let m_or_n = row_or_column_k_a.len();
    assert_eq!(m_or_n, row_or_column_kp1_m.len());

    let w = |i: usize, j: usize| -> f32 {
        if (i as isize - j as isize).abs() <= 1 {
            row_or_column_k_a[i] * row_or_column_kp1_m[j]
        } else {
            f32::NEG_INFINITY
        }
    };

    let mut f = vec![0.0; m_or_n + 1];
    let mut choices = vec![MatchDirection::Straight; m_or_n + 1];

    for i in 1..=m_or_n {
        let f1 = f[i - 1] + w(i - 1, i - 1);

        let f2 = if i >= 2 {
            f[i - 2] + w(i - 1, i - 2) + w(i - 2, i - 1)
        } else {
            f32::NEG_INFINITY
        };

        if f1 >= f2 {
            f[i] = f1;
            choices[i] = MatchDirection::Straight;
        } else {
            f[i] = f2;
            choices[i] = MatchDirection::Cross;
        }
    }

    let mut matches = vec![0; m_or_n];
    let mut i = m_or_n;
    while i > 0 {
        match choices[i] {
            MatchDirection::Straight => {
                matches[i - 1] = i - 1;
                i -= 1;
            }
            MatchDirection::Cross => {
                matches[i - 1] = i - 2;
                matches[i - 2] = i - 1;
                i -= 2;
            }
        }
    }

    matches
}

pub enum SeamDirection {
    Vertical,
    Horizontal,
}

pub fn remove_seams(
    img: &RgbImage,
    energy: &GrayImage,
    num_horizontal_seams_to_remove: usize,
    num_vertical_seams_to_remove: usize,
) -> RgbImage {
    let width = img.width() as usize;
    let height = img.height() as usize;
    assert_eq!(width, energy.width() as usize);
    assert_eq!(height, energy.height() as usize);
    assert!(num_vertical_seams_to_remove < width);
    assert!(num_horizontal_seams_to_remove < height);

    let mut e = vec![vec![0.; width]; height];
    for (x, y, Luma([value])) in energy.enumerate_pixels() {
        e[y as usize][x as usize] = (*value) as f32;
    }

    let mut m_matrix = vec![vec![0.; width]; height];
    for x in 0..width {
        m_matrix[height - 1][x] = e[height - 1][x];
    }
    for y in (0..height - 1).rev() {
        for x in 0..width {
            let mut min_next = m_matrix[y + 1][x];
            if x > 0 {
                min_next = min_next.min(m_matrix[y + 1][x - 1]);
            }
            if x < width - 1 {
                min_next = min_next.min(m_matrix[y + 1][x + 1]);
            }
            m_matrix[y][x] = e[y][x] + min_next;
        }
    }

    let mut a_matrix = vec![vec![0.0; width]; height];
    for x in 0..width {
        a_matrix[0][x] = e[0][x];
    }

    let mut parent_map = vec![vec![0; width]; height];

    for y in 0..height - 1 {
        let row_matches = match_adjacent(&a_matrix[y], &m_matrix[y + 1]);

        for x in 0..width {
            let matched_x = row_matches[x];
            a_matrix[y + 1][matched_x] = a_matrix[y][x] + e[y + 1][matched_x];
            parent_map[y + 1][matched_x] = x;
        }
    }

    let mut seams = Vec::with_capacity(width);
    for start_x in 0..width {
        let mut path = vec![0; height];
        let mut current_x = start_x;
        let total_energy = a_matrix[height - 1][start_x];

        for y in (0..height).rev() {
            path[y] = current_x;
            if y > 0 {
                current_x = parent_map[y][current_x];
            }
        }
        seams.push((total_energy, path));
    }

    seams.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let mut removed_mask = vec![vec![false; width]; height];
    for i in 0..num_vertical_seams_to_remove {
        let (_, path) = &seams[i];
        for y in 0..height {
            removed_mask[y][path[y]] = true;
        }
    }

    let new_width = width - num_vertical_seams_to_remove;
    let new_height = height - num_horizontal_seams_to_remove;
    let mut output_img = ImageBuffer::new(new_width as u32, new_height as u32);

    for y in 0..height {
        let mut out_x = 0;
        for x in 0..width {
            if !removed_mask[y][x] {
                let pixel = img[(x as u32, y as u32)];
                output_img.put_pixel(out_x as u32, y as u32, pixel);
                out_x += 1;
            }
        }
    }

    output_img
}
