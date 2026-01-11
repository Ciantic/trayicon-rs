use crate::{Error, IconBase};

/// Purpose of this struct is to keep hicon handle, and drop it when the struct
/// is dropped
pub struct KdeIcon {
    pub buffer: &'static [u8],
    pub width: u32,
    pub height: u32,
}

impl IconBase for KdeIcon {
    fn from_buffer(
        buffer: &'static [u8],
        width: Option<u32>,
        height: Option<u32>,
    ) -> Result<KdeIcon, Error> {
        Ok(KdeIcon {
            buffer,
            width: width.unwrap_or_default(),
            height: height.unwrap_or_default(),
        })
    }
}

impl Clone for KdeIcon {
    fn clone(&self) -> Self {
        KdeIcon {
            buffer: self.buffer,
            width: self.width,
            height: self.height,
        }
    }
}

unsafe impl Send for KdeIcon {}
unsafe impl Sync for KdeIcon {}
