# Maintainer: harry1489 <harry1489@users.noreply.github.com>
pkgname=voice2text
pkgver=0.1.0
pkgrel=1
pkgdesc="Linux voice-to-text dictation tool using Whisper"
arch=('x86_64')
url="https://github.com/harry1489/voice2text"
license=('MIT')
depends=('alsa-lib' 'gcc-libs' 'glibc')
makedepends=('cargo' 'cmake' 'clang' 'pkg-config')
optdepends=(
	'ydotool: fallback text injection on Wayland/X11'
	'wtype: fallback text injection on Wayland'
	'wl-clipboard: fallback clipboard copy on Wayland'
)
source=("$pkgname-$pkgver.tar.gz::https://github.com/harry1489/voice2text/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('SKIP')

prepare() {
	export RUSTUP_TOOLCHAIN=stable
	cargo fetch --locked --target "$(rustc -vV | sed -n 's|host: ||p')"
}

build() {
	export RUSTUP_TOOLCHAIN=stable
	export CARGO_TARGET_DIR=target
	cargo build --frozen --release
}

check() {
	export RUSTUP_TOOLCHAIN=stable
	cargo test --frozen
}

package() {
	install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
	install -Dm755 "install.sh" "$pkgdir/usr/share/$pkgname/install.sh"
	install -d "$pkgdir/usr/share/$pkgname/models"
}
