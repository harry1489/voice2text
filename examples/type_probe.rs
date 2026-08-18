use std::thread;
use std::time::Duration;

use evdev::uinput::VirtualDevice;
use evdev::{AttributeSet, KeyCode, KeyEvent};

fn main() {
    let text = std::env::args().nth(1).unwrap_or_else(|| "What's the time? 123!".into());
    let wait: u64 = std::env::args().nth(2).and_then(|s| s.parse().ok()).unwrap_or(3);
    let delay_ms: u64 = std::env::args().nth(3).and_then(|s| s.parse().ok()).unwrap_or(2);

    let mut keys = AttributeSet::new();
    for code in 1..=255 {
        keys.insert(KeyCode::new(code));
    }

    let builder = VirtualDevice::builder().expect("uinput");
    let mut device = builder.name("voice2text-kb").with_keys(&keys).and_then(|b| b.build()).expect("build");

    eprintln!("[probe] device created, waiting {wait}s for capture setup...");
    thread::sleep(Duration::from_secs(wait));
    eprintln!("[probe] typing: {text:?}");

    thread::sleep(Duration::from_millis(60));
    for ch in text.chars() {
        let Some((code, shifted)) = key_for_char(ch) else {
            continue;
        };
        eprintln!("[probe] char {ch:?} -> code {}, shifted {shifted}", code.code());
        if shifted {
            emit(&mut device, KeyCode::KEY_LEFTSHIFT, 1);
        }
        emit(&mut device, code, 1);
        emit(&mut device, code, 0);
        if shifted {
            emit(&mut device, KeyCode::KEY_LEFTSHIFT, 0);
        }
        thread::sleep(Duration::from_millis(delay_ms));
    }
    eprintln!("[probe] done");
    thread::sleep(Duration::from_secs(2));
}

fn emit(device: &mut VirtualDevice, code: KeyCode, value: i32) {
    if let Err(e) = device.emit(&[KeyEvent::new(code, value).into()]) {
        eprintln!("[probe] emit error: {e}");
    }
}

fn key_for_char(c: char) -> Option<(KeyCode, bool)> {
    let (base, shift) = match c {
        'a'..='z' => (keycode_for_letter(c), false),
        'A'..='Z' => (keycode_for_letter(c.to_ascii_lowercase()), true),
        '0' => (KeyCode::KEY_0, false),
        '1'..='9' => (KeyCode::new(KeyCode::KEY_1.code() + (c as u8 - b'1') as u16), false),
        ' ' => (KeyCode::KEY_SPACE, false),
        '\n' => (KeyCode::KEY_ENTER, false),
        '.' => (KeyCode::KEY_DOT, false),
        ',' => (KeyCode::KEY_COMMA, false),
        '/' => (KeyCode::KEY_SLASH, false),
        '`' => (KeyCode::KEY_GRAVE, false),
        '-' => (KeyCode::KEY_MINUS, false),
        '=' => (KeyCode::KEY_EQUAL, false),
        '[' => (KeyCode::KEY_LEFTBRACE, false),
        ']' => (KeyCode::KEY_RIGHTBRACE, false),
        '\\' => (KeyCode::KEY_BACKSLASH, false),
        ';' => (KeyCode::KEY_SEMICOLON, false),
        '\'' => (KeyCode::KEY_APOSTROPHE, false),
        '!' => (KeyCode::KEY_1, true),
        '@' => (KeyCode::KEY_2, true),
        '#' => (KeyCode::KEY_3, true),
        '$' => (KeyCode::KEY_4, true),
        '%' => (KeyCode::KEY_5, true),
        '^' => (KeyCode::KEY_6, true),
        '&' => (KeyCode::KEY_7, true),
        '*' => (KeyCode::KEY_8, true),
        '(' => (KeyCode::KEY_9, true),
        ')' => (KeyCode::KEY_0, true),
        '_' => (KeyCode::KEY_MINUS, true),
        '+' => (KeyCode::KEY_EQUAL, true),
        '{' => (KeyCode::KEY_LEFTBRACE, true),
        '}' => (KeyCode::KEY_RIGHTBRACE, true),
        '|' => (KeyCode::KEY_BACKSLASH, true),
        ':' => (KeyCode::KEY_SEMICOLON, true),
        '"' => (KeyCode::KEY_APOSTROPHE, true),
        '<' => (KeyCode::KEY_COMMA, true),
        '>' => (KeyCode::KEY_DOT, true),
        '?' => (KeyCode::KEY_SLASH, true),
        '~' => (KeyCode::KEY_GRAVE, true),
        _ => return None,
    };
    Some((base, shift))
}

fn keycode_for_letter(l: char) -> KeyCode {
    match l {
        'a' => KeyCode::KEY_A,
        'b' => KeyCode::KEY_B,
        'c' => KeyCode::KEY_C,
        'd' => KeyCode::KEY_D,
        'e' => KeyCode::KEY_E,
        'f' => KeyCode::KEY_F,
        'g' => KeyCode::KEY_G,
        'h' => KeyCode::KEY_H,
        'i' => KeyCode::KEY_I,
        'j' => KeyCode::KEY_J,
        'k' => KeyCode::KEY_K,
        'l' => KeyCode::KEY_L,
        'm' => KeyCode::KEY_M,
        'n' => KeyCode::KEY_N,
        'o' => KeyCode::KEY_O,
        'p' => KeyCode::KEY_P,
        'q' => KeyCode::KEY_Q,
        'r' => KeyCode::KEY_R,
        's' => KeyCode::KEY_S,
        't' => KeyCode::KEY_T,
        'u' => KeyCode::KEY_U,
        'v' => KeyCode::KEY_V,
        'w' => KeyCode::KEY_W,
        'x' => KeyCode::KEY_X,
        'y' => KeyCode::KEY_Y,
        'z' => KeyCode::KEY_Z,
        _ => unreachable!("keycode_for_letter called with {l:?}"),
    }
}
