use super::{hmenu::WinHMenu, msgs, notifyicon::NotifyIcon, MenuSys};
use winapi::shared::{
    basetsd::{DWORD_PTR, UINT_PTR},
    minwindef::{HIWORD, LOWORD, LPARAM, LPVOID, LRESULT, UINT, WPARAM},
    windef::{HBRUSH, HICON, HMENU, HWND, POINT},
};
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::winuser;
use winapi::um::winuser::{CreateWindowExA, DefWindowProcA, RegisterClassA};

use crate::{Error, Icon, TrayIconBase, TrayIconSender};
use std::{collections::HashMap, fmt::Debug, sync::mpsc::Sender};
use winapi::um::commctrl;

/// Tray Icon WINAPI Window
///
/// In Windows the Tray Icon requires a window for message pump, it's not shown.
#[derive(Debug)]
pub struct TrayIconWindow<T>
where
    T: PartialEq + Clone,
{
    hwnd: HWND,
    notify_icon: NotifyIcon,
    menu: Option<MenuSys<T>>,
    click_event: Option<T>,
    double_click_event: Option<T>,
    right_click_event: Option<T>,
    sender: TrayIconSender<T>,
}

unsafe impl<T> Send for TrayIconWindow<T> where T: PartialEq + Clone {}
unsafe impl<T> Sync for TrayIconWindow<T> where T: PartialEq + Clone {}

impl<T> TrayIconWindow<T>
where
    T: PartialEq + Clone,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sender: TrayIconSender<T>,
        menu: Option<MenuSys<T>>,
        notify_icon: NotifyIcon,
        parent_hwnd: Option<HWND>,
        click_event: Option<T>,
        double_click_event: Option<T>,
        right_click_event: Option<T>,
    ) -> Result<Box<TrayIconWindow<T>>, Error>
    where
        T: PartialEq + Clone,
    {
        unsafe {
            let hinstance = GetModuleHandleA(0 as _);
            let wnd_class_name = "TrayIconCls\0";
            let wnd_class = winuser::WNDCLASSA {
                style: 0,
                lpfnWndProc: Some(TrayIconWindow::<T>::winproc),
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
            let mut window = Box::new(TrayIconWindow {
                hwnd: 0 as HWND,
                notify_icon,
                menu,
                click_event,
                right_click_event,
                double_click_event,
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
                parent_hwnd.unwrap_or(0 as _),
                0 as HMENU,
                hinstance,
                ptr as *mut _ as LPVOID,
            ) as u32;
            if hwnd == 0 {
                return Err(Error::OsError);
            }
            let proc = commctrl::SetWindowSubclass(
                hwnd as HWND,
                Some(TrayIconWindow::<T>::subproc),
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
        let window: &mut TrayIconWindow<T> = &mut *(data as *mut _);
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
                        if let Some(e) = window.click_event.as_ref() {
                            let _ = window.sender.send(e.clone());
                        }
                    }

                    // Right click tray icon
                    winuser::WM_RBUTTONUP => {
                        // Send right click event
                        if let Some(e) = window.right_click_event.as_ref() {
                            let _ = window.sender.send(e.clone());
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
                        if let Some(e) = window.double_click_event.as_ref() {
                            let _ = window.sender.send(e.clone());
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
                        if let Some(event) = v.events.get(&(identifier as usize)) {
                            let _ = window.sender.send(event.clone());
                        }
                    }
                }
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

impl<T> TrayIconBase<T> for TrayIconWindow<T>
where
    T: PartialEq + Clone,
{
    fn set_icon(&mut self, icon: &Icon) -> Result<(), Error> {
        if !self.notify_icon.set_icon(&icon.0) {
            return Err(Error::IconLoadingFailed);
        }
        Ok(())
    }

    fn set_menu(&mut self, menu: crate::MenuBuilder<T>) -> Result<(), Error> {
        let menu = menu.build()?;
        self.menu = Some(menu);
        Ok(())
    }
}

impl<T> Drop for TrayIconWindow<T>
where
    T: PartialEq + Clone,
{
    fn drop(&mut self) {
        // https://devblogs.microsoft.com/oldnewthing/20110926-00/?p=9553
        unsafe { winuser::SendMessageA(self.hwnd, winuser::WM_CLOSE, 0, 0) };
    }
}
