use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use zbus::object_server::SignalEmitter;
use zbus::zvariant::OwnedValue;
use zbus::zvariant::Type;
use zbus::zvariant::Value;
use zbus::Connection;

#[derive(Debug, Default, Type, Serialize, Deserialize, Value, OwnedValue)]
pub struct Layout {
    pub id: i32,
    pub properties: HashMap<String, OwnedValue>,
    pub children: Vec<OwnedValue>,
}

pub struct DbusMenu<T>
where
    T: crate::TrayIconEvent,
{
    menu_sys: super::super::MenuSys<T>,
}

impl<T> DbusMenu<T>
where
    T: crate::TrayIconEvent,
{
    pub fn new(menu_sys: super::super::MenuSys<T>) -> Self {
        DbusMenu { menu_sys }
    }

    fn build_layout_from_items(&self, items: &[super::super::MenuItemData<T>]) -> Vec<OwnedValue> {
        let mut children = vec![];

        for item in items {
            if item.is_separator {
                let mut properties = HashMap::new();
                properties.insert(
                    "type".to_string(),
                    OwnedValue::try_from(Value::new("separator")).unwrap(),
                );

                let layout = Layout {
                    id: item.id,
                    properties,
                    children: vec![],
                };
                children.push(OwnedValue::try_from(layout).unwrap());
            } else {
                let mut properties = HashMap::new();
                properties.insert(
                    "label".to_string(),
                    OwnedValue::try_from(Value::new(item.label.as_str())).unwrap(),
                );

                // Always set the enabled property explicitly
                properties.insert(
                    "enabled".to_string(),
                    OwnedValue::try_from(Value::new(!item.is_disabled)).unwrap(),
                );

                if item.is_checkable {
                    properties.insert(
                        "toggle-type".to_string(),
                        OwnedValue::try_from(Value::new("checkbox")).unwrap(),
                    );
                    properties.insert(
                        "toggle-state".to_string(),
                        OwnedValue::try_from(Value::new(if item.is_checked { 1i32 } else { 0i32 }))
                            .unwrap(),
                    );
                }

                let child_layouts = if !item.children.is_empty() {
                    properties.insert(
                        "children-display".to_string(),
                        OwnedValue::try_from(Value::new("submenu")).unwrap(),
                    );
                    self.build_layout_from_items(&item.children)
                } else {
                    vec![]
                };

                let layout = Layout {
                    id: item.id,
                    properties,
                    children: child_layouts,
                };
                children.push(OwnedValue::try_from(layout).unwrap());
            }
        }

        children
    }

    fn find_item_by_id<'a>(
        &self,
        id: i32,
        items: &'a [super::super::MenuItemData<T>],
    ) -> Option<&'a super::super::MenuItemData<T>> {
        for item in items {
            if item.id == id {
                return Some(item);
            }
            if !item.children.is_empty() {
                if let Some(found) = self.find_item_by_id(id, &item.children) {
                    return Some(found);
                }
            }
        }
        None
    }
}

#[zbus::interface(name = "com.canonical.dbusmenu")]
impl<T> DbusMenu<T>
where
    T: crate::TrayIconEvent,
{
    // methods
    async fn get_layout(
        &self,
        parent_id: i32,
        _recursion_depth: i32,
        _property_names: Vec<String>,
    ) -> zbus::fdo::Result<(u32, Layout)> {
        println!("get_layout called for parent_id {}", parent_id);

        if parent_id == 0 {
            // Root menu
            let children = self.build_layout_from_items(&self.menu_sys.items);

            Ok((
                0,
                Layout {
                    id: parent_id,
                    properties: HashMap::new(),
                    children,
                },
            ))
        } else {
            // Submenu
            if let Some(item) = self.find_item_by_id(parent_id, &self.menu_sys.items) {
                let children = self.build_layout_from_items(&item.children);

                Ok((
                    0,
                    Layout {
                        id: parent_id,
                        properties: HashMap::new(),
                        children,
                    },
                ))
            } else {
                Err(zbus::fdo::Error::InvalidArgs(
                    "parentId not found".to_string(),
                ))
            }
        }
    }

    async fn get_group_properties(
        &self,
        _ids: Vec<i32>,
        _property_names: Vec<String>,
    ) -> zbus::fdo::Result<Vec<(i32, HashMap<String, OwnedValue>)>> {
        Ok(Vec::new())
    }

    async fn get_property(&self, id: i32, name: String) -> zbus::fdo::Result<OwnedValue> {
        Err(zbus::fdo::Error::InvalidArgs(format!(
            "Property '{}' for id {} not found",
            name, id
        )))
    }

    async fn event(
        &self,
        #[zbus(connection)] _conn: &Connection,
        id: i32,
        event_id: String,
        _data: OwnedValue,
        _timestamp: u32,
    ) -> zbus::fdo::Result<()> {
        println!(
            "Event received for id {} event_id {} timestamp {}",
            id, event_id, _timestamp
        );

        // Handle clicked events
        if event_id == "clicked" {
            if let Some(item) = self.find_item_by_id(id, &self.menu_sys.items) {
                if let Some(event) = &item.event_id {
                    if let Some(tx) = &self.menu_sys.event_sender {
                        let _ = tx.send((id, event.clone()));
                    }
                }
            }
        }

        Ok(())
    }

    async fn event_group(
        &self,
        #[zbus(connection)] _conn: &Connection,
        _events: Vec<(i32, String, OwnedValue, u32)>,
    ) -> zbus::fdo::Result<Vec<i32>> {
        Ok(vec![])
    }

    async fn about_to_show(&self) -> zbus::fdo::Result<bool> {
        Ok(false)
    }

    async fn about_to_show_group(&self) -> zbus::fdo::Result<(Vec<i32>, Vec<i32>)> {
        Ok(Default::default())
    }

    // properties
    #[zbus(property)]
    fn version(&self) -> zbus::fdo::Result<u32> {
        Ok(3)
    }

    #[zbus(property)]
    async fn text_direction(&self) -> zbus::fdo::Result<String> {
        Ok("ltr".to_string())
    }

    #[zbus(property)]
    async fn status(&self) -> zbus::fdo::Result<String> {
        Ok("normal".to_string())
    }

    #[zbus(property)]
    async fn icon_theme_path(&self) -> zbus::fdo::Result<Vec<String>> {
        Ok(vec![])
    }

    // signals
    #[zbus(signal)]
    pub async fn items_properties_updated(
        ctxt: &SignalEmitter<'_>,
        updated_props: Vec<(i32, HashMap<String, OwnedValue>)>,
        removed_props: Vec<(i32, Vec<String>)>,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    pub async fn layout_updated(
        ctxt: &SignalEmitter<'_>,
        revision: u32,
        parent: i32,
    ) -> zbus::Result<()>;
}
