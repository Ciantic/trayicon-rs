use super::{hicon::WinHIcon, msgs};
use std::fmt::Debug;
use winapi::shared::windef::{HICON, HWND};

/// Purpose of this struct is to retain NotifyIconDataW and remove it on drop
pub struct NotifyIcon {
    winhicon: WinHIcon,
    nid: winapi::um::shellapi::NOTIFYICONDATAW,
    tooltip: [u16; 128],
}

impl NotifyIcon {
    pub fn new(winhicon: WinHIcon) -> NotifyIcon {
        static mut ICON_ID: u32 = 1000;
        unsafe {
            ICON_ID += 1;
        }
        let mut icon = NotifyIcon {
            winhicon,
            nid: unsafe { std::mem::zeroed() },
            tooltip: unsafe { std::mem::zeroed() },
        };
        icon.nid.cbSize = std::mem::size_of::<winapi::um::shellapi::NOTIFYICONDATAW>() as u32; //prep
        icon.nid.uID = unsafe { ICON_ID };
        icon.nid.uCallbackMessage = msgs::WM_USER_TRAYICON;
        icon.nid.hIcon = icon.winhicon.hicon;
        icon.nid.szTip = icon.tooltip;
        icon.nid.uFlags = winapi::um::shellapi::NIF_MESSAGE
            | winapi::um::shellapi::NIF_ICON
            | winapi::um::shellapi::NIF_TIP;

        icon
    }
}

impl NotifyIcon {
    pub fn add(&mut self, hwnd: HWND) -> bool {
        self.nid.hWnd = hwnd;
        let res = unsafe {
            winapi::um::shellapi::Shell_NotifyIconW(winapi::um::shellapi::NIM_ADD, &mut self.nid)
        };
        res == 1
    }

    pub fn set_icon(&mut self, winhicon: &WinHIcon) -> bool {
        let winhicon = winhicon.clone();
        self.winhicon = winhicon;
        self.nid.hIcon = self.winhicon.hicon;
        let res = unsafe {
            winapi::um::shellapi::Shell_NotifyIconW(winapi::um::shellapi::NIM_MODIFY, &mut self.nid)
        };
        res == 1
    }
}
unsafe impl Send for NotifyIcon {}
unsafe impl Sync for NotifyIcon {}

impl Debug for NotifyIcon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TrayIcon")
    }
}

impl Drop for NotifyIcon {
    fn drop(&mut self) {
        unsafe {
            winapi::um::shellapi::Shell_NotifyIconW(
                winapi::um::shellapi::NIM_DELETE,
                &mut self.nid,
            );
            println!("Drop tray icon");
        }
    }
}
