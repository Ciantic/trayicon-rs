//! ## Example
//! [Open full example with winit here 🢅](https://github.com/Ciantic/trayicon-rs/blob/master/examples/winit/src/main.rs)

#[cfg(target_os = "windows")]
#[path = "./sys/windows/mod.rs"]
mod sys;

#[cfg(target_os = "macos")]
#[path = "./sys/macos/mod.rs"]
mod sys;

#[cfg(target_os = "linux")]
#[path = "./sys/linux/mod.rs"]
mod sys;

mod icon;
mod menubuilder;
mod trayicon;
mod trayiconbuilder;
mod trayiconsender;

/// Helper trait that combines the common trait bounds used throughout the library
pub trait TrayIconEvent: PartialEq + Clone + 'static + Send + Sync {}
impl<T> TrayIconEvent for T where T: PartialEq + Clone + 'static + Send + Sync {}

// Public api
pub use crate::icon::Icon;
pub use crate::menubuilder::{MenuBuilder, MenuItem};
pub use crate::trayicon::TrayIcon;
pub use crate::trayiconbuilder::Error;
pub use crate::trayiconbuilder::TrayIconBuilder;

/// Status/visibility state for the tray icon (KDE StatusNotifierItem status)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayIconStatus {
    /// Normal visible state
    Active,
    /// Icon blinks/animates to draw attention (KDE)
    NeedsAttention,
    /// Icon is hidden or minimized (KDE)
    Passive,
}

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
    T: PartialEq + Clone + 'static + Send + Sync,
{
    fn set_icon(&mut self, icon: &Icon) -> Result<(), Error>;
    fn set_menu(&mut self, menu: &MenuBuilder<T>) -> Result<(), Error>;
    fn set_tooltip(&mut self, tooltip: &str) -> Result<(), Error>;
    fn show_menu(&mut self) -> Result<(), Error>;

    /// Set the status of the tray icon.
    /// On KDE, this controls the StatusNotifierItem status:
    /// - Active: Normal visible state
    /// - NeedsAttention: Icon blinks/animates to draw attention
    /// - Passive: Icon is hidden or minimized
    ///
    /// On other platforms, this does nothing by default.
    fn set_status(&mut self, _status: TrayIconStatus) -> Result<(), Error> {
        Ok(())
    }

    /// KDE specific: Get the XDG activation token provided by the system tray
    /// when the user clicks the tray icon.
    fn get_xdg_activation_token(&self) -> Option<String> {
        None
    }
}

/// IconSys must implement this
pub(crate) trait IconBase {
    fn from_buffer(
        buffer: &'static [u8],
        width: Option<u32>,
        height: Option<u32>,
    ) -> Result<IconSys, Error>;
}
