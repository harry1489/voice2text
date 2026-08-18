use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

#[derive(Debug, Clone, Copy)]
pub enum KeyEvent {
    Down(u16),
    Up(u16),
}

pub fn spawn_listener(tx: Sender<KeyEvent>) {
    thread::spawn(move || {
        let mut prev_state: std::collections::HashMap<u16, bool> = std::collections::HashMap::new();
        loop {
            for vk in 0..=0xFF {
                let state = unsafe { GetAsyncKeyState(vk as i32) };
                let pressed = (state & 0x8000) != 0;
                let was_pressed = prev_state.get(&vk).copied().unwrap_or(false);

                if pressed && !was_pressed {
                    let _ = tx.send(KeyEvent::Down(vk));
                } else if !pressed && was_pressed {
                    let _ = tx.send(KeyEvent::Up(vk));
                }
                prev_state.insert(vk, pressed);
            }
            thread::sleep(Duration::from_millis(10));
        }
    });
}
