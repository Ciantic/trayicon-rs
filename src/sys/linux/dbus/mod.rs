mod canonical_dbus_menu;
mod status_notifier_item;
mod status_notifier_watcher;
pub use canonical_dbus_menu::*;
pub use status_notifier_item::{IconData, StatusNotifierEvent, StatusNotifierItemImpl};
pub use status_notifier_watcher::StatusNotifierWatcherProxy;
use std::sync::{Arc, LazyLock, Mutex};
use zbus::names::OwnedWellKnownName;

static DBUS_CONNECTION: LazyLock<zbus::Connection> = LazyLock::new(|| {
    futures::executor::block_on(async {
        zbus::Connection::session()
            .await
            .expect("Failed to connect to session bus")
    })
});

pub fn get_dbus_connection() -> &'static zbus::Connection {
    &DBUS_CONNECTION
}

pub fn register_dbus_menu_blocking<T>(connection: &zbus::Connection, menu_sys: super::MenuSys<T>)
where
    T: crate::TrayIconEvent,
{
    return futures::executor::block_on(async {
        let dbus_menu = DbusMenu::new(menu_sys);
        let _ = connection
            .object_server()
            .at("/MenuBar", dbus_menu)
            .await
            .unwrap();
    });
}

pub fn register_notifier_item_watcher_blocking(
    connection: &zbus::Connection,
    channel_sender: std::sync::mpsc::Sender<StatusNotifierEvent>,
    icon_buffer: Option<Vec<u8>>,
    icon_width: u32,
    icon_height: u32,
    tooltip: String,
) -> (
    StatusNotifierWatcherProxy<'static>,
    Arc<Mutex<IconData>>,
    Arc<Mutex<String>>,
) {
    // Create the StatusNotifierWatcher proxy and register our item
    return futures::executor::block_on(async {
        let unique_name = format!("org.kde.StatusNotifierItem-{}-1", std::process::id()); // TODO: make unique
        let owned_name = OwnedWellKnownName::try_from(unique_name.clone()).unwrap();
        let _ = connection.request_name(owned_name).await;

        let icon_data = Arc::new(Mutex::new(IconData {
            buffer: icon_buffer,
            width: icon_width,
            height: icon_height,
        }));

        let tooltip_data = Arc::new(Mutex::new(tooltip));

        let status_notifier_item = StatusNotifierItemImpl {
            id: unique_name.clone(),
            channel_sender,
            icon_data: icon_data.clone(),
            tooltip: tooltip_data.clone(),
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

        // println!("Connected to StatusNotifierWatcher");

        // Check if there's a StatusNotifierHost registered
        match proxy.is_status_notifier_host_registered().await {
            Ok(_registered) => {
                //println!("StatusNotifierHost registered: {}", _registered)
                ()
            }
            Err(e) => eprintln!("Failed to check host registration: {:?}", e),
        }

        match proxy.register_status_notifier_item(&unique_name).await {
            Ok(_) => {
                // println!("Successfully registered as: {}", unique_name)
                ()
            }
            Err(e) => println!("Failed to register: {:?}", e),
        }

        // Get the object from the server and emit the NewIcon signal
        // This tells the tray host that our icon is ready
        if let Ok(obj) = connection
            .object_server()
            .interface::<_, StatusNotifierItemImpl>("/StatusNotifierItem")
            .await
        {
            // println!("Emitting NewIcon signal to notify tray of icon availability");
            let emitter = obj.signal_emitter();
            if let Err(e) = StatusNotifierItemImpl::new_icon(&emitter).await {
                eprintln!("Failed to emit NewIcon signal: {:?}", e);
            }
        }

        (proxy, icon_data, tooltip_data)
    });
}
