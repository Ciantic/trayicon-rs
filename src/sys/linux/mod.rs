use crate::{Error, MenuBuilder, MenuItem, TrayIconBuilder, TrayIconEvent};

mod dbus;
mod kdeicon;
mod kdetrayicon;
use dbus::*;
pub use kdeicon::KdeIcon as IconSys;
pub use kdetrayicon::KdeTrayIconImpl as TrayIconSys;

#[derive(Debug, Clone)]
pub struct MenuItemData<T: TrayIconEvent> {
    pub id: i32,
    pub label: String,
    pub event_id: Option<T>,
    pub is_separator: bool,
    pub is_checkable: bool,
    pub is_checked: bool,
    pub is_disabled: bool,
    pub children: Vec<MenuItemData<T>>,
}

#[derive(Clone, Debug)]
pub struct MenuSys<T>
where
    T: TrayIconEvent,
{
    pub(crate) items: Vec<MenuItemData<T>>,
    pub(crate) event_sender: Option<std::sync::mpsc::Sender<(i32, T)>>,
}

impl<T> MenuSys<T>
where
    T: TrayIconEvent,
{
    pub(crate) fn new() -> Result<MenuSys<T>, Error> {
        Ok(MenuSys {
            items: vec![],
            event_sender: None,
        })
    }
}

/// Build the tray icon
pub fn build_trayicon<T>(builder: &TrayIconBuilder<T>) -> Result<TrayIconSys<T>, Error>
where
    T: TrayIconEvent,
{
    let mut menu: Option<MenuSys<T>> = None;
    let tooltip = builder.tooltip.clone().unwrap_or_default();
    let title = builder
        .title
        .clone()
        .unwrap_or_else(|| "Application".to_string());
    let icon = builder.icon.as_ref()?;
    let on_click = builder.on_click.clone();
    let on_right_click = builder.on_right_click.clone();
    let sender = builder.sender.clone().ok_or(Error::SenderMissing)?;
    let on_double_click = builder.on_double_click.clone();
    // let notify_icon = WinNotifyIcon::new(hicon, tooltip);

    // Try to get a popup menu
    if let Some(rhmenu) = &builder.menu {
        let mut built_menu = rhmenu.build()?;

        // Set up event handling channel
        let (event_tx, event_rx) = std::sync::mpsc::channel::<(i32, T)>();
        let sender_clone = sender.clone();

        // Store the sender in MenuSys
        built_menu.event_sender = Some(event_tx.clone());

        // Spawn thread to handle menu events
        std::thread::spawn(move || {
            while let Ok((_menu_id, event)) = event_rx.recv() {
                sender_clone.send(&event);
            }
        });

        // Register the menu with DBus
        let connection = get_dbus_connection();
        register_dbus_menu_blocking(connection, built_menu.clone());

        menu = Some(built_menu);
    }

    Ok(TrayIconSys::new(
        sender,
        menu,
        Some(icon),
        tooltip,
        title,
        // notify_icon,
        on_click,
        on_double_click,
        on_right_click,
    )?)
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
    let mut menu_sys = MenuSys::new()?;

    for item in &builder.menu_items {
        let menu_item_data = convert_menu_item(j, item)?;
        menu_sys.items.push(menu_item_data);
    }

    Ok(menu_sys)
}

fn convert_menu_item<T>(j: &mut usize, item: &MenuItem<T>) -> Result<MenuItemData<T>, Error>
where
    T: TrayIconEvent,
{
    *j += 1;
    let current_id = *j as i32;

    match item {
        MenuItem::Separator => Ok(MenuItemData {
            id: current_id,
            label: String::new(),
            event_id: None,
            is_separator: true,
            is_checkable: false,
            is_checked: false,
            is_disabled: false,
            children: vec![],
        }),
        MenuItem::Item {
            id, name, disabled, ..
        } => Ok(MenuItemData {
            id: current_id,
            label: name.clone(),
            event_id: Some(id.clone()),
            is_separator: false,
            is_checkable: false,
            is_checked: false,
            is_disabled: *disabled,
            children: vec![],
        }),
        MenuItem::Checkable {
            id,
            name,
            is_checked,
            disabled,
            ..
        } => Ok(MenuItemData {
            id: current_id,
            label: name.clone(),
            event_id: Some(id.clone()),
            is_separator: false,
            is_checkable: true,
            is_checked: *is_checked,
            is_disabled: *disabled,
            children: vec![],
        }),
        MenuItem::Submenu {
            name,
            children,
            disabled,
            ..
        } => {
            let mut child_items = vec![];
            for child in &children.menu_items {
                let child_data = convert_menu_item(j, child)?;
                child_items.push(child_data);
            }
            Ok(MenuItemData {
                id: current_id,
                label: name.clone(),
                event_id: None,
                is_separator: false,
                is_checkable: false,
                is_checked: false,
                is_disabled: *disabled,
                children: child_items,
            })
        }
    }
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
            // Count items recursively
            fn count_items<T: TrayIconEvent>(items: &[MenuItemData<T>]) -> usize {
                let mut count = items.len();
                for item in items {
                    count += count_items(&item.children);
                }
                count
            }

            let total_items = count_items(&menusys.items);
            assert_eq!(total_items, 11);
        } else {
            panic!()
        }
    }
}
