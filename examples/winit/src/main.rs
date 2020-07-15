use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use trayicon::{Icon, MenuBuilder, MenuItem, TrayIconBuilder};

#[derive(Clone, Eq, PartialEq, Debug)]
enum Events {
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
    let event_loop = EventLoop::<Events>::with_user_event();
    let your_app_window = WindowBuilder::new().build(&event_loop).unwrap();
    let proxy = event_loop.create_proxy();
    let icon = include_bytes!("../../../src/testresource/icon1.ico");
    let icon2 = include_bytes!("../../../src/testresource/icon2.ico");

    let second_icon = Icon::from_buffer(icon2, None, None).unwrap();
    let first_icon = Icon::from_buffer(icon, None, None).unwrap();

    // Needlessly complicated tray icon with all the whistles and bells
    let mut tray_icon = TrayIconBuilder::new()
        .sender_winit(proxy)
        .icon_from_buffer(icon)
        .tooltip("Cool Tray 👀 Icon")
        .on_click(Events::ClickTrayIcon)
        .on_double_click(Events::DoubleClickTrayIcon)
        .menu(
            MenuBuilder::new()
                .item("Item 4 Set Tooltip", Events::Item4)
                .item("Item 3 Replace Menu 👍", Events::Item3)
                .item("Item 2 Change Icon Green", Events::Item2)
                .item("Item 1 Change Icon Red", Events::Item1)
                .separator()
                .submenu(
                    "Sub Menu",
                    MenuBuilder::new()
                        .item("Sub item 1", Events::SubItem1)
                        .item("Sub Item 2", Events::SubItem2)
                        .item("Sub Item 3", Events::SubItem3),
                )
                .checkable("This checkbox toggles disable", true, Events::CheckItem1)
                .with(MenuItem::Item {
                    name: "Item Disabled".into(),
                    disabled: true, // Disabled entry example
                    id: Events::DisabledItem1,
                    icon: None,
                })
                .separator()
                .item("E&xit", Events::Exit),
        )
        .build()
        .unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            // Main window events
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == your_app_window.id() => *control_flow = ControlFlow::Exit,

            // User events
            Event::UserEvent(e) => match e {
                Events::Exit => *control_flow = ControlFlow::Exit,
                Events::CheckItem1 => {
                    // You can mutate single checked, disabled value followingly.
                    //
                    // However, I think better way is to use reactively
                    // `set_menu` by building the menu based on application
                    // state.
                    if let Some(old_value) = tray_icon.get_menu_item_checkable(Events::CheckItem1) {
                        // Set checkable example
                        let _ = tray_icon.set_menu_item_checkable(Events::CheckItem1, !old_value);

                        // Set disabled example
                        let _ = tray_icon.set_menu_item_disabled(Events::DisabledItem1, !old_value);
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
                        .set_menu(
                            &MenuBuilder::new()
                                .item("Another item", Events::Item1)
                                .item("Exit", Events::Exit),
                        )
                        .unwrap();
                }
                Events::Item4 => {
                    tray_icon.set_tooltip("Menu changed!").unwrap();
                }
                e => println!("Got event {:?}", e),
            },
            _ => (),
        }
    });
}
