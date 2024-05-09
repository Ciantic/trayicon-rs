//! ## Example
//! [Open full example with winit here 🢅](https://github.com/Ciantic/trayicon-rs/blob/master/examples/winit/src/main.rs)

#[cfg(target_os = "windows")]
#[path = "./sys/windows/mod.rs"]
mod sys;

mod icon;
mod menubuilder;
mod trayicon;
mod trayiconbuilder;
mod trayiconsender;

// Public api
pub use crate::icon::Icon;
pub use crate::menubuilder::{MenuBuilder, MenuItem};
pub use crate::trayicon::TrayIcon;
pub use crate::trayiconbuilder::Error;
pub use crate::trayiconbuilder::TrayIconBuilder;

// Each OS specific implementation must export following:
pub(crate) use crate::sys::{
    // MenuBuilder<T> -> Result<MenuSys<T>, Error>
    build_menu,

    // TrayIconBuilder<T> -> Result<Box<TrayIconSys<T>>, Error>
    build_trayicon,

    // Struct that must implement IconBase + Clone
    IconSys,

    // Struct
    MenuSys,

    // Struct that must implement TrayIconBase
    TrayIconSys,
};

/// TrayIconSys must implement this
pub(crate) trait TrayIconBase<T>
where
    T: PartialEq + Clone + 'static,
{
    fn set_icon(&mut self, icon: &Icon) -> Result<(), Error>;
    fn set_menu(&mut self, menu: &MenuBuilder<T>) -> Result<(), Error>;
    fn set_tooltip(&mut self, tooltip: &str) -> Result<(), Error>;
    fn show_menu(&mut self) -> Result<(), Error>;
}

/// IconSys must implement this
pub(crate) trait IconBase {
    fn from_buffer(
        buffer: &'static [u8],
        width: Option<u32>,
        height: Option<u32>,
    ) -> Result<IconSys, Error>;
}
