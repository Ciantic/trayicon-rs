use std::sync::mpsc::Sender;

#[cfg(target_os = "windows")]
#[path = "./sys/windows/mod.rs"]
mod sys;

#[derive(Debug)]
pub enum MenuItem<T> {
    Separator,
    Item(String, T),
    CheckableItem(String, bool, T),
    ChildMenu(String, Result<sys::MenuSys<T>, Error>),
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

    pub fn with_item(mut self, name: &str, on_click: T) -> Self {
        self.menu_items
            .push(MenuItem::Item(name.to_string(), on_click));
        self
    }

    pub fn with_separator(mut self) -> Self {
        self.menu_items.push(MenuItem::Separator);
        self
    }

    pub fn with_checkable_item(mut self, name: &str, is_checked: bool, on_click: T) -> Self {
        self.menu_items.push(MenuItem::CheckableItem(
            name.to_string(),
            is_checked,
            on_click,
        ));
        self
    }

    pub fn with_child_menu<F>(mut self, name: &str, f: F) -> Self
    where
        F: FnOnce(MenuBuilder<T>) -> Result<sys::MenuSys<T>, Error>,
    {
        let sub = MenuBuilder::new();
        self.menu_items
            .push(MenuItem::ChildMenu(name.to_string(), f(sub)));
        self
    }
    pub fn build(self) -> Result<sys::MenuSys<T>, Error> {
        sys::build_menu(self)
    }
}

#[derive(Debug)]
pub struct TrayIconBuilder<T> {
    parent_hwnd: Option<u32>,
    icon_buffer: Option<&'static [u8]>,
    width: Option<u32>,
    height: Option<u32>,
    menu: Option<Result<sys::MenuSys<T>, Error>>,
    on_click: Option<T>,
    on_double_click: Option<T>,
    on_right_click: Option<T>,
    sender: Sender<T>,
}

#[derive(Debug, Clone, Copy)]
pub enum Error {
    IconLoadingFailed,
    OsError,
}

impl<T> TrayIconBuilder<T>
where
    T: PartialEq + Clone,
{
    pub fn new(sender: Sender<T>) -> TrayIconBuilder<T> {
        TrayIconBuilder {
            parent_hwnd: None,
            icon_buffer: None,
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

    pub fn with_icon_from_buffer(mut self, buffer: &'static [u8]) -> Self {
        self.icon_buffer = Some(buffer);
        self
    }

    pub fn with_menu<F>(mut self, f: F) -> Self
    where
        F: FnOnce(MenuBuilder<T>) -> Result<sys::MenuSys<T>, Error>,
        T: PartialEq + Clone,
    {
        let mut builder = MenuBuilder::new();
        self.menu = Some(f(builder));
        self
    }

    pub fn build(self) -> Result<TrayIcon<T>, Error> {
        Ok(TrayIcon(sys::build_icon(self)?))
    }
}

pub struct TrayIcon<T>(crate::sys::TrayIconSys<T>)
where
    T: PartialEq + Clone;

pub trait TrayIconBase {
    fn set_icon_from_buffer(
        &mut self,
        buffer: &'static [u8],
        width: Option<u32>,
        height: Option<u32>,
    ) -> Result<(), Error>;
}

impl<T> TrayIconBase for TrayIcon<T>
where
    T: PartialEq + Clone,
{
    fn set_icon_from_buffer(
        &mut self,
        buffer: &'static [u8],
        width: Option<u32>,
        height: Option<u32>,
    ) -> Result<(), Error> {
        self.0.set_icon_from_buffer(buffer, width, height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Copy, Clone, Eq, PartialEq)]
    enum Events {
        ClickTrayIcon,
        DoubleClickTrayIcon,
        Exit,
        Item1,
        CheckItem1,
        SubItem1,
        SubItem2,
        SubItem3,
    }

    #[test]
    fn test_menu() {
        let (s, r) = std::sync::mpsc::channel::<Events>();
        let icon = include_bytes!("./testresource/icon1.ico");
        let _tray_icon = TrayIconBuilder::new(s)
            .with_click(Events::ClickTrayIcon)
            .with_double_click(Events::DoubleClickTrayIcon)
            .with_icon_from_buffer(icon)
            .with_menu(|menu| {
                menu.with_checkable_item("This is checkable", false, Events::CheckItem1)
                    .with_child_menu("Sub Menu", |menu| {
                        menu.with_item("Sub item 1", Events::SubItem1)
                            .with_item("Sub Item 2", Events::SubItem2)
                            .with_item("Sub Item 3", Events::SubItem3)
                            .build()
                    })
                    .with_item("Item 1", Events::Item1)
                    .with_separator()
                    .with_item("E&xit", Events::Exit)
                    .build()
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
                }
                _ => {}
            })
        });

        // This library does not provide main loop intentionally, this is up to
        // user program to provide
        #[cfg(target_os = "windows")]
        sys::tests::main_loop()
    }
}
