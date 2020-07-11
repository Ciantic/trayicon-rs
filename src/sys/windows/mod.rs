use winapi::shared::windef::HWND;
mod hicon;
mod hmenu;
mod notifyicon;
mod window;

use std::{collections::HashMap, pin::Pin};
use window::TrayIconWindow;

use crate::{Error, MenuBuilder, MenuItem, TrayIconBuilder};
use hicon::WinHIcon;
use hmenu::WinHMenu;
use notifyicon::NotifyIcon;

pub type TrayIconSys<T>
where
    T: PartialEq + Clone,
= Box<TrayIconWindow<T>>;

pub type MenuSys<T>
where
    T: PartialEq + Clone,
= (HashMap<usize, T>, WinHMenu);

pub fn build_icon<T>(builder: TrayIconBuilder<T>) -> Result<TrayIconSys<T>, Error>
where
    T: PartialEq + Clone,
{
    let mut hicon: Option<WinHIcon> = None;
    let mut menu: Option<WinHMenu> = None;
    let parent_hwnd: Option<HWND> = builder.parent_hwnd.map(|h| h as HWND);
    let on_click = builder.on_click;
    let on_right_click = builder.on_right_click;
    let sender = builder.sender;
    let mut menu_events = None;

    let on_double_click = builder.on_double_click;

    // Try to get icon from byte buffer
    if let Some(icon_buffer) = builder.icon_buffer {
        hicon = Some(
            WinHIcon::new_from_buffer(icon_buffer, builder.width, builder.height)
                .ok_or(Error::IconLoadingFailed)?,
        );
    }

    // Fall back to not having an icon
    if hicon.is_none() {
        hicon = Some(WinHIcon::new());
    }

    // Try to get a popup menu
    if let Some(rhmenu) = builder.menu {
        if let Ok((hash, hmenu)) = rhmenu {
            menu = Some(hmenu);
            menu_events = Some(hash);
        }
    }

    if let Some(hicon) = hicon {
        let n = NotifyIcon::new(hicon);
        Ok(TrayIconWindow::new(
            sender,
            n,
            menu,
            parent_hwnd,
            on_click,
            on_double_click,
            on_right_click,
            menu_events,
        )?)
    } else {
        Err(Error::IconLoadingFailed)
    }
}

pub fn build_menu<T>(mut builder: MenuBuilder<T>) -> Result<MenuSys<T>, Error>
where
    T: PartialEq + Clone,
{
    let hmenu = WinHMenu::new();
    let mut map: HashMap<usize, T> = HashMap::new();
    builder
        .menu_items
        .drain(0..)
        .enumerate()
        .for_each(|(i, item)| match item {
            MenuItem::Item(name, event) => {
                map.insert(i, event);
                hmenu.add_menu_item(&name, i);
            }
            MenuItem::Separator => hmenu.add_separator(),
            _ => {}
        });
    Ok((map, hmenu))
}

// For pattern matching, these are in own mod
mod msgs {
    pub const WM_USER_CREATE: u32 = 0x400 + 1000;
    pub const WM_USER_TRAYICON: u32 = 0x400 + 1001;
}

#[cfg(test)]
pub(crate) mod tests {
    use core::mem::MaybeUninit;
    use winapi::um::winuser;

    pub fn main_loop() {
        loop {
            unsafe {
                let mut msg = MaybeUninit::uninit();
                let bret = winuser::GetMessageA(msg.as_mut_ptr(), 0 as _, 0, 0);
                if bret > 0 {
                    winuser::TranslateMessage(msg.as_ptr());
                    winuser::DispatchMessageA(msg.as_ptr());
                } else {
                    break;
                }
            }
        }
    }
}
