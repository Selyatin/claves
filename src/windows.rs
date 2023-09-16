#![cfg(target_os = "windows")]
use super::*;
use std::{mem, sync::Mutex, thread};

use windows_sys::Win32::{
    Foundation::{LPARAM, LRESULT, WPARAM},
    System::Threading::GetCurrentThreadId,
    UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
};

static CAPTURING_THREAD_ID: Mutex<Option<u32>> = Mutex::new(None);

pub fn init() {
    thread::spawn(|| unsafe {
        {
            *CAPTURING_THREAD_ID.lock().unwrap() = Some(GetCurrentThreadId());
        }
        capture_peripherals();
    });

    thread::spawn(|| {
        if CLOSE_CHANNEL.1.recv().is_ok() {
            let id_option = *CAPTURING_THREAD_ID.lock().unwrap();
            unsafe {
                if let Some(id) = id_option {
                    PostThreadMessageA(id, WM_QUIT, 0, 0);
                }
            }
        }
    });
}

unsafe fn capture_peripherals() {
    let keyboard_hhook = SetWindowsHookExA(WH_KEYBOARD_LL, Some(hook_callback), 0, 0);

    let mouse_hhook = SetWindowsHookExA(WH_MOUSE_LL, Some(hook_callback), 0, 0);

    if keyboard_hhook == 0 || mouse_hhook == 0 {
        panic!(
            "Couldn't Setup Hooks, Keyboard: {} Mouse: {}",
            keyboard_hhook, mouse_hhook
        );
    }

    message_loop();

    if UnhookWindowsHookEx(keyboard_hhook) == 0 {
        panic!("Windows Unhook non-zero return");
    }

    if UnhookWindowsHookEx(mouse_hhook) == 0 {
        panic!("Windows Unhook non-zero return");
    }
}

/// This function handles the Event Loop, which is necessary in order for the hooks to function.
fn message_loop() {
    unsafe {
        let mut msg: MSG = mem::zeroed();
        while GetMessageA(&mut msg, 0, 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageA(&msg);
        }
    }
}

unsafe extern "system" fn hook_callback(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    use Keystroke::*;

    let call_next_hook = || CallNextHookEx(0, code, w_param, l_param);

    if code as u32 != HC_ACTION {
        return call_next_hook();
    }

    let sender = &CHANNEL.0;

    let event = match w_param as u32 {
        // Left Alt Key Didn't work as WM_KEYDOWN
        260 => {
            sender.send(Alt.into()).unwrap_unchecked();
            return call_next_hook();
        }
        WM_KEYDOWN => {
            let keyboard_dll_hook_struct = *(l_param as *mut KBDLLHOOKSTRUCT);

            let v_key = keyboard_dll_hook_struct.vkCode;

            let non_char_key = match v_key as u16 {
                VK_BACK => Some(Backspace),
                VK_RETURN => Some(Return),
                VK_TAB => Some(Tab),
                VK_SHIFT | VK_LSHIFT => Some(Shift),
                VK_RSHIFT => Some(RightShift),
                VK_CONTROL | VK_LCONTROL => Some(Control),
                VK_RCONTROL => Some(RightControl),
                VK_MENU => Some(Alt),
                VK_LWIN => Some(LeftWindows),
                VK_RWIN => Some(RightWindows),
                VK_ESCAPE => Some(Escape),
                VK_END => Some(End),
                VK_HOME => Some(Home),
                VK_LEFT => Some(LeftArrow),
                VK_RIGHT => Some(RightArrow),
                VK_UP => Some(UpArrow),
                VK_DOWN => Some(DownArrow),
                VK_PRIOR => Some(PageUp),
                VK_NEXT => Some(PageDown),
                VK_F1 => Some(F1),
                VK_F2 => Some(F2),
                VK_F3 => Some(F3),
                VK_F4 => Some(F4),
                VK_F5 => Some(F5),
                VK_F6 => Some(F6),
                VK_F7 => Some(F7),
                VK_F8 => Some(F8),
                VK_F9 => Some(F9),
                VK_F10 => Some(F10),
                VK_F11 => Some(F11),
                VK_F12 => Some(F12),
                VK_DELETE => Some(Delete),
                45 => Some(Insert),
                44 => Some(Printscreen),
                145 => Some(ScrollLock),
                19 => Some(Pause),
                93 => Some(Menu),
                _ => None,
            };

            if let Some(key) = non_char_key {
                sender.send(key.into()).unwrap_unchecked();
                return call_next_hook();
            }

            let is_lowercase = (GetKeyState(VK_CAPITAL as i32) & 0x0001) == 0
                && (GetKeyState(VK_SHIFT as i32) & 0x1000) == 0
                && (GetKeyState(VK_LSHIFT as i32) & 0x1000) == 0
                && (GetKeyState(VK_RSHIFT as i32) & 0x1000) == 0;

            if !is_lowercase {
                let c = match v_key {
                    0x30 => Some(')'),
                    0x31 => Some('!'),
                    0x32 => Some('@'),
                    0x33 => Some('#'),
                    0x34 => Some('$'),
                    0x35 => Some('%'),
                    0x36 => Some('^'),
                    0x37 => Some('&'),
                    0x38 => Some('*'),
                    0x39 => Some('('),
                    188 => Some('<'),
                    190 => Some('.'),
                    191 => Some('?'),
                    186 => Some(':'),
                    222 => Some('"'),
                    219 => Some('{'),
                    221 => Some('}'),
                    220 => Some('|'),
                    _ => None,
                };

                if let Some(c) = c {
                    sender.send(Keystroke::Char(c).into()).unwrap_unchecked();

                    return call_next_hook();
                }
            }

            let layout = GetKeyboardLayout(GetCurrentThreadId());

            let mapped_c = MapVirtualKeyExA(v_key, MAPVK_VK_TO_CHAR, layout);

            let mut c = char::from_u32(mapped_c).unwrap();

            if is_lowercase {
                c = c.to_ascii_lowercase();
            }

            Char(c).into()
        }
        WM_LBUTTONDOWN => {
            let mouse_hook_struct = *(l_param as *mut MOUSEHOOKSTRUCT);

            let (x, y) = (mouse_hook_struct.pt.x as i64, mouse_hook_struct.pt.y as i64);

            Mouse::new(MouseButton::Left, (x, y)).into()
        }
        WM_RBUTTONDOWN => {
            let mouse_hook_struct = *(l_param as *mut MOUSEHOOKSTRUCT);

            let (x, y) = (mouse_hook_struct.pt.x as i64, mouse_hook_struct.pt.y as i64);

            Mouse::new(MouseButton::Right, (x, y)).into()
        }
        WM_MBUTTONDOWN => {
            let mouse_hook_struct = *(l_param as *mut MOUSEHOOKSTRUCT);

            let (x, y) = (mouse_hook_struct.pt.x as i64, mouse_hook_struct.pt.y as i64);

            Mouse::new(MouseButton::Other, (x, y)).into()
        }
        _ => return call_next_hook(),
    };

    sender.send(event).unwrap_unchecked();

    call_next_hook()
}
