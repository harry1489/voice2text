#!/usr/bin/env bash
set -euo pipefail

MODEL="${V2T_MODEL:-ggml-base.en.bin}"
DIR="$(dirname "$(realpath "$0")")"
DEST="$DIR/models/$MODEL"
URL="https://huggingface.co/ggerganov/whisper.cpp/resolve/main/$MODEL"

mkdir -p "$(dirname "$DEST")"

if [ -f "$DEST" ]; then
    echo "model already present: $DEST"
    exit 0
fi

echo "downloading $URL"
curl -L --fail --progress-bar -o "$DEST" "$URL"
echo "saved: $DEST"
