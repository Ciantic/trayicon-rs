use std::fmt::Debug;
use winapi::shared::windef::{HMENU, HWND};
use winapi::um::winuser;

/// Purpose of this struct is to keep hmenu handle, and drop it when the struct
/// is dropped
#[derive(Debug)]
pub struct WinHMenu {
    pub hmenu: HMENU,
    child_menus: Vec<WinHMenu>,
}

impl WinHMenu {
    pub(crate) fn new() -> WinHMenu {
        WinHMenu {
            hmenu: unsafe { winuser::CreatePopupMenu() },
            child_menus: vec![],
        }
    }

    pub fn add_menu_item(&self, name: &str, id: usize, disabled: bool) {
        let _res = unsafe {
            winuser::AppendMenuW(
                self.hmenu,
                {
                    if disabled {
                        winuser::MF_GRAYED
                    } else {
                        winuser::MF_STRING
                    }
                },
                id,
                format!("{}\0", name)
                    .encode_utf16()
                    .collect::<Vec<_>>()
                    .as_ptr() as _,
            )
        };
    }

    pub fn add_checkable_item(&self, name: &str, is_checked: bool, id: usize, disabled: bool) {
        let mut flags = if is_checked {
            winuser::MF_CHECKED
        } else {
            winuser::MF_UNCHECKED
        };

        if disabled {
            flags |= winuser::MF_GRAYED
        }
        let _res = unsafe {
            winuser::AppendMenuW(
                self.hmenu,
                flags,
                id,
                format!("{}\0", name)
                    .encode_utf16()
                    .collect::<Vec<_>>()
                    .as_ptr() as _,
            )
        };
    }
    pub fn add_child_menu(&mut self, name: &str, menu: WinHMenu) {
        let _res = unsafe {
            winuser::AppendMenuW(
                self.hmenu,
                winuser::MF_POPUP,
                menu.hmenu as _,
                format!("{}\0", name)
                    .encode_utf16()
                    .collect::<Vec<_>>()
                    .as_ptr() as _,
            )
        };
        self.child_menus.push(menu);
    }

    pub fn add_separator(&self) {
        let _res =
            unsafe { winuser::AppendMenuW(self.hmenu, winuser::MF_SEPARATOR, 0 as _, 0 as _) };
    }

    pub fn track(&self, hwnd: HWND, x: i32, y: i32) {
        unsafe { winuser::TrackPopupMenu(self.hmenu, 0, x, y, 0, hwnd, std::ptr::null_mut()) };
    }
}

unsafe impl Send for WinHMenu {}
unsafe impl Sync for WinHMenu {}

impl Drop for WinHMenu {
    fn drop(&mut self) {
        unsafe { winuser::DestroyMenu(self.hmenu) };
    }
}