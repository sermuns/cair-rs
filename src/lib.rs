#![no_std]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;
use image::{GrayImage, ImageBuffer, Luma, Rgb, RgbImage};

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

fn set_removed(
    e: &Vec<Vec<f32>>,
    orig_width: usize,
    orig_height: usize,
    num_seams_to_remove: usize,
    seam_direction: SeamDirection,
    removed_mask: &mut Vec<Vec<bool>>,
) {
    let (rows, cols) = match seam_direction {
        SeamDirection::Vertical => (orig_height, orig_width),
        SeamDirection::Horizontal => (orig_width, orig_height), // HACK: transposed
    };
    let get_energy = |r: usize, c: usize| {
        match seam_direction {
            SeamDirection::Vertical => e[r][c],
            SeamDirection::Horizontal => e[c][r], // HACK: transposed
        }
    };

    let mut m_matrix = vec![vec![0.; cols]; rows];
    for c in 0..cols {
        m_matrix[rows - 1][c] = get_energy(rows - 1, c);
    }

    for r in (0..rows - 1).rev() {
        for c in 0..cols {
            let mut min_next = m_matrix[r + 1][c];
            if c > 0 {
                min_next = min_next.min(m_matrix[r + 1][c - 1]);
            }
            if c < cols - 1 {
                min_next = min_next.min(m_matrix[r + 1][c + 1]);
            }
            m_matrix[r][c] = get_energy(r, c) + min_next;
        }
    }

    let mut a_matrix = vec![vec![0.0; cols]; rows];
    for c in 0..cols {
        a_matrix[0][c] = get_energy(0, c);
    }

    let mut parent_map = vec![vec![0; cols]; rows];

    for r in 0..rows - 1 {
        let matches = match_adjacent(&a_matrix[r], &m_matrix[r + 1]);

        for c in 0..cols {
            let matched_c = matches[c];
            a_matrix[r + 1][matched_c] = a_matrix[r][c] + get_energy(r + 1, matched_c);
            parent_map[r + 1][matched_c] = c;
        }
    }

    let mut seams = Vec::with_capacity(cols);
    for start_c in 0..cols {
        let mut path = vec![0; rows];
        let mut current_c = start_c;
        let total_energy = a_matrix[rows - 1][start_c];

        for r in (0..rows).rev() {
            path[r] = current_c;
            if r > 0 {
                current_c = parent_map[r][current_c];
            }
        }
        seams.push((total_energy, path));
    }

    seams.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    for path in seams.iter().map(|(_, path)| path).take(num_seams_to_remove) {
        for (r, c) in path.iter().enumerate().take(rows) {
            match seam_direction {
                SeamDirection::Vertical => {
                    removed_mask[r][*c] = true;
                }
                SeamDirection::Horizontal => {
                    removed_mask[*c][r] = true;
                }
            }
        }
    }
}

// TODO: less alloc
pub fn remove_seams(
    img: &RgbImage,
    energy: &GrayImage,
    num_horizontal_seams_to_remove: usize,
    num_vertical_seams_to_remove: usize,
) -> RgbImage {
    let mut output_img = img.clone();

    let mut e = vec![vec![0.; img.width() as usize]; img.height() as usize];
    for (x, y, Luma([value])) in energy.enumerate_pixels() {
        e[y as usize][x as usize] = (*value) as f32;
    }

    if num_vertical_seams_to_remove > 0 {
        let width = output_img.width() as usize;
        let height = output_img.height() as usize;
        let mut should_be_removed = vec![vec![false; width]; height];

        set_removed(
            &e,
            width,
            height,
            num_vertical_seams_to_remove,
            SeamDirection::Vertical,
            &mut should_be_removed,
        );

        let new_width = width - num_vertical_seams_to_remove;
        let mut carved_img = ImageBuffer::new(new_width as u32, height as u32);
        let mut carved_e = vec![vec![0.0; new_width]; height];

        for y in 0..height {
            let mut out_x = 0;
            for x in 0..width {
                if !should_be_removed[y][x] && out_x < new_width {
                    carved_img.put_pixel(out_x as u32, y as u32, output_img[(x as u32, y as u32)]);
                    carved_e[y][out_x] = e[y][x];
                    out_x += 1;
                }
            }
        }
        output_img = carved_img;
        e = carved_e;
    }

    if num_horizontal_seams_to_remove > 0 {
        let width = output_img.width() as usize;
        let height = output_img.height() as usize;
        let mut should_be_removed = vec![vec![false; width]; height];

        set_removed(
            &e,
            width,
            height,
            num_horizontal_seams_to_remove,
            SeamDirection::Horizontal,
            &mut should_be_removed,
        );

        let new_height = height - num_horizontal_seams_to_remove;
        let mut carved_img = ImageBuffer::new(width as u32, new_height as u32);

        for x in 0..width {
            let mut out_y = 0;
            for y in 0..height {
                if !should_be_removed[y][x] && out_y < new_height {
                    carved_img.put_pixel(x as u32, out_y as u32, output_img[(x as u32, y as u32)]);
                    out_y += 1;
                }
            }
        }
        output_img = carved_img;
    }

    output_img
}
