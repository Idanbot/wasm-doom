#!/bin/sh
set -eu
cd "$(dirname "$0")"
if curl -sf -o /dev/null --max-time 2 http://127.0.0.1:8080/; then
  exit 0
fi
export PATH="$PATH:$HOME/.cargo/bin"
WASM=public/hellscan.wasm
SRC=engine/src/lib.rs
if [ ! -f "$WASM" ] || [ -f "$SRC" -a "$SRC" -nt "$WASM" ]; then
  (cd engine && cargo build --target wasm32-unknown-unknown --release)
  cp engine/target/wasm32-unknown-unknown/release/hellscan.wasm "$WASM"
fi
nohup npm run dev >>.grok/dev.log 2>&1 </dev/null &
