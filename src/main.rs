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

const DEFAULT_MODEL: &str = "ggml-small.en.bin";
const DEFAULT_TRIGGER: u16 = 193;

fn config_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
    PathBuf::from(home).join(".config").join("voice2text")
}

fn models_dir() -> PathBuf {
    let cfg = read_config();
    cfg.get("model_dir")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
            PathBuf::from(home).join(".local").join("share").join("voice2text").join("models")
        })
}

fn read_config() -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    let path = config_dir().join("config");
    if let Ok(content) = std::fs::read_to_string(&path) {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                map.insert(key.trim().to_string(), value.trim().to_string());
            }
        }
    }
    map
}

#[cfg(test)]
mod tests {
    #[test]
    fn trigger_is_f23() {
        assert_eq!(super::DEFAULT_TRIGGER, 193);
    }
}

fn main() {
    let cfg = read_config();
    let model_name = cfg.get("model").map(|s| s.as_str()).unwrap_or(DEFAULT_MODEL);
    let model_path = std::env::var("V2T_MODEL")
        .map(PathBuf::from)
        .unwrap_or_else(|_| models_dir().join(model_name));

    let transcriber = match stt::Transcriber::new(&model_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{e}");
            eprintln!("Run v2t-config to download a model.");
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
    if let Ok(v) = std::env::var("V2T_TRIGGER") {
        if let Ok(code) = u16::from_str_radix(v.trim_start_matches("0x"), 16) {
            return code;
        }
    }
    let cfg = read_config();
    if let Some(v) = cfg.get("trigger") {
        if let Ok(code) = u16::from_str_radix(v.trim_start_matches("0x"), 16) {
            return code;
        }
    }
    DEFAULT_TRIGGER
}
