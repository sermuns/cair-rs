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
    let num_vertical_seams_str = RwSignal::new("0".to_string());
    let num_horizontal_seams_str = RwSignal::new("0".to_string());

    let img = image::load_from_memory(include_bytes!("/home/sermuns/Git/cv/media/jag.jpg"))
        .unwrap()
        .into_rgb8();
    let (width, height) = img.dimensions();

    let mut grad_magnitude = RgbImage::new(width, height);
    compute_gradient_magnitude(&img, &mut grad_magnitude);

    let energy = DynamicImage::ImageRgb8(grad_magnitude).into_luma8();

    let carved_image_data_url = move || {
        let num_vertical_seams_to_remove = num_vertical_seams_str.get().parse().unwrap();
        let num_horizontal_seams_to_remove = num_horizontal_seams_str.get().parse().unwrap();

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
        <main>
        <h1>{env!("CARGO_PKG_NAME")}</h1>

        <div id="grid">
            <input type="range" bind:value=num_vertical_seams_str min=0 max=width - 1 />
            "num vertical seams to remove:" {num_vertical_seams_str}
            <input type="range" bind:value=num_horizontal_seams_str min=0 max=height - 1 />
            "num horizontal seams to remove:" {num_horizontal_seams_str}
        </div>

        <img src=carved_image_data_url />

        </main>

        <footer>
            <a href="https://github.com/sermuns/cair-rs/tree/main/examples/seam-removal-web">"source code"</a>
        </footer>
    }
}
