#[cfg(target_os = "linux")]
#[path = "hotkey.rs"]
mod hotkey;
#[cfg(target_os = "linux")]
#[path = "typeout.rs"]
mod typeout;
#[cfg(target_os = "linux")]
mod audio;
#[cfg(target_os = "linux")]
mod stt;

#[cfg(target_os = "windows")]
#[path = "hotkey_windows.rs"]
mod hotkey;
#[cfg(target_os = "windows")]
#[path = "typeout_windows.rs"]
mod typeout;
#[cfg(target_os = "windows")]
mod audio;
#[cfg(target_os = "windows")]
mod stt;

use std::path::PathBuf;
use std::sync::mpsc;

const DEFAULT_MODEL: &str = "/home/harry/copilot/models/ggml-small.en.bin";
const DEFAULT_TRIGGER: u16 = 193;

#[cfg(test)]
mod tests {
    #[test]
    fn trigger_is_f23() {
        assert_eq!(super::DEFAULT_TRIGGER, 193);
    }
}

fn main() {
    let model_path = std::env::var("V2T_MODEL")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(DEFAULT_MODEL));

    let transcriber = match stt::Transcriber::new(&model_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };

    let (work_tx, work_rx) = mpsc::channel::<Vec<f32>>();
    std::thread::spawn(move || {
        for samples in work_rx {
            let started = std::time::Instant::now();
            match transcriber.transcribe(&samples) {
                Ok(text) if !text.is_empty() => {
                    eprintln!("[stt] ({:.1}s) {text}", started.elapsed().as_secs_f32());
                    typeout::type_text(&text);
                }
                Ok(_) => eprintln!("[stt] no speech detected"),
                Err(e) => eprintln!("[stt] error: {e}"),
            }
        }
    });

    let trigger = parse_trigger();
    eprintln!(
        "[main] voice2text ready. Hold the trigger key (0x{trigger:02x}) to dictate."
    );

    let (key_tx, key_rx) = mpsc::channel::<hotkey::KeyEvent>();
    hotkey::spawn_listener(key_tx);

    let mut recording = false;

    for ev in key_rx {
        match ev {
            #[cfg(target_os = "linux")]
            hotkey::KeyEvent::Down(code) if code.0 == trigger => {
                if !recording {
                    recording = true;
                    match audio::start_recording() {
                        Ok(()) => eprintln!("[audio] recording..."),
                        Err(e) => {
                            eprintln!("[audio] failed to start recording: {e}");
                            recording = false;
                        }
                    }
                }
            }
            #[cfg(target_os = "linux")]
            hotkey::KeyEvent::Up(code) if code.0 == trigger => {
                if !recording {
                    continue;
                }
                recording = false;
                eprintln!("[audio] stopped");
                let samples = audio::stop_recording();
                if samples.len() < (16000 / 2) {
                    eprintln!("[audio] recording too short, skipping");
                } else if work_tx.send(samples).is_err() {
                    eprintln!("[worker] channel closed");
                }
            }
            #[cfg(target_os = "windows")]
            hotkey::KeyEvent::Down(code) if code == trigger => {
                if !recording {
                    recording = true;
                    match audio::start_recording() {
                        Ok(()) => eprintln!("[audio] recording..."),
                        Err(e) => {
                            eprintln!("[audio] failed to start recording: {e}");
                            recording = false;
                        }
                    }
                }
            }
            #[cfg(target_os = "windows")]
            hotkey::KeyEvent::Up(code) if code == trigger => {
                if !recording {
                    continue;
                }
                recording = false;
                eprintln!("[audio] stopped");
                let samples = audio::stop_recording();
                if samples.len() < (16000 / 2) {
                    eprintln!("[audio] recording too short, skipping");
                } else if work_tx.send(samples).is_err() {
                    eprintln!("[worker] channel closed");
                }
            }
            _ => continue,
        }
    }
}

fn parse_trigger() -> u16 {
    std::env::var("V2T_TRIGGER")
        .ok()
        .and_then(|v| u16::from_str_radix(v.trim_start_matches("0x"), 16).ok())
        .unwrap_or(DEFAULT_TRIGGER)
}
