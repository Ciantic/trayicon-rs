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

#[derive(Clone)]
pub struct Icon {
    buffer: Option<&'static [u8]>,
    sys: sys::IconSys,
}

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
        Ok(Icon {
            buffer: Some(buffer),
            sys: sys::IconSys::from_buffer(buffer, width, height)?,
        })
    }
}

impl PartialEq for Icon {
    fn eq(&self, other: &Self) -> bool {
        self.buffer == other.buffer
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MenuItem<T>
where
    T: PartialEq + Clone + 'static,
{
    Separator,
    Item {
        id: T,
        name: String,
        disabled: bool,
        icon: Option<Icon>,
    },
    Checkable {
        id: T,
        name: String,
        is_checked: bool,
        disabled: bool,
        icon: Option<Icon>,
    },
    Submenu {
        id: Option<T>,
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

/// Menu Builder
///
/// This is defined as consuming builder, could be converted to non-consuming
/// one. This builder includes conditional helper `when` for composing
/// conditionally some items.
impl<T> MenuBuilder<T>
where
    T: PartialEq + Clone + 'static,
{
    pub fn new() -> MenuBuilder<T> {
        MenuBuilder { menu_items: vec![] }
    }

    /// Conditionally include items, poor mans function composition
    pub fn when<F>(self, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        f(self)
    }

    pub fn with(mut self, item: MenuItem<T>) -> Self {
        self.menu_items.push(item);
        self
    }

    pub fn separator(mut self) -> Self {
        self.menu_items.push(MenuItem::Separator);
        self
    }

    pub fn item(mut self, name: &str, id: T) -> Self {
        self.menu_items.push(MenuItem::Item {
            id,
            name: name.to_string(),
            disabled: false,
            icon: None,
        });
        self
    }

    pub fn checkable(mut self, name: &str, is_checked: bool, id: T) -> Self {
        self.menu_items.push(MenuItem::Checkable {
            id,
            name: name.to_string(),
            is_checked,
            disabled: false,
            icon: None,
        });
        self
    }

    pub fn submenu(mut self, name: &str, menu: MenuBuilder<T>) -> Self {
        self.menu_items.push(MenuItem::Submenu {
            id: None,
            name: name.to_string(),
            children: menu,
            disabled: false,
            icon: None,
        });
        self
    }

    pub(crate) fn build(&self) -> Result<crate::sys::MenuSys<T>, Error> {
        sys::build_menu(self)
    }

    /// Get checkable state, if found.
    ///
    /// Prefer maintaining proper application state instead of getting checkable
    /// state with this method.
    fn get_checkable(&mut self, find_id: T) -> Option<bool> {
        let mut found_item = None;
        self.mutate_item(find_id, |i| {
            if let MenuItem::Checkable { is_checked, .. } = i {
                found_item = Some(*is_checked);
            }
        });
        found_item
    }

    /// Set checkable
    ///
    /// Prefer building a new menu instead of mutating it with this method.
    fn set_checkable(&mut self, id: T, checked: bool) -> Result<(), Error> {
        self.mutate_item(id, |i| {
            if let MenuItem::Checkable { is_checked, .. } = i {
                *is_checked = checked
            }
        });
        Ok(())
    }

    /// Set disabled state
    ///
    /// Prefer building a new menu instead of mutating it with this method.
    fn set_disabled(&mut self, id: T, disabled: bool) -> Result<(), Error> {
        self.mutate_item(id, |i| match i {
            MenuItem::Item { disabled: d, .. } => *d = disabled,
            MenuItem::Checkable { disabled: d, .. } => *d = disabled,
            MenuItem::Submenu { disabled: d, .. } => *d = disabled,
            MenuItem::Separator => (),
        });
        Ok(())
    }

    /// Find item and optionally mutate
    ///
    /// Recursively searches for item with id, and applies function f to item if
    /// found. There is no recursion depth limitation and may cause stack
    /// issues.
    fn mutate_item<F>(&mut self, id: T, f: F)
    where
        F: FnOnce(&mut MenuItem<T>) -> (),
    {
        self._mutate_item_recurse_ref(id, f);
    }

    fn _mutate_item_recurse_ref<F>(&mut self, find_id: T, f: F)
    where
        F: FnOnce(&mut MenuItem<T>) -> (),
    {
        let found_item = self.menu_items.iter_mut().find(|f| match f {
            MenuItem::Item { id, .. } if id == &find_id => true,
            MenuItem::Checkable { id, .. } if id == &find_id => true,
            MenuItem::Submenu { id, .. } if id.as_ref() == Some(&find_id) => true,
            _ => false,
        });

        if let Some(item) = found_item {
            f(item)
        } else {
            // Try to recurse, if submenus exist
            let maybe_found_submenu = self.menu_items.iter_mut().find(|i| match i {
                MenuItem::Submenu { .. } => true,
                _ => false,
            });
            if let Some(found_submenu) = maybe_found_submenu {
                if let MenuItem::Submenu { children, .. } = found_submenu {
                    children._mutate_item_recurse_ref(find_id, f)
                }
            }
        }
    }

    // Following is too functional programmery way, Rust isn't that immutable
    //
    // fn mutate_item<F>(mut self, id: T, f: &F) -> Self
    // where
    //     F: FnOnce(MenuItem<T>) -> MenuItem<T>,
    // {
    //     MenuBuilder::new_from_items(self._mutate_item_recurse(id, f))
    // }

    // fn _mutate_item_recurse<F>(mut self, id: T, f: &F) -> Vec<MenuItem<T>>
    // where
    //     F: FnOnce(MenuItem<T>) -> MenuItem<T>,
    // {
    //     self.menu_items
    //         .drain(..)
    //         .map(move |mut item| match item {
    //             MenuItem::Submenu {
    //                 id: submenu_id,
    //                 children,
    //                 disabled,
    //                 icon,
    //                 name,
    //             } => MenuItem::Submenu {
    //                 id: submenu_id,
    //                 children: MenuBuilder::new_from_items(
    //                     children._mutate_item_recurse(id.clone(), f),
    //                 ),
    //                 disabled,
    //                 icon,
    //                 name,
    //             },
    //             e => e,
    //         })
    //         .collect()
    // }
}

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
    icon: Result<Icon, Error>,
    width: Option<u32>,
    height: Option<u32>,
    menu: Option<MenuBuilder<T>>,
    tooltip: Option<String>,
    on_click: Option<T>,
    on_double_click: Option<T>,
    on_right_click: Option<T>,
    sender: Option<TrayIconSender<T>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Error {
    IconLoadingFailed,
    SenderMissing,
    IconMissing,
    OsError,
}

impl From<&Error> for Error {
    fn from(e: &Error) -> Self {
        *e
    }
}

impl<T> TrayIconBuilder<T>
where
    T: PartialEq + Clone + 'static,
{
    #[allow(clippy::new_without_default)]
    pub fn new() -> TrayIconBuilder<T> {
        TrayIconBuilder {
            icon: Err(Error::IconMissing),
            width: None,
            height: None,
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

    pub fn sender(mut self, s: std::sync::mpsc::Sender<T>) -> Self {
        self.sender = Some(TrayIconSender::Std(s));
        self
    }

    /// Optional feature, requires `winit` feature
    #[cfg(feature = "winit")]
    pub fn sender_winit(mut self, s: winit::event_loop::EventLoopProxy<T>) -> Self {
        self.sender = Some(TrayIconSender::Winit(s));
        self
    }

    /// Optional feature, requires `crossbeam-channel` feature
    #[cfg(feature = "crossbeam-channel")]
    pub fn sender_crossbeam(mut self, s: crossbeam_channel::Sender<T>) -> Self {
        self.sender = Some(TrayIconSender::Crossbeam(s));
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
        Ok(TrayIcon::new(crate::sys::build_trayicon(&self)?, self))
    }
}

pub struct TrayIcon<T>
where
    T: PartialEq + Clone + 'static,
{
    sys: Box<crate::sys::TrayIconSys<T>>,
    builder: TrayIconBuilder<T>,
}

impl<T> TrayIcon<T>
where
    T: PartialEq + Clone + 'static,
{
    pub(crate) fn new(
        sys: Box<crate::sys::TrayIconSys<T>>,
        builder: TrayIconBuilder<T>,
    ) -> TrayIcon<T> {
        TrayIcon { builder, sys }
    }

    /// Set the icon if changed
    pub fn set_icon(&mut self, icon: &Icon) -> Result<(), Error> {
        if self.builder.icon.as_ref() == Ok(icon) {
            return Ok(());
        }
        self.builder.icon = Ok(icon.clone());
        self.sys.set_icon(icon)
    }

    /// Set the menu if changed
    ///
    /// This can be used reactively, each time the application state changes,
    /// build a new menu and set it with this method. This way one can avoid
    /// using more imperative `set_item_checkable`, `get_item_checkable` and
    /// `set_item_disabled` methods.
    pub fn set_menu(&mut self, menu: &MenuBuilder<T>) -> Result<(), Error> {
        if self.builder.menu.as_ref() == Some(menu) {
            return Ok(());
        }
        self.builder.menu = Some(menu.clone());
        self.sys.set_menu(menu)
    }

    /// Set the tooltip if changed
    pub fn set_tooltip(&mut self, tooltip: &str) -> Result<(), Error> {
        if self.builder.tooltip.as_deref() == Some(tooltip) {
            return Ok(());
        }
        self.builder.tooltip = Some(tooltip.to_string());
        self.sys.set_tooltip(tooltip)
    }

    /// Set disabled
    ///
    /// Prefer building a new menu if application state changes instead of
    /// mutating a menu with this method. Suggestion is to use just `set_menu`
    /// method instead of this.
    pub fn set_item_disabled(&mut self, id: T, disabled: bool) -> Result<(), Error> {
        if let Some(menu) = self.builder.menu.as_mut() {
            let _ = menu.set_disabled(id, disabled);
            let _ = self.sys.set_menu(menu);
        }
        Ok(())
    }

    /// Set checkable
    ///
    /// Prefer building a new menu when application state changes instead of
    /// mutating a menu with this method.  Suggestion is to use just `set_menu`
    /// method instead of this.
    pub fn set_item_checkable(&mut self, id: T, checked: bool) -> Result<(), Error> {
        if let Some(menu) = self.builder.menu.as_mut() {
            let _ = menu.set_checkable(id, checked);
            let _ = self.sys.set_menu(menu);
        }
        Ok(())
    }

    /// Get checkable state
    ///
    /// Prefer maintaining proper application state instead of getting checkable
    /// state with this method. Suggestion is to use just `set_menu` method
    /// instead of this.
    pub fn get_item_checkable(&mut self, id: T) -> Option<bool> {
        if let Some(menu) = self.builder.menu.as_mut() {
            menu.get_checkable(id)
        } else {
            None
        }
    }
}

unsafe impl<T> Sync for TrayIcon<T> where T: PartialEq + Clone + 'static {}

unsafe impl<T> Send for TrayIcon<T> where T: PartialEq + Clone + 'static {}

/// This is just helper for Sys packages, not an enforcement through generics
pub(crate) trait TrayIconBase<T>
where
    T: PartialEq + Clone + 'static,
{
    fn set_icon(&mut self, icon: &Icon) -> Result<(), Error>;
    fn set_menu(&mut self, menu: &MenuBuilder<T>) -> Result<(), Error>;
    fn set_tooltip(&mut self, tooltip: &str) -> Result<(), Error>;
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    enum Events {
        Item1,
        Item2,
        Item3,
        Item4,
        DisabledItem1,
        CheckItem1,
        CheckItem2,
        SubItem1,
        SubItem2,
        SubItem3,
    }

    #[test]
    fn test_menu_mutation() {
        // This is a good way to create menu conditionally on application state, define a function "State -> Menu"
        let menu_builder = |checked, disabled| {
            MenuBuilder::new()
                .item("Item 4 Set Tooltip", Events::Item4)
                .item("Item 3 Replace Menu 👍", Events::Item3)
                .item("Item 2 Change Icon Green", Events::Item2)
                .item("Item 1 Change Icon Red", Events::Item1)
                .separator()
                .checkable("This is checkable", checked, Events::CheckItem1)
                .submenu(
                    "Sub Menu",
                    MenuBuilder::new()
                        .item("Sub item 1", Events::SubItem1)
                        .item("Sub Item 2", Events::SubItem2)
                        .checkable("This is checkable", checked, Events::CheckItem2)
                        .item("Sub Item 3", Events::SubItem3),
                )
                .with(MenuItem::Item {
                    name: "Item Disabled".into(),
                    disabled,
                    id: Events::DisabledItem1,
                    icon: None,
                })
        };

        let mut old = menu_builder(false, false);
        let _ = old.set_checkable(Events::CheckItem1, true);
        let _ = old.set_disabled(Events::DisabledItem1, true);
        let _ = old.set_checkable(Events::CheckItem2, true);
        assert_eq!(old, menu_builder(true, true));
    }
}
