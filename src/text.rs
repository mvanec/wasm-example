use std::error::Error;

use ::ab_glyph::{point, Font, FontRef, Glyph, PxScale, ScaleFont};
use glyph_brush::{
    ab_glyph, GlyphBrushBuilder, HorizontalAlign, Layout, Section, Text, VerticalAlign,
};

use image::{DynamicImage, ImageBuffer, Rgba, RgbaImage};
use imageproc::{distance_transform::Norm, morphology::dilate_mut};

pub fn draw_text() -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Box<dyn Error>> {
    let mut image = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(300, 100);

    let font = glyph_brush::ab_glyph::FontArc::try_from_slice(include_bytes!("../GothamBook.ttf"))?;
    let mut glyph_brush = GlyphBrushBuilder::using_font(font).build();

    let scale = glyph_brush::ab_glyph::PxScale { x: 48.0, y: 48.0 };

    let section = Section {
        screen_position: (100.0, 100.0),
        bounds: (600.0, 400.0),
        text: vec![
            Text::new("Hello Rust")
                .with_scale(scale)
                .with_color([1.0, 0.0, 0.0, 1.0]), // Black text
        ],
        layout: Layout::default()
            .h_align(HorizontalAlign::Left)
            .v_align(VerticalAlign::Bottom),
        ..Section::default()
    };

    println!("Queueing section: {:?}", section);
    glyph_brush.queue(section);

    glyph_brush.process_queued(
        |rect, tex_data| {
            println!("Processing glyph at rect: {:?}", rect);
            for y in 0..rect.height() as u32 {
                for x in 0..rect.width() as u32 {
                    let index = (y * rect.width() + x) as usize;
                    let alpha = tex_data[index] as f32 / 255.0;
                    let pixel = image.get_pixel_mut(x + rect.min[0] as u32, y + rect.min[1] as u32);

                    // Alpha blending: source over destination
                    let new_color = [
                        (alpha * 255.0 + (1.0 - alpha)) as u8, // R
                        (alpha * 255.0 + (1.0 - alpha)) as u8, // G
                        (alpha * 0.0 + (1.0 - alpha)) as u8,   // B
                        (alpha * 255.0 + (1.0 - alpha)) as u8, // AA
                    ];

                    *pixel = Rgba(new_color);
                }
            }
        },
        |_| {},
    )?;

    Ok(image)
}

fn create_text_image() -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Box<dyn std::error::Error>> {
    let font_data = include_bytes!("../GothamBook.ttf"); // Ensure the path to your font file is correct.
    let font = FontRef::try_from_slice(font_data)?;
    let transparent = Rgba([255, 255, 255, 0]);
    let white = Rgba([255, 255, 255, 255]);

    let mut background = ImageBuffer::from_pixel(800, 600, white);
    let mut outline_image = ImageBuffer::from_pixel(800, 600, transparent);
    let mut front_image = ImageBuffer::from_pixel(800, 600, transparent);

    let mut scale = PxScale { x: 96.0, y: 96.0 };
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
    // Get a grayscalee buffer and expand the letters by 4px
    let mut overlay_buf = DynamicImage::from(outline_image).to_luma8();
    dilate_mut(&mut overlay_buf, Norm::LInf, 4u8);

    // Transfer the expanded letter pixels to the transparent buffer
    let mut overlay_alpha_buf = overlay_buf.to_rgba8();

    // Overlay the outline
    image::imageops::overlay(&mut background, &outline_image, 0, 0);
    image::imageops::overlay(&mut background, &front_image, 0, 0);

    Ok(background)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{GenericImageView, Pixel};

    #[test]
    fn test_draw_text_with_outline() {
        let image_result = draw_text();
        assert!(image_result.is_ok(), "Failed to draw text with outline");
        let image = image_result.unwrap();
        image.save("text_with_outline.png").unwrap();

        // Check a pixel where the text is supposed to be rendered
        // let pixel = image.get_pixel(400, 300).to_rgba();
        // assert_eq!(
        //     pixel[3], 255,
        //     "Expected fully opaque pixel where text is rendered"
        // );
    }

    #[test]
    fn test_create_text_image() {
        let image_result = create_text_image();
        assert!(image_result.is_ok(), "Failed to draw text with outline");
        let image = image_result.unwrap();
        image.save("text_create_image.png").unwrap();
    }
}
