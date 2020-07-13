use core::mem::MaybeUninit;
use trayicon::*;
use winapi::um::winuser;

fn main() {
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
    }

    let (s, r) = crossbeam_channel::unbounded();
    let icon = include_bytes!("../../../src/testresource/icon1.ico");
    let icon2 = include_bytes!("../../../src/testresource/icon2.ico");

    let second_icon = Icon::from_buffer(icon2, None, None).unwrap();
    let first_icon = Icon::from_buffer(icon, None, None).unwrap();

    // Needlessly complicated tray icon with all the whistles and bells
    let mut tray_icon = TrayIconBuilder::new()
        .sender_crossbeam(s)
        .icon_from_buffer(icon)
        .on_click(Events::ClickTrayIcon)
        .on_double_click(Events::DoubleClickTrayIcon)
        .menu(
            MenuBuilder::new()
                .item("Item 3 Replace Menu", Events::Item3)
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
                .with(MenuItem::Item {
                    name: "Item Disabled".into(),
                    disabled: true, // Disabled entry example
                    event: Events::Item4,
                    icon: None,
                })
                .separator()
                .item("E&xit", Events::Exit),
        )
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
