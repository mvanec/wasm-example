mod error;
mod text;

use image::codecs::png::CompressionType;
use image::codecs::png::FilterType;
use image::codecs::png::PngEncoder;
use image::ImageReader;
use js_sys::{ArrayBuffer, Uint8Array};
use std::io::Cursor;
use wasm_bindgen::prelude::wasm_bindgen;

#[allow(unused)]
use crate::error::ConversionError;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Next let's define a macro that's like `println!`, only it works for
// `console.log`. Note that `println!` doesn't actually work on the Wasm target
// because the standard library currently just eats all output. To get
// `println!`-like behavior in your app you'll likely want a macro like this.
#[macro_export]
macro_rules! console_log {
    // Match against any number of arguments of any type
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// This defines the Node.js Buffer type
#[wasm_bindgen]
extern "C" {
    pub type Buffer;

    #[wasm_bindgen(method, getter)]
    fn buffer(this: &Buffer) -> ArrayBuffer;

    #[wasm_bindgen(method, getter, js_name = byteOffset)]
    fn byte_offset(this: &Buffer) -> u32;

    #[wasm_bindgen(method, getter)]
    fn length(this: &Buffer) -> u32;
}

#[wasm_bindgen]
pub fn convert_image(buffer: &Buffer) -> Vec<u8> {
    // This converts from a Node.js Buffer into a Vec<u8>
    let bytes: Vec<u8> = Uint8Array::new_with_byte_offset_and_length(
        &buffer.buffer(),
        buffer.byte_offset(),
        buffer.length(),
    )
    .to_vec();

    let img = ImageReader::new(Cursor::new(&bytes))
        .with_guessed_format()
        .expect("Error guessing image format")
        .decode()
        .expect("Error decoding image");

    console_log!("Incoming File Buffer length: {}", buffer.length());
    console_log!("Buffer to u8 Bytes length: {}", bytes.len());

    let mut new_vec: Vec<u8> = Vec::new();
    let encoder = PngEncoder::new_with_quality(&mut new_vec, CompressionType::Best, FilterType::Adaptive);
    img.write_with_encoder(encoder).expect("Error encoding and writing PNG Buffer");

    console_log!("PNG data size: {}", new_vec.len());

    new_vec
}

#[cfg(test)]
mod tests {
    use super::*;
    use ab_glyph::{PxScale, FontRef};
    use image::{DynamicImage, ImageBuffer, Luma, Rgba};
    use image::imageops::overlay;
    use imageproc::distance_transform::Norm;
    use imageproc::drawing::draw_text_mut;
    use imageproc::morphology::{dilate, dilate_mut};

    fn overlay_text(text: &str, img_data: &[u8]) -> Result<Vec<u8>, ConversionError> {
        // Load the image bytes into a DynamicImage and give it an alpha channel
        let sb_img: DynamicImage = image::load_from_memory(img_data)?;
        let mut image = sb_img.to_rgba8();

        // Load the font and give it a scale
        let font_bytes: &[u8] = include_bytes!("../GothamBook.ttf") as &[u8];
        let font = FontRef::try_from_slice(font_bytes)?;
        let scale = PxScale {
            x: image.width() as f32 * 0.10,
            y: image.height() as f32 * 0.10,
        };

        // The width and height are needed for drawing on the canvas
        let width_scaled = (image.width() as f32 * 0.10) as i32;
        let height_scaled = (image.height() as f32 * 0.10) as i32;

        // Draw the text onto the transparent buffer
        let mut overlay_alpha: DynamicImage = DynamicImage::new_rgba8(image.width(), image.height() as u32);
        draw_text_mut(
            &mut overlay_alpha,
            Rgba([255u8, 0u8, 0u8, 255u8]),
            width_scaled,
            height_scaled,
            scale,
            &font,
            text,
        );

        // Get a grayscalee buffer and expand the letters by 4px
        let mut overlay_buf = overlay_alpha.to_luma8();
        dilate_mut(&mut overlay_buf, Norm::LInf, 4u8);

        // Transfer the expanded letter pixels to the transparent buffer
        let mut overlay_alpha_buf = overlay_alpha.to_rgba8();
        for x in 0..overlay_buf.width() {
            for y in 0..overlay_buf.height() {
                let pixval = 255 - overlay_buf.get_pixel(x, y).0[0];
                if pixval != 255 {
                    let new_pix = Rgba([pixval, pixval, pixval, 255]);
                    overlay_alpha_buf.put_pixel(x, y, new_pix);
                }
            }
        }

        // Draw the text in color on top of the black letters, giving
        // colored letters with a black outline.
        draw_text_mut(
            &mut overlay_alpha_buf,
            Rgba([255u8, 128u8, 0u8, 128u8]),
            width_scaled,
            height_scaled,
            scale,
            &font,
            text,
        );

        // Overlay the transparent layer over the image layer
        overlay(&mut image, &overlay_alpha_buf, 0, 0);

        // Write the image buffer to a Vec<uu8> and return it to the caller.
        let mut bytes: Vec<u8> = Vec::new();
        image.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)?;
        Ok(bytes)
    }

    fn overlay_text_gpt(text: &str, img_data: &[u8]) -> Result<Vec<u8>, ConversionError> {
        let mut image = image::load_from_memory(img_data)?.to_rgba8();
        let font_bytes = include_bytes!("../GothamBook.ttf");
        let font = FontRef::try_from_slice(font_bytes)?;

        let scale = PxScale {
            x: image.width() as f32 * 0.1,
            y: image.height() as f32 * 0.1,
        };

        // let scale: PxScale = PxScale::from(128.0);
        let position = (
            (image.width() as f32 * 0.10) as i32,
            (image.height() as f32 * 0.10) as i32,
        );

        // Create an alpha mask for the text
        let mut mask:ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::from_pixel(image.width(), image.height(), Luma([0u8]));
        draw_text_mut(
            &mut mask,
            Luma([255u8]),
            position.0,
            position.1,
            scale,
            &font,
            text,
        );

        // Dilate the mask to create an outline effect
        let dilated_mask = dilate(&mask, Norm::LInf, 4u8);

        // Convert dilated mask into an RGBA image for overlaying
        let outline_image = ImageBuffer::from_fn(image.width(), image.height(), |x, y| {
            let pixel = dilated_mask[(x, y)];
            if pixel[0] > 0 && pixel[0] < 255 { // Assuming dilation creates a gradient
                Rgba([0, 0, 0, 255]) // Black outline
            } else {
                Rgba([0, 0, 0, 0]) // Fully transparent
            }
        });

        // Overlay the text directly in color
        draw_text_mut(
            &mut image,
            Rgba([255, 128, 0, 255]), // Semi-transparent orange text
            position.0,
            position.1,
            scale,
            &font,
            text,
        );

        // Overlay the outline
        image::imageops::overlay(&mut image, &outline_image, 0, 0);

        let mut bytes = Vec::new();
        image.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)?;
        Ok(bytes)
    }

    #[allow(unused)]
    #[test]
    fn it_works() -> Result<(), Box<dyn std::error::Error>> {
        let img = ImageReader::open("Audrey Hepburn.jpg")?.decode()?;
        let w = img.width();
        let h = img.height();
        let ct = img.color();
        // img.save_with_format("test.png", ImageFormat::Png)?;

        let file_bytes = std::fs::read("Audrey Hepburn.jpg").unwrap();
        let mut image = ImageReader::new(Cursor::new(file_bytes))
            .with_guessed_format()?
            .decode()?;

        // Convert to PNG
        let mut bytes: Vec<u8> = Vec::new();
        image.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)?;

        let image_bytes_to_write = overlay_text("WASM Test", &bytes)?;
        std::fs::write("test.png", image_bytes_to_write)?;

        let image_bytes_to_write = overlay_text_gpt("WASM Test", &bytes)?;
        std::fs::write("test_gpt.png", image_bytes_to_write)?;

        Ok(())
    }
}
