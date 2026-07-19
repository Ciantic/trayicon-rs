mod canonical_dbus_menu;
mod status_notifier_item;
mod status_notifier_watcher;
use super::kdeicon::KdeIcon;
pub use canonical_dbus_menu::*;
pub use status_notifier_item::{StatusNotifierEvent, StatusNotifierItemImpl};
pub use status_notifier_watcher::StatusNotifierWatcherProxy;
use std::sync::atomic::{AtomicU32, Ordering};
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

/// Monotonically increasing layout revision.
///
/// The com.canonical.dbusmenu spec requires `LayoutUpdated` to carry a
/// strictly increasing `revision`. Tray hosts (e.g. KDE's system tray applet)
/// cache the last revision they fetched and *ignore* `LayoutUpdated` signals
/// whose revision is not greater than the cached one. Previously this crate
/// always emitted revision 0, so the host fetched the layout once and then
/// ignored every subsequent rebuild — meaning `set_menu` updates (including
/// corrected checkable / radio toggle-state) never reached the UI. The
/// result was that mutually-exclusive items appeared to accumulate checks
/// instead of being exclusive.
///
/// There is exactly one menu per process, so a single process-global counter
/// is sufficient.
static LAYOUT_REVISION: AtomicU32 = AtomicU32::new(0);

/// Return the next layout revision, strictly greater than every revision
/// previously handed out.
pub fn next_layout_revision() -> u32 {
    LAYOUT_REVISION.fetch_add(1, Ordering::SeqCst) + 1
}

/// Return the most recently handed-out layout revision.
///
/// `GetLayout` must return the revision of the layout it is handing back, and
/// that revision must match the one carried by the corresponding
/// `LayoutUpdated` signal. Revision-aware hosts (KDE among them) cache the last
/// revision they fetched and ignore later `LayoutUpdated` signals whose
/// revision is not greater; if `GetLayout` reports a stale or constant
/// revision, a later update can be misordered or dropped. Returning the live
/// counter here keeps `GetLayout` and `LayoutUpdated` in agreement.
pub fn current_layout_revision() -> u32 {
    LAYOUT_REVISION.load(Ordering::SeqCst)
}

pub fn register_dbus_menu_blocking<T>(
    connection: &zbus::Connection,
    menu_sys: super::MenuSys<T>,
    menu_state: crate::SharedMenu<T>,
) -> Result<(), crate::Error>
where
    T: crate::TrayIconEvent,
{
    futures::executor::block_on(async {
        let dbus_menu = DbusMenu::new(menu_sys, menu_state);
        connection
            .object_server()
            .at("/MenuBar", dbus_menu)
            .await
            .map(|_| ())
            .map_err(|_| crate::Error::OsError)
    })
}

pub fn register_notifier_item_watcher_blocking(
    connection: &zbus::Connection,
    channel_sender: std::sync::mpsc::Sender<StatusNotifierEvent>,
    icon_buffer: Option<Vec<u8>>,
    icon_width: u32,
    icon_height: u32,
    tooltip: String,
    title: String,
    item_is_menu: bool,
) -> (
    StatusNotifierWatcherProxy<'static>,
    Arc<Mutex<KdeIcon>>,
    Arc<Mutex<String>>,
    Arc<Mutex<String>>,
) {
    // Create the StatusNotifierWatcher proxy and register our item
    return futures::executor::block_on(async {
        let unique_name = format!("org.kde.StatusNotifierItem-{}-1", std::process::id()); // TODO: make unique
        let owned_name = OwnedWellKnownName::try_from(unique_name.clone()).unwrap();
        let _ = connection.request_name(owned_name).await;

        let icon_data = Arc::new(Mutex::new(KdeIcon {
            argb_pixels: icon_buffer,
            width: icon_width,
            height: icon_height,
        }));

        let tooltip_data = Arc::new(Mutex::new(tooltip));
        let title_data = Arc::new(Mutex::new(title));

        let status_notifier_item = StatusNotifierItemImpl {
            id: unique_name.clone(),
            channel_sender,
            icon_data: icon_data.clone(),
            tooltip: tooltip_data.clone(),
            title: title_data.clone(),
            item_is_menu,
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

        (proxy, icon_data, tooltip_data, title_data)
    });
}
