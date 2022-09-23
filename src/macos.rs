#![cfg(target_os = "macos")]
use super::*;
use apple_sys::Carbon::{
    kTISPropertyUnicodeKeyLayoutData, kUCKeyActionDisplay, kUCKeyTranslateNoDeadKeysBit,
    LMGetKbdType, TISCopyCurrentKeyboardInputSource, TISGetInputSourceProperty, UCKeyTranslate,
    UCKeyboardLayout, UInt32, UniCharCount,
};
use core_foundation::{
    base::CFRelease,
    data::CFDataGetBytePtr,
    runloop::{kCFRunLoopCommonModes, kCFRunLoopDefaultMode, CFRunLoop},
};
use core_graphics::{
    event::{
        CGEvent, CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions,
        CGEventTapPlacement, CGEventTapProxy,
        CGEventType::{self, KeyDown, LeftMouseDown, OtherMouseDown, RightMouseDown, ScrollWheel},
        EventField,
    },
    geometry::CGPoint,
};
use crossbeam_channel::TryRecvError;
use std::{mem, thread, time::Duration};

pub fn init() {
    thread::spawn(|| unsafe {
        capture_peripherals();
    });
}

unsafe fn capture_peripherals() {
    let current = CFRunLoop::get_current();

    let event_tap = CGEventTap::new(
        CGEventTapLocation::Session,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::Default,
        vec![
            KeyDown,
            LeftMouseDown,
            RightMouseDown,
            ScrollWheel,
            OtherMouseDown,
        ],
        callback,
    )
    .unwrap();

    let loop_source = event_tap.mach_port.create_runloop_source(0).unwrap();

    current.add_source(&loop_source, kCFRunLoopCommonModes);

    event_tap.enable();

    let close_receiver = CLOSE_CHANNEL.1.clone();

    while let Err(TryRecvError::Empty) = close_receiver.try_recv() {
        CFRunLoop::run_in_mode(kCFRunLoopDefaultMode, Duration::from_millis(500), false);
    }

    current.remove_source(&loop_source, kCFRunLoopCommonModes);
}

fn callback(_proxy: CGEventTapProxy, event_type: CGEventType, event: &CGEvent) -> Option<CGEvent> {
    let sender = &CHANNEL.0;

    let event = match event_type {
        LeftMouseDown => {
            let CGPoint { x, y } = event.location();
            Event::Mouse(Mouse::new(MouseButton::Left, (x as i64, y as i64)))
        }
        RightMouseDown => {
            let CGPoint { x, y } = event.location();
            Event::Mouse(Mouse::new(MouseButton::Right, (x as i64, y as i64)))
        }
        OtherMouseDown => {
            let CGPoint { x, y } = event.location();
            Event::Mouse(Mouse::new(MouseButton::Other, (x as i64, y as i64)))
        }
        KeyDown => {
            let flags = event.get_flags();

            let is_shift_pressed = flags.contains(CGEventFlags::CGEventFlagShift);

            let is_capslock_on = flags.contains(CGEventFlags::CGEventFlagAlphaShift);

            let key_code = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);

            match key_code_to_sign(key_code, is_shift_pressed) {
                Some(keystroke) => keystroke,
                None => {
                    let mut c = char::from_u32(unsafe {key_code_to_unicode(key_code) as u32}).unwrap();

                    if (is_shift_pressed && !is_capslock_on)
                        || (!is_shift_pressed && is_capslock_on)
                    {
                        c = c.to_ascii_uppercase();
                    }

                    Keystroke::Char(c)
                }
            }
            .into()
        }
        _ => return None,
    };

    sender.send(event).unwrap();

    None
}

unsafe fn key_code_to_unicode(key_code: i64) -> u32 {
    let current_keyboard = TISCopyCurrentKeyboardInputSource();

    let layout_data = TISGetInputSourceProperty(current_keyboard, kTISPropertyUnicodeKeyLayoutData);

    let keyboard_layout = CFDataGetBytePtr(layout_data as _) as *mut UCKeyboardLayout;

    let mut keys_down: UInt32 = 0;

    let mut c: u16 = 0;

    let mut real_length: UniCharCount = 0;

    UCKeyTranslate(
        keyboard_layout,
        key_code as u16,
        kUCKeyActionDisplay as _,
        0,
        LMGetKbdType() as _,
        kUCKeyTranslateNoDeadKeysBit,
        (&mut keys_down) as _,
        (mem::size_of::<u16>() / mem::size_of::<u8>()) as u64,
        (&mut real_length) as _,
        &mut c,
    );

    CFRelease(current_keyboard as _);

    c.into()
}

fn key_code_to_sign(code: i64, is_shift_pressed: bool) -> Option<Keystroke> {
    use Keystroke::*;

    if is_shift_pressed {
        return match code {
            18 => Some(Char('!')),
            19 => Some(Char('@')),
            20 => Some(Char('#')),
            21 => Some(Char('$')),
            22 => Some(Char('^')),
            23 => Some(Char('%')),
            24 => Some(Char('+')),
            25 => Some(Char('(')),
            26 => Some(Char('&')),
            27 => Some(Char('_')),
            28 => Some(Char('*')),
            29 => Some(Char(')')),
            30 => Some(Char('{')),
            33 => Some(Char('}')),
            39 => Some(Char('"')),
            41 => Some(Char(':')),
            42 => Some(Char('|')),
            43 => Some(Char(')')),
            44 => Some(Char('?')),
            47 => Some(Char('>')),
            50 => Some(Char('~')),
            65 => Some(Char(')')),
            67 => Some(Char('*')),
            69 => Some(Char('+')),
            75 => Some(Char('/')),
            78 => Some(Char('-')),
            81 => Some(Char('=')),
            _ => None,
        };
    }

    match code {
        6 => Some(Return),
        36 => Some(Return),
        48 => Some(Tab),
        51 => Some(Delete),
        53 => Some(Escape),
        54 => Some(RightCommand),
        55 => Some(Command),
        56 => Some(Shift),
        57 => Some(CapsLock),
        58 => Some(_Option),
        59 => Some(Control),
        60 => Some(RightShift),
        61 => Some(RightOption),
        62 => Some(RightControl),
        63 => Some(Function),
        72 => Some(VolumeUp),
        73 => Some(VolumeDown),
        74 => Some(Mute),
        96 => Some(F5),
        97 => Some(F6),
        98 => Some(F7),
        99 => Some(F3),
        100 => Some(F8),
        101 => Some(F9),
        103 => Some(F11),
        109 => Some(F10),
        111 => Some(F12),
        114 => Some(Help),
        115 => Some(Home),
        116 => Some(PageUp),
        117 => Some(ForwardDelete),
        118 => Some(F4),
        119 => Some(End),
        120 => Some(F2),
        121 => Some(PageDown),
        122 => Some(F1),
        123 => Some(LeftArrow),
        124 => Some(RightArrow),
        125 => Some(DownArrow),
        126 => Some(UpArrow),
        _ => None,
    }
}
