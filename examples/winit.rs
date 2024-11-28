// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(unused)]

use tray_icon::{
    menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    TrayIconBuilder, TrayIconEvent,
};
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoopBuilder},
};

enum UserEvent {
    TrayIconEvent(tray_icon::TrayIconEvent),
}

fn main() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/icon.png");

    // Since winit doesn't use gtk on Linux, and we need gtk for
    // the tray icon to show up, we need to spawn a thread
    // where we initialize gtk and create the tray_icon
    #[cfg(target_os = "linux")]
    std::thread::spawn(|| {
        use tray_icon::menu::Menu;

        let icon = load_icon(std::path::Path::new(path));

        gtk::init().unwrap();
        let _tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(Menu::new()))
            .with_icon(icon)
            .build()
            .unwrap();

        gtk::main();
    });

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event()
        .build()
        .unwrap();

    // set a tray event handler that forwards the event and wakes up the event loop
    let proxy = event_loop.create_proxy();
    TrayIconEvent::set_event_handler(Some(move |event| {
        proxy.send_event(UserEvent::TrayIconEvent(event));
    }));

    #[cfg(not(target_os = "linux"))]
    let mut tray_icon = None;

    let menu_channel = MenuEvent::receiver();
    let tray_channel = TrayIconEvent::receiver();

    event_loop.run(move |event, event_loop| {
        event_loop.set_control_flow(ControlFlow::Wait);

        match event {
            #[cfg(not(target_os = "linux"))]
            Event::NewEvents(winit::event::StartCause::Init) => {
                let icon = load_icon(std::path::Path::new(path));

                // We create the icon once the event loop is actually running
                // to prevent issues like https://github.com/tauri-apps/tray-icon/issues/90
                tray_icon = Some(
                    TrayIconBuilder::new()
                        .with_menu(Box::new(Menu::new()))
                        .with_tooltip("winit - awesome windowing lib")
                        .with_icon(icon)
                        .with_title("x")
                        .build()
                        .unwrap(),
                );
                // We have to request a redraw here to have the icon actually show up.
                // Winit only exposes a redraw method on the Window so we use core-foundation directly.
                #[cfg(target_os = "macos")]
                unsafe {
                    use core_foundation::runloop::{CFRunLoopGetMain, CFRunLoopWakeUp};

                    let rl = CFRunLoopGetMain();
                    CFRunLoopWakeUp(rl);
                }
            }

            Event::UserEvent(UserEvent::TrayIconEvent(event)) => {
                println!("{event:?}");
            }

            _ => {}
        }
    });
}

fn load_icon(path: &std::path::Path) -> tray_icon::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}
