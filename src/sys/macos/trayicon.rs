use super::menu::{build_menu, MacMenu};
use crate::{
    trayiconsender::TrayIconSender, Error, Icon, MenuBuilder, TrayIconBase, TrayIconBuilder,
};
use objc2::msg_send;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_app_kit::{NSStatusBar, NSStatusItem, NSVariableStatusItemLength};
use objc2_foundation::NSString;

pub struct MacTrayIcon<T>
where
    T: PartialEq + Clone + 'static + Send + Sync,
{
    status_item: Retained<NSStatusItem>,
    menu: Option<MacMenu<T>>,
    #[allow(dead_code)]
    sender: TrayIconSender<T>,
    #[allow(dead_code)]
    on_click: Option<T>,
    #[allow(dead_code)]
    on_double_click: Option<T>,
    #[allow(dead_code)]
    on_right_click: Option<T>,
}

impl<T> TrayIconBase<T> for MacTrayIcon<T>
where
    T: PartialEq + Clone + 'static + Send + Sync,
{
    fn set_icon(&mut self, icon: &Icon) -> Result<(), Error> {
        unsafe {
            let button: *mut AnyObject = msg_send![&self.status_item, button];
            if !button.is_null() {
                let _: () = msg_send![button, setImage: &*icon.sys.ns_image];
            }
        }
        Ok(())
    }

    fn set_menu(&mut self, menu: &MenuBuilder<T>) -> Result<(), Error> {
        let mut menu_sys = build_menu(menu, &self.sender)?;
        menu_sys.update_sender(&self.sender);
        self.menu = Some(menu_sys);
        if let Some(ref menu_sys) = self.menu {
            unsafe {
                self.status_item.setMenu(Some(&menu_sys.menu));
            }
        }
        Ok(())
    }

    fn set_tooltip(&mut self, tooltip: &str) -> Result<(), Error> {
        unsafe {
            let button: *mut AnyObject = msg_send![&self.status_item, button];
            if !button.is_null() {
                let ns_tooltip = NSString::from_str(tooltip);
                let _: () = msg_send![button, setToolTip: &*ns_tooltip];
            }
        }
        Ok(())
    }

    fn show_menu(&mut self) -> Result<(), Error> {
        if let Some(ref menu_sys) = self.menu {
            unsafe {
                self.status_item.setMenu(Some(&menu_sys.menu));
            }
        }
        Ok(())
    }
}

impl<T> Drop for MacTrayIcon<T>
where
    T: PartialEq + Clone + 'static + Send + Sync,
{
    fn drop(&mut self) {
        unsafe {
            let status_bar = NSStatusBar::systemStatusBar();
            status_bar.removeStatusItem(&self.status_item);
        }
    }
}

unsafe impl<T> Send for MacTrayIcon<T> where T: PartialEq + Clone + 'static + Send + Sync {}
unsafe impl<T> Sync for MacTrayIcon<T> where T: PartialEq + Clone + 'static + Send + Sync {}

/// Build the tray icon
pub fn build_trayicon<T>(builder: &TrayIconBuilder<T>) -> Result<MacTrayIcon<T>, Error>
where
    T: PartialEq + Clone + 'static + Send + Sync,
{
    let icon = builder.icon.as_ref()?;
    let tooltip = builder.tooltip.as_deref().unwrap_or("");
    let sender = builder.sender.as_ref().ok_or(Error::SenderMissing)?;
    let on_click = builder.on_click.clone();
    let on_double_click = builder.on_double_click.clone();
    let on_right_click = builder.on_right_click.clone();

    let mut menu: Option<MacMenu<T>> = None;
    if let Some(ref menu_builder) = builder.menu {
        let mut menu_sys = build_menu(menu_builder, sender)?;
        menu_sys.update_sender(sender);
        menu = Some(menu_sys);
    }

    unsafe {
        let status_bar = NSStatusBar::systemStatusBar();
        let status_item = status_bar.statusItemWithLength(NSVariableStatusItemLength);

        let button: *mut AnyObject = msg_send![&status_item, button];
        if !button.is_null() {
            let _: () = msg_send![button, setImage: &*icon.sys.ns_image];
            let ns_tooltip = NSString::from_str(tooltip);
            let _: () = msg_send![button, setToolTip: &*ns_tooltip];
        }

        if let Some(ref menu_sys) = menu {
            status_item.setMenu(Some(&menu_sys.menu));
        }

        Ok(MacTrayIcon {
            status_item,
            menu,
            sender: sender.clone(),
            on_click,
            on_double_click,
            on_right_click,
        })
    }
}
