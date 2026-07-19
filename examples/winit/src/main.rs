use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::Window,
};

use trayicon::{Icon, MenuBuilder, MenuItem, TrayIcon, TrayIconBuilder, TrayIconStatus};

#[derive(Clone, Eq, PartialEq, Debug)]
enum UserEvents {
    RightClickTrayIcon,
    LeftClickTrayIcon,
    DoubleClickTrayIcon,
    StatusActive,
    StatusNeedsAttention,
    StatusPassive,
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
    // Mutually-exclusive radio options. Two independent groups live in one
    // submenu, split by a separator — each group is exclusive within itself.
    RadioRed,
    RadioGreen,
    RadioBlue,
    RadioCircle,
    RadioSquare,
    RadioTriangle,
}

fn build_menu(
    selected_color: &UserEvents,
    selected_shape: &UserEvents,
    check_item_checked: bool,
    disabled_item_disabled: bool,
    disabled_item_icon: Option<Icon>,
) -> MenuBuilder<UserEvents> {
    MenuBuilder::new()
        .item("Item 4 Set Tooltip", UserEvents::Item4)
        .item("Item 3 Replace Menu 👍", UserEvents::Item3)
        .item("Item 2 Change Icon Green", UserEvents::Item2)
        .item("Item 1 Change Icon Red", UserEvents::Item1)
        .submenu(
            "Set Status (KDE only feature)",
            MenuBuilder::new()
                .item("Active (Normal)", UserEvents::StatusActive)
                .item("NeedsAttention (Blink)", UserEvents::StatusNeedsAttention)
                .item("Passive (Hide behind arrow)", UserEvents::StatusPassive),
        )
        .separator()
        .submenu(
            "Sub Menu",
            MenuBuilder::new()
                .item("Sub item 1", UserEvents::SubItem1)
                .item("Sub Item 2", UserEvents::SubItem2)
                .item("Sub Item 3", UserEvents::SubItem3),
        )
        // Two radio groups in a single submenu. A group is a maximal run of
        // consecutive `radio` items; the `separator()` between them splits
        // them into two independent groups, each exclusive within itself.
        // (Without the separator the six items would form one group.)
        .submenu(
            "Radio Groups",
            MenuBuilder::new()
                // Group 1: color.
                .radio(
                    "Red",
                    *selected_color == UserEvents::RadioRed,
                    UserEvents::RadioRed,
                )
                .radio(
                    "Green",
                    *selected_color == UserEvents::RadioGreen,
                    UserEvents::RadioGreen,
                )
                .radio(
                    "Blue",
                    *selected_color == UserEvents::RadioBlue,
                    UserEvents::RadioBlue,
                )
                .separator()
                // Group 2: shape.
                .radio(
                    "Circle",
                    *selected_shape == UserEvents::RadioCircle,
                    UserEvents::RadioCircle,
                )
                .radio(
                    "Square",
                    *selected_shape == UserEvents::RadioSquare,
                    UserEvents::RadioSquare,
                )
                .radio(
                    "Triangle",
                    *selected_shape == UserEvents::RadioTriangle,
                    UserEvents::RadioTriangle,
                ),
        )
        .checkable(
            "This checkbox toggles disable",
            check_item_checked,
            UserEvents::CheckItem1,
        )
        .with(MenuItem::Item {
            name: "Item Disabled".into(),
            disabled: disabled_item_disabled,
            id: UserEvents::DisabledItem1,
            icon: disabled_item_icon,
        })
        .separator()
        .item("E&xit", UserEvents::Exit)
}

fn main() {
    let event_loop = EventLoop::<UserEvents>::with_user_event().build().unwrap();
    let proxy = event_loop.create_proxy();

    let icon = include_bytes!("../../../src/testresource/icon1.ico");
    let icon2 = include_bytes!("../../../src/testresource/icon2.ico");
    let second_icon = Icon::from_buffer(icon2, None, None).unwrap();
    let first_icon = Icon::from_buffer(icon, None, None).unwrap();
    let disabled_item_icon = Result::ok(Icon::from_buffer(icon, None, None));

    let selected_color = UserEvents::RadioRed;
    let selected_shape = UserEvents::RadioCircle;
    let check_item_checked = true;
    let disabled_item_disabled = true;

    // Needlessly complicated tray icon with all the whistles and bells
    let tray_icon = TrayIconBuilder::new()
        .sender(move |e: &UserEvents| {
            let _ = proxy.send_event(e.clone());
        })
        .icon_from_buffer(icon)
        .title("Cool Tray Icon App (KDE Title)")
        .tooltip("Cool Tray 👀 Icon")
        // Binding `on_click`, `on_double_click` and `on_right_click` is optional, if not bound it will still open the menu on right click (all platforms) and left click (MacOS).
        .on_click(UserEvents::LeftClickTrayIcon)
        .on_double_click(UserEvents::DoubleClickTrayIcon)
        .on_right_click(UserEvents::RightClickTrayIcon)
        .menu(build_menu(
            &selected_color,
            &selected_shape,
            check_item_checked,
            disabled_item_disabled,
            disabled_item_icon.clone(),
        ))
        .build()
        .unwrap();

    let mut app = MyApplication {
        window: None,
        tray_icon,
        first_icon,
        second_icon,
        selected_color,
        selected_shape,
        check_item_checked,
        disabled_item_disabled,
        disabled_item_icon,
    };
    event_loop.run_app(&mut app).unwrap();
}

struct MyApplication {
    window: Option<Window>,
    tray_icon: TrayIcon<UserEvents>,
    first_icon: Icon,
    second_icon: Icon,
    selected_color: UserEvents,
    selected_shape: UserEvents,
    check_item_checked: bool,
    disabled_item_disabled: bool,
    disabled_item_icon: Option<Icon>,
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
            UserEvents::LeftClickTrayIcon => {
                println!("Left click tray icon");
                if let Some(token) = self.tray_icon.get_xdg_activation_token() {
                    println!("XDG Activation Token: {}", token);
                }
                self.tray_icon.show_menu().unwrap();
            }
            UserEvents::DoubleClickTrayIcon => {
                println!("Double click tray icon");
            }
            UserEvents::RightClickTrayIcon => {
                println!("Right click tray icon");
                self.tray_icon.show_menu().unwrap();
            }
            UserEvents::CheckItem1 => {
                self.check_item_checked = !self.check_item_checked;
                self.disabled_item_disabled = self.check_item_checked;
                self.rebuild_menu();
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
            UserEvents::StatusActive => {
                self.tray_icon.set_status(TrayIconStatus::Active).unwrap();
            }
            UserEvents::StatusNeedsAttention => {
                self.tray_icon
                    .set_status(TrayIconStatus::NeedsAttention)
                    .unwrap();
            }
            UserEvents::StatusPassive => {
                self.tray_icon.set_status(TrayIconStatus::Passive).unwrap();
            }
            // Selecting a color radio rebuilds the menu so only that option in
            // the color group is checked. The shape group is untouched because
            // the groups are independent (separated by a separator).
            UserEvents::RadioRed | UserEvents::RadioGreen | UserEvents::RadioBlue => {
                self.selected_color = event.clone();
                self.rebuild_menu();
                println!("Color selected: {:?}", event);
            }
            // Selecting a shape radio is handled the same way; only the shape
            // group's selection changes, the color group keeps its selection.
            UserEvents::RadioCircle | UserEvents::RadioSquare | UserEvents::RadioTriangle => {
                self.selected_shape = event.clone();
                self.rebuild_menu();
                println!("Shape selected: {:?}", event);
            }
            // Events::DoubleClickTrayIcon => todo!(),
            // Events::DisabledItem1 => todo!(),
            // Events::SubItem1 => todo!(),
            // Events::SubItem2 => todo!(),
            // Events::SubItem3 => todo!(),
            _ => {}
        }
    }
}

impl MyApplication {
    fn rebuild_menu(&mut self) {
        self.tray_icon
            .set_menu(&build_menu(
                &self.selected_color,
                &self.selected_shape,
                self.check_item_checked,
                self.disabled_item_disabled,
                self.disabled_item_icon.clone(),
            ))
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn radio_rebuild_keeps_checkbox_and_disabled_state() {
        let rebuilt = build_menu(
            &UserEvents::RadioGreen,
            &UserEvents::RadioCircle,
            false,
            false,
            None,
        );

        assert_ne!(
            rebuilt,
            build_menu(
                &UserEvents::RadioGreen,
                &UserEvents::RadioCircle,
                true,
                false,
                None,
            )
        );
        assert_ne!(
            rebuilt,
            build_menu(
                &UserEvents::RadioGreen,
                &UserEvents::RadioCircle,
                false,
                true,
                None,
            )
        );
    }
}
