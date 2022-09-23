# Claves - A library for capturing Keystrokes and Mouse clicks on Linux, MacOS and Windows

This library uses the most efficient methods for capturing global **Keystrokes** and **Mouse Clicks**. 

It uses an **unbounded** [crossbeam_channel](https://lib.rs/crossbeam_channel) to provide a global channel from which you can receive the `Event` data on multiple threads.

## Supported Platforms

- [x] MacOS (Uses Core Graphics API to intercept Session Events and Carbon API to translate Virtual Keycodes into Unicode Characters). Requires Accessibility permissions, check the [MacOS Example](examples/macos.rs).
- [x] Windows (Uses Windows Hooks API to intercept Events and the Winuser API to translate Virtual Keycodes into Unicode Characters).
- [ ] Linux. _Coming soon_.
