use crate::{Error, Icon};

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
    pub(crate) menu_items: Vec<MenuItem<T>>,
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

    pub(crate) fn build(&self) -> Result<crate::MenuSys<T>, Error> {
        crate::build_menu(self)
    }

    /// Get checkable state, if found.
    ///
    /// Prefer maintaining proper application state instead of getting checkable
    /// state with this method.
    pub(crate) fn get_checkable(&mut self, find_id: T) -> Option<bool> {
        let mut found_item = None;
        let _ = self.mutate_item(find_id, |i| {
            if let MenuItem::Checkable { is_checked, .. } = i {
                found_item = Some(*is_checked);
                Ok(())
            } else {
                Err(Error::MenuItemNotFound)
            }
        });
        found_item
    }

    /// Set checkable
    ///
    /// Prefer building a new menu instead of mutating it with this method.
    pub(crate) fn set_checkable(&mut self, id: T, checked: bool) -> Result<(), Error> {
        self.mutate_item(id, |i| {
            if let MenuItem::Checkable { is_checked, .. } = i {
                *is_checked = checked;
                Ok(())
            } else {
                Err(Error::MenuItemNotFound)
            }
        })
    }

    /// Set disabled state
    ///
    /// Prefer building a new menu instead of mutating it with this method.
    pub(crate) fn set_disabled(&mut self, id: T, disabled: bool) -> Result<(), Error> {
        self.mutate_item(id, |i| match i {
            MenuItem::Item { disabled: d, .. } => {
                *d = disabled;
                Ok(())
            }
            MenuItem::Checkable { disabled: d, .. } => {
                *d = disabled;
                Ok(())
            }
            MenuItem::Submenu { disabled: d, .. } => {
                *d = disabled;
                Ok(())
            }
            MenuItem::Separator => Err(Error::MenuItemNotFound),
        })
    }

    /// Find item and optionally mutate
    ///
    /// Recursively searches for item with id, and applies function f to item if
    /// found. There is no recursion depth limitation and may cause stack
    /// issues.
    fn mutate_item<F>(&mut self, id: T, f: F) -> Result<(), Error>
    where
        F: FnOnce(&mut MenuItem<T>) -> Result<(), Error>,
    {
        self._mutate_item_recurse_ref(id, f)
    }

    fn _mutate_item_recurse_ref<F>(&mut self, find_id: T, f: F) -> Result<(), Error>
    where
        F: FnOnce(&mut MenuItem<T>) -> Result<(), Error>,
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
            // Try to find submenu
            let maybe_found_submenu = self
                .menu_items
                .iter_mut()
                .find(|i| matches!(i, MenuItem::Submenu { .. }));

            // Reurse
            if let Some(MenuItem::Submenu { children, .. }) = maybe_found_submenu {
                return children._mutate_item_recurse_ref(find_id, f);
            }

            Err(Error::MenuItemNotFound)
        }
    }
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
