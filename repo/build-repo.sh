#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="$(dirname "$0")"
PKG_DIR="$REPO_DIR/voice2text"

echo "=== Building voice2text package ==="
cd "$REPO_DIR/.."
makepkg -s --noconfirm --clean

echo "=== Setting up repo ==="
mkdir -p "$REPO_DIR"
cp *.pkg.tar.zst "$REPO_DIR/" 2>/dev/null || true

cd "$REPO_DIR"
repo-add voice2text.db.tar.gz *.pkg.tar.zst

echo "=== Done ==="
echo "Upload the contents of $REPO_DIR to your server:"
echo "  scp *.pkg.tar.zst *.db* user@server:/srv/repo/voice2text/"
echo ""
echo "Or serve locally:"
echo "  python3 -m http.server 8080 --directory $REPO_DIR"
