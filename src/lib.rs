//! ## Example
//! [Open full example with winit here 🢅](https://github.com/Ciantic/trayicon-rs/blob/master/examples/winit/src/main.rs)

use std::fmt::Debug;

#[cfg(target_os = "windows")]
#[path = "./sys/windows/mod.rs"]
mod sys;

mod icon;
mod menubuilder;
mod trayicon;
mod trayiconbuilder;

// Each OS specific implementation must export following:
pub(crate) use sys::{build_menu, build_trayicon, IconSys, MenuSys, TrayIconSys};

pub use crate::icon::Icon;
pub use crate::menubuilder::{MenuBuilder, MenuItem};
pub use crate::trayicon::TrayIcon;
pub use crate::trayiconbuilder::TrayIconBuilder;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Error {
    IconLoadingFailed,
    SenderMissing,
    IconMissing,
    OsError,
}

// Why do I need to do this, can't Rust do this automatically?
impl From<&Error> for Error {
    fn from(e: &Error) -> Self {
        *e
    }
}

/// Tray Icon event sender
#[derive(Debug, Clone)]
pub(crate) enum TrayIconSender<T>
where
    T: PartialEq + Clone + 'static,
{
    Std(std::sync::mpsc::Sender<T>),

    #[cfg(feature = "winit")]
    Winit(winit::event_loop::EventLoopProxy<T>),

    #[cfg(feature = "crossbeam-channel")]
    Crossbeam(crossbeam_channel::Sender<T>),
}

impl<T> TrayIconSender<T>
where
    T: PartialEq + Clone + 'static,
{
    pub fn send(&self, e: &T) {
        match self {
            TrayIconSender::Std(s) => {
                let _ = s.send(e.clone());
            }
            #[cfg(feature = "winit")]
            TrayIconSender::Winit(s) => {
                let _ = s.send_event(e.clone());
            }
            #[cfg(feature = "crossbeam-channel")]
            TrayIconSender::Crossbeam(s) => {
                let _ = s.try_send(e.clone());
            }
        }
    }
}
