use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

use windows::{Win32::Foundation::*, Win32::UI::WindowsAndMessaging::*};

static KEY_PRESS_COUNT: AtomicU32 = AtomicU32::new(0);
static MOUSE_CLICK_COUNT: AtomicU32 = AtomicU32::new(0);

unsafe extern "system" fn mouse_hook(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if n_code >= 0 {
        match w_param.0 as u32 {
            WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN | WM_XBUTTONDOWN => {
                MOUSE_CLICK_COUNT.fetch_add(1, Ordering::Relaxed);
                println!(
                    "mouse click count: {}",
                    MOUSE_CLICK_COUNT.load(Ordering::Relaxed)
                )
            }
            _ => {}
        }
    }
    unsafe { CallNextHookEx(HHOOK::default(), n_code, w_param, l_param) }
}

unsafe extern "system" fn keyboard_hook(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if n_code >= 0 {
        match w_param.0 as u32 {
            WM_KEYDOWN | WM_SYSKEYDOWN => {
                KEY_PRESS_COUNT.fetch_add(1, Ordering::Relaxed);
                println!("keydown count: {}", KEY_PRESS_COUNT.load(Ordering::Relaxed));
            }
            _ => {}
        }
    }
    unsafe { CallNextHookEx(HHOOK::default(), n_code, w_param, l_param) }
}

fn track_application_usage() {
    unsafe {
        let mut last_window: HWND = HWND::default();
        let mut start_time: Instant = Instant::now();

        loop {
            let current_window = GetForegroundWindow();
            if current_window != last_window {
                if last_window.0 != 0 {
                    let duration = start_time.elapsed();
                    let mut title = [0u16; 256];
                    GetWindowTextW(last_window, &mut title);
                    let title = String::from_utf16_lossy(&title);
                    println!(
                        "APP: {}, usage_time: {:?}",
                        title.trim_end_matches('\0'),
                        duration
                    );
                }

                last_window = current_window;
                start_time = Instant::now();
            }

            std::thread::sleep(Duration::from_secs(1));
        }
    }
}

fn main() -> Result<(), windows::core::Error> {
    std::thread::spawn(track_application_usage);

    unsafe {
        SetWindowsHookExA(WH_MOUSE_LL, Some(mouse_hook), None, 0)?;
        SetWindowsHookExA(WH_KEYBOARD_LL, Some(keyboard_hook), None, 0)?;

        let mut msg = MSG::default();
        while GetMessageA(&mut msg, HWND::default(), 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageA(&msg);
        }
    }

    Ok(())
}
