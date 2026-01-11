use super::MenuSys;
use crate::{
    sys::dbus::{
        get_dbus_connection, register_notifier_item_watcher_blocking, StatusNotifierEvent,
    },
    trayiconsender::TrayIconSender,
    Error, TrayIconBase, TrayIconEvent,
};

#[derive(Debug)]
pub struct KdeTrayIconImpl<T>
where
    T: TrayIconEvent,
{
    // connection: &'static zbus::Connection,
    // status_notifier_item: StatusNotifierItemImpl,
    // status_notifier_proxy: Box<StatusNotifierWatcherProxy<'static>>,
    // sender: TrayIconSender<T>,
    #[allow(dead_code)]
    menu: Option<MenuSys<T>>,
    // notify_icon: WinNotifyIcon,
    // on_click: Option<T>,
    // on_double_click: Option<T>,
    // on_right_click: Option<T>,
    // msg_taskbarcreated: Option<UINT>,
}

impl<T> KdeTrayIconImpl<T>
where
    T: TrayIconEvent,
{
    #[allow(clippy::new_ret_no_self)]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        tray_icon_sender: TrayIconSender<T>,
        menu: Option<MenuSys<T>>,
        // notify_icon: WinNotifyIcon,
        on_click: Option<T>,
        _on_double_click: Option<T>,
        _on_right_click: Option<T>,
    ) -> Result<KdeTrayIconImpl<T>, Error> {
        let connection = get_dbus_connection();
        let (sender, receiver) = std::sync::mpsc::channel();
        let _ = register_notifier_item_watcher_blocking(connection, sender.clone());
        std::thread::spawn(move || {
            while let Ok(event) = receiver.recv() {
                match event {
                    // Handle events here, e.g., map to tray icon actions
                    StatusNotifierEvent::Activate(_x, _y) => {
                        if let Some(on_click) = &on_click {
                            tray_icon_sender.send(on_click);
                        }
                    }
                    _ => {}
                }
            }
        });

        Ok(KdeTrayIconImpl {
            // connection,
            // status_notifier_proxy: Box::new(status_notifier_proxy),
            // status_notifier_item,
            // sender,
            menu,
            // notify_icon,
            // on_click,
            // on_double_click,
            // on_right_click,
        })
    }
}

impl<T> TrayIconBase<T> for KdeTrayIconImpl<T>
where
    T: TrayIconEvent,
{
    fn set_icon(&mut self, _kde_tray_icon: &crate::Icon) -> Result<(), Error> {
        // TODO: ...
        Ok(())
    }

    fn set_menu(&mut self, _menu: &crate::MenuBuilder<T>) -> Result<(), Error> {
        // TODO: ...
        Ok(())
    }

    fn set_tooltip(&mut self, _tooltip: &str) -> Result<(), Error> {
        // TODO: ...
        Ok(())
    }

    fn show_menu(&mut self) -> Result<(), Error> {
        // With KDE, we can't just show the menu programmatically like on Windows and MacOS, it always opens with right click on the tray icon. Leaving this empty for now.
        Ok(())
    }
}
