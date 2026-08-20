# voice2text

Voice-to-text dictation tool for Linux and Windows. Hold a hotkey, speak, and your words are transcribed locally using [Whisper](https://github.com/ggerganov/whisper.cpp) and typed into the active window.

- **Offline** — all inference runs locally via whisper.cpp
- **No cloud APIs** — your audio never leaves your machine
- **Cross-platform** — Linux (uinput/ydotool/wtype) and Windows (SendInput)
- **Multiple text injection methods** with automatic fallback
- **Optimized for old hardware** — 2 threads, small model default

## Quick install

### Arch Linux

Add to `/etc/pacman.conf`:

```ini
[voice2text]
Server = https://repo.notahomelab.com/voice2text
SigLevel = Optional TrustAll
```

```bash
sudo pacman -Syu voice2text
```

### Debian / Ubuntu

```bash
echo "deb [trusted=yes] https://repo.notahomelab.com/debian ./" | sudo tee /etc/apt/sources.list.d/voice2text.list
sudo apt update && sudo apt install voice2text
```

### Fedora

```bash
sudo dnf config-manager addrepo --from-repofile=https://repo.notahomelab.com/fedora/voice2text.repo
sudo dnf install voice2text
```

Or manually:

```bash
sudo dnf install https://repo.notahomelab.com/fedora/voice2text-0.3.0-1.x86_64.rpm
```

### NixOS

```bash
nix run https://repo.notahomelab.com/nixos/voice2text.nix
```

Or add to your `flake.nix`:

```nix
{
  inputs.voice2text.url = "https://repo.notahomelab.com/nixos/voice2text.nix";
  # ...
}
```

### Gentoo

```bash
sudo eselect repository add voice2text-overlay https://repo.notahomelab.com/gentoo
sudo emaint sync -r voice2text-overlay
sudo emerge --ask app-misc/voice2text
```

### From source (any distro)

```bash
git clone https://github.com/harry1489/voice2text.git
cd voice2text
cargo build --release
sudo cp target/release/voice2text target/release/v2t-config /usr/bin/
```

## Configuration

Run the config tool to select a model and change the trigger key:

```bash
v2t-config
```

This creates `~/.config/voice2text/config`:

```ini
model = ggml-base.en.bin
trigger = 0xc1
model_dir = /home/you/.local/share/voice2text/models
```

### Available models

| Model | Size | RAM | Best for |
|-------|------|-----|----------|
| `ggml-tiny.en.bin` | ~39 MB | ~500 MB | Oldest hardware |
| `ggml-base.en.bin` | ~142 MB | ~1 GB | Default, good balance |
| `ggml-small.en.bin` | ~461 MB | ~2 GB | Better accuracy |
| `ggml-medium.en.bin` | ~1.5 GB | ~4 GB | High accuracy |
| `ggml-large-v3.bin` | ~3.1 GB | ~6 GB | Best accuracy |

### Trigger keys

Default: F23 (0xc1) — typically the Copilot button on keyboards.

Change with `v2t-config` or set the environment variable:

```bash
V2T_TRIGGER=0x7f voice2text  # F24
V2T_TRIGGER=0xb3 voice2text  # F19
V2T_TRIGGER=0x3a voice2text  # Caps Lock
```

## Permissions (Linux only)

For uinput text injection (recommended), add your user to the `input` and `uinput` groups:

```bash
sudo usermod -aG input,uinput $USER
```

Log out and back in for changes to take effect.

## Run

```bash
voice2text
```

Hold the trigger key and speak. Release to transcribe.

### Run as a service (Linux)

```bash
sudo systemctl enable --now voice2text
```

Check status: `sudo systemctl status voice2text`
View logs: `journalctl -u voice2text -f`

## Text injection

### Linux (fallback order)

1. **uinput** — virtual keyboard device (requires `uinput` group)
2. **ydotool** — requires `ydotoold` daemon running
3. **wtype** — Wayland only
4. **wl-copy** — copies to clipboard as last resort
5. **stderr** — prints text if nothing else works

### Windows

Uses the `SendInput` API directly — no external tools needed.

## License

MIT
