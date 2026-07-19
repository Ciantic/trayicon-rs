use core::mem::MaybeUninit;
use trayicon::*;
use winapi::um::winuser;

fn main() {
    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    enum Events {
        RightClickTrayIcon,
        LeftClickTrayIcon,
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
        // Mutually-exclusive radio options. Two independent groups live in one
        // submenu, split by a separator — each group is exclusive within itself.
        RadioRed,
        RadioGreen,
        RadioBlue,
        RadioCircle,
        RadioSquare,
        RadioTriangle,
    }

    let (s, r) = crossbeam_channel::unbounded();
    let icon = include_bytes!("../../../src/testresource/icon1.ico");
    let icon2 = include_bytes!("../../../src/testresource/icon2.ico");

    let second_icon = Icon::from_buffer(icon2, None, None).unwrap();
    let first_icon = Icon::from_buffer(icon, None, None).unwrap();

    // Which radio option is currently selected in each group; rebuilt into the
    // menu so each group shows a single selection.
    let selected_color = std::sync::Arc::new(std::sync::Mutex::new(Events::RadioRed));
    let selected_shape = std::sync::Arc::new(std::sync::Mutex::new(Events::RadioCircle));
    let make_menu = |color: Events, shape: Events| {
        MenuBuilder::new()
            .item("Item 3 Replace Menu 👍", Events::Item3)
            .item("Item 2 Change Icon Green", Events::Item2)
            .item("Item 1 Change Icon Red", Events::Item1)
            .separator()
            .checkable("This is checkable", true, Events::CheckItem1)
            .submenu(
                "Sub Menu",
                MenuBuilder::new()
                    .item("Sub item 1", Events::SubItem1)
                    .item("Sub Item 2", Events::SubItem2)
                    .item("Sub Item 3", Events::SubItem3),
            )
            // Two radio groups in one submenu, split by a separator. A group is
            // a maximal run of consecutive `radio` items; the `separator` breaks
            // them into two independent groups (color, shape), each exclusive
            // within itself.
            .submenu(
                "Radio Groups",
                MenuBuilder::new()
                    .radio("Red", color == Events::RadioRed, Events::RadioRed)
                    .radio("Green", color == Events::RadioGreen, Events::RadioGreen)
                    .radio("Blue", color == Events::RadioBlue, Events::RadioBlue)
                    .separator()
                    .radio("Circle", shape == Events::RadioCircle, Events::RadioCircle)
                    .radio("Square", shape == Events::RadioSquare, Events::RadioSquare)
                    .radio(
                        "Triangle",
                        shape == Events::RadioTriangle,
                        Events::RadioTriangle,
                    ),
            )
            .with(MenuItem::Item {
                name: "Item Disabled".into(),
                disabled: true, // Disabled entry example
                id: Events::Item4,
                icon: None,
            })
            .separator()
            .item("E&xit", Events::Exit)
    };

    // Needlessly complicated tray icon with all the whistles and bells
    let mut tray_icon = TrayIconBuilder::new()
        .sender(move |e| {
            let _ = s.send(*e);
        })
        .icon_from_buffer(icon)
        .tooltip("Cool Tray 👀 Icon")
        .on_right_click(Events::RightClickTrayIcon)
        .on_click(Events::LeftClickTrayIcon)
        .on_double_click(Events::DoubleClickTrayIcon)
        .menu(make_menu(
            *selected_color.lock().unwrap(),
            *selected_shape.lock().unwrap(),
        ))
        .build()
        .unwrap();

    let color_for_events = selected_color.clone();
    let shape_for_events = selected_shape.clone();
    std::thread::spawn(move || {
        r.iter().for_each(|m| match m {
            Events::RightClickTrayIcon => {
                tray_icon.show_menu().unwrap();
            }
            Events::DoubleClickTrayIcon => {
                println!("Double click");
            }
            Events::LeftClickTrayIcon => {
                tray_icon.show_menu().unwrap();
            }
            Events::Exit => {
                println!("Please exit");
                std::process::exit(0);
            }
            Events::Item1 => {
                tray_icon.set_icon(&second_icon).unwrap();
            }
            Events::Item2 => {
                tray_icon.set_icon(&first_icon).unwrap();
            }
            // Selecting a color radio rebuilds the menu so only that color
            // option is checked; the shape group keeps its own selection.
            Events::RadioRed | Events::RadioGreen | Events::RadioBlue => {
                if let Ok(mut sel) = color_for_events.lock() {
                    *sel = m;
                    tray_icon
                        .set_menu(&make_menu(*sel, *shape_for_events.lock().unwrap()))
                        .unwrap();
                }
                println!("Color selected: {:?}", m);
            }
            // Selecting a shape radio — independent of the color group.
            Events::RadioCircle | Events::RadioSquare | Events::RadioTriangle => {
                if let Ok(mut sel) = shape_for_events.lock() {
                    *sel = m;
                    tray_icon
                        .set_menu(&make_menu(*color_for_events.lock().unwrap(), *sel))
                        .unwrap();
                }
                println!("Shape selected: {:?}", m);
            }
            Events::Item3 => {
                tray_icon
                    .set_menu(
                        &MenuBuilder::new()
                            .item("New menu item", Events::Item1)
                            .item("Exit", Events::Exit),
                    )
                    .unwrap();
            }
            e => {
                println!("{:?}", e);
            }
        })
    });

    // Your applications message loop. Because all applications require an
    // application loop, you are best served using an `winit` crate.
    loop {
        unsafe {
            let mut msg = MaybeUninit::uninit();
            let bret = winuser::GetMessageA(msg.as_mut_ptr(), 0 as _, 0, 0);
            if bret > 0 {
                winuser::TranslateMessage(msg.as_ptr());
                winuser::DispatchMessageA(msg.as_ptr());
            } else {
                break;
            }
        }
    }
}
