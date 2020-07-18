use crate::{Error, IconBase};
use winapi::shared::minwindef::PBYTE;
use winapi::shared::windef::HICON;
use winapi::um::winuser;

/// Purpose of this struct is to keep hicon handle, and drop it when the struct
/// is dropped
pub struct WinHIcon {
    pub hicon: HICON,
}

impl IconBase for WinHIcon {
    fn from_buffer(
        buffer: &'static [u8],
        width: Option<u32>,
        height: Option<u32>,
    ) -> Result<WinHIcon, Error> {
        let offset = unsafe {
            winuser::LookupIconIdFromDirectoryEx(
                buffer.as_ptr() as PBYTE,
                1,
                width.unwrap_or_default() as i32,
                height.unwrap_or_default() as i32,
                winuser::LR_DEFAULTCOLOR,
            )
        };
        if offset <= 0 {
            return Err(Error::IconLoadingFailed);
        }
        let icon_data = &buffer[offset as usize..];
        let hicon = unsafe {
            winuser::CreateIconFromResourceEx(
                icon_data.as_ptr() as PBYTE,
                icon_data.len() as u32,
                1,
                0x30000,
                width.unwrap_or_default() as i32,
                height.unwrap_or_default() as i32,
                winuser::LR_DEFAULTCOLOR,
            )
        };
        if hicon.is_null() {
            return Err(Error::IconLoadingFailed);
        }
        Ok(WinHIcon { hicon })
    }
}

impl Clone for WinHIcon {
    fn clone(&self) -> Self {
        WinHIcon {
            hicon: unsafe { winuser::CopyIcon(self.hicon) },
        }
    }
}

unsafe impl Send for WinHIcon {}
unsafe impl Sync for WinHIcon {}

impl Drop for WinHIcon {
    fn drop(&mut self) {
        unsafe { winuser::DestroyIcon(self.hicon) };
    }
}
