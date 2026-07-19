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

    /// Add a mutually-exclusive radio item.
    ///
    /// Uses `MFT_RADIOCHECK` so the item is rendered with a radio-button
    /// glyph instead of a checkmark. Exclusivity within a group of consecutive
    /// radio items is enforced natively on selection via `CheckMenuRadioItem`
    /// (see `MenuSys`), so applications do not have to rebuild the menu to keep
    /// a single selection per group.
    pub fn add_radio_item(&self, name: &str, is_checked: bool, id: usize, disabled: bool) -> bool {
        let label = wchar(name);

        // MFS_ENABLED / MFT_STRING are zero, so we only set the flags that
        // actually carry a bit and rely on the zero-initialized rest for the
        // defaults (enabled, string item).
        let mut state = if is_checked {
            winuser::MFS_CHECKED
        } else {
            winuser::MFS_UNCHECKED
        };
        if disabled {
            state |= winuser::MFS_GRAYED;
        }

        let mut info: winuser::MENUITEMINFOW = unsafe { std::mem::zeroed() };
        info.cbSize = std::mem::size_of::<winuser::MENUITEMINFOW>() as u32;
        info.fMask =
            winuser::MIIM_FTYPE | winuser::MIIM_STATE | winuser::MIIM_ID | winuser::MIIM_STRING;
        info.fType = winuser::MFT_RADIOCHECK;
        info.fState = state;
        info.wID = id as u32;
        info.dwTypeData = label.as_ptr() as *mut u16;
        info.cch = label.len() as u32 - 1; // exclude the null terminator

        // Append at the end: position is the current item count, which we
        // read with MF_BYPOSITION semantics.
        let res = unsafe {
            let count = winuser::GetMenuItemCount(self.hmenu);
            winuser::InsertMenuItemW(
                self.hmenu,
                count as u32,
                1, // TRUE -> by position
                &info,
            )
        };
        res != 0
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

    /// The underlying `HMENU` handle, for use with menu APIs such as
    /// `CheckMenuRadioItem` that need to operate on a specific (sub)menu.
    pub(crate) fn handle(&self) -> HMENU {
        self.hmenu
    }
}

unsafe impl Send for WinHMenu {}
unsafe impl Sync for WinHMenu {}

impl Drop for WinHMenu {
    fn drop(&mut self) {
        unsafe { winuser::DestroyMenu(self.hmenu) };
    }
}
