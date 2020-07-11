use std::fmt::Debug;
use winapi::shared::windef::{HMENU, HWND};
use winapi::um::winuser;

/// Purpose of this struct is to keep hmenu handle, and drop it when the struct
/// is dropped
#[derive(Debug)]
pub struct WinHMenu {
    pub hmenu: HMENU,
}

impl WinHMenu {
    pub(crate) fn new() -> WinHMenu {
        WinHMenu {
            hmenu: unsafe { winuser::CreatePopupMenu() },
        }
    }

    pub fn add_menu_item(&self, name: &str, id: usize) {
        let _res = unsafe {
            winuser::AppendMenuW(
                self.hmenu,
                winuser::MF_STRING,
                id,
                format!("{}\0", name)
                    .encode_utf16()
                    .collect::<Vec<_>>()
                    .as_ptr() as _,
            )
        };
    }

    pub fn add_separator(&self) {
        let _res =
            unsafe { winuser::AppendMenuW(self.hmenu, winuser::MF_SEPARATOR, 0 as _, 0 as _) };
    }

    pub fn track(&self, hwnd: HWND, x: i32, y: i32) {
        unsafe { winuser::TrackPopupMenu(self.hmenu, 0, x, y, 0, hwnd, std::ptr::null_mut()) };
    }
}

impl Drop for WinHMenu {
    fn drop(&mut self) {
        unsafe { winuser::DestroyMenu(self.hmenu) };
    }
}
