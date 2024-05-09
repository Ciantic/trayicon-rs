use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};
use winapi::shared::minwindef::{HIWORD, LOWORD, LPARAM, LPVOID, LRESULT, UINT, WPARAM};
use winapi::shared::windef::{HBRUSH, HICON, HMENU, HWND, POINT};
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser;

use super::wchar::wchar;
use super::{msgs, winnotifyicon::WinNotifyIcon, MenuSys};
use crate::{trayiconsender::TrayIconSender, Error, Icon, MenuBuilder, TrayIconBase};

pub type WinTrayIcon<T> = WindowBox<T>;

/// WindowBox retains the memory for the Window object until WM_NCDESTROY
#[derive(Debug)]
pub struct WindowBox<T>(*mut WinTrayIconImpl<T>)
where
    T: PartialEq + Clone + 'static;

impl<T> Drop for WindowBox<T>
where
    T: PartialEq + Clone + 'static,
{
    fn drop(&mut self) {
        unsafe {
            // PostMessage doesn't seem to work here, because winit exits before it manages to be processed

            // https://devblogs.microsoft.com/oldnewthing/20110926-00/?p=9553
            winuser::SendMessageW(self.hwnd, winuser::WM_CLOSE, 0, 0);
        }
    }
}

impl<T> Deref for WindowBox<T>
where
    T: PartialEq + Clone + 'static,
{
    type Target = WinTrayIconImpl<T>;

    fn deref(&self) -> &WinTrayIconImpl<T> {
        unsafe { &mut *(self.0) }
    }
}

impl<T> DerefMut for WindowBox<T>
where
    T: PartialEq + Clone + 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self.0) }
    }
}

/// Tray Icon WINAPI Window
///
/// In Windows the Tray Icon requires a window for message pump, it's not shown.
#[derive(Debug)]
pub struct WinTrayIconImpl<T>
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
    msg_taskbarcreated: Option<UINT>,
}

unsafe impl<T> Send for WinTrayIconImpl<T> where T: PartialEq + Clone {}
unsafe impl<T> Sync for WinTrayIconImpl<T> where T: PartialEq + Clone {}

impl<T> WinTrayIconImpl<T>
where
    T: PartialEq + Clone + 'static,
{
    #[allow(clippy::new_ret_no_self)]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        sender: TrayIconSender<T>,
        menu: Option<MenuSys<T>>,
        notify_icon: WinNotifyIcon,
        on_click: Option<T>,
        on_double_click: Option<T>,
        on_right_click: Option<T>,
    ) -> Result<WinTrayIcon<T>, Error>
    where
        T: PartialEq + Clone + 'static,
    {
        unsafe {
            let hinstance = GetModuleHandleW(0 as _);
            let wnd_class_name = wchar("TrayIconCls");
            let wnd_class = winuser::WNDCLASSW {
                style: 0,
                lpfnWndProc: Some(WinTrayIconImpl::<T>::winproc),
                hInstance: hinstance,
                lpszClassName: wnd_class_name.as_ptr() as _,
                cbClsExtra: 0,
                cbWndExtra: 0,
                hIcon: 0 as HICON,
                hCursor: 0 as HICON,
                hbrBackground: 0 as HBRUSH,
                lpszMenuName: 0 as _,
            };
            winuser::RegisterClassW(&wnd_class);

            // Create window in a memory location that doesn't change
            let window = Box::new(WinTrayIconImpl {
                hwnd: 0 as HWND,
                notify_icon,
                menu,
                on_click,
                on_right_click,
                on_double_click,
                sender,
                msg_taskbarcreated: None,
            });
            let ptr = Box::into_raw(window);
            let hwnd = winuser::CreateWindowExW(
                0,
                wnd_class_name.as_ptr() as _,
                wchar("TrayIcon").as_ptr() as _,
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

            Ok(WindowBox(ptr))
        }
    }

    pub fn wndproc(&mut self, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match msg {
            winuser::WM_CREATE => {
                // Create notification area icon
                self.notify_icon.add(self.hwnd);

                // Register to listen taskbar creation
                self.msg_taskbarcreated = unsafe {
                    Some(winuser::RegisterWindowMessageA(
                        "TaskbarCreated\0".as_ptr() as _
                    ))
                };
            }

            msgs::WM_USER_SHOW_MENU => {
                if let Some(menu) = &self.menu {
                    let mut pos = POINT { x: 0, y: 0 };
                    unsafe {
                        winuser::GetCursorPos(&mut pos as _);
                        winuser::SetForegroundWindow(self.hwnd);
                    }
                    menu.menu.track(self.hwnd, pos.x, pos.y);
                }
            }

            // Mouse events on the tray icon
            msgs::WM_USER_TRAYICON => {
                match lparam as u32 {
                    // Left click tray icon
                    winuser::WM_LBUTTONUP => {
                        if let Some(e) = self.on_click.as_ref() {
                            self.sender.send(e);
                        }
                    }

                    // Right click tray icon
                    winuser::WM_RBUTTONUP => {
                        // Send right click event
                        if let Some(e) = self.on_right_click.as_ref() {
                            self.sender.send(e);
                        }
                    }

                    // Double click tray icon
                    winuser::WM_LBUTTONDBLCLK => {
                        if let Some(e) = self.on_double_click.as_ref() {
                            self.sender.send(e);
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
                    if let Some(v) = self.menu.as_ref() {
                        if let Some(event) = v.ids.get(&(identifier as usize)) {
                            self.sender.send(event);
                        }
                    }
                }
            }

            // TaskbarCreated
            x if Some(x) == self.msg_taskbarcreated => {
                self.notify_icon.add(self.hwnd);
            }

            // Default
            _ => {
                return unsafe { winuser::DefWindowProcW(self.hwnd, msg, wparam, lparam) };
            }
        }
        0
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
                let create_struct: &mut winuser::CREATESTRUCTW = &mut *(lparam as *mut _);
                // Arc::from_raw(ptr)
                let window: &mut WinTrayIconImpl<T> =
                    &mut *(create_struct.lpCreateParams as *mut _);
                window.hwnd = hwnd;
                winuser::SetWindowLongPtrW(hwnd, winuser::GWL_USERDATA, window as *mut _ as _);
                window.wndproc(msg, wparam, lparam)
            }
            winuser::WM_NCDESTROY => {
                let window_ptr = winuser::SetWindowLongPtrW(hwnd, winuser::GWL_USERDATA, 0);
                if window_ptr != 0 {
                    let ptr = window_ptr as *mut WinTrayIconImpl<T>;
                    let mut window = Box::from_raw(ptr);
                    window.wndproc(msg, wparam, lparam)
                } else {
                    winuser::DefWindowProcW(hwnd, msg, wparam, lparam)
                }
            }
            _ => {
                let window_ptr = winuser::GetWindowLongPtrW(hwnd, winuser::GWL_USERDATA);
                if window_ptr != 0 {
                    let window: &mut WinTrayIconImpl<T> = &mut *(window_ptr as *mut _);
                    window.wndproc(msg, wparam, lparam)
                } else {
                    winuser::DefWindowProcW(hwnd, msg, wparam, lparam)
                }
            }
        }
    }
}

impl<T> TrayIconBase<T> for WinTrayIconImpl<T>
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

    /// Show menu
    ///
    /// Currently shows always in mouse cursor coordinates
    fn show_menu(&mut self) -> Result<(), Error> {
        unsafe {
            winuser::PostMessageW(self.hwnd, msgs::WM_USER_SHOW_MENU, 0, 0);
        }
        Ok(())
    }
}

impl<T> Drop for WinTrayIconImpl<T>
where
    T: PartialEq + Clone + 'static,
{
    fn drop(&mut self) {
        self.notify_icon.remove();
    }
}
