mod error;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "windows")]
mod windows;

use crossbeam_channel::{Receiver, Sender};
use once_cell::sync::Lazy;
use std::{
    fmt,
    sync::atomic::{AtomicBool, Ordering},
};

pub use error::*;

static INITIALIZED: AtomicBool = AtomicBool::new(false);

static CHANNEL: Lazy<(Sender<Event>, Receiver<Event>)> =
    Lazy::new(|| crossbeam_channel::unbounded());

static CLOSE_CHANNEL: Lazy<(Sender<()>, Receiver<()>)> =
    Lazy::new(|| crossbeam_channel::bounded(0));

/// Initializes the keylogging thread and returns a [`Receiver<Event>`] to listen for keystrokes
/// and mouse events.
pub fn init() -> Receiver<Event> {
    let receiver = CHANNEL.1.clone();

    if INITIALIZED.load(Ordering::Relaxed) {
        return receiver;
    }

    #[cfg(target_os = "macos")]
    {
        macos::init();
    }

    #[cfg(target_os = "windows")]
    {
        windows::init();
    }

    INITIALIZED.store(true, Ordering::SeqCst);

    receiver
}

pub fn deinit() {
    if !INITIALIZED.load(Ordering::Relaxed) {
        return;
    }

    let sender = &CLOSE_CHANNEL.0;

    unsafe {
        sender.send(()).unwrap_unchecked();
    }

    INITIALIZED.store(false, Ordering::SeqCst);

    // Clean the global channel.
    while CHANNEL.1.try_recv().is_ok() {}
}

#[derive(Debug, Copy, Clone)]
pub enum Event {
    Mouse(Mouse),
    Keystroke(Keystroke),
}

impl From<Mouse> for Event {
    fn from(mouse: Mouse) -> Self {
        Self::Mouse(mouse)
    }
}

impl From<Keystroke> for Event {
    fn from(keystroke: Keystroke) -> Self {
        Self::Keystroke(keystroke)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Keystroke {
    Escape,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Return,
    Tab,
    Space,
    Delete,
    Backspace,
    VolumeUp,
    VolumeDown,
    Mute,
    ForwardDelete,
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    Alt,
    #[cfg(target_os = "windows")]
    LeftWindows,
    #[cfg(target_os = "windows")]
    RightWindows,
    #[cfg(target_os = "macos")]
    Command,
    #[cfg(target_os = "macos")]
    RightCommand,
    #[cfg(target_os = "macos")]
    _Option,
    #[cfg(target_os = "macos")]
    RightOption,
    Control,
    RightControl,
    Shift,
    RightShift,
    CapsLock,
    Function,
    Home,
    PageUp,
    PageDown,
    End,
    LeftArrow,
    UpArrow,
    RightArrow,
    DownArrow,
    Help,
    Insert,
    Printscreen,
    ScrollLock,
    Pause,
    Menu,
    Char(char),
}

impl fmt::Display for Keystroke {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Keystroke::Char(c) => write!(f, "{}", c),
            _ => write!(f, "{:?}", self),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MouseButton {
    Left,
    Right,
    Other,
}

#[derive(Debug, Copy, Clone)]
pub struct Mouse {
    button: MouseButton,
    position: (i64, i64),
}

impl Mouse {
    pub fn new(button: MouseButton, position: (i64, i64)) -> Self {
        Self { button, position }
    }

    pub fn button(&self) -> MouseButton {
        self.button
    }

    pub fn position(&self) -> (i64, i64) {
        self.position
    }
}
