use std::io::Cursor;

use cair::{compute_gradient_magnitude, remove_seams};
use data_encoding::BASE64;
use image::{DynamicImage, ImageFormat, RgbImage};
use leptos::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App)
}

#[component]
fn App() -> impl IntoView {
    let slider_str = RwSignal::new("0".to_string());
    let slider_value = move || slider_str.get().parse::<u16>().unwrap();

    let img = image::load_from_memory(include_bytes!("/home/sermuns/Git/cv/media/jag.jpg"))
        .unwrap()
        .into_rgb8();
    let (width, height) = img.dimensions();

    let mut grad_magnitude = RgbImage::new(width, height);
    compute_gradient_magnitude(&img, &mut grad_magnitude);

    let energy = DynamicImage::ImageRgb8(grad_magnitude).into_luma8();

    let carved_image_data_url = move || {
        let num_vertical_seams_to_remove = slider_value() as usize;
        let num_horizontal_seams_to_remove = 0; // TODO:

        let carved = remove_seams(
            &img,
            &energy,
            num_horizontal_seams_to_remove,
            num_vertical_seams_to_remove,
        );
        let mut bytes = Vec::new();

        carved
            .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
            .unwrap();

        format!("data:image/png;base64,{}", BASE64.encode_display(&bytes))
    };

    view! {
        <input
            type="range"
            bind:value=slider_str
            min=0
            max=width-1
        />
        <p>
            {slider_value}
        </p>

        <img src=carved_image_data_url />
    }
}
