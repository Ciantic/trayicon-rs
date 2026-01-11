mod canonical_dbus_menu;
mod status_notifier_item;
mod status_notifier_watcher;
pub use canonical_dbus_menu::*;
pub use status_notifier_item::StatusNotifierItemImpl;
pub use status_notifier_watcher::StatusNotifierWatcherProxy;
use zbus::names::OwnedWellKnownName;

pub fn register_notifier_item_watcher_blocking(
) -> (zbus::Connection, StatusNotifierWatcherProxy<'static>) {
    // Create the StatusNotifierWatcher proxy and register our item
    return futures::executor::block_on(async {
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
}
