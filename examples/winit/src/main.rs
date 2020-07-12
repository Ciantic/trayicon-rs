use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use trayicon::{Icon, MenuItem, TrayIcon, TrayIconBuilder};

#[derive(Clone, Eq, PartialEq, Debug)]
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
}

fn main() {
    let event_loop = EventLoop::<Events>::with_user_event();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let proxy = event_loop.create_proxy();
    let icon = include_bytes!("../../../src/testresource/icon1.ico");
    let icon2 = include_bytes!("../../../src/testresource/icon2.ico");

    let second_icon = Icon::from_buffer(icon2, None, None).unwrap();
    let first_icon = Icon::from_buffer(icon, None, None).unwrap();

    // Needlessly complicated tray icon with all the whistles and bells
    let mut tray_icon = TrayIconBuilder::new()
        .with_sender_winit(proxy)
        .with_icon_from_buffer(icon)
        .with_click(Events::ClickTrayIcon)
        .with_double_click(Events::DoubleClickTrayIcon)
        .with_menu(|menu| {
            menu.with_item("Item 3 Replace Menu", Events::Item3)
                .with_item("Item 2 Change Icon Green", Events::Item2)
                .with_item("Item 1 Change Icon Red", Events::Item1)
                .with_separator()
                .with_checkable_item("This is checkable", true, Events::CheckItem1)
                .with_child_menu("Sub Menu", |menu| {
                    menu.with_item("Sub item 1", Events::SubItem1)
                        .with_item("Sub Item 2", Events::SubItem2)
                        .with_item("Sub Item 3", Events::SubItem3)
                })
                .with(MenuItem::Item {
                    name: "Item Disabled".into(),
                    disabled: true, // Disabled entry example
                    event: Events::Item4,
                    icon: None,
                })
                .with_separator()
                .with_item("E&xit", Events::Exit)
        })
        .build()
        .unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,

            // User events
            Event::UserEvent(e) => match e {
                Events::Exit => *control_flow = ControlFlow::Exit,

                Events::Item1 => {
                    tray_icon.set_icon(&second_icon).unwrap();
                }
                Events::Item2 => {
                    tray_icon.set_icon(&first_icon).unwrap();
                }
                Events::Item3 => {
                    tray_icon
                        .set_menu(|menu| {
                            menu.with_item("New menu item", Events::Item1)
                                .with_item("Exit", Events::Exit)
                        })
                        .unwrap();
                }
                e => println!("Got event {:?}", e),
            },
            _ => (),
        }
    });
}
