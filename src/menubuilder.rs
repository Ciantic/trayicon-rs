use crate::{Error, Icon, TrayIconEvent};

#[derive(Debug, Clone, PartialEq)]
pub enum MenuItem<T>
where
    T: TrayIconEvent,
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
    /// A mutually-exclusive selectable item.
    ///
    /// Radio items behave like checkable items for state purposes (one is
    /// `is_checked`), but the tray host renders them as a radio-button group:
    /// selecting one clears the others in the same group. A group is the
    /// maximal run of consecutive `Radio` items within the same (sub)menu; a
    /// separator or any other item kind breaks the run.
    ///
    /// On platforms without a native radio glyph (macOS), a `Radio` item uses
    /// an ordinary checkmark while preserving the same exclusive behavior.
    Radio {
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
    T: TrayIconEvent,
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
    T: TrayIconEvent,
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

    /// Add a mutually-exclusive selectable (radio) item.
    ///
    /// Consecutive `radio` items within the same (sub)menu form a radio group.
    /// Only one item in a group should be `is_checked == true`. See
    /// [`MenuItem::Radio`] for the grouping and platform-degradation rules.
    pub fn radio(mut self, name: &str, is_checked: bool, id: T) -> Self {
        self.menu_items.push(MenuItem::Radio {
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

    #[allow(dead_code)]
    pub(crate) fn build(&self) -> Result<crate::MenuSys<T>, Error> {
        crate::build_menu(self)
    }

    /// Get checkable state, if found.
    ///
    /// Prefer maintaining proper application state instead of getting checkable
    /// state with this method.
    pub(crate) fn get_checkable(&self, find_id: &T) -> Option<bool> {
        match self.find_item(find_id)? {
            MenuItem::Checkable { is_checked, .. } | MenuItem::Radio { is_checked, .. } => {
                Some(*is_checked)
            }
            _ => None,
        }
    }

    /// Set checkable
    ///
    /// Prefer building a new menu instead of mutating it with this method.
    pub(crate) fn set_checkable(&mut self, id: &T, checked: bool) -> Result<(), Error> {
        let mut changed = Vec::new();
        Self::set_checkable_in(&mut self.menu_items, id, checked, &mut changed)
    }

    /// Select a radio item and return every item whose checked state changed.
    pub(crate) fn select_radio(&mut self, id: &T) -> Result<Vec<(T, bool)>, Error> {
        let mut changed = Vec::new();
        Self::set_checkable_in(&mut self.menu_items, id, true, &mut changed)?;
        Ok(changed)
    }

    /// Set disabled state
    ///
    /// Prefer building a new menu instead of mutating it with this method.
    pub(crate) fn set_disabled(&mut self, id: &T, disabled: bool) -> Result<(), Error> {
        match self.find_item_mut(id).ok_or(Error::MenuItemNotFound)? {
            MenuItem::Item { disabled: d, .. }
            | MenuItem::Checkable { disabled: d, .. }
            | MenuItem::Radio { disabled: d, .. }
            | MenuItem::Submenu { disabled: d, .. } => {
                *d = disabled;
                Ok(())
            }
            MenuItem::Separator => Err(Error::MenuItemNotFound),
        }
    }

    fn item_id(item: &MenuItem<T>) -> Option<&T> {
        match item {
            MenuItem::Item { id, .. }
            | MenuItem::Checkable { id, .. }
            | MenuItem::Radio { id, .. } => Some(id),
            MenuItem::Submenu { id, .. } => id.as_ref(),
            MenuItem::Separator => None,
        }
    }

    fn find_item(&self, id: &T) -> Option<&MenuItem<T>> {
        for item in &self.menu_items {
            if Self::item_id(item) == Some(id) {
                return Some(item);
            }
            if let MenuItem::Submenu { children, .. } = item {
                if let Some(found) = children.find_item(id) {
                    return Some(found);
                }
            }
        }
        None
    }

    fn find_item_mut(&mut self, id: &T) -> Option<&mut MenuItem<T>> {
        for item in &mut self.menu_items {
            if Self::item_id(item) == Some(id) {
                return Some(item);
            }
            if let MenuItem::Submenu { children, .. } = item {
                if let Some(found) = children.find_item_mut(id) {
                    return Some(found);
                }
            }
        }
        None
    }

    fn set_checkable_in(
        items: &mut [MenuItem<T>],
        id: &T,
        checked: bool,
        changed: &mut Vec<(T, bool)>,
    ) -> Result<(), Error> {
        if let Some(pos) = items
            .iter()
            .position(|item| Self::item_id(item) == Some(id))
        {
            match &mut items[pos] {
                MenuItem::Checkable { id, is_checked, .. } => {
                    if *is_checked != checked {
                        *is_checked = checked;
                        changed.push((id.clone(), checked));
                    }
                    return Ok(());
                }
                MenuItem::Radio { .. } => {
                    if !checked {
                        if let MenuItem::Radio { id, is_checked, .. } = &mut items[pos] {
                            if *is_checked {
                                *is_checked = false;
                                changed.push((id.clone(), false));
                            }
                        }
                        return Ok(());
                    }

                    let mut start = pos;
                    while start > 0 && matches!(items[start - 1], MenuItem::Radio { .. }) {
                        start -= 1;
                    }
                    let mut end = pos;
                    while end + 1 < items.len() && matches!(items[end + 1], MenuItem::Radio { .. })
                    {
                        end += 1;
                    }

                    for (index, item) in items.iter_mut().enumerate().take(end + 1).skip(start) {
                        if let MenuItem::Radio { id, is_checked, .. } = item {
                            let selected = index == pos;
                            if *is_checked != selected {
                                *is_checked = selected;
                                changed.push((id.clone(), selected));
                            }
                        }
                    }
                    return Ok(());
                }
                _ => return Err(Error::MenuItemNotFound),
            }
        }

        for item in items {
            if let MenuItem::Submenu { children, .. } = item {
                match Self::set_checkable_in(&mut children.menu_items, id, checked, changed) {
                    Ok(()) => return Ok(()),
                    Err(Error::MenuItemNotFound) => {}
                    Err(error) => return Err(error),
                }
            }
        }

        Err(Error::MenuItemNotFound)
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
        RadioA,
        RadioB,
        RadioC,
        RadioD,
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
        let _ = old.set_checkable(&Events::CheckItem1, true);
        let _ = old.set_disabled(&Events::DisabledItem1, true);
        let _ = old.set_checkable(&Events::CheckItem2, true);
        assert_eq!(old, menu_builder(true, true));
    }

    #[test]
    fn radio_selection_traverses_sibling_submenus_and_preserves_group_boundaries() {
        let mut menu = MenuBuilder::new()
            .submenu("First", MenuBuilder::new().item("Item", Events::SubItem1))
            .submenu(
                "Radio groups",
                MenuBuilder::new()
                    .radio("A", true, Events::RadioA)
                    .radio("B", false, Events::RadioB)
                    .separator()
                    .radio("C", true, Events::RadioC)
                    .radio("D", false, Events::RadioD),
            );

        menu.set_checkable(&Events::RadioB, true).unwrap();
        assert_eq!(menu.get_checkable(&Events::RadioA), Some(false));
        assert_eq!(menu.get_checkable(&Events::RadioB), Some(true));
        assert_eq!(menu.get_checkable(&Events::RadioC), Some(true));

        let changed = menu.select_radio(&Events::RadioD).unwrap();
        assert_eq!(
            changed,
            vec![(Events::RadioC, false), (Events::RadioD, true)]
        );
        assert_eq!(menu.get_checkable(&Events::RadioB), Some(true));
        assert_eq!(menu.get_checkable(&Events::RadioD), Some(true));
    }

    #[test]
    fn mutation_reports_missing_or_wrong_item_types() {
        let mut menu = MenuBuilder::new().item("Item", Events::Item1);
        assert_eq!(
            menu.set_checkable(&Events::Item1, true),
            Err(Error::MenuItemNotFound)
        );
        assert_eq!(
            menu.set_disabled(&Events::RadioA, true),
            Err(Error::MenuItemNotFound)
        );
    }
}
