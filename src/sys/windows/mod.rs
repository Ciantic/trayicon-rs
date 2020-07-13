mod hicon;
mod hmenu;
mod notifyicon;
mod window;

use std::collections::HashMap;
use window::TrayIconWindow;

use crate::{Error, MenuBuilder, MenuItem, TrayIconBuilder};
use hicon::WinHIcon;
use hmenu::WinHMenu;
use notifyicon::NotifyIcon;

// Windows implementations of Icon, TrayIcon, and Menu
pub use hicon::WinHIcon as IconSys;
pub use window::TrayIconWindow as TrayIconSys;

#[derive(Debug)]
pub struct MenuSys<T>
where
    T: PartialEq + Clone + 'static,
{
    events: HashMap<usize, T>,
    menu: WinHMenu,
}

/// Build the tray icon
pub fn build_trayicon<T>(builder: TrayIconBuilder<T>) -> Result<Box<TrayIconWindow<T>>, Error>
where
    T: PartialEq + Clone + 'static,
{
    let mut menu: Option<MenuSys<T>> = None;
    let hicon: WinHIcon = builder.icon?.sys;
    let on_click = builder.on_click;
    let on_right_click = builder.on_right_click;
    let sender = builder.sender.ok_or(Error::SenderMissing)?;
    let on_double_click = builder.on_double_click;
    let notify_icon = NotifyIcon::new(hicon);

    // Try to get a popup menu
    if let Some(rhmenu) = builder.menu {
        menu = Some(rhmenu.build()?);
    }

    Ok(TrayIconWindow::new(
        sender,
        menu,
        notify_icon,
        on_click,
        on_double_click,
        on_right_click,
    )?)
}

/// Build the menu from Windows HMENU
pub fn build_menu<T>(builder: MenuBuilder<T>) -> Result<MenuSys<T>, Error>
where
    T: PartialEq + Clone + 'static,
{
    let mut j = 0;
    build_menu_inner(&mut j, builder)
}

/// Recursive menu builder
///
/// Having a j value as mutable reference it's capable of handling nested
/// submenus
fn build_menu_inner<T>(j: &mut usize, mut builder: MenuBuilder<T>) -> Result<MenuSys<T>, Error>
where
    T: PartialEq + Clone + 'static,
{
    let mut hmenu = WinHMenu::new();
    let mut map: HashMap<usize, T> = HashMap::new();
    builder.menu_items.drain(0..).for_each(|item| match item {
        MenuItem::ChildMenu {
            name,
            children,
            disabled,
            ..
        } => {
            if let Ok(menusys) = build_menu_inner(j, children) {
                map.extend(menusys.events.into_iter());
                hmenu.add_child_menu(&name, menusys.menu, disabled);
            }
        }
        MenuItem::CheckableItem {
            name,
            is_checked,
            event,
            disabled,
            ..
        } => {
            *j += 1;
            map.insert(*j, event);
            hmenu.add_checkable_item(&name, is_checked, *j, disabled);
        }
        MenuItem::Item {
            name,
            event,
            disabled,
            ..
        } => {
            *j += 1;
            map.insert(*j, event);
            hmenu.add_menu_item(&name, *j, disabled);
        }
        MenuItem::Separator => hmenu.add_separator(),
    });

    Ok(MenuSys {
        events: map,
        menu: hmenu,
    })
}

// For pattern matching, these are in own mod
mod msgs {
    pub const WM_USER_CREATE: u32 = 0x400 + 1000;
    pub const WM_USER_TRAYICON: u32 = 0x400 + 1001;
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    enum Events {
        CheckableItem1,
        Item1,
        SubItem1,
        SubItem2,
        SubItem3,
        SubItem4,
        SubSubItem1,
        SubSubItem2,
        SubSubItem3,
    }

    #[test]
    fn test_menu_build() {
        let builder = MenuBuilder::new()
            .with_checkable_item("This is checkable", true, Events::CheckableItem1)
            .with_child_menu(
                "Sub Menu",
                MenuBuilder::new()
                    .with_item("Sub item 1", Events::SubItem1)
                    .with_item("Sub Item 2", Events::SubItem2)
                    .with_item("Sub Item 3", Events::SubItem3)
                    .with_child_menu(
                        "Sub Sub menu",
                        MenuBuilder::new()
                            .with_item("Sub Sub item 1", Events::SubSubItem1)
                            .with_item("Sub Sub Item 2", Events::SubSubItem2)
                            .with_item("Sub Sub Item 3", Events::SubSubItem3),
                    )
                    .with_item("Sub Item 4", Events::SubItem4),
            )
            .with_item("Item 1", Events::Item1);

        if let Ok(menusys) = build_menu(builder) {
            assert_eq!(menusys.events.len(), 9);
        } else {
            panic!()
        }
    }
}
