#!/usr/bin/env bash
set -euo pipefail

NAME="bakery-experiment"
TARGET="wasm32-unknown-unknown"
PROFILE="release"

echo "Building for $TARGET..."
cargo build --profile "$PROFILE" --target "$TARGET"

echo "Generating JS bindings with wasm-bindgen..."
wasm-bindgen \
  --out-dir web \
  --target web \
  "target/$TARGET/$PROFILE/$NAME.wasm"

echo "Optimizing WASM with wasm-opt..."
wasm-opt -Oz -o web/"$NAME"_bg.wasm web/"$NAME"_bg.wasm

echo "Done. Output in web/ (serve with any HTTP server)"
echo "  e.g. python -m http.server 8080 -d ."
