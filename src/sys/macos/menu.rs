use crate::{trayiconsender::TrayIconSender, Error, MenuBuilder, MenuItem, TrayIconEvent};
use objc2::rc::{Allocated, Retained};
use objc2::runtime::Sel;
use objc2::{class, define_class, msg_send, DeclaredClass, MainThreadOnly};
use objc2_app_kit::{NSMenu, NSMenuItem};
use objc2_foundation::{MainThreadMarker, NSObject, NSString};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Menu target handler that receives menu item clicks
type MenuCallback = Box<dyn Fn(isize)>;

pub struct MenuTargetIvars {
    callback: RefCell<Option<MenuCallback>>,
    radio_groups: Arc<Mutex<HashMap<isize, Vec<isize>>>>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = MenuTargetIvars]
    #[derive(PartialEq, Eq, Hash)]
    pub struct MenuTarget;

    impl MenuTarget {
        #[unsafe(method(menuItemClicked:))]
        fn menu_item_clicked(&self, sender: &NSMenuItem) {
            let tag = sender.tag();
            let ivars = self.ivars();

            let radio_group = ivars
                .radio_groups
                .lock()
                .ok()
                .and_then(|groups| groups.get(&tag).cloned());
            if let (Some(group), Some(menu)) = (radio_group, unsafe { sender.menu() }) {
                for item_tag in group {
                    if let Some(item) = menu.itemWithTag(item_tag) {
                        let state = if item_tag == tag { 1_isize } else { 0_isize };
                        let _: () = unsafe { msg_send![&item, setState: state] };
                    }
                }
            }

            if let Some(ref callback) = *ivars.callback.borrow() {
                callback(tag);
            }
        }
    }
);

impl MenuTarget {
    fn new<T: TrayIconEvent>(
        sender: TrayIconSender<T>,
        menu_ids: Arc<Mutex<HashMap<isize, T>>>,
        radio_groups: Arc<Mutex<HashMap<isize, Vec<isize>>>>,
        menu_state: crate::SharedMenu<T>,
    ) -> Retained<Self> {
        let callback_radio_groups = radio_groups.clone();
        let callback: MenuCallback = Box::new(move |tag| {
            let event_id = menu_ids
                .lock()
                .ok()
                .and_then(|menu_ids| menu_ids.get(&tag).cloned());
            if let Some(event_id) = event_id {
                let is_radio = callback_radio_groups
                    .lock()
                    .is_ok_and(|groups| groups.contains_key(&tag));
                if is_radio {
                    if let Ok(mut menu) = menu_state.write() {
                        if let Some(menu) = menu.as_mut() {
                            let _ = menu.select_radio(&event_id);
                        }
                    }
                }
                sender.send(&event_id);
            }
        });

        let ivars = MenuTargetIvars {
            callback: RefCell::new(Some(callback)),
            radio_groups,
        };

        let mtm = unsafe { MainThreadMarker::new_unchecked() };
        let this = mtm.alloc().set_ivars(ivars);
        unsafe { msg_send![super(this), init] }
    }
}

impl Drop for MenuTarget {
    fn drop(&mut self) {
        // Clean up the callback by taking it from the RefCell
        let ivars = self.ivars();
        let _ = ivars.callback.borrow_mut().take(); // This will drop the Box<dyn Fn(isize)>
    }
}

pub struct MacMenu<T>
where
    T: TrayIconEvent,
{
    pub(crate) ids: HashMap<usize, T>,
    pub(crate) menu: Retained<NSMenu>,
    pub(crate) target: Retained<MenuTarget>,
    pub(crate) menu_ids: Arc<Mutex<HashMap<isize, T>>>,
    radio_groups: Arc<Mutex<HashMap<isize, Vec<isize>>>>,
    menu_state: crate::SharedMenu<T>,
}

/// Build the menu from MenuBuilder
pub fn build_menu<T>(
    builder: &MenuBuilder<T>,
    sender: &TrayIconSender<T>,
    menu_state: crate::SharedMenu<T>,
) -> Result<MacMenu<T>, Error>
where
    T: TrayIconEvent,
{
    let mut j = 0;
    let menu_ids = Arc::new(Mutex::new(HashMap::new()));
    let radio_groups = Arc::new(Mutex::new(HashMap::new()));
    let target = MenuTarget::new(
        sender.clone(),
        menu_ids.clone(),
        radio_groups.clone(),
        menu_state.clone(),
    );
    let result = build_menu_inner(
        &mut j,
        builder,
        &target,
        &menu_ids,
        &radio_groups,
        &menu_state,
    )?;

    Ok(MacMenu {
        ids: result.ids,
        menu: result.menu,
        target,
        menu_ids,
        radio_groups,
        menu_state,
    })
}

fn close_radio_group(run: &mut Vec<isize>, groups: &Arc<Mutex<HashMap<isize, Vec<isize>>>>) {
    if run.is_empty() {
        return;
    }
    let group = std::mem::take(run);
    if let Ok(mut groups) = groups.lock() {
        for tag in &group {
            groups.insert(*tag, group.clone());
        }
    }
}

/// Recursive menu builder
fn build_menu_inner<T>(
    j: &mut usize,
    builder: &MenuBuilder<T>,
    target: &Retained<MenuTarget>,
    menu_ids: &Arc<Mutex<HashMap<isize, T>>>,
    radio_groups: &Arc<Mutex<HashMap<isize, Vec<isize>>>>,
    menu_state: &crate::SharedMenu<T>,
) -> Result<MacMenu<T>, Error>
where
    T: TrayIconEvent,
{
    let mut map: HashMap<usize, T> = HashMap::new();
    let mtm = unsafe { MainThreadMarker::new_unchecked() };

    let menu = NSMenu::new(mtm);
    menu.setAutoenablesItems(false);

    let mut radio_run = Vec::new();

    for item in &builder.menu_items {
        if !matches!(item, MenuItem::Radio { .. }) {
            close_radio_group(&mut radio_run, radio_groups);
        }

        match item {
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

                let submenu_sys =
                    build_menu_inner(j, children, target, menu_ids, radio_groups, menu_state)?;
                map.extend(submenu_sys.ids);

                let ns_title = NSString::from_str(name);
                let empty_str = NSString::new();
                let menu_item = unsafe {
                    let allocated: Allocated<NSMenuItem> = msg_send![class!(NSMenuItem), alloc];
                    let menu_item: Retained<NSMenuItem> = msg_send![allocated,
                        initWithTitle: &*ns_title,
                        action: None::<Sel>,
                        keyEquivalent: &*empty_str
                    ];
                    menu_item
                };

                menu_item.setSubmenu(Some(&submenu_sys.menu));
                menu_item.setEnabled(!disabled);
                menu.addItem(&menu_item);
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

                let ns_title = NSString::from_str(name);
                let empty_str = NSString::new();
                let menu_item = unsafe {
                    let allocated: Allocated<NSMenuItem> = msg_send![class!(NSMenuItem), alloc];
                    let action_sel = Sel::register(c"menuItemClicked:");
                    let menu_item: Retained<NSMenuItem> = msg_send![allocated,
                        initWithTitle: &*ns_title,
                        action: Some(action_sel),
                        keyEquivalent: &*empty_str
                    ];
                    menu_item
                };

                unsafe {
                    menu_item.setTag(*j as isize);
                    menu_item.setTarget(Some(target));
                    menu_item.setEnabled(!disabled);
                    let _: () = msg_send![&menu_item, setState: if *is_checked { 1_isize } else { 0_isize }];
                    menu.addItem(&menu_item);
                }

                // Add to menu_ids mapping
                {
                    let mut menu_ids_lock = menu_ids.lock().unwrap();
                    menu_ids_lock.insert(*j as isize, id.clone());
                }
            }

            // macOS uses a checkmark for radio-style choices. The action target
            // updates the whole consecutive group before forwarding the event.
            MenuItem::Radio {
                name,
                is_checked,
                id,
                disabled,
                ..
            } => {
                *j += 1;
                map.insert(*j, id.clone());
                radio_run.push(*j as isize);

                let ns_title = NSString::from_str(name);
                let empty_str = NSString::new();
                let menu_item = unsafe {
                    let allocated: Allocated<NSMenuItem> = msg_send![class!(NSMenuItem), alloc];
                    let action_sel = Sel::register(c"menuItemClicked:");
                    let menu_item: Retained<NSMenuItem> = msg_send![allocated,
                        initWithTitle: &*ns_title,
                        action: Some(action_sel),
                        keyEquivalent: &*empty_str
                    ];
                    menu_item
                };

                unsafe {
                    menu_item.setTag(*j as isize);
                    menu_item.setTarget(Some(target));
                    menu_item.setEnabled(!disabled);
                    let _: () = msg_send![&menu_item, setState: if *is_checked { 1_isize } else { 0_isize }];
                    menu.addItem(&menu_item);
                }

                // Add to menu_ids mapping
                {
                    let mut menu_ids_lock = menu_ids.lock().unwrap();
                    menu_ids_lock.insert(*j as isize, id.clone());
                }
            }

            MenuItem::Item {
                name, id, disabled, ..
            } => {
                *j += 1;
                map.insert(*j, id.clone());

                let ns_title = NSString::from_str(name);
                let empty_str = NSString::new();
                let menu_item = unsafe {
                    let allocated: Allocated<NSMenuItem> = msg_send![class!(NSMenuItem), alloc];
                    let action_sel = Sel::register(c"menuItemClicked:");
                    let menu_item: Retained<NSMenuItem> = msg_send![allocated,
                        initWithTitle: &*ns_title,
                        action: Some(action_sel),
                        keyEquivalent: &*empty_str
                    ];
                    menu_item
                };

                unsafe {
                    menu_item.setTag(*j as isize);
                    menu_item.setTarget(Some(target));
                    menu_item.setEnabled(!disabled);
                    menu.addItem(&menu_item);
                }

                // Add to menu_ids mapping
                {
                    let mut menu_ids_lock = menu_ids.lock().unwrap();
                    menu_ids_lock.insert(*j as isize, id.clone());
                }
            }

            MenuItem::Separator => {
                let separator = NSMenuItem::separatorItem(mtm);
                menu.addItem(&separator);
            }
        }
    }

    close_radio_group(&mut radio_run, radio_groups);

    Ok(MacMenu {
        ids: map,
        menu,
        target: target.clone(),
        menu_ids: menu_ids.clone(),
        radio_groups: radio_groups.clone(),
        menu_state: menu_state.clone(),
    })
}

impl<T: TrayIconEvent> MacMenu<T> {
    /// Update the menu target with a new sender
    pub fn update_sender(&mut self, sender: &TrayIconSender<T>) {
        // Create new target with the real sender
        self.target = MenuTarget::new(
            sender.clone(),
            self.menu_ids.clone(),
            self.radio_groups.clone(),
            self.menu_state.clone(),
        );

        // Re-bind all menu items to the new target
        self.rebind_menu_items(&self.menu.clone());
    }

    fn rebind_menu_items(&self, menu: &NSMenu) {
        unsafe {
            let item_count = menu.numberOfItems();
            for i in 0..item_count {
                if let Some(item) = menu.itemAtIndex(i) {
                    // Only rebind items that have actions (not separators)
                    if item.action().is_some() {
                        item.setTarget(Some(&*self.target));
                    }

                    // Recursively rebind submenu items
                    if let Some(submenu) = item.submenu() {
                        self.rebind_menu_items(&submenu);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn radio_group_mapping_preserves_boundaries() {
        let groups = Arc::new(Mutex::new(HashMap::new()));
        let mut first_run = vec![3, 4];
        let mut second_run = vec![6];

        close_radio_group(&mut first_run, &groups);
        close_radio_group(&mut second_run, &groups);

        let groups = groups.lock().unwrap();
        assert_eq!(groups[&3], vec![3, 4]);
        assert_eq!(groups[&4], vec![3, 4]);
        assert_eq!(groups[&6], vec![6]);
        assert!(!groups.contains_key(&5));
    }
}
