mod icon;
mod menu;
mod trayicon;

use crate::{Error, MenuBuilder, TrayIconBuilder};
use std::collections::HashMap;

// macOS implementations of Icon, TrayIcon, and Menu
pub use icon::MacIcon as IconSys;
pub use trayicon::MacTrayIcon as TrayIconSys;

#[allow(dead_code)]
pub struct MenuSys<T>
where
    T: PartialEq + Clone + 'static + Send + Sync,
{
    #[allow(dead_code)]
    ids: HashMap<usize, T>,
    #[allow(dead_code)]
    menu: menu::MacMenu<T>,
}

/// Build the tray icon
pub fn build_trayicon<T>(builder: &TrayIconBuilder<T>) -> Result<TrayIconSys<T>, Error>
where
    T: PartialEq + Clone + 'static + Send + Sync,
{
    trayicon::build_trayicon(builder)
}

#[allow(dead_code)]
/// Build the menu from MenuBuilder
pub fn build_menu<T>(builder: &MenuBuilder<T>) -> Result<MenuSys<T>, Error>
where
    T: PartialEq + Clone + 'static + Send + Sync,
{
    // Create a dummy sender for menu building - real sender will be attached later
    let dummy_sender = crate::trayiconsender::TrayIconSender::new(|_| {});
    let mac_menu = menu::build_menu(builder, &dummy_sender)?;
    Ok(MenuSys {
        ids: mac_menu.ids.clone(),
        menu: mac_menu,
    })
}
