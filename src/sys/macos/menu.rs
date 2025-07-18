use crate::{trayiconsender::TrayIconSender, Error, MenuBuilder, MenuItem};
use objc2::rc::{Allocated, Retained};
use objc2::runtime::Sel;
use objc2::{class, msg_send, define_class, DeclaredClass, MainThreadOnly};
use objc2_app_kit::{NSMenu, NSMenuItem};
use objc2_foundation::{MainThreadMarker, NSString, NSObject};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Menu target handler that receives menu item clicks

pub struct MenuTargetIvars {
    callback: RefCell<Option<Box<dyn Fn(isize)>>>,
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
            let tag = unsafe { sender.tag() };
            let ivars = self.ivars();
            if let Some(ref callback) = *ivars.callback.borrow() {
                callback(tag);
            }
        }
    }
);

impl MenuTarget {
    fn new<T: PartialEq + Clone + 'static>(
        sender: TrayIconSender<T>,
        menu_ids: Arc<Mutex<HashMap<isize, T>>>
    ) -> Retained<Self> {
        let callback: Box<dyn Fn(isize)> = Box::new(move |tag| {
            let menu_ids = menu_ids.lock().unwrap();
            if let Some(event_id) = menu_ids.get(&tag) {
                sender.send(event_id);
            }
        });

        let ivars = MenuTargetIvars {
            callback: RefCell::new(Some(callback)),
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
    T: PartialEq + Clone + 'static,
{
    pub(crate) ids: HashMap<usize, T>,
    pub(crate) menu: Retained<NSMenu>,
    pub(crate) target: Retained<MenuTarget>,
    pub(crate) menu_ids: Arc<Mutex<HashMap<isize, T>>>,
}

/// Build the menu from MenuBuilder
pub fn build_menu<T>(builder: &MenuBuilder<T>, sender: &TrayIconSender<T>) -> Result<MacMenu<T>, Error>
where
    T: PartialEq + Clone + 'static,
{
    let mut j = 0;
    let menu_ids = Arc::new(Mutex::new(HashMap::new()));
    let target = MenuTarget::new(sender.clone(), menu_ids.clone());
    let result = build_menu_inner(&mut j, builder, &target, &menu_ids)?;

    Ok(MacMenu {
        ids: result.ids,
        menu: result.menu,
        target,
        menu_ids,
    })
}

/// Recursive menu builder
fn build_menu_inner<T>(
    j: &mut usize,
    builder: &MenuBuilder<T>,
    target: &Retained<MenuTarget>,
    menu_ids: &Arc<Mutex<HashMap<isize, T>>>
) -> Result<MacMenu<T>, Error>
where
    T: PartialEq + Clone + 'static,
{
    let mut map: HashMap<usize, T> = HashMap::new();
    let mtm = unsafe { MainThreadMarker::new_unchecked() };

    let menu = NSMenu::new(mtm);
    unsafe {
        menu.setAutoenablesItems(false);
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
                if let Some(id) = id {
                    *j += 1;
                    map.insert(*j, id.clone());
                }

                if let Ok(submenu_sys) = build_menu_inner(j, children, target, menu_ids) {
                    map.extend(submenu_sys.ids.into_iter());

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

                    unsafe {
                        menu_item.setSubmenu(Some(&submenu_sys.menu));
                        menu_item.setEnabled(!disabled);
                        menu.addItem(&menu_item);
                    }
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

    Ok(MacMenu {
        ids: map,
        menu,
        target: target.clone(),
        menu_ids: menu_ids.clone()
    })
}

impl<T: PartialEq + Clone + 'static> MacMenu<T> {
    /// Update the menu target with a new sender
    pub fn update_sender(&mut self, sender: &TrayIconSender<T>) {
        // Create new target with the real sender
        self.target = MenuTarget::new(sender.clone(), self.menu_ids.clone());

        // Re-bind all menu items to the new target
        self.rebind_menu_items(&self.menu.clone());
    }

    fn rebind_menu_items(&self, menu: &NSMenu) {
        unsafe {
            let item_count = menu.numberOfItems();
            for i in 0..item_count {
                if let Some(item) = menu.itemAtIndex(i) {
                    // Only rebind items that have actions (not separators)
                    if !item.action().is_none() {
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
