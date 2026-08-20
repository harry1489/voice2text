use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

const MODELS: &[(&str, &str, &str)] = &[
    ("1", "ggml-tiny.en.bin", "~39 MB - Fastest, lowest accuracy"),
    ("2", "ggml-base.en.bin", "~142 MB - Good balance"),
    ("3", "ggml-small.en.bin", "~461 MB - Recommended"),
    ("4", "ggml-medium.en.bin", "~1.5 GB - High accuracy"),
    ("5", "ggml-large-v3.bin", "~3.1 GB - Best accuracy"),
];

const KEYS: &[(&str, &str)] = &[
    ("1", "F23 (0xc1) - Copilot button"),
    ("2", "F24 (0x7f) - Alternative"),
    ("3", "F19 (0xb3)"),
    ("4", "F20 (0xb4)"),
    ("5", "Left Ctrl (0x1d)"),
    ("6", "Caps Lock (0x3a)"),
    ("7", "Custom - enter hex code"),
];

fn config_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
    PathBuf::from(home).join(".config").join("voice2text")
}

fn config_file() -> PathBuf {
    config_dir().join("config")
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
    let path = config_file();
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

fn write_config(key: &str, value: &str) {
    let dir = config_dir();
    std::fs::create_dir_all(&dir).ok();
    let path = config_file();
    let mut map = read_config();
    map.insert(key.to_string(), value.to_string());
    let mut content = String::from("# voice2text config\n");
    for (k, v) in &map {
        content.push_str(&format!("{k} = {v}\n"));
    }
    std::fs::write(&path, content).expect("failed to write config");
}

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
}

fn pause() {
    print!("\nPress Enter to continue...");
    io::stdout().flush().ok();
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).ok();
}

fn print_header() {
    println!("╔══════════════════════════════════════╗");
    println!("║       voice2text configuration       ║");
    println!("╚══════════════════════════════════════╝");
    println!();
}

fn show_status() {
    let cfg = read_config();
    let model = cfg.get("model").map(|s| s.as_str()).unwrap_or("ggml-small.en.bin");
    let trigger = cfg.get("trigger").map(|s| s.as_str()).unwrap_or("0xc1");
    let models_dir = models_dir();
    let model_path = models_dir.join(model);
    let installed = model_path.exists();
    let size = if installed {
        let bytes = std::fs::metadata(&model_path).map(|m| m.len()).unwrap_or(0);
        format!(" ({:.1} MB)", bytes as f64 / 1_048_576.0)
    } else {
        " (not downloaded)".to_string()
    };

    println!("  Current model:   {}{}", model, size);
    println!("  Model directory: {}", models_dir.display());
    println!("  Trigger key:     F{} (code {})", trigger_key_name(trigger), trigger);
    println!("  Status:          {}", if installed { "✓ Ready" } else { "✗ Model missing" });
    println!();
}

fn trigger_key_name(hex: &str) -> String {
    match hex {
        "0xc1" => "23".into(),
        "0x7f" => "24".into(),
        "0xb3" => "19".into(),
        "0xb4" => "20".into(),
        "0x1d" => "Ctrl".into(),
        "0x3a" => "CapsLock".into(),
        _ => format!("(0x{})", hex),
    }
}

fn list_models() {
    clear_screen();
    print_header();
    show_status();

    println!("  Available models:");
    println!("  ─────────────────────────────────────────────");
    for (num, name, desc) in MODELS {
        let models_dir = models_dir();
        let installed = models_dir.join(name).exists();
        let mark = if installed { "✓" } else { " " };
        println!("  [{}] {} {} - {}", mark, num, name, desc);
    }
    println!("  [0] Back");
    println!();
    print!("  Select model to download: ");
    io::stdout().flush().ok();

    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    let choice = input.trim();

    if choice == "0" {
        return;
    }

    let model = MODELS.iter().find(|(n, _, _)| *n == choice);
    if let Some((_, name, _)) = model {
        download_model(name);
    } else {
        println!("\n  Invalid choice.");
        pause();
    }
}

fn download_model(name: &str) {
    let models_dir = models_dir();
    std::fs::create_dir_all(&models_dir).ok();
    let dest = models_dir.join(name);

    if dest.exists() {
        println!("\n  {} is already downloaded.", name);
        pause();
        return;
    }

    let url = format!("https://huggingface.co/ggerganov/whisper.cpp/resolve/main/{}", name);
    println!("\n  Downloading {}...", name);
    println!("  URL: {}", url);
    println!();

    let status = Command::new("curl")
        .args(["-L", "--fail", "--progress-bar", "-o", dest.to_str().unwrap(), &url])
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("\n  ✓ Downloaded successfully!");
            write_config("model", name);
        }
        _ => {
            println!("\n  ✗ Download failed. Make sure curl is installed.");
            std::fs::remove_file(&dest).ok();
        }
    }
    pause();
}

fn set_trigger() {
    clear_screen();
    print_header();

    let cfg = read_config();
    let current = cfg.get("trigger").map(|s| s.as_str()).unwrap_or("0xc1");
    println!("  Current trigger: F{} (code {})", trigger_key_name(current), current);
    println!();
    println!("  Select new trigger key:");
    println!("  ─────────────────────────────────────────────");
    for (num, desc) in KEYS {
        println!("  [{}] {}", num, desc);
    }
    println!("  [0] Back");
    println!();
    print!("  Choice: ");
    io::stdout().flush().ok();

    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    let choice = input.trim();

    if choice == "0" {
        return;
    }

    let hex = match choice {
        "1" => "0xc1",
        "2" => "0x7f",
        "3" => "0xb3",
        "4" => "0xb4",
        "5" => "0x1d",
        "6" => "0x3a",
        "7" => {
            print!("\n  Enter hex code (e.g. 0xb5): ");
            io::stdout().flush().ok();
            let mut hex_input = String::new();
            io::stdin().read_line(&mut hex_input).ok();
            let h = hex_input.trim().to_string();
            if !h.starts_with("0x") {
                println!("  Invalid hex code. Must start with 0x");
                pause();
                return;
            }
            Box::leak(h.into_boxed_str())
        }
        _ => {
            println!("\n  Invalid choice.");
            pause();
            return;
        }
    };

    write_config("trigger", hex);
    println!("\n  ✓ Trigger key updated to 0x{}", hex);
    println!("  Restart voice2text for changes to take effect.");
    pause();
}

fn set_model_dir() {
    clear_screen();
    print_header();

    let current = models_dir();
    println!("  Current model directory: {}", current.display());
    println!();
    print!("  Enter new path (or press Enter to keep): ");
    io::stdout().flush().ok();

    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    let path = input.trim();

    if !path.is_empty() {
        let p = PathBuf::from(path);
        std::fs::create_dir_all(&p).ok();
        write_config("model_dir", path);
        println!("\n  ✓ Model directory updated to {}", path);
    }
    pause();
}

fn edit_config() {
    clear_screen();
    print_header();

    let cfg = read_config();
    println!("  Current configuration:");
    println!("  ─────────────────────────────────────────────");
    for (k, v) in &cfg {
        println!("  {} = {}", k, v);
    }
    println!();
    println!("  Config file: {}", config_file().display());
    println!();
    println!("  [1] Edit config file");
    println!("  [2] Reset to defaults");
    println!("  [0] Back");
    println!();
    print!("  Choice: ");
    io::stdout().flush().ok();

    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();

    match input.trim() {
        "1" => {
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".into());
            Command::new(&editor)
                .arg(config_file())
                .status()
                .ok();
        }
        "2" => {
            std::fs::remove_file(config_file()).ok();
            println!("\n  ✓ Config reset to defaults.");
            pause();
        }
        _ => {}
    }
}

fn main() {
    loop {
        clear_screen();
        print_header();
        show_status();

        println!("  [1] Download / manage models");
        println!("  [2] Change trigger key");
        println!("  [3] Change model directory");
        println!("  [4] Edit config");
        println!("  [0] Quit");
        println!();
        print!("  Choice: ");
        io::stdout().flush().ok();

        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();

        match input.trim() {
            "1" => list_models(),
            "2" => set_trigger(),
            "3" => set_model_dir(),
            "4" => edit_config(),
            "0" | "q" => break,
            _ => {}
        }
    }
}
