use futures::future;
use zbus::names::OwnedWellKnownName;

use super::MenuSys;
use crate::{
    sys::dbus::register_notifier_item_watcher_blocking, sys::dbus::StatusNotifierWatcherProxy,
    trayiconsender::TrayIconSender, Error, TrayIconBase,
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
        let (connection, status_notifier_proxy) = register_notifier_item_watcher_blocking();

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
