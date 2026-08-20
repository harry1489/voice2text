use std::process::Command;
use std::thread;
use std::time::Duration;

use evdev::uinput::VirtualDevice;
use evdev::{AttributeSet, KeyCode, KeyEvent};

pub fn type_text(text: &str) {
    if text.is_empty() {
        return;
    }

    if type_with_uinput(text) {
        return;
    }
    if type_with_ydotool(text) {
        return;
    }
    if type_with_wtype(text) {
        return;
    }
    if copy_to_clipboard(text) {
        return;
    }

    eprintln!("[typeout] could not inject text; transcription result:");
    eprintln!("{text}");
}

fn type_with_uinput(text: &str) -> bool {
    let mut keys = AttributeSet::new();
    for code in 1..=255 {
        keys.insert(KeyCode::new(code));
    }

    let builder = match VirtualDevice::builder() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("[typeout] uinput unavailable: {e}");
            return false;
        }
    };
    let mut device = match builder.name("voice2text-kb").with_keys(&keys).and_then(|b| b.build())
    {
        Ok(d) => d,
        Err(e) => {
            eprintln!("[typeout] uinput unavailable: {e}");
            return false;
        }
    };

    thread::sleep(Duration::from_millis(60));

    for ch in text.chars() {
        let Some((code, shifted)) = key_for_char(ch) else {
            continue;
        };
        if shifted {
            emit(&mut device, KeyCode::KEY_LEFTSHIFT, 1);
        }
        emit(&mut device, code, 1);
        emit(&mut device, code, 0);
        if shifted {
            emit(&mut device, KeyCode::KEY_LEFTSHIFT, 0);
        }
        thread::sleep(Duration::from_millis(2));
    }
    eprintln!("[typeout] typed via uinput");
    true
}

fn emit(device: &mut VirtualDevice, code: KeyCode, value: i32) {
    let _ = device.emit(&[KeyEvent::new(code, value).into()]);
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

fn type_with_ydotool(text: &str) -> bool {
    let Ok(output) = Command::new("ydotool")
        .args(["type", text])
        .output()
    else {
        return false;
    };
    if output.status.success() {
        return true;
    }
    eprintln!(
        "[typeout] ydotool failed (is the ydotool daemon running?): {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    false
}

fn type_with_wtype(text: &str) -> bool {
    let Ok(output) = Command::new("wtype")
        .arg(text)
        .output()
    else {
        return false;
    };
    if output.status.success() {
        return true;
    }
    eprintln!(
        "[typeout] wtype failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    false
}

fn copy_to_clipboard(text: &str) -> bool {
    if std::env::var("WAYLAND_DISPLAY").is_err() && std::env::var("DISPLAY").is_err() {
        return false;
    }
    let Ok(output) = Command::new("wl-copy").arg(text).output() else {
        return false;
    };
    if output.status.success() {
        eprintln!("[typeout] text copied to clipboard instead of typed");
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn types_via_uinput() {
        assert!(type_with_uinput("hello, world 123!"));
    }

    #[test]
    fn maps_ascii_keys() {
        assert!(key_for_char(' ').is_some());
        assert_eq!(key_for_char('a').unwrap().1, false);
        assert_eq!(key_for_char('A').unwrap().1, true);
        assert_eq!(key_for_char('!').unwrap(), (KeyCode::KEY_1, true));
        assert!(key_for_char('é').is_none());
    }

    #[test]
    fn maps_letters_to_physical_keycodes() {
        assert_eq!(key_for_char('a'), Some((KeyCode::KEY_A, false)));
        assert_eq!(key_for_char('h'), Some((KeyCode::KEY_H, false)));
        assert_eq!(key_for_char('t'), Some((KeyCode::KEY_T, false)));
        assert_eq!(key_for_char('e'), Some((KeyCode::KEY_E, false)));
        assert_eq!(key_for_char('m'), Some((KeyCode::KEY_M, false)));
        assert_eq!(key_for_char('s'), Some((KeyCode::KEY_S, false)));
        assert_eq!(key_for_char('b'), Some((KeyCode::KEY_B, false)));
        assert_eq!(key_for_char('z'), Some((KeyCode::KEY_Z, false)));
        assert_eq!(key_for_char('W'), Some((KeyCode::KEY_W, true)));
        assert_eq!(key_for_char('D'), Some((KeyCode::KEY_D, true)));
        assert_eq!(key_for_char('H'), Some((KeyCode::KEY_H, true)));
    }
}
