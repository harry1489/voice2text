#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(target_os = "windows"))]
fn main() {
    eprintln!("v2t-config-gui is only available on Windows");
}

#[cfg(target_os = "windows")]
fn main() {
    use std::fs;
    use std::path::PathBuf;
    use windows_sys::Win32::Foundation::*;
    use windows_sys::Win32::Graphics::Gdi::*;
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::UI::WindowsAndMessaging::*;

    const ID_MODEL_COMBO: usize = 1001;
    const ID_KEY_COMBO: usize = 1002;
    const ID_BTN_DOWNLOAD: usize = 1003;
    const ID_BTN_SAVE: usize = 1004;
    const ID_BTN_LAUNCH: usize = 1005;

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

    fn to_wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    unsafe fn create_child(
        parent: HWND,
        class: &str,
        title: &str,
        style: u32,
        x: i32, y: i32, w: i32, h: i32,
        id: usize,
    ) -> HWND {
        CreateWindowExW(
            0,
            to_wide(class).as_ptr(),
            to_wide(title).as_ptr(),
            style,
            x, y, w, h,
            parent,
            id as HMENU,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    }

    unsafe extern "system" fn wnd_proc(
        hwnd: HWND, msg: u32, wparam: WPARAM, _lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_CREATE => {
                let font = GetStockObject(DEFAULT_GUI_FONT);

                let label = create_child(
                    hwnd, "Static", "Model:",
                    WS_CHILD | WS_VISIBLE,
                    20, 20, 60, 20, 0,
                );
                SendMessageW(label, WM_SETFONT, font as WPARAM, 1);

                let combo = CreateWindowExW(
                    0,
                    to_wide("ComboBox").as_ptr(),
                    std::ptr::null_mut(),
                    WS_CHILD | WS_VISIBLE | CBS_DROPDOWNLIST as u32 | WS_VSCROLL,
                    90, 18, 350, 200,
                    hwnd,
                    ID_MODEL_COMBO as HMENU,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                );
                SendMessageW(combo, WM_SETFONT, font as WPARAM, 1);
                for (name, desc) in MODELS {
                    let item = format!("{name} - {desc}");
                    let wide = to_wide(&item);
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

                let label2 = create_child(
                    hwnd, "Static", "Key:",
                    WS_CHILD | WS_VISIBLE,
                    20, 60, 60, 20, 0,
                );
                SendMessageW(label2, WM_SETFONT, font as WPARAM, 1);

                let keycombo = CreateWindowExW(
                    0,
                    to_wide("ComboBox").as_ptr(),
                    std::ptr::null_mut(),
                    WS_CHILD | WS_VISIBLE | CBS_DROPDOWNLIST as u32 | WS_VSCROLL,
                    90, 58, 350, 200,
                    hwnd,
                    ID_KEY_COMBO as HMENU,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                );
                SendMessageW(keycombo, WM_SETFONT, font as WPARAM, 1);
                for (hex, desc) in KEYS {
                    let item = format!("{desc} - {hex}");
                    let wide = to_wide(&item);
                    SendMessageW(keycombo, CB_ADDSTRING, 0, wide.as_ptr() as LPARAM);
                }
                let current_key = cfg.get("trigger").map(|s| s.as_str()).unwrap_or("0xc1");
                for (i, (hex, _)) in KEYS.iter().enumerate() {
                    if *hex == current_key {
                        SendMessageW(keycombo, CB_SETCURSEL, i, 0);
                        break;
                    }
                }

                let btn_dl = create_child(
                    hwnd, "Button", "Download Model",
                    WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON as u32,
                    90, 110, 170, 35, ID_BTN_DOWNLOAD,
                );
                SendMessageW(btn_dl, WM_SETFONT, font as WPARAM, 1);

                let btn_save = create_child(
                    hwnd, "Button", "Save",
                    WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON as u32,
                    270, 110, 80, 35, ID_BTN_SAVE,
                );
                SendMessageW(btn_save, WM_SETFONT, font as WPARAM, 1);

                let btn_launch = create_child(
                    hwnd, "Button", "Launch voice2text",
                    WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON as u32,
                    90, 160, 260, 35, ID_BTN_LAUNCH,
                );
                SendMessageW(btn_launch, WM_SETFONT, font as WPARAM, 1);

                0
            }

            WM_COMMAND => {
                let id = wparam & 0xffff;
                match id {
                    ID_BTN_SAVE => {
                        let model_idx = SendMessageW(GetDlgItem(hwnd, ID_MODEL_COMBO as i32), CB_GETCURSEL, 0, 0);
                        if model_idx >= 0 && (model_idx as usize) < MODELS.len() {
                            write_config("model", MODELS[model_idx as usize].0);
                        }
                        let key_idx = SendMessageW(GetDlgItem(hwnd, ID_KEY_COMBO as i32), CB_GETCURSEL, 0, 0);
                        if key_idx >= 0 && (key_idx as usize) < KEYS.len() {
                            write_config("trigger", KEYS[key_idx as usize].0);
                        }
                        let _ = fs::create_dir_all(models_dir());
                        MessageBoxW(
                            hwnd,
                            to_wide("Config saved!").as_ptr(),
                            to_wide("voice2text").as_ptr(),
                            MB_OK | MB_ICONINFORMATION,
                        );
                        0
                    }
                    ID_BTN_DOWNLOAD => {
                        let model_idx = SendMessageW(GetDlgItem(hwnd, ID_MODEL_COMBO as i32), CB_GETCURSEL, 0, 0);
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
                                to_wide("Model already downloaded!").as_ptr(),
                                to_wide("voice2text").as_ptr(),
                                MB_OK | MB_ICONINFORMATION,
                            );
                            return 0;
                        }
                        let url = format!("https://huggingface.co/ggerganov/whisper.cpp/resolve/main/{model_name}");
                        let msg = format!("Downloading {model_name}...\nThis may take a while.");
                        MessageBoxW(
                            hwnd,
                            to_wide(&msg).as_ptr(),
                            to_wide("voice2text").as_ptr(),
                            MB_OK | MB_ICONINFORMATION,
                        );

                        let cmd = format!("curl -L --fail --progress-bar -o \"{}\" \"{}\"", dest.display(), url);
                        let _ = std::process::Command::new("cmd")
                            .args(["/C", "start", "cmd", "/C", &cmd])
                            .spawn();

                        0
                    }
                    ID_BTN_LAUNCH => {
                        let _ = std::process::Command::new("voice2text").spawn();
                        0
                    }
                    _ => 0,
                }
            }

            WM_DESTROY => {
                PostQuitMessage(0);
                0
            }

            _ => DefWindowProcW(hwnd, msg, wparam, _lparam),
        }
    }

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
            hIcon: LoadIconW(std::ptr::null_mut(), IDI_APPLICATION),
            hCursor: LoadCursorW(std::ptr::null_mut(), IDC_ARROW),
            hbrBackground: (COLOR_WINDOW + 1) as usize as HBRUSH,
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(),
            hIconSm: std::ptr::null_mut(),
        };
        RegisterClassExW(&wc);

        let title = to_wide("voice2text - Configuration");
        CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX,
            CW_USEDEFAULT, CW_USEDEFAULT, 480, 260,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            instance,
            std::ptr::null_mut(),
        );

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
