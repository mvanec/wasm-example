use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ConversionError {
    Image(image::ImageError),
    Font(ab_glyph::InvalidFont),
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ConversionError::Image(ref err) => err.fmt(f),
            ConversionError::Font(ref err) => err.fmt(f),
        }
    }
}

impl Error for ConversionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            ConversionError::Image(ref err) => Some(err),
            ConversionError::Font(ref err) => Some(err),
        }
    }
}

impl From<image::ImageError> for ConversionError {
    fn from(err: image::ImageError) -> ConversionError {
        ConversionError::Image(err)
    }
}

impl From<ab_glyph::InvalidFont> for ConversionError {
    fn from(err: ab_glyph::InvalidFont) -> ConversionError {
        ConversionError::Font(err)
    }
}
