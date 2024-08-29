use std::io::Cursor;

use bytes::{Bytes, BytesMut};
use image::{codecs::avif::AvifEncoder, DynamicImage, ImageEncoder, ImageFormat};
use tracing::{info, warn, error};

pub enum ImageError {
    DecodeFailed,
    UnsupportedFormat,
    InternalServerError,
}

pub trait FromImageError {
    fn from_image_error(image_error: ImageError) -> Self;
}

pub async fn convert_to_avif<T: FromImageError>
(image_bytes: &Bytes, mime_type: &String) -> Result<(), T> {

    let image_format = match mime_type.as_str() {
        "image/png" => ImageFormat::Png,
        "image/jpeg" => ImageFormat::Jpeg,
        "image/webp" => ImageFormat::WebP,
        _ => return Err(T::from_image_error(ImageError::UnsupportedFormat))
    };

    let image_bytes = image_bytes.to_vec();
    let image = image::load_from_memory_with_format(image_bytes.as_ref(), image_format);

    let image = match image {
        Ok(image) => image,
        Err(err) => {
            warn!("Image decode failed: {}", err);
            return Err(T::from_image_error(ImageError::DecodeFailed))
        }
    };


    info!("Start to Write");
    let mut avif: Vec<u8> = Vec::with_capacity(5 * 1024 * 1024);

    if let Err(err) = write_avif(&mut Cursor::new(&mut avif), &image).await {
        error!("{}", err);
    }

    info!("End to Write");

    info!("{}, {}", image.width(), image.height());

    Ok(())
}

async fn write_avif(cursor: &mut Cursor::<&mut Vec<u8>>, image: &DynamicImage)
-> Result<(), image::ImageError> {
    image.write_to(cursor, ImageFormat::Avif)
}
