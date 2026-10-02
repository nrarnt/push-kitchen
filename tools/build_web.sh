#!/bin/sh
# Builds the browser version of the game into web/dist: a folder of plain
# files that any web host can serve.
#
# Needs, once:
#   rustup target add wasm32-unknown-unknown
#   cargo install wasm-bindgen-cli --version <the wasm-bindgen version in Cargo.lock> --locked
#
# Then, from anywhere:  tools/build_web.sh
# And to try it:        python3 -m http.server --directory web/dist 8000
#                       (open http://localhost:8000)
set -e
cd "$(dirname "$0")/.."

# 1. Compile the game to WebAssembly.
cargo build --profile web --target wasm32-unknown-unknown

# 2. Start from an empty output folder.
rm -rf web/dist
mkdir -p web/dist/assets

# 3. Generate the JavaScript that loads the WebAssembly file into a page.
wasm-bindgen --target web --no-typescript \
    --out-dir web/dist --out-name push-kitchen \
    target/wasm32-unknown-unknown/web/push-kitchen.wasm

# 4. Shrink the WebAssembly file further, if wasm-opt is installed
#    (brew install binaryen). The game works without this step.
if command -v wasm-opt >/dev/null; then
    wasm-opt -Os --output web/dist/push-kitchen_bg.wasm web/dist/push-kitchen_bg.wasm
fi

# 5. Add the page and the files the game fetches while running. The levels
#    are baked into the program, so they are not needed here.
cp web/index.html web/dist/
cp -R assets/sprites assets/sounds web/dist/assets/

echo "Built web/dist:"
du -sh web/dist
