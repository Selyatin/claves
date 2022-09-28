# Claves - A library for capturing global Keystrokes and Mouse clicks on MacOS and Windows

- [x] MacOS (Uses Core Graphics API to intercept Session Events and Carbon API to translate Virtual Keycodes into Unicode Characters). Requires Accessibility permissions, check the [MacOS Example](examples/macos.rs).
- [x] Windows (Uses Windows Hooks API to intercept Events and the Winuser API to translate Virtual Keycodes into Unicode Characters).
- [ ] Linux. _Not planned for now_.
