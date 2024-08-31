use image::codecs::png::CompressionType;
use image::codecs::png::FilterType;
use image::codecs::png::PngEncoder;
use image::ImageFormat;
use image::ImageReader;
use js_sys::{ArrayBuffer, Uint8Array};
use std::io::Cursor;
use wasm_bindgen::prelude::wasm_bindgen;

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

    #[allow(unused)]
    #[test]
    fn it_works() -> Result<(), Box<dyn std::error::Error>> {
        let img = ImageReader::open("Audrey Hepburn.jpg")?.decode()?;
        let w = img.width();
        let h = img.height();
        let ct = img.color();
        // img.save_with_format("test.png", ImageFormat::Png)?;

        let file_bytes = std::fs::read("Audrey Hepburn.jpg").unwrap();
        let img2 = ImageReader::new(Cursor::new(file_bytes))
            .with_guessed_format()?
            .decode()?;

        let mut bytes: Vec<u8> = Vec::new();
        img2.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)?;

        //let output = convert_image_not_wasm(&bytes);
        std::fs::write("test.png", bytes)?;
        //image::save_buffer("test.png", &bytes, w, h, ct)?;
        Ok(())
    }
}
