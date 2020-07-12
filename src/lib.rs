//! ## Example
//! [Open full example with winit here 🢅](https://github.com/Ciantic/trayicon-rs/blob/master/examples/winit/src/main.rs)

use std::fmt::Debug;

#[cfg(target_os = "windows")]
#[path = "./sys/windows/mod.rs"]
mod sys;

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

#[derive(Clone, PartialEq)]
pub struct Icon(sys::IconSys);

impl Debug for Icon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Icon")
    }
}

impl Icon {
    pub fn from_buffer(
        buffer: &'static [u8],
        width: Option<u32>,
        height: Option<u32>,
    ) -> Result<Icon, Error> {
        sys::IconSys::from_buffer(buffer, width, height).map(Icon)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MenuItem<T>
where
    T: PartialEq + Clone + 'static,
{
    Separator,
    Item {
        name: String,
        event: T,
        disabled: bool,
        icon: Option<Icon>,
    },
    CheckableItem {
        name: String,
        is_checked: bool,
        event: T,
        disabled: bool,
        icon: Option<Icon>,
    },
    ChildMenu {
        name: String,
        children: MenuBuilder<T>,
        disabled: bool,
        icon: Option<Icon>,
    },
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct MenuBuilder<T>
where
    T: PartialEq + Clone + 'static,
{
    menu_items: Vec<MenuItem<T>>,
}

impl<T> MenuBuilder<T>
where
    T: PartialEq + Clone + 'static,
{
    pub fn new() -> MenuBuilder<T> {
        MenuBuilder { menu_items: vec![] }
    }

    pub fn with(mut self, item: MenuItem<T>) -> Self {
        self.menu_items.push(item);
        self
    }

    pub fn with_separator(mut self) -> Self {
        self.menu_items.push(MenuItem::Separator);
        self
    }

    pub fn with_item(mut self, name: &str, on_click: T) -> Self {
        self.menu_items.push(MenuItem::Item {
            name: name.to_string(),
            event: on_click,
            disabled: false,
            icon: None,
        });
        self
    }

    pub fn with_checkable_item(mut self, name: &str, is_checked: bool, on_click: T) -> Self {
        self.menu_items.push(MenuItem::CheckableItem {
            name: name.to_string(),
            is_checked,
            event: on_click,
            disabled: false,
            icon: None,
        });
        self
    }

    pub fn with_child_menu(mut self, name: &str, menu: MenuBuilder<T>) -> Self {
        self.menu_items.push(MenuItem::ChildMenu {
            name: name.to_string(),
            children: menu,
            disabled: false,
            icon: None,
        });
        self
    }

    pub(crate) fn build(self) -> Result<sys::MenuSys<T>, Error> {
        sys::build_menu(self)
    }
}

#[derive(Debug, Clone)]
pub struct TrayIconBuilder<T>
where
    T: PartialEq + Clone + 'static,
{
    parent_hwnd: Option<u32>,
    icon: Result<Icon, Error>,
    width: Option<u32>,
    height: Option<u32>,
    menu: Option<MenuBuilder<T>>,
    on_click: Option<T>,
    on_double_click: Option<T>,
    on_right_click: Option<T>,
    sender: Option<TrayIconSender<T>>,
}

#[derive(Debug, Clone, Copy)]
pub enum Error {
    IconLoadingFailed,
    SenderMissing,
    IconMissing,
    OsError,
}

impl<T> TrayIconBuilder<T>
where
    T: PartialEq + Clone + 'static,
{
    #[allow(clippy::new_without_default)]
    pub fn new() -> TrayIconBuilder<T> {
        TrayIconBuilder {
            parent_hwnd: None,
            icon: Err(Error::IconMissing),
            width: None,
            height: None,
            menu: None,
            on_click: None,
            on_double_click: None,
            on_right_click: None,
            sender: None,
        }
    }

    pub fn with_sender(mut self, s: std::sync::mpsc::Sender<T>) -> Self {
        self.sender = Some(TrayIconSender::Std(s));
        self
    }

    #[cfg(feature = "winit")]
    pub fn with_sender_winit(mut self, s: winit::event_loop::EventLoopProxy<T>) -> Self {
        self.sender = Some(TrayIconSender::Winit(s));
        self
    }

    #[cfg(feature = "crossbeam-channel")]
    pub fn with_sender_crossbeam(mut self, s: crossbeam_channel::Sender<T>) -> Self {
        self.sender = Some(TrayIconSender::Crossbeam(s));
        self
    }

    pub fn with_click(mut self, event: T) -> Self {
        self.on_click = Some(event);
        self
    }

    pub fn with_double_click(mut self, event: T) -> Self {
        self.on_double_click = Some(event);
        self
    }

    pub fn with_right_click(mut self, event: T) -> Self {
        self.on_right_click = Some(event);
        self
    }

    pub fn with_parent_hwnd(mut self, hwnd: u32) -> Self {
        self.parent_hwnd = Some(hwnd);
        self
    }

    pub fn with_icon(mut self, icon: Icon) -> Self {
        self.icon = Ok(icon);
        self
    }

    pub fn with_icon_from_buffer(mut self, buffer: &'static [u8]) -> Self {
        self.icon = Icon::from_buffer(buffer, None, None);
        self
    }

    pub fn with_menu(mut self, menu: MenuBuilder<T>) -> Self
    where
        T: PartialEq + Clone + 'static,
    {
        self.menu = Some(menu);
        self
    }

    pub fn build(self) -> Result<Box<impl TrayIcon<T> + Send + Sync>, Error> {
        Ok(sys::build_trayicon(self)?)
    }
}

pub trait TrayIcon<T>
where
    T: PartialEq + Clone + 'static,
{
    /// Set the icon
    fn set_icon(&mut self, icon: &Icon) -> Result<(), Error>;

    /// Set the menu
    fn set_menu(&mut self, menu: MenuBuilder<T>) -> Result<(), Error>;

    // TODO: Maybe not implement these, instead use reactively set_menu
    // fn set_item_check(&mut self, event: T, is_checked: bool) -> Result<(), Error>;
    // fn set_item_disabled(&mut self, event: T, disabled: bool) -> Result<(), Error>;
}
