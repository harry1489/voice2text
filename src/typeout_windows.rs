use std::thread;
use std::time::Duration;

use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;

pub fn type_text(text: &str) {
    if text.is_empty() {
        return;
    }

    for ch in text.chars() {
        let Some((vk, shift)) = vk_for_char(ch) else {
            continue;
        };
        if shift {
            key_press(VK_SHIFT);
        }
        key_press(vk);
        key_release(vk);
        if shift {
            key_release(VK_SHIFT);
        }
        thread::sleep(Duration::from_millis(2));
    }
    eprintln!("[typeout] typed via SendInput");
}

fn key_press(vk: u16) {
    let input = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: 0,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    unsafe {
        SendInput(1, &input, std::mem::size_of::<INPUT>() as i32);
    }
}

fn key_release(vk: u16) {
    let input = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: KEYEVENTF_KEYUP,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    unsafe {
        SendInput(1, &input, std::mem::size_of::<INPUT>() as i32);
    }
}

fn vk_for_char(c: char) -> Option<(u16, bool)> {
    let (base, shift) = match c {
        'a'..='z' => (VK_A + (c as u16 - 'a' as u16), false),
        'A'..='Z' => (VK_A + (c.to_ascii_lowercase() as u16 - 'a' as u16), true),
        '0' => (VK_0, false),
        '1'..='9' => (VK_1 + (c as u16 - '1' as u16), false),
        ' ' => (VK_SPACE, false),
        '\n' => (VK_RETURN, false),
        '.' => (VK_OEM_PERIOD, false),
        ',' => (VK_OEM_COMMA, false),
        '/' => (VK_OEM_2, false),
        '`' => (VK_OEM_3, false),
        '-' => (VK_OEM_MINUS, false),
        '=' => (VK_OEM_PLUS, false),
        '[' => (VK_OEM_4, false),
        ']' => (VK_OEM_6, false),
        '\\' => (VK_OEM_5, false),
        ';' => (VK_OEM_1, false),
        '\'' => (VK_OEM_7, false),
        '!' => (VK_1, true),
        '@' => (VK_2, true),
        '#' => (VK_3, true),
        '$' => (VK_4, true),
        '%' => (VK_5, true),
        '^' => (VK_6, true),
        '&' => (VK_7, true),
        '*' => (VK_8, true),
        '(' => (VK_9, true),
        ')' => (VK_0, true),
        '_' => (VK_OEM_MINUS, true),
        '+' => (VK_OEM_PLUS, true),
        '{' => (VK_OEM_4, true),
        '}' => (VK_OEM_6, true),
        '|' => (VK_OEM_5, true),
        ':' => (VK_OEM_1, true),
        '"' => (VK_OEM_7, true),
        '<' => (VK_OEM_COMMA, true),
        '>' => (VK_OEM_PERIOD, true),
        '?' => (VK_OEM_2, true),
        '~' => (VK_OEM_3, true),
        _ => return None,
    };
    Some((base, shift))
}
