use crate::{trayiconsender::TrayIconSender, Icon, MenuBuilder, TrayIcon, TrayIconEvent};
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
/// [Open full example with winit here 🢅](https://github.com/Ciantic/trayicon-rs/blob/master/examples/winit/src/main.rs)
#[derive(Debug, Clone)]
pub struct TrayIconBuilder<T>
where
    T: TrayIconEvent,
{
    pub(crate) icon: Result<Icon, Error>,
    pub(crate) menu: Option<MenuBuilder<T>>,
    pub(crate) title: Option<String>,
    pub(crate) tooltip: Option<String>,
    pub(crate) on_click: Option<T>,
    pub(crate) on_double_click: Option<T>,
    pub(crate) on_right_click: Option<T>,
    pub(crate) item_is_menu: bool,
    pub(crate) sender: Option<TrayIconSender<T>>,
}

impl<T> TrayIconBuilder<T>
where
    T: TrayIconEvent,
{
    #[allow(clippy::new_without_default)]
    pub fn new() -> TrayIconBuilder<T> {
        TrayIconBuilder {
            icon: Err(Error::IconMissing),
            menu: None,
            title: None,
            tooltip: None,
            on_click: None,
            on_double_click: None,
            on_right_click: None,
            item_is_menu: false,
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

    pub fn sender(mut self, cb: impl Fn(&T) + Send + Sync + 'static) -> Self {
        self.sender = Some(TrayIconSender::new(cb));
        self
    }

    /// Set title (KDE only)
    ///
    /// Used in KDE as the application title for the tray icon.
    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn tooltip(mut self, tooltip: &str) -> Self {
        self.tooltip = Some(tooltip.to_string());
        self
    }

    /// Set left-click event handler
    ///
    /// Works only on Windows and KDE. In MacOS left click by default opens the menu, binding this is not effective.
    pub fn on_click(mut self, id: T) -> Self {
        self.on_click = Some(id);
        self
    }

    /// Set double click event handler
    ///
    /// Works only on Windows. In MacOS and KDE double click is not registered.
    pub fn on_double_click(mut self, id: T) -> Self {
        self.on_double_click = Some(id);
        self
    }

    /// Set right click event handler
    ///
    /// Binding works only on Windows. In KDE and MacOS right click by default opens the menu, binding this is not effective.
    ///
    /// If not given in Windows it will default to showing the menu on right click like in KDE and MacOS.
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

    /// Indicates that this item only supports the context menu. 
    ///
    /// Set this to true if you want to be able to open the context menu with left click. KDE only
    pub fn item_is_menu(mut self, value: bool) -> Self {
        self.item_is_menu = value;
        self
    }
}
