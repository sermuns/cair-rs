use std::io::Cursor;

use cair::{compute_gradient_magnitude, remove_seams};
use data_encoding::BASE64;
use image::{DynamicImage, ImageFormat, RgbImage};
use leptos::{html::Input, logging::log, prelude::*};
use web_sys::js_sys::Uint8Array;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App)
}

#[component]
fn App() -> impl IntoView {
    let num_vertical_seams_str = RwSignal::new("0".to_string());
    let num_horizontal_seams_str = RwSignal::new("0".to_string());

    let file_input = NodeRef::<Input>::new();
    let img_file = RwSignal::new(None);

    let on_submit_file = move |_| {
        let Some(inp) = file_input.get() else {
            return;
        };

        let Some(files) = inp.files() else {
            return;
        };

        let Some(file) = files.get(0) else {
            return;
        };
        log!("submitted file");

        img_file.set(Some(file));
        num_vertical_seams_str.set("0".to_string());
        num_horizontal_seams_str.set("0".to_string());
    };

    let img_bytes = LocalResource::new(move || async move {
        let file = img_file.get()?;
        log!("got file");

        let bytes_js_value = file.bytes().await.ok()?;
        log!("got bytes js value");

        let bytes_array = Uint8Array::new(&bytes_js_value);
        log!("got Uint8Array");

        let bytes_vec = bytes_array.to_vec();
        log!("got vec");

        Some(bytes_vec)
    });

    let img = move || {
        let img = image::load_from_memory(&*img_bytes.get()??)
            .ok()
            .map(|img| img.into_rgb8());
        log!("loaded image!");
        img
    };

    let image_dimensions = move || {
        let img = img()?;
        Some(img.dimensions())
    };

    let carved_image_data_url = move || {
        let img = img()?;

        let num_vertical_seams_to_remove = num_vertical_seams_str.get().parse::<usize>().ok()?;

        let num_horizontal_seams_to_remove =
            num_horizontal_seams_str.get().parse::<usize>().ok()?;

        let (width, height) = img.dimensions();

        let mut grad_magnitude = RgbImage::new(width, height);
        compute_gradient_magnitude(&img, &mut grad_magnitude);

        let energy = DynamicImage::ImageRgb8(grad_magnitude).into_luma8();

        let carved = remove_seams(
            &img,
            &energy,
            num_horizontal_seams_to_remove,
            num_vertical_seams_to_remove,
        );

        let mut bytes = Vec::new();

        carved
            .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
            .ok()?;

        Some(format!(
            "data:image/png;base64,{}",
            BASE64.encode_display(&bytes)
        ))
    };

    view! {
        <main>
            <h1>{env!("CARGO_PKG_NAME")}</h1>

            <input
                type="file"
                accept="image/*"
                node_ref=file_input
                on:change=on_submit_file
            />

            <Show
                when=move || image_dimensions().is_some()
                fallback=|| view! {
                    <p>"Upload an image to begin."</p>
                }
            >
                {move || {
                    let Some((width, height)) = image_dimensions() else {
                        return ().into_any();
                    };

                    view! {
                        <div id="grid">
                            "vertical seams to remove: "
                            {num_vertical_seams_str}

                            <input
                                type="range"
                                min=0
                                max=width - 1
                                bind:value=num_vertical_seams_str
                            />

                            "horizontal seams to remove: "
                            {num_horizontal_seams_str}

                            <input
                                type="range"
                                min=0
                                max=height - 1
                                bind:value=num_horizontal_seams_str
                            />
                        </div>

                        <img
                            src=carved_image_data_url
                        />
                    }
                    .into_any()
                }}
            </Show>
        </main>

        <footer>
            <a href="https://github.com/sermuns/cair-rs/tree/main/examples/seam-removal-web">
                "source code"
            </a>
        </footer>
    }
}
