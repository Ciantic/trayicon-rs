use crate::{Error, IconBase};

/// Purpose of this struct is to keep hicon handle, and drop it when the struct
/// is dropped
pub struct KdeIcon {
    pub buffer: &'static [u8],
    pub width: u32,
    pub height: u32,
    pub rgba_pixels: Option<Vec<u8>>,
}

impl IconBase for KdeIcon {
    fn from_buffer(
        buffer: &'static [u8],
        width: Option<u32>,
        height: Option<u32>,
    ) -> Result<KdeIcon, Error> {
        // Try to decode the image to get actual pixel data
        let (decoded_width, decoded_height, rgba_pixels) = match image::load_from_memory(buffer) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let w = rgba.width();
                let h = rgba.height();
                (w, h, Some(rgba.into_raw()))
            }
            Err(e) => {
                eprintln!("Failed to decode icon: {:?}", e);
                (width.unwrap_or(0), height.unwrap_or(0), None)
            }
        };

        Ok(KdeIcon {
            buffer,
            width: width.unwrap_or(decoded_width),
            height: height.unwrap_or(decoded_height),
            rgba_pixels,
        })
    }
}

impl Clone for KdeIcon {
    fn clone(&self) -> Self {
        KdeIcon {
            buffer: self.buffer,
            width: self.width,
            height: self.height,
            rgba_pixels: self.rgba_pixels.clone(),
        }
    }
}

unsafe impl Send for KdeIcon {}
unsafe impl Sync for KdeIcon {}
