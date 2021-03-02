use super::wchar::wchar;
use crate::Error;
use std::fmt::Debug;
use winapi::shared::windef::{HMENU, HWND};
use winapi::um::winuser;

/// Purpose of this struct is to keep hmenu handle, and drop it when the struct
/// is dropped
#[derive(Debug)]
pub struct WinHMenu {
    hmenu: HMENU,
    child_menus: Vec<WinHMenu>,
}

impl WinHMenu {
    pub(crate) fn new() -> Result<WinHMenu, Error> {
        Ok(WinHMenu {
            hmenu: unsafe {
                let res = winuser::CreatePopupMenu();
                if res.is_null() {
                    return Err(Error::OsError);
                }
                res
            },
            child_menus: vec![],
        })
    }

    pub fn add_menu_item(&self, name: &str, id: usize, disabled: bool) -> bool {
        let res = unsafe {
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
                wchar(name).as_ptr() as _,
            )
        };
        res >= 0
    }

    pub fn add_checkable_item(
        &self,
        name: &str,
        is_checked: bool,
        id: usize,
        disabled: bool,
    ) -> bool {
        let mut flags = if is_checked {
            winuser::MF_CHECKED
        } else {
            winuser::MF_UNCHECKED
        };

        if disabled {
            flags |= winuser::MF_GRAYED
        }
        let res = unsafe { winuser::AppendMenuW(self.hmenu, flags, id, wchar(name).as_ptr() as _) };
        res >= 0
    }
    pub fn add_child_menu(&mut self, name: &str, menu: WinHMenu, disabled: bool) -> bool {
        let mut flags = winuser::MF_POPUP;
        if disabled {
            flags |= winuser::MF_GRAYED
        }
        let res = unsafe {
            winuser::AppendMenuW(
                self.hmenu,
                flags,
                menu.hmenu as _,
                wchar(name).as_ptr() as _,
            )
        };
        self.child_menus.push(menu);
        res >= 0
    }

    pub fn add_separator(&self) -> bool {
        let res = unsafe { winuser::AppendMenuW(self.hmenu, winuser::MF_SEPARATOR, 0, 0 as _) };
        res >= 0
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
