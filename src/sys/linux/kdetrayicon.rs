use super::kdeicon::KdeIcon;
use super::MenuSys;
use crate::{
    sys::dbus::{
        get_dbus_connection, register_notifier_item_watcher_blocking, StatusNotifierEvent,
        StatusNotifierItemImpl,
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
    event_sender: Option<std::sync::mpsc::Sender<(i32, T)>>,
    icon_data: Arc<Mutex<KdeIcon>>,
    tooltip_data: Arc<Mutex<String>>,
    title_data: Arc<Mutex<String>>,
    last_xdg_activation_token: Arc<Mutex<Option<String>>>,
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
        title: String,
        // notify_icon: WinNotifyIcon,
        on_click: Option<T>,
        _on_double_click: Option<T>,
        _on_right_click: Option<T>,
        item_is_menu: bool,
    ) -> Result<KdeTrayIconImpl<T>, Error> {
        let connection = get_dbus_connection();
        let (sender, receiver) = std::sync::mpsc::channel();

        // Extract icon data if available
        let (icon_buffer, icon_width, icon_height) = if let Some(icon) = icon {
            let buffer = icon.sys.argb_pixels.clone();
            (buffer, icon.sys.width, icon.sys.height)
        } else {
            (None, 0, 0)
        };

        let (_, icon_data_ref, tooltip_data_ref, title_data_ref) =
            register_notifier_item_watcher_blocking(
                connection,
                sender.clone(),
                icon_buffer,
                icon_width,
                icon_height,
                tooltip,
                title,
                item_is_menu,
            );

        // Store the event_sender if menu exists
        let event_sender = menu.as_ref().and_then(|m| m.event_sender.clone());

        let tray_sender = tray_icon_sender.clone();

        let last_xdg_activation_token = Arc::new(Mutex::new(None));
        let last_xdg_activation_token_clone = last_xdg_activation_token.clone();
        std::thread::spawn(move || {
            while let Ok(event) = receiver.recv() {
                match event {
                    // Handle events here, e.g., map to tray icon actions
                    StatusNotifierEvent::Activate(_x, _y) => {
                        if let Some(on_click) = &on_click {
                            tray_sender.send(on_click);
                        }
                    }
                    StatusNotifierEvent::ProvideXdgActivationToken(token) => {
                        if let Ok(mut last_token) = last_xdg_activation_token_clone.lock() {
                            *last_token = Some(token);
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
            tooltip_data: tooltip_data_ref,
            title_data: title_data_ref,
            last_xdg_activation_token,
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
        // Update the shared icon data
        if let Ok(mut icon_data) = self.icon_data.lock() {
            *icon_data = kde_tray_icon.clone().sys;
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

    fn set_tooltip(&mut self, tooltip: &str) -> Result<(), Error> {
        // Update the tooltip data
        if let Ok(mut tooltip_data) = self.tooltip_data.lock() {
            *tooltip_data = tooltip.to_string();
        }

        // Emit the NewToolTip signal to notify the system tray
        let connection = get_dbus_connection();
        futures::executor::block_on(async {
            if let Ok(obj) = connection
                .object_server()
                .interface::<_, StatusNotifierItemImpl>("/StatusNotifierItem")
                .await
            {
                let emitter = obj.signal_emitter();
                let _ = StatusNotifierItemImpl::new_tool_tip(&emitter).await;
            }
        });

        Ok(())
    }

    fn set_title(&mut self, title: &str) -> Result<(), Error> {
        // Update the title data
        if let Ok(mut title_data) = self.title_data.lock() {
            *title_data = title.to_string();
        }

        // Emit the NewTitle signal to notify the system tray
        let connection = get_dbus_connection();
        futures::executor::block_on(async {
            if let Ok(obj) = connection
                .object_server()
                .interface::<_, StatusNotifierItemImpl>("/StatusNotifierItem")
                .await
            {
                let emitter = obj.signal_emitter();
                let _ = StatusNotifierItemImpl::new_title(&emitter).await;
            }
        });

        Ok(())
    }

    fn set_status(&mut self, status: crate::TrayIconStatus) -> Result<(), Error> {
        use crate::TrayIconStatus;

        let status_str = match status {
            TrayIconStatus::Active => "Active",
            TrayIconStatus::NeedsAttention => "NeedsAttention",
            TrayIconStatus::Passive => "Passive",
        };

        let connection = get_dbus_connection();
        futures::executor::block_on(async {
            if let Ok(obj) = connection
                .object_server()
                .interface::<_, StatusNotifierItemImpl>("/StatusNotifierItem")
                .await
            {
                let emitter = obj.signal_emitter();
                if let Err(e) = StatusNotifierItemImpl::new_status(&emitter, status_str).await {
                    eprintln!("Failed to emit NewStatus signal: {:?}", e);
                }
            }
        });
        Ok(())
    }

    fn get_xdg_activation_token(&self) -> Option<String> {
        if let Ok(token_lock) = self.last_xdg_activation_token.lock() {
            token_lock.clone()
        } else {
            None
        }
    }
}
