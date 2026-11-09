#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$script_dir/.." && pwd)"

cd "$script_dir"

for tool in cargo wasm-bindgen node npm; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "error: '$tool' is required but was not found in PATH" >&2
    exit 1
  fi
done

echo "==> Building the Rust runtime (wasm32-unknown-unknown)"
cd "$root"
cargo +nightly --config web/.cargo/config.toml build -p webrogue-web-runtime --target wasm32-unknown-unknown --release

echo "==> Generating wasm-bindgen bindings"
wasm-bindgen \
  --target web \
  --out-dir web/frontend/generated/runtime \
  --out-name webrogue_web_runtime \
  target/wasm32-unknown-unknown/release/webrogue_web_runtime.wasm

echo "==> Building the frontend"
cd "$script_dir/frontend"
if [ ! -d node_modules ]; then
  npm install
fi
npm run build

echo "==> Serving on http://localhost:${PORT:-8080}/web/frontend/"
exec node serve.mjs
