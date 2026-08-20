#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::path::PathBuf;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;

const ID_MODEL_COMBO: isize = 1001;
const ID_KEY_COMBO: isize = 1002;
const ID_BTN_DOWNLOAD: isize = 1003;
const ID_BTN_SAVE: isize = 1004;
const ID_BTN_LAUNCH: isize = 1005;

const MODELS: &[(&str, &str)] = &[
    ("ggml-tiny.en.bin", "~39 MB - Fastest"),
    ("ggml-base.en.bin", "~142 MB - Recommended"),
    ("ggml-small.en.bin", "~461 MB - Better accuracy"),
    ("ggml-medium.en.bin", "~1.5 GB - High accuracy"),
    ("ggml-large-v3.bin", "~3.1 GB - Best accuracy"),
];

const KEYS: &[(&str, &str)] = &[
    ("0xc1", "F23 (Copilot button)"),
    ("0x7f", "F24"),
    ("0xb3", "F19"),
    ("0xb4", "F20"),
    ("0x1d", "Left Ctrl"),
    ("0x3a", "Caps Lock"),
];

fn config_dir() -> PathBuf {
    let home = std::env::var("APPDATA")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join("voice2text")
}

fn config_file() -> PathBuf {
    config_dir().join("config")
}

fn models_dir() -> PathBuf {
    config_dir().join("models")
}

fn read_config() -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    if let Ok(content) = fs::read_to_string(config_file()) {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                map.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
    }
    map
}

fn write_config(key: &str, value: &str) {
    let dir = config_dir();
    let _ = fs::create_dir_all(&dir);
    let mut map = read_config();
    map.insert(key.to_string(), value.to_string());
    let mut content = String::from("# voice2text config\n");
    for (k, v) in &map {
        content.push_str(&format!("{k} = {v}\n"));
    }
    let _ = fs::write(config_file(), content);
}

unsafe fn get_text(hwnd: HWND, id: isize) -> String {
    let len = GetWindowTextLengthW(hwnd) as usize;
    if len == 0 {
        return String::new();
    }
    let mut buf = vec![0u16; len + 1];
    GetWindowTextW(hwnd, buf.as_mut_ptr(), (len + 1) as i32);
    String::from_utf16_lossy(&buf[..len])
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND, msg: u32, wparam: WPARAM, _lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            let font = GetStockObject(DEFAULT_GUI_FONT);

            // Model label
            let label = CreateWindowExW(
                0, ["S","t","a","t","i","c","\0"].as_ptr(),
                ["M","o","d","e",":","\0"].as_ptr(),
                WS_CHILD | WS_VISIBLE,
                20, 20, 60, 20, hwnd, 0 as HMENU, 0 as HINSTANCE, std::ptr::null(),
            );
            SendMessageW(label, WM_SETFONT, font as WPARAM, 1);

            // Model combo
            let combo = CreateWindowExW(
                0, ["C","o","m","b","o","B","o","x","\0"].as_ptr(),
                std::ptr::null(),
                WS_CHILD | WS_VISIBLE | CBS_DROPDOWNLIST | WS_VSCROLL,
                90, 18, 350, 200, hwnd, ID_MODEL_COMBO as HMENU, 0 as HINSTANCE, std::ptr::null(),
            );
            SendMessageW(combo, WM_SETFONT, font as WPARAM, 1);
            for (name, desc) in MODELS {
                let item = format!("{} - {}", name, desc);
                let wide: Vec<u16> = item.encode_utf16().chain(std::iter::once(0)).collect();
                SendMessageW(combo, CB_ADDSTRING, 0, wide.as_ptr() as LPARAM);
            }
            let cfg = read_config();
            let current = cfg.get("model").map(|s| s.as_str()).unwrap_or("ggml-base.en.bin");
            for (i, (name, _)) in MODELS.iter().enumerate() {
                if *name == current {
                    SendMessageW(combo, CB_SETCURSEL, i, 0);
                    break;
                }
            }

            // Key label
            let label2 = CreateWindowExW(
                0, ["S","t","a","t","i","c","\0"].as_ptr(),
                ["K","e","y",":","\0"].as_ptr(),
                WS_CHILD | WS_VISIBLE,
                20, 60, 60, 20, hwnd, 0 as HMENU, 0 as HINSTANCE, std::ptr::null(),
            );
            SendMessageW(label2, WM_SETFONT, font as WPARAM, 1);

            // Key combo
            let keycombo = CreateWindowExW(
                0, ["C","o","m","b","o","B","o","x","\0"].as_ptr(),
                std::ptr::null(),
                WS_CHILD | WS_VISIBLE | CBS_DROPDOWNLIST | WS_VSCROLL,
                90, 58, 350, 200, hwnd, ID_KEY_COMBO as HMENU, 0 as HINSTANCE, std::ptr::null(),
            );
            SendMessageW(keycombo, WM_SETFONT, font as WPARAM, 1);
            for (hex, desc) in KEYS {
                let item = format!("{} - {}", desc, hex);
                let wide: Vec<u16> = item.encode_utf16().chain(std::iter::once(0)).collect();
                SendMessageW(keycombo, CB_ADDSTRING, 0, wide.as_ptr() as LPARAM);
            }
            let current_key = cfg.get("trigger").map(|s| s.as_str()).unwrap_or("0xc1");
            for (i, (hex, _)) in KEYS.iter().enumerate() {
                if *hex == current_key {
                    SendMessageW(keycombo, CB_SETCURSEL, i, 0);
                    break;
                }
            }

            // Download button
            let btn_dl = CreateWindowExW(
                0, ["B","u","t","t","o","n","\0"].as_ptr(),
                ["D","o","w","n","l","o","a","d"," ","M","o","d","e","l","\0"].as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
                90, 110, 170, 35, hwnd, ID_BTN_DOWNLOAD as HMENU, 0 as HINSTANCE, std::ptr::null(),
            );
            SendMessageW(btn_dl, WM_SETFONT, font as WPARAM, 1);

            // Save button
            let btn_save = CreateWindowExW(
                0, ["B","u","t","t","o","n","\0"].as_ptr(),
                ["S","a","v","e","\0"].as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
                270, 110, 80, 35, hwnd, ID_BTN_SAVE as HMENU, 0 as HINSTANCE, std::ptr::null(),
            );
            SendMessageW(btn_save, WM_SETFONT, font as WPARAM, 1);

            // Launch button
            let btn_launch = CreateWindowExW(
                0, ["B","u","t","t","o","n","\0"].as_ptr(),
                ["L","a","u","n","c","h"," ","v","o","i","c","e","2","t","e","x","t","\0"].as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
                90, 160, 260, 35, hwnd, ID_BTN_LAUNCH as HMENU, 0 as HINSTANCE, std::ptr::null(),
            );
            SendMessageW(btn_launch, WM_SETFONT, font as WPARAM, 1);

            0
        }

        WM_COMMAND => {
            let id = (wparam & 0xffff) as isize;
            match id {
                ID_BTN_SAVE => {
                    let model_idx = SendMessageW(GetDlgItem(hwnd, ID_MODEL_COMBO), CB_GETCURSEL, 0, 0);
                    if model_idx >= 0 && (model_idx as usize) < MODELS.len() {
                        write_config("model", MODELS[model_idx as usize].0);
                    }
                    let key_idx = SendMessageW(GetDlgItem(hwnd, ID_KEY_COMBO), CB_GETCURSEL, 0, 0);
                    if key_idx >= 0 && (key_idx as usize) < KEYS.len() {
                        write_config("trigger", KEYS[key_idx as usize].0);
                    }
                    let _ = fs::create_dir_all(models_dir());
                    MessageBoxW(
                        hwnd,
                        ["C","o","n","f","i","g"," ","s","a","v","e","d","!","\0"].as_ptr(),
                        ["v","o","i","c","e","2","t","e","x","t","\0"].as_ptr(),
                        MB_OK | MB_ICONINFORMATION,
                    );
                    0
                }
                ID_BTN_DOWNLOAD => {
                    let model_idx = SendMessageW(GetDlgItem(hwnd, ID_MODEL_COMBO), CB_GETCURSEL, 0, 0);
                    if model_idx < 0 || (model_idx as usize) >= MODELS.len() {
                        return 0;
                    }
                    let model_name = MODELS[model_idx as usize].0;
                    let dir = models_dir();
                    let _ = fs::create_dir_all(&dir);
                    let dest = dir.join(model_name);
                    if dest.exists() {
                        MessageBoxW(
                            hwnd,
                            ["M","o","d","e","l"," ","a","l","r","e","a","d","y"," ","d","o","w","n","l","o","a","d","e","d","!","\0"].as_ptr(),
                            ["v","o","i","c","e","2","t","e","x","t","\0"].as_ptr(),
                            MB_OK | MB_ICONINFORMATION,
                        );
                        return 0;
                    }
                    let url = format!("https://huggingface.co/ggerganov/whisper.cpp/resolve/main/{}", model_name);
                    let msg = format!("Downloading {}...\nThis may take a while.", model_name);
                    let msg_wide: Vec<u16> = msg.encode_utf16().chain(std::iter::once(0)).collect();
                    let title_wide: Vec<u16> = "voice2text".encode_utf16().chain(std::iter::once(0)).collect();
                    MessageBoxW(hwnd, msg_wide.as_ptr(), title_wide.as_ptr(), MB_OK | MB_ICONINFORMATION);

                    let cmd = format!("curl -L --fail --progress-bar -o \"{}\" \"{}\"", dest.display(), url);
                    let _ = std::process::Command::new("cmd")
                        .args(["/C", "start", "cmd", "/C", &cmd])
                        .spawn();

                    0
                }
                ID_BTN_LAUNCH => {
                    let _ = std::process::Command::new("voice2text")
                        .spawn();
                    0
                }
                _ => 0
            }
        }

        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }

        _ => DefWindowProcW(hwnd, msg, wparam, _lparam),
    }
}

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn main() {
    unsafe {
        let instance = GetModuleHandleW(std::ptr::null());
        let class_name = to_wide("Voice2TextConfig");

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance,
            hIcon: LoadIconW(0 as HINSTANCE, IDI_APPLICATION),
            hCursor: LoadCursorW(0 as HINSTANCE, IDC_ARROW),
            hbrBackground: (COLOR_WINDOW as isize + 1) as HBRUSH,
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(),
            hIconSm: 0,
        };
        RegisterClassExW(&wc);

        let title = to_wide("voice2text - Configuration");
        CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX,
            CW_USEDEFAULT, CW_USEDEFAULT, 480, 260,
            0, 0 as HMENU, instance, std::ptr::null(),
        );

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, 0, 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
