use ::ab_glyph::{point, Font, FontRef, Glyph, PxScale, ScaleFont};

use image::imageops::unsharpen;
use image::{DynamicImage, ImageBuffer, Luma, Rgba};
use imageproc::contrast::stretch_contrast;
use imageproc::map::map_subpixels;
use imageproc::{
    contrast::stretch_contrast_mut,
    distance_transform::Norm,
    edges::canny,
    filter::{gaussian_blur_f32, sharpen_gaussian},
    morphology::erode_mut,
};

fn try_sobel(
    buffer: &mut ImageBuffer<Luma<u8>, Vec<u8>>,
) -> Result<ImageBuffer<Luma<u8>, Vec<u8>>, Box<dyn std::error::Error>> {
    erode_mut(buffer, Norm::L1, 3u8);

    let overlay_buf = gaussian_blur_f32(buffer, 0.90);

    let sigma = 0.5;
    let amount = 0.5;
    let mut sharpened = sharpen_gaussian(&overlay_buf, sigma, amount);

    // Stretch the contrast of the image to enhance visibility
    stretch_contrast_mut(&mut sharpened, 0, 255, 0, 255);

    Ok(sharpened)
}

pub fn create_text_image() -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Box<dyn std::error::Error>> {
    let font_data = include_bytes!("../GothamBook.ttf"); // Ensure the path to your font file is correct.
    let font = FontRef::try_from_slice(font_data)?;
    let transparent = Rgba([255, 255, 255, 0]);
    let white = Rgba([255, 255, 255, 255]);

    let mut background = ImageBuffer::from_pixel(800, 600, white);
    let mut outline_image = ImageBuffer::from_pixel(800, 600, transparent);
    let mut front_image = ImageBuffer::from_pixel(800, 600, transparent);

    let scale = PxScale { x: 96.0, y: 96.0 };
    let mut position = point(50.0, 100.0);

    let text = "Hello Rust!";
    for character in text.chars() {
        let glyph_id = font.glyph_id(character);
        let glyph = Glyph {
            id: glyph_id,
            scale: scale,
            position: position,
        };

        if let Some(outline) = font.outline_glyph(glyph) {
            let bounds = outline.px_bounds();

            outline.draw(|x, y, v| {
                let width = (x as f32 + bounds.min.x) as u32;
                let height = (y as f32 + bounds.min.y) as u32;

                if width < outline_image.width() && height < outline_image.height() {
                    let pixel = outline_image.get_pixel_mut(width, height);
                    let alpha = (v * 255.0) as u8;
                    *pixel = Rgba([0, 0, 0, alpha]);
                }
            });
        }

        // Adjust the advancement calculation to use scaled width correctly
        let advance_width = font.as_scaled(scale).h_advance(glyph_id);
        position.x += advance_width; // Advance the position for the next character
    }

    position = point(50.0, 100.0);
    for character in text.chars() {
        let glyph_id = font.glyph_id(character);
        let glyph = Glyph {
            id: glyph_id,
            scale: scale,
            position: position,
        };

        if let Some(outline) = font.outline_glyph(glyph) {
            let bounds = outline.px_bounds();

            let mut repeat = true;
            outline.draw(|x, y, v| {
                let width = (x as f32 + bounds.min.x) as u32;
                let height = (y as f32 + bounds.min.y) as u32;
                if repeat {
                    repeat = false;
                }

                if width < front_image.width() && height < front_image.height() {
                    let pixel = front_image.get_pixel_mut(width, height);
                    let alpha = (v * 255.0) as u8;
                    *pixel = Rgba([255, 255, 0, alpha]);
                }
            });
        }

        // Adjust the advancement calculation to use scaled width correctly
        let advance_width = font.as_scaled(scale).h_advance(glyph_id);
        position.x += advance_width; // Advance the position for the next character
    }

    let mut tmp_back = ImageBuffer::from_pixel(800, 600, white);
    image::imageops::overlay(&mut tmp_back, &outline_image, 0, 0);

    // Get a grayscale buffer and expand the letters by 4px
    let mut overlay_buf = DynamicImage::from(tmp_back).to_luma8();
    overlay_buf.save("test_b4_erode.png")?;
    let overlay_buf = try_sobel(&mut overlay_buf)?;
    overlay_buf.save("test_af_erode.png")?;

    // Transfer the expanded letter pixels to the transparent buffer
    let overlay_alpha_image = DynamicImage::ImageLuma8(overlay_buf).to_rgba8();
    let mut overlay_alpha_image = unsharpen(&overlay_alpha_image, 2.0, 128);
    overlay_alpha_image.save("test_made_b4_transparent.png")?;

    // Making all white pixels transparent
    for pixel in overlay_alpha_image.pixels_mut() {
        let Rgba(data) = *pixel;
        if data[0] > 128 && data[1] > 128 && data[2] > 128 {
            // If the pixel is white, make it transparent
            *pixel = Rgba([0, 0, 0, 0]);
        }
    }
    overlay_alpha_image.save("test_made_transparent.png")?;
    front_image.save("test_01_front_image.png")?;

    // Overlay the outline
    image::imageops::overlay(&mut overlay_alpha_image, &front_image, 0, 0);
    image::imageops::overlay(&mut background, &overlay_alpha_image, 0, 0);

    Ok(background)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_text_image() {
        let image_result = create_text_image();
        assert!(image_result.is_ok(), "Failed to draw text with outline");
        let image = image_result.unwrap();
        image.save("text_create_image.png").unwrap();
    }
}
