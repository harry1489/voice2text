use std::thread;
use std::time::Duration;

use evdev::{AttributeSet, KeyCode, KeyEvent};
use evdev::uinput::VirtualDevice;

fn main() {
    let pre_sleep_s: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(4);
    let hold_ms: u64 = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(5_000);

    let mut keys = AttributeSet::new();
    for code in 1..=255 {
        keys.insert(KeyCode::new(code));
    }

    let mut device: VirtualDevice = VirtualDevice::builder()
        .expect("open /dev/uinput")
        .name("v2t-test-kb")
        .with_keys(&keys)
        .expect("declare keys")
        .build()
        .expect("build device");

    eprintln!("[send_keys] device ready; waiting {pre_sleep_s}s for discovery");
    thread::sleep(Duration::from_secs(pre_sleep_s));
    eprintln!("[send_keys] holding Win+Shift+F23 for {hold_ms} ms");
    let key = |device: &mut VirtualDevice, code: KeyCode, value: i32| {
        device
            .emit(&[KeyEvent::new(code, value).into()])
            .unwrap();
    };

    key(&mut device, KeyCode::KEY_LEFTMETA, 1);
    key(&mut device, KeyCode::KEY_LEFTSHIFT, 1);
    key(&mut device, KeyCode::KEY_F23, 1);
    thread::sleep(Duration::from_millis(hold_ms));
    key(&mut device, KeyCode::KEY_F23, 0);
    key(&mut device, KeyCode::KEY_LEFTSHIFT, 0);
    key(&mut device, KeyCode::KEY_LEFTMETA, 0);
    eprintln!("[send_keys] done");
}
