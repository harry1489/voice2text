# voice2text

Voice-to-text dictation tool for Linux and Windows. Hold a hotkey, speak, and your words are transcribed locally using [Whisper](https://github.com/ggerganov/whisper.cpp) and typed into the active window.

- **Offline** — all inference runs locally via whisper.cpp
- **No cloud APIs** — your audio never leaves your machine
- **Cross-platform** — Linux (uinput/ydotool/wtype) and Windows (SendInput)
- **Multiple text injection methods** with automatic fallback

## Platform support

| Platform | Text injection | Hotkey method |
|----------|---------------|---------------|
| Linux | uinput, ydotool, wtype, wl-copy | evdev (`/dev/input`) |
| Windows | SendInput API | `GetAsyncKeyState` polling |

## Dependencies

### Linux

| Dependency | Purpose |
|------------|---------|
| `alsa-lib` | Audio capture |
| `cmake` | Building whisper.cpp |
| `clang` | Generating FFI bindings |
| `pkg-config` | Finding system libraries |

Optional: `ydotool`, `wtype`, `wl-clipboard` (text injection fallbacks)

### Windows

- [Rust](https://rustup.rs/) toolchain
- Visual Studio Build Tools (C++ workload)
- CMake (usually bundled with vcpkg or install separately)

## Installation

### Windows

```powershell
# Build from source:
git clone https://github.com/harry1489/voice2text.git
cd voice2text
cargo build --release
copy target\release\voice2text.exe C:\Users\YourName\AppData\Local\Microsoft\WindowsApps\
```

Or add `target\release\` to your `PATH`.

#### Building the MSI installer

1. Install [WiX Toolset](https://wixtoolset.org/) v3.14+ and [cargo-wix](https://github.com/volks73/cargo-wix):
   ```powershell
   cargo install cargo-wix
   ```

2. Build the MSI:
   ```powershell
   cargo wix
   ```

3. The installer will be at `target\wix\voice2text-0.1.0-x64.msi`.

The MSI installer:
- Installs to `Program Files\voice2text\`
- Adds a Start Menu shortcut
- Optionally adds a Desktop shortcut
- Supports silent install: `voice2text-0.1.0-x64.msi /quiet /norestart`

### Arch Linux (AUR)

```bash
# Using an AUR helper like yay/paru:
yay -S voice2text

# Or manually:
git clone https://aur.archlinux.org/voice2text.git
cd voice2text
makepkg -si
```

#### Setting up your own pacman repo

1. Build the package:
   ```bash
   git clone https://github.com/harry1489/voice2text.git
   cd voice2text
   makepkg -s --noconfirm
   ```

2. Set up a local repo:
   ```bash
   sudo mkdir -p /srv/repo
   cp *.pkg.tar.zst /srv/repo/
   cd /srv/repo
   repo-add voice2text.db.tar.gz *.pkg.tar.zst
   ```

3. Add to `/etc/pacman.conf`:
   ```
   [voice2text]
   Server = file:///srv/repo
   ```

4. Install:
   ```bash
   sudo pacman -Syu voice2text
   ```

### Debian / Ubuntu

```bash
git clone https://github.com/harry1489/voice2text.git
cd voice2text
dpkg-buildpackage -us -uc
sudo dpkg -i ../voice2text_0.1.0-1_amd64.deb
sudo apt-get install -f
```

#### Setting up your own apt repo

1. Build the package:
   ```bash
   git clone https://github.com/harry1489/voice2text.git
   cd voice2text
   dpkg-buildpackage -us -uc
   ```

2. Set up the repo:
   ```bash
   sudo mkdir -p /srv/repo
   cp ../voice2text_*.deb /srv/repo/
   cd /srv/repo
   dpkg-scanpackages . /dev/null | gzip -9c > Packages.gz
   ```

3. Add to `/etc/apt/sources.list.d/voice2text.list`:
   ```
   deb [trusted=yes] file:///srv/repo ./
   ```

4. Install:
   ```bash
   sudo apt update
   sudo apt install voice2text
   ```

### Fedora

```bash
git clone https://github.com/harry1489/voice2text.git
cd voice2text
rpmbuild -bb packaging/fedora/voice2text.spec
sudo rpm -i ~/rpmbuild/RPMS/x86_64/voice2text-0.1.0-1.fc*.x86_64.rpm
```

### Gentoo

```bash
git clone https://github.com/harry1489/voice2text.git
cd voice2text
sudo cp packaging/gentoo/voice2text-0.1.0.ebuild /var/db/repos/local/sys-apps/voice2text/
sudo emerge --ask voice2text
```

### NixOS

```bash
git clone https://github.com/harry1489/voice2text.git
cd voice2text

# Build and run directly:
nix-build packaging/nix/default.nix

# Or add to your configuration.nix:
#   nixpkgs.overlays = [
#     (self: super: {
#       voice2text = self.callPackage ./path/to/voice2text/packaging/nix/default.nix {};
#     })
#   ];
#   environment.systemPackages = [ pkgs.voice2text ];
```

### From source (any distro)

```bash
git clone https://github.com/harry1489/voice2text.git
cd voice2text
cargo build --release
# Linux:
sudo cp target/release/voice2text /usr/bin/
# Windows: copy target\release\voice2text.exe to somewhere in PATH
```

## Setup

### 1. Download a Whisper model

**Linux:**
```bash
./install.sh
```

**Windows** (PowerShell):
```powershell
Invoke-WebRequest -Uri "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin" -OutFile "models\ggml-base.en.bin"
```

This downloads `ggml-base.en.bin` (~140MB) to `models/`. For better accuracy, set `V2T_MODEL=ggml-small.en.bin`.

Available models: `ggml-tiny.en.bin`, `ggml-base.en.bin`, `ggml-small.en.bin`, `ggml-medium.en.bin`, `ggml-large-v3.bin`

### 2. Permissions (Linux only)

For uinput text injection (recommended), add your user to the `input` and `uinput` groups:

```bash
sudo usermod -aG input,uinput $USER
```

Log out and back in for changes to take effect.

### 3. Run

```bash
voice2text
```

Hold the trigger key and speak. Release to transcribe.

### 4. Run as a service (Linux)

Install and enable the systemd service:

```bash
sudo cp voice2text.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now voice2text
```

Check status:
```bash
sudo systemctl status voice2text
```

View logs:
```bash
journalctl -u voice2text -f
```

Stop/restart:
```bash
sudo systemctl stop voice2text
sudo systemctl restart voice2text
```

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `V2T_MODEL` | `ggml-small.en.bin` | Path to Whisper model file |
| `V2T_TRIGGER` | `0xc1` (Linux) / `0xc1` (Windows) | Hex key code for the trigger key |

Example:

```bash
V2T_MODEL=./models/ggml-base.en.bin V2T_TRIGGER=0x3e voice2text
```

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
