#!/bin/bash

set -ex

cd "$(dirname "$BASH_SOURCE")/.."
cargo run --release
WASM=./submodules/wayfolio/wasm
wasm-pack \
  build \
  --release \
  --out-name wasm \
  --target web \
  --no-typescript \
  --no-pack \
  $WASM
cp $WASM/pkg/wasm* page
