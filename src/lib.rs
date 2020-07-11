use std::{fmt::Debug, sync::mpsc::Sender};

#[cfg(target_os = "windows")]
#[path = "./sys/windows/mod.rs"]
mod sys;

pub(crate) type TrayIconSender<T>
where
    T: PartialEq + Clone,
= std::sync::mpsc::Sender<T>;

#[derive(Clone)]
pub struct Icon(sys::IconSys);

impl Debug for Icon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Icon")
    }
}

impl Icon {
    pub fn from_buffer(
        buffer: &'static [u8],
        width: Option<u32>,
        height: Option<u32>,
    ) -> Result<Icon, Error> {
        sys::IconSys::from_buffer(buffer, width, height).map(Icon)
    }
}

#[derive(Debug)]
pub enum MenuItem<T>
where
    T: PartialEq + Clone,
{
    Separator,
    Item {
        name: String,
        event: T,
        disabled: bool,
        icon: Option<Icon>,
    },
    CheckableItem {
        name: String,
        is_checked: bool,
        event: T,
        disabled: bool,
        icon: Option<Icon>,
    },
    ChildMenu {
        name: String,
        children: MenuBuilder<T>,
        disabled: bool,
        icon: Option<Icon>,
    },
}

#[derive(Debug, Default)]
pub struct MenuBuilder<T>
where
    T: PartialEq + Clone,
{
    menu_items: Vec<MenuItem<T>>,
}

impl<T> MenuBuilder<T>
where
    T: PartialEq + Clone,
{
    pub fn new() -> MenuBuilder<T> {
        MenuBuilder { menu_items: vec![] }
    }

    pub fn with(mut self, item: MenuItem<T>) -> Self {
        self.menu_items.push(item);
        self
    }

    pub fn with_separator(mut self) -> Self {
        self.menu_items.push(MenuItem::Separator);
        self
    }

    pub fn with_item(mut self, name: &str, on_click: T) -> Self {
        self.menu_items.push(MenuItem::Item {
            name: name.to_string(),
            event: on_click,
            disabled: false,
            icon: None,
        });
        self
    }

    pub fn with_checkable_item(mut self, name: &str, is_checked: bool, on_click: T) -> Self {
        self.menu_items.push(MenuItem::CheckableItem {
            name: name.to_string(),
            is_checked,
            event: on_click,
            disabled: false,
            icon: None,
        });
        self
    }

    pub fn with_child_menu<F>(mut self, name: &str, f: F) -> Self
    where
        F: FnOnce(MenuBuilder<T>) -> MenuBuilder<T>,
    {
        let sub = MenuBuilder::new();
        self.menu_items.push(MenuItem::ChildMenu {
            name: name.to_string(),
            children: f(sub),
            disabled: false,
            icon: None,
        });
        self
    }

    pub(crate) fn build(self) -> Result<sys::MenuSys<T>, Error> {
        sys::build_menu(self)
    }
}

#[derive(Debug)]
pub struct TrayIconBuilder<T>
where
    T: PartialEq + Clone,
{
    parent_hwnd: Option<u32>,
    icon: Result<Icon, Error>,
    width: Option<u32>,
    height: Option<u32>,
    menu: Option<Result<sys::MenuSys<T>, Error>>,
    on_click: Option<T>,
    on_double_click: Option<T>,
    on_right_click: Option<T>,
    sender: TrayIconSender<T>,
}

#[derive(Debug, Clone, Copy)]
pub enum Error {
    IconLoadingFailed,
    IconMissing,
    OsError,
}

impl<T> TrayIconBuilder<T>
where
    T: PartialEq + Clone,
{
    pub fn new(sender: TrayIconSender<T>) -> TrayIconBuilder<T> {
        TrayIconBuilder {
            parent_hwnd: None,
            icon: Err(Error::IconMissing),
            width: None,
            height: None,
            menu: None,
            on_click: None,
            on_double_click: None,
            on_right_click: None,
            sender,
        }
    }

    pub fn with_click(mut self, event: T) -> Self {
        self.on_click = Some(event);
        self
    }

    pub fn with_double_click(mut self, event: T) -> Self {
        self.on_double_click = Some(event);
        self
    }

    pub fn with_right_click(mut self, event: T) -> Self {
        self.on_right_click = Some(event);
        self
    }

    pub fn with_parent_hwnd(mut self, hwnd: u32) -> Self {
        self.parent_hwnd = Some(hwnd);
        self
    }

    pub fn with_icon(mut self, icon: Icon) -> Self {
        self.icon = Ok(icon);
        self
    }

    pub fn with_icon_from_buffer(mut self, buffer: &'static [u8]) -> Self {
        self.icon = Icon::from_buffer(buffer, None, None);
        self
    }

    pub fn with_menu<F>(mut self, f: F) -> Self
    where
        F: FnOnce(MenuBuilder<T>) -> MenuBuilder<T>, //Result<sys::MenuSys<T>, Error>,
        T: PartialEq + Clone,
    {
        self.menu = Some(f(MenuBuilder::new()).build());
        self
    }

    pub fn build(self) -> Result<Box<impl TrayIconBase<T>>, Error> {
        Ok(sys::build_trayicon(self)?)
    }
}

pub trait TrayIconBase<T>
where
    T: PartialEq + Clone,
{
    fn set_icon(&mut self, icon: &Icon) -> Result<(), Error>;
    fn set_menu(&mut self, menu: MenuBuilder<T>) -> Result<(), Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    enum Events {
        ClickTrayIcon,
        DoubleClickTrayIcon,
        Exit,
        Item1,
        Item2,
        Item3,
        Item4,
        CheckItem1,
        SubItem1,
        SubItem2,
        SubItem3,
        SubSubItem1,
        SubSubItem2,
        SubSubItem3,
    }

    #[test]
    fn test_integration_test() {
        let (s, r) = std::sync::mpsc::channel::<Events>();
        let icon = include_bytes!("./testresource/icon1.ico");
        let icon2 = include_bytes!("./testresource/icon2.ico");

        let second_icon = Icon::from_buffer(icon2, None, None).unwrap();
        let first_icon = Icon::from_buffer(icon, None, None).unwrap();

        let mut tray_icon = TrayIconBuilder::new(s)
            .with_icon_from_buffer(icon)
            .with_click(Events::ClickTrayIcon)
            .with_double_click(Events::DoubleClickTrayIcon)
            .with_menu(|menu| {
                menu.with_checkable_item("This is checkable", true, Events::CheckItem1)
                    .with_child_menu("Sub Menu", |menu| {
                        menu.with_item("Sub item 1", Events::SubItem1)
                            .with_item("Sub Item 2", Events::SubItem2)
                            .with_item("Sub Item 3", Events::SubItem3)
                            .with_child_menu("Sub Sub menu", |menu| {
                                menu.with_item("Sub Sub item 1", Events::SubSubItem1)
                                    .with_item("Sub Sub Item 2", Events::SubSubItem2)
                                    .with_item("Sub Sub Item 3", Events::SubSubItem3)
                            })
                    })
                    .with(MenuItem::Item {
                        name: "Item Disabled".into(),
                        disabled: true, // Disabled entry example
                        event: Events::Item4,
                        icon: None,
                    })
                    .with_item("Item 3 Replace Menu", Events::Item3)
                    .with_item("Item 2 Change Icon Green", Events::Item2)
                    .with_item("Item 1 Change Icon Red", Events::Item1)
                    .with_separator()
                    .with_item("E&xit", Events::Exit)
            })
            .build()
            .unwrap();

        std::thread::spawn(move || {
            r.iter().for_each(|m| match m {
                Events::DoubleClickTrayIcon => {
                    println!("Double click");
                }
                Events::ClickTrayIcon => {
                    println!("Single click");
                }
                Events::Exit => {
                    println!("Please exit");
                    #[cfg(target_os = "windows")]
                    unsafe {
                        winapi::um::winuser::PostQuitMessage(0);
                    }
                }
                Events::Item1 => {
                    tray_icon.set_icon(&second_icon).unwrap();
                }
                Events::Item2 => {
                    tray_icon.set_icon(&first_icon).unwrap();
                }
                Events::Item3 => {
                    tray_icon
                        .set_menu(MenuBuilder::new().with_item("Exit", Events::Exit))
                        .unwrap();
                }
                e => {
                    println!("{:?}", e);
                }
            })
        });

        // This library does not provide main loop intentionally, this is up to
        // user program to provide
        #[cfg(target_os = "windows")]
        sys::tests::main_loop()
    }
}
