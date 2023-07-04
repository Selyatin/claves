#![cfg(target_os = "windows")]
use super::*;
use std::{ptr, sync::Mutex, thread};
use winapi::{
    ctypes::c_int,
    shared::minwindef::{DWORD, LPARAM, LRESULT, UINT, WPARAM},
    um::{
        processthreadsapi::GetCurrentThreadId,
        winuser::*
        // winuser::{
        //     CallNextHookEx, DispatchMessageA, GetKeyState, GetKeyboardLayout, GetMessageA,
        //     MapVirtualKeyExA, PostThreadMessageA, SetWindowsHookExA, TranslateMessage,
        //     UnhookWindowsHookEx, HC_ACTION, KBDLLHOOKSTRUCT, MAPVK_VK_TO_CHAR, MOUSEHOOKSTRUCT,
        //     MSG, VK_BACK, VK_CAPITAL, VK_CONTROL, VK_DOWN, VK_END, VK_ESCAPE, VK_HOME, VK_LCONTROL,
        //     VK_LEFT, VK_LSHIFT, VK_LWIN, VK_MENU, VK_NEXT, VK_PRIOR, VK_RCONTROL, VK_RETURN,
        //     VK_RIGHT, VK_RSHIFT, VK_RWIN, VK_SHIFT, VK_TAB, VK_UP, WH_KEYBOARD_LL, WH_MOUSE_LL,
        //     WM_KEYDOWN, WM_LBUTTONDOWN, WM_MBUTTONDOWN, WM_QUIT, WM_RBUTTONDOWN,
        // },
    },
};

static CAPTURING_THREAD_ID: Mutex<Option<DWORD>> = Mutex::new(None);

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
    let keyboard_hhook = SetWindowsHookExA(WH_KEYBOARD_LL, Some(hook_callback), ptr::null_mut(), 0);

    let mouse_hhook = SetWindowsHookExA(WH_MOUSE_LL, Some(hook_callback), ptr::null_mut(), 0);

    if keyboard_hhook.is_null() || mouse_hhook.is_null() {
        panic!(
            "Couldn't Setup Hooks, Keyboard: {} Mouse: {}",
            keyboard_hhook.is_null(),
            mouse_hhook.is_null()
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
    let mut msg = MSG::default();
    unsafe {
        while GetMessageA(&mut msg, ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageA(&msg);
        }
    }
}

unsafe extern "system" fn hook_callback(code: c_int, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let call_next_hook = || CallNextHookEx(ptr::null_mut(), code, w_param, l_param);

    if code != HC_ACTION {
        return call_next_hook();
    }

    let sender = &CHANNEL.0;

    let u_int = UINT::try_from(w_param).unwrap();

    let event = match u_int {
        WM_KEYDOWN => {
            use Keystroke::*;

            let keyboard_dll_hook_struct = *(l_param as *mut KBDLLHOOKSTRUCT);

            let v_key = keyboard_dll_hook_struct.vkCode;

            println!("{:#?}", v_key);

            let non_char_key = match v_key as i32 {
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

            let is_lowercase = (GetKeyState(VK_CAPITAL) & 0x0001) == 0
                && (GetKeyState(VK_SHIFT) & 0x1000) == 0
                && (GetKeyState(VK_LSHIFT) & 0x1000) == 0
                && (GetKeyState(VK_RSHIFT) & 0x1000) == 0;

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
