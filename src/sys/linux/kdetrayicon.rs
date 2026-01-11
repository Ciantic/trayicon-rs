use futures::future;
use zbus::names::OwnedWellKnownName;

use super::MenuSys;
use crate::{
    sys::{
        status_notifier_item::{self, StatusNotifierItemImpl},
        status_notifier_watcher::StatusNotifierWatcherProxy,
    },
    trayiconsender::TrayIconSender,
    Error, TrayIconBase,
};

#[derive(Debug)]
pub struct KdeTrayIconImpl<T>
where
    T: PartialEq + Clone + 'static,
{
    connection: zbus::Connection,
    // status_notifier_item: StatusNotifierItemImpl,
    status_notifier_proxy: Box<StatusNotifierWatcherProxy<'static>>,
    sender: TrayIconSender<T>,
    menu: Option<MenuSys<T>>,
    // notify_icon: WinNotifyIcon,
    on_click: Option<T>,
    on_double_click: Option<T>,
    on_right_click: Option<T>,
    // msg_taskbarcreated: Option<UINT>,
}

impl<T> KdeTrayIconImpl<T>
where
    T: PartialEq + Clone + 'static,
{
    #[allow(clippy::new_ret_no_self)]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        sender: TrayIconSender<T>,
        menu: Option<MenuSys<T>>,
        // notify_icon: WinNotifyIcon,
        on_click: Option<T>,
        on_double_click: Option<T>,
        on_right_click: Option<T>,
    ) -> Result<KdeTrayIconImpl<T>, Error>
    where
        T: PartialEq + Clone + 'static,
    {
        // Create the StatusNotifierWatcher proxy and register our item
        let (connection, status_notifier_proxy) = futures::executor::block_on(async {
            let connection = zbus::Connection::session().await.unwrap();
            let unique_name = format!("org.kde.StatusNotifierItem-{}-1", std::process::id()); // TODO: make unique
            let owned_name = OwnedWellKnownName::try_from(unique_name.clone()).unwrap();
            let _ = connection.request_name(owned_name).await;
            let status_notifier_item = StatusNotifierItemImpl {
                id: unique_name.clone(),
            };
            let _ = connection
                .object_server()
                .at("/StatusNotifierItem", status_notifier_item)
                .await
                .unwrap();
            let proxy = StatusNotifierWatcherProxy::builder(&connection)
                .destination("org.kde.StatusNotifierWatcher")
                .unwrap()
                .path("/StatusNotifierWatcher")
                .unwrap()
                .build()
                .await
                .unwrap();

            println!("Connected to StatusNotifierWatcher");

            // Check if there's a StatusNotifierHost registered
            match proxy.is_status_notifier_host_registered().await {
                Ok(registered) => println!("StatusNotifierHost registered: {}", registered),
                Err(e) => println!("Failed to check host registration: {:?}", e),
            }

            match proxy.register_status_notifier_item(&unique_name).await {
                Ok(_) => println!("Successfully registered as: {}", unique_name),
                Err(e) => println!("Failed to register: {:?}", e),
            }

            (connection, proxy)
        });

        Ok(KdeTrayIconImpl {
            connection,
            status_notifier_proxy: Box::new(status_notifier_proxy),
            // status_notifier_item,
            sender,
            menu,
            // notify_icon,
            on_click,
            on_double_click,
            on_right_click,
        })
    }
}

impl<T> TrayIconBase<T> for KdeTrayIconImpl<T>
where
    T: PartialEq + Clone + 'static,
{
    fn set_icon(&mut self, KdeTrayIconImpl: &crate::Icon) -> Result<(), Error> {
        // TODO: ...
        Ok(())
    }

    fn set_menu(&mut self, menu: &crate::MenuBuilder<T>) -> Result<(), Error> {
        // TODO: ...
        Ok(())
    }

    fn set_tooltip(&mut self, tooltip: &str) -> Result<(), Error> {
        // TODO: ...
        Ok(())
    }

    fn show_menu(&mut self) -> Result<(), Error> {
        // TODO: ...
        Ok(())
    }
}
