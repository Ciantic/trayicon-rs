use crate::{trayiconsender::TrayIconSender, Icon, MenuBuilder, TrayIcon};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Error {
    MenuItemNotFound,
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

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::error::Error for Error {}

/// Tray Icon builder
///
/// Start by choosing an event sender implementation. There are three different
/// senders depending on the optional features. By default the sender function
/// uses `std::sync::mpsc::Sender<T>`, additionally if `winit` feature is
/// enabled you can choose to use `winit::event_loop::EventLoopProxy<T>` or with
/// `crossbeam-channel` feature the `crossbeam_channel::Sender<T>` is available.
///
/// This is defined as consuming builder, this includes conditional helper
/// `when` for composing conditionally some settings.
///
/// [Open full example with winit here 🢅](https://github.com/Ciantic/trayicon-rs/blob/master/examples/winit/src/main.rs)
#[derive(Debug, Clone)]
pub struct TrayIconBuilder<T>
where
    T: PartialEq + Clone + 'static,
{
    pub(crate) icon: Result<Icon, Error>,
    pub(crate) menu: Option<MenuBuilder<T>>,
    pub(crate) tooltip: Option<String>,
    pub(crate) on_click: Option<T>,
    pub(crate) on_double_click: Option<T>,
    pub(crate) on_right_click: Option<T>,
    pub(crate) sender: Option<TrayIconSender<T>>,
}

impl<T> TrayIconBuilder<T>
where
    T: PartialEq + Clone + 'static,
{
    #[allow(clippy::new_without_default)]
    pub fn new() -> TrayIconBuilder<T> {
        TrayIconBuilder {
            icon: Err(Error::IconMissing),
            menu: None,
            tooltip: None,
            on_click: None,
            on_double_click: None,
            on_right_click: None,
            sender: None,
        }
    }

    /// Conditionally include items, poor mans function composition
    pub fn when<F>(self, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        f(self)
    }

    pub fn sender_callback(mut self, callback: impl Fn(&T) + 'static) -> Self {
        self.sender = Some(TrayIconSender::new(callback));
        self
    }

    pub fn sender(mut self, s: std::sync::mpsc::Sender<T>) -> Self {
        self.sender = Some(TrayIconSender::new(move |e: &T| {
            let _ = s.send(e.clone());
        }));
        self
    }

    /// Optional feature, requires `winit` feature
    #[cfg(feature = "winit")]
    pub fn sender_winit(mut self, s: winit::event_loop::EventLoopProxy<T>) -> Self {
        self.sender = Some(TrayIconSender::new(move |e: &T| {
            let _ = s.send_event(e.clone());
        }));
        self
    }

    /// Optional feature, requires `crossbeam-channel` feature
    #[cfg(feature = "crossbeam-channel")]
    pub fn sender_crossbeam(mut self, s: crossbeam_channel::Sender<T>) -> Self {
        self.sender = Some(TrayIconSender::new(move |e: &T| {
            let _ = s.try_send(e.clone());
        }));
        self
    }

    pub fn tooltip(mut self, tooltip: &str) -> Self {
        self.tooltip = Some(tooltip.to_string());
        self
    }

    pub fn on_click(mut self, id: T) -> Self {
        self.on_click = Some(id);
        self
    }

    pub fn on_double_click(mut self, id: T) -> Self {
        self.on_double_click = Some(id);
        self
    }

    pub fn on_right_click(mut self, id: T) -> Self {
        self.on_right_click = Some(id);
        self
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Ok(icon);
        self
    }

    pub fn icon_from_buffer(mut self, buffer: &'static [u8]) -> Self {
        self.icon = Icon::from_buffer(buffer, None, None);
        self
    }

    pub fn menu(mut self, menu: MenuBuilder<T>) -> Self
    where
        T: PartialEq + Clone + 'static,
    {
        self.menu = Some(menu);
        self
    }

    pub fn build(self) -> Result<TrayIcon<T>, Error> {
        Ok(TrayIcon::new(crate::build_trayicon(&self)?, self))
    }
}
