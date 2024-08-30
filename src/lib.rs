use js_sys::{ArrayBuffer, Uint8Array};
use wasm_bindgen::prelude::wasm_bindgen;
use image::ImageFormat;
use image::ImageReader;
use std::io::Cursor;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
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
        buffer.length()
    ).to_vec();

    let img2 = ImageReader::new(Cursor::new(bytes)).with_guessed_format().unwrap().decode().unwrap();

    let mut new_vec: Vec<u8> = Vec::new();
    img2.write_to(&mut Cursor::new(&mut new_vec), ImageFormat::Png).unwrap();

    new_vec
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() -> Result<(), Box<dyn std::error::Error>> {
        let img = ImageReader::open("Audrey Hepburn.jpg")?.decode()?;
        let w = img.width();
        let h = img.height();
        let ct = img.color();
        // img.save_with_format("test.png", ImageFormat::Png)?;

        let file_bytes = std::fs::read("Audrey Hepburn.jpg").unwrap();
        let img2 = ImageReader::new(Cursor::new(file_bytes)).with_guessed_format()?.decode()?;

        let mut bytes: Vec<u8> = Vec::new();
        img2.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)?;

        //let output = convert_image_not_wasm(&bytes);
        std::fs::write("test.png", bytes)?;
        //image::save_buffer("test.png", &bytes, w, h, ct)?;
        Ok(())
    }
}
