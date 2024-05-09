use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::Window,
};

use trayicon::{Icon, MenuBuilder, MenuItem, TrayIcon, TrayIconBuilder};

#[derive(Clone, Eq, PartialEq, Debug)]
enum UserEvents {
    ClickTrayIcon,
    DoubleClickTrayIcon,
    Exit,
    Item1,
    Item2,
    Item3,
    Item4,
    DisabledItem1,
    CheckItem1,
    SubItem1,
    SubItem2,
    SubItem3,
}

fn main() {
    let event_loop = EventLoop::<UserEvents>::with_user_event().build().unwrap();
    let proxy = event_loop.create_proxy();

    let icon = include_bytes!("../../../src/testresource/icon1.ico");
    let icon2 = include_bytes!("../../../src/testresource/icon2.ico");
    let second_icon = Icon::from_buffer(icon2, None, None).unwrap();
    let first_icon = Icon::from_buffer(icon, None, None).unwrap();

    // Needlessly complicated tray icon with all the whistles and bells
    let tray_icon = TrayIconBuilder::new()
        .sender(move |e: &UserEvents| {
            let _ = proxy.send_event(e.clone());
        })
        .icon_from_buffer(icon)
        .tooltip("Cool Tray 👀 Icon")
        .on_click(UserEvents::ClickTrayIcon)
        .on_double_click(UserEvents::DoubleClickTrayIcon)
        .menu(
            MenuBuilder::new()
                .item("Item 4 Set Tooltip", UserEvents::Item4)
                .item("Item 3 Replace Menu 👍", UserEvents::Item3)
                .item("Item 2 Change Icon Green", UserEvents::Item2)
                .item("Item 1 Change Icon Red", UserEvents::Item1)
                .separator()
                .submenu(
                    "Sub Menu",
                    MenuBuilder::new()
                        .item("Sub item 1", UserEvents::SubItem1)
                        .item("Sub Item 2", UserEvents::SubItem2)
                        .item("Sub Item 3", UserEvents::SubItem3),
                )
                .checkable(
                    "This checkbox toggles disable",
                    true,
                    UserEvents::CheckItem1,
                )
                .with(MenuItem::Item {
                    name: "Item Disabled".into(),
                    disabled: true, // Disabled entry example
                    id: UserEvents::DisabledItem1,
                    icon: Result::ok(Icon::from_buffer(icon, None, None)),
                })
                .separator()
                .item("E&xit", UserEvents::Exit),
        )
        .build()
        .unwrap();

    let mut app = MyApplication {
        window: None,
        tray_icon,
        first_icon,
        second_icon,
    };
    event_loop.run_app(&mut app).unwrap();
}

struct MyApplication {
    window: Option<Window>,
    tray_icon: TrayIcon<UserEvents>,
    first_icon: Icon,
    second_icon: Icon,
}

impl ApplicationHandler<UserEvents> for MyApplication {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
    }

    // Platform specific events
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        }
    }

    // Application specific events
    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvents) {
        match event {
            UserEvents::Exit => event_loop.exit(),
            UserEvents::CheckItem1 => {
                // You can mutate single checked, disabled value followingly.
                //
                // However, I think better way is to use reactively
                // `set_menu` by building the menu based on application
                // state.
                if let Some(old_value) = self
                    .tray_icon
                    .get_menu_item_checkable(UserEvents::CheckItem1)
                {
                    // Set checkable example
                    let _ = self
                        .tray_icon
                        .set_menu_item_checkable(UserEvents::CheckItem1, !old_value);

                    // Set disabled example
                    let _ = self
                        .tray_icon
                        .set_menu_item_disabled(UserEvents::DisabledItem1, !old_value);
                }
            }
            UserEvents::Item1 => {
                self.tray_icon.set_icon(&self.second_icon).unwrap();
            }
            UserEvents::Item2 => {
                self.tray_icon.set_icon(&self.first_icon).unwrap();
            }
            UserEvents::Item3 => {
                self.tray_icon
                    .set_menu(
                        &MenuBuilder::new()
                            .item("Another item", UserEvents::Item1)
                            .item("Exit", UserEvents::Exit),
                    )
                    .unwrap();
            }
            UserEvents::Item4 => {
                self.tray_icon.set_tooltip("Menu changed!").unwrap();
            }
            // Events::ClickTrayIcon => todo!(),
            // Events::DoubleClickTrayIcon => todo!(),
            // Events::DisabledItem1 => todo!(),
            // Events::SubItem1 => todo!(),
            // Events::SubItem2 => todo!(),
            // Events::SubItem3 => todo!(),
            _ => {}
        }
    }
}
