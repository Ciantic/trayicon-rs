mod wchar;
mod winhicon;
mod winhmenu;
mod winnotifyicon;
mod wintrayicon;

use std::collections::HashMap;
use wintrayicon::WinTrayIconImpl;

use crate::{Error, MenuBuilder, MenuItem, TrayIconBuilder};
use winhmenu::WinHMenu;
use winnotifyicon::WinNotifyIcon;

// Windows implementations of Icon, TrayIcon, and Menu
pub use winhicon::WinHIcon as IconSys;
pub use wintrayicon::WinTrayIcon as TrayIconSys;

#[derive(Debug)]
pub struct MenuSys<T>
where
    T: PartialEq + Clone + 'static,
{
    ids: HashMap<usize, T>,
    menu: WinHMenu,
}

/// Build the tray icon
pub fn build_trayicon<T>(builder: &TrayIconBuilder<T>) -> Result<TrayIconSys<T>, Error>
where
    T: PartialEq + Clone + 'static,
{
    let mut menu: Option<MenuSys<T>> = None;
    let tooltip = &builder.tooltip;
    let hicon = &builder.icon.as_ref()?.sys;
    let on_click = builder.on_click.clone();
    let on_right_click = builder.on_right_click.clone();
    let sender = builder.sender.clone().ok_or(Error::SenderMissing)?;
    let on_double_click = builder.on_double_click.clone();
    let notify_icon = WinNotifyIcon::new(hicon, tooltip);

    // Try to get a popup menu
    if let Some(rhmenu) = &builder.menu {
        menu = Some(rhmenu.build()?);
    }

    Ok(WinTrayIconImpl::new(
        sender,
        menu,
        notify_icon,
        on_click,
        on_double_click,
        on_right_click,
    )?)
}

/// Build the menu from Windows HMENU
pub fn build_menu<T>(builder: &MenuBuilder<T>) -> Result<MenuSys<T>, Error>
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
fn build_menu_inner<T>(j: &mut usize, builder: &MenuBuilder<T>) -> Result<MenuSys<T>, Error>
where
    T: PartialEq + Clone + 'static,
{
    let mut hmenu = WinHMenu::new()?;
    let mut map: HashMap<usize, T> = HashMap::new();
    builder.menu_items.iter().for_each(|item| match item {
        MenuItem::Submenu {
            id,
            name,
            children,
            disabled,
            ..
        } => {
            if let Some(id) = id {
                *j += 1;
                map.insert(*j, id.clone());
            }
            if let Ok(menusys) = build_menu_inner(j, children) {
                map.extend(menusys.ids.into_iter());
                hmenu.add_child_menu(&name, menusys.menu, *disabled);
            }
        }

        MenuItem::Checkable {
            name,
            is_checked,
            id,
            disabled,
            ..
        } => {
            *j += 1;
            map.insert(*j, id.clone());
            hmenu.add_checkable_item(&name, *is_checked, *j, *disabled);
        }

        MenuItem::Item {
            name, id, disabled, ..
        } => {
            *j += 1;
            map.insert(*j, id.clone());
            hmenu.add_menu_item(&name, *j, *disabled);
        }

        MenuItem::Separator => {
            hmenu.add_separator();
        }
    });

    Ok(MenuSys {
        ids: map,
        menu: hmenu,
    })
}

// For pattern matching, these are in own mod
mod msgs {
    pub const WM_USER_TRAYICON: u32 = 0x400 + 1001;
    pub const WM_USER_SHOW_MENU: u32 = 0x400 + 1002;
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
        let cond = false;
        let builder = MenuBuilder::new()
            .checkable("This is checkable", true, Events::CheckableItem1)
            .submenu(
                "Sub Menu",
                MenuBuilder::new()
                    .item("Sub item 1", Events::SubItem1)
                    .item("Sub Item 2", Events::SubItem2)
                    .item("Sub Item 3", Events::SubItem3)
                    .submenu(
                        "Sub Sub menu",
                        MenuBuilder::new()
                            .item("Sub Sub item 1", Events::SubSubItem1)
                            .item("Sub Sub Item 2", Events::SubSubItem2)
                            .item("Sub Sub Item 3", Events::SubSubItem3),
                    )
                    .when(|f| {
                        if cond {
                            f.item("Foo", Events::Item1)
                        } else {
                            f
                        }
                    })
                    .item("Sub Item 4", Events::SubItem4),
            )
            .item("Item 1", Events::Item1);

        if let Ok(menusys) = build_menu(&builder) {
            assert_eq!(menusys.ids.len(), 9);
        } else {
            panic!()
        }
    }
}
