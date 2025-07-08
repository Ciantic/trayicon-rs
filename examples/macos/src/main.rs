use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
use objc2_foundation::MainThreadMarker;
use std::sync::mpsc;
use trayicon::*;

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
}

fn build_menu(checked: bool) -> MenuBuilder<Events> {
    MenuBuilder::new()
        .item("Item 1", Events::Item1)
        .item("Item 2", Events::Item2)
        .item("Item 3", Events::Item3)
        .checkable("Checkable Item", checked, Events::CheckItem1)
        .submenu(
            "Submenu",
            MenuBuilder::new()
                .item("Sub Item 1", Events::SubItem1)
                .item("Sub Item 2", Events::SubItem2)
                .item("Sub Item 3", Events::SubItem3),
        )
        .separator()
        .item("Exit", Events::Exit)
}

fn main() {
    let (s, r) = mpsc::channel::<Events>();
    let icon = include_bytes!("../../../src/testresource/icon1.ico");
    let icon2 = include_bytes!("../../../src/testresource/icon2.ico");

    let second_icon = Icon::from_buffer(icon2, None, None).unwrap();
    let first_icon = Icon::from_buffer(icon, None, None).unwrap();

    // Initialize NSApplication for macOS
    let mtm = MainThreadMarker::new().unwrap();
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

    let mut checked = true; // Initial state for the checkable item

    // Needlessly complicated tray icon with all the whistles and bells
    let mut tray_icon = TrayIconBuilder::new()
        .sender(move |e: &Events| {
            let _ = s.send(*e);
        })
        .icon_from_buffer(icon)
        .tooltip("Cool macOS Tray 🍎 Icon")
        .on_right_click(Events::RightClickTrayIcon)
        .on_click(Events::LeftClickTrayIcon)
        .on_double_click(Events::DoubleClickTrayIcon)
        .menu(build_menu(checked))
        .build()
        .unwrap();

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
            Events::Item3 => {
                tray_icon
                    .set_menu(
                        &MenuBuilder::new()
                            .item("Back to main menu", Events::Item4)
                            .item("Exit", Events::Exit),
                    )
                    .unwrap();
            }
            Events::Item4 => {
                tray_icon.set_menu(&build_menu(checked)).unwrap();
            }
            Events::CheckItem1 => {
                checked = !checked;
                tray_icon.set_menu(&build_menu(checked)).unwrap();
            }
            e => {
                println!("{:?}", e);
            }
        })
    });

    // macOS application main loop using NSApplication
    unsafe {
        app.run();
    }
}
