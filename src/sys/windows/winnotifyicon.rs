use super::{msgs, wchar::wchar_array, winhicon::WinHIcon};
use std::fmt::Debug;
use winapi::shared::windef::HWND;

/// Purpose of this struct is to retain NotifyIconDataW and remove it on drop
pub struct WinNotifyIcon {
    winhicon: WinHIcon,
    nid: winapi::um::shellapi::NOTIFYICONDATAW,
}

impl WinNotifyIcon {
    pub fn new(winhicon: &WinHIcon, tooltip: &Option<String>) -> WinNotifyIcon {
        static mut ICON_ID: u32 = 1000;
        unsafe {
            ICON_ID += 1;
        }
        let mut icon = WinNotifyIcon {
            winhicon: winhicon.clone(),
            nid: unsafe { std::mem::zeroed() },
        };
        if let Some(tooltip) = tooltip {
            wchar_array(tooltip, icon.nid.szTip.as_mut());
        }
        icon.nid.cbSize = std::mem::size_of::<winapi::um::shellapi::NOTIFYICONDATAW>() as u32;
        icon.nid.uID = unsafe { ICON_ID };
        icon.nid.uCallbackMessage = msgs::WM_USER_TRAYICON;
        icon.nid.hIcon = icon.winhicon.hicon;
        icon.nid.uFlags = winapi::um::shellapi::NIF_MESSAGE
            | winapi::um::shellapi::NIF_ICON
            | winapi::um::shellapi::NIF_TIP;

        icon
    }
}

impl WinNotifyIcon {
    pub fn add(&mut self, hwnd: HWND) -> bool {
        self.nid.hWnd = hwnd;
        let res = unsafe {
            winapi::um::shellapi::Shell_NotifyIconW(winapi::um::shellapi::NIM_ADD, &mut self.nid)
        };
        res == 1
    }

    pub fn remove(&mut self) -> bool {
        let res = unsafe {
            winapi::um::shellapi::Shell_NotifyIconW(winapi::um::shellapi::NIM_DELETE, &mut self.nid)
        };
        res == 1
    }

    pub fn set_icon(&mut self, winhicon: &WinHIcon) -> bool {
        self.winhicon = winhicon.clone();
        self.nid.hIcon = self.winhicon.hicon;
        let res = unsafe {
            winapi::um::shellapi::Shell_NotifyIconW(winapi::um::shellapi::NIM_MODIFY, &mut self.nid)
        };
        res == 1
    }

    pub fn set_tooltip(&mut self, tooltip: &str) -> bool {
        wchar_array(tooltip, self.nid.szTip.as_mut());
        let res = unsafe {
            winapi::um::shellapi::Shell_NotifyIconW(winapi::um::shellapi::NIM_MODIFY, &mut self.nid)
        };
        res == 1
    }
}
unsafe impl Send for WinNotifyIcon {}
unsafe impl Sync for WinNotifyIcon {}

impl Debug for WinNotifyIcon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TrayIcon")
    }
}

impl Drop for WinNotifyIcon {
    fn drop(&mut self) {
        unsafe {
            winapi::um::shellapi::Shell_NotifyIconW(
                winapi::um::shellapi::NIM_DELETE,
                &mut self.nid,
            );
        }
    }
}
