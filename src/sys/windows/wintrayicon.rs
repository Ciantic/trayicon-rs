use super::{msgs, winnotifyicon::WinNotifyIcon, MenuSys};
use winapi::shared::{
    basetsd::{DWORD_PTR, UINT_PTR},
    minwindef::{HIWORD, LOWORD, LPARAM, LPVOID, LRESULT, UINT, WPARAM},
    windef::{HBRUSH, HICON, HMENU, HWND, POINT},
};
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::winuser;
use winapi::um::winuser::{CreateWindowExA, DefWindowProcA, RegisterClassA};

use crate::{trayiconsender::TrayIconSender, Error, Icon, MenuBuilder, TrayIconBase};
use std::fmt::Debug;
use winapi::um::commctrl;

/// Tray Icon WINAPI Window
///
/// In Windows the Tray Icon requires a window for message pump, it's not shown.
#[derive(Debug)]
pub struct WinTrayIcon<T>
where
    T: PartialEq + Clone + 'static,
{
    hwnd: HWND,
    sender: TrayIconSender<T>,
    menu: Option<MenuSys<T>>,
    notify_icon: WinNotifyIcon,
    on_click: Option<T>,
    on_double_click: Option<T>,
    on_right_click: Option<T>,
}

unsafe impl<T> Send for WinTrayIcon<T> where T: PartialEq + Clone {}
unsafe impl<T> Sync for WinTrayIcon<T> where T: PartialEq + Clone {}

impl<T> WinTrayIcon<T>
where
    T: PartialEq + Clone + 'static,
{
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        sender: TrayIconSender<T>,
        menu: Option<MenuSys<T>>,
        notify_icon: WinNotifyIcon,
        on_click: Option<T>,
        on_double_click: Option<T>,
        on_right_click: Option<T>,
    ) -> Result<Box<WinTrayIcon<T>>, Error>
    where
        T: PartialEq + Clone + 'static,
    {
        unsafe {
            let hinstance = GetModuleHandleA(0 as _);
            let wnd_class_name = "TrayIconCls\0";
            let wnd_class = winuser::WNDCLASSA {
                style: 0,
                lpfnWndProc: Some(WinTrayIcon::<T>::winproc),
                hInstance: hinstance,
                lpszClassName: wnd_class_name.as_ptr() as _,
                cbClsExtra: 0,
                cbWndExtra: 0,
                hIcon: 0 as HICON,
                hCursor: 0 as HICON,
                hbrBackground: 0 as HBRUSH,
                lpszMenuName: 0 as _,
            };
            RegisterClassA(&wnd_class);

            // Create window in a memory location that doesn't change
            let mut window = Box::new(WinTrayIcon {
                hwnd: 0 as HWND,
                notify_icon,
                menu,
                on_click,
                on_right_click,
                on_double_click,
                sender,
            });
            // Take the window memory location and pass it to wndproc and
            // subproc
            //
            // Note that inside wndproc the lParam is not fixed! Thus it doesn't
            // always succeed in setting the lparam, this is the reason we need
            // subproc which has a fixed parameter.
            let ptr = window.as_mut();
            let hwnd = CreateWindowExA(
                0,
                wnd_class_name.as_ptr() as _,
                "TrayIcon\0".as_ptr() as *const i8,
                0, //winuser::WS_OVERLAPPEDWINDOW | winuser::WS_VISIBLE,
                winuser::CW_USEDEFAULT,
                winuser::CW_USEDEFAULT,
                winuser::CW_USEDEFAULT,
                winuser::CW_USEDEFAULT,
                0 as _,
                0 as HMENU,
                hinstance,
                ptr as *mut _ as LPVOID,
            ) as u32;
            if hwnd == 0 {
                return Err(Error::OsError);
            }
            let proc = commctrl::SetWindowSubclass(
                hwnd as HWND,
                Some(WinTrayIcon::<T>::subproc),
                0,
                ptr as *mut _ as usize,
            );
            if proc == 0 {
                return Err(Error::OsError);
            }
            window.hwnd = hwnd as HWND;
            Ok(window)
        }
    }

    // This serves as a conduit for actual winproc in the subproc
    pub unsafe extern "system" fn winproc(
        hwnd: HWND,
        msg: UINT,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            winuser::WM_CREATE => {
                let hwnd = hwnd as u32;
                winuser::PostMessageA(hwnd as HWND, msgs::WM_USER_CREATE, wparam, lparam);
            }
            _ => {
                return DefWindowProcA(hwnd, msg, wparam, lparam);
            }
        }
        0
    }

    // Actual winproc
    unsafe extern "system" fn subproc(
        hwnd: HWND,
        msg: UINT,
        wparam: WPARAM,
        lparam: LPARAM,
        _id: UINT_PTR,
        data: DWORD_PTR,
    ) -> LRESULT {
        static mut WM_TASKBARCREATED: u32 = u32::MAX;
        let window: &mut WinTrayIcon<T> = &mut *(data as *mut _);
        match msg {
            // Window was created
            msgs::WM_USER_CREATE => {
                WM_TASKBARCREATED =
                    winuser::RegisterWindowMessageA("TaskbarCreated\0".as_ptr() as _);
                window.notify_icon.add(hwnd);
            }

            // Mouse events on the tray icon
            msgs::WM_USER_TRAYICON => {
                match lparam as u32 {
                    // Left click tray icon
                    winuser::WM_LBUTTONUP => {
                        if let Some(e) = window.on_click.as_ref() {
                            window.sender.send(e);
                        }
                    }

                    // Right click tray icon
                    winuser::WM_RBUTTONUP => {
                        // Send right click event
                        if let Some(e) = window.on_right_click.as_ref() {
                            window.sender.send(e);
                        }

                        // Show menu, if it's there
                        if let Some(menu) = &window.menu {
                            let mut pos = POINT { x: 0, y: 0 };
                            winuser::GetCursorPos(&mut pos as _);
                            winuser::SetForegroundWindow(hwnd);
                            menu.menu.track(hwnd, pos.x, pos.y);
                        }
                    }

                    // Double click tray icon
                    winuser::WM_LBUTTONDBLCLK => {
                        if let Some(e) = window.on_double_click.as_ref() {
                            window.sender.send(e);
                        }
                    }
                    _ => {}
                }
            }

            // Any of the menu commands
            //
            // https://docs.microsoft.com/en-us/windows/win32/menurc/wm-command#parameters
            winuser::WM_COMMAND => {
                let identifier = LOWORD(wparam as u32);
                let cmd = HIWORD(wparam as u32);

                // Menu command
                if cmd == 0 {
                    if let Some(v) = window.menu.as_ref() {
                        if let Some(event) = v.ids.get(&(identifier as usize)) {
                            window.sender.send(event);
                        }
                    }
                }
            }

            // Destroy
            winuser::WM_DESTROY => {
                window.notify_icon.remove();
            }

            // TaskbarCreated
            x if x == WM_TASKBARCREATED => {
                window.notify_icon.add(hwnd);
            }

            _ => {
                return commctrl::DefSubclassProc(hwnd, msg, wparam, lparam);
            }
        }
        0
    }
}

impl<T> TrayIconBase<T> for WinTrayIcon<T>
where
    T: PartialEq + Clone + 'static,
{
    /// Set the tooltip
    fn set_tooltip(&mut self, tooltip: &str) -> Result<(), Error> {
        if !self.notify_icon.set_tooltip(tooltip) {
            return Err(Error::OsError);
        }
        Ok(())
    }

    /// Set icon
    fn set_icon(&mut self, icon: &Icon) -> Result<(), Error> {
        if !self.notify_icon.set_icon(&icon.sys) {
            return Err(Error::IconLoadingFailed);
        }
        Ok(())
    }

    /// Set menu
    fn set_menu(&mut self, menu: &MenuBuilder<T>) -> Result<(), Error> {
        if menu.menu_items.is_empty() {
            self.menu = None
        } else {
            self.menu = Some(menu.build()?);
        }
        Ok(())
    }
}

impl<T> Drop for WinTrayIcon<T>
where
    T: PartialEq + Clone + 'static,
{
    fn drop(&mut self) {
        // https://devblogs.microsoft.com/oldnewthing/20110926-00/?p=9553
        unsafe { winuser::SendMessageA(self.hwnd, winuser::WM_CLOSE, 0, 0) };
    }
}
