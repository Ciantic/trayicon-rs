use super::MenuSys;
use crate::{
    sys::dbus::{
        get_dbus_connection, register_notifier_item_watcher_blocking, IconData,
        StatusNotifierEvent, StatusNotifierItemImpl,
    },
    trayiconsender::TrayIconSender,
    Error, TrayIconBase, TrayIconEvent,
};
use std::sync::{Arc, Mutex};

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
    event_sender: Arc<Mutex<Option<std::sync::mpsc::Sender<(i32, T)>>>>,
    icon_data: Arc<Mutex<IconData>>,
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
        icon: Option<&crate::Icon>,
        tooltip: String,
        // notify_icon: WinNotifyIcon,
        on_click: Option<T>,
        _on_double_click: Option<T>,
        _on_right_click: Option<T>,
    ) -> Result<KdeTrayIconImpl<T>, Error> {
        let connection = get_dbus_connection();
        let (sender, receiver) = std::sync::mpsc::channel();

        // Extract icon data if available
        let (icon_buffer, icon_width, icon_height) = if let Some(icon) = icon {
            // Use decoded RGBA pixels if available, otherwise fall back to raw buffer
            let buffer = if let Some(ref rgba) = icon.sys.rgba_pixels {
                Some(rgba.clone())
            } else {
                Some(icon.sys.buffer.to_vec())
            };
            (buffer, icon.sys.width, icon.sys.height)
        } else {
            (None, 0, 0)
        };

        let (_, icon_data_ref) = register_notifier_item_watcher_blocking(
            connection,
            sender.clone(),
            icon_buffer,
            icon_width,
            icon_height,
            tooltip,
        );

        // Store the event_sender if menu exists
        let event_sender = if let Some(ref m) = menu {
            m.event_sender.clone()
        } else {
            Arc::new(Mutex::new(None))
        };

        let tray_sender = tray_icon_sender.clone();
        std::thread::spawn(move || {
            while let Ok(event) = receiver.recv() {
                match event {
                    // Handle events here, e.g., map to tray icon actions
                    StatusNotifierEvent::Activate(_x, _y) => {
                        if let Some(on_click) = &on_click {
                            tray_sender.send(on_click);
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
            // sender: tray_icon_sender,
            menu,
            event_sender,
            icon_data: icon_data_ref,
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
    fn set_icon(&mut self, kde_tray_icon: &crate::Icon) -> Result<(), Error> {
        // Extract the new icon data
        let (buffer, width, height) = if let Some(ref rgba) = kde_tray_icon.sys.rgba_pixels {
            (
                Some(rgba.clone()),
                kde_tray_icon.sys.width,
                kde_tray_icon.sys.height,
            )
        } else {
            (
                Some(kde_tray_icon.sys.buffer.to_vec()),
                kde_tray_icon.sys.width,
                kde_tray_icon.sys.height,
            )
        };

        // Update the shared icon data
        if let Ok(mut icon_data) = self.icon_data.lock() {
            icon_data.buffer = buffer;
            icon_data.width = width;
            icon_data.height = height;
        }

        // Emit NewIcon signal to notify the tray that the icon changed
        let connection = get_dbus_connection();
        futures::executor::block_on(async {
            if let Ok(obj) = connection
                .object_server()
                .interface::<_, StatusNotifierItemImpl>("/StatusNotifierItem")
                .await
            {
                let emitter = obj.signal_emitter();
                if let Err(e) = StatusNotifierItemImpl::new_icon(&emitter).await {
                    eprintln!("Failed to emit NewIcon signal: {:?}", e);
                }
            }
        });

        Ok(())
    }

    fn set_menu(&mut self, menu: &crate::MenuBuilder<T>) -> Result<(), Error> {
        use crate::sys::dbus::get_dbus_connection;

        // Build the new menu
        let mut built_menu = super::build_menu(menu)?;

        // Reuse the existing event_sender so the event handling thread continues to work
        built_menu.event_sender = self.event_sender.clone();

        // Get the connection and update the DBus menu
        let connection = get_dbus_connection();

        // Replace the menu at /MenuBar with the new menu
        futures::executor::block_on(async {
            // Remove old menu
            let _ = connection
                .object_server()
                .remove::<crate::sys::dbus::DbusMenu<T>, _>("/MenuBar")
                .await;

            // Register new menu
            let dbus_menu = crate::sys::dbus::DbusMenu::new(built_menu.clone());
            let _ = connection
                .object_server()
                .at("/MenuBar", dbus_menu)
                .await
                .unwrap();

            // Get the interface and emit layout_updated signal
            if let Ok(iface) = connection
                .object_server()
                .interface::<_, crate::sys::dbus::DbusMenu<T>>("/MenuBar")
                .await
            {
                let emitter = iface.signal_emitter();
                let _ = crate::sys::dbus::DbusMenu::<T>::layout_updated(&emitter, 0, 0).await;
            }
        });

        // Store the new menu
        self.menu = Some(built_menu);

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
