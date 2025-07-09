use crate::{Error, IconBase};
use std::fmt::Debug;

#[derive(Clone)]
pub struct Icon {
    buffer: Option<&'static [u8]>,
    pub(crate) sys: crate::IconSys,
}

impl Debug for Icon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Icon")
    }
}

impl Icon {
    pub fn from_buffer(
        buffer: &'static [u8],
        width: Option<u32>,
        height: Option<u32>,
    ) -> Result<Icon, Error> {
        Ok(Icon {
            buffer: Some(buffer),
            sys: crate::IconSys::from_buffer(buffer, width, height)?,
        })
    }

    // Sets the icon template mode.
    // This is only applicable on macOS.
    // It allows the icon to be rendered as a template image,
    // which is useful for monochrome icons in the menu bar.
    #[cfg(target_os = "macos")]
    pub fn set_template(&mut self, _template: bool) {
        self.sys.set_template(_template);
    }
}

impl PartialEq for Icon {
    fn eq(&self, other: &Self) -> bool {
        self.buffer == other.buffer
    }
}
