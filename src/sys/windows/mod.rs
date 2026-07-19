mod wchar;
mod winhicon;
mod winhmenu;
mod winnotifyicon;
mod wintrayicon;

use std::collections::HashMap;
use wintrayicon::WinTrayIconImpl;

use crate::{Error, MenuBuilder, MenuItem, TrayIconBuilder, TrayIconEvent};
use winhmenu::WinHMenu;
use winnotifyicon::WinNotifyIcon;

// Windows implementations of Icon, TrayIcon, and Menu
pub use winhicon::WinHIcon as IconSys;
pub use wintrayicon::WinTrayIcon as TrayIconSys;

/// A radio group: the submenu that owns the items plus the inclusive command-id
/// range of the consecutive radio run. `CheckMenuRadioItem` is invoked with
/// `MF_BYCOMMAND` over `[first, last]` to enforce a single selection.
#[derive(Debug, Clone, Copy)]
struct RadioGroup {
    hmenu: usize,
    first: u32,
    last: u32,
}

#[derive(Debug)]
pub struct MenuSys<T>
where
    T: TrayIconEvent,
{
    ids: HashMap<usize, T>,
    menu: WinHMenu,
    /// Command id of every radio item -> the group it belongs to. Used on
    /// `WM_COMMAND` to apply native exclusivity via `CheckMenuRadioItem`.
    radio: HashMap<usize, RadioGroup>,
}

/// Build the tray icon
pub fn build_trayicon<T>(
    builder: &TrayIconBuilder<T>,
    menu_state: crate::SharedMenu<T>,
) -> Result<TrayIconSys<T>, Error>
where
    T: TrayIconEvent,
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
    let initial_menu = menu_state.read().map_err(|_| Error::OsError)?.clone();
    if let Some(rhmenu) = &initial_menu {
        menu = Some(rhmenu.build()?);
    }

    WinTrayIconImpl::new(
        sender,
        menu,
        notify_icon,
        on_click,
        on_double_click,
        on_right_click,
        menu_state,
    )
}

/// Build the menu from Windows HMENU
pub fn build_menu<T>(builder: &MenuBuilder<T>) -> Result<MenuSys<T>, Error>
where
    T: TrayIconEvent,
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
    T: TrayIconEvent,
{
    let mut hmenu = WinHMenu::new()?;
    let mut map: HashMap<usize, T> = HashMap::new();
    let mut radio: HashMap<usize, RadioGroup> = HashMap::new();
    // Current consecutive radio run, if any: its first and last assigned
    // command id. A non-radio item (or separator) closes the run and records
    // the group.
    let mut run_first: Option<usize> = None;
    let mut run_last: Option<usize> = None;
    // HMENU address for the run being accumulated — the current (sub)menu.
    let hmenu_addr = hmenu.handle() as usize;

    // Close the in-progress radio run, recording its group for every command
    // id it covers (which are consecutive: `j` advances by exactly one per
    // radio item, and separators consume no id). Defined as a nested `fn` so
    // it captures nothing and keeps the surrounding loop simple.
    fn close_run(
        run_first: &mut Option<usize>,
        run_last: &mut Option<usize>,
        radio: &mut HashMap<usize, RadioGroup>,
        hmenu_addr: usize,
    ) {
        if let (Some(first), Some(last)) = (*run_first, *run_last) {
            let group = RadioGroup {
                hmenu: hmenu_addr,
                first: first as u32,
                last: last as u32,
            };
            for cmd in first..=last {
                radio.insert(cmd, group);
            }
        }
        *run_first = None;
        *run_last = None;
    }

    for item in &builder.menu_items {
        match item {
            MenuItem::Submenu {
                id,
                name,
                children,
                disabled,
                ..
            } => {
                close_run(&mut run_first, &mut run_last, &mut radio, hmenu_addr);
                if let Some(id) = id {
                    *j += 1;
                    map.insert(*j, id.clone());
                }
                let menusys = build_menu_inner(j, children)?;
                map.extend(menusys.ids);
                radio.extend(menusys.radio);
                hmenu.add_child_menu(name, menusys.menu, *disabled);
            }

            MenuItem::Checkable {
                name,
                is_checked,
                id,
                disabled,
                ..
            } => {
                close_run(&mut run_first, &mut run_last, &mut radio, hmenu_addr);
                *j += 1;
                map.insert(*j, id.clone());
                hmenu.add_checkable_item(name, *is_checked, *j, *disabled);
            }

            MenuItem::Radio {
                name,
                is_checked,
                id,
                disabled,
                ..
            } => {
                *j += 1;
                let cmd = *j;
                map.insert(cmd, id.clone());
                hmenu.add_radio_item(name, *is_checked, cmd, *disabled);
                if run_first.is_none() {
                    run_first = Some(cmd);
                }
                run_last = Some(cmd);
            }

            MenuItem::Item {
                name, id, disabled, ..
            } => {
                close_run(&mut run_first, &mut run_last, &mut radio, hmenu_addr);
                *j += 1;
                map.insert(*j, id.clone());
                hmenu.add_menu_item(name, *j, *disabled);
            }

            MenuItem::Separator => {
                close_run(&mut run_first, &mut run_last, &mut radio, hmenu_addr);
                hmenu.add_separator();
            }
        }
    }

    // Close a run that runs to the end of the (sub)menu.
    close_run(&mut run_first, &mut run_last, &mut radio, hmenu_addr);

    Ok(MenuSys {
        ids: map,
        menu: hmenu,
        radio,
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
        RadioA,
        RadioB,
        RadioC,
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

    #[test]
    fn radio_groups_follow_consecutive_runs() {
        let builder = MenuBuilder::new().submenu(
            "Radio groups",
            MenuBuilder::new()
                .radio("A", true, Events::RadioA)
                .radio("B", false, Events::RadioB)
                .separator()
                .radio("C", true, Events::RadioC),
        );
        let menu = build_menu(&builder).unwrap();

        let id_for = |event| {
            *menu
                .ids
                .iter()
                .find_map(|(id, candidate)| (candidate == &event).then_some(id))
                .unwrap()
        };
        let a = menu.radio[&id_for(Events::RadioA)];
        let b = menu.radio[&id_for(Events::RadioB)];
        let c = menu.radio[&id_for(Events::RadioC)];

        assert_eq!((a.hmenu, a.first, a.last), (b.hmenu, b.first, b.last));
        assert_ne!((a.hmenu, a.first, a.last), (c.hmenu, c.first, c.last));
        assert_eq!(c.first, c.last);
    }
}
