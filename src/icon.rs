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
}

impl PartialEq for Icon {
    fn eq(&self, other: &Self) -> bool {
        self.buffer == other.buffer
    }
}
