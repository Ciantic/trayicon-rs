use crate::{Error, IconBase};
use objc2::rc::Retained;
use objc2::{msg_send, AnyThread};
use objc2_app_kit::NSImage;
use objc2_foundation::{NSData, NSSize};

#[derive(Clone)]
pub struct MacIcon {
    pub(crate) ns_image: Retained<NSImage>,
}

impl IconBase for MacIcon {
    fn from_buffer(
        buffer: &'static [u8],
        width: Option<u32>,
        height: Option<u32>,
    ) -> Result<MacIcon, Error> {
        let ns_data = NSData::with_bytes(buffer);
        let ns_image = NSImage::initWithData(NSImage::alloc(), &ns_data).ok_or(Error::OsError)?;

        // Set size if provided
        if let (Some(w), Some(h)) = (width, height) {
            let size = NSSize::new(w as f64, h as f64);
            unsafe {
                let _: () = msg_send![&ns_image, setSize: size];
            }
        }

        Ok(MacIcon { ns_image })
    }
}

// Drop is handled automatically by Id<NSImage>

unsafe impl Send for MacIcon {}
unsafe impl Sync for MacIcon {}