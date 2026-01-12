use crate::{Error, IconBase};

/// Purpose of this struct is to keep hicon handle, and drop it when the struct
/// is dropped
#[derive(Debug, Clone)]
pub struct KdeIcon {
    pub width: u32,
    pub height: u32,
    pub argb_pixels: Option<Vec<u8>>,
}

impl IconBase for KdeIcon {
    fn from_buffer(
        buffer: &'static [u8],
        _width: Option<u32>,
        _height: Option<u32>,
    ) -> Result<KdeIcon, Error> {
        // Try to decode the ICO file to get the largest icon
        let (decoded_width, decoded_height, argb_pixels) =
            match ico::IconDir::read(std::io::Cursor::new(buffer)) {
                Ok(icon_dir) => {
                    // Get the largest icon entry
                    if let Some(entry) = icon_dir
                        .entries()
                        .iter()
                        .max_by_key(|e| e.width() as u32 * e.height() as u32)
                    {
                        match entry.decode() {
                            Ok(image) => {
                                let w = image.width();
                                let h = image.height();
                                let rgba = image.rgba_data();

                                // Convert RGBA to ARGB
                                let mut argb_pixmap = Vec::with_capacity(rgba.len());
                                for chunk in rgba.chunks(4) {
                                    if chunk.len() == 4 {
                                        argb_pixmap.push(chunk[3]); // Alpha
                                        argb_pixmap.push(chunk[0]); // Red
                                        argb_pixmap.push(chunk[1]); // Green
                                        argb_pixmap.push(chunk[2]); // Blue
                                    }
                                }

                                (w, h, Some(argb_pixmap))
                            }
                            Err(e) => {
                                eprintln!("Failed to decode icon entry: {:?}", e);
                                (0, 0, None)
                            }
                        }
                    } else {
                        eprintln!("No icon entries found");
                        (0, 0, None)
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read ICO file: {:?}", e);
                    (0, 0, None)
                }
            };

        Ok(KdeIcon {
            width: decoded_width,
            height: decoded_height,
            argb_pixels,
        })
    }
}

unsafe impl Send for KdeIcon {}
unsafe impl Sync for KdeIcon {}
