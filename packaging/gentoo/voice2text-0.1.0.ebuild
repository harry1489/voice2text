# Copyright 2025
# Distributed under the terms of the MIT License

EAPI=8

CRATES=""

inherit cargo

DESCRIPTION="Linux voice-to-text dictation tool using Whisper"
HOMEPAGE="https://github.com/harry1489/voice2text"
SRC_URI="https://github.com/harry1489/voice2text/archive/refs/tags/v${PV}.tar.gz -> ${P}.tar.gz"

LICENSE="MIT"
SLOT="0"
KEYWORDS="~amd64"

DEPEND="
	media-libs/alsa-lib
"
RDEPEND="${DEPEND}"
BDEPEND="
	dev-util/cmake
	sys-devel/clang
	virtual/pkgconfig
"

src_install() {
	cargo_src_install

	dobin "${WORKDIR}/target/release/voice2text"

	insinto "/usr/share/${PN}"
	doins install.sh
	keepdir "/usr/share/${PN}/models"

	einstalldocs
}
