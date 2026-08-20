use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use evdev::{Device, EventType, KeyCode};

#[derive(Debug, Clone, Copy)]
pub enum KeyEvent {
    Down(KeyCode),
    Up(KeyCode),
}

pub fn spawn_listener(tx: Sender<KeyEvent>) {
    let registry: Arc<Mutex<std::collections::HashSet<String>>> =
        Arc::new(Mutex::new(std::collections::HashSet::new()));

    thread::spawn(move || loop {
        scan(&tx, &registry);
        thread::sleep(Duration::from_secs(3));
    });
}

fn scan(tx: &Sender<KeyEvent>, registry: &Arc<Mutex<std::collections::HashSet<String>>>) {
    let dir = Path::new("/dev/input");
    let Ok(entries) = fs::read_dir(dir) else {
        eprintln!("[hotkey] cannot read {}", dir.display());
        return;
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with("event") {
            continue;
        }
        let full: PathBuf = entry.path();

        let mut reg = registry.lock().unwrap();
        if reg.contains(&name) {
            continue;
        }

        let Ok(device) = Device::open(&full) else {
            continue;
        };

        let is_keyboard = device
            .supported_keys()
            .map(|keys| keys.contains(KeyCode::KEY_A))
            .unwrap_or(false);
        if !is_keyboard {
            continue;
        }

        reg.insert(name);
        drop(reg);

        let tx = tx.clone();
        thread::spawn(move || read_events(device, tx));
    }
}

fn read_events(mut device: Device, tx: Sender<KeyEvent>) {
    loop {
        match device.fetch_events() {
            Ok(events) => {
                for event in events {
                    if event.event_type() != EventType::KEY {
                        continue;
                    }
                    let code = KeyCode::new(event.code());
                    match event.value() {
                        1 => {
                            if tx.send(KeyEvent::Down(code)).is_err() {
                                return;
                            }
                        }
                        0 => {
                            if tx.send(KeyEvent::Up(code)).is_err() {
                                return;
                            }
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                eprintln!("[hotkey] device error: {e}");
                return;
            }
        }
    }
}
