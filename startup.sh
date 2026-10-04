#!/bin/sh
set -eu
cd "$(dirname "$0")"
if curl -sf -o /dev/null --max-time 2 http://127.0.0.1:8080/; then
  exit 0
fi
export PATH="$PATH:$HOME/.cargo/bin"
mkdir -p .blacksite
if ! node scripts/check-wasm-sync.mjs >/dev/null 2>&1; then
  npm run build:wasm
fi
nohup setsid npm run dev >>.blacksite/dev.log 2>&1 </dev/null &
