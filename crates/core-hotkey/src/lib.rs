use std::mem::size_of;
use windows::Win32::UI::Input::KeyboardAndMouse::{SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_UNICODE, KEYEVENTF_KEYUP, VK_CONTROL, VIRTUAL_KEY, GetAsyncKeyState};
use windows::Win32::System::DataExchange::{OpenClipboard, CloseClipboard, GetClipboardData, SetClipboardData, EmptyClipboard};
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GHND};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{GetCursorPos, GetGUIThreadInfo, GUITHREADINFO};
use windows::Win32::Foundation::POINT;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InjectionStrategy {
    SendInput,
    ClipboardPaste,
    UiAutomation,
}

pub trait TextInjector: Send + Sync {
    fn inject(&self, text: &str, strategy: InjectionStrategy) -> Result<(), String>;
    fn capture_caret_position(&self) -> Option<(i32, i32)>;
}

pub struct DummyInjector;

impl TextInjector for DummyInjector {
    fn inject(&self, text: &str, _strategy: InjectionStrategy) -> Result<(), String> {
        println!("[Mock Injection]: {}", text);
        Ok(())
    }

    fn capture_caret_position(&self) -> Option<(i32, i32)> {
        Some((100, 100))
    }
}

pub struct WindowsTextInjector;

impl WindowsTextInjector {
    pub fn new() -> Self {
        Self
    }
}

impl TextInjector for WindowsTextInjector {
    fn inject(&self, text: &str, strategy: InjectionStrategy) -> Result<(), String> {
        match strategy {
            InjectionStrategy::SendInput => inject_str_via_send_input(text),
            InjectionStrategy::ClipboardPaste => {
                let saved = get_clipboard_text().unwrap_or_default();
                set_clipboard_text(text)?;
                simulate_ctrl_v();
                std::thread::sleep(std::time::Duration::from_millis(100));
                let _ = set_clipboard_text(&saved);
                Ok(())
            }
            InjectionStrategy::UiAutomation => {
                // UI Automation fallback to SendInput
                inject_str_via_send_input(text)
            }
        }
    }

    fn capture_caret_position(&self) -> Option<(i32, i32)> {
        get_caret_position()
    }
}

fn inject_str_via_send_input(text: &str) -> Result<(), String> {
    let mut inputs = Vec::new();
    for c in text.encode_utf16() {
        inputs.push(INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(0),
                    wScan: c,
                    dwFlags: KEYEVENTF_UNICODE,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        });
        inputs.push(INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(0),
                    wScan: c,
                    dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        });
    }
    unsafe {
        let sent = SendInput(&inputs, size_of::<INPUT>() as i32);
        if sent != inputs.len() as u32 {
            return Err("Failed to send all keyboard inputs".to_string());
        }
    }
    Ok(())
}

fn set_clipboard_text(text: &str) -> Result<(), String> {
    unsafe {
        if !OpenClipboard(HWND(0)).is_ok() {
            return Err("Failed to open clipboard".to_string());
        }
        let _ = EmptyClipboard();
        let utf16: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let size = utf16.len() * 2;
        let handle = GlobalAlloc(GHND, size).map_err(|e| e.to_string())?;
        if handle.is_invalid() {
            let _ = CloseClipboard();
            return Err("Failed to allocate global memory".to_string());
        }
        let ptr = GlobalLock(handle);
        if ptr.is_null() {
            let _ = CloseClipboard();
            return Err("Failed to lock global memory".to_string());
        }
        std::ptr::copy_nonoverlapping(utf16.as_ptr(), ptr as *mut u16, utf16.len());
        let _ = GlobalUnlock(handle);
        if SetClipboardData(13, windows::Win32::Foundation::HANDLE(handle.0 as isize)).is_err() {
            let _ = CloseClipboard();
            return Err("Failed to set clipboard data".to_string());
        }
        let _ = CloseClipboard();
    }
    Ok(())
}

fn get_clipboard_text() -> Result<String, String> {
    unsafe {
        if !OpenClipboard(HWND(0)).is_ok() {
            return Err("Failed to open clipboard".to_string());
        }
        let handle = GetClipboardData(13);
        if handle.is_err() {
            let _ = CloseClipboard();
            return Ok(String::new());
        }
        let h_mem = windows::Win32::Foundation::HGLOBAL(handle.unwrap().0 as *mut ::core::ffi::c_void);
        let ptr = GlobalLock(h_mem);
        if ptr.is_null() {
            let _ = CloseClipboard();
            return Err("Failed to lock global memory".to_string());
        }
        let mut len = 0;
        let mut p = ptr as *const u16;
        while *p != 0 {
            len += 1;
            p = p.add(1);
        }
        let slice = std::slice::from_raw_parts(ptr as *const u16, len);
        let string = String::from_utf16_lossy(slice);
        let _ = GlobalUnlock(h_mem);
        let _ = CloseClipboard();
        Ok(string)
    }
}

fn simulate_ctrl_v() {
    let vk_c = VK_CONTROL;
    let vk_v = VIRTUAL_KEY(0x56);
    let inputs = [
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT { wVk: vk_c, wScan: 0, dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS(0), time: 0, dwExtraInfo: 0 },
            },
        },
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT { wVk: vk_v, wScan: 0, dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS(0), time: 0, dwExtraInfo: 0 },
            },
        },
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT { wVk: vk_v, wScan: 0, dwFlags: KEYEVENTF_KEYUP, time: 0, dwExtraInfo: 0 },
            },
        },
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT { wVk: vk_c, wScan: 0, dwFlags: KEYEVENTF_KEYUP, time: 0, dwExtraInfo: 0 },
            },
        },
    ];
    unsafe {
        SendInput(&inputs, size_of::<INPUT>() as i32);
    }
}

fn get_caret_position() -> Option<(i32, i32)> {
    unsafe {
        let mut info = GUITHREADINFO {
            cbSize: size_of::<GUITHREADINFO>() as u32,
            ..Default::default()
        };
        let thread_id = windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId(
            windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow(),
            None,
        );
        if GetGUIThreadInfo(thread_id, &mut info).is_ok() {
            let mut pt = POINT {
                x: info.rcCaret.left,
                y: info.rcCaret.bottom,
            };
            windows::Win32::Graphics::Gdi::ClientToScreen(info.hwndCaret, &mut pt);
            if pt.x != 0 || pt.y != 0 {
                return Some((pt.x, pt.y));
            }
        }
        let mut pt = POINT::default();
        if GetCursorPos(&mut pt).is_ok() {
            return Some((pt.x, pt.y));
        }
    }
    None
}

/// Key virtual codes
/// VK_CONTROL = 0x11, VK_MENU (Alt) = 0x12, VK_F9 = 0x78
const VK_CTRL: i32 = 0x11;
const VK_ALT: i32 = 0x12;
const VK_F9: i32 = 0x78;

pub struct HotkeyListener;

impl HotkeyListener {
    pub fn new() -> Self {
        Self
    }

    pub fn register(&self) -> Result<(), String> {
        // GetAsyncKeyState needs no registration — it polls hardware directly
        println!("[HotkeyListener] Using GetAsyncKeyState polling on Ctrl+Alt+F9");
        Ok(())
    }

    /// Blocks until Ctrl+Alt+F9 is pressed (leading edge only).
    /// Polls every 50ms — works from any thread, any window focus.
    pub fn wait_for_hotkey(&self) -> bool {
        let mut was_down = false;
        loop {
            std::thread::sleep(std::time::Duration::from_millis(50));
            let triggered = unsafe {
                let ctrl = (GetAsyncKeyState(VK_CTRL) as u16) & 0x8000 != 0;
                let alt  = (GetAsyncKeyState(VK_ALT)  as u16) & 0x8000 != 0;
                let f9   = (GetAsyncKeyState(VK_F9)   as u16) & 0x8000 != 0;
                ctrl && alt && f9
            };
            if triggered && !was_down {
                // Wait until the hotkey is released to prevent instant double-triggering
                while unsafe {
                    let ctrl = (GetAsyncKeyState(VK_CTRL) as u16) & 0x8000 != 0;
                    let alt  = (GetAsyncKeyState(VK_ALT)  as u16) & 0x8000 != 0;
                    let f9   = (GetAsyncKeyState(VK_F9)   as u16) & 0x8000 != 0;
                    ctrl && alt && f9
                } {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                return true;
            }
            if !triggered {
                was_down = false;
            }
        }
    }
}

