use crate::{
    Error, Icon, MenuBuilder, TrayIconBase, TrayIconBuilder, TrayIconEvent, TrayIconStatus,
};

pub struct TrayIcon<T>
where
    T: TrayIconEvent,
{
    sys: crate::TrayIconSys<T>,
    builder: TrayIconBuilder<T>,
}

impl<T> TrayIcon<T>
where
    T: TrayIconEvent,
{
    pub(crate) fn new(sys: crate::TrayIconSys<T>, builder: TrayIconBuilder<T>) -> TrayIcon<T> {
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
    pub fn set_menu_item_disabled(&mut self, id: T, disabled: bool) -> Result<(), Error> {
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
    pub fn set_menu_item_checkable(&mut self, id: T, checked: bool) -> Result<(), Error> {
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
    pub fn get_menu_item_checkable(&mut self, id: T) -> Option<bool> {
        if let Some(menu) = self.builder.menu.as_mut() {
            menu.get_checkable(id)
        } else {
            None
        }
    }

    /// Show the menu (Windows only)
    ///
    /// On KDE and MacOS right click by default opens the menu, there is no programmatic way to open it.
    pub fn show_menu(&mut self) -> Result<(), Error> {
        self.sys.show_menu()
    }

    /// Set the status of the tray icon (KDE only)
    ///
    /// On KDE, this controls the StatusNotifierItem status:
    /// - Active: Normal visible state
    /// - NeedsAttention: Icon blinks/animates to draw attention
    /// - Passive: Icon is hidden or minimized
    ///
    /// On other platforms, this does nothing by default.
    pub fn set_status(&mut self, status: TrayIconStatus) -> Result<(), Error> {
        self.sys.set_status(status)
    }

    /// Get the XDG activation token (KDE only)
    pub fn get_xdg_activation_token(&self) -> Option<String> {
        self.sys.get_xdg_activation_token()
    }
}

unsafe impl<T> Sync for TrayIcon<T> where T: TrayIconEvent {}

unsafe impl<T> Send for TrayIcon<T> where T: TrayIconEvent {}
