# TrayIcon

Currently I target Windows tray icon implementation, with popup menu, click, double click events. Goal is to provide a channel for events and ability to plug in [winit](https://github.com/rust-windowing/winit) event loop easily.

[Open here a full working example with winit crate 🢅](https://github.com/Ciantic/trayicon-rs/blob/master/examples/winit/src/main.rs)

## TODO

Provide coordinates of the Tray Icon area for custom popups.

## Alternatives

Most mature alternative is qdot's [systray-rs](https://github.com/qdot/systray-rs). Unfortunately I got frustrated with the API in it and decided to rewrite my own. This however largely does not use the code in it, instead I loaned my old C/C++ code repository as a template.

## Development

```
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-apple-darwin
rustup target add x86_64-pc-windows-msvc
```

Then you can check the code compiles for all targets with:

```
cargo check --target x86_64-apple-darwin
cargo check --target x86_64-pc-windows-msvc
cargo check --target x86_64-unknown-linux-gnu
```

## Change log

- 0.4.0 - 2026-01-12

  - KDE Support
  - Backwards incompatible change: Event type must be also `Send + Sync`

- 0.3.0 - 2025-07-18

  - Thanks to [@bearice](https://github.com/bearice) for MacOS support.

- 0.2.0 - 2024-05-09

  - Removed dependency to `winit` crate, now setting a sender is a function.
  - Added `show_menu`, this means user must call it to show the menu even on right click. Previously right click always showed the menu.
